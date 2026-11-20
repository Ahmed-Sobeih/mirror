//! Explicit residual-append recovery against frozen, independently derived MRB2.
//!
//! Every filesystem path belongs to this test invocation. No test changes real
//! stores or derives recovery authority from the untrusted pending selector.
#![cfg(target_os = "linux")]

mod support_v2;

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mirror_chain::v2::{ChainV2, ManifestIdentityV2};
use mirror_core::v2::{BlockHeaderV2, BlockV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{
    AppendCheckpointV2, AppendRecoveryActionV2 as Action, AppendRecoveryErrorV2 as Error,
    AppendRecoveryRequestV2 as Request, AppendSelectionV2, AppendStoreV2, StorageErrorV2,
    StoreAccessV2, StoreV2,
};

use support_v2::{fixture_bytes, main_chain};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "mirror-v2-recovery-integration-{}-{}",
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
        // Remove only this test's freshly created parent, including its own
        // synthetic corrupt files. Recovery itself must preserve rejected data.
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
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

fn checkpoint(chain: &ChainV2, generation: u64) -> AppendCheckpointV2 {
    AppendCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}

fn identity() -> ManifestIdentityV2 {
    main_chain(0).manifest().identity()
}

fn request(parent: &ChainV2, generation: u64, candidate: &ChainV2) -> Request {
    Request::new(
        checkpoint(parent, generation),
        checkpoint(candidate, generation + 1),
        candidate.block_at(candidate.height()).unwrap().clone(),
    )
    .unwrap()
}

fn main_request(parent_height: u64) -> Request {
    request(
        &main_chain(parent_height),
        0,
        &main_chain(parent_height + 1),
    )
}

fn block_path(root: &Path, height: u64) -> PathBuf {
    root.join("blocks").join(format!("{height:020}.mrb2"))
}

fn block(name: &str) -> BlockV2 {
    BlockV2::decode(
        main_chain(0).config(),
        &fixture_bytes(&format!("{name}_block")),
    )
    .unwrap()
}

fn initialize(root: &Path, chain: &ChainV2) {
    drop(AppendStoreV2::initialize(root, chain).unwrap());
}

fn selected_bytes(chain: &ChainV2, generation: u64) -> Vec<u8> {
    AppendSelectionV2::from_chain(chain, generation)
        .unwrap()
        .encode()
        .to_vec()
}

fn stage_tail(root: &Path, request: &Request, length: usize) {
    let bytes = request.block().encode(main_chain(0).config()).unwrap();
    fs::write(
        block_path(root, request.candidate().chain.height),
        &bytes[..length],
    )
    .unwrap();
}

fn stage_pending(root: &Path, candidate: &ChainV2, generation: u64, length: usize) {
    fs::write(
        root.join("selection.pending"),
        &selected_bytes(candidate, generation)[..length],
    )
    .unwrap();
}

fn recover(root: &Path, request: &Request, action: Action) -> AppendStoreV2 {
    AppendStoreV2::recover_interrupted(root, identity(), request, action).unwrap()
}

fn assert_state(store: &AppendStoreV2, expected: &ChainV2, generation: u64) {
    assert_eq!(store.access(), StoreAccessV2::Writer);
    assert!(!store.is_fenced());
    assert_eq!(
        store.checkpoint().unwrap(),
        checkpoint(expected, generation)
    );
    assert_eq!(store.chain().unwrap(), expected);
    assert_eq!(
        store.chain().unwrap().encoded_bytes(),
        expected.encoded_bytes()
    );
    assert_eq!(
        store.chain().unwrap().state_items_used(),
        expected.state_items_used()
    );
    assert_eq!(
        store.chain().unwrap().ledger().account_bytes().unwrap(),
        expected.ledger().account_bytes().unwrap()
    );
    assert_eq!(
        store
            .chain()
            .unwrap()
            .ledger()
            .membership()
            .encode()
            .unwrap(),
        expected.ledger().membership().encode().unwrap()
    );
}

fn assert_clean_files(root: &Path, expected: &ChainV2, generation: u64) {
    assert_eq!(fs::read_dir(root).unwrap().count(), 3);
    assert!(!root.join("selection.pending").exists());
    assert_eq!(
        fs::read(root.join("manifest.v2")).unwrap(),
        expected.manifest().encode().unwrap()
    );
    assert_eq!(
        fs::read(root.join("selection.v2")).unwrap(),
        selected_bytes(expected, generation)
    );
    assert_eq!(
        fs::read_dir(root.join("blocks")).unwrap().count(),
        expected.blocks().len()
    );
    for block in expected.blocks() {
        assert_eq!(
            fs::read(block_path(root, block.header().height())).unwrap(),
            block.encode(expected.config()).unwrap()
        );
    }
}

fn reject_both_without_mutation(scratch: &Scratch, request: &Request) {
    let before = snapshot(&scratch.0);
    for action in [Action::Complete, Action::RetainParent] {
        let result =
            AppendStoreV2::recover_interrupted(scratch.root(), identity(), request, action);
        assert!(result.is_err(), "unexpectedly recovered {action:?}");
        assert_eq!(snapshot(&scratch.0), before);
    }
}

fn changed_hash(value: Hash256) -> Hash256 {
    let mut bytes = *value.as_bytes();
    bytes[0] ^= 1;
    Hash256::from_bytes(bytes)
}

fn refresh_selector_checksums(bytes: &mut [u8]) {
    let mut inner = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    inner.extend_from_slice(&bytes[16..216]);
    bytes[216..248].copy_from_slice(sha256d(&inner).as_bytes());
    let mut outer = b"MIRROR_STORAGE_APPEND_SELECTION_V2".to_vec();
    outer.extend_from_slice(&bytes[..248]);
    bytes[248..280].copy_from_slice(sha256d(&outer).as_bytes());
}

#[test]
fn complete_can_materialize_an_independently_authorized_missing_child() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let candidate = main_chain(1);
    initialize(&root, &parent);
    let request = main_request(0);
    assert_eq!(request.parent(), checkpoint(&parent, 0));
    assert_eq!(request.candidate(), checkpoint(&candidate, 1));
    assert_eq!(request.block(), candidate.block_at(1).unwrap());
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &candidate, 1);
    assert_clean_files(&root, &candidate, 1);
}

#[test]
fn complete_finishes_empty_and_partial_child_prefixes_without_changing_the_prefix() {
    let candidate = main_chain(1);
    let bytes = fixture_bytes("block1_block");
    for cut in [0, 1, 4, 164, bytes.len() / 2, bytes.len() - 1] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, cut);
        let store = recover(&root, &request, Action::Complete);
        assert_state(&store, &candidate, 1);
        assert_clean_files(&root, &candidate, 1);
        assert_eq!(
            &fs::read(block_path(&root, 1)).unwrap()[..cut],
            &bytes[..cut]
        );
    }
}

#[test]
fn complete_publishes_a_full_child_without_a_pending_selector() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(1));
    let request = main_request(1);
    stage_tail(&root, &request, request.block().encoded_len());
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &main_chain(2), 1);
    assert_clean_files(&root, &main_chain(2), 1);
}

#[test]
fn complete_finishes_empty_partial_and_full_pending_selector_prefixes() {
    for cut in [0, 1, 4, 16, 100, 248, 279, 280] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        stage_pending(&root, &main_chain(1), 1, cut);
        let store = recover(&root, &request, Action::Complete);
        assert_state(&store, &main_chain(1), 1);
        assert_clean_files(&root, &main_chain(1), 1);
    }
}

#[test]
fn retaining_a_clean_parent_is_idempotent_and_does_not_advance_generation() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(2);
    initialize(&root, &parent);
    let request = main_request(2);
    let before = snapshot(&scratch.0);
    for _ in 0..2 {
        let store = recover(&root, &request, Action::RetainParent);
        assert_state(&store, &parent, 0);
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn retain_parent_removes_only_the_named_matching_child_prefix() {
    let bytes = fixture_bytes("block1_block");
    for cut in [0, 1, 164, bytes.len() - 1, bytes.len()] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let parent = main_chain(0);
        initialize(&root, &parent);
        let before = snapshot(&scratch.0);
        let request = main_request(0);
        stage_tail(&root, &request, cut);
        let store = recover(&root, &request, Action::RetainParent);
        assert_state(&store, &parent, 0);
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn retain_parent_cleans_matching_pending_then_child_and_can_be_retried() {
    for cut in [0, 1, 100, 279, 280] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let parent = main_chain(2);
        initialize(&root, &parent);
        let before = snapshot(&scratch.0);
        let request = main_request(2);
        stage_tail(&root, &request, request.block().encoded_len());
        stage_pending(&root, &main_chain(3), 1, cut);
        let store = recover(&root, &request, Action::RetainParent);
        assert_state(&store, &parent, 0);
        assert_eq!(snapshot(&scratch.0), before);
        drop(store);
        assert_state(&recover(&root, &request, Action::RetainParent), &parent, 0);
    }
}

#[test]
fn repeating_complete_after_new_selection_resynchronizes_without_a_second_advance() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(1));
    let request = main_request(1);
    drop(recover(&root, &request, Action::Complete));
    let before = snapshot(&scratch.0);
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &main_chain(2), 1);
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn retain_parent_never_rolls_back_an_already_selected_candidate() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(1));
    let request = main_request(1);
    drop(recover(&root, &request, Action::Complete));
    let before = snapshot(&scratch.0);
    assert!(
        AppendStoreV2::recover_interrupted(&root, identity(), &request, Action::RetainParent)
            .is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
    assert_state(
        &recover(&root, &request, Action::Complete),
        &main_chain(2),
        1,
    );
}

#[test]
fn recovered_child_can_append_and_reopen_with_all_state_and_resource_counters() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(0));
    let request = main_request(0);
    stage_tail(&root, &request, 164);
    let mut store = recover(&root, &request, Action::Complete);
    for height in 2..=4 {
        store.append(block(&format!("block{height}"))).unwrap();
        let expected = main_chain(height);
        assert_state(&store, &expected, height);
        assert_clean_files(&root, &expected, height);
        drop(store);
        store =
            AppendStoreV2::open_writer(&root, identity(), checkpoint(&expected, height)).unwrap();
        assert_state(&store, &expected, height);
    }
}

#[test]
fn recovered_empty_block_applies_expiry_without_changing_account_bytes() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(1);
    initialize(&root, &parent);
    let request = main_request(1);
    stage_tail(&root, &request, 1);
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &main_chain(2), 1);
    assert_eq!(
        store.chain().unwrap().ledger().account_bytes().unwrap(),
        parent.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        store.chain().unwrap().ledger().membership().root().unwrap(),
        parent.ledger().membership().root().unwrap()
    );
}

#[test]
fn exact_independent_request_selects_membership_history_even_when_accounts_match() {
    let parent = main_chain(1);
    let main = main_chain(2);
    let mut fork = parent.clone();
    fork.append(block("fork2")).unwrap();
    assert_eq!(
        main.ledger().account_bytes().unwrap(),
        fork.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        main.ledger().membership().root().unwrap(),
        fork.ledger().membership().root().unwrap()
    );
    for candidate in [&main, &fork] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &parent);
        let request = request(&parent, 0, candidate);
        stage_tail(&root, &request, request.block().encoded_len());
        stage_pending(&root, candidate, 1, 280);
        let store = recover(&root, &request, Action::Complete);
        assert_state(&store, candidate, 1);
        assert_clean_files(&root, candidate, 1);
    }
}

#[test]
fn a_complete_membership_fork_is_not_overwritten_or_discarded_by_another_request() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(1));
    fs::write(block_path(&root, 2), fixture_bytes("fork2_block")).unwrap();
    reject_both_without_mutation(&scratch, &main_request(1));
}

#[test]
fn generation_tracks_append_count_independently_of_initial_height() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(3));
    let request = main_request(3);
    stage_tail(&root, &request, 1);
    let store = recover(&root, &request, Action::Complete);
    assert_state(&store, &main_chain(4), 1);
    assert_clean_files(&root, &main_chain(4), 1);
}

#[test]
fn a_stale_request_cannot_recover_after_a_later_successful_append() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(0));
    let request = main_request(0);
    let mut store = recover(&root, &request, Action::Complete);
    store.append(block("block2")).unwrap();
    drop(store);
    reject_both_without_mutation(&scratch, &request);
}

#[test]
fn each_parent_and_candidate_pin_is_checked_before_mutation() {
    for side in ["parent", "candidate"] {
        for field in 0..6 {
            let scratch = Scratch::new();
            let root = scratch.root();
            let parent = main_chain(1);
            let candidate = main_chain(2);
            initialize(&root, &parent);
            let good = main_request(1);
            stage_tail(&root, &good, good.block().encoded_len());
            let before = snapshot(&scratch.0);
            let mut parent_pin = checkpoint(&parent, 0);
            let mut candidate_pin = checkpoint(&candidate, 1);
            let pin = if side == "parent" {
                &mut parent_pin
            } else {
                &mut candidate_pin
            };
            match field {
                0 => pin.generation += 1,
                1 => pin.chain.config_hash = changed_hash(pin.chain.config_hash),
                2 => pin.chain.genesis_hash = changed_hash(pin.chain.genesis_hash),
                3 => pin.chain.height += 1,
                4 => pin.chain.tip_hash = changed_hash(pin.chain.tip_hash),
                5 => pin.chain.ledger_root = changed_hash(pin.chain.ledger_root),
                _ => unreachable!(),
            }
            // Static inconsistencies may reject at request construction. Pins
            // that survive that layer still must reject during complete replay.
            if let Ok(request) = Request::new(parent_pin, candidate_pin, good.block().clone()) {
                reject_both_without_mutation(&scratch, &request);
            }
            assert_eq!(snapshot(&scratch.0), before, "{side} field {field}");
        }
    }
}

#[test]
fn identity_disagreement_cannot_create_or_initialize_a_missing_root() {
    let scratch = Scratch::new();
    let request = main_request(0);
    let mut wrong = identity();
    wrong.config_hash = changed_hash(wrong.config_hash);
    assert!(
        AppendStoreV2::recover_interrupted(scratch.root(), wrong, &request, Action::Complete)
            .is_err()
    );
    assert!(!scratch.root().exists());
}

#[test]
fn full_candidate_validation_rejects_valid_work_with_a_false_ledger_root() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    initialize(&root, &parent);
    let invalid = block("wrong_root1");
    let mut candidate_pin = checkpoint(&main_chain(1), 1);
    candidate_pin.chain.tip_hash = invalid.hash();
    candidate_pin.chain.ledger_root = invalid.header().ledger_root();
    let request = Request::new(checkpoint(&parent, 0), candidate_pin, invalid).unwrap();
    reject_both_without_mutation(&scratch, &request);
}

#[test]
fn full_candidate_validation_rejects_incorrect_work_target_before_any_cleanup() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    initialize(&root, &parent);
    let good = block("block1");
    let mut header = good.header().encode();
    header[148..152].copy_from_slice(&0x207f_fffeu32.to_le_bytes());
    let invalid = BlockV2::new(
        parent.config(),
        BlockHeaderV2::decode(&header).unwrap(),
        good.operations(),
    )
    .unwrap();
    let mut candidate_pin = checkpoint(&main_chain(1), 1);
    candidate_pin.chain.tip_hash = invalid.hash();
    let request = Request::new(checkpoint(&parent, 0), candidate_pin, invalid).unwrap();
    stage_tail(&root, &request, 0);
    reject_both_without_mutation(&scratch, &request);
}

#[test]
fn request_relationship_rejects_skipped_height_or_generation_and_wrong_child() {
    let parent = main_chain(1);
    let candidate = main_chain(2);
    let before_parent = checkpoint(&parent, 0);
    let before_candidate = checkpoint(&candidate, 1);
    let child = candidate.block_at(2).unwrap().clone();
    let mut skipped_generation = before_candidate;
    skipped_generation.generation = 2;
    assert!(Request::new(before_parent, skipped_generation, child.clone()).is_err());
    assert!(
        Request::new(
            before_parent,
            checkpoint(&main_chain(3), 1),
            block("block3")
        )
        .is_err()
    );
    assert!(Request::new(before_parent, before_candidate, block("fork2")).is_err());
    assert!(Request::new(checkpoint(&parent, u64::MAX), before_candidate, child).is_err());
}

#[test]
fn nonprefix_child_and_pending_files_are_preserved_for_both_actions() {
    for target in ["child", "pending"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        let path = if target == "child" {
            block_path(&root, 1)
        } else {
            stage_pending(&root, &main_chain(1), 1, 280);
            root.join("selection.pending")
        };
        let mut bytes = fs::read(&path).unwrap();
        let middle = bytes.len() / 2;
        bytes[middle] ^= 1;
        fs::write(path, bytes).unwrap();
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn extra_trailing_bytes_do_not_become_authorized_prefix_data() {
    for target in ["child", "pending"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        let path = if target == "child" {
            block_path(&root, 1)
        } else {
            stage_pending(&root, &main_chain(1), 1, 280);
            root.join("selection.pending")
        };
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(0);
        fs::write(path, bytes).unwrap();
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn pending_selector_with_missing_or_partial_child_is_unexplained_and_rejected() {
    for cut in [None, Some(0), Some(164)] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        if let Some(cut) = cut {
            stage_tail(&root, &request, cut);
        }
        stage_pending(&root, &main_chain(1), 1, 100);
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn new_selection_with_pending_or_incomplete_child_is_not_silently_repaired() {
    for shape in ["pending", "missing", "partial"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        drop(recover(&root, &request, Action::Complete));
        match shape {
            "pending" => stage_pending(&root, &main_chain(1), 1, 280),
            "missing" => fs::remove_file(block_path(&root, 1)).unwrap(),
            "partial" => stage_tail(&root, &request, 164),
            _ => unreachable!(),
        }
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn unknown_root_names_extra_heights_and_noncanonical_block_names_are_preserved() {
    for shape in ["root", "height", "alias", "directory"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, 1);
        match shape {
            "root" => fs::write(root.join("notes.txt"), b"unrelated user data").unwrap(),
            "height" => fs::write(block_path(&root, 2), fixture_bytes("block2_block")).unwrap(),
            "alias" => fs::write(root.join("blocks/1.mrb2"), b"unrelated alias").unwrap(),
            "directory" => fs::create_dir(root.join("private-data")).unwrap(),
            _ => unreachable!(),
        }
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn missing_or_corrupt_selected_history_cannot_be_hidden_by_good_staging() {
    for shape in [
        "missing-parent",
        "corrupt-parent",
        "missing-manifest",
        "corrupt-manifest",
    ] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(1));
        let request = main_request(1);
        stage_tail(&root, &request, request.block().encoded_len());
        stage_pending(&root, &main_chain(2), 1, 280);
        let path = if shape.ends_with("parent") {
            block_path(&root, 1)
        } else {
            root.join("manifest.v2")
        };
        if shape.starts_with("missing") {
            fs::remove_file(path).unwrap();
        } else {
            let mut bytes = fs::read(&path).unwrap();
            bytes[20] ^= 1;
            fs::write(path, bytes).unwrap();
        }
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn missing_partial_corrupt_or_foreign_selected_marker_is_preserved_and_rejected() {
    for shape in ["missing", "partial", "corrupt", "foreign"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        let path = root.join("selection.v2");
        match shape {
            "missing" => fs::remove_file(path).unwrap(),
            "partial" => fs::write(path, b"MSU2").unwrap(),
            "corrupt" => {
                let mut bytes = fs::read(&path).unwrap();
                bytes[279] ^= 1;
                fs::write(path, bytes).unwrap();
            }
            "foreign" => fs::write(path, selected_bytes(&main_chain(2), 1)).unwrap(),
            _ => unreachable!(),
        }
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn sparse_oversized_residual_files_are_rejected_without_truncation_or_completion() {
    for target in ["child", "pending"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        let path = if target == "child" {
            block_path(&root, 1)
        } else {
            root.join("selection.pending")
        };
        let oversized = 1u64 << 32;
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap()
            .set_len(oversized)
            .unwrap();
        let selector = fs::read(root.join("selection.v2")).unwrap();
        for action in [Action::Complete, Action::RetainParent] {
            assert!(
                AppendStoreV2::recover_interrupted(&root, identity(), &request, action).is_err()
            );
            assert_eq!(fs::metadata(&path).unwrap().len(), oversized);
            assert_eq!(fs::read(root.join("selection.v2")).unwrap(), selector);
        }
    }
}

#[test]
fn combined_parent_and_candidate_size_is_checked_before_invalid_block_payloads_are_read() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let genesis = main_chain(0);
    initialize(&root, &genesis);
    let per_block = u64::from(genesis.config().limits().max_block_bytes);
    let cap = mirror_chain::v2::MAX_CHAIN_BYTES_V2 as u64;
    assert_eq!(
        cap % per_block,
        0,
        "frozen fixture block cap divides history cap"
    );
    let parent_count = cap / per_block;
    assert!(parent_count + 1 <= mirror_chain::v2::MAX_CHAIN_BLOCKS_V2 as u64);

    // Both checksums and all selected pins are internally coherent, but they
    // cannot bless the claimed history. The old sparse file lengths alone fill
    // the history budget; accounting for the caller's complete child must fail
    // before attempting to decode the deliberately invalid zero-filled blocks.
    let mut parent_pin = checkpoint(&genesis, 0);
    parent_pin.chain.height = parent_count - 1;
    let mut old_selector = selected_bytes(&genesis, 0);
    old_selector[96..104].copy_from_slice(&parent_pin.chain.height.to_le_bytes());
    old_selector[168..172].copy_from_slice(&(parent_count as u32).to_le_bytes());
    old_selector[176..184].copy_from_slice(&cap.to_le_bytes());
    refresh_selector_checksums(&mut old_selector);
    AppendSelectionV2::decode_for(&old_selector, parent_pin).unwrap();
    fs::write(root.join("selection.v2"), &old_selector).unwrap();
    for height in 0..parent_count {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(block_path(&root, height))
            .unwrap()
            .set_len(per_block)
            .unwrap();
    }

    let empty = block("block2");
    let mut header = empty.header().encode();
    header[36..44].copy_from_slice(&parent_count.to_le_bytes());
    header[44..76].copy_from_slice(parent_pin.chain.tip_hash.as_bytes());
    let child = BlockV2::new(
        genesis.config(),
        BlockHeaderV2::decode(&header).unwrap(),
        &[],
    )
    .unwrap();
    let mut candidate_pin = parent_pin;
    candidate_pin.generation = 1;
    candidate_pin.chain.height = parent_count;
    candidate_pin.chain.tip_hash = child.hash();
    candidate_pin.chain.ledger_root = child.header().ledger_root();
    let request = Request::new(parent_pin, candidate_pin, child).unwrap();

    for action in [Action::Complete, Action::RetainParent] {
        assert!(matches!(
            AppendStoreV2::recover_interrupted(&root, identity(), &request, action),
            Err(Error::Storage(StorageErrorV2::InputTooLarge { limit, actual }))
                if limit == cap && actual > cap
        ));
        assert_eq!(fs::read(root.join("selection.v2")).unwrap(), old_selector);
        assert!(!block_path(&root, parent_count).exists());
        assert!(!root.join("selection.pending").exists());
        assert_eq!(
            fs::read_dir(root.join("blocks")).unwrap().count(),
            parent_count as usize
        );
        for height in 0..parent_count {
            assert_eq!(
                fs::metadata(block_path(&root, height)).unwrap().len(),
                per_block
            );
        }
    }
}

#[test]
fn a_checksum_correct_selected_byte_total_must_match_the_replayed_history() {
    for selected in ["parent", "candidate"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        let mut bytes = if selected == "parent" {
            selected_bytes(&main_chain(0), 0)
        } else {
            selected_bytes(&main_chain(1), 1)
        };
        let declared = u64::from_le_bytes(bytes[176..184].try_into().unwrap());
        bytes[176..184].copy_from_slice(&(declared + 1).to_le_bytes());
        refresh_selector_checksums(&mut bytes);
        let expected = if selected == "parent" {
            request.parent()
        } else {
            request.candidate()
        };
        AppendSelectionV2::decode_for(&bytes, expected).unwrap();
        fs::write(root.join("selection.v2"), bytes).unwrap();
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn symlinked_roots_directories_selected_files_and_residuals_are_never_followed() {
    for target in [
        "root",
        "blocks",
        "manifest.v2",
        "selection.v2",
        "parent",
        "child",
        "selection.pending",
    ] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        stage_pending(&root, &main_chain(1), 1, 280);
        let path = match target {
            "root" => root.clone(),
            "parent" => block_path(&root, 0),
            "child" => block_path(&root, 1),
            name => root.join(name),
        };
        let outside = scratch.0.join("retained-target");
        fs::rename(&path, &outside).unwrap();
        symlink(&outside, &path).unwrap();
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn hardlinked_selected_files_and_residuals_keep_all_aliases_on_rejection() {
    for target in [
        "manifest.v2",
        "selection.v2",
        "parent",
        "child",
        "selection.pending",
    ] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        stage_pending(&root, &main_chain(1), 1, 280);
        let path = match target {
            "parent" => block_path(&root, 0),
            "child" => block_path(&root, 1),
            name => root.join(name),
        };
        let outside = scratch.0.join("retained-hardlink");
        fs::hard_link(&path, &outside).unwrap();
        reject_both_without_mutation(&scratch, &request);
        assert_eq!(fs::metadata(path).unwrap().nlink(), 2);
        assert!(outside.exists());
    }
}

#[test]
fn directory_in_place_of_a_managed_file_is_not_deleted_or_replaced() {
    for target in ["child", "selection.pending"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        initialize(&root, &main_chain(0));
        let request = main_request(0);
        stage_tail(&root, &request, request.block().encoded_len());
        let path = if target == "child" {
            let path = block_path(&root, 1);
            fs::remove_file(&path).unwrap();
            path
        } else {
            root.join(target)
        };
        fs::create_dir(&path).unwrap();
        fs::write(path.join("do-not-delete"), b"unrelated data").unwrap();
        reject_both_without_mutation(&scratch, &request);
    }
}

#[test]
fn recovery_never_initializes_or_adopts_missing_empty_legacy_or_immutable_stores() {
    for shape in ["missing", "empty", "legacy", "immutable"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        match shape {
            "missing" => {}
            "empty" => fs::create_dir(&root).unwrap(),
            "legacy" => {
                fs::create_dir(&root).unwrap();
                fs::write(root.join("00000000000000000000.mrb"), b"legacy data").unwrap();
            }
            "immutable" => drop(StoreV2::initialize(&root, &main_chain(0)).unwrap()),
            _ => unreachable!(),
        }
        reject_both_without_mutation(&scratch, &main_request(0));
    }
}

#[test]
fn ordinary_open_and_reconcile_remain_strict_until_explicit_recovery_is_requested() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let store = AppendStoreV2::initialize(&root, &parent).unwrap();
    let request = main_request(0);
    stage_tail(&root, &request, 1);
    let before = snapshot(&scratch.0);
    assert!(store.reconcile(checkpoint(&parent, 0)).is_err());
    assert!(AppendStoreV2::open_writer(&root, identity(), checkpoint(&parent, 0)).is_err());
    assert!(AppendStoreV2::inspect(&root, identity(), checkpoint(&parent, 0)).is_err());
    assert_eq!(snapshot(&scratch.0), before);
    assert_state(
        &recover(&root, &request, Action::Complete),
        &main_chain(1),
        1,
    );
}

#[test]
fn a_live_writer_or_shared_inspectors_exclude_static_recovery() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let writer = AppendStoreV2::initialize(&root, &parent).unwrap();
    let request = main_request(0);
    let before = snapshot(&scratch.0);
    assert!(
        AppendStoreV2::recover_interrupted(&root, identity(), &request, Action::Complete).is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
    drop(writer);
    let first = AppendStoreV2::inspect(&root, identity(), checkpoint(&parent, 0)).unwrap();
    let second = AppendStoreV2::inspect(&root, identity(), checkpoint(&parent, 0)).unwrap();
    assert!(
        AppendStoreV2::recover_interrupted(&root, identity(), &request, Action::Complete).is_err()
    );
    drop(first);
    assert!(
        AppendStoreV2::recover_interrupted(&root, identity(), &request, Action::Complete).is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
    drop(second);
    assert_state(
        &recover(&root, &request, Action::Complete),
        &main_chain(1),
        1,
    );
}

#[test]
fn consuming_recovery_keeps_the_stable_root_and_returns_a_writer() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let store = AppendStoreV2::initialize(&root, &main_chain(0)).unwrap();
    let inode = fs::metadata(&root).unwrap();
    let request = main_request(0);
    stage_tail(&root, &request, 164);
    let mut store = store.recover(&request, Action::Complete).unwrap();
    assert_state(&store, &main_chain(1), 1);
    let after = fs::metadata(&root).unwrap();
    assert_eq!((inode.dev(), inode.ino()), (after.dev(), after.ino()));
    assert!(AppendStoreV2::open_writer(&root, identity(), checkpoint(&main_chain(1), 1)).is_err());
    store.append(block("block2")).unwrap();
    assert_state(&store, &main_chain(2), 2);
}

#[test]
fn consuming_recovery_rejects_read_only_or_unrelated_live_checkpoint_and_releases_lock() {
    let scratch = Scratch::new();
    let root = scratch.root();
    initialize(&root, &main_chain(0));
    let parent = main_chain(0);
    let reader = AppendStoreV2::inspect(&root, identity(), checkpoint(&parent, 0)).unwrap();
    let before = snapshot(&scratch.0);
    assert!(reader.recover(&main_request(0), Action::Complete).is_err());
    assert_eq!(snapshot(&scratch.0), before);
    let writer = AppendStoreV2::open_writer(&root, identity(), checkpoint(&parent, 0)).unwrap();
    assert!(writer.recover(&main_request(2), Action::Complete).is_err());
    assert_eq!(snapshot(&scratch.0), before);
    assert_state(
        &recover(&root, &main_request(0), Action::Complete),
        &main_chain(1),
        1,
    );
}

#[test]
fn consuming_recovery_can_retain_parent_and_then_accept_the_original_child() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(1);
    let store = AppendStoreV2::initialize(&root, &parent).unwrap();
    let request = main_request(1);
    stage_tail(&root, &request, request.block().encoded_len());
    stage_pending(&root, &main_chain(2), 1, 100);
    let mut store = store.recover(&request, Action::RetainParent).unwrap();
    assert_state(&store, &parent, 0);
    assert_clean_files(&root, &parent, 0);
    store.append(block("block2")).unwrap();
    assert_state(&store, &main_chain(2), 1);
}
