//! Deterministic publication, cleanup and clean-reconciliation failure checks.
#![cfg(target_os = "linux")]

use super::*;
use std::{
    collections::BTreeMap,
    os::unix::fs::MetadataExt,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../../tests/support_v2/mod.rs"]
mod fixture;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mirror-replacement-fault-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn root(&self) -> PathBuf {
        self.0.join("store")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn failure() -> ReplacementStorageErrorV2 {
    io::Error::other("injected replacement boundary failure").into()
}

fn checkpoint(chain: &ChainV2, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}

// Include directory entries so empty staging is not silently ignored by a test.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(base: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let relative = path.strip_prefix(base).unwrap().to_path_buf();
        if metadata.is_dir() {
            out.insert(relative, None);
            for entry in fs::read_dir(path).unwrap() {
                visit(base, &entry.unwrap().path(), out);
            }
        } else {
            out.insert(relative, Some(fs::read(path).unwrap()));
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

fn assert_busy(root: &Path, expected: ReplacementCheckpointV2) {
    assert!(matches!(
        ReplacementStoreV2::open_writer(root, fixture::fixture_identity(), expected),
        Err(ReplacementStorageErrorV2::Storage(StorageErrorV2::Busy))
    ));
    assert!(matches!(
        ReplacementStoreV2::inspect(root, fixture::fixture_identity(), expected),
        Err(ReplacementStorageErrorV2::Storage(StorageErrorV2::Busy))
    ));
}

fn assert_fenced(store: &ReplacementStoreV2, old: ReplacementCheckpointV2) {
    assert!(store.is_fenced());
    assert_eq!(store.last_confirmed_checkpoint(), old);
    assert!(matches!(
        store.chain(),
        Err(ReplacementStorageErrorV2::Fenced)
    ));
    assert!(matches!(
        store.selection(),
        Err(ReplacementStorageErrorV2::Fenced)
    ));
    assert!(matches!(
        store.checkpoint(),
        Err(ReplacementStorageErrorV2::Fenced)
    ));
}

#[test]
fn every_replacement_boundary_fences_without_exposing_mixed_state() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let old_checkpoint = checkpoint(&old, 0);
    let next_checkpoint = checkpoint(&candidate, 1);
    let sample = Scratch::new();
    let mut successful = ReplacementStoreV2::initialize(sample.root(), &old).unwrap();
    let mut trace = Vec::new();
    successful
        .replace_inner(
            fixture::fixture_identity(),
            old_checkpoint,
            &candidate,
            |at| {
                trace.push(at);
                Ok(())
            },
        )
        .unwrap();
    let published = trace
        .iter()
        .position(|at| *at == ReplacementBoundary::PublicationAttempt)
        .unwrap();
    assert!(trace.contains(&ReplacementBoundary::OldGenerationDirectoryRemoved));
    assert!(trace.contains(&ReplacementBoundary::CleanupRootSynced));
    drop(successful);

    for (index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
        let inode = fs::metadata(&root).unwrap();
        let old_files = snapshot(&root);
        let mut observed = 0;
        let error = writer
            .replace_inner(
                fixture::fixture_identity(),
                old_checkpoint,
                &candidate,
                |at| {
                    assert_eq!(at, trace[observed]);
                    let fail = observed == index;
                    observed += 1;
                    if fail { Err(failure()) } else { Ok(()) }
                },
            )
            .unwrap_err();
        assert_eq!(observed, index + 1, "{boundary:?}");
        match error {
            ReplacementStorageErrorV2::UpdateIncomplete { candidate, .. } => {
                assert!(index < published, "{boundary:?}");
                assert_eq!(candidate, next_checkpoint);
            }
            ReplacementStorageErrorV2::PublicationUncertain { candidate, .. } => {
                assert!(index >= published, "{boundary:?}");
                assert_eq!(candidate, next_checkpoint);
            }
            other => panic!("{boundary:?}: {other:?}"),
        }
        assert_fenced(&writer, old_checkpoint);
        assert_eq!(
            &writer.chain, &old,
            "{boundary:?}: in-memory history advanced"
        );
        assert_busy(&root, old_checkpoint);
        let retained = snapshot(&root);
        assert!(matches!(
            writer.replace(fixture::fixture_identity(), old_checkpoint, &candidate),
            Err(ReplacementStorageErrorV2::Fenced)
        ));
        assert_eq!(snapshot(&root), retained);
        let after = fs::metadata(&root).unwrap();
        assert_eq!((inode.dev(), inode.ino()), (after.dev(), after.ino()));
        drop(writer);

        // Only a completely clean old/new selection may reopen. Every success
        // is compared with the ENTIRE accepted chain, not just account balances.
        let old_result =
            ReplacementStoreV2::open_writer(&root, fixture::fixture_identity(), old_checkpoint);
        let old_ok = match old_result {
            Ok(store) => {
                assert_eq!(store.chain().unwrap(), &old);
                true
            }
            Err(_) => false,
        };
        let new_result =
            ReplacementStoreV2::open_writer(&root, fixture::fixture_identity(), next_checkpoint);
        let new_ok = match new_result {
            Ok(store) => {
                assert_eq!(store.chain().unwrap(), &candidate);
                true
            }
            Err(_) => false,
        };
        assert!(
            !(old_ok && new_ok),
            "two checkpoints reopened at {boundary:?}"
        );
        assert_eq!(old_ok, retained == old_files, "{boundary:?}");
        let cleanup_done = trace
            .iter()
            .position(|at| *at == ReplacementBoundary::OldGenerationDirectoryRemoved)
            .unwrap();
        assert_eq!(new_ok, index >= cleanup_done, "{boundary:?}");
        assert_eq!(
            snapshot(&root),
            retained,
            "opening repaired leftovers at {boundary:?}"
        );
    }
}

#[test]
fn every_initialization_boundary_preserves_partial_data_and_returns_no_writer() {
    let candidate = fixture::main_chain(3);
    let expected = checkpoint(&candidate, 0);
    let sample = Scratch::new();
    let mut trace = Vec::new();
    drop(
        initialize_inner(&sample.root(), &candidate, |at| {
            trace.push(at);
            Ok(())
        })
        .unwrap(),
    );
    assert!(trace.contains(&ReplacementBoundary::SelectionPublished));
    for (index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        let mut observed = 0;
        let result = initialize_inner(&root, &candidate, |at| {
            assert_eq!(at, trace[observed]);
            let fail = observed == index;
            observed += 1;
            if fail { Err(failure()) } else { Ok(()) }
        });
        assert!(result.is_err(), "{boundary:?}");
        assert_eq!(observed, index + 1);
        let retained = snapshot(&root);
        let reopened =
            ReplacementStoreV2::open_writer(&root, fixture::fixture_identity(), expected);
        let selected_at = trace
            .iter()
            .position(|at| *at == ReplacementBoundary::SelectionPublished)
            .unwrap();
        if index >= selected_at {
            assert_eq!(
                reopened.unwrap().chain().unwrap(),
                &candidate,
                "{boundary:?}"
            );
        } else {
            assert!(reopened.is_err(), "{boundary:?}");
        }
        assert_eq!(snapshot(&root), retained, "{boundary:?}");
    }
}

#[test]
fn every_clean_reconciliation_boundary_retains_ownership_and_never_returns_failed_handle() {
    let selected = fixture::main_chain(3);
    let expected = checkpoint(&selected, 0);
    let sample = Scratch::new();
    let writer = ReplacementStoreV2::initialize(sample.root(), &selected).unwrap();
    let mut trace = Vec::new();
    drop(
        writer
            .reconcile_inner(expected, |at| {
                trace.push(at);
                Ok(())
            })
            .unwrap(),
    );
    assert!(trace.contains(&ReplacementBoundary::SelectedRootSynced));
    for (index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        let writer = ReplacementStoreV2::initialize(&root, &selected).unwrap();
        let before = snapshot(&root);
        let inode = fs::metadata(&root).unwrap();
        let mut observed = 0;
        let result = writer.reconcile_inner(expected, |at| {
            assert_eq!(at, trace[observed]);
            assert_busy(&root, expected);
            let fail = observed == index;
            observed += 1;
            if fail { Err(failure()) } else { Ok(()) }
        });
        assert!(
            matches!(
                result,
                Err(ReplacementStorageErrorV2::ReconciliationFailed(_))
            ),
            "{boundary:?}"
        );
        assert_eq!(observed, index + 1);
        let reopened =
            ReplacementStoreV2::open_writer(&root, fixture::fixture_identity(), expected).unwrap();
        assert_eq!(reopened.chain().unwrap(), &selected);
        assert_eq!(snapshot(&root), before);
        let after = fs::metadata(&root).unwrap();
        assert_eq!((inode.dev(), inode.ino()), (after.dev(), after.ino()));
    }
}

#[test]
fn successful_replacement_keeps_root_exclusively_owned_at_every_boundary() {
    let old = fixture::fork_chain();
    let next = fixture::main_chain(4);
    let expected = checkpoint(&old, 0);
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let inode = fs::metadata(&root).unwrap();
    writer
        .replace_inner(fixture::fixture_identity(), expected, &next, |_| {
            assert_busy(&root, expected);
            let now = fs::metadata(&root).unwrap();
            assert_eq!((inode.dev(), inode.ino()), (now.dev(), now.ino()));
            Ok(())
        })
        .unwrap();
    assert_eq!(writer.chain().unwrap(), &next);
}

#[test]
fn partial_candidate_block_is_preserved_and_cannot_be_silently_reconciled() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let expected = checkpoint(&old, 0);
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let block = candidate.blocks()[0].encode(candidate.config()).unwrap();
    let path = root.join("generation-00000000000000000001/blocks/00000000000000000000.mrb2");
    let error = writer
        .replace_inner(fixture::fixture_identity(), expected, &candidate, |at| {
            if at == ReplacementBoundary::BlockCreated(0) {
                fs::write(&path, &block[..31]).unwrap();
                Err(failure())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert!(matches!(
        error,
        ReplacementStorageErrorV2::UpdateIncomplete { .. }
    ));
    assert_fenced(&writer, expected);
    assert_eq!(fs::read(&path).unwrap(), &block[..31]);
    let retained = snapshot(&root);
    assert!(writer.reconcile(expected).is_err());
    assert!(ReplacementStoreV2::open_writer(&root, fixture::fixture_identity(), expected).is_err());
    assert!(
        ReplacementStoreV2::open_writer(
            &root,
            fixture::fixture_identity(),
            checkpoint(&candidate, 1)
        )
        .is_err()
    );
    assert_eq!(snapshot(&root), retained);
}

#[test]
fn leftover_old_history_after_publication_cannot_roll_selection_back() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let expected = checkpoint(&old, 0);
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let error = writer
        .replace_inner(fixture::fixture_identity(), expected, &candidate, |at| {
            if at == ReplacementBoundary::OldBlockRemoved(0) {
                Err(failure())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert!(matches!(
        error,
        ReplacementStorageErrorV2::PublicationUncertain { .. }
    ));
    assert_fenced(&writer, expected);
    let selected = fs::read(root.join("selection.v2")).unwrap();
    assert_eq!(
        selected,
        ReplacementSelectionV2::from_chain(&candidate, 1)
            .unwrap()
            .encode()
    );
    let retained = snapshot(&root);
    assert!(writer.reconcile(expected).is_err());
    assert!(
        ReplacementStoreV2::open_writer(
            &root,
            fixture::fixture_identity(),
            checkpoint(&candidate, 1)
        )
        .is_err()
    );
    assert_eq!(snapshot(&root), retained);
    assert_eq!(fs::read(root.join("selection.v2")).unwrap(), selected);
}

#[test]
fn missing_current_manifest_fences_before_any_candidate_artifact_is_created() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    fs::remove_file(root.join("generation-00000000000000000000/manifest.v2")).unwrap();
    let retained = snapshot(&root);
    assert!(
        writer
            .replace(fixture::fixture_identity(), checkpoint(&old, 0), &candidate)
            .is_err()
    );
    assert_fenced(&writer, checkpoint(&old, 0));
    assert!(!root.join("generation-00000000000000000001").exists());
    assert_eq!(snapshot(&root), retained);
}

#[test]
fn failed_after_final_sync_can_reconcile_only_the_complete_new_chain() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    writer
        .replace_inner(
            fixture::fixture_identity(),
            checkpoint(&old, 0),
            &candidate,
            |at| {
                if at == ReplacementBoundary::CleanupRootSynced {
                    Err(failure())
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
    assert_fenced(&writer, checkpoint(&old, 0));
    let reopened = writer.reconcile(checkpoint(&candidate, 1)).unwrap();
    assert_eq!(reopened.chain().unwrap(), &candidate);
    assert!(!reopened.is_fenced());
    assert_eq!(reopened.checkpoint().unwrap(), checkpoint(&candidate, 1));
}

#[test]
fn candidate_corruption_before_publish_preserves_the_old_selection_and_data() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let old_marker = fs::read(root.join("selection.v2")).unwrap();
    let old_dir = root.join("generation-00000000000000000000");
    let old_files = snapshot(&old_dir);
    let path = root.join("generation-00000000000000000001/blocks/00000000000000000004.mrb2");
    let result = writer.replace_inner(
        fixture::fixture_identity(),
        checkpoint(&old, 0),
        &candidate,
        |at| {
            if at == ReplacementBoundary::StagingRootSynced {
                let mut bytes = fs::read(&path).unwrap();
                let last = bytes.len() - 1;
                bytes[last] ^= 1;
                fs::write(&path, bytes).unwrap();
            }
            Ok(())
        },
    );
    assert!(matches!(
        result,
        Err(ReplacementStorageErrorV2::UpdateIncomplete { .. })
    ));
    assert_fenced(&writer, checkpoint(&old, 0));
    assert_eq!(fs::read(root.join("selection.v2")).unwrap(), old_marker);
    assert_eq!(snapshot(&old_dir), old_files);
}

#[test]
fn unexpected_old_file_after_publication_prevents_any_cleanup_deletion() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let old_dir = root.join("generation-00000000000000000000");
    let before = snapshot(&old_dir);
    let foreign = old_dir.join("blocks/do-not-delete");
    let result = writer.replace_inner(
        fixture::fixture_identity(),
        checkpoint(&old, 0),
        &candidate,
        |at| {
            if at == ReplacementBoundary::PublishedRootSynced {
                fs::write(&foreign, b"foreign evidence").unwrap();
            }
            Ok(())
        },
    );
    assert!(matches!(
        result,
        Err(ReplacementStorageErrorV2::PublicationUncertain { .. })
    ));
    assert_fenced(&writer, checkpoint(&old, 0));
    let after = snapshot(&old_dir);
    for (path, bytes) in before {
        assert_eq!(after.get(&path), Some(&bytes));
    }
    assert_eq!(fs::read(foreign).unwrap(), b"foreign evidence");
}

#[test]
fn selected_candidate_corruption_after_publish_prevents_old_history_cleanup() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let old_dir = root.join("generation-00000000000000000000");
    let before = snapshot(&old_dir);
    let path = root.join("generation-00000000000000000001/blocks/00000000000000000004.mrb2");
    let result = writer.replace_inner(
        fixture::fixture_identity(),
        checkpoint(&old, 0),
        &candidate,
        |at| {
            if at == ReplacementBoundary::PublishedRootSynced {
                let mut bytes = fs::read(&path).unwrap();
                bytes[0] ^= 1;
                fs::write(&path, bytes).unwrap();
            }
            Ok(())
        },
    );
    assert!(matches!(
        result,
        Err(ReplacementStorageErrorV2::PublicationUncertain { .. })
    ));
    assert_fenced(&writer, checkpoint(&old, 0));
    assert_eq!(snapshot(&old_dir), before);
}

#[test]
fn all_old_bytes_are_checked_before_any_earlier_old_file_is_deleted() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    for relative in ["blocks/00000000000000000003.mrb2", "manifest.v2"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
        let old_dir = root.join("generation-00000000000000000000");
        let path = old_dir.join(relative);
        let mut changed = None;
        let result = writer.replace_inner(
            fixture::fixture_identity(),
            checkpoint(&old, 0),
            &candidate,
            |at| {
                if at == ReplacementBoundary::PublishedRootSynced {
                    let mut bytes = fs::read(&path).unwrap();
                    bytes[0] ^= 1;
                    fs::write(&path, bytes).unwrap();
                    changed = Some(snapshot(&old_dir));
                }
                Ok(())
            },
        );
        assert!(
            matches!(
                result,
                Err(ReplacementStorageErrorV2::PublicationUncertain { .. })
            ),
            "{relative}"
        );
        assert_fenced(&writer, checkpoint(&old, 0));
        assert_eq!(
            snapshot(&old_dir),
            changed.unwrap(),
            "{relative}: preflight deleted valid earlier files"
        );
    }
}

#[test]
fn repeated_payload_reads_share_one_budget_and_charge_growth_sentinels() {
    let scratch = Scratch::new();
    let payload = scratch.0.join("payload");
    fs::write(&payload, b"data").unwrap();
    let mut budget = ReadBudget {
        used: MAX_REPLACEMENT_READ_BYTES_V2 - 10,
    };
    assert_eq!(budget.read_exact(&payload, 4).unwrap(), b"data");
    assert_eq!(budget.used, MAX_REPLACEMENT_READ_BYTES_V2 - 5);
    assert_eq!(budget.read_exact(&payload, 4).unwrap(), b"data");
    assert_eq!(budget.used, MAX_REPLACEMENT_READ_BYTES_V2);
    // A nonexistent path would give an I/O error if opened. The exhausted
    // allowance rejects first, including the sentinel for a zero-byte read.
    assert!(matches!(
        budget.read_exact(&scratch.0.join("absent"), 0),
        Err(ReplacementStorageErrorV2::ReadBudgetExceeded { .. })
    ));
    assert_eq!(budget.used, MAX_REPLACEMENT_READ_BYTES_V2);
    assert_eq!(fs::read(&payload).unwrap(), b"data");
}

#[test]
fn overflowing_or_oversized_read_allowance_rejects_before_payload_open() {
    let scratch = Scratch::new();
    let missing = scratch.0.join("absent");
    for length in [MAX_REPLACEMENT_READ_BYTES_V2, u64::MAX] {
        let mut budget = ReadBudget::new();
        assert!(matches!(
            budget.read_exact(&missing, length),
            Err(ReplacementStorageErrorV2::ReadBudgetExceeded { .. })
        ));
        assert_eq!(budget.used, 0);
        assert!(!missing.exists());
    }
    let mut budget = ReadBudget { used: 1 };
    assert!(matches!(
        budget.charge(u64::MAX),
        Err(ReplacementStorageErrorV2::ReadBudgetExceeded {
            attempted: u64::MAX,
            ..
        })
    ));
    assert_eq!(budget.used, 1);
}
