//! Finite, public-synthetic two-process V2 peer exchange and durable adoption.
//! This example is not a persistent daemon, wallet, enrollment or launch action.

#[path = "../tests/support_peer_process_v2/mod.rs"]
mod support_peer_process_v2;

use std::time::Duration;

use mirror_chain::v2::{ChainV2, ForkChoiceV2};
use mirror_network::v2::adoption::{AdoptionAttemptV2, initialize_received_v2};
use mirror_storage::v2::ReplacementStoreV2;
use support_peer_process_v2::{
    PeerChild, Scratch, assert_complete_state, checkpoint,
    history::{independent_identity, main_chain},
    preferred_fork, run_child, sessions,
};

fn show(label: &str, chain: &ChainV2, generation: u64) {
    let membership = chain.ledger().membership();
    println!(
        "{label}: height={} generation={generation} accounts={} active={} pending={} peak={} approvals={} MRB2={} state_items={}",
        chain.height(),
        chain.ledger().account_count(),
        membership.active_count(),
        membership.pending_count(),
        membership.active_high_water(),
        membership.retained_approval_count(),
        chain.encoded_bytes(),
        chain.state_items_used(),
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--synthetic-provider") {
        return run_child();
    }

    let fresh = Scratch::new("fresh-peer-demo");
    let source = main_chain(4);
    let expected = checkpoint(&source, 0);
    let peer = PeerChild::spawn(&["--synthetic-provider"], "main");
    let mut client = sessions(Duration::from_secs(5));
    let received = client.request(peer.address(), independent_identity())?;
    peer.finish();
    assert_complete_state(received.chain(), &source);
    println!(
        "Fresh peer received {} frames / {} framed bytes over TCP in a separate process.",
        received.frame_count(),
        received.wire_bytes()
    );
    let store = initialize_received_v2(fresh.root(), independent_identity(), &received)?;
    assert_eq!(store.checkpoint()?, expected);
    drop(store);
    let reopened = ReplacementStoreV2::open_writer(fresh.root(), independent_identity(), expected)?;
    assert_complete_state(reopened.chain()?, &source);
    show(
        "Fresh peer strict reopen",
        reopened.chain()?,
        expected.generation,
    );
    drop(reopened);

    let branch = Scratch::new("fork-peer-demo");
    let old = main_chain(3);
    let mut store = ReplacementStoreV2::initialize(branch.root(), &old)?;
    let peer = PeerChild::spawn(&["--synthetic-provider"], "fork");
    let received =
        sessions(Duration::from_secs(5)).request(peer.address(), independent_identity())?;
    peer.finish();
    assert_eq!(
        old.ledger().account_bytes()?,
        received.chain().ledger().account_bytes()?
    );
    assert_ne!(
        old.ledger().membership(),
        received.chain().ledger().membership()
    );
    let attempt =
        AdoptionAttemptV2::prepare(store, independent_identity(), checkpoint(&old, 0), received)?;
    assert_eq!(attempt.choice(), ForkChoiceV2::KeepCurrent);
    let outcome = attempt.publish()?;
    assert_complete_state(outcome.store().chain()?, &old);
    store = outcome.into_store();
    show(
        "Equal-height membership fork kept current",
        store.chain()?,
        0,
    );

    let candidate = preferred_fork();
    let selected = checkpoint(&candidate, 1);
    let peer = PeerChild::spawn(&["--synthetic-provider"], "preferred-fork");
    let received =
        sessions(Duration::from_secs(5)).request(peer.address(), independent_identity())?;
    peer.finish();
    assert_complete_state(received.chain(), &candidate);
    assert_eq!(
        old.ledger().account_bytes()?,
        candidate.ledger().account_bytes()?
    );
    assert!(
        old.ledger().membership().active_high_water()
            > candidate.ledger().membership().active_high_water()
    );
    let attempt =
        AdoptionAttemptV2::prepare(store, independent_identity(), checkpoint(&old, 0), received)?;
    assert_eq!(attempt.choice(), ForkChoiceV2::PreferCandidate);
    let outcome = attempt.publish()?;
    assert_eq!(outcome.checkpoint(), selected);
    assert_complete_state(outcome.store().chain()?, &candidate);
    drop(outcome);
    let reopened =
        ReplacementStoreV2::open_writer(branch.root(), independent_identity(), selected)?;
    assert_complete_state(reopened.chain()?, &candidate);
    show(
        "Preferred whole membership fork strict reopen",
        reopened.chain()?,
        selected.generation,
    );
    println!(
        "Complete accounts, memberships, permanent records, approvals, sequences and resource counters match each selected branch."
    );
    println!(
        "Only synthetic data was used. Persistent node/wallet operations and human consensus remain separate work."
    );
    drop(reopened);
    Ok(())
}
