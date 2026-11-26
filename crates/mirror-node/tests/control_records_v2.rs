//! Pure control codecs: independent vectors, complete execution and local pins.

#[path = "../../mirror-network/tests/support_history_v2/mod.rs"]
mod support_history_v2;

use std::{collections::BTreeMap, sync::OnceLock};

use mirror_chain::v2::{ChainErrorV2, ChainV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_network::v2::{HistoryErrorV2, HistoryFramesV2};
use mirror_node::v2::records::{
    CONTROL_RECORD_LEN_V2, ConfirmedSideV2, ControlRecordErrorV2, ControlRecordV2,
    INTENT_FIXED_LEN_V2, INTENT_HEADER_LEN_V2, IntentRecordV2, MAX_INTENT_BYTES_V2,
};
use mirror_protocol::v2::MAX_HISTORY_WIRE_BYTES_V2;
use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementPlanErrorV2};
use support_history_v2::{fixture_bytes, fork_chain, independent_identity, main_chain, unhex};

const CONTROL_DOMAIN: &[u8] = b"MIRROR_NODE_CONTROL_RECORD_V2";
const INTENT_DOMAIN: &[u8] = b"MIRROR_NODE_REPLACEMENT_INTENT_V2";

fn vectors() -> &'static BTreeMap<&'static str, &'static str> {
    static VALUES: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    VALUES.get_or_init(|| {
        include_str!("vectors/node_control_v2.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_once('=').unwrap())
            .collect()
    })
}

fn vector(name: &str) -> Vec<u8> {
    unhex(vectors()[name])
}
fn pin(chain: &ChainV2, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}
fn control(chain: &ChainV2, generation: u64) -> ControlRecordV2 {
    ControlRecordV2::from_chain(independent_identity(), chain, generation).unwrap()
}
fn intent(old: ChainV2, generation: u64, candidate: ChainV2) -> IntentRecordV2 {
    IntentRecordV2::new(
        independent_identity(),
        pin(&old, generation),
        old,
        candidate,
    )
    .unwrap()
}
fn checksum(domain: &[u8], prefix: &[u8]) -> Hash256 {
    let mut input = domain.to_vec();
    input.extend_from_slice(prefix);
    sha256d(&input)
}
fn resign(bytes: &mut [u8], domain: &[u8]) {
    let offset = bytes.len() - 32;
    let hash = checksum(domain, &bytes[..offset]);
    bytes[offset..].copy_from_slice(hash.as_bytes());
}
fn history_bytes(chain: &ChainV2) -> Vec<u8> {
    HistoryFramesV2::new(chain)
        .unwrap()
        .flat_map(Result::unwrap)
        .collect()
}
fn replace_histories(bytes: &mut Vec<u8>, old: &[u8], candidate: &[u8]) {
    bytes.truncate(INTENT_HEADER_LEN_V2);
    bytes[936..944].copy_from_slice(&(old.len() as u64).to_le_bytes());
    bytes[944..952].copy_from_slice(&(candidate.len() as u64).to_le_bytes());
    bytes.extend_from_slice(old);
    bytes.extend_from_slice(candidate);
    bytes.extend_from_slice(&[0; 32]);
    resign(bytes, INTENT_DOMAIN);
}
fn history_ranges(bytes: &[u8]) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let old_len = u64::from_le_bytes(bytes[936..944].try_into().unwrap()) as usize;
    (952..952 + old_len, 952 + old_len..bytes.len() - 32)
}

#[test]
fn confirmed_records_match_six_independently_generated_vectors() {
    for (name, chain, generation) in [
        ("genesis_g0", main_chain(0), 0),
        ("main3_g0", main_chain(3), 0),
        ("main3_g3", main_chain(3), 3),
        ("main4_g1", main_chain(4), 1),
        ("main4_g4", main_chain(4), 4),
        ("fork3_g0", fork_chain(), 0),
    ] {
        let record = control(&chain, generation);
        let bytes = vector(&format!("{name}_control"));
        assert_eq!(record.encode().as_slice(), bytes);
        assert_eq!(
            ControlRecordV2::decode_for(&bytes, independent_identity()).unwrap(),
            record
        );
        assert_eq!(record.checkpoint(), pin(&chain, generation));
        assert_eq!(record.identity(), independent_identity());
    }
    assert_eq!(CONTROL_RECORD_LEN_V2, 464);
}

#[test]
fn confirmed_record_rejects_every_truncation_and_trailing_byte() {
    let bytes = vector("main3_g0_control");
    for end in 0..bytes.len() {
        assert!(matches!(
            ControlRecordV2::decode_for(&bytes[..end], independent_identity()),
            Err(ControlRecordErrorV2::InvalidLength)
        ));
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(matches!(
        ControlRecordV2::decode_for(&extra, independent_identity()),
        Err(ControlRecordErrorV2::InvalidLength)
    ));
}

#[test]
fn confirmed_record_checks_magic_version_and_kind() {
    for (offset, expected) in [(0, 0), (4, 1), (6, 2)] {
        let mut bytes = vector("main3_g0_control");
        bytes[offset] ^= 0x10;
        let error = ControlRecordV2::decode_for(&bytes, independent_identity()).unwrap_err();
        assert!(matches!(
            (expected, error),
            (0, ControlRecordErrorV2::InvalidMagic)
                | (1, ControlRecordErrorV2::UnsupportedVersion(_))
                | (2, ControlRecordErrorV2::UnsupportedKind(_))
        ));
    }
}

#[test]
fn confirmed_record_checks_all_corruption_regions() {
    for offset in [
        8, 16, 48, 80, 88, 120, 152, 167, 168, 327, 399, 431, 432, 463,
    ] {
        let mut bytes = vector("main3_g0_control");
        bytes[offset] ^= 1;
        assert!(matches!(
            ControlRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::ChecksumMismatch)
        ));
    }
}

#[test]
fn confirmed_record_requires_independent_configuration_and_genesis() {
    let bytes = vector("main3_g0_control");
    let mut wrong = independent_identity();
    wrong.config_hash = Hash256::from_bytes([0; 32]);
    assert!(matches!(
        ControlRecordV2::decode_for(&bytes, wrong),
        Err(ControlRecordErrorV2::WrongIdentity)
    ));
    wrong = independent_identity();
    wrong.genesis_hash = Hash256::from_bytes([0; 32]);
    assert!(matches!(
        ControlRecordV2::decode_for(&bytes, wrong),
        Err(ControlRecordErrorV2::WrongIdentity)
    ));
}

#[test]
fn construction_checks_identity_and_local_generation_bound() {
    let chain = main_chain(3);
    let mut wrong = independent_identity();
    wrong.config_hash = Hash256::from_bytes([0; 32]);
    assert!(matches!(
        ControlRecordV2::from_chain(wrong, &chain, 0),
        Err(ControlRecordErrorV2::WrongIdentity)
    ));
    assert!(matches!(
        ControlRecordV2::from_chain(independent_identity(), &chain, 4),
        Err(ControlRecordErrorV2::Selection(_))
    ));
    assert!(ControlRecordV2::from_chain(independent_identity(), &chain, 3).is_ok());
}

#[test]
fn duplicated_complete_checkpoint_must_match_nested_msr2() {
    for offset in [8, 80, 88, 120] {
        let mut bytes = vector("main3_g0_control");
        bytes[offset] ^= 1;
        resign(&mut bytes, CONTROL_DOMAIN);
        assert!(matches!(
            ControlRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::Selection(_))
        ));
    }
}

#[test]
fn nested_selection_integrity_is_not_replaced_by_outer_checksum() {
    for offset in [160, 168, 328, 400, 431] {
        let mut bytes = vector("main3_g0_control");
        bytes[offset] ^= 1;
        resign(&mut bytes, CONTROL_DOMAIN);
        assert!(matches!(
            ControlRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::Selection(_))
        ));
    }
}

#[test]
fn intents_match_four_independent_complete_two_history_vectors() {
    for (name, old, generation) in [
        ("genesis_main4", main_chain(0), 0),
        ("main3_main4", main_chain(3), 0),
        ("fork3_main4", fork_chain(), 0),
        ("main3_g3_main4", main_chain(3), 3),
    ] {
        let candidate = main_chain(4);
        let value = intent(old.clone(), generation, candidate.clone());
        let bytes = vector(&format!("{name}_intent"));
        assert_eq!(value.encode().unwrap(), bytes);
        assert_eq!(
            bytes.len().to_string(),
            vectors()[&*format!("{name}_intent_len")]
        );
        assert_eq!(
            sha256d(&bytes).to_hex(),
            vectors()[&*format!("{name}_intent_sha256d")]
        );
        let loaded = IntentRecordV2::decode_for(&bytes, independent_identity()).unwrap();
        assert_eq!(loaded.old_chain(), &old);
        assert_eq!(loaded.candidate_chain(), &candidate);
        assert_eq!(loaded.old(), pin(&old, generation));
        assert_eq!(loaded.candidate(), pin(&candidate, generation + 1));
        assert_eq!(loaded.request().unwrap().candidate_chain(), &candidate);
        assert_eq!(loaded.encode().unwrap(), bytes);
    }
    assert_eq!(INTENT_HEADER_LEN_V2, 952);
    assert_eq!(INTENT_FIXED_LEN_V2, 984);
    assert_eq!(MAX_INTENT_BYTES_V2, 39_283_208);
}

#[test]
fn intent_confirmed_state_accepts_exact_old_or_candidate_only() {
    let value = intent(main_chain(3), 0, main_chain(4));
    assert_eq!(
        value.validate_confirmed(value.old_record()).unwrap(),
        ConfirmedSideV2::Old
    );
    assert_eq!(
        value.validate_confirmed(value.candidate_record()).unwrap(),
        ConfirmedSideV2::Candidate
    );
    for other in [
        control(&main_chain(3), 1),
        control(&main_chain(4), 0),
        control(&fork_chain(), 0),
    ] {
        assert!(matches!(
            value.validate_confirmed(&other),
            Err(ControlRecordErrorV2::ConflictingConfirmed)
        ));
    }
}

#[test]
fn intent_requires_strict_preference_not_equal_accounts_or_ties() {
    for (old, candidate) in [
        (main_chain(3), main_chain(3)),
        (main_chain(3), fork_chain()),
        (main_chain(4), main_chain(3)),
    ] {
        assert!(matches!(
            IntentRecordV2::new(independent_identity(), pin(&old, 0), old, candidate),
            Err(ControlRecordErrorV2::Plan(
                ReplacementPlanErrorV2::CandidateNotPreferred
            ))
        ));
    }
}

#[test]
fn intent_checks_complete_old_pin_before_accepting_preferred_candidate() {
    for field in 0..4 {
        let old = main_chain(3);
        let mut expected = pin(&old, 0);
        match field {
            0 => expected.chain.height -= 1,
            1 => expected.chain.tip_hash = Hash256::from_bytes([0; 32]),
            2 => expected.chain.ledger_root = Hash256::from_bytes([0; 32]),
            _ => expected.chain.config_hash = Hash256::from_bytes([0; 32]),
        }
        assert!(matches!(
            IntentRecordV2::new(independent_identity(), expected, old, main_chain(4)),
            Err(ControlRecordErrorV2::WrongCurrentCheckpoint)
        ));
    }
}

#[test]
fn intent_decode_requires_independent_network() {
    let bytes = vector("main3_main4_intent");
    let mut wrong = independent_identity();
    wrong.genesis_hash = Hash256::from_bytes([0; 32]);
    assert!(matches!(
        IntentRecordV2::decode_for(&bytes, wrong),
        Err(ControlRecordErrorV2::WrongIdentity)
    ));
}

#[test]
fn intent_bounds_both_declared_histories_before_execution() {
    for offset in [936, 944] {
        for length in [
            0,
            MAX_HISTORY_WIRE_BYTES_V2 as u64 + 1,
            u32::MAX as u64 + 1,
            u64::MAX,
        ] {
            let mut bytes = vector("main3_main4_intent");
            bytes[offset..offset + 8].copy_from_slice(&length.to_le_bytes());
            assert!(matches!(
                IntentRecordV2::decode_for(&bytes, independent_identity()),
                Err(ControlRecordErrorV2::HistoryLengthOutOfBounds)
            ));
        }
    }
}

#[test]
fn intent_rejects_truncated_headers_bodies_checksum_and_extra_bytes() {
    let bytes = vector("main3_main4_intent");
    for end in [
        0,
        7,
        8,
        471,
        472,
        935,
        936,
        951,
        952,
        983,
        984,
        985,
        bytes.len() / 2,
        bytes.len() - 33,
        bytes.len() - 1,
    ] {
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes[..end], independent_identity()),
            Err(ControlRecordErrorV2::InvalidLength)
        ));
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(matches!(
        IntentRecordV2::decode_for(&extra, independent_identity()),
        Err(ControlRecordErrorV2::InvalidLength)
    ));
}

#[test]
fn intent_rejects_corruption_in_either_record_history_or_checksum() {
    let original = vector("main3_main4_intent");
    let (old, candidate) = history_ranges(&original);
    for offset in [
        8,
        471,
        472,
        935,
        old.start + 20,
        old.end - 1,
        candidate.start + 20,
        candidate.end - 1,
        original.len() - 1,
    ] {
        let mut bytes = original.clone();
        bytes[offset] ^= 1;
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::ChecksumMismatch)
        ));
    }
}

#[test]
fn intent_rejects_valid_outer_checksum_with_broken_nested_control() {
    let mut bytes = vector("main3_main4_intent");
    bytes[8 + 20] ^= 1;
    resign(&mut bytes, INTENT_DOMAIN);
    assert!(matches!(
        IntentRecordV2::decode_for(&bytes, independent_identity()),
        Err(ControlRecordErrorV2::ChecksumMismatch)
    ));
}

#[test]
fn intent_rejects_unsupported_magic_version_or_kind() {
    for offset in [0, 4, 6] {
        let mut bytes = vector("main3_main4_intent");
        bytes[offset] ^= 0x40;
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::InvalidMagic
                | ControlRecordErrorV2::UnsupportedVersion(_)
                | ControlRecordErrorV2::UnsupportedKind(_))
        ));
    }
}

#[test]
fn both_histories_require_complete_matching_end() {
    let original = vector("main3_main4_intent");
    let (old, candidate) = history_ranges(&original);
    for which in [0, 1] {
        let mut old_bytes = original[old.clone()].to_vec();
        let mut candidate_bytes = original[candidate.clone()].to_vec();
        let selected = if which == 0 {
            &mut old_bytes
        } else {
            &mut candidate_bytes
        };
        selected.truncate(selected.len() - 164);
        let mut bytes = original.clone();
        replace_histories(&mut bytes, &old_bytes, &candidate_bytes);
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::History(
                HistoryErrorV2::IncompleteHistory
            ))
        ));
    }
}

#[test]
fn both_histories_reject_input_after_their_end() {
    let original = vector("main3_main4_intent");
    let (old, candidate) = history_ranges(&original);
    for which in [0, 1] {
        let mut old_bytes = original[old.clone()].to_vec();
        let mut candidate_bytes = original[candidate.clone()].to_vec();
        let selected = if which == 0 {
            &mut old_bytes
        } else {
            &mut candidate_bytes
        };
        let end = selected[selected.len() - 164..].to_vec();
        selected.extend_from_slice(&end);
        let mut bytes = original.clone();
        replace_histories(&mut bytes, &old_bytes, &candidate_bytes);
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::History(
                HistoryErrorV2::AlreadyComplete
            ))
        ));
    }
}

#[test]
fn independently_frozen_wrong_actual_ledger_root_rejects_in_either_history() {
    let original = vector("main3_main4_intent");
    let (old, candidate) = history_ranges(&original);
    let bad_block = fixture_bytes("wrong_root1_block");
    for range in [old, candidate] {
        let mut bytes = original.clone();
        let mut offset = range.start;
        let mut changed = false;
        while offset < range.end {
            let kind = u16::from_le_bytes(bytes[offset + 6..offset + 8].try_into().unwrap());
            let length =
                u32::from_le_bytes(bytes[offset + 8..offset + 12].try_into().unwrap()) as usize;
            if kind == 3
                && u64::from_le_bytes(bytes[offset + 12..offset + 20].try_into().unwrap()) == 1
            {
                assert_eq!(length - 8, bad_block.len());
                bytes[offset + 20..offset + 12 + length].copy_from_slice(&bad_block);
                changed = true;
                break;
            }
            offset += 12 + length;
        }
        assert!(changed);
        resign(&mut bytes, INTENT_DOMAIN);
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::History(HistoryErrorV2::Chain(
                ChainErrorV2::WrongLedgerRoot
            )))
        ));
    }
}

#[test]
fn valid_history_cannot_substitute_a_different_membership_fork() {
    let mut bytes = vector("main3_main4_intent");
    replace_histories(
        &mut bytes,
        &history_bytes(&fork_chain()),
        &history_bytes(&main_chain(4)),
    );
    assert!(matches!(
        IntentRecordV2::decode_for(&bytes, independent_identity()),
        Err(ControlRecordErrorV2::WrongCurrentCheckpoint)
    ));
}

#[test]
fn forged_metadata_with_valid_nested_checksums_cannot_replace_actual_history_totals() {
    for record_offset in [8, 472] {
        let mut bytes = vector("main3_main4_intent");
        let record = &mut bytes[record_offset..record_offset + 464];
        let block_bytes = u64::from_le_bytes(record[328..336].try_into().unwrap());
        record[328..336].copy_from_slice(&(block_bytes + 1).to_le_bytes());
        resign(&mut record[168..400], b"MIRROR_STORAGE_SELECTION_V2");
        resign(
            &mut record[152..432],
            b"MIRROR_STORAGE_REPLACEMENT_SELECTION_V2",
        );
        resign(record, CONTROL_DOMAIN);
        assert!(ControlRecordV2::decode_for(record, independent_identity()).is_ok());
        resign(&mut bytes, INTENT_DOMAIN);
        assert!(matches!(
            IntentRecordV2::decode_for(&bytes, independent_identity()),
            Err(ControlRecordErrorV2::NonCanonicalRecord)
        ));
    }
}

#[test]
fn candidate_generation_must_be_exactly_old_plus_one() {
    let mut bytes = vector("main3_main4_intent");
    bytes[472..936].copy_from_slice(&vector("main4_g4_control"));
    resign(&mut bytes, INTENT_DOMAIN);
    assert!(matches!(
        IntentRecordV2::decode_for(&bytes, independent_identity()),
        Err(ControlRecordErrorV2::NonCanonicalRecord)
    ));
}
