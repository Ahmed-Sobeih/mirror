//! MB-02B full-body positions and consuming membership staging. All identities,
//! keys, evidence commitments and parent views in these fixtures are synthetic.
use std::cell::Cell;

use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipRegistryV2 as Registry, MembershipStatusV2 as Status, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as Op, StagedMembershipStateV2,
    StateCodecErrorV2, StateErrorV2 as Error, UntrustedMembershipSnapshotV2 as Snapshot,
};

fn hash(n: u8) -> Hash256 {
    Hash256::from_bytes([n; 32])
}

fn key(n: u8) -> Keypair {
    Keypair::from_secret_bytes([n; 32])
}

fn registry(limits: LimitsV2) -> Registry {
    let descriptors: Vec<_> = (1..=2)
        .map(|n| BootstrapDescriptor {
            public_key: key(n).public_key(),
            nonce: hash(20 + n),
            evidence: hash(30 + n),
        })
        .collect();
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(10),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 1,
            evidence_profile: hash(11),
        },
        limits,
        &descriptors,
        &[],
    )
    .unwrap();
    let proofs: Vec<_> = (1..=2)
        .map(|n| config.sign_seed(u32::from(n - 1), &key(n)).unwrap())
        .collect();
    Registry::bootstrap(&config.verify_bootstrap(&proofs).unwrap())
        .unwrap()
        .bind_to_validated_block(hash(90))
}

struct View {
    context: Hash256,
    height: u64,
    tip: Option<Hash256>,
    lookups: Cell<usize>,
}

impl View {
    fn of(registry: &Registry) -> Self {
        Self {
            context: registry.state().config().context(),
            height: registry.state().height(),
            tip: Some(registry.tip_hash()),
            lookups: Cell::new(0),
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
        self.lookups.set(self.lookups.get() + 1);
        (height == self.height).then_some(self.tip).flatten()
    }
}

fn signed(r: &Registry, actor: u8, sequence: u64, until: u64, action: Action) -> Op {
    Op::sign(
        Header {
            config_hash: r.state().config().context(),
            anchor_height: r.state().height(),
            anchor_hash: r.tip_hash(),
            valid_from: r.state().height() + 1,
            valid_until: until,
            actor_public_key: key(actor).public_key(),
            actor_sequence: sequence,
        },
        action,
        &key(actor),
    )
    .unwrap()
}

fn enroll(r: &Registry, actor: u8, until: u64) -> Op {
    signed(
        r,
        actor,
        0,
        until,
        Action::Enroll {
            applicant_nonce: hash(40 + actor),
            evidence_commitment: hash(50 + actor),
            evidence_profile: r.state().config().parameters().evidence_profile,
        },
    )
}

fn approve(r: &Registry, actor: u8, sequence: u64, target: Hash256, until: u64) -> Op {
    signed(
        r,
        actor,
        sequence,
        until,
        Action::Approve {
            target_enrollment_id: target,
        },
    )
}

fn stage(r: &Registry, total: u32, operations: &[(u32, Op)]) -> StagedMembershipStateV2 {
    let view = View::of(r);
    let mut stage = r.begin_indexed_block(&view, total, operations).unwrap();
    for &(index, _) in operations {
        stage = stage.apply_next(index).unwrap();
    }
    stage.finish().unwrap()
}

fn sparse_limits() -> LimitsV2 {
    LimitsV2 {
        max_operations: 8,
        max_membership_operations: 2,
        max_new_enrollments: 2,
        ..LimitsV2::default()
    }
}

#[test]
fn contiguous_indexed_plan_preserves_legacy_state_bytes_roots_and_activation() {
    let r = registry(LimitsV2::default());
    let request = enroll(&r, 3, 8);
    let operations = [
        request,
        approve(&r, 1, 0, request.enrollment_id().unwrap(), 8),
    ];
    let legacy = r.stage_block(&View::of(&r), &operations).unwrap();
    let indexed = stage(&r, 2, &[(0, operations[0]), (1, operations[1])]);
    assert_eq!(legacy, indexed);
    assert_eq!(
        legacy.state().encode().unwrap(),
        indexed.state().encode().unwrap()
    );
    assert_eq!(
        legacy.state().root().unwrap(),
        indexed.state().root().unwrap()
    );
    assert_eq!(indexed.state().active_count(), 3);
}

#[test]
fn full_block_position_above_membership_cap_is_preserved_and_cache_round_trips() {
    let r = registry(sparse_limits());
    let request = enroll(&r, 3, 8);
    let staged = stage(&r, 8, &[(6, request)]);
    let state = staged.state();
    let record = state.record(request.enrollment_id().unwrap()).unwrap();
    assert_eq!(record.inclusion().operation_index, 6);
    assert_eq!(record.inclusion().height, 1);
    let bytes = state.encode().unwrap();
    let cache = Snapshot::decode(state.config(), &bytes).unwrap();
    assert_eq!(cache.encode().unwrap(), bytes);
    assert_eq!(cache.verify_against(state), Ok(()));
    assert_eq!(cache.root().unwrap(), state.root().unwrap());
}

// Locate the enrollment index in this one-enrollment fixture using the frozen
// MPS2 record prefix and exact embedded signed enrollment, not Rust memory layout.
fn enrollment_index_offset(bytes: &[u8], request: &Op) -> usize {
    let mut prefix = request.enrollment_id().unwrap().as_bytes().to_vec();
    prefix.push(1); // Enrollment origin.
    prefix.extend_from_slice(&request.encode());
    let matches: Vec<_> = bytes
        .windows(prefix.len())
        .enumerate()
        .filter_map(|(offset, value)| (value == prefix).then_some(offset))
        .collect();
    assert_eq!(matches.len(), 1);
    matches[0] + prefix.len() + 1 + 8 // Status and inclusion height precede index.
}

#[test]
fn plausible_changed_full_index_remains_untrusted_and_out_of_full_range_rejects() {
    let r = registry(sparse_limits());
    let request = enroll(&r, 3, 8);
    let staged = stage(&r, 8, &[(6, request)]);
    let state = staged.state();
    let mut bytes = state.encode().unwrap();
    let index = enrollment_index_offset(&bytes, &request);
    bytes[index..index + 4].copy_from_slice(&7u32.to_le_bytes());
    let changed = Snapshot::decode(state.config(), &bytes).unwrap();
    assert_eq!(
        changed.verify_against(state),
        Err(StateCodecErrorV2::SnapshotMismatch)
    );
    assert_ne!(changed.root().unwrap(), state.root().unwrap());
    bytes[index..index + 4].copy_from_slice(&8u32.to_le_bytes());
    assert_eq!(
        Snapshot::decode(state.config(), &bytes),
        Err(StateCodecErrorV2::Inconsistent("inclusion operation index"))
    );
}

#[test]
fn invalid_full_index_plan_rejects_before_parent_lookup() {
    let r = registry(sparse_limits());
    let request = enroll(&r, 3, 8);
    let cases = [
        (0, vec![(0, request)], Error::MembershipCountExceedsTotal),
        (
            8,
            vec![(8, request)],
            Error::OperationIndexOutOfRange { index: 8 },
        ),
        (
            8,
            vec![(u32::MAX, request)],
            Error::OperationIndexOutOfRange { index: u32::MAX },
        ),
        (
            8,
            vec![(3, request), (3, request)],
            Error::OperationIndicesNotIncreasing {
                previous: 3,
                got: 3,
            },
        ),
        (
            8,
            vec![(4, request), (2, request)],
            Error::OperationIndicesNotIncreasing {
                previous: 4,
                got: 2,
            },
        ),
    ];
    let before = r.clone();
    for (total, operations, error) in cases {
        let view = View::of(&r);
        assert_eq!(
            r.begin_indexed_block(&view, total, &operations).err(),
            Some(error)
        );
        assert_eq!(view.lookups.get(), 0);
    }
    assert_eq!(r, before);
}

#[test]
fn all_operation_and_membership_counts_are_preflighted() {
    let r = registry(sparse_limits());
    let request = enroll(&r, 3, 8);
    let view = View::of(&r);
    assert_eq!(
        r.begin_indexed_block(&view, 9, &[]).err(),
        Some(Error::OperationLimit)
    );
    assert_eq!(
        r.begin_indexed_block(&view, 8, &[(0, request), (1, request), (2, request)])
            .err(),
        Some(Error::MembershipOperationLimit)
    );
    assert_eq!(view.lookups.get(), 0);
}

#[test]
fn all_enrollment_counts_precede_signatures_and_parent_lookups() {
    let r = registry(LimitsV2 {
        max_new_enrollments: 1,
        ..sparse_limits()
    });
    let request = enroll(&r, 3, 8);
    let bad = Op::from_parts(
        *request.header(),
        *request.action(),
        SignatureBytes::from_bytes([0; 64]),
    );
    let view = View::of(&r);
    assert_eq!(
        r.begin_indexed_block(&view, 8, &[(2, bad), (7, bad)]).err(),
        Some(Error::EnrollmentLimit)
    );
    assert_eq!(view.lookups.get(), 0);
}

#[test]
fn omitted_transfers_charge_exact_243_byte_operations_and_four_byte_prefixes() {
    // 168-byte MRB2 overhead, one 291-byte enrollment, two 243-byte transfers.
    const EXACT_BYTES: u32 = 168 + 4 + 291 + 2 * (4 + 243);
    let exact = registry(LimitsV2 {
        max_operation_bytes: 291,
        max_block_bytes: EXACT_BYTES,
        ..sparse_limits()
    });
    let request = enroll(&exact, 3, 8);
    assert_eq!(stage(&exact, 3, &[(1, request)]).state().pending_count(), 1);

    let short = registry(LimitsV2 {
        max_operation_bytes: 291,
        max_block_bytes: EXACT_BYTES - 1,
        ..sparse_limits()
    });
    let request = enroll(&short, 3, 8);
    let view = View::of(&short);
    assert_eq!(
        short.begin_indexed_block(&view, 3, &[(1, request)]).err(),
        Some(Error::BlockBytesLimit)
    );
    assert_eq!(view.lookups.get(), 0);
}

#[test]
fn transfer_only_plan_cannot_bypass_full_block_byte_budget() {
    let r = registry(LimitsV2 {
        max_operation_bytes: 291,
        max_block_bytes: 168 + 2 * (4 + 243),
        ..sparse_limits()
    });
    assert_eq!(stage(&r, 2, &[]).state().height(), 1);
    let view = View::of(&r);
    assert_eq!(
        r.begin_indexed_block(&view, 3, &[]).err(),
        Some(Error::BlockBytesLimit)
    );
    assert_eq!(view.lookups.get(), 0);
}

#[test]
fn finish_requires_all_preflighted_membership_operations() {
    let r = registry(sparse_limits());
    let before = r.clone();
    let view = View::of(&r);
    let operations = [(2, enroll(&r, 3, 8)), (7, enroll(&r, 4, 8))];
    let builder = r.begin_indexed_block(&view, 8, &operations).unwrap();
    assert_eq!(
        builder.finish(),
        Err(Error::IncompleteMembershipStage { remaining: 2 })
    );
    let builder = r.begin_indexed_block(&view, 8, &operations).unwrap();
    assert_eq!(
        builder.apply_next(2).unwrap().finish(),
        Err(Error::IncompleteMembershipStage { remaining: 1 })
    );
    assert_eq!(r, before);
}

#[test]
fn wrong_step_index_cannot_skip_or_reapply_a_planned_operation() {
    let r = registry(sparse_limits());
    let before = r.clone();
    let view = View::of(&r);
    let operations = [(2, enroll(&r, 3, 8)), (7, enroll(&r, 4, 8))];
    let builder = r.begin_indexed_block(&view, 8, &operations).unwrap();
    assert_eq!(
        builder.apply_next(7).err(),
        Some(Error::UnexpectedOperationIndex {
            expected: 2,
            got: 7
        })
    );
    let builder = r
        .begin_indexed_block(&view, 8, &operations)
        .unwrap()
        .apply_next(2)
        .unwrap();
    assert_eq!(
        builder.apply_next(2).err(),
        Some(Error::UnexpectedOperationIndex {
            expected: 7,
            got: 2
        })
    );
    assert_eq!(r, before);
}

#[test]
fn applying_after_plan_exhaustion_is_rejected() {
    let r = registry(sparse_limits());
    let view = View::of(&r);
    let builder = r.begin_indexed_block(&view, 2, &[]).unwrap();
    assert_eq!(
        builder.apply_next(0).err(),
        Some(Error::NoRemainingMembershipOperations)
    );
    let operations = [(1, enroll(&r, 3, 8))];
    let builder = r
        .begin_indexed_block(&view, 2, &operations)
        .unwrap()
        .apply_next(1)
        .unwrap();
    assert_eq!(
        builder.apply_next(1).err(),
        Some(Error::NoRemainingMembershipOperations)
    );
}

#[test]
fn authorization_failure_reports_original_full_block_index_and_preserves_parent() {
    let r = registry(sparse_limits());
    let before = r.clone();
    let request = enroll(&r, 3, 8);
    let bad = Op::from_parts(
        *request.header(),
        *request.action(),
        SignatureBytes::from_bytes([0; 64]),
    );
    let view = View::of(&r);
    let operations = [(6, bad)];
    let builder = r.begin_indexed_block(&view, 8, &operations).unwrap();
    assert!(matches!(
        builder.apply_next(6),
        Err(Error::Operation { index: 6, .. })
    ));
    assert_eq!(r, before);
}

#[test]
fn late_stateful_failure_discards_earlier_candidate_mutation() {
    let r = registry(sparse_limits());
    let before = r.clone();
    let request = enroll(&r, 3, 8);
    let view = View::of(&r);
    let operations = [(1, request), (7, request)];
    let builder = r
        .begin_indexed_block(&view, 8, &operations)
        .unwrap()
        .apply_next(1)
        .unwrap();
    assert_eq!(builder.apply_next(7).err(), Some(Error::DuplicateId));
    assert_eq!(r, before);
    assert_eq!(r.state().record_count(), 2);
}

#[test]
fn activation_follows_original_full_enrollment_positions_at_block_end() {
    let r = registry(LimitsV2::default());
    let e3 = enroll(&r, 3, 8);
    let e4 = enroll(&r, 4, 8);
    let id3 = e3.enrollment_id().unwrap();
    let id4 = e4.enrollment_id().unwrap();
    let staged = stage(
        &r,
        10,
        &[
            (1, e4),
            (4, e3),
            (6, approve(&r, 1, 0, id3, 8)),
            (8, approve(&r, 1, 1, id4, 8)),
        ],
    );
    let state = staged.state();
    assert_eq!(state.record(id4).unwrap().inclusion().operation_index, 1);
    assert_eq!(state.record(id3).unwrap().inclusion().operation_index, 4);
    assert_eq!(state.record(id4).unwrap().activation().unwrap().ordinal, 3);
    assert_eq!(state.record(id3).unwrap().activation().unwrap().ordinal, 4);
    assert_eq!(state.active_count(), 4);
    assert_eq!(state.pending_count(), 0);
}

#[test]
fn same_block_activation_cannot_authorize_a_later_membership_operation() {
    let r = registry(LimitsV2::default());
    let before = r.clone();
    let e3 = enroll(&r, 3, 8);
    let e4 = enroll(&r, 4, 8);
    let operations = [
        (1, e3),
        (3, approve(&r, 1, 0, e3.enrollment_id().unwrap(), 8)),
        (5, e4),
        (7, approve(&r, 3, 1, e4.enrollment_id().unwrap(), 8)),
    ];
    let view = View::of(&r);
    let mut builder = r.begin_indexed_block(&view, 8, &operations).unwrap();
    for index in [1, 3, 5] {
        builder = builder.apply_next(index).unwrap();
    }
    assert_eq!(builder.apply_next(7).err(), Some(Error::ReviewerNotActive));
    assert_eq!(r, before);
}

#[test]
fn later_withdrawal_prevents_reviewers_approval_from_activating_new_member() {
    let r = registry(LimitsV2::default());
    let request = enroll(&r, 3, 8);
    let target = request.enrollment_id().unwrap();
    let reviewer = r.state().record_by_key(key(1).public_key()).unwrap().id();
    let withdraw = signed(
        &r,
        1,
        1,
        8,
        Action::Withdraw {
            own_member_id: reviewer,
        },
    );
    let staged = stage(
        &r,
        8,
        &[
            (1, request),
            (3, approve(&r, 1, 0, target, 8)),
            (6, withdraw),
        ],
    );
    assert_eq!(
        staged.state().record(target).unwrap().status(),
        Status::Pending
    );
    assert_eq!(
        staged.state().record(reviewer).unwrap().status(),
        Status::Revoked
    );
    assert_eq!(staged.state().active_count(), 1);
}

#[test]
fn transfer_only_membership_stage_still_advances_expiry() {
    let r = registry(sparse_limits());
    let request = enroll(&r, 3, 1);
    let target = request.enrollment_id().unwrap();
    let r = stage(&r, 8, &[(6, request)]).bind_to_validated_block(hash(91));
    assert_eq!(r.state().record(target).unwrap().status(), Status::Pending);
    let next = stage(&r, 3, &[]);
    assert_eq!(next.state().height(), 2);
    assert_eq!(
        next.state().record(target).unwrap().status(),
        Status::Expired
    );
    assert_eq!(next.state().pending_count(), 0);
    assert_eq!(r.state().record(target).unwrap().status(), Status::Pending);
}

#[test]
fn indexed_begin_preserves_parent_binding_checks_even_without_membership_operations() {
    let r = registry(sparse_limits());
    let mut view = View::of(&r);
    view.context = hash(99);
    assert_eq!(
        r.begin_indexed_block(&view, 2, &[]).err(),
        Some(Error::WrongParentContext)
    );
    view = View::of(&r);
    view.height = 1;
    assert_eq!(
        r.begin_indexed_block(&view, 2, &[]).err(),
        Some(Error::WrongParentHeight)
    );
    view = View::of(&r);
    view.tip = None;
    assert_eq!(
        r.begin_indexed_block(&view, 2, &[]).err(),
        Some(Error::UnknownParent)
    );
    view = View::of(&r);
    view.tip = Some(hash(99));
    assert_eq!(
        r.begin_indexed_block(&view, 2, &[]).err(),
        Some(Error::WrongParentHash)
    );
}
