//! Real OS-process peers over TCP, followed by explicit local durable selection.
//! All histories, identities and storage directories are synthetic and private
//! to these tests. No process-wide network availability guarantee is inferred.

mod support_peer_process_v2;

use std::{
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use mirror_chain::v2::{ChainErrorV2, ForkChoiceV2};
use mirror_network::v2::{
    HistoryErrorV2,
    adoption::{AdoptionAttemptV2, initialize_received_v2},
    session::{REQUEST_LEN_V2, SessionErrorV2},
};

// Unrelated concurrent forks may briefly inherit another test's directory-lock
// descriptors before exec. Serialize every parent process case in this binary;
// each still exercises a genuinely concurrent child provider.
fn process_guard() -> MutexGuard<'static, ()> {
    static PROCESS_TEST: Mutex<()> = Mutex::new(());
    PROCESS_TEST
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
use mirror_storage::v2::ReplacementStoreV2;
use support_peer_process_v2::{
    PeerChild, Scratch, assert_complete_state, checkpoint,
    history::{fork_chain, independent_identity, main_chain},
    preferred_fork, run_child, sessions, store_bytes,
};

fn peer(mode: &str) -> PeerChild {
    PeerChild::spawn(
        &[
            "--ignored",
            "--exact",
            "peer_process_child",
            "--nocapture",
            "--test-threads=1",
        ],
        mode,
    )
}

#[test]
#[ignore = "subprocess harness entry; exercised by parent process tests"]
fn peer_process_child() {
    if std::env::var_os(support_peer_process_v2::CHILD_MODE).is_some() {
        run_child().unwrap();
    }
}

#[test]
fn fresh_process_transfer_reconstructs_and_durably_reopens_complete_state() {
    let _guard = process_guard();
    let source = main_chain(4);
    let expected = checkpoint(&source, 0);
    let scratch = Scratch::new("fresh-process");
    let provider = peer("main");
    let mut client = sessions(Duration::from_secs(5));
    let received = client
        .request(provider.address(), independent_identity())
        .unwrap();
    assert_complete_state(received.chain(), &source);
    assert_eq!(client.stats().attempts, 1);
    assert_eq!(
        client.stats().wire_bytes,
        (REQUEST_LEN_V2 + received.wire_bytes()) as u64
    );
    assert!(
        !scratch.root().exists(),
        "network reception must not initialize a store"
    );
    provider.finish();

    let store = initialize_received_v2(scratch.root(), independent_identity(), &received).unwrap();
    assert_eq!(store.checkpoint().unwrap(), expected);
    assert_complete_state(store.chain().unwrap(), &source);
    drop(store);
    let reopened =
        ReplacementStoreV2::open_writer(scratch.root(), independent_identity(), expected).unwrap();
    assert_complete_state(reopened.chain().unwrap(), &source);
}

#[test]
fn equal_height_remote_membership_fork_keeps_current_exact_store() {
    let _guard = process_guard();
    let old = main_chain(3);
    let offered = fork_chain();
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        offered.ledger().account_bytes().unwrap()
    );
    assert_ne!(old.ledger().membership(), offered.ledger().membership());
    let scratch = Scratch::new("tie-process");
    let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = store_bytes(&scratch.root());
    let provider = peer("fork");
    let received = sessions(Duration::from_secs(5))
        .request(provider.address(), independent_identity())
        .unwrap();
    provider.finish();
    assert_complete_state(received.chain(), &offered);
    let attempt =
        AdoptionAttemptV2::prepare(store, independent_identity(), checkpoint(&old, 0), received)
            .unwrap();
    assert_eq!(attempt.choice(), ForkChoiceV2::KeepCurrent);
    let outcome = attempt.publish().unwrap();
    assert_eq!(outcome.checkpoint(), checkpoint(&old, 0));
    assert_complete_state(outcome.store().chain().unwrap(), &old);
    assert_eq!(store_bytes(&scratch.root()), before);
}

#[test]
fn preferred_remote_fork_replaces_memberships_and_population_with_whole_branch() {
    let _guard = process_guard();
    let old = main_chain(3);
    let candidate = preferred_fork();
    let expected = checkpoint(&candidate, 1);
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(old.ledger().membership(), candidate.ledger().membership());
    assert!(
        old.ledger().membership().active_high_water()
            > candidate.ledger().membership().active_high_water()
    );
    let scratch = Scratch::new("fork-process");
    let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let provider = peer("preferred-fork");
    let received = sessions(Duration::from_secs(5))
        .request(provider.address(), independent_identity())
        .unwrap();
    provider.finish();
    assert_complete_state(received.chain(), &candidate);
    let attempt =
        AdoptionAttemptV2::prepare(store, independent_identity(), checkpoint(&old, 0), received)
            .unwrap();
    assert_eq!(attempt.choice(), ForkChoiceV2::PreferCandidate);
    let outcome = attempt.publish().unwrap();
    assert_eq!(outcome.checkpoint(), expected);
    assert_complete_state(outcome.store().chain().unwrap(), &candidate);
    drop(outcome);
    assert!(
        ReplacementStoreV2::open_writer(
            scratch.root(),
            independent_identity(),
            checkpoint(&old, 0)
        )
        .is_err()
    );
    let reopened =
        ReplacementStoreV2::open_writer(scratch.root(), independent_identity(), expected).unwrap();
    assert_complete_state(reopened.chain().unwrap(), &candidate);
}

#[test]
fn fragmented_process_transcript_reconstructs_the_complete_history() {
    let _guard = process_guard();
    let provider = peer("fragmented");
    let received = sessions(Duration::from_secs(5))
        .request(provider.address(), independent_identity())
        .unwrap();
    provider.finish();
    assert_complete_state(received.chain(), &main_chain(4));
}

fn rejected_process_preserves_selected(
    mode: &str,
    timeout: Duration,
) -> (SessionErrorV2, u64, Duration) {
    let _guard = process_guard();
    let old = main_chain(3);
    let expected = checkpoint(&old, 0);
    let scratch = Scratch::new("rejected-process");
    let store = ReplacementStoreV2::initialize(scratch.root(), &old).unwrap();
    let before = store_bytes(&scratch.root());
    let provider = peer(mode);
    let mut client = sessions(timeout);
    let started = Instant::now();
    let error = client
        .request(provider.address(), independent_identity())
        .unwrap_err();
    let elapsed = started.elapsed();
    assert_eq!(client.stats().attempts, 1);
    assert!(!store.is_fenced());
    assert_eq!(store.checkpoint().unwrap(), expected);
    assert_complete_state(store.chain().unwrap(), &old);
    assert_eq!(store_bytes(&scratch.root()), before);
    provider.finish();
    drop(store);
    let reopened =
        ReplacementStoreV2::open_writer(scratch.root(), independent_identity(), expected).unwrap();
    assert_complete_state(reopened.chain().unwrap(), &old);
    (error, client.stats().wire_bytes, elapsed)
}

#[test]
fn early_closed_process_never_exposes_partial_candidate_or_changes_storage() {
    let (error, _, _) = rejected_process_preserves_selected("early-close", Duration::from_secs(3));
    assert!(matches!(error, SessionErrorV2::EarlyClose), "{error:?}");
}

#[test]
fn stalled_process_is_rejected_by_absolute_deadline_and_keeps_store() {
    let (error, bytes, elapsed) =
        rejected_process_preserves_selected("stall", Duration::from_millis(100));
    assert!(
        matches!(error, SessionErrorV2::DeadlineExceeded),
        "{error:?}"
    );
    assert_eq!(bytes, (REQUEST_LEN_V2 + 3) as u64);
    assert!(elapsed < Duration::from_secs(1));
}

#[test]
fn oversized_header_from_process_rejects_without_reading_declared_body() {
    let (error, bytes, _) =
        rejected_process_preserves_selected("oversized", Duration::from_secs(3));
    assert!(
        matches!(error, SessionErrorV2::History(HistoryErrorV2::Protocol(_))),
        "{error:?}"
    );
    assert_eq!(bytes, (REQUEST_LEN_V2 + 12) as u64);
}

#[test]
fn changed_terminal_claim_from_process_does_not_publish_completed_blocks() {
    let (error, _, _) = rejected_process_preserves_selected("changed-end", Duration::from_secs(3));
    assert!(
        matches!(
            error,
            SessionErrorV2::History(HistoryErrorV2::ChangedSummary)
        ),
        "{error:?}"
    );
}

#[test]
fn actual_invalid_block_from_process_is_reexecuted_and_rejected() {
    let (error, _, _) =
        rejected_process_preserves_selected("invalid-block", Duration::from_secs(3));
    assert!(
        matches!(
            error,
            SessionErrorV2::History(HistoryErrorV2::Chain(ChainErrorV2::WrongLedgerRoot))
        ),
        "{error:?}"
    );
}
