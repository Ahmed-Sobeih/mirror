//! Deterministic file/publication/resynchronization fault boundaries.
#![cfg(target_os = "linux")]

use super::*;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../../tests/support_v2/mod.rs"]
mod fixture;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let parent = std::env::temp_dir().join(format!(
            "mirror-append-fault-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&parent).unwrap();
        Self(parent)
    }

    fn store(&self) -> PathBuf {
        self.0.join("store")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn failure() -> AppendStorageErrorV2 {
    io::Error::other("injected storage boundary failure").into()
}

fn checkpoint(height: u64, generation: u64) -> AppendCheckpointV2 {
    AppendCheckpointV2 {
        generation,
        chain: fixture::main_expectation(height),
    }
}

fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn every_append_boundary_fences_and_reopen_returns_complete_old_new_or_rejection() {
    let old = fixture::main_chain(0);
    let next = fixture::main_chain(1);
    let child = next.blocks()[1].clone();
    for boundary in [
        AppendBoundary::BeforeBlockCreate,
        AppendBoundary::BlockCreated,
        AppendBoundary::BlockWritten,
        AppendBoundary::BlockSynced,
        AppendBoundary::BlocksDirectorySynced,
        AppendBoundary::SelectorCreated,
        AppendBoundary::SelectorWritten,
        AppendBoundary::SelectorSynced,
        AppendBoundary::PublicationAttempt,
        AppendBoundary::SelectionPublished,
        AppendBoundary::RootSynced,
    ] {
        let scratch = Scratch::new();
        let root = scratch.store();
        let mut writer = AppendStoreV2::initialize(&root, &old).unwrap();
        let error = writer
            .append_inner(child.clone(), |at| {
                if at == boundary {
                    Err(failure())
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        let expected_uncertainty = matches!(
            boundary,
            AppendBoundary::PublicationAttempt
                | AppendBoundary::SelectionPublished
                | AppendBoundary::RootSynced
        );
        match error {
            AppendStorageErrorV2::UpdateIncomplete { candidate, .. } => {
                assert!(!expected_uncertainty, "{boundary:?}");
                assert_eq!(candidate, checkpoint(1, 1));
            }
            AppendStorageErrorV2::PublicationUncertain { candidate, .. } => {
                assert!(expected_uncertainty, "{boundary:?}");
                assert_eq!(candidate, checkpoint(1, 1));
            }
            other => panic!("{boundary:?}: {other}"),
        }
        assert!(writer.is_fenced());
        assert_eq!(
            writer.chain, old,
            "{boundary:?}: memory advanced before success"
        );
        assert_eq!(writer.last_confirmed_checkpoint(), checkpoint(0, 0));
        assert!(matches!(writer.chain(), Err(AppendStorageErrorV2::Fenced)));
        assert!(matches!(
            writer.selection(),
            Err(AppendStorageErrorV2::Fenced)
        ));
        assert!(matches!(
            writer.checkpoint(),
            Err(AppendStorageErrorV2::Fenced)
        ));
        let retained = bytes(&root);
        assert!(matches!(
            writer.append(child.clone()),
            Err(AppendStorageErrorV2::Fenced)
        ));
        assert_eq!(bytes(&root), retained);
        drop(writer);

        let old_result =
            AppendStoreV2::open_writer(&root, fixture::fixture_identity(), checkpoint(0, 0));
        let old_success = match old_result {
            Ok(store) => {
                assert_eq!(store.chain().unwrap(), &old);
                true
            }
            Err(_) => false,
        };
        let new_result =
            AppendStoreV2::open_writer(&root, fixture::fixture_identity(), checkpoint(1, 1));
        let new_success = match new_result {
            Ok(store) => {
                assert_eq!(store.chain().unwrap(), &next);
                true
            }
            Err(_) => false,
        };
        assert_eq!(
            old_success,
            boundary == AppendBoundary::BeforeBlockCreate,
            "{boundary:?}"
        );
        assert_eq!(
            new_success,
            matches!(
                boundary,
                AppendBoundary::SelectionPublished | AppendBoundary::RootSynced
            ),
            "{boundary:?}"
        );
        assert_eq!(
            bytes(&root),
            retained,
            "{boundary:?}: reopen modified artifacts"
        );
    }
}

#[test]
fn consuming_reconciliation_retains_lock_and_allows_next_child_only_after_sync() {
    let scratch = Scratch::new();
    let root = scratch.store();
    let history = fixture::main_chain(2);
    let mut writer = AppendStoreV2::initialize(&root, &fixture::main_chain(0)).unwrap();
    assert!(matches!(
        writer.append_inner(history.blocks()[1].clone(), |at| {
            if at == AppendBoundary::SelectionPublished {
                Err(failure())
            } else {
                Ok(())
            }
        }),
        Err(AppendStorageErrorV2::PublicationUncertain { .. })
    ));
    assert!(writer.is_fenced());
    let mut visited = Vec::new();
    let mut recovered = writer
        .reconcile_inner(checkpoint(1, 1), |at| {
            // Ownership must never be released between the old and recovered handles.
            assert!(matches!(
                DirectoryLockV2::acquire(&root, StoreAccessV2::Writer),
                Err(StorageErrorV2::Busy)
            ));
            assert!(matches!(
                DirectoryLockV2::acquire(&root, StoreAccessV2::ReadOnly),
                Err(StorageErrorV2::Busy)
            ));
            visited.push(at);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        visited,
        vec![
            ReconcileBoundary::Validated,
            ReconcileBoundary::ManifestSynced,
            ReconcileBoundary::BlockSynced(0),
            ReconcileBoundary::BlockSynced(1),
            ReconcileBoundary::BlocksDirectorySynced,
            ReconcileBoundary::SelectionSynced,
            ReconcileBoundary::RootSynced,
            ReconcileBoundary::SelectionRechecked,
        ]
    );
    assert!(!recovered.is_fenced());
    assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(1));
    assert_eq!(
        recovered.append(history.blocks()[2].clone()).unwrap(),
        checkpoint(2, 2)
    );
    assert_eq!(recovered.chain().unwrap(), &history);
}

#[test]
fn stale_pin_cannot_reconcile_an_uncertain_new_selection_and_releases_ownership() {
    let scratch = Scratch::new();
    let root = scratch.store();
    let mut writer = AppendStoreV2::initialize(&root, &fixture::main_chain(0)).unwrap();
    writer
        .append_inner(fixture::main_chain(1).blocks()[1].clone(), |at| {
            if at == AppendBoundary::SelectionPublished {
                Err(failure())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    let retained = bytes(&root);
    assert!(matches!(
        writer.reconcile(checkpoint(0, 0)),
        Err(AppendStorageErrorV2::ReconciliationFailed(_))
    ));
    assert_eq!(bytes(&root), retained);
    let recovered =
        AppendStoreV2::open_writer(&root, fixture::fixture_identity(), checkpoint(1, 1)).unwrap();
    assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(1));
}

#[test]
fn every_reconciliation_sync_boundary_returns_no_handle_on_error() {
    let chain = fixture::main_chain(1);
    let expected = checkpoint(1, 0);
    for boundary in [
        ReconcileBoundary::Validated,
        ReconcileBoundary::ManifestSynced,
        ReconcileBoundary::BlockSynced(0),
        ReconcileBoundary::BlockSynced(1),
        ReconcileBoundary::BlocksDirectorySynced,
        ReconcileBoundary::SelectionSynced,
        ReconcileBoundary::RootSynced,
        ReconcileBoundary::SelectionRechecked,
    ] {
        for consuming in [false, true] {
            let scratch = Scratch::new();
            let root = scratch.store();
            let writer = AppendStoreV2::initialize(&root, &chain).unwrap();
            let retained = bytes(&root);
            let inject = |at| {
                if at == boundary {
                    Err(failure())
                } else {
                    Ok(())
                }
            };
            let result = if consuming {
                writer.reconcile_inner(expected, inject)
            } else {
                drop(writer);
                open_inner(
                    &root,
                    fixture::fixture_identity(),
                    expected,
                    StoreAccessV2::Writer,
                    inject,
                )
            };
            assert!(
                matches!(result, Err(AppendStorageErrorV2::ReconciliationFailed(_))),
                "{boundary:?}"
            );
            assert_eq!(bytes(&root), retained);
            let recovered =
                AppendStoreV2::open_writer(&root, fixture::fixture_identity(), expected).unwrap();
            assert_eq!(recovered.chain().unwrap(), &chain);
        }
    }
}

#[test]
fn partial_block_and_partial_pending_selector_are_preserved_and_never_promoted() {
    let old = fixture::main_chain(0);
    let next = fixture::main_chain(1);
    let child = next.blocks()[1].clone();
    let block_bytes = child.encode(next.config()).unwrap();
    let selector_bytes = AppendSelectionV2::from_chain(&next, 1).unwrap().encode();
    for boundary in [
        AppendBoundary::BlockCreated,
        AppendBoundary::SelectorCreated,
    ] {
        let scratch = Scratch::new();
        let root = scratch.store();
        let mut writer = AppendStoreV2::initialize(&root, &old).unwrap();
        writer
            .append_inner(child.clone(), |at| {
                if at != boundary {
                    return Ok(());
                }
                if boundary == AppendBoundary::BlockCreated {
                    fs::write(
                        root.join(BLOCKS_NAME).join(block_name(1)),
                        &block_bytes[..block_bytes.len() / 2],
                    )?;
                } else {
                    fs::write(
                        root.join(PENDING_NAME),
                        &selector_bytes[..selector_bytes.len() / 2],
                    )?;
                }
                Err(failure())
            })
            .unwrap_err();
        let retained = bytes(&root);
        assert!(matches!(
            writer.reconcile(checkpoint(0, 0)),
            Err(AppendStorageErrorV2::ReconciliationFailed(_))
        ));
        assert!(
            AppendStoreV2::open_writer(&root, fixture::fixture_identity(), checkpoint(1, 1))
                .is_err()
        );
        assert_eq!(bytes(&root), retained);
        let selected = AppendSelectionV2::decode_for(
            &fs::read(root.join(SELECTION_NAME)).unwrap(),
            checkpoint(0, 0),
        )
        .unwrap();
        assert_eq!(selected.checkpoint(), checkpoint(0, 0));
    }
}

#[test]
fn selector_changed_during_preparation_is_not_overwritten_or_accepted() {
    let scratch = Scratch::new();
    let root = scratch.store();
    let old = fixture::main_chain(1);
    let changed = AppendSelectionV2::from_chain(&old, 1).unwrap().encode();
    let mut writer = AppendStoreV2::initialize(&root, &old).unwrap();
    let error = writer
        .append_inner(fixture::main_chain(2).blocks()[2].clone(), |at| {
            if at == AppendBoundary::SelectorSynced {
                fs::write(root.join(SELECTION_NAME), changed)?;
            }
            Ok(())
        })
        .unwrap_err();
    assert!(
        matches!(error, AppendStorageErrorV2::UpdateIncomplete { source, .. }
        if matches!(*source, AppendStorageErrorV2::SelectionChanged))
    );
    assert!(writer.is_fenced());
    assert_eq!(writer.chain, old);
    assert_eq!(fs::read(root.join(SELECTION_NAME)).unwrap(), changed);
    assert!(root.join(PENDING_NAME).is_file());
}

#[test]
fn replaced_root_is_detected_before_any_append_write() {
    let scratch = Scratch::new();
    let root = scratch.store();
    let moved = scratch.0.join("moved-store");
    let mut writer = AppendStoreV2::initialize(&root, &fixture::main_chain(0)).unwrap();
    let retained = bytes(&root);
    fs::rename(&root, &moved).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(matches!(
        writer.append(fixture::main_chain(1).blocks()[1].clone()),
        Err(AppendStorageErrorV2::Storage(StorageErrorV2::UnsafePath(_)))
    ));
    assert!(writer.is_fenced());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    assert_eq!(bytes(&moved), retained);
}

#[test]
fn profile_two_initialization_faults_preserve_partial_data_or_checked_complete_selection() {
    let chain = fixture::main_chain(1);
    let expected = checkpoint(1, 0);
    for boundary in [
        InitializeBoundary::ParentSynced,
        InitializeBoundary::ManifestSynced,
        InitializeBoundary::BlockSynced(0),
        InitializeBoundary::BlockSynced(1),
        InitializeBoundary::BlocksDirectorySynced,
        InitializeBoundary::RootPrepared,
        InitializeBoundary::SelectionStaged,
        InitializeBoundary::SelectionPublished,
        InitializeBoundary::RootSynced,
    ] {
        let scratch = Scratch::new();
        let root = scratch.store();
        let result = initialize_inner(&root, &chain, |at| {
            if at == boundary {
                Err(failure())
            } else {
                Ok(())
            }
        });
        let published = matches!(
            boundary,
            InitializeBoundary::SelectionPublished | InitializeBoundary::RootSynced
        );
        assert!(
            if published {
                matches!(
                    result,
                    Err(AppendStorageErrorV2::InitializationUncertain(_))
                )
            } else {
                matches!(
                    result,
                    Err(AppendStorageErrorV2::InitializationIncomplete(_))
                )
            },
            "{boundary:?}"
        );
        let retained = bytes(&root);
        let reopened = AppendStoreV2::open_writer(&root, fixture::fixture_identity(), expected);
        match reopened {
            Ok(store) => {
                assert!(published);
                assert_eq!(store.chain().unwrap(), &chain);
            }
            Err(_) => assert!(!published),
        }
        assert_eq!(bytes(&root), retained);
    }
}
