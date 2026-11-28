//! Mixed V2 ledger transitions use public synthetic keys and a trusted fixture
//! view. These tests do not stand in for actual block/genesis/PoW acceptance.

use std::{cell::Cell, collections::BTreeMap};

use mirror_core::{
    Address,
    v2::{
        BlockErrorV2, SignedOperationV2 as Operation, SignedTransferV2 as Transfer,
        TRANSFER_OPERATION_LEN_V2, TransferBodyV2, TransferErrorV2, validate_operation_limits_v2,
    },
};
use mirror_crypto::{Hash256, Keypair, SignatureBytes, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipRegistryV2, MembershipStatusV2 as Status, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as MemberOperation, StateErrorV2,
    TestAllocation, UntrustedMembershipSnapshotV2, VerifiedBootstrap,
};
use mirror_state::v2::{
    LedgerErrorV2 as Error, LedgerOperationErrorV2 as OperationError, LedgerV2 as Ledger,
};

fn tag(domain: u8, number: u64) -> Hash256 {
    let mut bytes = [0; 32];
    bytes[0] = domain;
    bytes[1..9].copy_from_slice(&number.to_le_bytes());
    Hash256::from_bytes(bytes)
}

fn key(number: u64) -> Keypair {
    let mut bytes = [54; 32];
    bytes[..8].copy_from_slice(&number.to_le_bytes());
    Keypair::from_secret_bytes(bytes)
}

fn address(number: u64) -> Address {
    Address::from_public_key(&key(number).public_key())
}

fn block_hash(height: u64) -> Hash256 {
    tag(10, height)
}

fn verified(limits: LimitsV2, allocations: &[(u64, u64)], threshold: u16) -> VerifiedBootstrap {
    let descriptors: Vec<_> = (1..=2)
        .map(|number| BootstrapDescriptor {
            public_key: key(number).public_key(),
            nonce: tag(1, number),
            evidence: tag(2, number),
        })
        .collect();
    let mut allocations: Vec<_> = allocations
        .iter()
        .map(|&(actor, balance)| TestAllocation {
            address: *address(actor).as_bytes(),
            balance_nusa: balance,
        })
        .collect();
    allocations.sort_unstable_by_key(|allocation| allocation.address);
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: tag(3, 0),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: threshold,
            evidence_profile: tag(4, 0),
        },
        limits,
        &descriptors,
        &allocations,
    )
    .unwrap();
    let proofs: Vec<_> = (1..=2)
        .map(|actor| config.sign_seed((actor - 1) as u32, &key(actor)).unwrap())
        .collect();
    config.verify_bootstrap(&proofs).unwrap()
}

fn ledger_with(limits: LimitsV2, allocations: &[(u64, u64)], threshold: u16) -> Ledger {
    Ledger::bootstrap(&verified(limits, allocations, threshold))
        .unwrap()
        .bind_to_validated_block(block_hash(0))
}

fn ledger() -> Ledger {
    ledger_with(LimitsV2::default(), &[(1, 100), (2, 0)], 1)
}

struct View {
    context: Hash256,
    height: u64,
    hashes: BTreeMap<u64, Hash256>,
    calls: Cell<usize>,
}

impl View {
    fn of(ledger: &Ledger) -> Self {
        let height = ledger.height();
        let mut hashes: BTreeMap<_, _> = (0..=height).map(|h| (h, block_hash(h))).collect();
        hashes.insert(height, ledger.tip_hash());
        Self {
            context: ledger.config().context(),
            height,
            hashes,
            calls: Cell::new(0),
        }
    }
}

impl ParentChainViewV2 for View {
    fn config_hash(&self) -> Hash256 {
        self.calls.set(self.calls.get() + 1);
        self.context
    }

    fn parent_height(&self) -> u64 {
        self.calls.set(self.calls.get() + 1);
        self.height
    }

    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        self.calls.set(self.calls.get() + 1);
        self.hashes.get(&height).copied()
    }
}

fn header(ledger: &Ledger, actor: u64, sequence: u64, until: u64) -> Header {
    Header {
        config_hash: ledger.config().context(),
        anchor_height: ledger.height(),
        anchor_hash: ledger.tip_hash(),
        valid_from: ledger.height() + 1,
        valid_until: until,
        actor_public_key: key(actor).public_key(),
        actor_sequence: sequence,
    }
}

fn transfer(ledger: &Ledger, actor: u64, recipient: u64, sequence: u64, amount: u64) -> Transfer {
    Transfer::sign(
        header(ledger, actor, sequence, ledger.height() + 8),
        TransferBodyV2 {
            recipient: address(recipient),
            amount,
            fee: 0,
        },
        &key(actor),
    )
    .unwrap()
}

fn enroll(ledger: &Ledger, actor: u64, until: u64) -> MemberOperation {
    MemberOperation::sign(
        header(ledger, actor, 0, until),
        Action::Enroll {
            applicant_nonce: tag(5, actor),
            evidence_commitment: tag(6, actor),
            evidence_profile: ledger.config().parameters().evidence_profile,
        },
        &key(actor),
    )
    .unwrap()
}

fn approve(
    ledger: &Ledger,
    actor: u64,
    sequence: u64,
    target: Hash256,
    until: u64,
) -> MemberOperation {
    MemberOperation::sign(
        header(ledger, actor, sequence, until),
        Action::Approve {
            target_enrollment_id: target,
        },
        &key(actor),
    )
    .unwrap()
}

fn withdraw(ledger: &Ledger, actor: u64, sequence: u64) -> MemberOperation {
    let id = ledger
        .membership()
        .record_by_key(key(actor).public_key())
        .unwrap()
        .id();
    MemberOperation::sign(
        header(ledger, actor, sequence, ledger.height() + 8),
        Action::Withdraw { own_member_id: id },
        &key(actor),
    )
    .unwrap()
}

fn next(ledger: &Ledger, operations: &[Operation]) -> Ledger {
    ledger
        .stage_operations(&View::of(ledger), operations)
        .unwrap()
        .bind_to_validated_block(block_hash(ledger.height() + 1))
}

fn operation_error(index: u32, error: OperationError) -> Error {
    Error::Operation { index, error }
}

fn assert_unchanged_after(ledger: &Ledger, operations: &[Operation], expected: Error) {
    let before = ledger.clone();
    let root = ledger.root().unwrap();
    let accounts = ledger.account_bytes().unwrap();
    let membership = ledger.membership().encode().unwrap();
    assert_eq!(
        ledger.stage_operations(&View::of(ledger), operations),
        Err(expected)
    );
    assert_eq!(ledger, &before);
    assert_eq!(ledger.root().unwrap(), root);
    assert_eq!(ledger.account_bytes().unwrap(), accounts);
    assert_eq!(ledger.membership().encode().unwrap(), membership);
}

#[test]
fn bootstrap_uses_only_explicit_allocations_including_zero_records() {
    let ledger = ledger_with(LimitsV2::default(), &[(1, 7), (5, 0)], 1);
    assert_eq!(ledger.initial_test_supply(), 7);
    assert_eq!(ledger.account_count(), 2);
    assert_eq!(ledger.account(address(1)).balance(), 7);
    assert!(ledger.contains_account(address(5)));
    assert!(!ledger.contains_account(address(2)));
    assert_eq!(ledger.account(address(2)).balance(), 0);
    assert_eq!(ledger.account(address(2)).next_sequence(), 0);
    assert_eq!(ledger.membership().active_count(), 2);
    assert_eq!(ledger.height(), 0);
}

#[test]
fn empty_allocations_create_no_supply_or_implicit_member_accounts() {
    let ledger = ledger_with(LimitsV2::default(), &[], 1);
    assert_eq!(ledger.initial_test_supply(), 0);
    assert_eq!(ledger.account_count(), 0);
    assert_eq!(ledger.account_bytes().unwrap(), 0u32.to_le_bytes());
    let child = next(&ledger, &[]);
    assert_eq!(child.account_count(), 0);
    assert_eq!(child.initial_test_supply(), 0);
    assert_ne!(child.root().unwrap(), ledger.root().unwrap());
}

#[test]
fn account_encoding_and_component_roots_follow_canonical_domains() {
    let ledger = ledger_with(LimitsV2::default(), &[(1, 41), (2, 0), (3, 9)], 1);
    let mut expected = 3u32.to_le_bytes().to_vec();
    let mut entries = [(address(1), 41u64), (address(2), 0), (address(3), 9)];
    entries.sort_unstable_by_key(|entry| entry.0);
    for (address, balance) in entries {
        expected.extend_from_slice(address.as_bytes());
        expected.extend_from_slice(&balance.to_le_bytes());
        expected.extend_from_slice(&0u64.to_le_bytes());
    }
    assert_eq!(ledger.account_bytes().unwrap(), expected);
    assert_eq!(expected.len(), 4 + 3 * 48);
    let mut accounts_preimage = b"MIRROR_ACCOUNTS_V2".to_vec();
    accounts_preimage.extend_from_slice(&expected);
    assert_eq!(ledger.accounts_root().unwrap(), sha256d(&accounts_preimage));

    let mut economics_preimage = b"MIRROR_ECONOMICS_DISABLED_V2".to_vec();
    economics_preimage.extend_from_slice(&50u64.to_le_bytes());
    assert_eq!(ledger.economics_root(), sha256d(&economics_preimage));

    let mut ledger_preimage = b"MIRROR_LEDGER_STATE_V2".to_vec();
    ledger_preimage.extend_from_slice(ledger.config().context().as_bytes());
    ledger_preimage.extend_from_slice(&0u64.to_le_bytes());
    ledger_preimage.extend_from_slice(ledger.accounts_root().unwrap().as_bytes());
    ledger_preimage.extend_from_slice(ledger.membership().root().unwrap().as_bytes());
    ledger_preimage.extend_from_slice(ledger.economics_root().as_bytes());
    assert_eq!(ledger.root().unwrap(), sha256d(&ledger_preimage));
    let order: Vec<_> = ledger.accounts().map(|(address, _)| *address).collect();
    assert_eq!(order, entries.map(|entry| entry.0));
}

#[test]
fn provenance_hash_is_excluded_from_committed_state() {
    let stage = Ledger::bootstrap(&verified(LimitsV2::default(), &[(1, 100)], 1)).unwrap();
    let root = stage.root().unwrap();
    let first = stage.clone().bind_to_validated_block(tag(20, 1));
    let second = stage.bind_to_validated_block(tag(20, 2));
    assert_ne!(first.tip_hash(), second.tip_hash());
    assert_eq!(first.root().unwrap(), root);
    assert_eq!(second.root().unwrap(), root);
    assert_eq!(
        first.account_bytes().unwrap(),
        second.account_bytes().unwrap()
    );
}

#[test]
fn successful_transfer_creates_recipient_and_preserves_supply() {
    let ledger = ledger();
    let before = ledger.clone();
    let child = next(&ledger, &[transfer(&ledger, 1, 3, 0, 31).into()]);
    assert_eq!(ledger, before);
    assert_eq!(child.account_count(), 3);
    assert_eq!(child.account(address(1)).balance(), 69);
    assert_eq!(child.account(address(1)).next_sequence(), 1);
    assert_eq!(child.account(address(3)).balance(), 31);
    assert_eq!(child.account(address(3)).next_sequence(), 0);
    assert!(
        child
            .membership()
            .record_by_key(key(3).public_key())
            .is_none()
    );
    assert_eq!(child.initial_test_supply(), 100);
    assert_eq!(
        child
            .accounts()
            .map(|(_, account)| account.balance())
            .sum::<u64>(),
        100
    );
    assert_eq!(child.economics_root(), ledger.economics_root());
}

#[test]
fn zero_balance_sender_history_survives_recredit_and_replay() {
    let ledger = ledger();
    let first = transfer(&ledger, 1, 2, 0, 100);
    let child = next(&ledger, &[first.into()]);
    assert!(child.contains_account(address(1)));
    assert_eq!(child.account(address(1)).balance(), 0);
    assert_eq!(child.account(address(1)).next_sequence(), 1);
    let grandchild = next(&child, &[transfer(&child, 2, 1, 0, 40).into()]);
    assert_eq!(grandchild.account(address(1)).balance(), 40);
    assert_eq!(grandchild.account(address(1)).next_sequence(), 1);
    assert_eq!(grandchild.account(address(2)).next_sequence(), 1);
    assert_unchanged_after(
        &grandchild,
        &[first.into()],
        operation_error(
            0,
            OperationError::AccountSequenceMismatch {
                expected: 1,
                got: 0,
            },
        ),
    );
}

#[test]
fn replay_or_skipped_account_sequence_cannot_spend() {
    let ledger = ledger();
    assert_unchanged_after(
        &ledger,
        &[transfer(&ledger, 1, 2, 1, 1).into()],
        operation_error(
            0,
            OperationError::AccountSequenceMismatch {
                expected: 0,
                got: 1,
            },
        ),
    );
    let transfer = transfer(&ledger, 1, 2, 0, 1);
    assert_unchanged_after(
        &ledger,
        &[transfer.into(), transfer.into()],
        operation_error(
            1,
            OperationError::AccountSequenceMismatch {
                expected: 1,
                got: 0,
            },
        ),
    );
}

#[test]
fn missing_sender_or_insufficient_funds_never_creates_accounts() {
    let ledger = ledger();
    for (actor, amount, balance) in [(3, 1, 0), (1, 101, 100), (2, 1, 0)] {
        assert_unchanged_after(
            &ledger,
            &[transfer(&ledger, actor, 4, 0, amount).into()],
            operation_error(
                0,
                OperationError::InsufficientBalance {
                    available: balance,
                    required: amount,
                },
            ),
        );
    }
}

#[test]
fn self_transfer_requires_funds_and_consumes_exactly_one_sequence() {
    let ledger = ledger();
    let child = next(&ledger, &[transfer(&ledger, 1, 1, 0, 100).into()]);
    assert_eq!(child.account(address(1)).balance(), 100);
    assert_eq!(child.account(address(1)).next_sequence(), 1);
    assert_eq!(child.account_count(), ledger.account_count());
    assert_ne!(
        child.accounts_root().unwrap(),
        ledger.accounts_root().unwrap()
    );
    assert_unchanged_after(
        &ledger,
        &[transfer(&ledger, 1, 1, 0, 101).into()],
        operation_error(
            0,
            OperationError::InsufficientBalance {
                available: 100,
                required: 101,
            },
        ),
    );
}

#[test]
fn maximum_supply_self_transfer_and_full_transfer_do_not_overflow() {
    let ledger = ledger_with(LimitsV2::default(), &[(1, u64::MAX)], 1);
    let child = next(&ledger, &[transfer(&ledger, 1, 1, 0, u64::MAX).into()]);
    assert_eq!(child.account(address(1)).balance(), u64::MAX);
    assert_eq!(child.account(address(1)).next_sequence(), 1);
    let grandchild = next(&child, &[transfer(&child, 1, 2, 1, u64::MAX).into()]);
    assert_eq!(grandchild.account(address(1)).balance(), 0);
    assert_eq!(grandchild.account(address(2)).balance(), u64::MAX);
    assert_eq!(grandchild.initial_test_supply(), u64::MAX);
}

#[test]
fn zero_amount_nonzero_fee_and_bad_signature_precede_state_changes() {
    let ledger = ledger();
    for (amount, fee, expected) in [
        (0, 0, TransferErrorV2::ZeroAmount),
        (1, 1, TransferErrorV2::UnsupportedFee),
    ] {
        let transfer = Transfer::sign(
            header(&ledger, 1, 7, 8),
            TransferBodyV2 {
                recipient: address(2),
                amount,
                fee,
            },
            &key(1),
        )
        .unwrap();
        assert_unchanged_after(
            &ledger,
            &[transfer.into()],
            operation_error(0, OperationError::Transfer(expected)),
        );
    }
    let good = transfer(&ledger, 1, 2, 0, 1);
    let bad = Transfer::from_parts(
        *good.header(),
        *good.body(),
        SignatureBytes::from_bytes([0; 64]),
    );
    let before = ledger.clone();
    assert!(matches!(
        ledger.stage_operations(&View::of(&ledger), &[bad.into()]),
        Err(Error::Operation {
            index: 0,
            error: OperationError::Transfer(TransferErrorV2::Crypto(_))
        })
    ));
    assert_eq!(ledger, before);
}

#[test]
fn account_limit_rejects_new_recipient_but_keeps_existing_and_zero_records() {
    let limits = LimitsV2 {
        max_accounts: 2,
        ..LimitsV2::default()
    };
    let ledger = ledger_with(limits, &[(1, 100), (2, 0)], 1);
    assert_unchanged_after(
        &ledger,
        &[transfer(&ledger, 1, 3, 0, 100).into()],
        operation_error(0, OperationError::AccountLimit),
    );
    let child = next(&ledger, &[transfer(&ledger, 1, 2, 0, 100).into()]);
    assert_eq!(child.account_count(), 2);
    assert!(child.contains_account(address(1)));
    assert_unchanged_after(
        &child,
        &[transfer(&child, 2, 3, 0, 1).into()],
        operation_error(0, OperationError::AccountLimit),
    );
    let returned = next(&child, &[transfer(&child, 2, 1, 0, 1).into()]);
    assert_eq!(returned.account(address(1)).balance(), 1);
    assert_eq!(returned.account(address(1)).next_sequence(), 1);
}

#[test]
fn transfer_and_membership_sequences_are_independent() {
    let ledger = ledger();
    let enrollment = enroll(&ledger, 3, 8);
    let enrollment_id = enrollment.enrollment_id().unwrap();
    let operations = [
        transfer(&ledger, 1, 2, 0, 10).into(),
        enrollment.into(),
        approve(&ledger, 1, 0, enrollment_id, 8).into(),
        transfer(&ledger, 1, 2, 1, 10).into(),
    ];
    let child = next(&ledger, &operations);
    assert_eq!(child.account(address(1)).next_sequence(), 2);
    assert_eq!(
        child
            .membership()
            .record_by_key(key(1).public_key())
            .unwrap()
            .next_sequence(),
        1
    );
    assert_eq!(
        child.membership().record(enrollment_id).unwrap().status(),
        Status::Active
    );
    assert!(!child.contains_account(address(3)));
    assert_eq!(child.initial_test_supply(), 100);
}

#[test]
fn withdrawing_membership_does_not_destroy_or_restrict_accounts() {
    let ledger = ledger();
    let operations = [
        withdraw(&ledger, 1, 0).into(),
        transfer(&ledger, 1, 3, 0, 25).into(),
    ];
    let child = next(&ledger, &operations);
    assert_eq!(
        child
            .membership()
            .record_by_key(key(1).public_key())
            .unwrap()
            .status(),
        Status::Revoked
    );
    assert_eq!(child.account(address(1)).balance(), 75);
    assert_eq!(child.account(address(1)).next_sequence(), 1);
    let grandchild = next(&child, &[transfer(&child, 1, 3, 1, 25).into()]);
    assert_eq!(grandchild.account(address(1)).balance(), 50);
    assert_eq!(grandchild.account(address(3)).balance(), 50);
}

#[test]
fn full_block_indices_survive_sparse_membership_and_snapshot_roundtrip() {
    let limits = LimitsV2 {
        max_operations: 8,
        max_membership_operations: 2,
        max_new_enrollments: 1,
        ..LimitsV2::default()
    };
    let ledger = ledger_with(limits, &[(1, 100), (2, 0)], 1);
    let enrollment = enroll(&ledger, 3, 8);
    let id = enrollment.enrollment_id().unwrap();
    let operations = [
        transfer(&ledger, 1, 1, 0, 1).into(),
        transfer(&ledger, 1, 1, 1, 1).into(),
        transfer(&ledger, 1, 1, 2, 1).into(),
        transfer(&ledger, 1, 1, 3, 1).into(),
        enrollment.into(),
        transfer(&ledger, 1, 1, 4, 1).into(),
        approve(&ledger, 1, 0, id, 8).into(),
    ];
    let child = next(&ledger, &operations);
    let record = child.membership().record(id).unwrap();
    assert_eq!(record.inclusion().operation_index, 4);
    assert_eq!(record.status(), Status::Active);
    let snapshot = UntrustedMembershipSnapshotV2::decode(
        child.config(),
        &child.membership().encode().unwrap(),
    )
    .unwrap();
    snapshot.verify_against(child.membership()).unwrap();
}

#[test]
fn empty_block_expires_membership_without_changing_account_component() {
    let ledger = ledger();
    let enrollment = enroll(&ledger, 3, 1);
    let id = enrollment.enrollment_id().unwrap();
    let child = next(&ledger, &[enrollment.into()]);
    assert_eq!(
        child.membership().record(id).unwrap().status(),
        Status::Pending
    );
    let grandchild = next(&child, &[]);
    assert_eq!(
        grandchild.membership().record(id).unwrap().status(),
        Status::Expired
    );
    assert_eq!(grandchild.membership().pending_count(), 0);
    assert_eq!(
        grandchild.account_bytes().unwrap(),
        child.account_bytes().unwrap()
    );
    assert_eq!(
        grandchild.accounts_root().unwrap(),
        child.accounts_root().unwrap()
    );
    assert_ne!(grandchild.root().unwrap(), child.root().unwrap());
}

#[test]
fn failure_after_expiry_does_not_publish_the_expiry() {
    let ledger = ledger();
    let enrollment = enroll(&ledger, 3, 1);
    let id = enrollment.enrollment_id().unwrap();
    let child = next(&ledger, &[enrollment.into()]);
    assert_unchanged_after(
        &child,
        &[transfer(&child, 1, 2, 0, 101).into()],
        operation_error(
            0,
            OperationError::InsufficientBalance {
                available: 100,
                required: 101,
            },
        ),
    );
    assert_eq!(
        child.membership().record(id).unwrap().status(),
        Status::Pending
    );
}

#[test]
fn late_transfer_failure_discards_earlier_membership_and_accounts() {
    let ledger = ledger();
    let operations = [
        transfer(&ledger, 1, 2, 0, 10).into(),
        enroll(&ledger, 3, 8).into(),
        transfer(&ledger, 1, 2, 1, 100).into(),
    ];
    assert_unchanged_after(
        &ledger,
        &operations,
        operation_error(
            2,
            OperationError::InsufficientBalance {
                available: 90,
                required: 100,
            },
        ),
    );
}

#[test]
fn late_membership_failure_discards_earlier_accounts_and_membership() {
    let ledger = ledger();
    let enrollment = enroll(&ledger, 3, 8);
    let operations = [
        transfer(&ledger, 1, 2, 0, 10).into(),
        enrollment.into(),
        enrollment.into(),
    ];
    assert_unchanged_after(
        &ledger,
        &operations,
        operation_error(2, OperationError::Membership(StateErrorV2::DuplicateId)),
    );
}

#[test]
fn first_failure_is_selected_in_original_mixed_order() {
    let ledger = ledger();
    let invalid_transfer = transfer(&ledger, 1, 2, 0, 101);
    let invalid_member = approve(&ledger, 1, 0, tag(99, 0), 8);
    assert_unchanged_after(
        &ledger,
        &[invalid_transfer.into(), invalid_member.into()],
        operation_error(
            0,
            OperationError::InsufficientBalance {
                available: 100,
                required: 101,
            },
        ),
    );
    assert_unchanged_after(
        &ledger,
        &[invalid_member.into(), invalid_transfer.into()],
        operation_error(0, OperationError::Membership(StateErrorV2::UnknownTarget)),
    );
}

#[test]
fn same_block_activation_does_not_grant_review_rights() {
    let ledger = ledger();
    let first = enroll(&ledger, 3, 8);
    let second = enroll(&ledger, 4, 8);
    let operations = [
        transfer(&ledger, 1, 2, 0, 10).into(),
        first.into(),
        approve(&ledger, 1, 0, first.enrollment_id().unwrap(), 8).into(),
        second.into(),
        approve(&ledger, 3, 0, second.enrollment_id().unwrap(), 8).into(),
    ];
    assert_unchanged_after(
        &ledger,
        &operations,
        operation_error(
            4,
            OperationError::Membership(StateErrorV2::ReviewerNotActive),
        ),
    );
}

#[test]
fn reviewer_withdrawal_is_reflected_before_end_block_activation() {
    let ledger = ledger();
    let enrollment = enroll(&ledger, 3, 8);
    let id = enrollment.enrollment_id().unwrap();
    let operations = [
        enrollment.into(),
        approve(&ledger, 1, 0, id, 8).into(),
        transfer(&ledger, 1, 2, 0, 1).into(),
        withdraw(&ledger, 1, 1).into(),
    ];
    let child = next(&ledger, &operations);
    assert_eq!(
        child.membership().record(id).unwrap().status(),
        Status::Pending
    );
    assert_eq!(child.membership().active_count(), 1);
    assert_eq!(child.account(address(2)).balance(), 1);
}

#[test]
fn wrong_parent_context_height_hash_or_missing_hash_rejects_whole_body() {
    let ledger = ledger();
    let operation = transfer(&ledger, 1, 2, 0, 1);
    let mut views = Vec::new();
    let mut context = View::of(&ledger);
    context.context = tag(99, 1);
    views.push((context, StateErrorV2::WrongParentContext));
    let mut height = View::of(&ledger);
    height.height = 1;
    views.push((height, StateErrorV2::WrongParentHeight));
    let mut hash = View::of(&ledger);
    hash.hashes.insert(0, tag(99, 2));
    views.push((hash, StateErrorV2::WrongParentHash));
    let mut unknown = View::of(&ledger);
    unknown.hashes.clear();
    views.push((unknown, StateErrorV2::UnknownParent));
    let before = ledger.clone();
    for (view, error) in views {
        assert_eq!(
            ledger.stage_operations(&view, &[operation.into()]),
            Err(Error::Membership(error))
        );
        assert_eq!(ledger, before);
    }
}

#[test]
fn operation_lifetime_and_anchor_are_checked_before_execution() {
    let ledger = ledger();
    let good = transfer(&ledger, 1, 2, 0, 1);
    let cases: Vec<(Header, TransferErrorV2)> = vec![
        (
            Header {
                config_hash: tag(99, 1),
                ..*good.header()
            },
            TransferErrorV2::WrongContext,
        ),
        (
            Header {
                valid_from: 2,
                ..*good.header()
            },
            TransferErrorV2::NotYetValid,
        ),
        (
            Header {
                anchor_hash: tag(99, 2),
                ..*good.header()
            },
            TransferErrorV2::WrongAnchorHash,
        ),
    ];
    for (header, expected) in cases {
        let operation = Transfer::sign(header, *good.body(), &key(1)).unwrap();
        assert_unchanged_after(
            &ledger,
            &[operation.into()],
            operation_error(0, OperationError::Transfer(expected)),
        );
    }
    let expires = Transfer::sign(
        Header {
            valid_until: 1,
            ..*good.header()
        },
        *good.body(),
        &key(1),
    )
    .unwrap();
    let child = next(&ledger, &[]);
    assert_unchanged_after(
        &child,
        &[expires.into()],
        operation_error(0, OperationError::Transfer(TransferErrorV2::Expired)),
    );
}

#[test]
fn full_operation_count_preflight_precedes_parent_access() {
    let limits = LimitsV2 {
        max_operations: 2,
        max_membership_operations: 1,
        max_new_enrollments: 1,
        ..LimitsV2::default()
    };
    let ledger = ledger_with(limits, &[(1, 100)], 1);
    let view = View::of(&ledger);
    let operation = transfer(&ledger, 1, 1, 0, 1);
    assert_eq!(
        ledger.stage_operations(&view, &[operation.into(); 3]),
        Err(Error::Limits(BlockErrorV2::TooManyOperations))
    );
    assert_eq!(view.calls.get(), 0);
}

#[test]
fn full_membership_and_enrollment_quota_preflight_precedes_parent_access() {
    for (limits, operations, error) in [
        (
            LimitsV2 {
                max_membership_operations: 1,
                max_new_enrollments: 1,
                ..LimitsV2::default()
            },
            2,
            BlockErrorV2::TooManyMembershipOperations,
        ),
        (
            LimitsV2 {
                max_membership_operations: 3,
                max_new_enrollments: 1,
                ..LimitsV2::default()
            },
            2,
            BlockErrorV2::TooManyEnrollments,
        ),
    ] {
        let ledger = ledger_with(limits, &[(1, 100)], 1);
        let view = View::of(&ledger);
        let enrollment = enroll(&ledger, 3, 8);
        assert_eq!(
            ledger.stage_operations(&view, &vec![enrollment.into(); operations]),
            Err(Error::Limits(error))
        );
        assert_eq!(view.calls.get(), 0);
    }
}

#[test]
fn indexed_membership_and_full_block_share_exact_transfer_byte_budget() {
    assert_eq!(TRANSFER_OPERATION_LEN_V2, 243);
    let exact_size = 168 + 4 + TRANSFER_OPERATION_LEN_V2 + 4 + 291;
    for (limit, fits) in [(exact_size, true), (exact_size - 1, false)] {
        let limits = LimitsV2 {
            max_block_bytes: limit as u32,
            max_operation_bytes: 291,
            ..LimitsV2::default()
        };
        let bootstrap = verified(limits, &[(1, 100)], 1);
        let ledger = Ledger::bootstrap(&bootstrap)
            .unwrap()
            .bind_to_validated_block(block_hash(0));
        let registry = MembershipRegistryV2::bootstrap(&bootstrap)
            .unwrap()
            .bind_to_validated_block(block_hash(0));
        let view = View::of(&ledger);
        let enrollment = enroll(&ledger, 3, 8);
        let operations = [transfer(&ledger, 1, 1, 0, 1).into(), enrollment.into()];
        let plan = [(1, enrollment)];
        if fits {
            assert_eq!(
                validate_operation_limits_v2(ledger.config(), &operations),
                Ok(exact_size)
            );
            let stage = registry
                .begin_indexed_block(&view, 2, &plan)
                .unwrap()
                .apply_next(1)
                .unwrap()
                .finish()
                .unwrap();
            let mixed = ledger.stage_operations(&view, &operations).unwrap();
            assert_eq!(stage.state(), mixed.membership());
        } else {
            assert_eq!(
                validate_operation_limits_v2(ledger.config(), &operations),
                Err(BlockErrorV2::TooLarge)
            );
            assert_eq!(
                ledger.stage_operations(&view, &operations),
                Err(Error::Limits(BlockErrorV2::TooLarge))
            );
            assert!(matches!(
                registry.begin_indexed_block(&view, 2, &plan),
                Err(StateErrorV2::BlockBytesLimit)
            ));
            assert_eq!(view.calls.get(), 0);
        }
    }
}

#[test]
fn same_signed_transfers_require_receive_before_spend() {
    let ledger = ledger();
    let receive = transfer(&ledger, 1, 2, 0, 10);
    let spend = transfer(&ledger, 2, 3, 0, 10);
    let child = next(&ledger, &[receive.into(), spend.into()]);
    assert_eq!(child.account(address(1)).balance(), 90);
    assert_eq!(child.account(address(2)).balance(), 0);
    assert_eq!(child.account(address(2)).next_sequence(), 1);
    assert_eq!(child.account(address(3)).balance(), 10);
    assert_unchanged_after(
        &ledger,
        &[spend.into(), receive.into()],
        operation_error(
            0,
            OperationError::InsufficientBalance {
                available: 0,
                required: 10,
            },
        ),
    );
}
