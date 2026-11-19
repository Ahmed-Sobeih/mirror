//! Synthetic MB-02A authorization/codec checks. These do not execute a ledger.
use std::{cell::Cell, collections::BTreeMap};

use mirror_core::{
    Address, SignedTransaction, TRANSACTION_KIND_TRANSFER, TRANSACTION_VERSION, TransactionBody,
    v2::{
        SignedOperationV2, SignedTransferV2, TRANSFER_KIND_V2, TRANSFER_OPERATION_LEN_V2,
        TransferBodyV2, TransferErrorV2 as Error, UnifiedOperationErrorV2 as UnifiedError,
    },
};
use mirror_crypto::{CryptoError, Hash256, Keypair, PublicKey, SignatureBytes, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    OPERATION_PREFIX_LEN_V2, OperationErrorV2 as MemberError, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as MemberOperation,
};

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

fn key() -> Keypair {
    Keypair::from_secret_bytes([1; 32])
}

fn config_with(limits: LimitsV2) -> ConfigV2 {
    ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(0x11),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 1,
            evidence_profile: hash(0x22),
        },
        limits,
        &[BootstrapDescriptor {
            public_key: key().public_key(),
            nonce: hash(3),
            evidence: hash(4),
        }],
        &[],
    )
    .unwrap()
}

fn config() -> ConfigV2 {
    config_with(LimitsV2::default())
}

fn header(config: &ConfigV2) -> Header {
    Header {
        config_hash: config.context(),
        anchor_height: 7,
        anchor_hash: hash(0x44),
        valid_from: 8,
        valid_until: 24,
        actor_public_key: key().public_key(),
        actor_sequence: 9,
    }
}

fn body() -> TransferBodyV2 {
    TransferBodyV2 {
        recipient: Address::from_bytes([0x55; 32]),
        amount: 10_000_000,
        fee: 0,
    }
}

fn sign(header: Header, body: TransferBodyV2) -> SignedTransferV2 {
    SignedTransferV2::sign(header, body, &key()).unwrap()
}

fn sample(config: &ConfigV2) -> SignedTransferV2 {
    sign(header(config), body())
}

struct View {
    context: Hash256,
    height: u64,
    hashes: BTreeMap<u64, Hash256>,
    calls: Cell<usize>,
}

impl View {
    fn new(config: &ConfigV2) -> Self {
        Self {
            context: config.context(),
            height: 7,
            hashes: [(7, hash(0x44))].into(),
            calls: Cell::new(0),
        }
    }
}

impl ParentChainViewV2 for View {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        self.height
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        self.calls.set(self.calls.get() + 1);
        self.hashes.get(&height).copied()
    }
}

fn members(config: &ConfigV2) -> [MemberOperation; 3] {
    let actions = [
        Action::Enroll {
            applicant_nonce: hash(0x66),
            evidence_commitment: hash(0x77),
            evidence_profile: config.parameters().evidence_profile,
        },
        Action::Approve {
            target_enrollment_id: hash(0x88),
        },
        Action::Withdraw {
            own_member_id: hash(0x99),
        },
    ];
    actions.map(|action| {
        let mut header = header(config);
        if action.kind() == 1 {
            header.actor_sequence = 0;
        }
        MemberOperation::sign(header, action, &key()).unwrap()
    })
}

fn all_operations(config: &ConfigV2) -> Vec<SignedOperationV2> {
    let mut operations = vec![sample(config).into()];
    operations.extend(members(config).map(SignedOperationV2::from));
    operations
}

#[test]
fn transfer_has_exact_frozen_offsets_and_length() {
    let c = config();
    let operation = sample(&c);
    let bytes = operation.encode();
    assert_eq!(TRANSFER_OPERATION_LEN_V2, 243);
    assert_eq!(TRANSFER_KIND_V2, 0);
    assert_eq!(operation.unsigned_bytes().len(), 179);
    assert_eq!(bytes.len(), 243);
    assert_eq!(&bytes[0..2], &2u16.to_le_bytes());
    assert_eq!(bytes[2], 0);
    assert_eq!(&bytes[3..35], c.context().as_bytes());
    assert_eq!(&bytes[35..43], &7u64.to_le_bytes());
    assert_eq!(&bytes[43..75], hash(0x44).as_bytes());
    assert_eq!(&bytes[75..83], &8u64.to_le_bytes());
    assert_eq!(&bytes[83..91], &24u64.to_le_bytes());
    assert_eq!(&bytes[91..123], key().public_key().as_bytes());
    assert_eq!(&bytes[123..131], &9u64.to_le_bytes());
    assert_eq!(&bytes[131..163], body().recipient.as_bytes());
    assert_eq!(&bytes[163..171], &10_000_000u64.to_le_bytes());
    assert_eq!(&bytes[171..179], &0u64.to_le_bytes());
    assert_eq!(&bytes[179..], operation.signature().as_bytes());
}

#[test]
fn common_prefix_matches_existing_membership_encoding_except_kind() {
    let c = config();
    let operation = sample(&c);
    let membership = MemberOperation::sign(
        *operation.header(),
        Action::Withdraw {
            own_member_id: hash(0x88),
        },
        &key(),
    )
    .unwrap();
    let transfer_bytes = operation.encode();
    let member_bytes = membership.encode();
    assert_eq!(OPERATION_PREFIX_LEN_V2, 131);
    assert_eq!(&transfer_bytes[..2], &member_bytes[..2]);
    assert_ne!(transfer_bytes[2], member_bytes[2]);
    assert_eq!(&transfer_bytes[3..131], &member_bytes[3..131]);
}

#[test]
fn unified_round_trip_preserves_all_kinds_bytes_hashes_and_signatures() {
    let c = config();
    let expected_kinds = [0, 1, 2, 3];
    for (operation, kind) in all_operations(&c).into_iter().zip(expected_kinds) {
        let bytes = operation.encode();
        let decoded = SignedOperationV2::decode(&c, &bytes).unwrap();
        assert_eq!(decoded, operation);
        assert_eq!(decoded.kind(), kind);
        assert_eq!(decoded.encode(), bytes);
        assert_eq!(decoded.id(), operation.id());
        assert_eq!(decoded.signing_hash(), operation.signing_hash());
        assert_eq!(decoded.verify_signature(), Ok(()));
        assert_eq!(decoded.validate_for_parent(&c, &View::new(&c)), Ok(()));
    }
}

#[test]
fn membership_only_decoder_remains_closed_to_transfer_kind() {
    let c = config();
    let bytes = sample(&c).encode();
    assert_eq!(
        MemberOperation::decode(&c, &bytes),
        Err(MemberError::UnsupportedMembershipKind)
    );
    assert!(matches!(
        SignedOperationV2::decode(&c, &bytes),
        Ok(SignedOperationV2::Transfer(_))
    ));
}

#[test]
fn transfer_decoder_rejects_every_other_kind() {
    let c = config();
    let bytes = sample(&c).encode();
    for kind in 1u8..=255 {
        let mut changed = bytes.clone();
        changed[2] = kind;
        assert_eq!(
            SignedTransferV2::decode(&c, &changed),
            Err(Error::UnsupportedTransferKind)
        );
    }
}

#[test]
fn every_transfer_truncation_and_unified_truncation_is_rejected() {
    let c = config();
    let bytes = sample(&c).encode();
    for end in 0..bytes.len() {
        assert!(SignedTransferV2::decode(&c, &bytes[..end]).is_err());
        assert!(SignedOperationV2::decode(&c, &bytes[..end]).is_err());
    }
}

#[test]
fn trailing_bytes_and_known_kind_wrong_lengths_are_rejected() {
    let c = config();
    let original = sample(&c).encode();
    let mut trailing = original.clone();
    trailing.push(0);
    assert_eq!(
        SignedTransferV2::decode(&c, &trailing),
        Err(Error::InvalidLength)
    );
    assert_eq!(
        SignedOperationV2::decode(&c, &trailing),
        Err(UnifiedError::Transfer(Error::InvalidLength))
    );
    for kind in 1..=3 {
        let mut changed = original.clone();
        changed[2] = kind;
        assert_eq!(
            SignedOperationV2::decode(&c, &changed),
            Err(UnifiedError::Membership(MemberError::InvalidLength))
        );
    }
}

#[test]
fn versions_unknown_kinds_and_legacy_inputs_have_no_fallback() {
    let c = config();
    let original = sample(&c).encode();
    for version in [0u16, 1, 3, u16::MAX] {
        let mut changed = original.clone();
        changed[..2].copy_from_slice(&version.to_le_bytes());
        assert_eq!(
            SignedTransferV2::decode(&c, &changed),
            Err(Error::UnsupportedVersion)
        );
        assert_eq!(
            SignedOperationV2::decode(&c, &changed),
            Err(UnifiedError::UnsupportedVersion)
        );
    }
    for kind in 4u8..=255 {
        let mut changed = original.clone();
        changed[2] = kind;
        assert_eq!(
            SignedOperationV2::decode(&c, &changed),
            Err(UnifiedError::UnsupportedKind)
        );
    }
    for magic in [b"MPH1", b"MRB1", b"MCF2"] {
        assert_eq!(
            SignedOperationV2::decode(&c, magic),
            Err(UnifiedError::UnsupportedVersion)
        );
    }
}

#[test]
fn oversized_input_rejects_before_version_or_kind_parsing() {
    let c = config();
    let bytes = vec![0; 1_025];
    assert_eq!(SignedTransferV2::decode(&c, &bytes), Err(Error::TooLarge));
    assert_eq!(
        SignedOperationV2::decode(&c, &bytes),
        Err(UnifiedError::TooLarge)
    );
}

#[test]
fn smallest_configuration_operation_cap_accepts_every_supported_kind() {
    let c = config_with(LimitsV2 {
        max_operation_bytes: 291,
        ..LimitsV2::default()
    });
    for operation in all_operations(&c) {
        assert_eq!(
            SignedOperationV2::decode(&c, &operation.encode()),
            Ok(operation)
        );
    }
    assert_eq!(
        SignedOperationV2::decode(&c, &[0; 292]),
        Err(UnifiedError::TooLarge)
    );
}

#[test]
fn every_unsigned_transfer_byte_is_authenticated() {
    let c = config();
    let original = sample(&c).encode();
    // Version and kind are constrained structurally by separate tests.
    for offset in 3..179 {
        let mut changed = original.clone();
        changed[offset] ^= 1;
        let decoded = SignedTransferV2::decode(&c, &changed).unwrap();
        assert!(
            decoded.verify_signature().is_err(),
            "unauthenticated byte {offset}"
        );
    }
}

#[test]
fn every_transfer_signature_byte_is_checked() {
    let c = config();
    let original = sample(&c).encode();
    for offset in 179..243 {
        let mut changed = original.clone();
        changed[offset] ^= 1;
        let decoded = SignedOperationV2::decode(&c, &changed).unwrap();
        assert!(
            decoded.verify_signature().is_err(),
            "unchecked signature byte {offset}"
        );
    }
}

#[test]
fn wrong_signing_key_is_rejected_before_signature_construction() {
    let c = config();
    assert_eq!(
        SignedTransferV2::sign(header(&c), body(), &Keypair::from_secret_bytes([2; 32])),
        Err(Error::WrongSigningKey)
    );
}

#[test]
fn weak_key_and_invalid_signature_inputs_cannot_authorize() {
    let c = config();
    let mut h = header(&c);
    let mut identity = [0u8; 32];
    identity[0] = 1;
    h.actor_public_key = PublicKey::from_bytes(identity);
    let operation = SignedTransferV2::from_parts(h, body(), SignatureBytes::from_bytes([0; 64]));
    assert_eq!(
        operation.verify_signature(),
        Err(Error::Crypto(CryptoError::WeakPublicKey))
    );
    let invalid =
        SignedTransferV2::from_parts(header(&c), body(), SignatureBytes::from_bytes([0; 64]));
    assert!(matches!(invalid.verify_signature(), Err(Error::Crypto(_))));
    assert!(matches!(
        invalid.validate_for_parent(&c, &View::new(&c)),
        Err(Error::Crypto(_))
    ));
}

#[test]
fn decoded_corrupt_signature_remains_untrusted_until_validation() {
    let c = config();
    let mut bytes = sample(&c).encode();
    bytes[220] ^= 1;
    let operation = SignedTransferV2::decode(&c, &bytes).unwrap();
    assert_eq!(operation.encode(), bytes);
    assert!(matches!(
        operation.validate_for_parent(&c, &View::new(&c)),
        Err(Error::Crypto(_))
    ));
}

#[test]
fn raw_hash_and_seed_domain_signatures_cannot_authorize_transfer() {
    let c = config();
    let original = sample(&c);
    let raw_signature = key().sign_hash(&sha256d(&original.unsigned_bytes()));
    let seed_signature = c.sign_seed(0, &key()).unwrap();
    for signature in [raw_signature, seed_signature] {
        let changed = SignedTransferV2::from_parts(*original.header(), *original.body(), signature);
        assert!(changed.verify_signature().is_err());
    }
    assert_eq!(original.verify_signature(), Ok(()));
}

#[test]
fn membership_signature_cannot_be_transplanted_into_transfer() {
    let c = config();
    for membership in members(&c) {
        assert_eq!(membership.verify_signature(), Ok(()));
        let changed =
            SignedTransferV2::from_parts(*membership.header(), body(), membership.signature());
        assert!(changed.verify_signature().is_err());
    }
}

#[test]
fn legacy_transaction_signature_cannot_be_transplanted_into_v2() {
    let c = config();
    let legacy = SignedTransaction::sign(
        TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            1,
            9,
            Address::from_public_key(&key().public_key()),
            body().recipient,
            body().amount,
            0,
            Vec::new(),
        ),
        &key(),
    )
    .unwrap();
    assert_eq!(legacy.verify(), Ok(()));
    let changed = SignedTransferV2::from_parts(header(&c), body(), legacy.signature());
    assert!(changed.verify_signature().is_err());
    assert!(SignedOperationV2::decode(&c, &legacy.encode().unwrap()).is_err());
}

#[test]
fn changing_signature_changes_operation_id_without_changing_signing_hash() {
    let c = config();
    let original = sample(&c);
    let mut signature = *original.signature().as_bytes();
    signature[0] ^= 1;
    let changed = SignedTransferV2::from_parts(
        *original.header(),
        *original.body(),
        SignatureBytes::from_bytes(signature),
    );
    assert_eq!(changed.signing_hash(), original.signing_hash());
    assert_ne!(changed.id(), original.id());
    assert!(changed.verify_signature().is_err());
}

#[test]
fn wrong_operation_and_parent_configuration_are_rejected() {
    let c = config();
    let view = View::new(&c);
    let mut h = header(&c);
    h.config_hash = hash(0x33);
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &view),
        Err(Error::WrongContext)
    );
    assert_eq!(view.calls.get(), 0);
    let mut wrong_parent = View::new(&c);
    wrong_parent.context = hash(0x33);
    assert_eq!(
        sample(&c).validate_for_parent(&c, &wrong_parent),
        Err(Error::WrongParentContext)
    );
    assert_eq!(wrong_parent.calls.get(), 0);
}

#[test]
fn changing_committed_configuration_does_not_reauthorize_old_signature() {
    let c = config();
    let changed = config_with(LimitsV2 {
        max_pending: 1_023,
        ..LimitsV2::default()
    });
    assert_ne!(c.context(), changed.context());
    let original = sample(&c);
    assert_eq!(original.verify_signature(), Ok(()));
    assert_eq!(
        original.validate_for_parent(&changed, &View::new(&changed)),
        Err(Error::WrongContext)
    );
}

#[test]
fn height_overflow_is_rejected_before_ancestor_lookup() {
    let c = config();
    let mut view = View::new(&c);
    view.height = u64::MAX;
    assert_eq!(
        sample(&c).validate_for_parent(&c, &view),
        Err(Error::HeightOverflow)
    );
    assert_eq!(view.calls.get(), 0);
}

#[test]
fn present_or_future_anchor_is_rejected() {
    let c = config();
    for anchor_height in [8, 9, u64::MAX] {
        let mut h = header(&c);
        h.anchor_height = anchor_height;
        assert_eq!(
            sign(h, body()).validate_for_parent(&c, &View::new(&c)),
            Err(Error::AnchorNotPast)
        );
    }
}

#[test]
fn maximum_anchor_age_is_inclusive_and_one_more_rejects() {
    let c = config();
    let mut h = header(&c);
    h.valid_until = 40;
    let operation = sign(h, body());
    let mut view = View::new(&c);
    view.height = 38;
    assert_eq!(operation.validate_for_parent(&c, &view), Ok(()));
    view.height = 39;
    assert_eq!(
        operation.validate_for_parent(&c, &view),
        Err(Error::AnchorTooOld)
    );
}

#[test]
fn inverted_and_overflowing_windows_are_rejected() {
    let c = config();
    let mut h = header(&c);
    h.valid_from = 9;
    h.valid_until = 8;
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &View::new(&c)),
        Err(Error::InvertedWindow)
    );
    h.valid_from = 0;
    h.valid_until = u64::MAX;
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &View::new(&c)),
        Err(Error::WindowOverflow)
    );
}

#[test]
fn maximum_window_span_is_inclusive_and_one_more_rejects() {
    let c = config();
    let mut h = header(&c);
    h.valid_until = h.valid_from + c.limits().max_validity_blocks - 1;
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &View::new(&c)),
        Ok(())
    );
    h.valid_until += 1;
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &View::new(&c)),
        Err(Error::WindowTooLong)
    );
}

#[test]
fn anchor_must_precede_window_start() {
    let c = config();
    let mut h = header(&c);
    h.valid_from = 7;
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &View::new(&c)),
        Err(Error::AnchorNotBeforeWindow)
    );
}

#[test]
fn inclusion_window_accepts_both_edges_and_rejects_outside() {
    let c = config();
    let operation = sample(&c);
    let mut view = View::new(&c);
    assert_eq!(operation.validate_for_parent(&c, &view), Ok(()));
    view.height = 23;
    assert_eq!(operation.validate_for_parent(&c, &view), Ok(()));
    view.height = 24;
    assert_eq!(
        operation.validate_for_parent(&c, &view),
        Err(Error::Expired)
    );
    let mut h = header(&c);
    h.valid_from = 9;
    assert_eq!(
        sign(h, body()).validate_for_parent(&c, &View::new(&c)),
        Err(Error::NotYetValid)
    );
}

#[test]
fn missing_and_competing_branch_ancestors_are_rejected() {
    let c = config();
    let operation = sample(&c);
    let mut view = View::new(&c);
    view.hashes.clear();
    assert_eq!(
        operation.validate_for_parent(&c, &view),
        Err(Error::UnknownAnchor)
    );
    view.hashes.insert(7, hash(0x45));
    assert_eq!(
        operation.validate_for_parent(&c, &view),
        Err(Error::WrongAnchorHash)
    );
}

#[test]
fn zero_amount_is_signed_and_decoded_but_rejected_contextually() {
    let c = config();
    let operation = sign(
        header(&c),
        TransferBodyV2 {
            amount: 0,
            ..body()
        },
    );
    assert_eq!(operation.verify_signature(), Ok(()));
    assert_eq!(
        SignedTransferV2::decode(&c, &operation.encode()),
        Ok(operation)
    );
    let view = View::new(&c);
    assert_eq!(
        operation.validate_for_parent(&c, &view),
        Err(Error::ZeroAmount)
    );
    assert_eq!(view.calls.get(), 0);
}

#[test]
fn nonzero_fees_remain_unsupported_by_synthetic_profile() {
    let c = config();
    for fee in [1, u64::MAX] {
        let operation = sign(header(&c), TransferBodyV2 { fee, ..body() });
        assert_eq!(operation.verify_signature(), Ok(()));
        assert_eq!(
            SignedTransferV2::decode(&c, &operation.encode()),
            Ok(operation)
        );
        let unified = SignedOperationV2::from(operation);
        assert_eq!(
            unified.validate_for_parent(&c, &View::new(&c)),
            Err(UnifiedError::Transfer(Error::UnsupportedFee))
        );
    }
}

#[test]
fn authorization_does_not_claim_balance_current_sequence_or_membership() {
    let c = config();
    assert_eq!(c.initial_test_supply(), 0);
    let outsider = Keypair::from_secret_bytes([0x42; 32]);
    let mut h = header(&c);
    h.actor_public_key = outsider.public_key();
    h.actor_sequence = u64::MAX;
    let operation = SignedTransferV2::sign(
        h,
        TransferBodyV2 {
            amount: u64::MAX,
            ..body()
        },
        &outsider,
    )
    .unwrap();
    let before = operation;
    assert_eq!(operation.validate_for_parent(&c, &View::new(&c)), Ok(()));
    assert_eq!(operation, before);
    assert_eq!(c.initial_test_supply(), 0);
    assert_eq!(operation.header().actor_sequence, u64::MAX);
}

#[test]
fn self_transfer_uses_existing_address_namespace_and_no_hidden_profile_rule() {
    let c = config();
    let address = Address::from_public_key(&key().public_key());
    let operation = sign(
        header(&c),
        TransferBodyV2 {
            recipient: address,
            ..body()
        },
    );
    assert_eq!(operation.sender(), address);
    assert_eq!(operation.body().recipient, address);
    assert_eq!(operation.validate_for_parent(&c, &View::new(&c)), Ok(()));
}

#[test]
fn valid_boundary_at_maximum_height_does_not_wrap() {
    let c = config();
    let mut h = header(&c);
    h.anchor_height = u64::MAX - 2;
    h.valid_from = u64::MAX - 1;
    h.valid_until = u64::MAX;
    let operation = sign(h, body());
    let mut view = View::new(&c);
    view.height = u64::MAX - 1;
    view.hashes = [(h.anchor_height, h.anchor_hash)].into();
    assert_eq!(operation.validate_for_parent(&c, &view), Ok(()));
    assert_eq!(view.calls.get(), 1);
}

#[test]
fn one_block_window_from_genesis_is_valid() {
    let c = config();
    let mut h = header(&c);
    h.anchor_height = 0;
    h.valid_from = 1;
    h.valid_until = 1;
    let mut view = View::new(&c);
    view.height = 0;
    view.hashes = [(0, h.anchor_hash)].into();
    assert_eq!(sign(h, body()).validate_for_parent(&c, &view), Ok(()));
}

#[test]
fn unified_membership_dispatch_preserves_deferred_state_rules_and_exact_errors() {
    let c = config();
    let mut h = header(&c);
    h.actor_sequence = u64::MAX;
    let approval = MemberOperation::sign(
        h,
        Action::Approve {
            target_enrollment_id: hash(0xff),
        },
        &key(),
    )
    .unwrap();
    let unified = SignedOperationV2::from(approval);
    assert_eq!(unified.validate_for_parent(&c, &View::new(&c)), Ok(()));
    let mut changed = h;
    changed.valid_until = 7;
    let expired = MemberOperation::sign(changed, *approval.action(), &key()).unwrap();
    assert_eq!(
        SignedOperationV2::from(expired).validate_for_parent(&c, &View::new(&c)),
        Err(UnifiedError::Membership(MemberError::InvertedWindow))
    );
}

#[test]
fn failed_authorization_and_input_mutation_leave_signed_value_unchanged() {
    let c = config();
    let mut h = header(&c);
    let mut b = body();
    let operation = sign(h, b);
    let original = operation;
    h.actor_sequence = 0;
    b.amount = 0;
    assert_ne!(h.actor_sequence, operation.header().actor_sequence);
    assert_ne!(b.amount, operation.body().amount);
    let mut view = View::new(&c);
    view.hashes.clear();
    assert_eq!(
        operation.validate_for_parent(&c, &view),
        Err(Error::UnknownAnchor)
    );
    assert_eq!(operation, original);
    let valid_view = View::new(&c);
    assert_eq!(operation.validate_for_parent(&c, &valid_view), Ok(()));
    assert_eq!(valid_view.calls.get(), 1);
}
