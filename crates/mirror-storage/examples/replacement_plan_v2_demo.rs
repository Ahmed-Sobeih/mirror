//! Pure replacement preparation using frozen public synthetic histories.
//!
//! Both histories are fully replayed against independent literal endpoints.
//! This example creates no directory, publishes no selection and issues no money.

use std::{collections::BTreeMap, error::Error};

use mirror_chain::v2::{
    BranchRelationV2, ChainReplayExpectationV2, ChainV2, ManifestIdentityV2, NetworkManifestV2,
};
use mirror_crypto::Hash256;
use mirror_storage::v2::{
    REPLACEMENT_SELECTION_LEN_V2, ReplacementCheckpointV2, ReplacementSelectionV2,
    prepare_replacement_v2,
};

fn unhex(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if value.len() % 2 != 0 {
        return Err("odd synthetic fixture hex length".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|offset| Ok(u8::from_str_radix(&value[offset..offset + 2], 16)?))
        .collect()
}

fn hash(value: &str) -> Result<Hash256, Box<dyn Error>> {
    Ok(Hash256::from_bytes(
        unhex(value)?
            .try_into()
            .map_err(|_| "synthetic fixture hash length")?,
    ))
}

fn values(contents: &'static str) -> BTreeMap<&'static str, &'static str> {
    contents
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split_once('=').expect("frozen key/value fixture"))
        .collect()
}

fn replay(
    manifest: &NetworkManifestV2,
    fixture: &BTreeMap<&str, &str>,
    names: &[&str],
    expected: ChainReplayExpectationV2,
) -> Result<ChainV2, Box<dyn Error>> {
    let blocks: Vec<Vec<u8>> = names
        .iter()
        .map(|name| unhex(fixture[format!("{name}_block").as_str()]))
        .collect::<Result<_, _>>()?;
    let borrowed: Vec<&[u8]> = blocks.iter().map(Vec::as_slice).collect();
    Ok(ChainV2::replay_encoded(manifest, &borrowed, expected)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Trusted synthetic pins are selected before reading any manifest, block or
    // selector. A real caller must retain its own independently chosen endpoints.
    let identity = ManifestIdentityV2 {
        config_hash: hash("9abab0c437d2883d70c68084a3f2dc843f3f2ab7ccb69a80db3fbf1ebff27e5e")?,
        genesis_hash: hash("4415cf74a204f2df610aa1e4ed737df220d93e1ad8a931e9533c00cf624c14ea")?,
    };
    let current_expected = ReplacementCheckpointV2 {
        generation: 0,
        chain: ChainReplayExpectationV2 {
            config_hash: identity.config_hash,
            genesis_hash: identity.genesis_hash,
            height: 3,
            tip_hash: hash("1aa52937138f926316c679da618a84770ce0984be648b5d9ed78cbad9590f51c")?,
            ledger_root: hash("f9abe0680be28fbcf79d599f10808c7d3dac868ed33777218e33cd34fe7dbb9c")?,
        },
    };
    let candidate_expected = ReplacementCheckpointV2 {
        generation: 1,
        chain: ChainReplayExpectationV2 {
            config_hash: identity.config_hash,
            genesis_hash: identity.genesis_hash,
            height: 4,
            tip_hash: hash("33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d")?,
            ledger_root: hash("785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68")?,
        },
    };
    let ancestor_expected = ChainReplayExpectationV2 {
        config_hash: identity.config_hash,
        genesis_hash: identity.genesis_hash,
        height: 1,
        tip_hash: hash("4e02ea0f77a2f1eed2606a7a5d24343f431fee408d7928041e7e68946a2b4697")?,
        ledger_root: hash("4d620b7fc5dae805a6652e325ca775a35efd402ad79fcb41459b2083b513a528")?,
    };

    let fixture = values(include_str!(
        "../../mirror-chain/tests/vectors/chain_v2.txt"
    ));
    let selection_fixture = values(include_str!("../tests/vectors/replacement_storage_v2.txt"));
    let manifest = NetworkManifestV2::decode_for(&unhex(fixture["manifest_hex"])?, identity)?;
    let current = replay(
        &manifest,
        &fixture,
        &["block0", "block1", "fork2", "fork3"],
        current_expected.chain,
    )?;
    let candidate = replay(
        &manifest,
        &fixture,
        &["block0", "block1", "block2", "block3", "block4"],
        candidate_expected.chain,
    )?;
    let current_selection = ReplacementSelectionV2::decode_for(
        &unhex(selection_fixture["initialized_fork_selection_hex"])?,
        current_expected,
    )?;
    let current_before = current.clone();
    let candidate_before = candidate.clone();
    let plan = prepare_replacement_v2(
        identity,
        current_expected,
        &current_selection,
        &current,
        &candidate,
    )?;

    assert_eq!(plan.identity(), identity);
    assert_eq!(plan.relation(), BranchRelationV2::Diverged);
    assert_eq!(plan.common_ancestor(), ancestor_expected);
    assert_eq!(plan.current_checkpoint(), current_expected);
    assert_eq!(plan.candidate_checkpoint(), candidate_expected);
    assert!(std::ptr::eq(plan.current_chain(), &current));
    assert!(std::ptr::eq(plan.candidate_chain(), &candidate));
    assert_eq!(plan.current_chain(), &current_before);
    assert_eq!(plan.candidate_chain(), &candidate_before);
    assert_eq!(current, current_before);
    assert_eq!(candidate, candidate_before);
    assert_eq!(
        plan.candidate_selection(),
        &ReplacementSelectionV2::from_chain(&candidate, 1)?,
    );

    let encoded = plan.candidate_selection().encode();
    assert_eq!(encoded.len(), REPLACEMENT_SELECTION_LEN_V2);
    assert_eq!(
        encoded.as_slice(),
        unhex(selection_fixture["main4_after_replacement_selection_hex"])?,
    );
    let decoded = ReplacementSelectionV2::decode_for(&encoded, candidate_expected)?;
    assert_eq!(&decoded, plan.candidate_selection());
    decoded.verify_manifest_bytes(&manifest.encode()?)?;

    // These branches differ in both account and membership state. The returned
    // borrow keeps every candidate component and resource counter together.
    assert_ne!(
        current.ledger().account_bytes()?,
        plan.candidate_chain().ledger().account_bytes()?,
    );
    assert_ne!(
        current.ledger().membership().root()?,
        plan.candidate_chain().ledger().membership().root()?,
    );

    println!("Mirror V2 replacement preparation — public synthetic fixtures");
    println!("Both complete histories accepted under independent network and endpoint pins");
    println!("Current: height 3 / local generation 0");
    println!("Preferred candidate: height 4 / prepared local generation 1");
    println!("Common ancestor: height {}", plan.common_ancestor().height);
    println!("Candidate tip: {}", candidate_expected.chain.tip_hash);
    println!(
        "Candidate ledger root: {}",
        candidate_expected.chain.ledger_root
    );
    println!(
        "Candidate active / pending / high-water: {} / {} / {}",
        candidate.ledger().membership().active_count(),
        candidate.ledger().membership().pending_count(),
        candidate.ledger().membership().active_high_water(),
    );
    println!(
        "Candidate encoded bytes / cumulative state items: {} / {}",
        candidate.encoded_bytes(),
        candidate.state_items_used(),
    );
    println!("MSR2 candidate record: 280 bytes; independent vector and pinned round trip agree");
    println!("Whole candidate borrowed; both input histories remain unchanged");
    println!("PREPARED ONLY: no filesystem switch, durable publication or recovery occurred");
    Ok(())
}
