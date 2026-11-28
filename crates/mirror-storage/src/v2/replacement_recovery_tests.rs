//! Real replacement/recovery failure boundaries and bounded retry behavior.
#![cfg(target_os = "linux")]

use super::*;
use std::{
    collections::BTreeMap,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../../tests/support_v2/mod.rs"]
mod fixture;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "mirror-replacement-recovery-fault-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
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

fn request<'a>(old: &'a ChainV2, candidate: &'a ChainV2) -> ReplacementRecoveryRequestV2<'a> {
    ReplacementRecoveryRequestV2::new(
        ReplacementSelectionV2::from_chain(old, 0)
            .unwrap()
            .checkpoint(),
        ReplacementSelectionV2::from_chain(candidate, 1)
            .unwrap()
            .checkpoint(),
        old,
        candidate,
    )
    .unwrap()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        if fs::symlink_metadata(path).unwrap().is_dir() {
            out.insert(relative, None);
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), out);
            }
        } else {
            out.insert(relative, Some(fs::read(path).unwrap()));
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

fn failure() -> ReplacementRecoveryErrorV2 {
    io::Error::other("injected replacement recovery boundary failure").into()
}

fn contains_busy(error: &(dyn std::error::Error + 'static)) -> bool {
    matches!(
        error.downcast_ref::<StorageErrorV2>(),
        Some(StorageErrorV2::Busy)
    ) || error.source().is_some_and(contains_busy)
}

fn assert_owned(root: &Path, request: &ReplacementRecoveryRequestV2<'_>) {
    let error = ReplacementStoreV2::recover_interrupted(
        root,
        fixture::fixture_identity(),
        request,
        ReplacementRecoveryActionV2::Complete,
    )
    .unwrap_err();
    assert!(
        contains_busy(&error),
        "recovery lost its continuous root lock: {error:?}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    CleanOld,
    SparseCandidate,
    PendingCandidate,
    SelectedCandidate,
    SelectedCandidateManifestOnly,
}

fn seed(root: &Path, old: &ChainV2, candidate: &ChainV2, shape: Shape) {
    drop(ReplacementStoreV2::initialize(root, old).unwrap());
    if matches!(shape, Shape::CleanOld) {
        return;
    }
    let generation = root.join(generation_name(1));
    let blocks = generation.join(BLOCKS_NAME);
    fs::create_dir(&generation).unwrap();
    fs::create_dir(&blocks).unwrap();
    let encoded = EncodedGeneration::from_chain(candidate).unwrap();
    if matches!(shape, Shape::SparseCandidate) {
        // Directory-entry persistence is not constrained to a height prefix.
        // Include an empty block, a complete block and a later partial block,
        // while the candidate manifest entry is absent.
        fs::write(blocks.join(block_name(0)), []).unwrap();
        fs::write(blocks.join(block_name(2)), &encoded.blocks[2]).unwrap();
        fs::write(blocks.join(block_name(4)), &encoded.blocks[4][..71]).unwrap();
        return;
    }
    fs::write(generation.join(MANIFEST_NAME), &encoded.manifest).unwrap();
    for (height, bytes) in encoded.blocks.iter().enumerate() {
        fs::write(blocks.join(block_name(height as u64)), bytes).unwrap();
    }
    let next = ReplacementSelectionV2::from_chain(candidate, 1)
        .unwrap()
        .encode();
    if matches!(shape, Shape::PendingCandidate) {
        fs::write(root.join(PENDING_NAME), &next[..83]).unwrap();
    } else {
        fs::write(root.join(SELECTION_NAME), next).unwrap();
        let old_blocks = root.join(generation_name(0)).join(BLOCKS_NAME);
        fs::remove_file(old_blocks.join(block_name(1))).unwrap();
        fs::remove_file(old_blocks.join(block_name(3))).unwrap();
        if matches!(shape, Shape::SelectedCandidateManifestOnly) {
            fs::remove_file(old_blocks.join(block_name(0))).unwrap();
            fs::remove_file(old_blocks.join(block_name(2))).unwrap();
            fs::remove_dir(&old_blocks).unwrap();
            // Deliberately do not sync the old generation: the recovery call
            // must establish this removal before it unlinks the old manifest.
        }
    }
}

fn exercise_recovery_failures(shape: Shape, action: ReplacementRecoveryActionV2) {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    let sample = Scratch::new();
    seed(&sample.root(), &old, &candidate, shape);
    let mut trace = Vec::new();
    let successful = recover_open_inner(
        &sample.root(),
        fixture::fixture_identity(),
        &request,
        action,
        |at| {
            trace.push(at);
            Ok(())
        },
    )
    .unwrap();
    let expected = match action {
        ReplacementRecoveryActionV2::Complete => &candidate,
        ReplacementRecoveryActionV2::RetainOld => &old,
    };
    assert_eq!(successful.chain().unwrap(), expected);
    assert!(!trace.is_empty());
    assert!(trace.contains(&RecoveryBoundary::Validated));
    assert!(trace.contains(&RecoveryBoundary::CleanValidated));
    if matches!(shape, Shape::SelectedCandidateManifestOnly) {
        let absence_synced = trace
            .iter()
            .position(|at| *at == RecoveryBoundary::RetiredGenerationAfterBlocksSynced)
            .unwrap();
        let manifest_removed = trace
            .iter()
            .position(|at| *at == RecoveryBoundary::RetiredManifest)
            .unwrap();
        assert!(
            absence_synced < manifest_removed,
            "old blocks-directory absence must be durable before manifest unlink"
        );
    }
    drop(successful);

    for (failure_index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        seed(&root, &old, &candidate, shape);
        let inode = fs::metadata(&root).unwrap();
        let mut observed = 0;
        let result =
            recover_open_inner(&root, fixture::fixture_identity(), &request, action, |at| {
                assert_eq!(at, trace[observed], "{shape:?} {action:?}");
                assert_owned(&root, &request);
                let fail = observed == failure_index;
                observed += 1;
                if fail { Err(failure()) } else { Ok(()) }
            });
        assert!(result.is_err(), "{shape:?} {action:?} {boundary:?}");
        assert_eq!(observed, failure_index + 1);
        let after = fs::metadata(&root).unwrap();
        assert_eq!((inode.dev(), inode.ino()), (after.dev(), after.ino()));
        let residue = snapshot(&root);

        // Ordinary opening must leave every incomplete layout untouched. If a
        // boundary happens to leave a clean layout, its entire state must match
        // exactly one of the independently retained histories.
        for (pins, history) in [(request.old(), &old), (request.candidate(), &candidate)] {
            if let Ok(store) =
                ReplacementStoreV2::open_writer(&root, fixture::fixture_identity(), pins)
            {
                assert_eq!(store.chain().unwrap(), history);
            }
        }
        assert_eq!(snapshot(&root), residue);
        let recovered = ReplacementStoreV2::recover_interrupted(
            &root,
            fixture::fixture_identity(),
            &request,
            action,
        )
        .unwrap_or_else(|error| panic!("retry after {shape:?} {action:?} {boundary:?}: {error:?}"));
        assert_eq!(recovered.chain().unwrap(), expected);
        assert_eq!(recovered.access(), StoreAccessV2::Writer);
        assert!(!recovered.is_fenced());
        let recovered = recovered.recover(&request, action).unwrap();
        assert_eq!(recovered.chain().unwrap(), expected);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    }
}

#[test]
fn completion_from_clean_old_retries_every_recovery_boundary() {
    exercise_recovery_failures(Shape::CleanOld, ReplacementRecoveryActionV2::Complete);
}

#[test]
fn completion_from_sparse_staging_retries_every_recovery_boundary() {
    exercise_recovery_failures(
        Shape::SparseCandidate,
        ReplacementRecoveryActionV2::Complete,
    );
}

#[test]
fn completion_from_partial_pending_selector_retries_every_recovery_boundary() {
    exercise_recovery_failures(
        Shape::PendingCandidate,
        ReplacementRecoveryActionV2::Complete,
    );
}

#[test]
fn selected_candidate_cleanup_retries_every_recovery_boundary() {
    exercise_recovery_failures(
        Shape::SelectedCandidate,
        ReplacementRecoveryActionV2::Complete,
    );
}

#[test]
fn already_absent_old_blocks_directory_is_resynchronized_and_every_boundary_retries() {
    exercise_recovery_failures(
        Shape::SelectedCandidateManifestOnly,
        ReplacementRecoveryActionV2::Complete,
    );
}

#[test]
fn retention_from_sparse_staging_retries_every_recovery_boundary() {
    exercise_recovery_failures(
        Shape::SparseCandidate,
        ReplacementRecoveryActionV2::RetainOld,
    );
}

#[test]
fn retention_from_pending_selector_retries_every_recovery_boundary() {
    exercise_recovery_failures(
        Shape::PendingCandidate,
        ReplacementRecoveryActionV2::RetainOld,
    );
}

fn replacement_trace(old: &ChainV2, candidate: &ChainV2) -> Vec<ReplacementBoundary> {
    let scratch = Scratch::new();
    let mut store = ReplacementStoreV2::initialize(scratch.root(), old).unwrap();
    let mut trace = Vec::new();
    store
        .replace_inner(
            fixture::fixture_identity(),
            request(old, candidate).old(),
            candidate,
            |at| {
                trace.push(at);
                Ok(())
            },
        )
        .unwrap();
    trace
}

#[test]
fn every_original_replacement_failure_can_complete_without_releasing_ownership() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    let trace = replacement_trace(&old, &candidate);
    for (index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
        let mut seen = 0;
        assert!(
            writer
                .replace_inner(
                    fixture::fixture_identity(),
                    request.old(),
                    &candidate,
                    |at| {
                        assert_eq!(at, trace[seen]);
                        let fail = seen == index;
                        seen += 1;
                        if fail {
                            Err(io::Error::other("original replacement interrupted").into())
                        } else {
                            Ok(())
                        }
                    }
                )
                .is_err()
        );
        assert!(writer.is_fenced());
        assert_eq!(writer.last_confirmed_checkpoint(), request.old());
        assert_eq!(writer.chain, old);
        let recovered = writer
            .recover_inner(&request, ReplacementRecoveryActionV2::Complete, |_| {
                assert_owned(&root, &request);
                Ok(())
            })
            .unwrap_or_else(|error| panic!("completion after original {boundary:?}: {error:?}"));
        assert_eq!(recovered.chain().unwrap(), &candidate);
        assert_eq!(recovered.checkpoint().unwrap(), request.candidate());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    }
}

#[test]
fn every_prepublication_replacement_failure_can_retain_exact_old_history() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    let trace = replacement_trace(&old, &candidate);
    let selected = trace
        .iter()
        .position(|at| *at == ReplacementBoundary::SelectionPublished)
        .unwrap();
    for index in 0..selected {
        let scratch = Scratch::new();
        let root = scratch.root();
        let mut writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
        let original = snapshot(&root);
        let mut seen = 0;
        assert!(
            writer
                .replace_inner(
                    fixture::fixture_identity(),
                    request.old(),
                    &candidate,
                    |_| {
                        let fail = seen == index;
                        seen += 1;
                        if fail {
                            Err(io::Error::other("prepublication replacement interrupted").into())
                        } else {
                            Ok(())
                        }
                    }
                )
                .is_err()
        );
        let recovered = writer
            .recover_inner(&request, ReplacementRecoveryActionV2::RetainOld, |_| {
                assert_owned(&root, &request);
                Ok(())
            })
            .unwrap_or_else(|error| {
                panic!("retention after original {:?}: {error:?}", trace[index])
            });
        assert_eq!(recovered.chain().unwrap(), &old);
        assert_eq!(recovered.checkpoint().unwrap(), request.old());
        assert_eq!(snapshot(&root), original);
    }
}

#[test]
fn consuming_recovery_failure_returns_no_writer_and_can_retry_under_new_ownership() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    let scratch = Scratch::new();
    let root = scratch.root();
    let writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let original = snapshot(&root);
    let error = writer
        .recover_inner(&request, ReplacementRecoveryActionV2::Complete, |at| {
            assert_owned(&root, &request);
            if at == RecoveryBoundary::Validated {
                Err(failure())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert!(!contains_busy(&error));
    assert_eq!(snapshot(&root), original);
    let recovered = ReplacementStoreV2::recover_interrupted(
        &root,
        fixture::fixture_identity(),
        &request,
        ReplacementRecoveryActionV2::Complete,
    )
    .unwrap();
    assert_eq!(recovered.chain().unwrap(), &candidate);
}

#[test]
fn interrupted_retention_can_explicitly_switch_to_completion_at_every_boundary() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    let sample = Scratch::new();
    seed(&sample.root(), &old, &candidate, Shape::PendingCandidate);
    let mut trace = Vec::new();
    drop(
        recover_open_inner(
            &sample.root(),
            fixture::fixture_identity(),
            &request,
            ReplacementRecoveryActionV2::RetainOld,
            |at| {
                trace.push(at);
                Ok(())
            },
        )
        .unwrap(),
    );
    for (index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        seed(&root, &old, &candidate, Shape::PendingCandidate);
        let mut seen = 0;
        assert!(
            recover_open_inner(
                &root,
                fixture::fixture_identity(),
                &request,
                ReplacementRecoveryActionV2::RetainOld,
                |_| {
                    let fail = seen == index;
                    seen += 1;
                    if fail { Err(failure()) } else { Ok(()) }
                },
            )
            .is_err()
        );
        let recovered = ReplacementStoreV2::recover_interrupted(
            &root,
            fixture::fixture_identity(),
            &request,
            ReplacementRecoveryActionV2::Complete,
        )
        .unwrap_or_else(|error| {
            panic!("switch to completion after retention {boundary:?}: {error:?}")
        });
        assert_eq!(recovered.chain().unwrap(), &candidate);
        assert_eq!(recovered.checkpoint().unwrap(), request.candidate());
    }
}

#[test]
fn interrupted_completion_can_retain_old_only_before_candidate_selection() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    let sample = Scratch::new();
    seed(&sample.root(), &old, &candidate, Shape::SparseCandidate);
    let mut trace = Vec::new();
    drop(
        recover_open_inner(
            &sample.root(),
            fixture::fixture_identity(),
            &request,
            ReplacementRecoveryActionV2::Complete,
            |at| {
                trace.push(at);
                Ok(())
            },
        )
        .unwrap(),
    );
    let old_marker = ReplacementSelectionV2::from_chain(&old, 0)
        .unwrap()
        .encode();
    for (index, boundary) in trace.iter().enumerate() {
        let scratch = Scratch::new();
        let root = scratch.root();
        seed(&root, &old, &candidate, Shape::SparseCandidate);
        let mut seen = 0;
        assert!(
            recover_open_inner(
                &root,
                fixture::fixture_identity(),
                &request,
                ReplacementRecoveryActionV2::Complete,
                |_| {
                    let fail = seen == index;
                    seen += 1;
                    if fail { Err(failure()) } else { Ok(()) }
                },
            )
            .is_err()
        );
        let before = snapshot(&root);
        let still_old = fs::read(root.join(SELECTION_NAME)).unwrap() == old_marker;
        let result = ReplacementStoreV2::recover_interrupted(
            &root,
            fixture::fixture_identity(),
            &request,
            ReplacementRecoveryActionV2::RetainOld,
        );
        if still_old {
            let retained = result.unwrap_or_else(|error| {
                panic!("retain old after completion {boundary:?}: {error:?}")
            });
            assert_eq!(retained.chain().unwrap(), &old);
            assert_eq!(retained.checkpoint().unwrap(), request.old());
        } else {
            assert!(
                result.is_err(),
                "selected candidate rolled back after {boundary:?}"
            );
            assert_eq!(snapshot(&root), before);
            let completed = ReplacementStoreV2::recover_interrupted(
                &root,
                fixture::fixture_identity(),
                &request,
                ReplacementRecoveryActionV2::Complete,
            )
            .unwrap();
            assert_eq!(completed.chain().unwrap(), &candidate);
        }
    }
}

#[test]
fn changed_residue_after_selected_sync_stops_before_any_cleanup() {
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = request(&old, &candidate);
    for corruption in [
        "old-block",
        "old-manifest",
        "candidate-block",
        "unknown-old-entry",
    ] {
        let scratch = Scratch::new();
        let root = scratch.root();
        seed(&root, &old, &candidate, Shape::SelectedCandidate);
        let mut tampered = None;
        let result = recover_open_inner(
            &root,
            fixture::fixture_identity(),
            &request,
            ReplacementRecoveryActionV2::Complete,
            |at| {
                if at == RecoveryBoundary::SelectedDurable {
                    let path = match corruption {
                        "old-block" => root
                            .join(generation_name(0))
                            .join(BLOCKS_NAME)
                            .join(block_name(2)),
                        "old-manifest" => root.join(generation_name(0)).join(MANIFEST_NAME),
                        "candidate-block" => root
                            .join(generation_name(1))
                            .join(BLOCKS_NAME)
                            .join(block_name(4)),
                        _ => root.join(generation_name(0)).join("unknown"),
                    };
                    if corruption == "unknown-old-entry" {
                        fs::write(&path, b"preserve this unknown artifact").unwrap();
                    } else {
                        let mut bytes = fs::read(&path).unwrap();
                        *bytes.last_mut().unwrap() ^= 1;
                        fs::write(&path, bytes).unwrap();
                    }
                    tampered = Some(snapshot(&root));
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "{corruption}");
        assert_eq!(
            snapshot(&root),
            tampered.expect("selected sync hook observed"),
            "{corruption}"
        );
    }
}

#[test]
fn residual_prefix_reads_charge_the_growth_sentinel_and_exhaust_without_mutation() {
    let scratch = Scratch::new();
    let path = scratch.0.join("prefix");
    let intended = b"synthetic canonical prefix bytes";
    fs::write(&path, &intended[..7]).unwrap();
    let mut budget = ReadBudget::new();
    compare_prefix(&path, Some(7), intended, &mut budget).unwrap();
    assert_eq!(budget.used, 8);
    budget.used = super::super::MAX_REPLACEMENT_READ_BYTES_V2 - 7;
    let error = compare_prefix(&path, Some(7), intended, &mut budget).unwrap_err();
    assert!(matches!(
        error,
        ReplacementRecoveryErrorV2::Replacement(
            ReplacementStorageErrorV2::ReadBudgetExceeded { .. }
        )
    ));
    assert_eq!(fs::read(&path).unwrap(), &intended[..7]);
}

#[test]
fn exhausted_prefix_read_budget_prevents_open_hook_and_suffix_write() {
    let scratch = Scratch::new();
    let path = scratch.0.join("prefix");
    let intended = b"synthetic canonical prefix bytes";
    fs::write(&path, &intended[..7]).unwrap();
    let mut budget = ReadBudget {
        used: super::super::MAX_REPLACEMENT_READ_BYTES_V2 - 7,
    };
    let mut called = false;
    let error = extend_prefix(
        &path,
        Some(7),
        intended,
        &mut budget,
        &mut |_| {
            called = true;
            Ok(())
        },
        FileKind::Manifest,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ReplacementRecoveryErrorV2::Replacement(
            ReplacementStorageErrorV2::ReadBudgetExceeded { .. }
        )
    ));
    assert!(!called);
    assert_eq!(fs::read(&path).unwrap(), &intended[..7]);
}
