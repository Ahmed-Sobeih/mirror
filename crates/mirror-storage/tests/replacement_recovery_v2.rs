//! Explicit whole-branch recovery using independently retained synthetic history.
//! Residual files are constructed only inside newly owned test directories.
#![cfg(target_os = "linux")]

mod support_v2;

use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mirror_chain::v2::{ChainErrorV2, ChainV2, NetworkManifestV2};
use mirror_core::v2::{BlockHeaderV2, BlockV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{
    AppendStoreV2, ReplacementCheckpointV2, ReplacementRecoveryActionV2 as Action,
    ReplacementRecoveryRequestV2 as Request, ReplacementSelectionV2, ReplacementStoreV2,
    StoreAccessV2, StoreV2,
};
use support_v2::{fixture_bytes, fixture_hash, fixture_identity, fork_chain, main_chain, manifest};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "mirror-replacement-recovery-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("cannot create owned test directory: {error}"),
            }
        }
        panic!("test directory namespace exhausted");
    }

    fn root(&self) -> PathBuf {
        self.0.join("store")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // This parent was created exclusively by this synthetic test invocation.
        // Production recovery must preserve rejected files and never recurse.
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
    Other,
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<PathBuf, Entry>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let name = path.strip_prefix(root).unwrap().to_path_buf();
        if metadata.file_type().is_symlink() {
            output.insert(name, Entry::Symlink(fs::read_link(path).unwrap()));
        } else if metadata.is_dir() {
            output.insert(name, Entry::Directory);
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), output);
            }
        } else if metadata.is_file() {
            output.insert(name, Entry::File(fs::read(path).unwrap()));
        } else {
            output.insert(name, Entry::Other);
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn checkpoint(chain: &ChainV2, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}

fn request<'a>(old: &'a ChainV2, generation: u64, candidate: &'a ChainV2) -> Request<'a> {
    Request::new(
        checkpoint(old, generation),
        checkpoint(candidate, generation + 1),
        old,
        candidate,
    )
    .unwrap()
}

fn generation_path(root: &Path, generation: u64) -> PathBuf {
    root.join(format!("generation-{generation:020}"))
}

fn block_path(root: &Path, generation: u64, height: u64) -> PathBuf {
    generation_path(root, generation)
        .join("blocks")
        .join(format!("{height:020}.mrb2"))
}

fn selector(chain: &ChainV2, generation: u64) -> [u8; 280] {
    ReplacementSelectionV2::from_chain(chain, generation)
        .unwrap()
        .encode()
}

fn initialize(root: &Path, old: &ChainV2) {
    drop(ReplacementStoreV2::initialize(root, old).unwrap());
}

fn stage_complete(root: &Path, generation: u64, chain: &ChainV2) {
    let path = generation_path(root, generation);
    fs::create_dir_all(path.join("blocks")).unwrap();
    fs::write(path.join("manifest.v2"), chain.manifest().encode().unwrap()).unwrap();
    for block in chain.blocks() {
        fs::write(
            block_path(root, generation, block.header().height()),
            block.encode(chain.config()).unwrap(),
        )
        .unwrap();
    }
}

fn stage_sparse(root: &Path, generation: u64, candidate: &ChainV2) {
    let path = generation_path(root, generation);
    fs::create_dir_all(path.join("blocks")).unwrap();
    // Surviving directory entries need not be an ordered height prefix.
    // A file may survive even if the manifest entry was never synchronized.
    fs::write(block_path(root, generation, 0), []).unwrap();
    let last = candidate.blocks().last().unwrap();
    let bytes = last.encode(candidate.config()).unwrap();
    fs::write(
        block_path(root, generation, last.header().height()),
        &bytes[..bytes.len() / 2],
    )
    .unwrap();
}

fn publish_candidate(root: &Path, candidate: &ChainV2) {
    stage_complete(root, 1, candidate);
    fs::write(root.join("selection.v2"), selector(candidate, 1)).unwrap();
}

fn recover(root: &Path, request: &Request<'_>, action: Action) -> ReplacementStoreV2 {
    ReplacementStoreV2::recover_interrupted(root, fixture_identity(), request, action).unwrap()
}

fn assert_state(store: &ReplacementStoreV2, expected: &ChainV2, generation: u64) {
    assert_eq!(store.access(), StoreAccessV2::Writer);
    assert!(!store.is_fenced());
    assert_eq!(
        store.checkpoint().unwrap(),
        checkpoint(expected, generation)
    );
    let actual = store.chain().unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual.encoded_bytes(), expected.encoded_bytes());
    assert_eq!(actual.state_items_used(), expected.state_items_used());
    assert_eq!(
        actual.ledger().account_bytes().unwrap(),
        expected.ledger().account_bytes().unwrap()
    );
    assert_eq!(
        actual.ledger().membership().encode().unwrap(),
        expected.ledger().membership().encode().unwrap()
    );
}

fn assert_clean_files(root: &Path, expected: &ChainV2, generation: u64) {
    assert_eq!(fs::read_dir(root).unwrap().count(), 2);
    assert!(!root.join("selection.pending").exists());
    assert_eq!(
        fs::read(root.join("selection.v2")).unwrap(),
        selector(expected, generation)
    );
    let path = generation_path(root, generation);
    assert_eq!(fs::read_dir(&path).unwrap().count(), 2);
    assert_eq!(
        fs::read(path.join("manifest.v2")).unwrap(),
        expected.manifest().encode().unwrap()
    );
    assert_eq!(
        fs::read_dir(path.join("blocks")).unwrap().count(),
        expected.blocks().len()
    );
    for block in expected.blocks() {
        assert_eq!(
            fs::read(block_path(root, generation, block.header().height())).unwrap(),
            block.encode(expected.config()).unwrap()
        );
    }
}

fn reject_both_unchanged(root: &Path, request: &Request<'_>) {
    let before = snapshot(root);
    for action in [Action::Complete, Action::RetainOld] {
        assert!(
            ReplacementStoreV2::recover_interrupted(root, fixture_identity(), request, action)
                .is_err(),
            "unexpected recovery acceptance for {action:?}"
        );
        assert_eq!(snapshot(root), before, "rejected recovery changed files");
    }
}

fn append_empty(chain: &mut ChainV2) {
    let proposal = chain
        .prepare_candidate(chain.blocks().last().unwrap().header().timestamp() + 1, &[])
        .unwrap();
    let header = proposal.header();
    for nonce in 0..10_000 {
        let mined = BlockHeaderV2::new(
            header.config_hash(),
            header.height(),
            header.previous_hash(),
            header.operations_root(),
            header.ledger_root(),
            header.timestamp(),
            header.pow_bits(),
            nonce,
        );
        let block = BlockV2::new(chain.config(), mined, proposal.operations()).unwrap();
        match chain.append(block) {
            Ok(()) => return,
            Err(ChainErrorV2::Work(_)) => {}
            Err(error) => panic!("unexpected synthetic child rejection: {error}"),
        }
    }
    panic!("public synthetic target search exhausted");
}

fn rechecksum(bytes: &mut [u8; 280]) {
    let mut inner = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    inner.extend_from_slice(&bytes[16..216]);
    bytes[216..248].copy_from_slice(sha256d(&inner).as_bytes());
    let mut outer = b"MIRROR_STORAGE_REPLACEMENT_SELECTION_V2".to_vec();
    outer.extend_from_slice(&bytes[..248]);
    bytes[248..].copy_from_slice(sha256d(&outer).as_bytes());
}

#[test]
fn request_retains_both_accepted_histories_and_external_checkpoints() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    assert_eq!(request.old(), checkpoint(&old, 0));
    assert_eq!(request.candidate(), checkpoint(&candidate, 1));
    assert!(std::ptr::eq(request.old_chain(), &old));
    assert!(std::ptr::eq(request.candidate_chain(), &candidate));
}

#[test]
fn clean_old_can_complete_and_complete_can_be_repeated() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    let before_inode = fs::metadata(&root).unwrap().ino();
    for _ in 0..2 {
        let store = recover(&root, &request, Action::Complete);
        assert_state(&store, &candidate, 1);
        assert_clean_files(&root, &candidate, 1);
        assert_eq!(fs::metadata(&root).unwrap().ino(), before_inode);
        drop(store);
    }
    let reopened =
        ReplacementStoreV2::open_writer(&root, fixture_identity(), request.candidate()).unwrap();
    assert_state(&reopened, &candidate, 1);
}

#[test]
fn clean_old_retention_is_idempotent_and_does_not_publish() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    let before = snapshot(&root);
    for _ in 0..2 {
        let store = recover(&root, &request, Action::RetainOld);
        assert_state(&store, &old, 0);
        assert_eq!(snapshot(&root), before);
    }
}

#[test]
fn sparse_staging_completes_missing_and_partial_candidate_files() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    stage_sparse(&root, 1, &candidate);
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &candidate, 1);
    assert_clean_files(&root, &candidate, 1);
}

#[test]
fn sparse_staging_can_be_removed_while_retaining_the_exact_old_history() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    let before = snapshot(&root);
    stage_sparse(&root, 1, &candidate);
    let bytes = candidate.manifest().encode().unwrap();
    fs::write(generation_path(&root, 1).join("manifest.v2"), &bytes[..11]).unwrap();
    let store = recover(&root, &request, Action::RetainOld);
    assert_state(&store, &old, 0);
    assert_eq!(snapshot(&root), before);
}

#[test]
fn candidate_directory_only_and_manifest_only_shapes_support_both_actions() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..4 {
        for action in [Action::Complete, Action::RetainOld] {
            let scratch = Scratch::new();
            let root = scratch.root();
            initialize(&root, &old);
            let next = generation_path(&root, 1);
            fs::create_dir(&next).unwrap();
            if shape == 1 || shape == 3 {
                fs::create_dir(next.join("blocks")).unwrap();
            }
            if shape >= 2 {
                fs::write(next.join("manifest.v2"), []).unwrap();
            }
            let store = recover(&root, &request, action);
            let (expected, generation) = if action == Action::Complete {
                (&candidate, 1)
            } else {
                (&old, 0)
            };
            assert_state(&store, expected, generation);
            assert_clean_files(&root, expected, generation);
        }
    }
}

#[test]
fn manifest_and_block_prefixes_preserve_existing_file_inodes_during_completion() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    stage_sparse(&root, 1, &candidate);
    let path = generation_path(&root, 1).join("manifest.v2");
    let bytes = candidate.manifest().encode().unwrap();
    fs::write(&path, &bytes[..bytes.len() / 2]).unwrap();
    let before_manifest = fs::metadata(&path).unwrap().ino();
    let before_block = fs::metadata(block_path(&root, 1, 4)).unwrap().ino();
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &candidate, 1);
    assert_eq!(fs::metadata(&path).unwrap().ino(), before_manifest);
    assert_eq!(
        fs::metadata(block_path(&root, 1, 4)).unwrap().ino(),
        before_block
    );
}

#[test]
fn pending_selector_prefixes_require_complete_candidate_and_support_both_actions() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    let bytes = selector(&candidate, 1);
    for length in [0, 1, 4, 16, 232, 248, 279, 280] {
        for action in [Action::Complete, Action::RetainOld] {
            let scratch = Scratch::new();
            let root = scratch.root();
            initialize(&root, &old);
            stage_complete(&root, 1, &candidate);
            fs::write(root.join("selection.pending"), &bytes[..length]).unwrap();
            let store = recover(&root, &request, action);
            let (expected, generation) = if action == Action::Complete {
                (&candidate, 1)
            } else {
                (&old, 0)
            };
            assert_state(&store, expected, generation);
            assert_clean_files(&root, expected, generation);
        }
    }
}

#[test]
fn candidate_selected_cleanup_accepts_arbitrary_complete_old_file_subsets() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    // Neither the order of surviving heights nor the number removed is authority.
    for keep in [vec![0, 1, 2, 3], vec![1, 3], vec![0, 2], vec![3], vec![]] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        for height in 0..=old.height() {
            if !keep.contains(&height) {
                fs::remove_file(block_path(&root, 0, height)).unwrap();
            }
        }
        let store = recover(&root, &request, Action::Complete);
        assert_state(&store, &candidate, 1);
        assert_clean_files(&root, &candidate, 1);
    }
}

#[test]
fn candidate_selected_cleanup_accepts_manifest_only_empty_and_absent_old_generation() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..3 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        let old_path = generation_path(&root, 0);
        fs::remove_dir_all(old_path.join("blocks")).unwrap();
        if shape >= 1 {
            fs::remove_file(old_path.join("manifest.v2")).unwrap();
        }
        if shape >= 2 {
            fs::remove_dir(&old_path).unwrap();
        }
        let store = recover(&root, &request, Action::Complete);
        assert_state(&store, &candidate, 1);
        assert_clean_files(&root, &candidate, 1);
    }
}

#[test]
fn selected_candidate_can_never_be_rolled_back_to_old() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for old_present in [true, false] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        if !old_present {
            fs::remove_dir_all(generation_path(&root, 0)).unwrap();
        }
        let before = snapshot(&root);
        assert!(
            ReplacementStoreV2::recover_interrupted(
                &root,
                fixture_identity(),
                &request,
                Action::RetainOld
            )
            .is_err()
        );
        assert_eq!(snapshot(&root), before);
    }
}

#[test]
fn same_accounts_do_not_preserve_orphaned_memberships_or_population_history() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = main_chain(3);
    let mut candidate = fork_chain();
    append_empty(&mut candidate);
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(old.ledger().membership(), candidate.ledger().membership());
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    publish_candidate(&root, &candidate);
    fs::remove_file(block_path(&root, 0, 0)).unwrap();
    fs::remove_file(block_path(&root, 0, 2)).unwrap();
    let recovered = recover(&root, &request, Action::Complete);
    assert_state(&recovered, &candidate, 1);
    drop(recovered);
    let reopened =
        ReplacementStoreV2::open_writer(&root, fixture_identity(), request.candidate()).unwrap();
    assert_state(&reopened, &candidate, 1);
    for id in [fixture_hash("c_member_id"), fixture_hash("seed0_id")] {
        assert_eq!(
            reopened.chain().unwrap().ledger().membership().record(id),
            candidate.ledger().membership().record(id)
        );
        assert_ne!(
            reopened.chain().unwrap().ledger().membership().record(id),
            old.ledger().membership().record(id)
        );
    }
    assert_eq!(
        reopened
            .chain()
            .unwrap()
            .ledger()
            .membership()
            .active_high_water(),
        candidate.ledger().membership().active_high_water()
    );
    assert_eq!(
        reopened
            .chain()
            .unwrap()
            .ledger()
            .membership()
            .retained_approval_count(),
        candidate.ledger().membership().retained_approval_count()
    );
    assert_eq!(
        reopened
            .chain()
            .unwrap()
            .ledger()
            .membership()
            .total_activations(),
        candidate.ledger().membership().total_activations()
    );
}

#[test]
fn recovery_keeps_ordinary_open_inspect_and_reconcile_strict() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    let store = ReplacementStoreV2::initialize(&root, &old).unwrap();
    stage_sparse(&root, 1, &candidate);
    let before = snapshot(&root);
    assert!(store.reconcile(request.old()).is_err());
    assert_eq!(snapshot(&root), before);
    assert!(ReplacementStoreV2::open_writer(&root, fixture_identity(), request.old()).is_err());
    assert!(ReplacementStoreV2::inspect(&root, fixture_identity(), request.old()).is_err());
    assert_eq!(snapshot(&root), before);
    let recovered = recover(&root, &request, Action::Complete);
    assert_state(&recovered, &candidate, 1);
}

#[test]
fn consuming_recovery_accepts_a_fenced_writer_and_retains_the_root_inode() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    let mut store = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let inode = fs::metadata(&root).unwrap().ino();
    stage_sparse(&root, 1, &candidate);
    assert!(
        store
            .replace(fixture_identity(), request.old(), &candidate)
            .is_err()
    );
    assert!(store.is_fenced());
    let recovered = store.recover(&request, Action::Complete).unwrap();
    assert_state(&recovered, &candidate, 1);
    assert_eq!(fs::metadata(&root).unwrap().ino(), inode);
    assert!(
        ReplacementStoreV2::open_writer(&root, fixture_identity(), request.candidate()).is_err()
    );
}

#[test]
fn consuming_recovery_rejects_shared_inspection_and_unrelated_memory_checkpoint() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let initial_request = request(&old, 0, &candidate);
    initialize(&root, &old);
    let inspector =
        ReplacementStoreV2::inspect(&root, fixture_identity(), initial_request.old()).unwrap();
    let before = snapshot(&root);
    assert!(
        inspector
            .recover(&initial_request, Action::RetainOld)
            .is_err()
    );
    assert_eq!(snapshot(&root), before);
    let unrelated = main_chain(2);
    let request = request(&unrelated, 0, &candidate);
    let writer =
        ReplacementStoreV2::open_writer(&root, fixture_identity(), checkpoint(&old, 0)).unwrap();
    assert!(writer.recover(&request, Action::Complete).is_err());
    assert_eq!(snapshot(&root), before);
}

#[test]
fn rejected_consuming_recovery_returns_no_handle_and_preserves_unknown_residue() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    let store = ReplacementStoreV2::initialize(&root, &old).unwrap();
    fs::write(root.join("unknown"), b"keep").unwrap();
    let before = snapshot(&root);
    assert!(store.recover(&request, Action::Complete).is_err());
    assert_eq!(snapshot(&root), before);
    // Invalid residue remains intact. After the test owner removes its own
    // unrelated file, a new explicit attempt can acquire ownership successfully.
    reject_both_unchanged(&root, &request);
    fs::remove_file(root.join("unknown")).unwrap();
    assert_state(&recover(&root, &request, Action::Complete), &candidate, 1);
}

#[test]
fn wrong_identity_is_rejected_before_any_staging_changes() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    stage_sparse(&root, 1, &candidate);
    let before = snapshot(&root);
    for change_config in [true, false] {
        let mut identity = fixture_identity();
        if change_config {
            identity.config_hash = Hash256::from_bytes([0x77; 32]);
        } else {
            identity.genesis_hash = Hash256::from_bytes([0x77; 32]);
        }
        assert!(
            ReplacementStoreV2::recover_interrupted(&root, identity, &request, Action::Complete)
                .is_err()
        );
        assert_eq!(snapshot(&root), before);
    }
}

#[test]
fn request_rejects_wrong_history_endpoints_and_generation_transitions() {
    let old = fork_chain();
    let candidate = main_chain(4);
    for bad_generation in [0, 2, u64::MAX] {
        assert!(
            Request::new(
                checkpoint(&old, 0),
                checkpoint(&candidate, bad_generation),
                &old,
                &candidate
            )
            .is_err()
        );
    }
    assert!(
        Request::new(
            checkpoint(&old, 4),
            checkpoint(&candidate, 5),
            &old,
            &candidate
        )
        .is_err()
    );
    let wrong_old = main_chain(3);
    assert!(
        Request::new(
            checkpoint(&wrong_old, 0),
            checkpoint(&candidate, 1),
            &old,
            &candidate
        )
        .is_err()
    );
    let mut wrong_candidate = checkpoint(&candidate, 1);
    wrong_candidate.chain.ledger_root = Hash256::from_bytes([0x44; 32]);
    assert!(Request::new(checkpoint(&old, 0), wrong_candidate, &old, &candidate).is_err());
}

#[test]
fn request_rejects_shorter_tied_and_foreign_genesis_histories() {
    let old = fork_chain();
    for candidate in [main_chain(2), main_chain(3), old.clone()] {
        assert!(
            Request::new(
                checkpoint(&old, 0),
                checkpoint(&candidate, 1),
                &old,
                &candidate
            )
            .is_err()
        );
    }
    let base = manifest();
    let genesis =
        BlockV2::decode(base.config(), &fixture_bytes("alternate_genesis_block")).unwrap();
    let alternate =
        NetworkManifestV2::new(base.config(), base.seed_proofs(), genesis.hash()).unwrap();
    let mut foreign = ChainV2::from_genesis(&alternate, genesis).unwrap();
    for _ in 0..4 {
        append_empty(&mut foreign);
    }
    assert!(Request::new(checkpoint(&old, 0), checkpoint(&foreign, 1), &old, &foreign).is_err());
}

#[test]
fn valid_request_with_wrong_local_generation_cannot_adopt_other_residue() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    initialize(&root, &old);
    stage_sparse(&root, 1, &candidate);
    let wrong = request(&old, 1, &candidate);
    reject_both_unchanged(&root, &wrong);
}

#[test]
fn missing_truncated_corrupt_and_unpinned_selection_markers_are_preserved() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..6 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        stage_complete(&root, 1, &candidate);
        match shape {
            0 => fs::remove_file(root.join("selection.v2")).unwrap(),
            1 => fs::write(root.join("selection.v2"), []).unwrap(),
            2 => fs::write(root.join("selection.v2"), &selector(&old, 0)[..279]).unwrap(),
            3 => {
                let mut bytes = selector(&old, 0);
                bytes[100] ^= 1;
                fs::write(root.join("selection.v2"), bytes).unwrap();
            }
            4 => fs::write(root.join("selection.v2"), selector(&old, 1)).unwrap(),
            5 => fs::write(root.join("selection.v2"), selector(&main_chain(3), 0)).unwrap(),
            _ => unreachable!(),
        }
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn valid_checksum_with_wrong_canonical_metadata_is_not_selection_authority() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    let mut bytes = selector(&old, 0);
    // Preserve checkpoint fields while falsifying embedded manifest length.
    bytes[16 + 156] ^= 1;
    rechecksum(&mut bytes);
    assert!(ReplacementSelectionV2::decode_for(&bytes, request.old()).is_ok());
    fs::write(root.join("selection.v2"), bytes).unwrap();
    reject_both_unchanged(&root, &request);
}

#[test]
fn candidate_selected_with_pending_or_incomplete_selected_history_is_preserved() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..5 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        match shape {
            0 => fs::write(root.join("selection.pending"), []).unwrap(),
            1 => fs::remove_file(block_path(&root, 1, 4)).unwrap(),
            2 => fs::write(block_path(&root, 1, 4), []).unwrap(),
            3 => fs::remove_file(generation_path(&root, 1).join("manifest.v2")).unwrap(),
            4 => {
                let path = block_path(&root, 1, 4);
                let mut bytes = fs::read(&path).unwrap();
                let last = bytes.len() - 1;
                bytes[last] ^= 1;
                fs::write(path, bytes).unwrap();
            }
            _ => unreachable!(),
        }
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn any_pending_selector_with_incomplete_candidate_is_rejected_unchanged() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for length in [0, 4, 280] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        stage_sparse(&root, 1, &candidate);
        fs::write(
            root.join("selection.pending"),
            &selector(&candidate, 1)[..length],
        )
        .unwrap();
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn foreign_or_oversized_candidate_prefixes_are_preserved() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..4 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        stage_sparse(&root, 1, &candidate);
        let (path, mut bytes) = if shape < 2 {
            (
                generation_path(&root, 1).join("manifest.v2"),
                candidate.manifest().encode().unwrap(),
            )
        } else {
            (
                block_path(&root, 1, 4),
                candidate
                    .block_at(4)
                    .unwrap()
                    .encode(candidate.config())
                    .unwrap(),
            )
        };
        if shape % 2 == 0 {
            bytes[0] ^= 1;
            bytes.truncate(7);
        } else {
            bytes.push(0);
        }
        fs::write(path, bytes).unwrap();
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn foreign_or_oversized_pending_selector_is_preserved() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..3 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        stage_complete(&root, 1, &candidate);
        let mut bytes = selector(&candidate, 1).to_vec();
        match shape {
            0 => {
                bytes[0] ^= 1;
                bytes.truncate(1);
            }
            1 => {
                bytes[279] ^= 1;
            }
            2 => bytes.push(0),
            _ => unreachable!(),
        }
        fs::write(root.join("selection.pending"), bytes).unwrap();
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn every_surviving_old_file_is_checked_before_any_cleanup_deletion() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..4 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        let path = if shape < 2 {
            block_path(&root, 0, 3)
        } else {
            generation_path(&root, 0).join("manifest.v2")
        };
        let mut bytes = fs::read(&path).unwrap();
        if shape % 2 == 0 {
            bytes.pop();
        } else {
            let last = bytes.len() - 1;
            bytes[last] ^= 1;
        }
        fs::write(path, bytes).unwrap();
        reject_both_unchanged(&root, &request);
        assert!(block_path(&root, 0, 0).exists());
    }
}

#[test]
fn selected_old_requires_its_complete_original_history_before_either_action() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..3 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        stage_sparse(&root, 1, &candidate);
        match shape {
            0 => fs::remove_file(block_path(&root, 0, 2)).unwrap(),
            1 => fs::write(block_path(&root, 0, 3), []).unwrap(),
            2 => fs::remove_file(generation_path(&root, 0).join("manifest.v2")).unwrap(),
            _ => unreachable!(),
        }
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn old_cleanup_cannot_skip_manifest_ordering_while_block_directory_survives() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for empty_blocks in [false, true] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        if empty_blocks {
            for height in 0..=old.height() {
                fs::remove_file(block_path(&root, 0, height)).unwrap();
            }
        }
        fs::remove_file(generation_path(&root, 0).join("manifest.v2")).unwrap();
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn unknown_root_generation_and_block_entries_are_rejected_before_mutation() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for shape in 0..7 {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        let path = match shape {
            0 => root.join("unknown"),
            1 => generation_path(&root, 2),
            2 => generation_path(&root, 0).join("unknown"),
            3 => generation_path(&root, 1).join("unknown"),
            4 => block_path(&root, 0, 4),
            5 => block_path(&root, 1, 5),
            6 => generation_path(&root, 0).join("blocks/1.mrb2"),
            _ => unreachable!(),
        };
        fs::write(path, b"preserve unknown bytes").unwrap();
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn candidate_and_old_generation_symlinks_are_never_followed() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for generation in [0, 1] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &old);
        publish_candidate(&root, &candidate);
        let original = generation_path(&root, generation);
        let outside = scratch.0.join("outside-generation");
        fs::rename(&original, &outside).unwrap();
        symlink(&outside, &original).unwrap();
        let before = snapshot(&scratch.0);
        reject_both_unchanged(&root, &request);
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn recovery_rejects_symlink_and_hardlinked_payloads_and_selectors() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for hardlink in [false, true] {
        for target in 0..3 {
            let scratch = Scratch::new();
            let root = scratch.root();
            initialize(&root, &old);
            stage_complete(&root, 1, &candidate);
            let path = match target {
                0 => block_path(&root, 1, 4),
                1 => generation_path(&root, 1).join("manifest.v2"),
                2 => root.join("selection.v2"),
                _ => unreachable!(),
            };
            let outside = scratch.0.join("outside-file");
            fs::rename(&path, &outside).unwrap();
            if hardlink {
                fs::hard_link(&outside, &path).unwrap();
            } else {
                symlink(&outside, &path).unwrap();
            }
            let before = snapshot(&scratch.0);
            reject_both_unchanged(&root, &request);
            assert_eq!(snapshot(&scratch.0), before);
        }
    }
}

#[test]
fn recovery_never_creates_missing_root_or_adopts_interrupted_initialization() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    assert!(
        ReplacementStoreV2::recover_interrupted(
            &root,
            fixture_identity(),
            &request,
            Action::Complete
        )
        .is_err()
    );
    assert!(!root.exists());
    fs::create_dir(&root).unwrap();
    reject_both_unchanged(&root, &request);
    stage_complete(&root, 0, &old);
    fs::write(root.join("selection.pending"), selector(&old, 0)).unwrap();
    reject_both_unchanged(&root, &request);
}

#[test]
fn recovery_does_not_migrate_immutable_append_or_legacy_profiles() {
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    for profile in 0..3 {
        let scratch = Scratch::new();
        let root = scratch.root();
        match profile {
            0 => drop(StoreV2::initialize(&root, &old).unwrap()),
            1 => drop(AppendStoreV2::initialize(&root, &old).unwrap()),
            2 => {
                fs::create_dir(&root).unwrap();
                fs::write(root.join("00000000000000000000.mrb"), b"legacy").unwrap();
            }
            _ => unreachable!(),
        }
        reject_both_unchanged(&root, &request);
    }
}

#[test]
fn an_existing_writer_or_shared_inspector_excludes_recovery() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    let writer = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let before = snapshot(&root);
    assert!(
        ReplacementStoreV2::recover_interrupted(
            &root,
            fixture_identity(),
            &request,
            Action::Complete
        )
        .is_err()
    );
    assert_eq!(snapshot(&root), before);
    drop(writer);
    let inspector = ReplacementStoreV2::inspect(&root, fixture_identity(), request.old()).unwrap();
    assert!(
        ReplacementStoreV2::recover_interrupted(
            &root,
            fixture_identity(),
            &request,
            Action::RetainOld
        )
        .is_err()
    );
    assert_eq!(snapshot(&root), before);
    drop(inspector);
    assert_state(&recover(&root, &request, Action::Complete), &candidate, 1);
}

#[test]
fn full_accepted_histories_remain_available_after_recovery_failure() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let old_copy = old.clone();
    let candidate_copy = candidate.clone();
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    fs::write(root.join("unknown"), b"preserve").unwrap();
    reject_both_unchanged(&root, &request);
    assert_eq!(old, old_copy);
    assert_eq!(candidate, candidate_copy);
}

#[test]
fn multi_height_candidate_advances_only_one_local_generation() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = main_chain(0);
    let candidate = main_chain(4);
    let request = request(&old, 0, &candidate);
    initialize(&root, &old);
    stage_sparse(&root, 1, &candidate);
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &candidate, 1);
    assert!(!generation_path(&root, 4).exists());
}

#[test]
fn later_generation_recovery_uses_pinned_names_not_highest_directory() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let base = main_chain(0);
    let old = main_chain(2);
    let candidate = main_chain(4);
    let mut store = ReplacementStoreV2::initialize(&root, &base).unwrap();
    store
        .replace(fixture_identity(), checkpoint(&base, 0), &old)
        .unwrap();
    drop(store);
    let request = request(&old, 1, &candidate);
    stage_sparse(&root, 2, &candidate);
    let recovered = recover(&root, &request, Action::Complete);
    assert_state(&recovered, &candidate, 2);
    assert_clean_files(&root, &candidate, 2);
    assert!(!generation_path(&root, 1).exists());
}
