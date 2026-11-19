//! MSS2 comparisons against Python-derived bytes frozen before these tests ran.
//! The retained MB-02C fixture supplies complete synthetic histories, including
//! an actual membership fork. No device input, personal data or private key is
//! used. Structural metadata acceptance never replaces full chain replay.

use std::collections::BTreeMap;

use mirror_chain::v2::{
    ChainReplayExpectationV2, ChainV2, MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2, ManifestIdentityV2,
    NetworkManifestV2,
    manifest::{MANIFEST_FIXED_LEN_V2, MAX_MANIFEST_BYTES_V2},
};
use mirror_core::v2::MIN_ENCODED_BLOCK_LEN_V2;
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::selection::{
    MAX_STORAGE_READ_BYTES_V2, STORAGE_SELECTION_LEN_V2, STORAGE_SELECTION_MAGIC_V2,
    STORAGE_SELECTION_PROFILE_V2, STORAGE_SELECTION_VERSION_V2, SelectionErrorV2,
    SelectionRecordV2,
};

fn values(source: &'static str) -> BTreeMap<&'static str, &'static str> {
    source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split_once('=')
                .expect("well-formed independent fixture")
        })
        .collect()
}

fn storage_values() -> BTreeMap<&'static str, &'static str> {
    values(include_str!("vectors/storage_v2.txt"))
}

fn chain_values() -> BTreeMap<&'static str, &'static str> {
    values(include_str!(
        "../../mirror-chain/tests/vectors/chain_v2.txt"
    ))
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

fn hash(value: &str) -> Hash256 {
    Hash256::from_bytes(unhex(value).try_into().unwrap())
}

fn checkpoint(name: &str) -> ChainReplayExpectationV2 {
    let values = storage_values();
    ChainReplayExpectationV2 {
        config_hash: hash(values["config_hash"]),
        genesis_hash: hash(values["genesis_hash"]),
        height: values[format!("{name}_height").as_str()].parse().unwrap(),
        tip_hash: hash(values[format!("{name}_tip_hash").as_str()]),
        ledger_root: hash(values[format!("{name}_ledger_root").as_str()]),
    }
}

fn encoded_record(name: &str) -> Vec<u8> {
    unhex(storage_values()[format!("{name}_selection_hex").as_str()])
}

fn manifest_bytes() -> Vec<u8> {
    unhex(chain_values()["manifest_hex"])
}

fn encoded_blocks(name: &str) -> Vec<Vec<u8>> {
    let values = chain_values();
    let labels: &[&str] = match name {
        "genesis" => &["block0"],
        "main" => &["block0", "block1", "block2", "block3", "block4"],
        "fork" => &["block0", "block1", "fork2", "fork3"],
        _ => panic!("unknown synthetic history"),
    };
    labels
        .iter()
        .map(|name| unhex(values[format!("{name}_block").as_str()]))
        .collect()
}

fn accepted_chain(name: &str) -> ChainV2 {
    let expected = checkpoint(name);
    let manifest = NetworkManifestV2::decode_for(
        &manifest_bytes(),
        ManifestIdentityV2 {
            config_hash: expected.config_hash,
            genesis_hash: expected.genesis_hash,
        },
    )
    .unwrap();
    let encoded = encoded_blocks(name);
    let borrowed: Vec<_> = encoded.iter().map(Vec::as_slice).collect();
    ChainV2::replay_encoded(&manifest, &borrowed, expected).unwrap()
}

/// Deliberately recompute integrity so negative tests exercise semantic guards,
/// not just accidental-corruption detection. This is not an authorization key.
fn rechecksum(bytes: &mut [u8]) {
    assert_eq!(bytes.len(), STORAGE_SELECTION_LEN_V2);
    let mut input = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    input.extend_from_slice(&bytes[..200]);
    bytes[200..].copy_from_slice(sha256d(&input).as_bytes());
}

fn assert_vector(name: &str) {
    let values = storage_values();
    let chain = accepted_chain(name);
    let record = SelectionRecordV2::from_chain(&chain).unwrap();
    assert_eq!(record.encode().as_slice(), encoded_record(name));
    assert_eq!(record.checkpoint(), checkpoint(name));
    assert_eq!(record.generation(), 0);
    assert_eq!(
        record.block_count(),
        values[format!("{name}_block_count").as_str()]
            .parse::<u32>()
            .unwrap()
    );
    assert_eq!(
        record.manifest_len(),
        values["manifest_len"].parse::<u32>().unwrap()
    );
    assert_eq!(
        record.blocks_bytes(),
        values[format!("{name}_blocks_bytes").as_str()]
            .parse::<u64>()
            .unwrap()
    );
    assert_eq!(
        record.manifest_checksum(),
        hash(values["manifest_checksum"])
    );
    assert_eq!(
        &record.encode()[200..],
        unhex(values[format!("{name}_record_checksum").as_str()])
    );
    record.verify_manifest_bytes(&manifest_bytes()).unwrap();
    let decoded = SelectionRecordV2::decode_for(&record.encode(), checkpoint(name)).unwrap();
    assert_eq!(decoded, record);
    assert_eq!(decoded.encode(), record.encode());
}

#[test]
fn genesis_selection_matches_independent_vector() {
    assert_vector("genesis");
}

#[test]
fn mixed_history_selection_matches_independent_vector() {
    assert_vector("main");
}

#[test]
fn membership_fork_selection_matches_independent_vector() {
    assert_vector("fork");
}

#[test]
fn selection_length_and_complete_read_budget_match_reference() {
    let values = storage_values();
    assert_eq!(STORAGE_SELECTION_MAGIC_V2, *b"MSS2");
    assert_eq!(STORAGE_SELECTION_VERSION_V2, 2);
    assert_eq!(STORAGE_SELECTION_PROFILE_V2, 1);
    assert_eq!(STORAGE_SELECTION_LEN_V2, 232);
    assert_eq!(
        MAX_STORAGE_READ_BYTES_V2,
        values["max_encoded_read_bytes"].parse::<usize>().unwrap()
    );
    assert_eq!(MAX_STORAGE_READ_BYTES_V2, 19_559_084);
}

#[test]
fn every_truncated_record_and_trailing_bytes_are_rejected() {
    let bytes = encoded_record("main");
    for length in 0..STORAGE_SELECTION_LEN_V2 {
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes[..length], checkpoint("main")),
            Err(SelectionErrorV2::InvalidLength),
            "length {length}"
        );
    }
    let mut extra = bytes;
    extra.push(0);
    assert_eq!(
        SelectionRecordV2::decode_for(&extra, checkpoint("main")),
        Err(SelectionErrorV2::InvalidLength)
    );
}

#[test]
fn wrong_magic_version_and_profile_are_rejected() {
    for (offset, replacement, error) in [
        (0, 0, SelectionErrorV2::InvalidMagic),
        (4, 3, SelectionErrorV2::UnsupportedVersion(3)),
        (6, 2, SelectionErrorV2::UnsupportedProfile(2)),
    ] {
        let mut bytes = encoded_record("main");
        bytes[offset] = replacement;
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(error)
        );
    }
}

#[test]
fn checksum_covers_each_payload_field_and_its_own_bytes() {
    for offset in [8, 16, 48, 80, 88, 120, 152, 156, 160, 168, 199, 200, 231] {
        let mut bytes = encoded_record("main");
        bytes[offset] ^= 1;
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(SelectionErrorV2::ChecksumMismatch),
            "offset {offset}"
        );
    }
}

#[test]
fn checksum_correct_nonzero_generation_is_rejected() {
    for generation in [1u64, u64::MAX] {
        let mut bytes = encoded_record("main");
        bytes[8..16].copy_from_slice(&generation.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(SelectionErrorV2::UnsupportedGeneration(generation))
        );
    }
}

#[test]
fn checksum_correct_zero_or_excess_block_count_is_rejected() {
    for count in [0u32, MAX_CHAIN_BLOCKS_V2 as u32 + 1, u32::MAX] {
        let mut bytes = encoded_record("main");
        bytes[152..156].copy_from_slice(&count.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(SelectionErrorV2::InvalidBlockCount)
        );
    }
}

#[test]
fn checksum_correct_height_count_disagreement_is_rejected_without_overflow() {
    for height in [0u64, 3, 5, u64::MAX] {
        let mut bytes = encoded_record("main");
        bytes[80..88].copy_from_slice(&height.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(SelectionErrorV2::HeightCountMismatch)
        );
    }
}

#[test]
fn checksum_correct_manifest_size_outside_bounds_is_rejected() {
    for length in [
        0u32,
        MANIFEST_FIXED_LEN_V2 as u32 - 1,
        MAX_MANIFEST_BYTES_V2 as u32 + 1,
        u32::MAX,
    ] {
        let mut bytes = encoded_record("main");
        bytes[156..160].copy_from_slice(&length.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(SelectionErrorV2::InvalidManifestLength)
        );
    }
}

#[test]
fn checksum_correct_block_byte_sum_outside_bounds_is_rejected() {
    for length in [
        0u64,
        5 * MIN_ENCODED_BLOCK_LEN_V2 as u64 - 1,
        MAX_CHAIN_BYTES_V2 as u64 + 1,
        u64::MAX,
    ] {
        let mut bytes = encoded_record("main");
        bytes[160..168].copy_from_slice(&length.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(SelectionErrorV2::InvalidBlocksBytes)
        );
    }
}

#[test]
fn inclusive_structural_limits_do_not_require_unbounded_allocations() {
    // These are metadata-boundary checks, not claims that the fabricated sizes
    // describe the fixture's real files. Actual reads/replay must cross-check.
    for manifest_len in [MANIFEST_FIXED_LEN_V2 as u32, MAX_MANIFEST_BYTES_V2 as u32] {
        for block_bytes in [
            (MAX_CHAIN_BLOCKS_V2 * MIN_ENCODED_BLOCK_LEN_V2) as u64,
            MAX_CHAIN_BYTES_V2 as u64,
        ] {
            let mut bytes = encoded_record("main");
            let mut expected = checkpoint("main");
            expected.height = MAX_CHAIN_BLOCKS_V2 as u64 - 1;
            bytes[80..88].copy_from_slice(&expected.height.to_le_bytes());
            bytes[152..156].copy_from_slice(&(MAX_CHAIN_BLOCKS_V2 as u32).to_le_bytes());
            bytes[156..160].copy_from_slice(&manifest_len.to_le_bytes());
            bytes[160..168].copy_from_slice(&block_bytes.to_le_bytes());
            rechecksum(&mut bytes);
            let record = SelectionRecordV2::decode_for(&bytes, expected).unwrap();
            assert_eq!(record.block_count(), MAX_CHAIN_BLOCKS_V2 as u32);
            assert_eq!(record.manifest_len(), manifest_len);
            assert_eq!(record.blocks_bytes(), block_bytes);
        }
    }
}

#[test]
fn external_configuration_pin_is_required() {
    let mut expected = checkpoint("main");
    expected.config_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("main"), expected),
        Err(SelectionErrorV2::WrongConfiguration)
    );
}

#[test]
fn external_genesis_pin_is_required() {
    let mut expected = checkpoint("main");
    expected.genesis_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("main"), expected),
        Err(SelectionErrorV2::WrongGenesis)
    );
}

#[test]
fn external_height_pin_is_required() {
    let mut expected = checkpoint("main");
    expected.height = 3;
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("main"), expected),
        Err(SelectionErrorV2::WrongHeight)
    );
}

#[test]
fn external_tip_pin_is_required() {
    let mut expected = checkpoint("main");
    expected.tip_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("main"), expected),
        Err(SelectionErrorV2::WrongTip)
    );
}

#[test]
fn external_ledger_root_pin_is_required() {
    let mut expected = checkpoint("main");
    expected.ledger_root = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("main"), expected),
        Err(SelectionErrorV2::WrongLedgerRoot)
    );
}

#[test]
fn recomputing_checksum_does_not_authorize_changed_checkpoint_fields() {
    for (offset, error) in [
        (16, SelectionErrorV2::WrongConfiguration),
        (48, SelectionErrorV2::WrongGenesis),
        (88, SelectionErrorV2::WrongTip),
        (120, SelectionErrorV2::WrongLedgerRoot),
    ] {
        let mut bytes = encoded_record("main");
        bytes[offset] ^= 1;
        rechecksum(&mut bytes);
        assert_eq!(
            SelectionRecordV2::decode_for(&bytes, checkpoint("main")),
            Err(error)
        );
    }
}

#[test]
fn old_valid_selection_is_rejected_against_later_checkpoint() {
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("genesis"), checkpoint("main")),
        Err(SelectionErrorV2::WrongHeight)
    );
}

#[test]
fn valid_membership_fork_is_rejected_against_other_same_height_branch() {
    let values = chain_values();
    let mut expected = checkpoint("fork");
    expected.tip_hash = hash(values["block3_hash"]);
    expected.ledger_root = hash(values["state3_ledger_root"]);
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded_record("fork"), expected),
        Err(SelectionErrorV2::WrongTip)
    );
}

#[test]
fn manifest_truncation_or_trailing_bytes_are_rejected_before_hashing() {
    let record =
        SelectionRecordV2::decode_for(&encoded_record("main"), checkpoint("main")).unwrap();
    let mut manifest = manifest_bytes();
    assert_eq!(
        record.verify_manifest_bytes(&manifest[..manifest.len() - 1]),
        Err(SelectionErrorV2::ManifestLengthMismatch)
    );
    manifest.push(0);
    assert_eq!(
        record.verify_manifest_bytes(&manifest),
        Err(SelectionErrorV2::ManifestLengthMismatch)
    );
}

#[test]
fn altered_manifest_is_rejected_at_the_same_declared_length() {
    let record =
        SelectionRecordV2::decode_for(&encoded_record("main"), checkpoint("main")).unwrap();
    let mut manifest = manifest_bytes();
    manifest[100] ^= 1;
    assert_eq!(
        record.verify_manifest_bytes(&manifest),
        Err(SelectionErrorV2::ManifestChecksumMismatch)
    );
}

#[test]
fn manifest_checksum_is_cross_checked_against_actual_bytes() {
    let mut bytes = encoded_record("main");
    bytes[168] ^= 1;
    rechecksum(&mut bytes);
    let record = SelectionRecordV2::decode_for(&bytes, checkpoint("main")).unwrap();
    assert_eq!(
        record.verify_manifest_bytes(&manifest_bytes()),
        Err(SelectionErrorV2::ManifestChecksumMismatch)
    );
}
