//! Failure injection exercises real files while keeping external intent fixed.
//! These boundaries do not simulate power loss or certify filesystem firmware.
#![cfg(target_os = "linux")]

use super::super::AppendBoundary;
use super::*;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../../../tests/support_v2/mod.rs"]
mod fixture;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mirror-recovery-fault-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
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

fn request() -> AppendRecoveryRequestV2 {
    AppendRecoveryRequestV2::new(
        AppendCheckpointV2 {
            generation: 0,
            chain: fixture::main_expectation(1),
        },
        AppendCheckpointV2 {
            generation: 1,
            chain: fixture::main_expectation(2),
        },
        fixture::main_chain(2).blocks()[2].clone(),
    )
    .unwrap()
}

fn failure() -> AppendRecoveryErrorV2 {
    io::Error::other("injected explicit recovery interruption").into()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, here: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(here).unwrap() {
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

fn stage(root: &Path, shape: u8) {
    let request = request();
    let child = request.block.encode(fixture::manifest().config()).unwrap();
    let tail = root.join(BLOCKS_NAME).join(block_name(2));
    match shape {
        0 => {}
        1 => fs::write(tail, &child[..child.len() / 2]).unwrap(),
        2 => {
            fs::write(tail, child).unwrap();
            let selector = AppendSelectionV2::from_chain(&fixture::main_chain(2), 1)
                .unwrap()
                .encode();
            fs::write(root.join(PENDING_NAME), &selector[..71]).unwrap();
        }
        _ => unreachable!(),
    }
}

fn prefix_boundaries() -> Vec<RecoveryBoundary> {
    vec![
        RecoveryBoundary::Validated,
        RecoveryBoundary::ParentManifestSynced,
        RecoveryBoundary::ParentBlockSynced(0),
        RecoveryBoundary::ParentBlockSynced(1),
        RecoveryBoundary::ParentBlocksDirectorySynced,
        RecoveryBoundary::SelectedMarkerSynced,
        RecoveryBoundary::ParentRootSynced,
    ]
}

fn clean_boundaries(height: u64) -> Vec<RecoveryBoundary> {
    let mut result = vec![
        RecoveryBoundary::CleanValidated,
        RecoveryBoundary::CleanSync(ReconcileBoundary::ManifestSynced),
    ];
    result.extend(
        (0..=height).map(|h| RecoveryBoundary::CleanSync(ReconcileBoundary::BlockSynced(h))),
    );
    result.extend([
        RecoveryBoundary::CleanSync(ReconcileBoundary::BlocksDirectorySynced),
        RecoveryBoundary::CleanSync(ReconcileBoundary::SelectionSynced),
        RecoveryBoundary::CleanSync(ReconcileBoundary::RootSynced),
        RecoveryBoundary::CleanSync(ReconcileBoundary::SelectionRechecked),
    ]);
    result
}

fn complete_boundaries() -> Vec<RecoveryBoundary> {
    let mut result = prefix_boundaries();
    result.extend([
        RecoveryBoundary::BeforeTailWrite,
        RecoveryBoundary::TailOpened,
        RecoveryBoundary::TailWritten,
        RecoveryBoundary::TailSynced,
        RecoveryBoundary::TailDirectorySynced,
        RecoveryBoundary::BeforePendingWrite,
        RecoveryBoundary::PendingOpened,
        RecoveryBoundary::PendingWritten,
        RecoveryBoundary::PendingSynced,
        RecoveryBoundary::PublicationAttempt,
        RecoveryBoundary::SelectionPublished,
        RecoveryBoundary::RootSynced,
    ]);
    result.extend(clean_boundaries(2));
    result
}

fn assert_parent_unchanged(root: &Path, original: &BTreeMap<PathBuf, Vec<u8>>) {
    for (path, bytes) in original {
        if path != Path::new(SELECTION_NAME) {
            assert_eq!(&fs::read(root.join(path)).unwrap(), bytes, "{path:?}");
        }
    }
}

#[test]
fn completion_retries_after_each_boundary_from_absent_partial_and_pending_shapes() {
    let request = request();
    let boundaries = complete_boundaries();
    assert_eq!(boundaries.len(), 28);
    for shape in 0..=2 {
        for (index, target) in boundaries.iter().enumerate() {
            let scratch = Scratch::new();
            let root = scratch.root();
            drop(AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap());
            let original = snapshot(&root);
            stage(&root, shape);
            let mut reached = false;
            let error = recover_open_inner(
                &root,
                fixture::fixture_identity(),
                &request,
                AppendRecoveryActionV2::Complete,
                |at| {
                    if at == *target {
                        reached = true;
                        Err(failure())
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap_err();
            assert!(reached, "{shape} {target:?}");
            let may_publish = index
                >= boundaries
                    .iter()
                    .position(|at| *at == RecoveryBoundary::PublicationAttempt)
                    .unwrap();
            assert_eq!(
                matches!(error, AppendRecoveryErrorV2::PublicationUncertain { .. }),
                may_publish,
                "{shape} {target:?}: {error}"
            );
            assert_parent_unchanged(&root, &original);
            let mut recovered = AppendStoreV2::recover_interrupted(
                &root,
                fixture::fixture_identity(),
                &request,
                AppendRecoveryActionV2::Complete,
            )
            .unwrap();
            assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(2));
            assert_eq!(recovered.checkpoint().unwrap(), request.candidate);
            recovered
                .append(fixture::main_chain(3).blocks()[3].clone())
                .unwrap();
            assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(3));
        }
    }
}

#[test]
fn retain_parent_retries_after_each_cleanup_and_resynchronization_boundary() {
    let request = request();
    let mut boundaries = prefix_boundaries();
    boundaries.extend([
        RecoveryBoundary::BeforePendingRemoval,
        RecoveryBoundary::PendingRemoved,
        RecoveryBoundary::PendingRemovalSynced,
        RecoveryBoundary::BeforeTailRemoval,
        RecoveryBoundary::TailRemoved,
        RecoveryBoundary::TailRemovalSynced,
    ]);
    boundaries.extend(clean_boundaries(1));
    assert_eq!(boundaries.len(), 21);
    for target in boundaries {
        let scratch = Scratch::new();
        let root = scratch.root();
        drop(AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap());
        let original = snapshot(&root);
        stage(&root, 2);
        let mut reached = false;
        let error = recover_open_inner(
            &root,
            fixture::fixture_identity(),
            &request,
            AppendRecoveryActionV2::RetainParent,
            |at| {
                if at == target {
                    reached = true;
                    Err(failure())
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert!(reached, "{target:?}");
        assert!(
            matches!(error, AppendRecoveryErrorV2::RecoveryIncomplete { .. }),
            "{target:?}: {error}"
        );
        assert_parent_unchanged(&root, &original);
        let mut recovered = AppendStoreV2::recover_interrupted(
            &root,
            fixture::fixture_identity(),
            &request,
            AppendRecoveryActionV2::RetainParent,
        )
        .unwrap();
        assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(1));
        assert_eq!(recovered.checkpoint().unwrap(), request.parent);
        assert_eq!(snapshot(&root), original);
        recovered.append(request.block.clone()).unwrap();
        assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(2));
    }
}

#[test]
fn already_selected_candidate_sync_failures_retry_without_another_generation() {
    let request = request();
    let mut boundaries = prefix_boundaries();
    boundaries.extend(clean_boundaries(2));
    assert_eq!(boundaries.len(), 16);
    for target in boundaries {
        let scratch = Scratch::new();
        let root = scratch.root();
        let mut writer = AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap();
        writer.append(request.block.clone()).unwrap();
        let original = snapshot(&root);
        let mut reached = false;
        let error = writer
            .recover_inner(&request, AppendRecoveryActionV2::Complete, |at| {
                if at == target {
                    reached = true;
                    Err(failure())
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert!(reached, "{target:?}");
        assert!(matches!(
            error,
            AppendRecoveryErrorV2::PublicationUncertain { .. }
        ));
        assert_eq!(snapshot(&root), original);
        let recovered = AppendStoreV2::recover_interrupted(
            &root,
            fixture::fixture_identity(),
            &request,
            AppendRecoveryActionV2::Complete,
        )
        .unwrap();
        assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(2));
        assert_eq!(recovered.checkpoint().unwrap(), request.candidate);
        assert_eq!(snapshot(&root), original);
    }
}

#[test]
fn every_original_append_failure_recovers_without_releasing_live_ownership() {
    let request = request();
    for target in [
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
        for action in [
            AppendRecoveryActionV2::Complete,
            AppendRecoveryActionV2::RetainParent,
        ] {
            let scratch = Scratch::new();
            let root = scratch.root();
            let mut writer = AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap();
            writer
                .append_inner(request.block.clone(), |at| {
                    if at == target {
                        Err(io::Error::other("original append interrupted").into())
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err();
            assert!(writer.is_fenced());
            assert_eq!(writer.chain, fixture::main_chain(1));
            let published = matches!(
                target,
                AppendBoundary::SelectionPublished | AppendBoundary::RootSynced
            );
            let result = writer.recover_inner(&request, action, |_| {
                for access in [StoreAccessV2::Writer, StoreAccessV2::ReadOnly] {
                    assert!(matches!(
                        DirectoryLockV2::acquire(&root, access),
                        Err(StorageErrorV2::Busy)
                    ));
                }
                Ok(())
            });
            if action == AppendRecoveryActionV2::RetainParent && published {
                assert!(matches!(
                    result,
                    Err(AppendRecoveryErrorV2::CandidateAlreadySelected)
                ));
                assert!(
                    AppendStoreV2::open_writer(
                        &root,
                        fixture::fixture_identity(),
                        request.candidate
                    )
                    .is_ok()
                );
            } else {
                let recovered = result.unwrap();
                assert!(!recovered.is_fenced());
                let height = if action == AppendRecoveryActionV2::Complete {
                    2
                } else {
                    1
                };
                assert_eq!(recovered.chain().unwrap(), &fixture::main_chain(height));
            }
        }
    }
}

#[test]
fn changed_root_marker_or_unknown_layout_at_validation_is_not_mutated() {
    for change in 0..3 {
        let scratch = Scratch::new();
        let root = scratch.root();
        drop(AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap());
        stage(&root, 1);
        let original = snapshot(&root);
        let moved = scratch.0.join("retained-original-root");
        let mut changed = None;
        recover_open_inner(
            &root,
            fixture::fixture_identity(),
            &request(),
            AppendRecoveryActionV2::Complete,
            |at| {
                if at == RecoveryBoundary::Validated {
                    match change {
                        0 => {
                            fs::rename(&root, &moved).unwrap();
                            fs::create_dir(&root).unwrap();
                        }
                        1 => {
                            let path = root.join(SELECTION_NAME);
                            let mut bytes = fs::read(&path).unwrap();
                            bytes[0] ^= 1;
                            fs::write(path, bytes).unwrap();
                            changed = Some(snapshot(&root));
                        }
                        2 => {
                            fs::write(root.join("unexplained"), b"preserve me").unwrap();
                            changed = Some(snapshot(&root));
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            },
        )
        .unwrap_err();
        if change == 0 {
            assert_eq!(snapshot(&moved), original);
            assert!(snapshot(&root).is_empty());
        } else {
            assert_eq!(snapshot(&root), changed.unwrap());
        }
    }
}

#[test]
fn changed_prefixes_are_never_overwritten_or_deleted_after_validation() {
    for (action, boundary, name) in [
        (
            AppendRecoveryActionV2::Complete,
            RecoveryBoundary::BeforeTailWrite,
            block_name(2),
        ),
        (
            AppendRecoveryActionV2::Complete,
            RecoveryBoundary::BeforePendingWrite,
            PENDING_NAME.into(),
        ),
        (
            AppendRecoveryActionV2::RetainParent,
            RecoveryBoundary::BeforePendingRemoval,
            PENDING_NAME.into(),
        ),
        (
            AppendRecoveryActionV2::RetainParent,
            RecoveryBoundary::BeforeTailRemoval,
            block_name(2),
        ),
    ] {
        let scratch = Scratch::new();
        let root = scratch.root();
        drop(AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap());
        stage(&root, 2);
        let selected = fs::read(root.join(SELECTION_NAME)).unwrap();
        let path = if name == PENDING_NAME {
            root.join(name)
        } else {
            root.join(BLOCKS_NAME).join(name)
        };
        let mut modified = None;
        recover_open_inner(
            &root,
            fixture::fixture_identity(),
            &request(),
            action,
            |at| {
                if at == boundary {
                    let mut bytes = fs::read(&path).unwrap();
                    bytes[0] ^= 1;
                    fs::write(&path, &bytes).unwrap();
                    modified = Some(bytes);
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(fs::read(&path).unwrap(), modified.unwrap());
        assert_eq!(fs::read(root.join(SELECTION_NAME)).unwrap(), selected);
    }
}

#[test]
fn postwrite_or_prepublication_changes_cannot_be_selected() {
    for boundary in [
        RecoveryBoundary::TailWritten,
        RecoveryBoundary::PendingWritten,
        RecoveryBoundary::PendingSynced,
        RecoveryBoundary::PublicationAttempt,
    ] {
        let scratch = Scratch::new();
        let root = scratch.root();
        drop(AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap());
        let selected = fs::read(root.join(SELECTION_NAME)).unwrap();
        let path = if matches!(
            boundary,
            RecoveryBoundary::PendingWritten | RecoveryBoundary::PublicationAttempt
        ) {
            root.join(PENDING_NAME)
        } else {
            root.join(BLOCKS_NAME).join(block_name(2))
        };
        let mut modified = None;
        recover_open_inner(
            &root,
            fixture::fixture_identity(),
            &request(),
            AppendRecoveryActionV2::Complete,
            |at| {
                if at == boundary {
                    let mut bytes = fs::read(&path).unwrap();
                    bytes[0] ^= 1;
                    fs::write(&path, &bytes).unwrap();
                    modified = Some(bytes);
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(fs::read(&path).unwrap(), modified.unwrap());
        assert_eq!(fs::read(root.join(SELECTION_NAME)).unwrap(), selected);
    }
}

#[test]
fn descriptor_alias_created_after_open_blocks_prefix_completion() {
    use std::os::unix::fs::MetadataExt;
    let scratch = Scratch::new();
    let root = scratch.root();
    drop(AppendStoreV2::initialize(&root, &fixture::main_chain(1)).unwrap());
    stage(&root, 1);
    let tail = root.join(BLOCKS_NAME).join(block_name(2));
    let original = snapshot(&root);
    let alias = scratch.0.join("alias");
    recover_open_inner(
        &root,
        fixture::fixture_identity(),
        &request(),
        AppendRecoveryActionV2::Complete,
        |at| {
            if at == RecoveryBoundary::TailOpened {
                fs::hard_link(&tail, &alias).unwrap();
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(fs::metadata(&tail).unwrap().nlink(), 2);
    assert_eq!(snapshot(&root), original);
    assert_eq!(fs::read(alias).unwrap(), fs::read(tail).unwrap());
}

#[test]
fn recovery_read_allowance_charges_sentinels_before_touching_payload() {
    assert_eq!(MAX_APPEND_RECOVERY_INPUT_BYTES_V2, 19_559_412);
    assert_eq!(MAX_APPEND_RECOVERY_READ_BYTES_V2, 156_508_096);
    let scratch = Scratch::new();
    let path = scratch.0.join("bytes");
    fs::write(&path, b"abc").unwrap();
    let mut budget = ReadBudget { remaining: 3 };
    assert!(matches!(
        budget.read(&path, 3, Some(3)),
        Err(AppendRecoveryErrorV2::ReadBudgetExceeded)
    ));
    assert_eq!(budget.remaining, 3);
    let mut budget = ReadBudget { remaining: 4 };
    assert_eq!(budget.read(&path, 3, Some(3)).unwrap(), b"abc");
    assert_eq!(budget.remaining, 0);
    assert!(matches!(
        budget.read(&scratch.0.join("absent"), 0, None),
        Err(AppendRecoveryErrorV2::ReadBudgetExceeded)
    ));
    assert_eq!(fs::read(path).unwrap(), b"abc");
}
