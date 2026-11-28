//! In-memory frame transfer of independently frozen public synthetic histories.
//! No socket, storage selection, biometric reading or monetary issuance is used.

#[path = "../tests/support_history_v2/mod.rs"]
mod support_history_v2;

use mirror_chain::v2::{ChainV2, ForkChoiceV2, compare_branches_v2};
use mirror_network::v2::{HistoryFramesV2, HistoryReceiverV2, ReceivedCandidateV2};
use mirror_protocol::v2::FRAME_HEADER_LEN_V2;
use support_history_v2::{fork_chain, independent_identity, main_chain};

fn transfer(source: &ChainV2) -> Result<ReceivedCandidateV2, Box<dyn std::error::Error>> {
    let mut receiver = HistoryReceiverV2::new(independent_identity());
    for frame in HistoryFramesV2::new(source)? {
        let frame = frame?;
        // A future socket adapter will validate this header before allocating or
        // reading the payload. This demonstration already owns bounded frames.
        let pending = receiver.begin_frame(&frame[..FRAME_HEADER_LEN_V2])?;
        assert_eq!(pending.payload_len(), frame.len() - FRAME_HEADER_LEN_V2);
        receiver = pending.accept_payload(&frame[FRAME_HEADER_LEN_V2..])?;
    }
    Ok(receiver.finish()?)
}

fn show(name: &str, received: &ReceivedCandidateV2) {
    let chain = received.chain();
    println!("{name}");
    println!(
        "  accepted height / tip: {} / {}",
        chain.height(),
        chain.tip_hash()
    );
    println!("  ledger root: {}", chain.checkpoint().ledger_root);
    println!(
        "  active / pending / high-water: {} / {} / {}",
        chain.ledger().membership().active_count(),
        chain.ledger().membership().pending_count(),
        chain.ledger().membership().active_high_water()
    );
    println!(
        "  frames / wire bytes: {} / {}",
        received.frame_count(),
        received.wire_bytes()
    );
    println!(
        "  complete MRB2 bytes / state-work items: {} / {}",
        chain.encoded_bytes(),
        chain.state_items_used()
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror V2 history receiver — public synthetic fixtures");
    println!("Independent configuration/genesis are selected before receiving frames.");

    let main = main_chain(4);
    let received_main = transfer(&main)?;
    assert_eq!(received_main.chain(), &main);
    show("Complete main branch reconstructed:", &received_main);

    let fork = fork_chain();
    let received_fork = transfer(&fork)?;
    assert_eq!(received_fork.chain(), &fork);
    show("Complete competing branch reconstructed:", &received_fork);

    let same_height_main = main_chain(3);
    let received_same_height = transfer(&same_height_main)?;
    assert_eq!(
        received_same_height.chain().ledger().account_bytes()?,
        received_fork.chain().ledger().account_bytes()?
    );
    assert_ne!(
        received_same_height.chain().ledger().membership().root()?,
        received_fork.chain().ledger().membership().root()?
    );
    let comparison = compare_branches_v2(
        independent_identity(),
        received_same_height.chain(),
        received_fork.chain(),
    )?;
    assert_eq!(comparison.choice(), ForkChoiceV2::KeepCurrent);
    println!("Equal account bytes preserve different membership histories at height 3.");
    println!("Independent branch comparison keeps current on equal fixed-work scores.");
    println!("All received claims were checked by actual genesis-to-tip block execution.");
    println!("No partial chain is available before the terminal frame and finish().");
    println!(
        "This is the in-memory receiver component; TCP sessions and durable adoption are next."
    );
    Ok(())
}
