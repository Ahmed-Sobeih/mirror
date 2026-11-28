//! Whole accepted-branch replacement, strict reopening and preserved artifacts.
//! All inputs are frozen public synthetic histories; no process is spawned here.
#![cfg(target_os = "linux")]

mod support_v2;

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mirror_chain::v2::{ChainErrorV2, ChainV2, NetworkManifestV2};
use mirror_core::v2::{BlockHeaderV2, BlockV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{
    AppendCheckpointV2, AppendStoreV2, ReplacementCheckpointV2, ReplacementPlanErrorV2,
    ReplacementSelectionV2, ReplacementStorageErrorV2 as Error, ReplacementStoreV2, StorageErrorV2,
    StoreAccessV2, StoreV2,
};
use support_v2::{fixture_bytes, fixture_hash, fixture_identity, fork_chain, main_chain, manifest};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "mirror-replacement-integration-{}-{}",
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
        // Cleanup belongs only to this newly created test parent, never a user
        // path or the production replacement implementation.
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
    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn checkpoint(chain: &ChainV2, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}

fn generation_path(root: &Path, generation: u64) -> PathBuf {
    root.join(format!("generation-{generation:020}"))
}

fn block_path(root: &Path, generation: u64, height: u64) -> PathBuf {
    generation_path(root, generation)
        .join("blocks")
        .join(format!("{height:020}.mrb2"))
}

fn writer(root: &Path, chain: &ChainV2, generation: u64) -> Result<ReplacementStoreV2, Error> {
    ReplacementStoreV2::open_writer(
        root,
        chain.manifest().identity(),
        checkpoint(chain, generation),
    )
}

fn reader(root: &Path, chain: &ChainV2, generation: u64) -> Result<ReplacementStoreV2, Error> {
    ReplacementStoreV2::inspect(
        root,
        chain.manifest().identity(),
        checkpoint(chain, generation),
    )
}

fn initialize(root: &Path, chain: &ChainV2) {
    drop(ReplacementStoreV2::initialize(root, chain).unwrap());
}

fn assert_rejected_unchanged(root: &Path, chain: &ChainV2, generation: u64) {
    let before = snapshot(root);
    assert!(reader(root, chain, generation).is_err());
    assert_eq!(snapshot(root), before, "inspection changed rejected files");
    assert!(writer(root, chain, generation).is_err());
    assert_eq!(
        snapshot(root),
        before,
        "writer opening changed rejected files"
    );
}

fn append_empty(chain: &mut ChainV2) {
    let proposal = chain
        .prepare_candidate(chain.blocks().last().unwrap().header().timestamp() + 1, &[])
        .unwrap();
    let header = proposal.header();
    for nonce in 0..10_000 {
        let candidate = BlockHeaderV2::new(
            header.config_hash(),
            header.height(),
            header.previous_hash(),
            header.operations_root(),
            header.ledger_root(),
            header.timestamp(),
            header.pow_bits(),
            nonce,
        );
        let block = BlockV2::new(chain.config(), candidate, proposal.operations()).unwrap();
        match chain.append(block) {
            Ok(()) => return,
            Err(ChainErrorV2::Work(_)) => {}
            Err(error) => panic!("unexpected empty-child rejection: {error}"),
        }
    }
    panic!("public synthetic target search exhausted");
}

fn alternate_genesis() -> ChainV2 {
    let base = manifest();
    let genesis =
        BlockV2::decode(base.config(), &fixture_bytes("alternate_genesis_block")).unwrap();
    let alternate =
        NetworkManifestV2::new(base.config(), base.seed_proofs(), genesis.hash()).unwrap();
    ChainV2::from_genesis(&alternate, genesis).unwrap()
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
fn initialization_persists_exact_msr2_manifest_and_complete_block_bytes() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let history = main_chain(3);
    let store = ReplacementStoreV2::initialize(&root, &history).unwrap();
    assert_eq!(store.access(), StoreAccessV2::Writer);
    assert_eq!(store.root(), root);
    assert!(!store.is_fenced());
    assert_eq!(store.chain().unwrap(), &history);
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&history, 0));
    assert_eq!(store.last_confirmed_checkpoint(), checkpoint(&history, 0));
    assert_eq!(
        fs::read(root.join("selection.v2")).unwrap(),
        ReplacementSelectionV2::from_chain(&history, 0)
            .unwrap()
            .encode()
    );
    assert_eq!(
        fs::read(generation_path(&root, 0).join("manifest.v2")).unwrap(),
        fixture_bytes("manifest_hex")
    );
    for height in 0..=3 {
        assert_eq!(
            fs::read(block_path(&root, 0, height)).unwrap(),
            fixture_bytes(&format!("block{height}_block"))
        );
    }
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    assert_eq!(fs::read_dir(generation_path(&root, 0)).unwrap().count(), 2);
    assert!(!root.join("selection.pending").exists());
}

#[test]
fn explicit_initialization_accepts_an_existing_empty_directory() {
    let scratch = Scratch::new();
    fs::create_dir(scratch.root()).unwrap();
    let inode = fs::metadata(scratch.root()).unwrap().ino();
    let history = main_chain(0);
    let store = ReplacementStoreV2::initialize(scratch.root(), &history).unwrap();
    assert_eq!(fs::metadata(store.root()).unwrap().ino(), inode);
    assert_eq!(store.chain().unwrap(), &history);
}

#[test]
fn initialization_never_overwrites_nonempty_or_existing_selected_storage() {
    let scratch = Scratch::new();
    let root = scratch.root();
    fs::create_dir(&root).unwrap();
    fs::write(root.join("unrelated.txt"), b"retain this file").unwrap();
    let before = snapshot(&root);
    assert!(ReplacementStoreV2::initialize(&root, &main_chain(0)).is_err());
    assert_eq!(snapshot(&root), before);
    let second = scratch.0.join("selected");
    initialize(&second, &main_chain(3));
    let before = snapshot(&second);
    assert!(ReplacementStoreV2::initialize(&second, &fork_chain()).is_err());
    assert_eq!(snapshot(&second), before);
}

#[test]
fn whole_divergent_history_replaces_old_branch_and_reopens_exactly() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let old = fork_chain();
    let candidate = main_chain(4);
    let candidate_before = candidate.clone();
    let mut store = ReplacementStoreV2::initialize(&root, &old).unwrap();
    let selected = store
        .replace(fixture_identity(), checkpoint(&old, 0), &candidate)
        .unwrap();
    assert_eq!(selected, checkpoint(&candidate, 1));
    assert_eq!(store.chain().unwrap(), &candidate);
    assert_eq!(candidate, candidate_before);
    assert!(!generation_path(&root, 0).exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    assert_eq!(
        fs::read(root.join("selection.v2")).unwrap(),
        ReplacementSelectionV2::from_chain(&candidate, 1)
            .unwrap()
            .encode()
    );
    for height in 0..=4 {
        assert_eq!(
            fs::read(block_path(&root, 1, height)).unwrap(),
            fixture_bytes(&format!("block{height}_block"))
        );
    }
    drop(store);
    assert!(writer(&root, &old, 0).is_err());
    let reopened = writer(&root, &candidate, 1).unwrap();
    assert_eq!(reopened.chain().unwrap(), &candidate);
    assert_eq!(
        reopened.chain().unwrap().encoded_bytes(),
        candidate.encoded_bytes()
    );
    assert_eq!(
        reopened.chain().unwrap().state_items_used(),
        candidate.state_items_used()
    );
}

#[test]
fn equal_account_bytes_cannot_keep_losing_membership_or_population_history() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let mut candidate = fork_chain();
    append_empty(&mut candidate);
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(old.ledger().membership(), candidate.ledger().membership());
    assert!(
        old.ledger().membership().active_high_water()
            > candidate.ledger().membership().active_high_water()
    );
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let selected = store
        .replace(fixture_identity(), checkpoint(&old, 0), &candidate)
        .unwrap();
    assert_eq!(store.chain().unwrap(), &candidate);
    drop(store);
    let reopened =
        ReplacementStoreV2::open_writer(scratch.root(), fixture_identity(), selected).unwrap();
    let actual = reopened.chain().unwrap();
    assert_eq!(actual, &candidate);
    for id in [fixture_hash("c_member_id"), fixture_hash("seed0_id")] {
        assert_eq!(
            actual.ledger().membership().record(id),
            candidate.ledger().membership().record(id)
        );
        assert_ne!(
            actual.ledger().membership().record(id),
            old.ledger().membership().record(id)
        );
    }
    assert_eq!(
        actual.ledger().membership().retained_approval_count(),
        candidate.ledger().membership().retained_approval_count()
    );
    assert_eq!(
        actual.ledger().membership().total_activations(),
        candidate.ledger().membership().total_activations()
    );
    assert_eq!(
        actual.ledger().membership().active_high_water(),
        candidate.ledger().membership().active_high_water()
    );
}

#[test]
fn multi_block_extension_is_one_publication_generation() {
    let scratch = Scratch::new();
    let old = main_chain(0);
    let candidate = main_chain(4);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    assert_eq!(
        store
            .replace(fixture_identity(), checkpoint(&old, 0), &candidate)
            .unwrap(),
        checkpoint(&candidate, 1)
    );
    assert_eq!(store.chain().unwrap().height(), 4);
    assert!(!generation_path(&scratch.root(), 0).exists());
    assert!(!generation_path(&scratch.root(), 4).exists());
}

#[test]
fn successive_replacements_keep_only_one_complete_selected_generation() {
    let scratch = Scratch::new();
    let root = scratch.root();
    let mut store = ReplacementStoreV2::initialize(&root, &main_chain(0)).unwrap();
    let mut expected = checkpoint(&main_chain(0), 0);
    for (generation, height) in [(1, 2), (2, 3), (3, 4)] {
        let candidate = main_chain(height);
        expected = store
            .replace(fixture_identity(), expected, &candidate)
            .unwrap();
        assert_eq!(expected, checkpoint(&candidate, generation));
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        assert!(!generation_path(&root, generation - 1).exists());
        drop(store);
        store = writer(&root, &candidate, generation).unwrap();
        assert_eq!(store.chain().unwrap(), &candidate);
    }
}

#[test]
fn successful_replacement_preserves_the_locked_root_inode() {
    let scratch = Scratch::new();
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &fork_chain()).unwrap();
    let before = fs::metadata(store.root()).unwrap();
    store
        .replace(
            fixture_identity(),
            checkpoint(&fork_chain(), 0),
            &main_chain(4),
        )
        .unwrap();
    let after = fs::metadata(store.root()).unwrap();
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    assert!(matches!(
        writer(store.root(), &main_chain(4), 1),
        Err(Error::Storage(StorageErrorV2::Busy))
    ));
}

fn reject_preference(current: &ChainV2, candidate: &ChainV2) {
    let scratch = Scratch::new();
    let mut store = ReplacementStoreV2::initialize(scratch.root(), current).unwrap();
    let before = snapshot(store.root());
    assert!(matches!(
        store.replace(fixture_identity(), checkpoint(current, 0), candidate),
        Err(Error::Plan(ReplacementPlanErrorV2::CandidateNotPreferred))
    ));
    assert!(!store.is_fenced());
    assert_eq!(store.chain().unwrap(), current);
    assert_eq!(snapshot(store.root()), before);
}

#[test]
fn identical_chain_is_not_a_new_generation() {
    let chain = main_chain(3);
    reject_preference(&chain, &chain);
}

#[test]
fn equal_height_divergent_candidate_keeps_current_without_writes() {
    reject_preference(&main_chain(3), &fork_chain());
    reject_preference(&fork_chain(), &main_chain(3));
}

#[test]
fn shorter_candidates_cannot_roll_back_selected_storage() {
    reject_preference(&main_chain(4), &main_chain(1));
    reject_preference(&main_chain(4), &fork_chain());
}

#[test]
fn foreign_genesis_and_wrong_network_identity_reject_without_fencing() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = snapshot(store.root());
    assert!(
        store
            .replace(
                fixture_identity(),
                checkpoint(&old, 0),
                &alternate_genesis()
            )
            .is_err()
    );
    let mut wrong = fixture_identity();
    wrong.config_hash = Hash256::default();
    assert!(
        store
            .replace(wrong, checkpoint(&old, 0), &main_chain(4))
            .is_err()
    );
    wrong = fixture_identity();
    wrong.genesis_hash = Hash256::default();
    assert!(
        store
            .replace(wrong, checkpoint(&old, 0), &main_chain(4))
            .is_err()
    );
    assert!(!store.is_fenced());
    assert_eq!(store.chain().unwrap(), &old);
    assert_eq!(snapshot(store.root()), before);
}

#[test]
fn stale_caller_endpoint_and_generation_cannot_authorize_replacement() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = snapshot(store.root());
    let mut variants = [checkpoint(&old, 0); 6];
    variants[0].chain.config_hash = Hash256::default();
    variants[1].chain.genesis_hash = Hash256::default();
    variants[2].chain.height = 2;
    variants[3].chain.tip_hash = Hash256::default();
    variants[4].chain.ledger_root = Hash256::default();
    variants[5].generation = 1;
    for wrong in variants {
        assert!(
            store
                .replace(fixture_identity(), wrong, &main_chain(4))
                .is_err()
        );
        assert!(!store.is_fenced());
        assert_eq!(store.chain().unwrap(), &old);
        assert_eq!(snapshot(store.root()), before);
    }
}

#[test]
fn reopening_requires_every_independent_endpoint_and_generation_pin() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    let before = snapshot(&scratch.root());
    let mut variants = [checkpoint(&old, 0); 6];
    variants[0].chain.config_hash = Hash256::default();
    variants[1].chain.genesis_hash = Hash256::default();
    variants[2].chain.height = 2;
    variants[3].chain.tip_hash = Hash256::default();
    variants[4].chain.ledger_root = Hash256::default();
    variants[5].generation = 1;
    for wrong in variants {
        assert!(
            ReplacementStoreV2::open_writer(scratch.root(), fixture_identity(), wrong).is_err()
        );
        assert_eq!(snapshot(&scratch.root()), before);
    }
}

#[test]
fn inspection_is_shared_and_cannot_replace_or_reconcile() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    let before = snapshot(&scratch.root());
    let mut first = reader(&scratch.root(), &old, 0).unwrap();
    let second = reader(&scratch.root(), &old, 0).unwrap();
    assert_eq!(first.access(), StoreAccessV2::ReadOnly);
    assert!(matches!(
        first.replace(fixture_identity(), checkpoint(&old, 0), &main_chain(4)),
        Err(Error::ReadOnly)
    ));
    assert!(!first.is_fenced());
    assert!(matches!(
        first.reconcile(checkpoint(&old, 0)),
        Err(Error::ReadOnly)
    ));
    assert_eq!(second.chain().unwrap(), &old);
    assert!(matches!(
        writer(&scratch.root(), &old, 0),
        Err(Error::Storage(StorageErrorV2::Busy))
    ));
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn clean_consuming_reconciliation_reexecutes_whole_selected_state() {
    let scratch = Scratch::new();
    let candidate = main_chain(4);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &fork_chain()).unwrap();
    let selected = store
        .replace(fixture_identity(), checkpoint(&fork_chain(), 0), &candidate)
        .unwrap();
    let before = snapshot(store.root());
    store = store.reconcile(selected).unwrap();
    assert!(!store.is_fenced());
    assert_eq!(store.chain().unwrap(), &candidate);
    assert_eq!(store.checkpoint().unwrap(), selected);
    assert_eq!(snapshot(store.root()), before);
}

#[test]
fn failed_consuming_reconciliation_returns_no_handle_and_keeps_artifacts() {
    let scratch = Scratch::new();
    let history = main_chain(3);
    let store = ReplacementStoreV2::initialize(scratch.root(), &history).unwrap();
    fs::write(
        store.root().join("selection.pending"),
        b"retained interruption",
    )
    .unwrap();
    let before = snapshot(store.root());
    assert!(store.reconcile(checkpoint(&history, 0)).is_err());
    assert_eq!(snapshot(&scratch.root()), before);
    assert_rejected_unchanged(&scratch.root(), &history, 0);
}

#[test]
fn selected_disk_discrepancy_fences_access_and_prevents_further_writes() {
    let scratch = Scratch::new();
    let history = main_chain(3);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &history).unwrap();
    let path = block_path(store.root(), 0, 2);
    let mut corrupt = fs::read(&path).unwrap();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    fs::write(path, corrupt).unwrap();
    let before = snapshot(store.root());
    assert!(
        store
            .replace(fixture_identity(), checkpoint(&history, 0), &main_chain(4))
            .is_err()
    );
    assert!(store.is_fenced());
    assert_eq!(store.last_confirmed_checkpoint(), checkpoint(&history, 0));
    assert!(matches!(store.chain(), Err(Error::Fenced)));
    assert!(matches!(store.selection(), Err(Error::Fenced)));
    assert!(matches!(store.checkpoint(), Err(Error::Fenced)));
    assert!(matches!(
        store.replace(fixture_identity(), checkpoint(&history, 0), &main_chain(4)),
        Err(Error::Fenced)
    ));
    assert_eq!(snapshot(store.root()), before);
}

#[test]
fn pending_selector_is_preserved_even_when_empty_or_exact_candidate_bytes() {
    for pending in [
        vec![],
        ReplacementSelectionV2::from_chain(&main_chain(4), 1)
            .unwrap()
            .encode()
            .to_vec(),
    ] {
        let scratch = Scratch::new();
        let old = main_chain(3);
        initialize(&scratch.root(), &old);
        fs::write(scratch.root().join("selection.pending"), pending).unwrap();
        assert_rejected_unchanged(&scratch.root(), &old, 0);
    }
}

#[test]
fn higher_generation_directory_does_not_select_or_authorize_cleanup() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    let unused = generation_path(&scratch.root(), 99);
    fs::create_dir(&unused).unwrap();
    fs::write(unused.join("unrelated"), b"do not remove").unwrap();
    assert_rejected_unchanged(&scratch.root(), &old, 0);
}

#[test]
fn unexpected_generation_or_root_entries_reject_before_any_cleanup() {
    for extra in ["unexpected.txt", "generation-00000000000000000001"] {
        let scratch = Scratch::new();
        let old = main_chain(3);
        initialize(&scratch.root(), &old);
        fs::write(scratch.root().join(extra), b"foreign payload").unwrap();
        assert_rejected_unchanged(&scratch.root(), &old, 0);
    }
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    fs::write(
        generation_path(&scratch.root(), 0).join("cache.bin"),
        b"not authority",
    )
    .unwrap();
    assert_rejected_unchanged(&scratch.root(), &old, 0);
}

#[test]
fn missing_or_corrupt_selected_marker_never_promotes_existing_history() {
    for bytes in [None, Some(vec![]), Some(vec![0; 280])] {
        let scratch = Scratch::new();
        let old = main_chain(3);
        initialize(&scratch.root(), &old);
        let path = scratch.root().join("selection.v2");
        if let Some(bytes) = bytes {
            fs::write(path, bytes).unwrap();
        } else {
            fs::remove_file(path).unwrap();
        }
        assert_rejected_unchanged(&scratch.root(), &old, 0);
    }
}

#[test]
fn checksum_correct_but_false_manifest_or_block_metadata_is_rejected() {
    let old = main_chain(3);
    let canonical = ReplacementSelectionV2::from_chain(&old, 0).unwrap();
    let mut variants = [canonical.encode(); 3];
    variants[0][172..176].copy_from_slice(&(canonical.snapshot().manifest_len() + 1).to_le_bytes());
    variants[1][176..184].copy_from_slice(&(canonical.snapshot().blocks_bytes() + 1).to_le_bytes());
    variants[2][184] ^= 1;
    for mut record in variants {
        rechecksum(&mut record);
        ReplacementSelectionV2::decode_for(&record, checkpoint(&old, 0)).unwrap();
        let scratch = Scratch::new();
        initialize(&scratch.root(), &old);
        fs::write(scratch.root().join("selection.v2"), record).unwrap();
        assert_rejected_unchanged(&scratch.root(), &old, 0);
    }
}

#[test]
fn full_replay_rejects_changed_block_despite_intact_selected_marker() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    let path = block_path(&scratch.root(), 0, 2);
    let mut bytes = fs::read(&path).unwrap();
    bytes[20] ^= 1;
    fs::write(path, bytes).unwrap();
    assert_rejected_unchanged(&scratch.root(), &old, 0);
}

#[test]
fn missing_extra_and_noncanonical_block_names_reject_unchanged() {
    for action in 0..3 {
        let scratch = Scratch::new();
        let old = main_chain(3);
        initialize(&scratch.root(), &old);
        match action {
            0 => fs::remove_file(block_path(&scratch.root(), 0, 2)).unwrap(),
            1 => fs::write(
                block_path(&scratch.root(), 0, 4),
                fixture_bytes("block4_block"),
            )
            .unwrap(),
            _ => fs::rename(
                block_path(&scratch.root(), 0, 2),
                generation_path(&scratch.root(), 0).join("blocks/2.mrb2"),
            )
            .unwrap(),
        }
        assert_rejected_unchanged(&scratch.root(), &old, 0);
    }
}

#[test]
fn generation_directory_name_is_exact_not_an_alias() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    fs::rename(
        generation_path(&scratch.root(), 0),
        scratch.root().join("generation-0"),
    )
    .unwrap();
    assert_rejected_unchanged(&scratch.root(), &old, 0);
}

#[test]
fn symlinked_managed_files_and_directories_are_not_followed() {
    for kind in ["block", "manifest", "generation", "blocks"] {
        let scratch = Scratch::new();
        let old = main_chain(3);
        initialize(&scratch.root(), &old);
        let path = match kind {
            "block" => block_path(&scratch.root(), 0, 2),
            "manifest" => generation_path(&scratch.root(), 0).join("manifest.v2"),
            "generation" => generation_path(&scratch.root(), 0),
            _ => generation_path(&scratch.root(), 0).join("blocks"),
        };
        let outside = scratch.0.join("outside");
        fs::rename(&path, &outside).unwrap();
        symlink(&outside, &path).unwrap();
        let external_before = snapshot(&outside);
        assert_rejected_unchanged(&scratch.root(), &old, 0);
        assert_eq!(snapshot(&outside), external_before);
    }
}

#[test]
fn hardlinked_managed_file_is_rejected_without_touching_other_link() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    let block = block_path(&scratch.root(), 0, 2);
    let outside = scratch.0.join("other-link");
    fs::hard_link(&block, &outside).unwrap();
    let before = fs::read(&outside).unwrap();
    assert_rejected_unchanged(&scratch.root(), &old, 0);
    assert_eq!(fs::read(outside).unwrap(), before);
}

#[test]
fn oversized_sparse_file_is_rejected_by_length_without_reading_or_truncating_it() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    initialize(&scratch.root(), &old);
    let block = block_path(&scratch.root(), 0, 2);
    let size = u64::from(old.config().limits().max_block_bytes) + 1;
    OpenOptions::new()
        .write(true)
        .open(&block)
        .unwrap()
        .set_len(size)
        .unwrap();
    let marker = fs::read(scratch.root().join("selection.v2")).unwrap();
    assert!(writer(&scratch.root(), &old, 0).is_err());
    assert!(reader(&scratch.root(), &old, 0).is_err());
    assert_eq!(fs::metadata(&block).unwrap().len(), size);
    assert_eq!(
        fs::read(scratch.root().join("selection.v2")).unwrap(),
        marker
    );
}

#[test]
fn bounded_root_enumeration_rejects_excess_entries_and_preserves_every_one() {
    let scratch = Scratch::new();
    let old = main_chain(0);
    initialize(&scratch.root(), &old);
    for index in 0..8 {
        fs::write(
            scratch.root().join(format!("unexpected-{index}")),
            [index as u8],
        )
        .unwrap();
    }
    assert_rejected_unchanged(&scratch.root(), &old, 0);
}

#[test]
fn prior_storage_profiles_are_never_implicitly_migrated_in_either_direction() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    for profile in [1, 2] {
        let path = scratch.0.join(format!("profile-{profile}"));
        if profile == 1 {
            drop(StoreV2::initialize(&path, &old).unwrap());
        } else {
            drop(AppendStoreV2::initialize(&path, &old).unwrap());
        }
        let before = snapshot(&path);
        assert!(ReplacementStoreV2::initialize(&path, &old).is_err());
        assert!(writer(&path, &old, 0).is_err());
        assert_eq!(snapshot(&path), before);
    }
    let path = scratch.root();
    initialize(&path, &old);
    let before = snapshot(&path);
    assert!(StoreV2::open_writer(&path, fixture_identity(), old.checkpoint()).is_err());
    assert!(
        AppendStoreV2::open_writer(
            &path,
            fixture_identity(),
            AppendCheckpointV2 {
                generation: 0,
                chain: old.checkpoint()
            }
        )
        .is_err()
    );
    assert_eq!(snapshot(&path), before);
}

#[test]
fn selected_old_storage_with_unexplained_staging_fences_replacement_without_cleanup() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let unused = generation_path(store.root(), 1);
    fs::create_dir(&unused).unwrap();
    fs::write(unused.join("not-our-candidate"), b"preserve").unwrap();
    let before = snapshot(store.root());
    assert!(
        store
            .replace(fixture_identity(), checkpoint(&old, 0), &main_chain(4))
            .is_err()
    );
    assert!(store.is_fenced());
    assert_eq!(store.last_confirmed_checkpoint(), checkpoint(&old, 0));
    assert_eq!(snapshot(store.root()), before);
}
