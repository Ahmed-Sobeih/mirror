use mirror_consensus::{
    PowError, PowTarget,
    v2::{MAX_MINING_ATTEMPTS_V2, PowErrorV2, mine_header_v2, validate_header_pow_v2},
};
use mirror_core::v2::{BLOCK_HEADER_LEN_V2, BlockHeaderV2};
use mirror_crypto::{Hash256, sha256d};

const BITS: u32 = 0x1f0f_ffff;
const KNOWN_NONCE: u64 = 9_079;

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

fn header(bits: u32, nonce: u64) -> BlockHeaderV2 {
    BlockHeaderV2::new(
        hash(0x11),
        1,
        hash(0x22),
        hash(0x33),
        hash(0x44),
        1_800_000_001,
        bits,
        nonce,
    )
}

#[test]
fn mining_matches_independent_v2_header_vector() {
    // Independent Python struct/hashlib calculation before this Rust test:
    // SHA256d(b"MIRROR_BLOCK_V2" + struct.pack('<I',2) + C32 + heightu64
    // + parent32 + ops32 + ledger32 + timestampu64 + bitsu32 + nonceu64),
    // ascending nonces from 0, raw big-endian numeric digest <= target.
    let original = header(BITS, 0);
    assert_eq!(original.encode().len(), BLOCK_HEADER_LEN_V2);
    assert_eq!(BLOCK_HEADER_LEN_V2, 160);
    assert_eq!(
        original.hash().to_hex(),
        "ff2beba7eb1c38b9a9aab8c3495a11433244b31240a2e549d843dec5505f78a2"
    );
    let result = mine_header_v2(original, BITS, 9_080).unwrap();
    assert_eq!(result.header.nonce(), KNOWN_NONCE);
    assert_eq!(result.attempts, 9_080);
    assert_eq!(
        result.hash.to_hex(),
        "000b09418ee8e55d171d2a59c4e6b9fb21222e2c59ced39094a4835bbcefe02e"
    );
    assert_eq!(result.hash, result.header.hash());
    assert_eq!(validate_header_pow_v2(&result.header, BITS), Ok(()));
    assert_eq!(&result.header.encode()[..152], &original.encode()[..152]);
    assert_eq!(original, header(BITS, 0));
}

#[test]
fn unmined_real_v2_header_fails_work() {
    assert_eq!(
        validate_header_pow_v2(&header(BITS, 0), BITS),
        Err(PowErrorV2::HashAboveTarget)
    );
}

#[test]
fn v2_work_hash_uses_domain_and_full_160_byte_header() {
    let candidate = header(BITS, KNOWN_NONCE);
    let bytes = candidate.encode();
    let mut preimage = b"MIRROR_BLOCK_V2".to_vec();
    preimage.extend_from_slice(&bytes);
    assert_eq!(candidate.hash(), sha256d(&preimage));
    assert_ne!(candidate.hash(), sha256d(&bytes));
    assert_ne!(candidate.hash(), sha256d(&bytes[..120]));
    let mut changed = candidate.encode();
    changed[108] ^= 1; // First byte of the combined ledger root.
    let changed = BlockHeaderV2::decode(&changed).unwrap();
    assert_ne!(candidate.hash(), changed.hash());
}

#[test]
fn configured_bits_reject_a_valid_self_selected_easier_target() {
    let easier = header(0x207f_ffff, 4);
    assert_eq!(
        easier.hash().to_hex(),
        "0be5bc089f7946c7dcfcc4c1dca32b557f76a41e4329ea740cdd320234729e8d"
    );
    assert_eq!(validate_header_pow_v2(&easier, 0x207f_ffff), Ok(()));
    let expected = Err(PowErrorV2::WrongBits {
        expected: BITS,
        got: 0x207f_ffff,
    });
    assert_eq!(validate_header_pow_v2(&easier, BITS), expected);
    assert_eq!(
        mine_header_v2(easier, BITS, 1),
        Err(PowErrorV2::WrongBits {
            expected: BITS,
            got: 0x207f_ffff,
        })
    );
}

#[test]
fn numerically_equal_compact_aliases_do_not_replace_configured_bits() {
    let expected = 0x0300_1000;
    let alias = 0x0210_0000;
    assert_eq!(
        PowTarget::from_compact(expected),
        PowTarget::from_compact(alias)
    );
    assert_eq!(
        validate_header_pow_v2(&header(alias, 0), expected),
        Err(PowErrorV2::WrongBits {
            expected,
            got: alias
        })
    );
}

#[test]
fn an_explicitly_configured_positive_noncanonical_target_remains_supported() {
    // Python independently found nonce327 for this exact header/compact value.
    // Existing MCF2 permits this representation; V2 must not silently narrow it.
    let bits = 0x2000_0fff;
    let canonical = 0x1f0f_ff00;
    assert_eq!(
        PowTarget::from_compact(bits),
        PowTarget::from_compact(canonical)
    );
    let candidate = header(bits, 327);
    assert_eq!(
        candidate.hash().to_hex(),
        "0002d613d8229453585eb05c9d692e6b7ae542761fd4d4940bb7b395cd5e5806"
    );
    assert_eq!(validate_header_pow_v2(&candidate, bits), Ok(()));
    assert_eq!(mine_header_v2(candidate, bits, 1).unwrap().attempts, 1);
    assert_eq!(
        validate_header_pow_v2(&candidate, canonical),
        Err(PowErrorV2::WrongBits {
            expected: canonical,
            got: bits,
        })
    );
}

#[test]
fn invalid_configured_targets_reuse_existing_mirror_range_checks() {
    for (bits, error) in [
        (0, PowError::ZeroTarget),
        (0x1f00_0000, PowError::ZeroTarget),
        (0x0100_0001, PowError::ZeroTarget),
        (0x1f8f_ffff, PowError::NegativeTarget),
        (0x2100_0001, PowError::TargetOverflow),
    ] {
        assert_eq!(
            validate_header_pow_v2(&header(bits, 0), bits),
            Err(PowErrorV2::Target(error))
        );
        assert_eq!(
            mine_header_v2(header(bits, 0), bits, 1),
            Err(PowErrorV2::Target(error))
        );
    }
}

#[test]
fn work_comparison_is_big_endian_and_includes_exact_target() {
    let target = PowTarget::from_compact(BITS).unwrap();
    assert_eq!(
        target.to_hex(),
        "000fffff00000000000000000000000000000000000000000000000000000000"
    );
    let exact = Hash256::from_bytes(*target.as_bytes());
    assert!(target.accepts(exact));
    let mut above = *target.as_bytes();
    above[31] = 1;
    assert!(!target.accepts(Hash256::from_bytes(above)));
    let mut below = *target.as_bytes();
    below[3] -= 1;
    below[4..].fill(0xff);
    assert!(target.accepts(Hash256::from_bytes(below)));
    let valid = header(BITS, KNOWN_NONCE).hash();
    assert!(target.accepts(valid));
    let mut reversed = *valid.as_bytes();
    reversed.reverse();
    assert!(!target.accepts(Hash256::from_bytes(reversed)));
}

#[test]
fn known_solution_is_not_found_when_budget_ends_one_attempt_early() {
    let original = header(BITS, 0);
    assert_eq!(
        mine_header_v2(original, BITS, KNOWN_NONCE),
        Err(PowErrorV2::AttemptsExhausted {
            attempts: KNOWN_NONCE
        })
    );
    assert_eq!(original, header(BITS, 0));
    let last = mine_header_v2(header(BITS, KNOWN_NONCE), BITS, 1).unwrap();
    assert_eq!(last.header.nonce(), KNOWN_NONCE);
    assert_eq!(last.attempts, 1);
}

#[test]
fn search_starts_at_supplied_nonce_instead_of_resetting_to_zero() {
    let result = mine_header_v2(header(BITS, KNOWN_NONCE - 1), BITS, 2).unwrap();
    assert_eq!(result.header.nonce(), KNOWN_NONCE);
    assert_eq!(result.attempts, 2);
}

#[test]
fn zero_and_over_cap_budgets_are_rejected_before_work() {
    for requested in [0, MAX_MINING_ATTEMPTS_V2 + 1, u64::MAX] {
        assert_eq!(
            mine_header_v2(header(BITS, KNOWN_NONCE), BITS, requested),
            Err(PowErrorV2::InvalidAttemptBudget {
                requested,
                maximum: MAX_MINING_ATTEMPTS_V2,
            })
        );
    }
}

#[test]
fn exact_local_attempt_cap_is_allowed() {
    let result = mine_header_v2(header(BITS, KNOWN_NONCE), BITS, MAX_MINING_ATTEMPTS_V2).unwrap();
    assert_eq!(result.attempts, 1);
}

#[test]
fn failed_final_nonce_never_wraps() {
    let bits = 0x0101_0000; // Numeric target = 1; both fixed hashes are greater.
    let final_header = header(bits, u64::MAX);
    assert_eq!(
        final_header.hash().to_hex(),
        "446ca78e29746c468623953d9d4461aaa838ad34eb0e02f15262280f95c8c8c4"
    );
    assert_eq!(
        mine_header_v2(final_header, bits, 1),
        Err(PowErrorV2::NonceExhausted { attempts: 1 })
    );
    assert_eq!(
        mine_header_v2(header(bits, u64::MAX - 1), bits, 2),
        Err(PowErrorV2::NonceExhausted { attempts: 2 })
    );
}

#[test]
fn valid_final_nonce_is_accepted_without_incrementing() {
    // Separate independent Python hash at nonce u64::MAX, timestamp 1800000002.
    let input = header(0x207f_ffff, u64::MAX);
    let final_header = BlockHeaderV2::new(
        input.config_hash(),
        input.height(),
        input.previous_hash(),
        input.operations_root(),
        input.ledger_root(),
        1_800_000_002,
        input.pow_bits(),
        input.nonce(),
    );
    let result = mine_header_v2(final_header, 0x207f_ffff, 1).unwrap();
    assert_eq!(result.header.nonce(), u64::MAX);
    assert_eq!(result.attempts, 1);
    assert_eq!(
        result.hash.to_hex(),
        "1a8dc97bbd63e13f4090543c5bf8f849a7f3f84c3458754e553c3fb387c6150f"
    );
    assert_eq!(validate_header_pow_v2(&result.header, 0x207f_ffff), Ok(()));
}

#[test]
fn valid_work_does_not_assert_other_header_or_ledger_authority() {
    // Impossible height/configuration and invented zero roots must be rejected
    // by chain/ledger validation even though this isolated work predicate passes.
    let candidate = BlockHeaderV2::new(
        Hash256::default(),
        u64::MAX,
        Hash256::default(),
        Hash256::default(),
        Hash256::default(),
        0,
        0x207f_ffff,
        0,
    );
    let result = mine_header_v2(candidate, 0x207f_ffff, 128).unwrap();
    assert_eq!(result.header.height(), u64::MAX);
    assert_eq!(result.header.config_hash(), Hash256::default());
    assert_eq!(result.header.ledger_root(), Hash256::default());
    assert_eq!(validate_header_pow_v2(&result.header, 0x207f_ffff), Ok(()));
}
