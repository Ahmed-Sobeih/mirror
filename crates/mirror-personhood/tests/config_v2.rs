//! B1 public-API regression tests. All evidence/keys are synthetic fixtures.
use mirror_crypto::{sha256d, Hash256, Keypair, PublicKey, SignatureBytes};
use mirror_personhood::v2::*;

fn hash(byte: u8) -> Hash256 { Hash256::from_bytes([byte; 32]) }
fn key(byte: u8) -> Keypair { Keypair::from_secret_bytes([byte; 32]) }
fn parameters() -> ConfigParameters {
    ConfigParameters { format_version: 2, profile_version: 1, network: hash(0x11),
        pow_bits: 0x1f0f_ffff, genesis_timestamp: 1_800_000_000,
        required_approvals: 2, evidence_profile: hash(0x22) }
}
fn seeds() -> Vec<BootstrapDescriptor> {
    (1..=2).map(|i| BootstrapDescriptor { public_key: key(i).public_key(),
        nonce: hash(0x30 + i), evidence: hash(0x40 + i) }).collect()
}
fn allocations() -> Vec<TestAllocation> {
    vec![TestAllocation { address: [0x51; 32], balance_nusa: 0 },
        TestAllocation { address: [0x52; 32], balance_nusa: 123_456_789 }]
}
fn build(p: ConfigParameters, l: LimitsV2, s: &[BootstrapDescriptor], a: &[TestAllocation]) -> ConfigV2 {
    ConfigV2::new(p, l, s, a).unwrap()
}
fn fixture() -> ConfigV2 { build(parameters(), LimitsV2::default(), &seeds(), &allocations()) }
fn proofs(config: &ConfigV2) -> Vec<SignatureBytes> {
    vec![config.sign_seed(0, &key(1)).unwrap(), config.sign_seed(1, &key(2)).unwrap()]
}
fn assert_bound(base: &ConfigV2, other: &ConfigV2) {
    assert_ne!(base.context(), other.context());
    assert_ne!(base.seed_id(0).unwrap(), other.seed_id(0).unwrap());
    assert!(other.verify_bootstrap(&proofs(base)).is_err());
}
fn reference(name: &str) -> &'static str {
    include_str!("vectors/config_v2.txt").lines().filter_map(|line| line.split_once('='))
        .find(|(key, _)| *key == name).unwrap().1
}
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

#[test]
fn config_and_seed_hashes_match_independent_python_vectors() {
    let c = fixture();
    assert_eq!(c.encode().len(), 422);
    assert_eq!(hex(&c.encode()), reference("config_hex"));
    assert_eq!(c.context().to_hex(), reference("context"));
    assert_eq!(c.seed_signing_hash(0).unwrap().to_hex(), reference("seed0_hash"));
    assert_eq!(c.seed_id(0).unwrap().to_hex(), reference("seed0_id"));
    assert_eq!(c.seed_signing_hash(1).unwrap().to_hex(), reference("seed1_hash"));
    assert_eq!(c.seed_id(1).unwrap().to_hex(), reference("seed1_id"));
}

#[test]
fn canonical_round_trip_recomputes_context_and_verifies_proofs() {
    let c = fixture();
    let decoded = ConfigV2::decode(&c.encode()).unwrap();
    assert_eq!(decoded, c);
    assert_eq!(decoded.encode(), c.encode());
    assert_eq!(decoded.verify_bootstrap(&proofs(&c)), c.verify_bootstrap(&proofs(&c)));
}

#[test]
fn every_scalar_configuration_setting_changes_context() {
    let base = fixture();
    let changes: [fn(&mut ConfigParameters); 5] = [
        |p| p.network = hash(0x12), |p| p.pow_bits = 0x207f_ffff,
        |p| p.genesis_timestamp += 1, |p| p.required_approvals = 1,
        |p| p.evidence_profile = hash(0x23),
    ];
    for change in changes {
        let mut p = parameters(); change(&mut p);
        assert_bound(&base, &build(p, LimitsV2::default(), &seeds(), &allocations()));
    }
}

#[test]
fn seed_key_nonce_evidence_and_order_are_bound() {
    let base = fixture();
    let mut cases = Vec::new();
    let mut s = seeds(); s[0].public_key = key(3).public_key(); cases.push(s);
    let mut s = seeds(); s[1].nonce = hash(0x33); cases.push(s);
    let mut s = seeds(); s[1].evidence = hash(0x43); cases.push(s);
    let mut s = seeds(); s.swap(0, 1); cases.push(s);
    for s in cases { assert_bound(&base, &build(parameters(), LimitsV2::default(), &s, &allocations())); }
}

#[test]
fn allocation_address_balance_and_zero_entry_are_bound() {
    let base = fixture();
    let mut cases = Vec::new();
    let mut a = allocations(); a[1].address = [0x53; 32]; cases.push(a);
    let mut a = allocations(); a[1].balance_nusa += 1; cases.push(a);
    let mut a = allocations(); a.remove(0); cases.push(a);
    for a in cases { assert_bound(&base, &build(parameters(), LimitsV2::default(), &seeds(), &a)); }
}

fn relaxed_limits() -> LimitsV2 {
    LimitsV2 { max_block_bytes: 524_288, max_operations: 200,
        max_membership_operations: 180, max_new_enrollments: 32,
        max_operation_bytes: 512, max_pending: 512, max_records: 4096,
        max_accounts: 4096, max_bootstrap: 512, max_log_bytes: 2_097_152,
        max_anchor_age: 16, max_validity_blocks: 128 }
}
fn increment_limits() -> [fn(&mut LimitsV2); 12] {
    [|l| l.max_block_bytes += 1, |l| l.max_operations += 1,
        |l| l.max_membership_operations += 1, |l| l.max_new_enrollments += 1,
        |l| l.max_operation_bytes += 1, |l| l.max_pending += 1,
        |l| l.max_records += 1, |l| l.max_accounts += 1,
        |l| l.max_bootstrap += 1, |l| l.max_log_bytes += 1,
        |l| l.max_anchor_age += 1, |l| l.max_validity_blocks += 1]
}

#[test]
fn each_resource_limit_changes_context_and_invalidates_old_proofs() {
    let base = build(parameters(), relaxed_limits(), &seeds(), &allocations());
    for change in increment_limits() {
        let mut limits = relaxed_limits(); change(&mut limits);
        assert_bound(&base, &build(parameters(), limits, &seeds(), &allocations()));
    }
}

#[test]
fn zero_domains_and_unsupported_versions_are_rejected() {
    for field in 0..4 {
        let mut p = parameters();
        let error = match field {
            0 => { p.network = Hash256::default(); ConfigError::ZeroDomain },
            1 => { p.evidence_profile = Hash256::default(); ConfigError::ZeroDomain },
            2 => { p.format_version = 1; ConfigError::UnsupportedVersion },
            _ => { p.profile_version = 2; ConfigError::UnsupportedProfile },
        };
        assert_eq!(ConfigV2::new(p, LimitsV2::default(), &seeds(), &allocations()), Err(error));
    }
}

#[test]
fn invalid_compact_targets_are_rejected_without_shifts_panicking() {
    for bits in [0, 0x1f80_0001, 0x2100_ffff, 0x0100_0001, 0x0001_0000, u32::MAX] {
        let mut p = parameters(); p.pow_bits = bits;
        assert_eq!(ConfigV2::new(p, LimitsV2::default(), &seeds(), &[]), Err(ConfigError::InvalidPowTarget));
    }
    for bits in [0x0101_0000, 0x0200_0100, 0x0300_0001, 0x1f0f_ffff, 0x207f_ffff] {
        let mut p = parameters(); p.pow_bits = bits;
        assert!(ConfigV2::new(p, LimitsV2::default(), &seeds(), &[]).is_ok());
    }
}

#[test]
fn threshold_and_bootstrap_count_boundaries_are_checked() {
    for threshold in [0, 65, u16::MAX] {
        let mut p = parameters(); p.required_approvals = threshold;
        assert_eq!(ConfigV2::new(p, LimitsV2::default(), &seeds(), &[]), Err(ConfigError::InvalidThreshold));
    }
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &[], &[]), Err(ConfigError::InvalidBootstrapCount));
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &seeds()[..1], &[]), Err(ConfigError::InvalidBootstrapCount));
    let mut p = parameters(); p.required_approvals = 1;
    let c = build(p, LimitsV2::default(), &seeds()[..1], &[]);
    assert!(c.verify_bootstrap(&[c.sign_seed(0, &key(1)).unwrap()]).is_ok());
}

#[test]
fn duplicate_seed_keys_are_rejected() {
    let mut s = seeds(); s[1].public_key = s[0].public_key;
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &s, &[]), Err(ConfigError::DuplicateKey));
}
#[test]
fn duplicate_seed_nonces_are_rejected() {
    let mut s = seeds(); s[1].nonce = s[0].nonce;
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &s, &[]), Err(ConfigError::DuplicateNonce));
}
#[test]
fn duplicate_seed_evidence_is_rejected() {
    let mut s = seeds(); s[1].evidence = s[0].evidence;
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &s, &[]), Err(ConfigError::DuplicateEvidence));
}
#[test]
fn zero_seed_commitments_are_rejected() {
    for evidence in [false, true] {
        let mut s = seeds();
        if evidence { s[0].evidence = Hash256::default(); } else { s[0].nonce = Hash256::default(); }
        assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &s, &[]), Err(ConfigError::ZeroCommitment));
    }
}

#[test]
fn unsorted_and_duplicate_allocations_including_zero_are_rejected() {
    let mut a = allocations(); a.reverse();
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &seeds(), &a), Err(ConfigError::AllocationsNotStrictlySorted));
    let duplicate = [allocations()[0], allocations()[0]];
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &seeds(), &duplicate), Err(ConfigError::AllocationsNotStrictlySorted));
}

#[test]
fn supply_sum_is_checked_and_empty_allocations_create_no_money() {
    let mut a = allocations(); a[1].balance_nusa = u64::MAX;
    let c = build(parameters(), LimitsV2::default(), &seeds(), &a);
    assert_eq!(c.initial_test_supply(), u64::MAX);
    a[0].balance_nusa = 1;
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &seeds(), &a), Err(ConfigError::SupplyOverflow));
    let empty = build(parameters(), LimitsV2::default(), &seeds(), &[]);
    assert_eq!(empty.initial_test_supply(), 0);
    assert!(empty.allocations().is_empty());
    assert_eq!(fixture().initial_test_supply(), 123_456_789);
}

#[test]
fn limit_ceilings_and_zero_values_are_rejected() {
    assert_eq!(LimitsV2::default().validate(), Ok(()));
    for change in increment_limits() {
        let mut limits = LimitsV2::default(); change(&mut limits);
        assert_eq!(limits.validate(), Err(ConfigError::InvalidLimits));
    }
    let zero: [fn(&mut LimitsV2); 12] = [
        |l| l.max_block_bytes = 0, |l| l.max_operations = 0,
        |l| l.max_membership_operations = 0, |l| l.max_new_enrollments = 0,
        |l| l.max_operation_bytes = 0, |l| l.max_pending = 0,
        |l| l.max_records = 0, |l| l.max_accounts = 0,
        |l| l.max_bootstrap = 0, |l| l.max_log_bytes = 0,
        |l| l.max_anchor_age = 0, |l| l.max_validity_blocks = 0,
    ];
    for change in zero {
        let mut limits = LimitsV2::default(); change(&mut limits);
        assert_eq!(ConfigV2::new(parameters(), limits, &seeds(), &[]), Err(ConfigError::InvalidLimits));
    }
}

#[test]
fn inconsistent_limits_cannot_be_used_directly() {
    let changes: [fn(&mut LimitsV2); 7] = [
        |l| l.max_operations = 255, |l| l.max_membership_operations = 63,
        |l| l.max_pending = 63, |l| l.max_records = 1000,
        |l| l.max_operation_bytes = 290, |l| l.max_block_bytes = 1195,
        |l| l.max_log_bytes = 1031,
    ];
    for change in changes {
        let mut limits = LimitsV2::default(); change(&mut limits);
        assert_eq!(limits.validate(), Err(ConfigError::InvalidLimits));
    }
    let mut limits = LimitsV2::default(); limits.max_pending = 64; limits.max_records = 999;
    assert_eq!(limits.validate(), Err(ConfigError::InvalidLimits)); // bootstrap limit > record limit
}

#[test]
fn smallest_consistent_limits_and_largest_timestamp_round_trip() {
    let limits = LimitsV2 { max_block_bytes: 463, max_operations: 1,
        max_membership_operations: 1, max_new_enrollments: 1, max_operation_bytes: 291,
        max_pending: 1, max_records: 1, max_accounts: 1, max_bootstrap: 1,
        max_log_bytes: 299, max_anchor_age: 1, max_validity_blocks: 1 };
    let mut p = parameters(); p.required_approvals = 1; p.genesis_timestamp = u64::MAX;
    let c = build(p, limits, &seeds()[..1], &allocations()[..1]);
    assert_eq!(ConfigV2::decode(&c.encode()).unwrap(), c);
}

#[test]
fn direct_constructor_rejects_count_over_limits_before_copying() {
    let mut l = LimitsV2::default(); l.max_bootstrap = 2; l.max_accounts = 2;
    assert!(ConfigV2::new(parameters(), l, &seeds(), &allocations()).is_ok());
    let mut s = seeds(); s.push(s[0]);
    assert_eq!(ConfigV2::new(parameters(), l, &s, &[]), Err(ConfigError::InvalidBootstrapCount));
    let mut a = allocations(); a.push(a[0]);
    assert_eq!(ConfigV2::new(parameters(), l, &seeds(), &a), Err(ConfigError::TooManyAllocations));
}

#[test]
fn maximum_config_counts_fit_and_one_more_is_rejected() {
    // These generated key bytes are unverified descriptors, not accepted members.
    let s: Vec<_> = (0u32..1000).map(|i| {
        let nonce = sha256d(&i.to_le_bytes());
        let evidence = sha256d(nonce.as_bytes());
        BootstrapDescriptor { public_key: PublicKey::from_bytes(*sha256d(evidence.as_bytes()).as_bytes()), nonce, evidence }
    }).collect();
    let a: Vec<_> = (0u32..65_536).map(|i| {
        let mut address = [0; 32]; address[28..].copy_from_slice(&i.to_be_bytes());
        TestAllocation { address, balance_nusa: 0 }
    }).collect();
    let c = build(parameters(), LimitsV2::default(), &s, &a);
    assert_eq!(c.encode().len(), MAX_CONFIG_BYTES);
    assert_eq!(ConfigV2::decode(&c.encode()).unwrap(), c);
    let mut more_seeds = s.clone(); more_seeds.push(s[0]);
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &more_seeds, &[]), Err(ConfigError::InvalidBootstrapCount));
    let mut more_accounts = a; more_accounts.push(TestAllocation { address: [255; 32], balance_nusa: 0 });
    assert_eq!(ConfigV2::new(parameters(), LimitsV2::default(), &s, &more_accounts), Err(ConfigError::TooManyAllocations));
}

#[test]
fn all_truncations_and_trailing_bytes_are_rejected() {
    let bytes = fixture().encode();
    for end in 0..bytes.len() { assert!(ConfigV2::decode(&bytes[..end]).is_err(), "end={end}"); }
    let mut extra = bytes; extra.push(0);
    assert_eq!(ConfigV2::decode(&extra), Err(ConfigError::TrailingBytes));
}

#[test]
fn declared_huge_counts_are_rejected_before_reserving_memory() {
    let mut bytes = fixture().encode(); bytes[142..146].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(ConfigV2::decode(&bytes), Err(ConfigError::InvalidBootstrapCount));
    let mut bytes = fixture().encode(); bytes[338..342].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(ConfigV2::decode(&bytes), Err(ConfigError::TooManyAllocations));
    assert_eq!(ConfigV2::decode(&vec![0; MAX_CONFIG_BYTES + 1]), Err(ConfigError::TooLarge));
}

#[test]
fn legacy_unknown_magic_and_versions_never_fall_back() {
    for magic in [*b"MRB1", *b"MPH1", *b"MCF1", *b"????"] {
        let mut bytes = fixture().encode(); bytes[..4].copy_from_slice(&magic);
        assert_eq!(ConfigV2::decode(&bytes), Err(ConfigError::InvalidMagic));
    }
    let mut bytes = fixture().encode(); bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    assert_eq!(ConfigV2::decode(&bytes), Err(ConfigError::UnsupportedVersion));
}

#[test]
fn decoding_rechecks_descriptor_and_allocation_semantics() {
    let mut bytes = fixture().encode(); bytes[178..210].fill(0); // seed0 nonce
    assert_eq!(ConfigV2::decode(&bytes), Err(ConfigError::ZeroCommitment));
    let mut bytes = fixture().encode(); bytes[382..414].fill(0x51); // duplicate allocation address
    assert_eq!(ConfigV2::decode(&bytes), Err(ConfigError::AllocationsNotStrictlySorted));
}

#[test]
fn every_seed_proof_is_required_and_seed_indices_are_not_privileges() {
    let c = fixture(); let signatures = proofs(&c);
    let verified = c.verify_bootstrap(&signatures).unwrap();
    assert_eq!(verified.config(), &c);
    assert_eq!(verified.seeds().len(), 2);
    for i in 0..2 {
        assert_eq!(verified.seeds()[i].id(), c.seed_id(i as u32).unwrap());
        assert_eq!(verified.seeds()[i].public_key(), c.bootstrap()[i].public_key);
        assert_eq!(verified.seeds()[i].ordinal(), i as u64 + 1);
    }
    assert_eq!(c.verify_bootstrap(&signatures[..1]), Err(ConfigError::WrongProofCount));
    let mut extra = signatures.clone(); extra.push(signatures[0]);
    assert_eq!(c.verify_bootstrap(&extra), Err(ConfigError::WrongProofCount));
    assert!(c.verify_bootstrap(&[signatures[0], signatures[0]]).is_err());
    assert!(c.verify_bootstrap(&[signatures[1], signatures[0]]).is_err());
}

#[test]
fn forged_last_proof_returns_no_partially_verified_bootstrap() {
    let c = fixture(); let before = c.clone(); let mut signatures = proofs(&c);
    let mut bytes = *signatures[1].as_bytes(); bytes[17] ^= 1;
    signatures[1] = SignatureBytes::from_bytes(bytes);
    assert!(matches!(c.verify_bootstrap(&signatures), Err(ConfigError::InvalidSeedProof { index: 1, .. })));
    assert_eq!(c, before);
    assert!(c.verify_bootstrap(&proofs(&c)).is_ok());
}

#[test]
fn unknown_indices_and_wrong_signing_keys_fail() {
    let c = fixture();
    assert_eq!(c.seed_id(2), Err(ConfigError::UnknownSeed));
    assert_eq!(c.seed_signing_hash(u32::MAX), Err(ConfigError::UnknownSeed));
    assert_eq!(c.sign_seed(0, &key(2)), Err(ConfigError::WrongSeedKey));
    assert_eq!(c.sign_seed(u32::MAX, &key(1)), Err(ConfigError::UnknownSeed));
}

#[test]
fn malformed_or_weak_seed_keys_cannot_become_verified_bootstrap() {
    for bytes in [[0; 32], [255; 32]] {
        let mut s = seeds(); s[0].public_key = PublicKey::from_bytes(bytes);
        let c = build(parameters(), LimitsV2::default(), &s, &[]);
        assert!(matches!(c.verify_bootstrap(&[SignatureBytes::from_bytes([0; 64]); 2]),
            Err(ConfigError::InvalidSeedProof { index: 0, .. })));
    }
}

#[test]
fn no_circular_context_or_caller_supplied_context() {
    let c = fixture(); let encoded = c.encode(); let context = c.context();
    let signatures = proofs(&c);
    let verified = c.verify_bootstrap(&signatures).unwrap();
    assert_eq!(verified.config().encode(), encoded);
    assert_eq!(verified.config().context(), context);
    assert_ne!(c.seed_signing_hash(0).unwrap(), c.seed_id(0).unwrap());
    let mut appended_context = encoded; appended_context.extend_from_slice(context.as_bytes());
    assert_eq!(ConfigV2::decode(&appended_context), Err(ConfigError::TrailingBytes));
}

#[test]
fn changing_construction_inputs_cannot_mutate_a_validated_config() {
    let mut p = parameters(); let mut l = LimitsV2::default();
    let mut s = seeds(); let mut a = allocations();
    let c = build(p, l, &s, &a); let before = c.clone();
    p.network = hash(99); l.max_accounts = 2; s.reverse(); a.clear();
    assert_eq!(c, before);
    assert_bound(&c, &build(p, l, &s, &a));
}

#[test]
fn valid_legacy_enrollment_signatures_are_not_v2_seed_proofs() {
    use mirror_personhood::{Enrollment, MembershipEvent, RegistryRules, encode_log};
    let rules = RegistryRules::new(hash(0x11), 2).unwrap();
    let old: Vec<_> = (1..=2).map(|i| {
        let request = Enrollment::sign(&rules, &key(i), hash(0x30 + i), hash(0x40 + i)).unwrap();
        let bytes = encode_log(&[MembershipEvent::Enroll(request)]).unwrap();
        SignatureBytes::from_bytes(bytes[bytes.len() - 64..].try_into().unwrap())
    }).collect();
    assert!(fixture().verify_bootstrap(&old).is_err());
}
