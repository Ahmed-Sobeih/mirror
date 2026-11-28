//! Shared synthetic, independently frozen actual-chain inputs for storage tests.
// Each integration target uses a different subset of these fixture helpers.
#![allow(dead_code)]

use std::{collections::BTreeMap, sync::OnceLock};

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2, ManifestIdentityV2, NetworkManifestV2};
use mirror_crypto::Hash256;

fn fixture_values() -> &'static BTreeMap<&'static str, &'static str> {
    static VALUES: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    VALUES.get_or_init(|| {
        include_str!("../../../mirror-chain/tests/vectors/chain_v2.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_once('=').expect("chain fixture key/value"))
            .collect()
    })
}

pub fn fixture_bytes(key: &str) -> Vec<u8> {
    let text = fixture_values()[key];
    assert_eq!(text.len() % 2, 0, "even-length fixture hex for {key}");
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

pub fn fixture_hash(key: &str) -> Hash256 {
    Hash256::from_bytes(fixture_bytes(key).try_into().unwrap())
}

pub fn fixture_identity() -> ManifestIdentityV2 {
    ManifestIdentityV2 {
        config_hash: fixture_hash("context"),
        genesis_hash: fixture_hash("block0_hash"),
    }
}

pub fn manifest() -> NetworkManifestV2 {
    NetworkManifestV2::decode_for(&fixture_bytes("manifest_hex"), fixture_identity()).unwrap()
}

fn expectation(block: &str, state: &str) -> ChainReplayExpectationV2 {
    ChainReplayExpectationV2 {
        config_hash: fixture_hash("context"),
        genesis_hash: fixture_hash("block0_hash"),
        height: fixture_values()[format!("{block}_height").as_str()]
            .parse()
            .unwrap(),
        tip_hash: fixture_hash(&format!("{block}_hash")),
        ledger_root: fixture_hash(&format!("{state}_ledger_root")),
    }
}

pub fn main_expectation(height: u64) -> ChainReplayExpectationV2 {
    assert!(
        height <= 4,
        "fixture contains main heights zero through four"
    );
    expectation(&format!("block{height}"), &format!("state{height}"))
}

pub fn fork_expectation() -> ChainReplayExpectationV2 {
    expectation("fork3", "fork_state3")
}

pub fn main_chain(height: u64) -> ChainV2 {
    let expected = main_expectation(height);
    let bytes: Vec<Vec<u8>> = (0..=height)
        .map(|index| fixture_bytes(&format!("block{index}_block")))
        .collect();
    let slices: Vec<&[u8]> = bytes.iter().map(Vec::as_slice).collect();
    ChainV2::replay_encoded(&manifest(), &slices, expected).unwrap()
}

pub fn fork_chain() -> ChainV2 {
    let bytes: Vec<Vec<u8>> = ["block0", "block1", "fork2", "fork3"]
        .iter()
        .map(|name| fixture_bytes(&format!("{name}_block")))
        .collect();
    let slices: Vec<&[u8]> = bytes.iter().map(Vec::as_slice).collect();
    ChainV2::replay_encoded(&manifest(), &slices, fork_expectation()).unwrap()
}
