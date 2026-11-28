//! MSR2 exact-byte comparisons against Python vectors generated before Rust
//! comparison. Frozen MSS2 semantics remain the embedded snapshot contract.

mod support_v2;

use std::collections::BTreeMap;

use mirror_chain::v2::{ChainV2, MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2};
use mirror_core::v2::MIN_ENCODED_BLOCK_LEN_V2;
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{
    append_selection::{AppendCheckpointV2, AppendSelectionErrorV2, AppendSelectionV2},
    replacement_selection::{
        REPLACEMENT_SELECTION_LEN_V2, REPLACEMENT_SELECTION_MAGIC_V2,
        REPLACEMENT_SELECTION_PROFILE_V2, REPLACEMENT_SELECTION_VERSION_V2,
        ReplacementCheckpointV2, ReplacementSelectionErrorV2, ReplacementSelectionV2,
    },
    selection::{SelectionErrorV2, SelectionRecordV2},
};

use support_v2::{fixture_bytes, fork_chain, main_chain};

fn values() -> BTreeMap<&'static str, &'static str> {
    include_str!("vectors/replacement_storage_v2.txt")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split_once('=').expect("independent fixture key/value"))
        .collect()
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

fn encoded(name: &str) -> Vec<u8> {
    unhex(values()[format!("{name}_selection_hex").as_str()])
}

fn chain(name: &str) -> ChainV2 {
    match name {
        "genesis" => main_chain(0),
        "main1" => main_chain(1),
        "main2" => main_chain(2),
        "main3" => main_chain(3),
        "main4" | "initialized_main" | "main4_after_replacement" => main_chain(4),
        "fork" | "initialized_fork" => fork_chain(),
        _ => panic!("unknown fixture branch"),
    }
}

fn expected(name: &str) -> ReplacementCheckpointV2 {
    let values = values();
    ReplacementCheckpointV2 {
        generation: values[format!("{name}_generation").as_str()]
            .parse()
            .unwrap(),
        chain: chain(name).checkpoint(),
    }
}

fn assert_vector(name: &str) {
    let values = values();
    let expected = expected(name);
    let record = ReplacementSelectionV2::from_chain(&chain(name), expected.generation).unwrap();
    assert_eq!(record.encode().as_slice(), encoded(name));
    assert_eq!(record.checkpoint(), expected);
    assert_eq!(record.generation(), expected.generation);
    assert_eq!(record.snapshot().generation(), 0);
    assert_eq!(
        record.snapshot().encode().as_slice(),
        unhex(values[format!("{name}_snapshot_hex").as_str()])
    );
    assert_eq!(
        record.snapshot().block_count(),
        values[format!("{name}_block_count").as_str()]
            .parse::<u32>()
            .unwrap()
    );
    assert_eq!(
        record.snapshot().blocks_bytes(),
        values[format!("{name}_blocks_bytes").as_str()]
            .parse::<u64>()
            .unwrap()
    );
    assert_eq!(
        &record.encode()[248..],
        unhex(values[format!("{name}_record_checksum").as_str()])
    );
    assert_eq!(
        ReplacementSelectionV2::decode_for(&record.encode(), expected).unwrap(),
        record
    );
    record
        .verify_manifest_bytes(&fixture_bytes("manifest_hex"))
        .unwrap();
}

// Recompute checksums deliberately so negative tests reach semantic validation.
fn rechecksum(bytes: &mut [u8]) {
    assert_eq!(bytes.len(), REPLACEMENT_SELECTION_LEN_V2);
    let mut input = b"MIRROR_STORAGE_REPLACEMENT_SELECTION_V2".to_vec();
    input.extend_from_slice(&bytes[..248]);
    bytes[248..].copy_from_slice(sha256d(&input).as_bytes());
}

fn rechecksum_snapshot(bytes: &mut [u8]) {
    let mut input = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    input.extend_from_slice(&bytes[16..216]);
    bytes[216..248].copy_from_slice(sha256d(&input).as_bytes());
    rechecksum(bytes);
}

#[test]
fn genesis_generation_zero_matches_independent_vector() {
    assert_vector("genesis");
}

#[test]
fn bounded_generation_metadata_matches_independent_vectors() {
    for name in ["main1", "main2", "main3", "main4"] {
        assert_vector(name);
    }
}

#[test]
fn initialization_at_existing_height_starts_at_generation_zero() {
    assert_vector("initialized_main");
    let published = encoded("main4");
    let initialized = encoded("initialized_main");
    assert_eq!(&published[16..248], &initialized[16..248]);
    assert_ne!(published, initialized);
}

#[test]
fn different_membership_branch_matches_independent_vectors() {
    assert_vector("fork");
    assert_vector("initialized_fork");
}

#[test]
fn one_generation_can_select_existing_height_four_metadata() {
    // This codec describes an endpoint; it does not prove a publication occurred.
    // A replacement can advance height by more than one while generation advances
    // only once. The independent fixture pins that distinction.
    assert_vector("main4_after_replacement");
    let bytes = encoded("main4_after_replacement");
    assert_eq!(&bytes[8..16], &1u64.to_le_bytes());
    assert_eq!(&bytes[16..248], &encoded("main4")[16..248]);
    assert_ne!(bytes, encoded("main4"));
}

#[test]
fn exact_wrapper_fields_and_unchanged_snapshot_are_pinned() {
    assert_eq!(REPLACEMENT_SELECTION_MAGIC_V2, *b"MSR2");
    assert_eq!(REPLACEMENT_SELECTION_VERSION_V2, 2);
    assert_eq!(REPLACEMENT_SELECTION_PROFILE_V2, 3);
    assert_eq!(REPLACEMENT_SELECTION_LEN_V2, 280);
    let record = ReplacementSelectionV2::decode_for(&encoded("main4"), expected("main4")).unwrap();
    let bytes = record.encode();
    assert_eq!(&bytes[..4], b"MSR2");
    assert_eq!(&bytes[4..8], &[2, 0, 3, 0]);
    assert_eq!(&bytes[8..16], &4u64.to_le_bytes());
    assert_eq!(
        &bytes[16..248],
        SelectionRecordV2::from_chain(&main_chain(4))
            .unwrap()
            .encode()
    );
}

#[test]
fn every_truncation_and_trailing_bytes_are_rejected() {
    let bytes = encoded("main4");
    let expected = expected("main4");
    for length in 0..REPLACEMENT_SELECTION_LEN_V2 {
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes[..length], expected),
            Err(ReplacementSelectionErrorV2::InvalidLength),
            "length {length}",
        );
    }
    let mut extra = bytes;
    extra.push(0);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&extra, expected),
        Err(ReplacementSelectionErrorV2::InvalidLength),
    );
}

#[test]
fn wrong_magic_version_and_profile_are_rejected() {
    for (offset, replacement, error) in [
        (0, 0, ReplacementSelectionErrorV2::InvalidMagic),
        (4, 3, ReplacementSelectionErrorV2::UnsupportedVersion(3)),
        (6, 1, ReplacementSelectionErrorV2::UnsupportedProfile(1)),
    ] {
        let mut bytes = encoded("main4");
        bytes[offset] = replacement;
        rechecksum(&mut bytes);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(error)
        );
    }
}

#[test]
fn full_u16_version_and_profile_are_checked() {
    for version in [0u16, 1, 3, 258, u16::MAX] {
        let mut bytes = encoded("main4");
        bytes[4..6].copy_from_slice(&version.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(ReplacementSelectionErrorV2::UnsupportedVersion(version)),
        );
    }
    for profile in [0u16, 1, 2, 4, 259, u16::MAX] {
        let mut bytes = encoded("main4");
        bytes[6..8].copy_from_slice(&profile.to_le_bytes());
        rechecksum(&mut bytes);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(ReplacementSelectionErrorV2::UnsupportedProfile(profile)),
        );
    }
}

#[test]
fn outer_checksum_covers_every_generation_snapshot_and_checksum_byte() {
    let original = encoded("main4");
    let expected = expected("main4");
    for offset in 8..REPLACEMENT_SELECTION_LEN_V2 {
        let mut bytes = original.clone();
        bytes[offset] ^= 1;
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected),
            Err(ReplacementSelectionErrorV2::ChecksumMismatch),
            "offset {offset}",
        );
    }
}

#[test]
fn old_snapshot_and_append_domains_do_not_match_replacement_domain() {
    for domain in [
        b"MIRROR_STORAGE_SELECTION_V2".as_slice(),
        b"MIRROR_STORAGE_APPEND_SELECTION_V2".as_slice(),
    ] {
        let mut bytes = encoded("main4");
        let mut input = domain.to_vec();
        input.extend_from_slice(&bytes[..248]);
        bytes[248..].copy_from_slice(sha256d(&input).as_bytes());
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(ReplacementSelectionErrorV2::ChecksumMismatch),
        );
    }
}

#[test]
fn valid_outer_checksum_does_not_hide_corrupted_embedded_checksum() {
    let mut bytes = encoded("main4");
    bytes[216] ^= 1;
    rechecksum(&mut bytes);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::ChecksumMismatch
        )),
    );
}

#[test]
fn nested_framing_still_requires_original_mss2_profile_one() {
    for (offset, replacement, error) in [
        (16, 0, SelectionErrorV2::InvalidMagic),
        (20, 3, SelectionErrorV2::UnsupportedVersion(3)),
        (22, 2, SelectionErrorV2::UnsupportedProfile(2)),
    ] {
        let mut bytes = encoded("main4");
        bytes[offset] = replacement;
        rechecksum_snapshot(&mut bytes);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(ReplacementSelectionErrorV2::Snapshot(error)),
        );
    }
}

#[test]
fn nonzero_embedded_generation_remains_rejected() {
    for generation in [1u64, u64::MAX] {
        let mut bytes = encoded("main4");
        bytes[24..32].copy_from_slice(&generation.to_le_bytes());
        rechecksum_snapshot(&mut bytes);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(ReplacementSelectionErrorV2::Snapshot(
                SelectionErrorV2::UnsupportedGeneration(generation)
            )),
        );
    }
}

#[test]
fn nested_count_height_and_length_limits_cannot_be_bypassed_by_wrapper() {
    for (offset, bytes, error) in [
        (
            168,
            0u32.to_le_bytes().to_vec(),
            SelectionErrorV2::InvalidBlockCount,
        ),
        (
            168,
            4097u32.to_le_bytes().to_vec(),
            SelectionErrorV2::InvalidBlockCount,
        ),
        (
            96,
            u64::MAX.to_le_bytes().to_vec(),
            SelectionErrorV2::HeightCountMismatch,
        ),
        (
            172,
            0u32.to_le_bytes().to_vec(),
            SelectionErrorV2::InvalidManifestLength,
        ),
        (
            172,
            u32::MAX.to_le_bytes().to_vec(),
            SelectionErrorV2::InvalidManifestLength,
        ),
        (
            176,
            0u64.to_le_bytes().to_vec(),
            SelectionErrorV2::InvalidBlocksBytes,
        ),
        (
            176,
            u64::MAX.to_le_bytes().to_vec(),
            SelectionErrorV2::InvalidBlocksBytes,
        ),
    ] {
        let mut encoded = encoded("main4");
        encoded[offset..offset + bytes.len()].copy_from_slice(&bytes);
        rechecksum_snapshot(&mut encoded);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&encoded, expected("main4")),
            Err(ReplacementSelectionErrorV2::Snapshot(error)),
        );
    }
}

#[test]
fn generation_from_zero_through_selected_height_is_structurally_valid() {
    // Structural metadata acceptance does not claim these publications occurred.
    let chain = main_chain(4);
    for generation in 0..=chain.height() {
        let record = ReplacementSelectionV2::from_chain(&chain, generation).unwrap();
        assert_eq!(
            ReplacementSelectionV2::decode_for(&record.encode(), record.checkpoint()).unwrap(),
            record
        );
    }
}

#[test]
fn constructor_rejects_generation_above_selected_height() {
    let chain = main_chain(4);
    for generation in [5u64, u64::MAX] {
        assert_eq!(
            ReplacementSelectionV2::from_chain(&chain, generation),
            Err(ReplacementSelectionErrorV2::GenerationExceedsHeight),
        );
    }
}

#[test]
fn checksum_correct_excess_generation_is_rejected_without_overflow() {
    for generation in [5u64, u64::MAX] {
        let mut bytes = encoded("main4");
        bytes[8..16].copy_from_slice(&generation.to_le_bytes());
        rechecksum(&mut bytes);
        let mut expected = expected("main4");
        expected.generation = generation;
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected),
            Err(ReplacementSelectionErrorV2::GenerationExceedsHeight),
        );
    }
}

#[test]
fn maximum_chain_height_bounds_generation_without_large_allocation() {
    // Sizes here are metadata-only; full file reads and replay must still agree.
    let mut bytes = encoded("main4");
    let mut expected = expected("main4");
    expected.chain.height = MAX_CHAIN_BLOCKS_V2 as u64 - 1;
    expected.generation = expected.chain.height;
    bytes[8..16].copy_from_slice(&expected.generation.to_le_bytes());
    bytes[96..104].copy_from_slice(&expected.chain.height.to_le_bytes());
    bytes[168..172].copy_from_slice(&(MAX_CHAIN_BLOCKS_V2 as u32).to_le_bytes());
    for block_bytes in [
        (MAX_CHAIN_BLOCKS_V2 * MIN_ENCODED_BLOCK_LEN_V2) as u64,
        MAX_CHAIN_BYTES_V2 as u64,
    ] {
        bytes[176..184].copy_from_slice(&block_bytes.to_le_bytes());
        rechecksum_snapshot(&mut bytes);
        let record = ReplacementSelectionV2::decode_for(&bytes, expected).unwrap();
        assert_eq!(record.generation(), 4095);
    }
}

#[test]
fn independent_generation_pin_is_required() {
    for generation in [0u64, 3, 5, u64::MAX] {
        let mut expected = expected("main4");
        expected.generation = generation;
        assert_eq!(
            ReplacementSelectionV2::decode_for(&encoded("main4"), expected),
            Err(ReplacementSelectionErrorV2::WrongGeneration),
        );
    }
}

#[test]
fn validation_order_checks_nested_record_before_generation() {
    let mut bytes = encoded("main4");
    bytes[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    rechecksum(&mut bytes);
    let mut expected = expected("main4");
    expected.chain.config_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&bytes, expected),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongConfiguration
        )),
    );
    expected.chain = main_chain(4).checkpoint();
    // The record's generation bound is checked before equality to a caller pin.
    assert_eq!(
        ReplacementSelectionV2::decode_for(&bytes, expected),
        Err(ReplacementSelectionErrorV2::GenerationExceedsHeight),
    );
}

#[test]
fn independent_configuration_pin_is_required() {
    let mut expected = expected("main4");
    expected.chain.config_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("main4"), expected),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongConfiguration
        )),
    );
}

#[test]
fn independent_genesis_pin_is_required() {
    let mut expected = expected("main4");
    expected.chain.genesis_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("main4"), expected),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongGenesis
        )),
    );
}

#[test]
fn independent_height_pin_is_required() {
    let mut expected = expected("main4");
    expected.chain.height = 3;
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("main4"), expected),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongHeight
        )),
    );
}

#[test]
fn independent_tip_pin_is_required() {
    let mut expected = expected("main4");
    expected.chain.tip_hash = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("main4"), expected),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongTip
        )),
    );
}

#[test]
fn independent_ledger_root_pin_is_required() {
    let mut expected = expected("main4");
    expected.chain.ledger_root = Hash256::from_bytes([0xff; 32]);
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("main4"), expected),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongLedgerRoot
        )),
    );
}

#[test]
fn correct_checksums_cannot_authorize_changed_selection_fields() {
    for (offset, error) in [
        (32, SelectionErrorV2::WrongConfiguration),
        (64, SelectionErrorV2::WrongGenesis),
        (104, SelectionErrorV2::WrongTip),
        (136, SelectionErrorV2::WrongLedgerRoot),
    ] {
        let mut bytes = encoded("main4");
        bytes[offset] ^= 1;
        rechecksum_snapshot(&mut bytes);
        assert_eq!(
            ReplacementSelectionV2::decode_for(&bytes, expected("main4")),
            Err(ReplacementSelectionErrorV2::Snapshot(error)),
        );
    }
}

#[test]
fn old_valid_selection_is_rejected_against_later_checkpoint() {
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("genesis"), expected("main4")),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongHeight
        )),
    );
}

#[test]
fn valid_same_height_membership_fork_does_not_match_selected_branch() {
    assert_eq!(
        ReplacementSelectionV2::decode_for(&encoded("fork"), expected("main3")),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::WrongTip
        )),
    );
}

#[test]
fn old_and_new_selection_decoders_do_not_silently_adopt_other_format() {
    let chain = main_chain(4);
    let old = SelectionRecordV2::from_chain(&chain).unwrap();
    assert_eq!(
        ReplacementSelectionV2::decode_for(&old.encode(), expected("main4")),
        Err(ReplacementSelectionErrorV2::InvalidLength),
    );
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded("main4"), chain.checkpoint()),
        Err(SelectionErrorV2::InvalidLength),
    );
    assert_eq!(
        SelectionRecordV2::decode_for(&encoded("main4")[..232], chain.checkpoint()),
        Err(SelectionErrorV2::InvalidMagic),
    );
    let append = AppendSelectionV2::from_chain(&chain, 4).unwrap();
    assert_eq!(
        ReplacementSelectionV2::decode_for(&append.encode(), expected("main4")),
        Err(ReplacementSelectionErrorV2::InvalidMagic),
    );
    assert_eq!(
        AppendSelectionV2::decode_for(
            &encoded("main4"),
            AppendCheckpointV2 {
                generation: 4,
                chain: chain.checkpoint(),
            },
        ),
        Err(AppendSelectionErrorV2::InvalidMagic),
    );
}

#[test]
fn exact_manifest_length_and_bytes_are_still_required() {
    let record = ReplacementSelectionV2::decode_for(&encoded("main4"), expected("main4")).unwrap();
    let manifest = fixture_bytes("manifest_hex");
    record.verify_manifest_bytes(&manifest).unwrap();
    assert_eq!(
        record.verify_manifest_bytes(&manifest[..manifest.len() - 1]),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::ManifestLengthMismatch
        )),
    );
    let mut trailing = manifest.clone();
    trailing.push(0);
    assert_eq!(
        record.verify_manifest_bytes(&trailing),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::ManifestLengthMismatch
        )),
    );
    let mut corrupted = manifest;
    corrupted[100] ^= 1;
    assert_eq!(
        record.verify_manifest_bytes(&corrupted),
        Err(ReplacementSelectionErrorV2::Snapshot(
            SelectionErrorV2::ManifestChecksumMismatch
        )),
    );
}
