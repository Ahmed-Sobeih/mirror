//! Completed received candidates enter existing owned storage only by local choice.
//! All histories are public synthetic fixtures. These tests create their own roots;
//! actual TCP/process coverage is separate from this adoption/failure boundary.
#![cfg(target_os = "linux")]

mod support_history_v2;

use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mirror_chain::v2::{
    BranchRelationV2, BranchSideV2, ChainV2, ForkChoiceErrorV2, ForkChoiceV2, ManifestIdentityV2,
    NetworkManifestV2,
};
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::BlockV2;
use mirror_crypto::Hash256;
use mirror_network::v2::{
    HistoryFramesV2, HistoryReceiverV2, ReceivedCandidateV2,
    adoption::{
        AdoptionAttemptV2, AdoptionErrorV2, AdoptionPublicationFailureV2, initialize_received_v2,
    },
};
use mirror_storage::v2::{
    ReplacementCheckpointV2, ReplacementRecoveryActionV2, ReplacementSelectionV2,
    ReplacementStorageErrorV2, ReplacementStoreV2, StorageErrorV2,
};
use support_history_v2::{
    fixture_bytes, fixture_hash, fork_chain, independent_identity, main_chain, manifest,
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "mirror-candidate-adoption-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("creating owned test parent failed: {error}"),
            }
        }
        panic!("owned test-directory namespace exhausted");
    }
    fn root(&self) -> PathBuf {
        self.0.join("store")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // Only this test's newly created parent is removed, never a runtime root.
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    directory: bool,
    inode: u64,
    modified: (i64, i64),
    bytes: Vec<u8>,
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        assert!(!metadata.file_type().is_symlink());
        entries.insert(
            path.strip_prefix(root).unwrap().to_path_buf(),
            Entry {
                directory: metadata.is_dir(),
                inode: metadata.ino(),
                modified: (metadata.mtime(), metadata.mtime_nsec()),
                bytes: if metadata.is_dir() {
                    Vec::new()
                } else {
                    fs::read(path).unwrap()
                },
            },
        );
        if metadata.is_dir() {
            for child in fs::read_dir(path).unwrap() {
                visit(root, &child.unwrap().path(), entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

fn checkpoint(chain: &ChainV2, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}

fn received_under(identity: ManifestIdentityV2, chain: &ChainV2) -> ReceivedCandidateV2 {
    HistoryFramesV2::new(chain)
        .unwrap()
        .try_fold(HistoryReceiverV2::new(identity), |receiver, frame| {
            receiver.accept_frame(&frame.unwrap())
        })
        .unwrap()
        .finish()
        .unwrap()
}

fn received(chain: &ChainV2) -> ReceivedCandidateV2 {
    received_under(independent_identity(), chain)
}

fn prepared(root: &Path, old: &ChainV2, candidate: &ChainV2) -> AdoptionAttemptV2 {
    let store = ReplacementStoreV2::initialize(root, old).unwrap();
    AdoptionAttemptV2::prepare(
        store,
        independent_identity(),
        checkpoint(old, 0),
        received(candidate),
    )
    .unwrap()
}

fn reopen(root: &Path, expected: ReplacementCheckpointV2) -> ReplacementStoreV2 {
    ReplacementStoreV2::open_writer(root, independent_identity(), expected).unwrap()
}

fn assert_busy(root: &Path, expected: ReplacementCheckpointV2) {
    assert!(matches!(
        ReplacementStoreV2::open_writer(root, independent_identity(), expected),
        Err(ReplacementStorageErrorV2::Storage(StorageErrorV2::Busy))
    ));
    assert!(matches!(
        ReplacementStoreV2::inspect(root, independent_identity(), expected),
        Err(ReplacementStorageErrorV2::Storage(StorageErrorV2::Busy))
    ));
}

fn extended_fork() -> ChainV2 {
    let mut chain = fork_chain();
    let proposal = chain
        .prepare_candidate(chain.blocks().last().unwrap().header().timestamp() + 1, &[])
        .unwrap();
    let mined = mine_header_v2(
        *proposal.header(),
        chain.config().parameters().pow_bits,
        10_000,
    )
    .unwrap();
    chain
        .append(BlockV2::new(chain.config(), mined.header, proposal.operations()).unwrap())
        .unwrap();
    chain
}

fn generation(root: &Path, number: u64) -> PathBuf {
    root.join(format!("generation-{number:020}"))
}

fn partial_candidate(root: &Path, candidate: &ChainV2) {
    let path = generation(root, 1);
    fs::create_dir(&path).unwrap();
    fs::create_dir(path.join("blocks")).unwrap();
    let manifest = candidate.manifest().encode().unwrap();
    fs::write(path.join("manifest.v2"), &manifest[..19]).unwrap();
    let block = candidate.blocks()[2].encode(candidate.config()).unwrap();
    fs::write(path.join("blocks/00000000000000000002.mrb2"), &block[..13]).unwrap();
}

fn complete_candidate(root: &Path, candidate: &ChainV2) {
    let path = generation(root, 1);
    fs::create_dir(&path).unwrap();
    fs::create_dir(path.join("blocks")).unwrap();
    fs::write(
        path.join("manifest.v2"),
        candidate.manifest().encode().unwrap(),
    )
    .unwrap();
    for block in candidate.blocks() {
        fs::write(
            path.join("blocks")
                .join(format!("{:020}.mrb2", block.header().height())),
            block.encode(candidate.config()).unwrap(),
        )
        .unwrap();
    }
}

fn failed_with_prefix(
    root: &Path,
    old: &ChainV2,
    candidate: &ChainV2,
) -> AdoptionPublicationFailureV2 {
    let attempt = prepared(root, old, candidate);
    partial_candidate(root, candidate);
    let before = snapshot(root);
    let failure = attempt.publish().unwrap_err();
    assert!(failure.is_fenced());
    assert_eq!(
        snapshot(root),
        before,
        "ordinary publication must not recover staging"
    );
    failure
}

#[test]
fn explicit_fresh_initialization_reopens_complete_received_state() {
    let scratch = Scratch::new();
    let source = main_chain(4);
    let input = received(&source);
    let store = initialize_received_v2(scratch.root(), independent_identity(), &input).unwrap();
    let independently_retained = checkpoint(&source, 0);
    assert_eq!(store.checkpoint().unwrap(), independently_retained);
    assert_eq!(store.chain().unwrap(), &source);
    assert_eq!(input.chain(), &source);
    drop(store);
    let reopened = reopen(&scratch.root(), independently_retained);
    assert_eq!(reopened.chain().unwrap(), &source);
    assert_eq!(
        reopened.chain().unwrap().state_items_used(),
        source.state_items_used()
    );
}

#[test]
fn fresh_wrong_identity_rejects_before_creating_a_directory() {
    let source = main_chain(4);
    for change_config in [true, false] {
        let scratch = Scratch::new();
        let mut identity = independent_identity();
        if change_config {
            identity.config_hash = Hash256::from_bytes([0xab; 32]);
        } else {
            identity.genesis_hash = Hash256::from_bytes([0xcd; 32]);
        }
        assert!(matches!(
            initialize_received_v2(scratch.root(), identity, &received(&source)),
            Err(AdoptionErrorV2::Comparison(_))
        ));
        assert!(!scratch.root().exists());
    }
}

#[test]
fn fresh_initialization_preserves_unrelated_nonempty_data() {
    let scratch = Scratch::new();
    fs::create_dir(scratch.root()).unwrap();
    fs::write(scratch.root().join("operator-owned"), b"keep this").unwrap();
    let before = snapshot(&scratch.root());
    assert!(
        initialize_received_v2(
            scratch.root(),
            independent_identity(),
            &received(&main_chain(4))
        )
        .is_err()
    );
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn fresh_initialization_never_falls_back_to_overwriting_an_existing_store() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    drop(ReplacementStoreV2::initialize(scratch.root(), &old).unwrap());
    let before = snapshot(&scratch.root());
    assert!(
        initialize_received_v2(
            scratch.root(),
            independent_identity(),
            &received(&main_chain(4))
        )
        .is_err()
    );
    assert_eq!(snapshot(&scratch.root()), before);
    assert_eq!(
        reopen(&scratch.root(), checkpoint(&old, 0))
            .chain()
            .unwrap(),
        &old
    );
}

#[test]
fn preparation_is_pure_and_retains_exact_recovery_intent_before_publication() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = snapshot(&scratch.root());
    let attempt = AdoptionAttemptV2::prepare(
        store,
        independent_identity(),
        checkpoint(&old, 0),
        received(&candidate),
    )
    .unwrap();
    assert_eq!(attempt.choice(), ForkChoiceV2::PreferCandidate);
    assert_eq!(
        attempt.report().relation,
        BranchRelationV2::CandidateExtendsCurrent
    );
    assert_eq!(attempt.report().common_ancestor, old.checkpoint());
    assert_eq!(attempt.report().selected, checkpoint(&candidate, 1));
    let retained = attempt.recovery_inputs().unwrap();
    assert_eq!(retained.identity(), independent_identity());
    assert_eq!(retained.old(), checkpoint(&old, 0));
    assert_eq!(retained.candidate(), checkpoint(&candidate, 1));
    assert_eq!(retained.old_chain(), &old);
    assert_eq!(retained.candidate_chain(), &candidate);
    let request = retained.request().unwrap();
    assert_eq!(request.old_chain(), &old);
    assert_eq!(request.candidate_chain(), &candidate);
    assert_eq!(snapshot(&scratch.root()), before);
    assert_busy(&scratch.root(), checkpoint(&old, 0));
}

#[test]
fn multi_block_extension_publishes_once_and_reopens_with_new_local_pins() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let attempt = prepared(&scratch.root(), &old, &candidate);
    let inode = fs::metadata(scratch.root()).unwrap().ino();
    let outcome = attempt.publish().unwrap();
    assert_eq!(outcome.checkpoint(), checkpoint(&candidate, 1));
    assert_eq!(outcome.report().previous, checkpoint(&old, 0));
    assert_eq!(outcome.store().chain().unwrap(), &candidate);
    assert_eq!(fs::metadata(scratch.root()).unwrap().ino(), inode);
    assert!(!generation(&scratch.root(), 0).exists());
    let pins = outcome.checkpoint();
    drop(outcome.into_store());
    assert!(
        ReplacementStoreV2::open_writer(
            scratch.root(),
            independent_identity(),
            checkpoint(&old, 0)
        )
        .is_err()
    );
    assert_eq!(reopen(&scratch.root(), pins).chain().unwrap(), &candidate);
}

#[test]
fn consecutive_adoptions_use_retained_local_generation_without_peer_authority() {
    let scratch = Scratch::new();
    let initial = main_chain(0);
    let middle = main_chain(2);
    let final_chain = main_chain(4);
    let first = prepared(&scratch.root(), &initial, &middle)
        .publish()
        .unwrap();
    let independently_retained_middle = first.checkpoint();
    assert_eq!(independently_retained_middle, checkpoint(&middle, 1));
    let keep = AdoptionAttemptV2::prepare(
        first.into_store(),
        independent_identity(),
        independently_retained_middle,
        received(&middle),
    )
    .unwrap()
    .publish()
    .unwrap();
    assert_eq!(keep.checkpoint(), independently_retained_middle);
    let second = AdoptionAttemptV2::prepare(
        keep.into_store(),
        independent_identity(),
        independently_retained_middle,
        received(&final_chain),
    )
    .unwrap();
    assert_eq!(
        second.recovery_inputs().unwrap().old(),
        checkpoint(&middle, 1)
    );
    assert_eq!(
        second.recovery_inputs().unwrap().candidate(),
        checkpoint(&final_chain, 2)
    );
    assert_eq!(second.recovery_inputs().unwrap().old_chain(), &middle);
    let outcome = second.publish().unwrap();
    assert_eq!(outcome.checkpoint(), checkpoint(&final_chain, 2));
    drop(outcome.into_store());
    assert_eq!(
        reopen(&scratch.root(), checkpoint(&final_chain, 2))
            .chain()
            .unwrap(),
        &final_chain,
    );
}

#[test]
fn equal_account_divergent_tie_preserves_all_current_files_and_membership() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let candidate = fork_chain();
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(old.ledger().membership(), candidate.ledger().membership());
    let attempt = prepared(&scratch.root(), &old, &candidate);
    let before = snapshot(&scratch.root());
    assert_eq!(attempt.choice(), ForkChoiceV2::KeepCurrent);
    assert_eq!(attempt.report().relation, BranchRelationV2::Diverged);
    assert_eq!(attempt.report().common_ancestor.height, 1);
    assert!(attempt.recovery_inputs().is_none());
    let outcome = attempt.publish().unwrap();
    assert_eq!(outcome.checkpoint(), checkpoint(&old, 0));
    assert_eq!(outcome.store().chain().unwrap(), &old);
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn shorter_divergent_history_cannot_replace_current_or_advance_generation() {
    let scratch = Scratch::new();
    let old = main_chain(4);
    let candidate = fork_chain();
    let attempt = prepared(&scratch.root(), &old, &candidate);
    let before = snapshot(&scratch.root());
    assert_eq!(attempt.choice(), ForkChoiceV2::KeepCurrent);
    let outcome = attempt.publish().unwrap();
    assert_eq!(outcome.store().chain().unwrap(), &old);
    assert_eq!(outcome.checkpoint(), checkpoint(&old, 0));
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn identical_history_is_a_no_op() {
    let scratch = Scratch::new();
    let old = main_chain(4);
    let attempt = prepared(&scratch.root(), &old, &old);
    let before = snapshot(&scratch.root());
    assert_eq!(attempt.report().relation, BranchRelationV2::Identical);
    let outcome = attempt.publish().unwrap();
    assert_eq!(outcome.report().choice, ForkChoiceV2::KeepCurrent);
    assert_eq!(outcome.checkpoint(), checkpoint(&old, 0));
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn preferred_equal_account_fork_replaces_entire_membership_and_resource_history() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let candidate = extended_fork();
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(old.ledger().membership(), candidate.ledger().membership());
    assert!(
        old.ledger().membership().active_high_water()
            > candidate.ledger().membership().active_high_water()
    );
    let attempt = prepared(&scratch.root(), &old, &candidate);
    assert_eq!(attempt.report().relation, BranchRelationV2::Diverged);
    assert_eq!(attempt.report().common_ancestor, main_chain(1).checkpoint());
    let outcome = attempt.publish().unwrap();
    let pins = checkpoint(&candidate, 1);
    assert_eq!(outcome.checkpoint(), pins);
    assert_eq!(outcome.store().chain().unwrap(), &candidate);
    drop(outcome.into_store());
    let reopened = reopen(&scratch.root(), pins);
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
    assert_eq!(actual.state_items_used(), candidate.state_items_used());
    assert_eq!(actual.encoded_bytes(), candidate.encoded_bytes());
}

#[test]
fn wrong_independent_network_rejects_without_mutation_and_returns_owned_inputs() {
    let old = main_chain(3);
    let candidate = main_chain(4);
    for change_config in [true, false] {
        let scratch = Scratch::new();
        let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
        let before = snapshot(&scratch.root());
        let mut wrong = independent_identity();
        if change_config {
            wrong.config_hash = Hash256::from_bytes([0x51; 32]);
        } else {
            wrong.genesis_hash = Hash256::from_bytes([0x52; 32]);
        }
        let failure =
            AdoptionAttemptV2::prepare(store, wrong, checkpoint(&old, 0), received(&candidate))
                .unwrap_err();
        assert!(matches!(failure.error(), AdoptionErrorV2::Comparison(_)));
        assert_eq!(snapshot(&scratch.root()), before);
        let (store, input) = failure.into_inputs();
        assert!(!store.is_fenced());
        assert_eq!(store.chain().unwrap(), &old);
        assert_eq!(input.chain(), &candidate);
        assert_busy(&scratch.root(), checkpoint(&old, 0));
    }
}

#[test]
fn a_completed_candidate_from_another_genesis_is_not_eligible() {
    let scratch = Scratch::new();
    let base = manifest();
    let genesis =
        BlockV2::decode(base.config(), &fixture_bytes("alternate_genesis_block")).unwrap();
    let alternate_manifest =
        NetworkManifestV2::new(base.config(), base.seed_proofs(), genesis.hash()).unwrap();
    let alternate = ChainV2::from_genesis(&alternate_manifest, genesis).unwrap();
    let foreign = received_under(alternate_manifest.identity(), &alternate);
    let old = main_chain(3);
    let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = snapshot(&scratch.root());
    let failure =
        AdoptionAttemptV2::prepare(store, independent_identity(), checkpoint(&old, 0), foreign)
            .unwrap_err();
    assert!(matches!(
        failure.error(),
        AdoptionErrorV2::Comparison(ForkChoiceErrorV2::WrongGenesis {
            branch: BranchSideV2::Candidate
        })
    ));
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn all_current_checkpoint_fields_and_generation_remain_independent_pins() {
    let old = main_chain(3);
    for field in 0..6 {
        let scratch = Scratch::new();
        let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
        let mut wrong = checkpoint(&old, 0);
        match field {
            0 => wrong.generation = 1,
            1 => wrong.chain.height = 2,
            2 => wrong.chain.tip_hash = Hash256::from_bytes([0x61; 32]),
            3 => wrong.chain.ledger_root = Hash256::from_bytes([0x62; 32]),
            4 => wrong.chain.config_hash = Hash256::from_bytes([0x63; 32]),
            5 => wrong.chain.genesis_hash = Hash256::from_bytes([0x64; 32]),
            _ => unreachable!(),
        }
        let before = snapshot(&scratch.root());
        let failure = AdoptionAttemptV2::prepare(
            store,
            independent_identity(),
            wrong,
            received(&main_chain(4)),
        )
        .unwrap_err();
        assert!(matches!(
            failure.error(),
            AdoptionErrorV2::WrongCurrentCheckpoint
        ));
        let (store, _) = failure.into_inputs();
        assert!(!store.is_fenced());
        assert_eq!(snapshot(&scratch.root()), before);
    }
}

#[test]
fn even_keep_current_requires_the_actual_local_generation() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = snapshot(&scratch.root());
    let failure = AdoptionAttemptV2::prepare(
        store,
        independent_identity(),
        checkpoint(&old, 1),
        received(&fork_chain()),
    )
    .unwrap_err();
    assert!(matches!(
        failure.error(),
        AdoptionErrorV2::WrongCurrentCheckpoint
    ));
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn read_only_store_cannot_prepare_a_keep_or_replace_operation() {
    let old = main_chain(3);
    for candidate in [fork_chain(), main_chain(4)] {
        let scratch = Scratch::new();
        drop(ReplacementStoreV2::initialize(scratch.root(), &old).unwrap());
        let reader = ReplacementStoreV2::inspect(
            scratch.root(),
            independent_identity(),
            checkpoint(&old, 0),
        )
        .unwrap();
        let before = snapshot(&scratch.root());
        let failure = AdoptionAttemptV2::prepare(
            reader,
            independent_identity(),
            checkpoint(&old, 0),
            received(&candidate),
        )
        .unwrap_err();
        assert!(matches!(
            failure.error(),
            AdoptionErrorV2::Storage(ReplacementStorageErrorV2::ReadOnly)
        ));
        assert_eq!(snapshot(&scratch.root()), before);
    }
}

#[test]
fn a_fenced_store_is_not_reactivated_by_receiving_another_candidate() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let mut store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    fs::write(scratch.root().join("foreign"), b"do not delete").unwrap();
    assert!(
        store
            .replace(independent_identity(), checkpoint(&old, 0), &candidate)
            .is_err()
    );
    assert!(store.is_fenced());
    let before = snapshot(&scratch.root());
    let failure = AdoptionAttemptV2::prepare(
        store,
        independent_identity(),
        checkpoint(&old, 0),
        received(&candidate),
    )
    .unwrap_err();
    assert!(matches!(
        failure.error(),
        AdoptionErrorV2::Storage(ReplacementStorageErrorV2::Fenced)
    ));
    let (store, input) = failure.into_inputs();
    assert!(store.is_fenced());
    assert!(store.chain().is_err());
    assert_eq!(input.chain(), &candidate);
    assert_eq!(snapshot(&scratch.root()), before);
}

#[test]
fn changed_disk_after_preparation_fences_and_preserves_exact_inputs_and_lock() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let attempt = prepared(&scratch.root(), &old, &candidate);
    fs::write(
        scratch.root().join("foreign"),
        b"never delete automatically",
    )
    .unwrap();
    let before = snapshot(&scratch.root());
    let failure = attempt.publish().unwrap_err();
    assert!(failure.is_fenced());
    assert_eq!(failure.last_confirmed_checkpoint(), checkpoint(&old, 0));
    assert_eq!(failure.recovery_inputs().old_chain(), &old);
    assert_eq!(failure.recovery_inputs().candidate_chain(), &candidate);
    assert_eq!(
        failure.recovery_inputs().candidate(),
        checkpoint(&candidate, 1)
    );
    assert_eq!(snapshot(&scratch.root()), before);
    assert_busy(&scratch.root(), checkpoint(&old, 0));
}

#[test]
fn explicit_completion_uses_retained_both_histories_and_same_root_ownership() {
    let scratch = Scratch::new();
    let old = main_chain(3);
    let candidate = extended_fork();
    let failure = failed_with_prefix(&scratch.root(), &old, &candidate);
    let inode = fs::metadata(scratch.root()).unwrap().ino();
    assert_busy(&scratch.root(), checkpoint(&old, 0));
    let store = failure
        .recover(ReplacementRecoveryActionV2::Complete)
        .unwrap();
    assert_eq!(fs::metadata(scratch.root()).unwrap().ino(), inode);
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&candidate, 1));
    assert_eq!(store.chain().unwrap(), &candidate);
    assert_busy(&scratch.root(), checkpoint(&candidate, 1));
    drop(store);
    assert_eq!(
        reopen(&scratch.root(), checkpoint(&candidate, 1))
            .chain()
            .unwrap(),
        &candidate
    );
}

#[test]
fn explicit_retention_removes_only_authorized_candidate_prefixes() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let failure = failed_with_prefix(&scratch.root(), &old, &candidate);
    let old_bytes = fs::read(scratch.root().join("selection.v2")).unwrap();
    let store = failure
        .recover(ReplacementRecoveryActionV2::RetainOld)
        .unwrap();
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&old, 0));
    assert_eq!(store.chain().unwrap(), &old);
    assert_eq!(
        fs::read(scratch.root().join("selection.v2")).unwrap(),
        old_bytes
    );
    assert!(!generation(&scratch.root(), 1).exists());
    drop(store);
    assert_eq!(
        reopen(&scratch.root(), checkpoint(&old, 0))
            .chain()
            .unwrap(),
        &old
    );
}

#[test]
fn failed_recovery_returns_no_handle_but_keeps_inputs_for_explicit_retry() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let attempt = prepared(&scratch.root(), &old, &candidate);
    let foreign = scratch.root().join("foreign");
    fs::write(&foreign, b"operator-owned test residue").unwrap();
    let failure = attempt.publish().unwrap_err();
    let before = snapshot(&scratch.root());
    let recovery_failure = failure
        .recover(ReplacementRecoveryActionV2::Complete)
        .unwrap_err();
    assert_eq!(recovery_failure.root(), scratch.root());
    assert_eq!(
        recovery_failure.recovery_inputs().identity(),
        independent_identity()
    );
    assert_eq!(recovery_failure.recovery_inputs().old_chain(), &old);
    assert_eq!(
        recovery_failure.recovery_inputs().candidate_chain(),
        &candidate
    );
    assert_eq!(snapshot(&scratch.root()), before);
    // Retry is explicit and still cannot delete unknown bytes.
    let recovery_failure = recovery_failure
        .retry(ReplacementRecoveryActionV2::Complete)
        .unwrap_err();
    assert_eq!(snapshot(&scratch.root()), before);
    // Only the test owner removes its own injected residue, after inspecting the
    // failure. Neither production publication nor recovery performs this action.
    fs::remove_file(&foreign).unwrap();
    // No handle survived the failed recovery, so a strict opener can own the
    // clean old root. Its lifetime also proves retry must reacquire exclusivity.
    let blocker = reopen(&scratch.root(), checkpoint(&old, 0));
    let recovery_failure = recovery_failure
        .retry(ReplacementRecoveryActionV2::Complete)
        .unwrap_err();
    assert_eq!(
        recovery_failure.recovery_inputs().candidate_chain(),
        &candidate
    );
    drop(blocker);
    let store = recovery_failure
        .retry(ReplacementRecoveryActionV2::Complete)
        .unwrap();
    assert_eq!(store.chain().unwrap(), &candidate);
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&candidate, 1));
}

#[test]
fn selected_candidate_is_never_rolled_back_by_retention_after_failure() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let attempt = prepared(&scratch.root(), &old, &candidate);
    // A recognized post-publication/unfinished-old-cleanup layout. This test
    // emulates its bytes; production uses only the existing publication path.
    complete_candidate(&scratch.root(), &candidate);
    fs::write(
        scratch.root().join("selection.v2"),
        ReplacementSelectionV2::from_chain(&candidate, 1)
            .unwrap()
            .encode(),
    )
    .unwrap();
    let before = snapshot(&scratch.root());
    let failure = attempt.publish().unwrap_err();
    assert_eq!(failure.last_confirmed_checkpoint(), checkpoint(&old, 0));
    assert_eq!(snapshot(&scratch.root()), before);
    let failure = failure
        .recover(ReplacementRecoveryActionV2::RetainOld)
        .unwrap_err();
    assert_eq!(snapshot(&scratch.root()), before);
    let store = failure
        .retry(ReplacementRecoveryActionV2::Complete)
        .unwrap();
    assert_eq!(store.checkpoint().unwrap(), checkpoint(&candidate, 1));
    assert_eq!(store.chain().unwrap(), &candidate);
    assert!(!generation(&scratch.root(), 0).exists());
}

#[test]
fn changing_selected_bytes_after_preparation_never_turns_peer_input_into_repair_authority() {
    let scratch = Scratch::new();
    let old = main_chain(1);
    let candidate = main_chain(4);
    let attempt = prepared(&scratch.root(), &old, &candidate);
    let selector_path = scratch.root().join("selection.v2");
    let mut bytes = fs::read(&selector_path).unwrap();
    bytes[279] ^= 1;
    fs::write(&selector_path, bytes).unwrap();
    let before = snapshot(&scratch.root());
    let failure = attempt.publish().unwrap_err();
    assert!(failure.is_fenced());
    assert_eq!(snapshot(&scratch.root()), before);
    let failure = failure
        .recover(ReplacementRecoveryActionV2::Complete)
        .unwrap_err();
    assert_eq!(failure.recovery_inputs().old_chain(), &old);
    assert_eq!(failure.recovery_inputs().candidate_chain(), &candidate);
    assert_eq!(snapshot(&scratch.root()), before);
}
