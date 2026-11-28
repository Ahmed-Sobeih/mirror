//! Durable append integration against independently frozen actual-chain inputs.
//!
//! These tests do not spawn processes. Existing isolated ownership tests cover
//! kernel/process behavior; these cases exercise the append profile's handles.
#![cfg(target_os = "linux")]

mod support_v2;

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mirror_chain::v2::{ChainErrorV2, ChainReplayErrorV2, ChainV2, ManifestIdentityV2};
use mirror_core::v2::{BlockHeaderV2, BlockV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{
    AppendCheckpointV2, AppendSelectionV2, AppendStorageErrorV2 as Error, AppendStoreV2,
    StorageErrorV2, StoreAccessV2, StoreV2,
};

use support_v2::{fixture_bytes, fixture_hash, fork_chain, main_chain};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "mirror-v2-append-integration-{}-{}",
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
        // Only the freshly created, test-owned parent is removed. No production
        // path or recovery routine uses this cleanup helper.
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

fn writer(root: &Path, chain: &ChainV2, generation: u64) -> Result<AppendStoreV2, Error> {
    AppendStoreV2::open_writer(
        root,
        chain.manifest().identity(),
        checkpoint(chain, generation),
    )
}

fn inspect(root: &Path, chain: &ChainV2, generation: u64) -> Result<AppendStoreV2, Error> {
    AppendStoreV2::inspect(
        root,
        chain.manifest().identity(),
        checkpoint(chain, generation),
    )
}

fn initialize(root: &Path, chain: &ChainV2) {
    drop(AppendStoreV2::initialize(root, chain).unwrap());
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

fn changed_hash(hash: Hash256) -> Hash256 {
    let mut bytes = *hash.as_bytes();
    bytes[0] ^= 1;
    Hash256::from_bytes(bytes)
}

fn reheader(block: &BlockV2, change: impl FnOnce(&mut [u8; 160])) -> BlockV2 {
    let mut encoded = block.header().encode();
    change(&mut encoded);
    BlockV2::new(
        main_chain(0).config(),
        BlockHeaderV2::decode(&encoded).unwrap(),
        block.operations(),
    )
    .unwrap()
}

fn assert_busy(result: Result<AppendStoreV2, Error>) {
    assert!(matches!(result, Err(Error::Storage(StorageErrorV2::Busy))));
}

fn assert_fenced(store: &AppendStoreV2) {
    assert!(store.is_fenced());
    assert!(matches!(store.chain(), Err(Error::Fenced)));
    assert!(matches!(store.selection(), Err(Error::Fenced)));
    assert!(matches!(store.checkpoint(), Err(Error::Fenced)));
}

#[test]
fn initialization_uses_a_separate_selector_and_preserves_exact_chain_bytes() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(0);
    let store = AppendStoreV2::initialize(&root, &chain).unwrap();
    assert_eq!(store.access(), StoreAccessV2::Writer);
    assert_eq!(store.root(), root);
    assert_eq!(store.chain().unwrap(), &chain);
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&chain, 0));
    assert_eq!(store.last_confirmed_checkpoint(), checkpoint(&chain, 0));
    assert!(!store.is_fenced());
    let selection = fs::read(root.join("selection.v2")).unwrap();
    assert_eq!(selection.len(), 280);
    assert_eq!(&selection[..4], b"MSU2");
    assert_eq!(
        fs::read(root.join("manifest.v2")).unwrap(),
        fixture_bytes("manifest_hex")
    );
    assert_eq!(
        fs::read(block_path(&root, 0)).unwrap(),
        fixture_bytes("block0_block")
    );
    assert!(!root.join("selection.pending").exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 3);
    drop(store);
    assert_eq!(writer(&root, &chain, 0).unwrap().chain().unwrap(), &chain);
}

#[test]
fn initialization_at_a_nonzero_height_still_starts_generation_zero() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(3);
    let mut store = AppendStoreV2::initialize(&root, &chain).unwrap();
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&chain, 0));
    assert_eq!(
        store.append(block("block4")).unwrap(),
        checkpoint(&main_chain(4), 1)
    );
    drop(store);
    assert!(inspect(&root, &main_chain(4), 4).is_err());
    assert_eq!(
        writer(&root, &main_chain(4), 1).unwrap().chain().unwrap(),
        &main_chain(4)
    );
}

#[test]
fn every_append_reopens_to_exact_mixed_accounts_membership_and_budget_state() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut store = AppendStoreV2::initialize(&root, &main_chain(0)).unwrap();
    let initial_inode = fs::metadata(&root).unwrap();
    for height in 1..=4 {
        let expected = main_chain(height);
        let result = store.append(block(&format!("block{height}"))).unwrap();
        assert_eq!(result, checkpoint(&expected, height));
        assert_eq!(store.chain().unwrap(), &expected);
        assert_eq!(store.last_confirmed_checkpoint(), result);
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
            fixture_bytes(&format!("state{height}_accounts_hex")),
        );
        assert_eq!(
            store
                .chain()
                .unwrap()
                .ledger()
                .membership()
                .encode()
                .unwrap(),
            fixture_bytes(&format!("state{height}_membership_hex")),
        );
        let membership = store.chain().unwrap().ledger().membership();
        assert_eq!(
            membership
                .record(fixture_hash("c_member_id"))
                .unwrap()
                .inclusion()
                .operation_index,
            3
        );
        assert_eq!(
            membership
                .record(fixture_hash("d_member_id"))
                .unwrap()
                .inclusion()
                .operation_index,
            6
        );
        for selected_height in 0..=height {
            assert_eq!(
                fs::read(block_path(&root, selected_height)).unwrap(),
                fixture_bytes(&format!("block{selected_height}_block")),
            );
        }
        assert!(!root.join("selection.pending").exists());
        let current_inode = fs::metadata(&root).unwrap();
        assert_eq!(
            (initial_inode.dev(), initial_inode.ino()),
            (current_inode.dev(), current_inode.ino())
        );
        drop(store);
        store = writer(&root, &expected, height).unwrap();
        assert_eq!(store.chain().unwrap(), &expected);
    }
    assert_eq!(store.chain().unwrap().ledger().initial_test_supply(), 1000);
    assert_eq!(
        store.chain().unwrap().ledger().membership().record_count(),
        4
    );
    assert_eq!(
        store.chain().unwrap().ledger().membership().active_count(),
        1
    );
}

#[test]
fn empty_appended_block_executes_expiry_and_keeps_accounts_unchanged() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(1);
    let next = main_chain(2);
    assert!(block("block2").operations().is_empty());
    assert_ne!(
        parent.ledger().membership().root().unwrap(),
        next.ledger().membership().root().unwrap()
    );
    let mut store = AppendStoreV2::initialize(&root, &parent).unwrap();
    let accounts = parent.ledger().account_bytes().unwrap();
    store.append(block("block2")).unwrap();
    assert_eq!(
        store.chain().unwrap().ledger().account_bytes().unwrap(),
        accounts
    );
    assert_eq!(store.chain().unwrap(), &next);
    drop(store);
    assert_eq!(inspect(&root, &next, 1).unwrap().chain().unwrap(), &next);
}

#[test]
fn rejected_height_parent_and_time_do_not_change_files_or_fence_the_writer() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let mut store = AppendStoreV2::initialize(&root, &parent).unwrap();
    let snapshot_before = snapshot(&scratch.0);
    let child = block("block1");
    let cases = [
        (
            block("block2"),
            ChainErrorV2::WrongHeight {
                expected: 1,
                got: 2,
            },
        ),
        (
            reheader(&child, |bytes| bytes[44..76].copy_from_slice(&[0x88; 32])),
            ChainErrorV2::WrongParent,
        ),
        (
            reheader(&child, |bytes| {
                bytes[140..148].copy_from_slice(&0u64.to_le_bytes())
            }),
            ChainErrorV2::InvalidTimestamp { parent: 0, got: 0 },
        ),
    ];
    for (candidate, expected) in cases {
        assert!(matches!(store.append(candidate), Err(Error::Chain(error)) if error == expected));
        assert!(!store.is_fenced());
        assert_eq!(store.chain().unwrap(), &parent);
        assert_eq!(store.checkpoint().unwrap(), checkpoint(&parent, 0));
        assert_eq!(snapshot(&scratch.0), snapshot_before);
    }
    store.append(child).unwrap();
    assert_eq!(store.chain().unwrap(), &main_chain(1));
}

#[test]
fn valid_work_with_a_false_root_cannot_publish_partial_mixed_execution() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let mut store = AppendStoreV2::initialize(&root, &parent).unwrap();
    let before = snapshot(&scratch.0);
    assert!(matches!(
        store.append(block("wrong_root1")),
        Err(Error::Chain(ChainErrorV2::WrongLedgerRoot))
    ));
    assert_eq!(store.chain().unwrap(), &parent);
    assert!(!store.is_fenced());
    assert_eq!(snapshot(&scratch.0), before);
    store.append(block("block1")).unwrap();
    assert_eq!(store.chain().unwrap(), &main_chain(1));
}

#[test]
fn repeated_already_selected_block_is_rejected_without_mutation() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut store = AppendStoreV2::initialize(&root, &main_chain(0)).unwrap();
    store.append(block("block1")).unwrap();
    let before = snapshot(&scratch.0);
    assert!(matches!(
        store.append(block("block1")),
        Err(Error::Chain(ChainErrorV2::WrongHeight {
            expected: 2,
            got: 1
        }))
    ));
    assert_eq!(store.chain().unwrap(), &main_chain(1));
    assert_eq!(snapshot(&scratch.0), before);
    store.append(block("block2")).unwrap();
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&main_chain(2), 2));
}

#[test]
fn inspection_is_read_only_and_cannot_append() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(0);
    initialize(&root, &chain);
    let before = snapshot(&scratch.0);
    let mut reader = inspect(&root, &chain, 0).unwrap();
    assert_eq!(reader.access(), StoreAccessV2::ReadOnly);
    assert!(matches!(
        reader.append(block("block1")),
        Err(Error::ReadOnly)
    ));
    assert_eq!(reader.chain().unwrap(), &chain);
    assert!(!reader.is_fenced());
    drop(reader);
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn writer_excludes_other_handles_before_and_after_append() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut store = AppendStoreV2::initialize(&root, &main_chain(0)).unwrap();
    assert_busy(writer(&root, &main_chain(0), 0));
    assert_busy(inspect(&root, &main_chain(0), 0));
    store.append(block("block1")).unwrap();
    assert_busy(writer(&root, &main_chain(1), 1));
    assert_busy(inspect(&root, &main_chain(1), 1));
    drop(store);
    assert_eq!(
        writer(&root, &main_chain(1), 1).unwrap().chain().unwrap(),
        &main_chain(1)
    );
}

#[test]
fn shared_inspectors_exclude_writer_until_all_are_dropped() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(1);
    initialize(&root, &chain);
    let first = inspect(&root, &chain, 0).unwrap();
    let second = inspect(&root, &chain, 0).unwrap();
    assert_busy(writer(&root, &chain, 0));
    drop(first);
    assert_busy(writer(&root, &chain, 0));
    drop(second);
    assert_eq!(writer(&root, &chain, 0).unwrap().chain().unwrap(), &chain);
}

#[test]
fn clean_reconciliation_retains_root_ownership_and_allows_the_next_append() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(0);
    let store = AppendStoreV2::initialize(&root, &chain).unwrap();
    let inode = fs::metadata(&root).unwrap();
    let before = snapshot(&scratch.0);
    let store = store.reconcile(checkpoint(&chain, 0)).unwrap();
    let mut store = store.reconcile(checkpoint(&chain, 0)).unwrap();
    assert_eq!(store.chain().unwrap(), &chain);
    assert_busy(writer(&root, &chain, 0));
    assert_eq!(snapshot(&scratch.0), before);
    let current_inode = fs::metadata(&root).unwrap();
    assert_eq!(
        (inode.dev(), inode.ino()),
        (current_inode.dev(), current_inode.ino())
    );
    store.append(block("block1")).unwrap();
    drop(store);
    assert_eq!(
        writer(&root, &main_chain(1), 1).unwrap().chain().unwrap(),
        &main_chain(1)
    );
}

#[test]
fn failed_reconciliation_preserves_files_and_releases_consumed_ownership() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(0);
    let store = AppendStoreV2::initialize(&root, &chain).unwrap();
    let before = snapshot(&scratch.0);
    assert!(store.reconcile(checkpoint(&main_chain(1), 1)).is_err());
    assert_eq!(snapshot(&scratch.0), before);
    assert_eq!(writer(&root, &chain, 0).unwrap().chain().unwrap(), &chain);
}

#[test]
fn independent_identity_disagreement_is_rejected_before_accessing_a_missing_path() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(0);
    let mut identity = chain.manifest().identity();
    identity.config_hash = changed_hash(identity.config_hash);
    assert!(matches!(
        AppendStoreV2::inspect(&root, identity, checkpoint(&chain, 0)),
        Err(Error::Storage(StorageErrorV2::IdentityMismatch))
    ));
    assert!(!root.exists());
}

#[test]
fn all_network_branch_and_generation_pins_are_required_on_reopening() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(3);
    initialize(&root, &chain);
    let before = snapshot(&scratch.0);
    for field in 0..6 {
        let mut expected = checkpoint(&chain, 0);
        match field {
            0 => expected.generation = 1,
            1 => expected.chain.config_hash = changed_hash(expected.chain.config_hash),
            2 => expected.chain.genesis_hash = changed_hash(expected.chain.genesis_hash),
            3 => expected.chain.height = 2,
            4 => expected.chain.tip_hash = changed_hash(expected.chain.tip_hash),
            5 => expected.chain.ledger_root = changed_hash(expected.chain.ledger_root),
            _ => unreachable!(),
        }
        let identity = ManifestIdentityV2 {
            config_hash: expected.chain.config_hash,
            genesis_hash: expected.chain.genesis_hash,
        };
        assert!(
            AppendStoreV2::inspect(&root, identity, expected).is_err(),
            "field {field}"
        );
        assert!(
            AppendStoreV2::open_writer(&root, identity, expected).is_err(),
            "field {field}"
        );
    }
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn an_old_generation_cannot_reopen_a_successfully_appended_store() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut store = AppendStoreV2::initialize(&root, &main_chain(0)).unwrap();
    store.append(block("block1")).unwrap();
    drop(store);
    let before = snapshot(&scratch.0);
    assert!(inspect(&root, &main_chain(0), 0).is_err());
    assert!(inspect(&root, &main_chain(1), 0).is_err());
    assert!(inspect(&root, &main_chain(1), 2).is_err());
    assert_eq!(snapshot(&scratch.0), before);
    assert_eq!(
        writer(&root, &main_chain(1), 1).unwrap().chain().unwrap(),
        &main_chain(1)
    );
}

#[test]
fn same_height_account_identical_membership_fork_does_not_satisfy_selection() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let main = main_chain(3);
    let fork = fork_chain();
    assert_eq!(
        main.ledger().account_bytes().unwrap(),
        fork.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        main.ledger().membership().root().unwrap(),
        fork.ledger().membership().root().unwrap()
    );
    initialize(&root, &main);
    let before = snapshot(&scratch.0);
    assert!(writer(&root, &fork, 0).is_err());
    assert!(inspect(&root, &fork, 0).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn storage_profiles_cannot_be_silently_reinterpreted_in_either_direction() {
    let scratch = Scratch::new();
    let immutable_root = scratch.0.join("immutable");
    let append_root = scratch.0.join("append");
    let chain = main_chain(0);
    drop(StoreV2::initialize(&immutable_root, &chain).unwrap());
    initialize(&append_root, &chain);
    let before = snapshot(&scratch.0);
    assert!(writer(&immutable_root, &chain, 0).is_err());
    assert!(AppendStoreV2::initialize(&immutable_root, &chain).is_err());
    assert!(
        StoreV2::open_writer(
            &append_root,
            chain.manifest().identity(),
            chain.checkpoint()
        )
        .is_err()
    );
    assert!(StoreV2::initialize(&append_root, &chain).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn missing_empty_foreign_and_legacy_directories_are_never_initialized_by_open() {
    for layout in ["missing", "empty", "foreign", "legacy"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        if layout != "missing" {
            fs::create_dir(&root).unwrap();
        }
        if layout == "foreign" {
            fs::write(root.join("notes.txt"), b"retain unrelated data").unwrap();
        } else if layout == "legacy" {
            fs::write(root.join("00000000000000000000.mrb"), b"legacy data").unwrap();
        }
        let before = snapshot(&scratch.0);
        assert!(writer(&root, &main_chain(0), 0).is_err(), "{layout}");
        assert!(inspect(&root, &main_chain(0), 0).is_err(), "{layout}");
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn missing_selected_block_is_not_repaired_or_replaced() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(3);
    initialize(&root, &chain);
    fs::remove_file(block_path(&root, 2)).unwrap();
    let before = snapshot(&scratch.0);
    assert!(writer(&root, &chain, 0).is_err());
    assert!(inspect(&root, &chain, 0).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn unselected_tail_pending_marker_and_unknown_files_are_retained_and_rejected() {
    for extra in ["tail", "pending", "root-note", "block-note"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let chain = main_chain(3);
        initialize(&root, &chain);
        let path = match extra {
            "tail" => block_path(&root, 4),
            "pending" => root.join("selection.pending"),
            "root-note" => root.join("notes.txt"),
            "block-note" => root.join("blocks/notes.txt"),
            _ => unreachable!(),
        };
        fs::write(path, fixture_bytes("block4_block")).unwrap();
        let before = snapshot(&scratch.0);
        assert!(inspect(&root, &chain, 0).is_err(), "{extra}");
        assert!(writer(&root, &chain, 0).is_err(), "{extra}");
        assert!(AppendStoreV2::initialize(&root, &chain).is_err(), "{extra}");
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn unpublished_selection_is_not_promoted_from_a_pending_file() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(1);
    initialize(&root, &chain);
    fs::rename(root.join("selection.v2"), root.join("selection.pending")).unwrap();
    let before = snapshot(&scratch.0);
    assert!(writer(&root, &chain, 0).is_err());
    assert!(inspect(&root, &chain, 0).is_err());
    assert!(AppendStoreV2::initialize(&root, &chain).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn corrupt_truncated_and_oversized_selection_never_fall_back_to_a_prior_state() {
    for mode in ["corrupt", "short", "long"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let chain = main_chain(1);
        initialize(&root, &chain);
        let path = root.join("selection.v2");
        let mut bytes = fs::read(&path).unwrap();
        match mode {
            "corrupt" => bytes[90] ^= 1,
            "short" => {
                bytes.pop();
            }
            "long" => bytes.push(0),
            _ => unreachable!(),
        }
        fs::write(&path, bytes).unwrap();
        let before = snapshot(&scratch.0);
        assert!(writer(&root, &chain, 0).is_err(), "{mode}");
        assert!(inspect(&root, &chain, 0).is_err(), "{mode}");
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn stale_checksum_valid_selector_with_new_tail_rejects_both_checkpoints() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let mut store = AppendStoreV2::initialize(&root, &parent).unwrap();
    let old_selector = fs::read(root.join("selection.v2")).unwrap();
    store.append(block("block1")).unwrap();
    drop(store);
    fs::write(root.join("selection.v2"), old_selector).unwrap();
    let before = snapshot(&scratch.0);
    assert!(writer(&root, &parent, 0).is_err());
    assert!(writer(&root, &main_chain(1), 1).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn invalid_intermediate_ledger_root_is_detected_by_complete_replay() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(4);
    initialize(&root, &chain);
    fs::write(block_path(&root, 1), fixture_bytes("wrong_root1_block")).unwrap();
    let before = snapshot(&scratch.0);
    assert!(matches!(
        writer(&root, &chain, 0),
        Err(Error::Storage(StorageErrorV2::Replay(
            ChainReplayErrorV2::Block {
                index: 1,
                error: ChainErrorV2::WrongLedgerRoot
            }
        )))
    ));
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn tampered_manifest_and_trailing_block_bytes_are_rejected_without_repair() {
    for file in ["manifest", "block"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let chain = main_chain(1);
        initialize(&root, &chain);
        let path = if file == "manifest" {
            root.join("manifest.v2")
        } else {
            block_path(&root, 1)
        };
        let mut bytes = fs::read(&path).unwrap();
        if file == "manifest" {
            bytes[20] ^= 1;
        } else {
            bytes.push(0);
        }
        fs::write(path, bytes).unwrap();
        let before = snapshot(&scratch.0);
        assert!(inspect(&root, &chain, 0).is_err());
        assert!(writer(&root, &chain, 0).is_err());
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn oversized_block_file_is_bounded_before_payload_read() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(1);
    initialize(&root, &chain);
    let path = block_path(&root, 1);
    let oversized = u64::from(chain.config().limits().max_block_bytes) + 1;
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(oversized)
        .unwrap();
    assert!(matches!(
        writer(&root, &chain, 0),
        Err(Error::Storage(StorageErrorV2::InputTooLarge { .. }))
    ));
    assert_eq!(fs::metadata(&path).unwrap().len(), oversized);
}

#[test]
fn aggregate_block_metadata_is_bounded_before_any_invalid_payload_is_decoded() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let chain = main_chain(0);
    initialize(&root, &chain);
    let per_block = u64::from(chain.config().limits().max_block_bytes);
    let aggregate_limit = mirror_chain::v2::MAX_CHAIN_BYTES_V2 as u64;
    let count = aggregate_limit / per_block + 1;
    assert!(count <= mirror_chain::v2::MAX_CHAIN_BLOCKS_V2 as u64);

    // The selector is deliberately checksum-correct but describes untrusted
    // claimed history. Its declared total is legal, while the checked sparse
    // file lengths exceed the aggregate limit. Zero-filled MRB2 data would fail
    // decoding, so observing the size error proves preflight precedes decoding.
    let mut expected = checkpoint(&chain, 0);
    expected.chain.height = count - 1;
    let mut selector = fs::read(root.join("selection.v2")).unwrap();
    selector[96..104].copy_from_slice(&expected.chain.height.to_le_bytes());
    selector[168..172].copy_from_slice(&(count as u32).to_le_bytes());
    selector[176..184].copy_from_slice(&aggregate_limit.to_le_bytes());
    let mut inner_input = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    inner_input.extend_from_slice(&selector[16..216]);
    selector[216..248].copy_from_slice(sha256d(&inner_input).as_bytes());
    let mut outer_input = b"MIRROR_STORAGE_APPEND_SELECTION_V2".to_vec();
    outer_input.extend_from_slice(&selector[..248]);
    selector[248..280].copy_from_slice(sha256d(&outer_input).as_bytes());
    AppendSelectionV2::decode_for(&selector, expected).unwrap();
    fs::write(root.join("selection.v2"), &selector).unwrap();
    for height in 0..count {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(block_path(&root, height))
            .unwrap()
            .set_len(per_block)
            .unwrap();
    }
    assert!(matches!(
        AppendStoreV2::open_writer(&root, chain.manifest().identity(), expected),
        Err(Error::Storage(StorageErrorV2::InputTooLarge { limit, actual }))
            if limit == aggregate_limit && actual == count * per_block
    ));
    assert_eq!(fs::read(root.join("selection.v2")).unwrap(), selector);
    assert_eq!(
        fs::read_dir(root.join("blocks")).unwrap().count(),
        count as usize
    );
    for height in 0..count {
        assert_eq!(
            fs::metadata(block_path(&root, height)).unwrap().len(),
            per_block
        );
    }
}

#[test]
fn symlinked_roots_selected_files_and_blocks_directories_are_rejected() {
    for alias in ["root", "manifest.v2", "selection.v2", "blocks", "block"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let chain = main_chain(1);
        initialize(&root, &chain);
        let selected = match alias {
            "root" => root.clone(),
            "block" => block_path(&root, 1),
            name => root.join(name),
        };
        let moved = scratch.0.join("retained-target");
        fs::rename(&selected, &moved).unwrap();
        symlink(&moved, &selected).unwrap();
        let before = snapshot(&scratch.0);
        assert!(
            matches!(
                writer(&root, &chain, 0),
                Err(Error::Storage(StorageErrorV2::UnsafePath(_)))
            ),
            "{alias}"
        );
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn hardlinked_selected_files_are_rejected_without_removing_aliases() {
    for selected in ["manifest", "selection", "block"] {
        let scratch = Scratch::new();
        let root = scratch.root();
        let chain = main_chain(1);
        initialize(&root, &chain);
        let source = match selected {
            "manifest" => root.join("manifest.v2"),
            "selection" => root.join("selection.v2"),
            _ => block_path(&root, 1),
        };
        let alias = scratch.0.join("outside-hardlink");
        fs::hard_link(&source, &alias).unwrap();
        let before = snapshot(&scratch.0);
        assert!(
            matches!(
                writer(&root, &chain, 0),
                Err(Error::Storage(StorageErrorV2::UnsafePath(_)))
            ),
            "{selected}"
        );
        assert_eq!(snapshot(&scratch.0), before);
        assert_eq!(fs::metadata(source).unwrap().nlink(), 2);
    }
}

#[test]
fn live_disk_mismatch_fences_all_state_access_and_preserves_confirmed_checkpoint() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(0);
    let mut store = AppendStoreV2::initialize(&root, &parent).unwrap();
    fs::write(root.join("unexplained.txt"), b"must not be discarded").unwrap();
    let before = snapshot(&scratch.0);
    assert!(store.append(block("block1")).is_err());
    assert_fenced(&store);
    assert_eq!(store.last_confirmed_checkpoint(), checkpoint(&parent, 0));
    assert!(matches!(store.append(block("block1")), Err(Error::Fenced)));
    assert_eq!(snapshot(&scratch.0), before);
    assert!(store.reconcile(checkpoint(&parent, 0)).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn changed_selected_bytes_are_not_hidden_by_the_in_memory_chain() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let parent = main_chain(1);
    let mut store = AppendStoreV2::initialize(&root, &parent).unwrap();
    fs::write(block_path(&root, 1), fixture_bytes("wrong_root1_block")).unwrap();
    let before = snapshot(&scratch.0);
    assert!(store.append(block("block2")).is_err());
    assert_fenced(&store);
    assert_eq!(store.last_confirmed_checkpoint(), checkpoint(&parent, 0));
    assert_eq!(snapshot(&scratch.0), before);
    drop(store);
    assert!(writer(&root, &parent, 0).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}
