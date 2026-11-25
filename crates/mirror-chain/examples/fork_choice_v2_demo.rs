//! In-memory development branch comparison with public, synthetic fixture data.
//! This example publishes no disk selection, sends no messages and issues no money.

use std::collections::BTreeMap;

use mirror_chain::v2::{
    BranchRelationV2, ChainV2, ForkChoiceV2, ManifestIdentityV2, NetworkManifestV2,
    compare_branches_v2,
};
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::BlockV2;
use mirror_crypto::Hash256;
use mirror_personhood::v2::MembershipStatusV2;

fn unhex(value: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if value.len() % 2 != 0 {
        return Err("odd fixture hex length".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| Ok(u8::from_str_radix(&value[index..index + 2], 16)?))
        .collect()
}

fn hash(value: &str) -> Result<Hash256, Box<dyn std::error::Error>> {
    Ok(Hash256::from_bytes(
        unhex(value)?
            .try_into()
            .map_err(|_| "fixture hash length")?,
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture: BTreeMap<_, _> = include_str!("../tests/vectors/chain_v2.txt")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split_once('=').expect("frozen key/value fixture"))
        .collect();
    // These pins are chosen independently, before reading the manifest or branches.
    let identity = ManifestIdentityV2 {
        config_hash: hash("9abab0c437d2883d70c68084a3f2dc843f3f2ab7ccb69a80db3fbf1ebff27e5e")?,
        genesis_hash: hash("4415cf74a204f2df610aa1e4ed737df220d93e1ad8a931e9533c00cf624c14ea")?,
    };
    let manifest = NetworkManifestV2::decode_for(&unhex(fixture["manifest_hex"])?, identity)?;
    let block = |name: &str| -> Result<BlockV2, Box<dyn std::error::Error>> {
        let bytes = unhex(fixture[format!("{name}_block").as_str()])?;
        Ok(BlockV2::decode(manifest.config(), &bytes)?)
    };
    let mut current = ChainV2::from_genesis(&manifest, block("block0")?)?;
    current.append(block("block1")?)?;
    let mut candidate = ChainV2::replay(&manifest, current.blocks(), current.checkpoint())?;
    current.append(block("block2")?)?;
    current.append(block("block3")?)?;
    candidate.append(block("fork2")?)?;
    candidate.append(block("fork3")?)?;

    let member_id = hash(fixture["c_member_id"])?;
    assert_eq!(
        current.ledger().account_bytes()?,
        candidate.ledger().account_bytes()?
    );
    assert_ne!(
        current.ledger().membership().root()?,
        candidate.ledger().membership().root()?
    );
    assert_eq!(
        current
            .ledger()
            .membership()
            .record(member_id)
            .unwrap()
            .status(),
        MembershipStatusV2::Active
    );
    assert_eq!(
        candidate
            .ledger()
            .membership()
            .record(member_id)
            .unwrap()
            .status(),
        MembershipStatusV2::Pending
    );

    // Equal scores keep the current branch despite different membership or a lower hash.
    let tied = compare_branches_v2(identity, &current, &candidate)?;
    assert_eq!(tied.relation(), BranchRelationV2::Diverged);
    assert_eq!(tied.choice(), ForkChoiceV2::KeepCurrent);
    assert_eq!(tied.common_ancestor().height, 1);
    println!("Mirror V2 development fork choice — public synthetic fixtures");
    println!("Same independent configuration and genesis; both histories fully accepted");
    println!("Equal height 3: keep current; account bytes equal, membership histories differ");
    println!(
        "Common ancestor: height {} / {}",
        tied.common_ancestor().height,
        tied.common_ancestor().tip_hash
    );

    // End the immutable comparison borrow before extending the candidate.
    let proposal = candidate.prepare_candidate(4, &[])?;
    let mined = mine_header_v2(
        *proposal.header(),
        candidate.config().parameters().pow_bits,
        10_000,
    )?;
    candidate.append(BlockV2::new(
        candidate.config(),
        mined.header,
        proposal.operations(),
    )?)?;
    let current_before = current.clone();
    let candidate_before = candidate.clone();
    let decision = compare_branches_v2(identity, &current, &candidate)?;
    assert_eq!(decision.relation(), BranchRelationV2::Diverged);
    assert_eq!(decision.choice(), ForkChoiceV2::PreferCandidate);
    assert_eq!(decision.current_score(), 3);
    assert_eq!(decision.candidate_score(), 4);
    let selected = decision.selected_chain();
    assert!(std::ptr::eq(selected, &candidate));
    assert_eq!(selected, &candidate_before);
    assert_eq!(current, current_before);
    assert_eq!(candidate, candidate_before);
    assert_eq!(
        selected
            .ledger()
            .membership()
            .record(member_id)
            .unwrap()
            .status(),
        MembershipStatusV2::Pending
    );
    println!("Candidate accepted one empty child: scores current 3 / candidate 4");
    println!("Choice: prefer candidate, common ancestor remains height 1");
    println!("Selected height: {}", selected.height());
    println!("Selected tip: {}", selected.tip_hash());
    println!("Selected ledger root: {}", selected.ledger().root()?);
    println!(
        "Selected membership root: {}",
        selected.ledger().membership().root()?
    );
    println!(
        "Selected active / pending / high-water: {} / {} / {}",
        selected.ledger().membership().active_count(),
        selected.ledger().membership().pending_count(),
        selected.ledger().membership().active_high_water()
    );
    println!(
        "Selected encoded bytes / cumulative state items: {} / {}",
        selected.encoded_bytes(),
        selected.state_items_used()
    );
    println!("Complete candidate ledger is selected; neither input was mutated");
    println!(
        "Development comparison only; no durable replacement, network finality, fees or rewards"
    );
    Ok(())
}
