//! Public synthetic histories frozen independently before this receiver existed.
#![allow(dead_code)]

use std::{collections::BTreeMap, sync::OnceLock};

use mirror_chain::v2::{ChainV2, ManifestIdentityV2, NetworkManifestV2};
use mirror_core::v2::BlockV2;
use mirror_crypto::Hash256;

fn fixture_values() -> &'static BTreeMap<&'static str, &'static str> {
    static VALUES: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    VALUES.get_or_init(|| {
        include_str!("../../../mirror-chain/tests/vectors/chain_v2.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_once('=').expect("frozen chain key/value"))
            .collect()
    })
}

pub fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

pub fn fixture_bytes(name: &str) -> Vec<u8> {
    unhex(fixture_values()[name])
}

pub fn fixture_hash(name: &str) -> Hash256 {
    Hash256::from_bytes(fixture_bytes(name).try_into().unwrap())
}

/// These local pins are selected before decoding any transfer or manifest.
pub fn independent_identity() -> ManifestIdentityV2 {
    ManifestIdentityV2 {
        config_hash: Hash256::from_bytes(
            unhex("9abab0c437d2883d70c68084a3f2dc843f3f2ab7ccb69a80db3fbf1ebff27e5e")
                .try_into()
                .unwrap(),
        ),
        genesis_hash: Hash256::from_bytes(
            unhex("4415cf74a204f2df610aa1e4ed737df220d93e1ad8a931e9533c00cf624c14ea")
                .try_into()
                .unwrap(),
        ),
    }
}

pub fn manifest() -> NetworkManifestV2 {
    NetworkManifestV2::decode_for(&fixture_bytes("manifest_hex"), independent_identity()).unwrap()
}

pub fn block(name: &str) -> BlockV2 {
    BlockV2::decode(
        manifest().config(),
        &fixture_bytes(&format!("{name}_block")),
    )
    .unwrap()
}

pub fn main_chain(height: u64) -> ChainV2 {
    assert!(height <= 4);
    let manifest = manifest();
    let mut chain = ChainV2::from_genesis(&manifest, block("block0")).unwrap();
    for height in 1..=height {
        chain.append(block(&format!("block{height}"))).unwrap();
    }
    chain
}

pub fn fork_prefix(height: u64) -> ChainV2 {
    assert!((1..=3).contains(&height));
    let mut chain = main_chain(1);
    for height in 2..=height {
        chain.append(block(&format!("fork{height}"))).unwrap();
    }
    chain
}

pub fn fork_chain() -> ChainV2 {
    fork_prefix(3)
}
