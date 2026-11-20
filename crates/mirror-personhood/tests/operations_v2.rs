//! Synthetic B2 authorization tests, NOT live admission/biometric tests.
use std::{cell::Cell, collections::BTreeMap};
use mirror_crypto::{Hash256, Keypair, PublicKey, SignatureBytes};
use mirror_personhood::v2::{BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2,
    MembershipActionV2 as Action, OperationErrorV2 as Error, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as Op,
    ENROLL_OPERATION_LEN_V2, OPERATION_PREFIX_LEN_V2, REVIEW_OPERATION_LEN_V2};

fn hash(byte: u8) -> Hash256 { Hash256::from_bytes([byte; 32]) }
fn key() -> Keypair { Keypair::from_secret_bytes([1; 32]) }
fn config_with(limits: LimitsV2) -> ConfigV2 {
    ConfigV2::new(ConfigParameters { format_version: 2, profile_version: 1,
        network: hash(0x11), pow_bits: 0x207f_ffff, genesis_timestamp: 0,
        required_approvals: 1, evidence_profile: hash(0x22) }, limits,
        &[BootstrapDescriptor { public_key: key().public_key(), nonce: hash(3), evidence: hash(4) }], &[]).unwrap()
}
fn config() -> ConfigV2 { config_with(LimitsV2::default()) }
fn header(c: &ConfigV2) -> Header {
    Header { config_hash: c.context(), anchor_height: 7, anchor_hash: hash(0x44),
        valid_from: 8, valid_until: 24, actor_public_key: key().public_key(), actor_sequence: 0 }
}
fn enroll() -> Action {
    Action::Enroll { applicant_nonce: hash(0x55), evidence_commitment: hash(0x66), evidence_profile: hash(0x22) }
}
fn actions() -> [Action; 3] {
    [enroll(), Action::Approve { target_enrollment_id: hash(0x77) }, Action::Withdraw { own_member_id: hash(0x88) }]
}
fn sign(h: Header, action: Action) -> Op { Op::sign(h, action, &key()).unwrap() }
fn samples(c: &ConfigV2) -> Vec<Op> {
    actions().into_iter().map(|a| { let mut h = header(c); if a.kind() != 1 { h.actor_sequence = 9; } sign(h, a) }).collect()
}
#[derive(Clone)]
struct View { context: Hash256, height: u64, hashes: BTreeMap<u64, Hash256>, calls: Cell<usize> }
impl View {
    fn new(c: &ConfigV2) -> Self { Self { context: c.context(), height: 7,
        hashes: [(7, hash(0x44))].into(), calls: Cell::new(0) } }
}
impl ParentChainViewV2 for View {
    fn config_hash(&self) -> Hash256 { self.context }
    fn parent_height(&self) -> u64 { self.height }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        self.calls.set(self.calls.get() + 1); self.hashes.get(&height).copied()
    }
}
fn assert_rejected(c: &ConfigV2, view: &View, op: Op, error: Error) {
    let before = op;
    assert_eq!(op.validate_for_parent(c, view), Err(error));
    assert_eq!(op, before);
}
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i+2], 16).unwrap()).collect()
}

#[test]
fn membership_operations_have_exact_lengths_and_offsets() {
    let c = config();
    assert_eq!(OPERATION_PREFIX_LEN_V2, 131);
    assert_eq!(ENROLL_OPERATION_LEN_V2, 291);
    assert_eq!(REVIEW_OPERATION_LEN_V2, 227);
    for op in samples(&c) {
        let b = op.encode();
        assert_eq!(b.len(), op.encoded_len());
        assert_eq!(&b[..2], &2u16.to_le_bytes());
        assert_eq!(b[2], op.action().kind());
        assert_eq!(&b[3..35], c.context().as_bytes());
        assert_eq!(&b[35..43], &7u64.to_le_bytes());
        assert_eq!(&b[43..75], hash(0x44).as_bytes());
        assert_eq!(&b[75..83], &8u64.to_le_bytes());
        assert_eq!(&b[83..91], &24u64.to_le_bytes());
        assert_eq!(&b[91..123], key().public_key().as_bytes());
        assert_eq!(&b[123..131], &op.header().actor_sequence.to_le_bytes());
        assert_eq!(&b[b.len()-64..], op.signature().as_bytes());
    }
}

#[test]
fn canonical_round_trip_all_membership_kinds() {
    let c = config();
    for op in samples(&c) {
        let decoded = Op::decode(&c, &op.encode()).unwrap();
        assert_eq!(decoded, op);
        assert_eq!(decoded.encode(), op.encode());
        assert_eq!(decoded.verify_signature(), Ok(()));
        assert_eq!(decoded.validate_for_parent(&c, &View::new(&c)), Ok(()));
    }
}

#[test]
fn every_truncation_is_rejected() {
    let c = config();
    for op in samples(&c) {
        let b = op.encode();
        for end in 0..b.len() { assert!(Op::decode(&c, &b[..end]).is_err(), "kind={} len={end}", op.action().kind()); }
    }
}

#[test]
fn trailing_and_wrong_kind_lengths_are_rejected() {
    let c = config();
    for op in samples(&c) {
        let mut b = op.encode(); b.push(0);
        assert_eq!(Op::decode(&c, &b), Err(Error::InvalidLength));
    }
    let mut b = sign(header(&c), enroll()).encode(); b[2] = 2;
    assert_eq!(Op::decode(&c, &b), Err(Error::InvalidLength));
}

#[test]
fn unknown_versions_transfer_and_unknown_kinds_are_rejected() {
    let c = config(); let b = sign(header(&c), enroll()).encode();
    for version in [0u16, 1, 3, u16::MAX] {
        let mut changed = b.clone(); changed[..2].copy_from_slice(&version.to_le_bytes());
        assert_eq!(Op::decode(&c, &changed), Err(Error::UnsupportedVersion));
    }
    for kind in [0u8, 4, 255] {
        let mut changed = b.clone(); changed[2] = kind;
        assert_eq!(Op::decode(&c, &changed), Err(Error::UnsupportedMembershipKind));
    }
}

#[test]
fn legacy_magic_never_falls_back_to_old_decoder() {
    let c = config();
    for magic in [b"MPH1", b"MRB1", b"MCF2"] {
        assert_eq!(Op::decode(&c, magic), Err(Error::UnsupportedVersion));
    }
}

#[test]
fn oversized_input_rejects_before_field_parsing() {
    let c = config();
    assert_eq!(Op::decode(&c, &vec![0; 1_025]), Err(Error::TooLarge));
}

#[test]
fn configured_byte_boundary_accepts_exact_enrollment_size() {
    let c = config_with(LimitsV2 { max_operation_bytes: 291, ..LimitsV2::default() });
    let op = sign(header(&c), enroll());
    assert_eq!(Op::decode(&c, &op.encode()), Ok(op));
    let mut b = op.encode(); b.push(0);
    assert_eq!(Op::decode(&c, &b), Err(Error::TooLarge));
}

#[test]
fn every_unsigned_payload_byte_is_covered_by_signature() {
    let c = config();
    for op in samples(&c) {
        let original = op.encode();
        // Version and kind are constrained structurally and tested separately.
        for offset in 3..original.len()-64 {
            let mut b = original.clone(); b[offset] ^= 1;
            let changed = Op::decode(&c, &b).unwrap();
            assert!(changed.verify_signature().is_err(), "kind={} byte={offset}", op.action().kind());
        }
    }
}

#[test]
fn every_signature_byte_is_checked() {
    let c = config();
    for op in samples(&c) {
        let original = op.encode();
        for offset in original.len()-64..original.len() {
            let mut b = original.clone(); b[offset] ^= 1;
            assert!(Op::decode(&c, &b).unwrap().verify_signature().is_err());
        }
    }
}

#[test]
fn same_length_approval_cannot_be_changed_into_withdrawal() {
    let c = config(); let mut b = samples(&c)[1].encode(); b[2] = 3;
    assert!(Op::decode(&c, &b).unwrap().verify_signature().is_err());
}

#[test]
fn signing_rejects_wrong_selected_key() {
    let c = config();
    assert_eq!(Op::sign(header(&c), enroll(), &Keypair::from_secret_bytes([2;32])), Err(Error::WrongSigningKey));
}

#[test]
fn weak_or_invalid_keys_and_zero_signatures_do_not_panic() {
    let c = config(); let op = sign(header(&c), enroll());
    let mut identity = [0u8;32]; identity[0] = 1;
    for bytes in [[0;32], identity, [255;32]] {
        let mut h = header(&c); h.actor_public_key = PublicKey::from_bytes(bytes);
        assert!(matches!(Op::from_parts(h, enroll(), op.signature()).verify_signature(), Err(Error::Crypto(_))));
    }
    assert!(matches!(Op::from_parts(header(&c), enroll(), SignatureBytes::from_bytes([0;64])).verify_signature(), Err(Error::Crypto(_))));
}

#[test]
fn seed_and_v1_signatures_do_not_authorize_v2_operations() {
    let c = config();
    let seed_sig = c.sign_seed(0, &key()).unwrap();
    assert!(Op::from_parts(header(&c), enroll(), seed_sig).verify_signature().is_err());
    let r = mirror_personhood::RegistryRules::new(hash(0x11), 1).unwrap();
    let legacy = mirror_personhood::Enrollment::sign(&r, &key(), hash(0x55), hash(0x66)).unwrap();
    let bytes = mirror_personhood::encode_log(&[mirror_personhood::MembershipEvent::Enroll(legacy)]).unwrap();
    let sig: [u8;64] = bytes[bytes.len()-64..].try_into().unwrap();
    assert!(Op::from_parts(header(&c), enroll(), SignatureBytes::from_bytes(sig)).verify_signature().is_err());
}

#[test]
fn valid_signature_on_wrong_configuration_is_rejected() {
    let c = config(); let mut p = c.parameters(); p.network = hash(0x12);
    let other = ConfigV2::new(p, *c.limits(), c.bootstrap(), c.allocations()).unwrap();
    let op = sign(header(&other), enroll()); assert_eq!(op.verify_signature(), Ok(()));
    assert_rejected(&c, &View::new(&c), op, Error::WrongContext);
}

#[test]
fn changed_bootstrap_configuration_cannot_reuse_operation() {
    let c = config(); let mut seeds = c.bootstrap().to_vec(); seeds[0].evidence = hash(0x45);
    let other = ConfigV2::new(c.parameters(), *c.limits(), &seeds, &[]).unwrap();
    let op = sign(header(&c), enroll());
    assert_rejected(&other, &View::new(&other), op, Error::WrongContext);
    let mut h = *op.header(); h.config_hash = other.context();
    assert!(Op::from_parts(h, *op.action(), op.signature()).verify_signature().is_err());
}

#[test]
fn mismatched_parent_context_is_rejected_before_lookup() {
    let c = config(); let mut view = View::new(&c); view.context = hash(0x99);
    assert_rejected(&c, &view, sign(header(&c), enroll()), Error::WrongParentContext);
    assert_eq!(view.calls.get(), 0);
}

#[test]
fn candidate_height_is_parent_plus_one_and_overflow_rejects() {
    let c = config(); let mut view = View::new(&c); view.height = u64::MAX;
    assert_rejected(&c, &view, sign(header(&c), enroll()), Error::HeightOverflow);
    assert_eq!(view.calls.get(), 0);
}

#[test]
fn current_and_future_height_anchors_are_rejected() {
    let c = config(); let view = View::new(&c);
    for height in [8, 9, u64::MAX] {
        let mut h = header(&c); h.anchor_height = height;
        assert_rejected(&c, &view, sign(h, enroll()), Error::AnchorNotPast);
    }
}

#[test]
fn anchor_age_maximum_is_inclusive() {
    let c = config(); let mut view = View::new(&c); let mut h = header(&c); h.valid_until = 100;
    let op = sign(h, enroll()); view.height = 38; // candidate 39 minus anchor 7 = 32
    assert_eq!(op.validate_for_parent(&c, &view), Ok(()));
    view.height = 39;
    assert_rejected(&c, &view, op, Error::AnchorTooOld);
}

#[test]
fn unknown_canonical_ancestor_is_rejected() {
    let c = config(); let mut view = View::new(&c); view.hashes.clear();
    assert_rejected(&c, &view, sign(header(&c), enroll()), Error::UnknownAnchor);
}

#[test]
fn wrong_branch_hash_is_rejected_at_same_height() {
    let c = config(); let mut view = View::new(&c); view.hashes.insert(7, hash(0x45));
    assert_rejected(&c, &view, sign(header(&c), enroll()), Error::WrongAnchorHash);
}

#[test]
fn candidate_branch_view_not_old_canonical_branch_controls_anchor() {
    let c = config(); let old = View::new(&c); let mut candidate = old.clone();
    candidate.hashes.insert(7, hash(0x45));
    let mut h = header(&c); h.anchor_hash = hash(0x45); let op = sign(h, enroll());
    assert_eq!(op.validate_for_parent(&c, &candidate), Ok(()));
    assert_rejected(&c, &old, op, Error::WrongAnchorHash);
}

#[test]
fn both_authorization_endpoints_are_inclusive() {
    let c = config(); let mut view = View::new(&c); let op = sign(header(&c), enroll());
    assert_eq!(op.validate_for_parent(&c, &view), Ok(())); // h=8, from=8
    view.height = 23;
    assert_eq!(op.validate_for_parent(&c, &view), Ok(())); // h=24, until=24
    view.height = 24;
    assert_rejected(&c, &view, op, Error::Expired);
}

#[test]
fn future_window_is_not_yet_valid() {
    let c = config(); let mut h = header(&c); h.valid_from = 9;
    assert_rejected(&c, &View::new(&c), sign(h, enroll()), Error::NotYetValid);
}

#[test]
fn anchor_must_precede_window_not_merely_candidate() {
    let c = config(); let view = View::new(&c);
    for from in [6, 7] {
        let mut h = header(&c); h.valid_from = from;
        assert_rejected(&c, &view, sign(h, enroll()), Error::AnchorNotBeforeWindow);
    }
}

#[test]
fn inverted_and_overflowing_windows_reject_without_wrap() {
    let c = config(); let view = View::new(&c); let mut h = header(&c);
    h.valid_from = 24; h.valid_until = 8;
    assert_rejected(&c, &view, sign(h, enroll()), Error::InvertedWindow);
    h.valid_from = 0; h.valid_until = u64::MAX;
    assert_rejected(&c, &view, sign(h, enroll()), Error::WindowOverflow);
}

#[test]
fn maximum_window_span_is_inclusive_and_checked() {
    let c = config(); let view = View::new(&c); let mut h = header(&c); h.valid_until = 263;
    assert_eq!(sign(h, enroll()).validate_for_parent(&c, &view), Ok(())); // 263-8+1=256
    h.valid_until = 264;
    assert_rejected(&c, &view, sign(h, enroll()), Error::WindowTooLong);
}

#[test]
fn lowered_window_and_anchor_limits_are_enforced() {
    let c = config_with(LimitsV2 { max_anchor_age: 1, max_validity_blocks: 1, ..LimitsV2::default() });
    let mut h = header(&c); h.valid_until = h.valid_from;
    assert_eq!(sign(h, enroll()).validate_for_parent(&c, &View::new(&c)), Ok(()));
    h.valid_until += 1;
    assert_rejected(&c, &View::new(&c), sign(h, enroll()), Error::WindowTooLong);
    let mut view = View::new(&c); view.height = 8; h.valid_from = 9; h.valid_until = 9;
    assert_rejected(&c, &view, sign(h, enroll()), Error::AnchorTooOld);
}

#[test]
fn maximum_height_and_genesis_anchor_boundaries_work() {
    let c = config(); let mut view = View::new(&c); let mut h = header(&c);
    view.height = u64::MAX-1; h.anchor_height = u64::MAX-2;
    h.valid_from = u64::MAX-1; h.valid_until = u64::MAX;
    view.hashes = [(h.anchor_height, h.anchor_hash)].into();
    assert_eq!(sign(h, enroll()).validate_for_parent(&c, &view), Ok(()));
    view.height = 0; h.anchor_height = 0; h.valid_from = 1; h.valid_until = 1;
    view.hashes = [(0, h.anchor_hash)].into();
    assert_eq!(sign(h, enroll()).validate_for_parent(&c, &view), Ok(()));
}

#[test]
fn nonce_evidence_and_required_profile_are_checked() {
    let c = config(); let view = View::new(&c);
    for action in [
        Action::Enroll { applicant_nonce: Hash256::default(), evidence_commitment: hash(0x66), evidence_profile: hash(0x22) },
        Action::Enroll { applicant_nonce: hash(0x55), evidence_commitment: Hash256::default(), evidence_profile: hash(0x22) },
    ] { assert_rejected(&c, &view, sign(header(&c), action), Error::ZeroCommitment); }
    let action = Action::Enroll { applicant_nonce: hash(0x55), evidence_commitment: hash(0x66), evidence_profile: hash(0x23) };
    assert_rejected(&c, &view, sign(header(&c), action), Error::WrongEvidenceProfile);
}

#[test]
fn enrollment_must_sign_sequence_zero() {
    let c = config(); let view = View::new(&c);
    for seq in [1, u64::MAX] {
        let mut h = header(&c); h.actor_sequence = seq;
        assert_rejected(&c, &view, sign(h, enroll()), Error::InvalidEnrollmentSequence);
    }
}

#[test]
fn contextual_checks_do_not_claim_stateful_permission_or_consume_sequences() {
    let c = config(); let view = View::new(&c); let mut h = header(&c); h.actor_sequence = u64::MAX;
    let op = sign(h, Action::Approve { target_enrollment_id: hash(0xfe) });
    // B3 must reject exhausted/wrong sequences, unknown targets and ineligible
    // reviewers. B2 explicitly has no registry and cannot establish those facts.
    assert_eq!(op.validate_for_parent(&c, &view), Ok(()));
    assert_eq!(op.header().actor_sequence, u64::MAX);
}

#[test]
fn complete_unsigned_request_determines_enrollment_id() {
    let c = config(); let op = sign(header(&c), enroll()); let original = op.encode();
    for offset in 3..original.len()-64 {
        let mut b = original.clone(); b[offset] ^= 1;
        let changed = Op::decode(&c, &b).unwrap();
        assert_ne!(changed.enrollment_id(), op.enrollment_id());
    }
    assert_eq!(samples(&c)[1].enrollment_id(), None);
    assert_eq!(samples(&c)[2].enrollment_id(), None);
}

#[test]
fn changed_signature_changes_operation_id_but_not_enrollment_id() {
    let c = config(); let op = sign(header(&c), enroll()); let mut b = op.encode();
    let last = b.len()-1; b[last] ^= 1; let changed = Op::decode(&c, &b).unwrap();
    assert_ne!(changed.id(), op.id());
    assert_eq!(changed.enrollment_id(), op.enrollment_id());
    assert!(changed.verify_signature().is_err());
}

#[test]
fn approval_signature_binds_the_exact_request_id() {
    let c = config(); let request = sign(header(&c), enroll());
    let approval = sign(header(&c), Action::Approve { target_enrollment_id: request.enrollment_id().unwrap() });
    let mut h = header(&c); h.valid_until -= 1;
    let other = sign(h, enroll()).enrollment_id().unwrap();
    let changed = Op::from_parts(*approval.header(), Action::Approve { target_enrollment_id: other }, approval.signature());
    assert!(changed.verify_signature().is_err());
}

#[test]
fn immutable_signed_value_is_not_changed_by_construction_inputs() {
    let c = config(); let mut h = header(&c); let op = sign(h, enroll());
    h.actor_sequence = 30;
    assert_eq!(op.header().actor_sequence, 0);
    assert_ne!(op.header(), &h);
}

#[test]
fn parent_lookup_is_bounded_and_validation_has_no_clock_input() {
    let c = config(); let view = View::new(&c); let op = sign(header(&c), enroll());
    for _ in 0..3 { assert_eq!(op.validate_for_parent(&c, &view), Ok(())); }
    assert_eq!(view.calls.get(), 3); // exactly one local ancestor lookup per call
}

#[test]
fn independent_python_preimages_signatures_and_ids_match() {
    let expected: BTreeMap<_,_> = include_str!("vectors/operations_v2.txt").lines()
        .map(|line| line.split_once('=').unwrap()).collect();
    let c = config();
    assert_eq!(hex(&c.encode()), expected["config"]);
    assert_eq!(c.context().to_hex(), expected["context"]);
    assert_eq!(key().public_key().to_hex(), expected["public_key"]);
    for (name, op) in ["enroll", "approve", "withdraw"].into_iter().zip(samples(&c)) {
        for (suffix, value) in [("unsigned", hex(&op.unsigned_bytes())), ("hash", op.signing_hash().to_hex()),
            ("signature", op.signature().to_hex()), ("signed", hex(&op.encode())), ("id", op.id().to_hex())] {
            assert_eq!(value, expected[format!("{name}_{suffix}").as_str()]);
        }
        let bytes = unhex(expected[format!("{name}_signed").as_str()]);
        assert_eq!(Op::decode(&c, &bytes).unwrap().verify_signature(), Ok(()));
    }
    assert_eq!(samples(&c)[0].enrollment_id().unwrap().to_hex(), expected["enrollment_id"]);
}
