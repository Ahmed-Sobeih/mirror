//! Filesystem integration for the explicit immutable V2 selected-history store.
//!
//! All histories are the existing independent public synthetic chain vectors.
//! Temporary directories belong only to these tests; production failures must
//! preserve partial initialization and never repair unrelated user files.

#![cfg(target_os = "linux")]

mod support_v2;

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mirror_chain::v2::{ChainErrorV2, ChainReplayErrorV2, ChainV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{SelectionRecordV2, StorageErrorV2, StoreAccessV2, StoreV2};

use support_v2::{fixture_bytes, fixture_hash, fork_chain, main_chain};

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        for _ in 0..100 {
            let number = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "mirror-v2-storage-integration-{}-{number}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("cannot create test directory: {error}"),
            }
        }
        panic!("test directory namespace exhausted");
    }

    fn store(&self) -> PathBuf {
        self.0.join("store")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        // This path was atomically created by this individual test. It is not
        // an application data directory and no recovery code calls this helper.
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

fn snapshot(path: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        if metadata.file_type().is_symlink() {
            entries.insert(relative, Entry::Symlink(fs::read_link(path).unwrap()));
        } else if metadata.is_dir() {
            entries.insert(relative, Entry::Directory);
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), entries);
            }
        } else if metadata.is_file() {
            entries.insert(relative, Entry::File(fs::read(path).unwrap()));
        } else {
            entries.insert(relative, Entry::Other);
        }
    }

    let mut result = BTreeMap::new();
    visit(path, path, &mut result);
    result
}

fn block_path(root: &Path, height: u64) -> PathBuf {
    root.join("blocks").join(format!("{height:020}.mrb2"))
}

fn initialize(root: &Path, chain: &ChainV2) {
    drop(StoreV2::initialize(root, chain).unwrap());
}

fn inspect(root: &Path, chain: &ChainV2) -> Result<StoreV2, StorageErrorV2> {
    StoreV2::inspect(root, chain.manifest().identity(), chain.checkpoint())
}

fn open_writer(root: &Path, chain: &ChainV2) -> Result<StoreV2, StorageErrorV2> {
    StoreV2::open_writer(root, chain.manifest().identity(), chain.checkpoint())
}

fn error(result: Result<StoreV2, StorageErrorV2>) -> StorageErrorV2 {
    match result {
        Err(error) => error,
        Ok(_) => panic!("untrusted or unavailable store unexpectedly opened"),
    }
}

fn changed_hash(hash: Hash256) -> Hash256 {
    let mut bytes = *hash.as_bytes();
    bytes[0] ^= 1;
    Hash256::from_bytes(bytes)
}

#[test]
fn genesis_initialization_and_checked_writer_reopen_preserve_exact_bytes() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    let initialized = StoreV2::initialize(&root, &chain).unwrap();
    assert_eq!(initialized.access(), StoreAccessV2::Writer);
    assert_eq!(initialized.chain(), &chain);
    assert_eq!(initialized.selection().checkpoint(), chain.checkpoint());
    assert_eq!(initialized.root(), root);
    assert_eq!(
        fs::read(root.join("manifest.v2")).unwrap(),
        fixture_bytes("manifest_hex")
    );
    assert_eq!(
        fs::read(block_path(&root, 0)).unwrap(),
        fixture_bytes("block0_block")
    );
    assert_eq!(fs::metadata(root.join("selection.v2")).unwrap().len(), 232);
    assert!(!root.join("selection.pending").exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 3);
    drop(initialized);

    let before = snapshot(&temp.0);
    let reopened = open_writer(&root, &chain).unwrap();
    assert_eq!(reopened.access(), StoreAccessV2::Writer);
    assert_eq!(reopened.chain(), &chain);
    drop(reopened);
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn complete_selected_history_replays_transfers_membership_and_withdrawals() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(4);
    initialize(&root, &chain);
    let reopened = open_writer(&root, &chain).unwrap();
    assert_eq!(reopened.chain(), &chain);
    assert_eq!(reopened.chain().blocks().len(), 5);
    assert_eq!(reopened.chain().encoded_bytes(), chain.encoded_bytes());
    assert_eq!(
        reopened.chain().state_items_used(),
        chain.state_items_used()
    );
    assert_eq!(reopened.chain().ledger().initial_test_supply(), 1000);
    assert_eq!(reopened.chain().ledger().membership().active_count(), 1);
    assert_eq!(
        reopened.chain().ledger().account_bytes().unwrap(),
        fixture_bytes("state4_accounts_hex")
    );
    assert_eq!(
        reopened.chain().ledger().membership().encode().unwrap(),
        fixture_bytes("state4_membership_hex")
    );
    for height in 0..=4 {
        assert_eq!(
            fs::read(block_path(&root, height)).unwrap(),
            fixture_bytes(&format!("block{height}_block"))
        );
    }
}

#[test]
fn distinct_membership_branches_remain_independently_selected() {
    let temp = TestDirectory::new();
    let main = main_chain(3);
    let fork = fork_chain();
    let main_root = temp.0.join("main");
    let fork_root = temp.0.join("fork");
    initialize(&main_root, &main);
    initialize(&fork_root, &fork);
    let reopened_main = inspect(&main_root, &main).unwrap();
    let reopened_fork = inspect(&fork_root, &fork).unwrap();
    assert_eq!(reopened_main.chain(), &main);
    assert_eq!(reopened_fork.chain(), &fork);
    assert_eq!(main.height(), fork.height());
    assert_eq!(
        main.ledger().account_bytes().unwrap(),
        fork.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        main.ledger().membership().root().unwrap(),
        fork.ledger().membership().root().unwrap()
    );
    assert_eq!(main.ledger().membership().active_count(), 3);
    // Fork block2 withdraws reviewer A. At fork block3 only B is eligible, so
    // applicant C remains pending instead of becoming the main branch's third
    // active member. The sole active member on this fork is seed B.
    assert_eq!(fork.ledger().membership().active_count(), 1);
    assert!(matches!(
        error(inspect(&main_root, &fork)),
        StorageErrorV2::Selection(_)
    ));
    assert!(matches!(
        error(inspect(&fork_root, &main)),
        StorageErrorV2::Selection(_)
    ));
}

#[test]
fn read_only_inspection_keeps_all_file_names_and_contents_unchanged() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(4);
    initialize(&root, &chain);
    let before = snapshot(&temp.0);
    let read_only = inspect(&root, &chain).unwrap();
    assert_eq!(read_only.access(), StoreAccessV2::ReadOnly);
    assert_eq!(read_only.chain(), &chain);
    assert_eq!(snapshot(&temp.0), before);
    drop(read_only);
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn missing_and_empty_directories_never_initialize_during_open_or_inspection() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::MissingDirectory
    ));
    assert!(matches!(
        error(open_writer(&root, &chain)),
        StorageErrorV2::MissingDirectory
    ));
    assert!(!root.exists());

    fs::create_dir(&root).unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::EmptyDirectory
    ));
    assert!(matches!(
        error(open_writer(&root, &chain)),
        StorageErrorV2::EmptyDirectory
    ));
    assert_eq!(snapshot(&temp.0), before);
    initialize(&root, &chain);
    assert_eq!(inspect(&root, &chain).unwrap().chain(), &chain);
}

#[test]
fn explicit_initialization_does_not_create_missing_parent_directories() {
    let temp = TestDirectory::new();
    let absent_parent = temp.0.join("absent-parent");
    let result = StoreV2::initialize(absent_parent.join("store"), &main_chain(0));
    assert!(result.is_err());
    assert!(!absent_parent.exists());
}

#[test]
fn existing_regular_file_is_never_treated_as_a_store_directory() {
    let temp = TestDirectory::new();
    let root = temp.store();
    fs::write(&root, b"unrelated user file").unwrap();
    let chain = main_chain(0);
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::NotDirectory
    ));
    assert!(matches!(
        error(StoreV2::initialize(&root, &chain)),
        StorageErrorV2::NotDirectory
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn foreign_directory_is_rejected_without_touching_user_files() {
    let temp = TestDirectory::new();
    let root = temp.store();
    fs::create_dir(&root).unwrap();
    fs::write(root.join("important.txt"), b"preserve unrelated user data").unwrap();
    let chain = main_chain(0);
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::ForeignDirectory
    ));
    assert!(matches!(
        error(open_writer(&root, &chain)),
        StorageErrorV2::ForeignDirectory
    ));
    assert!(matches!(
        error(StoreV2::initialize(&root, &chain)),
        StorageErrorV2::ForeignDirectory
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn legacy_directory_and_recovery_siblings_are_preserved_without_migration() {
    let temp = TestDirectory::new();
    let root = temp.store();
    fs::create_dir(&root).unwrap();
    fs::write(
        root.join("00000000000000000000.mrb"),
        b"legacy bytes remain untouched",
    )
    .unwrap();
    for suffix in ["backup", "staging"] {
        let directory = temp.0.join(format!(".store.mirror-reorg-{suffix}"));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("retained"), suffix.as_bytes()).unwrap();
    }
    let chain = main_chain(0);
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::LegacyDirectory
    ));
    assert!(matches!(
        error(open_writer(&root, &chain)),
        StorageErrorV2::LegacyDirectory
    ));
    assert!(matches!(
        error(StoreV2::initialize(&root, &chain)),
        StorageErrorV2::LegacyDirectory
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn missing_store_does_not_restore_or_remove_legacy_backup_siblings() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let backup = temp.0.join(".store.mirror-reorg-backup");
    fs::create_dir(&backup).unwrap();
    fs::write(backup.join("retained"), b"do not restore or delete").unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &main_chain(0))),
        StorageErrorV2::MissingDirectory
    ));
    assert!(!root.exists());
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn partial_initialization_is_not_repaired_deleted_or_reinitialized() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(3);
    initialize(&root, &chain);
    fs::rename(root.join("selection.v2"), root.join("selection.pending")).unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::PartialInitialization
    ));
    assert!(matches!(
        error(open_writer(&root, &chain)),
        StorageErrorV2::PartialInitialization
    ));
    assert!(matches!(
        error(StoreV2::initialize(&root, &chain)),
        StorageErrorV2::PartialInitialization
    ));
    assert_eq!(snapshot(&temp.0), before);
    assert!(!root.join("selection.v2").exists());
}

#[test]
fn committed_history_cannot_be_silently_reinitialized() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(4);
    initialize(&root, &chain);
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(StoreV2::initialize(&root, &main_chain(0))),
        StorageErrorV2::AlreadyInitialized
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn independent_identity_must_agree_with_the_supplied_checkpoint_before_io() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    let mut identity = chain.manifest().identity();
    identity.config_hash = changed_hash(identity.config_hash);
    assert!(matches!(
        error(StoreV2::inspect(&root, identity, chain.checkpoint())),
        StorageErrorV2::IdentityMismatch
    ));
    assert!(!root.exists());
}

#[test]
fn every_external_checkpoint_field_is_required_and_never_selected_from_disk() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(3);
    initialize(&root, &chain);
    let before = snapshot(&temp.0);
    for field in 0..5 {
        let mut expected = chain.checkpoint();
        match field {
            0 => expected.config_hash = changed_hash(expected.config_hash),
            1 => expected.genesis_hash = changed_hash(expected.genesis_hash),
            2 => expected.height -= 1,
            3 => expected.tip_hash = changed_hash(expected.tip_hash),
            4 => expected.ledger_root = changed_hash(expected.ledger_root),
            _ => unreachable!(),
        }
        let identity = mirror_chain::v2::ManifestIdentityV2 {
            config_hash: expected.config_hash,
            genesis_hash: expected.genesis_hash,
        };
        assert!(matches!(
            error(StoreV2::inspect(&root, identity, expected)),
            StorageErrorV2::Selection(_)
        ));
    }
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn a_valid_older_prefix_does_not_satisfy_a_later_selected_checkpoint() {
    let temp = TestDirectory::new();
    let root = temp.store();
    initialize(&root, &main_chain(1));
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &main_chain(4))),
        StorageErrorV2::Selection(_)
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn missing_selected_block_rejects_without_resetting_or_repairing() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(3);
    initialize(&root, &chain);
    fs::remove_file(block_path(&root, 2)).unwrap();
    let before = snapshot(&temp.0);
    assert!(inspect(&root, &chain).is_err());
    assert!(open_writer(&root, &chain).is_err());
    assert_eq!(snapshot(&temp.0), before);
    assert!(!block_path(&root, 2).exists());
}

#[test]
fn unselected_tail_and_unexpected_files_are_rejected_and_retained() {
    for extra in ["tail", "root-file", "block-file"] {
        let temp = TestDirectory::new();
        let root = temp.store();
        let chain = main_chain(3);
        initialize(&root, &chain);
        let path = match extra {
            "tail" => block_path(&root, 4),
            "root-file" => root.join("user-note.txt"),
            "block-file" => root.join("blocks").join("user-note.txt"),
            _ => unreachable!(),
        };
        fs::write(path, fixture_bytes("block4_block")).unwrap();
        let before = snapshot(&temp.0);
        assert!(matches!(
            error(inspect(&root, &chain)),
            StorageErrorV2::InvalidLayout(_)
        ));
        assert_eq!(snapshot(&temp.0), before);
    }
}

#[test]
fn stale_pending_marker_beside_committed_selection_is_not_silently_removed() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    initialize(&root, &chain);
    fs::write(root.join("selection.pending"), b"unresolved content").unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::InvalidLayout(_)
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn truncated_and_corrupt_selection_markers_do_not_fall_back_to_a_new_chain() {
    for corrupt in [false, true] {
        let temp = TestDirectory::new();
        let root = temp.store();
        let chain = main_chain(0);
        initialize(&root, &chain);
        let path = root.join("selection.v2");
        let mut bytes = fs::read(&path).unwrap();
        if corrupt {
            bytes[20] ^= 1;
        } else {
            bytes.truncate(bytes.len() - 1);
        }
        fs::write(&path, bytes).unwrap();
        let before = snapshot(&temp.0);
        assert!(inspect(&root, &chain).is_err());
        assert_eq!(snapshot(&temp.0), before);
    }
}

#[test]
fn an_oversized_selection_marker_is_rejected_at_its_exact_size_boundary() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    initialize(&root, &chain);
    OpenOptions::new()
        .append(true)
        .open(root.join("selection.v2"))
        .unwrap()
        .write_all(&[0])
        .unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::InputTooLarge {
            limit: 232,
            actual: 233
        }
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn truncated_and_tampered_manifest_bytes_are_rejected_without_side_effects() {
    for corrupt in [false, true] {
        let temp = TestDirectory::new();
        let root = temp.store();
        let chain = main_chain(0);
        initialize(&root, &chain);
        let path = root.join("manifest.v2");
        let mut bytes = fs::read(&path).unwrap();
        if corrupt {
            bytes[20] ^= 1;
        } else {
            bytes.pop();
        }
        fs::write(path, bytes).unwrap();
        let before = snapshot(&temp.0);
        assert!(inspect(&root, &chain).is_err());
        assert_eq!(snapshot(&temp.0), before);
    }
}

#[test]
fn oversized_manifest_metadata_is_rejected_before_unbounded_read() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    initialize(&root, &chain);
    let path = root.join("manifest.v2");
    let oversized = 2_781_637;
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(oversized)
        .unwrap();
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::InputTooLarge { .. }
    ));
    assert_eq!(fs::metadata(path).unwrap().len(), oversized);
}

#[test]
fn per_block_limit_prevents_oversized_file_reads() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(1);
    initialize(&root, &chain);
    let path = block_path(&root, 1);
    let oversized = 16 * 1024 * 1024 + 1;
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(oversized)
        .unwrap();
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::InputTooLarge { .. }
    ));
    assert_eq!(fs::metadata(path).unwrap().len(), oversized);
}

#[test]
fn aggregate_history_limit_rejects_individually_bounded_sparse_files_before_decode() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    initialize(&root, &chain);
    let per_block = u64::from(chain.config().limits().max_block_bytes);
    let aggregate_limit = mirror_chain::v2::MAX_CHAIN_BYTES_V2 as u64;
    let count = aggregate_limit / per_block + 1;
    assert!(count <= mirror_chain::v2::MAX_CHAIN_BLOCKS_V2 as u64);

    // This is deliberately hostile, checksum-correct selected metadata, not an
    // accepted new branch. Its claimed size fits the global profile, while all
    // disk files individually fit the configured cap and exceed it in aggregate.
    // An actual decode of these zero-filled bodies would fail block framing;
    // observing the aggregate error proves that the length preflight runs first.
    let mut expected = chain.checkpoint();
    expected.height = count - 1;
    let mut selection = fs::read(root.join("selection.v2")).unwrap();
    selection[80..88].copy_from_slice(&expected.height.to_le_bytes());
    selection[152..156].copy_from_slice(&(count as u32).to_le_bytes());
    selection[160..168].copy_from_slice(&aggregate_limit.to_le_bytes());
    let mut checksum_input = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    checksum_input.extend_from_slice(&selection[..200]);
    selection[200..232].copy_from_slice(sha256d(&checksum_input).as_bytes());
    SelectionRecordV2::decode_for(&selection, expected).unwrap();
    fs::write(root.join("selection.v2"), &selection).unwrap();
    for height in 0..count {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(block_path(&root, height))
            .unwrap();
        file.set_len(per_block).unwrap();
    }

    assert!(matches!(
        error(StoreV2::inspect(&root, chain.manifest().identity(), expected)),
        StorageErrorV2::InputTooLarge { limit, actual }
            if limit == aggregate_limit && actual == count * per_block
    ));
    assert_eq!(fs::read(root.join("selection.v2")).unwrap(), selection);
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
fn truncated_and_trailing_block_bytes_are_never_accepted_or_rewritten() {
    for trailing in [false, true] {
        let temp = TestDirectory::new();
        let root = temp.store();
        let chain = main_chain(1);
        initialize(&root, &chain);
        let path = block_path(&root, 1);
        let mut bytes = fs::read(&path).unwrap();
        if trailing {
            bytes.push(0);
        } else {
            bytes.pop();
        }
        fs::write(path, bytes).unwrap();
        let before = snapshot(&temp.0);
        assert!(inspect(&root, &chain).is_err());
        assert_eq!(snapshot(&temp.0), before);
    }
}

#[test]
fn valid_work_with_a_false_intermediate_ledger_root_is_rejected_by_full_replay() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(4);
    initialize(&root, &chain);
    let good_size = fs::metadata(block_path(&root, 1)).unwrap().len();
    let forged = fixture_bytes("wrong_root1_block");
    assert_eq!(forged.len() as u64, good_size);
    fs::write(block_path(&root, 1), forged).unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::Replay(ChainReplayErrorV2::Block {
            index: 1,
            error: ChainErrorV2::WrongLedgerRoot,
        })
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn another_valid_genesis_nonce_does_not_replace_the_pinned_genesis() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    initialize(&root, &chain);
    fs::write(
        block_path(&root, 0),
        fixture_bytes("alternate_genesis_block"),
    )
    .unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::Replay(ChainReplayErrorV2::Block {
            index: 0,
            error: ChainErrorV2::WrongGenesisHash,
        })
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn root_symlink_is_rejected_without_changing_its_target() {
    let temp = TestDirectory::new();
    let real_root = temp.0.join("real-store");
    let link = temp.store();
    let chain = main_chain(0);
    initialize(&real_root, &chain);
    symlink(&real_root, &link).unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&link, &chain)),
        StorageErrorV2::UnsafePath(_)
    ));
    assert!(matches!(
        error(StoreV2::initialize(&link, &chain)),
        StorageErrorV2::UnsafePath(_)
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn selected_files_and_blocks_directory_cannot_be_symlink_aliases() {
    for relative in [
        "manifest.v2",
        "selection.v2",
        "blocks",
        "blocks/00000000000000000000.mrb2",
    ] {
        let temp = TestDirectory::new();
        let root = temp.store();
        let chain = main_chain(0);
        initialize(&root, &chain);
        let source = root.join(relative);
        let target = temp.0.join("external-target");
        fs::rename(&source, &target).unwrap();
        symlink(&target, &source).unwrap();
        let before = snapshot(&temp.0);
        assert!(matches!(
            error(inspect(&root, &chain)),
            StorageErrorV2::UnsafePath(_)
        ));
        assert_eq!(snapshot(&temp.0), before);
    }
}

#[test]
fn selected_regular_files_cannot_share_inodes_with_external_hard_links() {
    for relative in [
        "manifest.v2",
        "selection.v2",
        "blocks/00000000000000000000.mrb2",
    ] {
        let temp = TestDirectory::new();
        let root = temp.store();
        let chain = main_chain(0);
        initialize(&root, &chain);
        fs::hard_link(root.join(relative), temp.0.join("external-alias")).unwrap();
        let before = snapshot(&temp.0);
        assert!(matches!(
            error(inspect(&root, &chain)),
            StorageErrorV2::UnsafePath(_)
        ));
        assert_eq!(snapshot(&temp.0), before);
    }
}

#[test]
fn nonregular_selected_file_is_rejected_without_reading_from_it() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(0);
    initialize(&root, &chain);
    let selection = root.join("selection.v2");
    fs::remove_file(&selection).unwrap();
    fs::create_dir(&selection).unwrap();
    let before = snapshot(&temp.0);
    assert!(matches!(
        error(inspect(&root, &chain)),
        StorageErrorV2::UnsafePath(_)
    ));
    assert_eq!(snapshot(&temp.0), before);
}

#[test]
fn exact_recovered_ledger_preserves_permanent_records_and_capacity_counters() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(4);
    initialize(&root, &chain);
    let recovered = inspect(&root, &chain).unwrap();
    assert_eq!(recovered.chain().ledger().membership().record_count(), 4);
    assert!(
        recovered
            .chain()
            .ledger()
            .membership()
            .record(fixture_hash("c_member_id"))
            .is_some()
    );
    assert!(
        recovered
            .chain()
            .ledger()
            .membership()
            .record(fixture_hash("d_member_id"))
            .is_some()
    );
    assert_eq!(
        recovered.chain().state_items_used(),
        chain.state_items_used()
    );
    assert_eq!(recovered.chain().ledger(), chain.ledger());
}

#[test]
fn unsuccessful_inspection_releases_ownership_for_a_later_checked_open() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(1);
    initialize(&root, &chain);
    let mut wrong = chain.checkpoint();
    wrong.tip_hash = changed_hash(wrong.tip_hash);
    assert!(StoreV2::inspect(&root, chain.manifest().identity(), wrong).is_err());
    let writer = open_writer(&root, &chain).unwrap();
    assert_eq!(writer.chain(), &chain);
}

#[test]
fn block_files_are_plain_canonical_mrb2_bytes_without_cached_ledger_authority() {
    let temp = TestDirectory::new();
    let root = temp.store();
    let chain = main_chain(4);
    initialize(&root, &chain);
    let expected_files: Vec<_> = (0..=4).map(|height| format!("{height:020}.mrb2")).collect();
    let mut actual_files: Vec<_> = fs::read_dir(root.join("blocks"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    actual_files.sort();
    assert_eq!(actual_files, expected_files);
    for block in chain.blocks() {
        let path = block_path(&root, block.header().height());
        let stored = fs::read(path).unwrap();
        assert_eq!(&stored[..4], b"MRB2");
        assert_eq!(stored, block.encode(chain.config()).unwrap());
    }
    assert!(!root.join("ledger.cache").exists());
}
