//! B3 transition tests with public synthetic keys and a trusted in-memory view.
//! These exercise registry execution, not biological matching or live consensus.
use std::{cell::Cell, collections::BTreeMap};

use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipRegistryV2 as Registry, MembershipStatusV2 as Status, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as Op, StateErrorV2,
};

fn tag(domain: u8, number: u64) -> Hash256 {
    let mut bytes = [0; 32];
    bytes[0] = domain;
    bytes[1..9].copy_from_slice(&number.to_le_bytes());
    Hash256::from_bytes(bytes)
}

fn key(number: u64) -> Keypair {
    let mut bytes = [42; 32];
    bytes[..8].copy_from_slice(&number.to_le_bytes());
    Keypair::from_secret_bytes(bytes)
}

fn block_hash(height: u64) -> Hash256 {
    tag(10, height)
}

fn registry_with(seeds: u16, threshold: u16, limits: LimitsV2) -> Registry {
    let descriptors: Vec<_> = (1..=u64::from(seeds))
        .map(|number| BootstrapDescriptor {
            public_key: key(number).public_key(),
            nonce: tag(1, number),
            evidence: tag(2, number),
        })
        .collect();
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
        &[],
    )
    .unwrap();
    let proofs: Vec<_> = (1..=u64::from(seeds))
        .map(|number| config.sign_seed((number - 1) as u32, &key(number)).unwrap())
        .collect();
    Registry::bootstrap(&config.verify_bootstrap(&proofs).unwrap())
        .unwrap()
        .bind_to_validated_block(block_hash(0))
}

fn registry(seeds: u16, threshold: u16) -> Registry {
    registry_with(seeds, threshold, LimitsV2::default())
}

struct View {
    context: Hash256,
    height: u64,
    hashes: BTreeMap<u64, Hash256>,
    lookups: Cell<usize>,
}

impl View {
    fn of(registry: &Registry) -> Self {
        let height = registry.state().height();
        let mut hashes: BTreeMap<_, _> = (0..=height).map(|h| (h, block_hash(h))).collect();
        hashes.insert(height, registry.tip_hash());
        Self {
            context: registry.state().config().context(),
            height,
            hashes,
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
        self.hashes.get(&height).copied()
    }
}

fn header(registry: &Registry, actor: u64, sequence: u64, until: u64) -> Header {
    Header {
        config_hash: registry.state().config().context(),
        anchor_height: registry.state().height(),
        anchor_hash: registry.tip_hash(),
        valid_from: registry.state().height() + 1,
        valid_until: until,
        actor_public_key: key(actor).public_key(),
        actor_sequence: sequence,
    }
}

fn signed(registry: &Registry, actor: u64, sequence: u64, until: u64, action: Action) -> Op {
    Op::sign(
        header(registry, actor, sequence, until),
        action,
        &key(actor),
    )
    .unwrap()
}

fn enroll(registry: &Registry, actor: u64, until: u64) -> Op {
    signed(
        registry,
        actor,
        0,
        until,
        Action::Enroll {
            applicant_nonce: tag(5, actor),
            evidence_commitment: tag(6, actor),
            evidence_profile: registry.state().config().parameters().evidence_profile,
        },
    )
}

fn approve(registry: &Registry, actor: u64, sequence: u64, target: Hash256, until: u64) -> Op {
    signed(
        registry,
        actor,
        sequence,
        until,
        Action::Approve {
            target_enrollment_id: target,
        },
    )
}

fn withdraw(registry: &Registry, actor: u64, sequence: u64, target: Hash256) -> Op {
    signed(
        registry,
        actor,
        sequence,
        registry.state().height() + 8,
        Action::Withdraw {
            own_member_id: target,
        },
    )
}

fn member(registry: &Registry, actor: u64) -> Hash256 {
    registry
        .state()
        .record_by_key(key(actor).public_key())
        .unwrap()
        .id()
}

fn status(registry: &Registry, id: Hash256) -> Status {
    registry.state().record(id).unwrap().status()
}

fn next(registry: &Registry, operations: &[Op]) -> Registry {
    registry
        .stage_block(&View::of(registry), operations)
        .unwrap()
        .bind_to_validated_block(block_hash(registry.state().height() + 1))
}

fn rejects(registry: &Registry, operations: &[Op]) {
    let before = registry.clone();
    assert!(
        registry
            .stage_block(&View::of(registry), operations)
            .is_err()
    );
    assert_eq!(registry, &before, "a failed block changed its parent");
}

#[test]
fn bootstrap_has_ordinary_active_members_with_ordered_ordinals_and_no_money() {
    let r = registry(3, 2);
    assert_eq!(r.state().height(), 0);
    assert_eq!(r.state().record_count(), 3);
    assert_eq!(r.state().active_count(), 3);
    assert_eq!(r.state().pending_count(), 0);
    assert_eq!(r.state().active_high_water(), 3);
    assert_eq!(r.state().total_activations(), 3);
    assert_eq!(r.state().retained_approval_count(), 0);
    assert_eq!(r.state().config().initial_test_supply(), 0);
    for actor in 1..=3 {
        let record = r.state().record_by_key(key(actor).public_key()).unwrap();
        assert_eq!(record.status(), Status::Active);
        assert_eq!(record.next_sequence(), 0);
        assert_eq!(record.activation().unwrap().height, 0);
        assert_eq!(record.activation().unwrap().ordinal, actor);
        assert_eq!(record.inclusion().height, 0);
        assert_eq!(record.inclusion().operation_index, actor as u32 - 1);
        assert_eq!(record.approval_count(), 0);
    }
}

#[test]
fn enrollment_creates_pending_record_and_consumes_sequence_zero() {
    let r = registry(2, 2);
    let request = enroll(&r, 101, 8);
    let id = request.enrollment_id().unwrap();
    let n = next(&r, &[request]);
    assert_eq!(status(&n, id), Status::Pending);
    assert_eq!(n.state().record(id).unwrap().next_sequence(), 1);
    assert_eq!(n.state().record(id).unwrap().activation(), None);
    assert_eq!(n.state().record(id).unwrap().inclusion().height, 1);
    assert_eq!(n.state().record(id).unwrap().inclusion().operation_index, 0);
    assert_eq!(n.state().active_count(), 2);
    assert_eq!(n.state().pending_count(), 1);
    assert_eq!(n.state().total_activations(), 2);
    assert_eq!(
        r.state().record_count(),
        2,
        "successful staging also preserves parent"
    );
}

#[test]
fn distinct_approvals_activate_once_and_consume_each_reviewers_sequence() {
    let r = registry(2, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let pending = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    assert_eq!(status(&pending, id), Status::Pending);
    let active = next(&pending, &[approve(&pending, 2, 0, id, 8)]);
    assert_eq!(status(&active, id), Status::Active);
    assert_eq!(
        active
            .state()
            .record(id)
            .unwrap()
            .activation()
            .unwrap()
            .ordinal,
        3
    );
    assert_eq!(
        active
            .state()
            .record(id)
            .unwrap()
            .activation()
            .unwrap()
            .height,
        2
    );
    assert_eq!(active.state().pending_count(), 0);
    assert_eq!(active.state().active_count(), 3);
    assert_eq!(active.state().active_high_water(), 3);
    for reviewer in [1, 2] {
        assert_eq!(
            active
                .state()
                .record(member(&active, reviewer))
                .unwrap()
                .next_sequence(),
            1
        );
    }
    assert_eq!(active.state().record(id).unwrap().approval_count(), 2);
    assert_eq!(active.state().retained_approval_count(), 2);
}

#[test]
fn enrollment_then_approval_in_same_block_is_valid_but_reverse_order_is_not() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let a = approve(&r, 1, 0, id, 8);
    rejects(&r, &[a, e]);
    let n = next(&r, &[e, a]);
    assert_eq!(status(&n, id), Status::Active);
    assert_eq!(
        n.state().record(id).unwrap().activation().unwrap().height,
        1
    );
}

#[test]
fn first_seed_has_no_threshold_or_forced_withdrawal_bypass() {
    let r = registry(2, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    assert_eq!(status(&n, id), Status::Pending);
    rejects(&n, &[withdraw(&n, 1, 1, id)]);
    rejects(&n, &[withdraw(&n, 1, 1, member(&n, 2))]);
    assert_eq!(status(&n, member(&n, 1)), Status::Active);
}

#[test]
fn newly_activated_member_cannot_approve_until_next_block() {
    let r = registry(1, 1);
    let a = enroll(&r, 101, 8);
    let b = enroll(&r, 102, 8);
    let aid = a.enrollment_id().unwrap();
    let bid = b.enrollment_id().unwrap();
    let approve_a = approve(&r, 1, 0, aid, 8);
    rejects(&r, &[a, b, approve_a, approve(&r, 101, 1, bid, 8)]);
    let first = next(&r, &[a, b, approve_a]);
    let second = next(&first, &[approve(&first, 101, 1, bid, 8)]);
    assert_eq!(status(&second, bid), Status::Active);
    assert_eq!(second.state().record(aid).unwrap().next_sequence(), 2);
}

#[test]
fn skipped_reused_and_maximum_claimed_sequences_are_rejected() {
    let r = registry(2, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let second_request = enroll(&r, 102, 8);
    let second_id = second_request.enrollment_id().unwrap();
    let pending = next(&r, &[e, second_request]);
    for sequence in [1, 2, u64::MAX] {
        let bad = approve(&pending, 1, sequence, id, 8);
        assert!(
            bad.validate_for_parent(pending.state().config(), &View::of(&pending))
                .is_ok()
        );
        rejects(&pending, &[bad]);
    }
    let accepted = next(&pending, &[approve(&pending, 1, 0, id, 8)]);
    // Use a different target so duplicate-review rejection cannot mask a
    // missing stale-sequence check.
    let stale = approve(&accepted, 1, 0, second_id, 8);
    assert_eq!(
        accepted.stage_block(&View::of(&accepted), &[stale]),
        Err(StateErrorV2::SequenceMismatch {
            expected: 1,
            got: 0
        })
    );
    rejects(&accepted, &[stale]);
    assert_eq!(
        accepted
            .state()
            .record(member(&accepted, 1))
            .unwrap()
            .next_sequence(),
        1
    );
    let retried = next(&accepted, &[approve(&accepted, 1, 1, second_id, 8)]);
    assert_eq!(
        retried.state().record(second_id).unwrap().approval_count(),
        1
    );
    assert_eq!(
        retried
            .state()
            .record(member(&retried, 1))
            .unwrap()
            .next_sequence(),
        2
    );
}

#[test]
fn ordered_operations_require_and_consume_consecutive_sequences() {
    let r = registry(2, 2);
    let a = enroll(&r, 101, 8);
    let b = enroll(&r, 102, 8);
    let aid = a.enrollment_id().unwrap();
    let bid = b.enrollment_id().unwrap();
    let first = approve(&r, 1, 0, aid, 8);
    let second = approve(&r, 1, 1, bid, 8);
    rejects(&r, &[a, b, second, first]);
    let n = next(&r, &[a, b, first, second]);
    assert_eq!(n.state().record(member(&n, 1)).unwrap().next_sequence(), 2);
    assert_eq!(n.state().record(aid).unwrap().approval_count(), 1);
    assert_eq!(n.state().record(bid).unwrap().approval_count(), 1);
}

#[test]
fn repeated_reviewer_cannot_add_weight_even_with_a_new_sequence() {
    let r = registry(2, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    rejects(&n, &[approve(&n, 1, 1, id, 8)]);
    assert_eq!(status(&n, id), Status::Pending);
    assert_eq!(n.state().record(id).unwrap().approval_count(), 1);
    assert_eq!(n.state().record(member(&n, 1)).unwrap().next_sequence(), 1);
}

#[test]
fn context_valid_unknown_target_is_not_state_authorization() {
    let r = registry(1, 1);
    let a = approve(&r, 1, 0, tag(20, 900), 8);
    assert!(
        a.validate_for_parent(r.state().config(), &View::of(&r))
            .is_ok()
    );
    rejects(&r, &[a]);
    rejects(&r, &[withdraw(&r, 1, 0, tag(20, 901))]);
}

#[test]
fn unknown_pending_and_self_reviewers_are_ineligible() {
    let r = registry(2, 2);
    let a = enroll(&r, 101, 8);
    let b = enroll(&r, 102, 8);
    let aid = a.enrollment_id().unwrap();
    let bid = b.enrollment_id().unwrap();
    let n = next(&r, &[a, b]);
    rejects(&n, &[approve(&n, 999, 0, aid, 8)]);
    rejects(&n, &[approve(&n, 101, 1, bid, 8)]);
    rejects(&n, &[approve(&n, 101, 1, aid, 8)]);
}

#[test]
fn pending_owner_can_withdraw_but_cannot_repeat_or_reregister() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(&r, &[e, withdraw(&r, 101, 1, id)]);
    assert_eq!(status(&n, id), Status::Revoked);
    assert_eq!(n.state().record(id).unwrap().next_sequence(), 2);
    assert_eq!(n.state().record(id).unwrap().activation(), None);
    assert_eq!(n.state().pending_count(), 0);
    rejects(&n, &[withdraw(&n, 101, 2, id)]);
    rejects(&n, &[enroll(&n, 101, 8)]);
    rejects(&n, &[approve(&n, 1, 0, id, 8)]);
}

#[test]
fn active_withdrawal_preserves_original_activation_and_high_water() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let active = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    let original = active.state().record(id).unwrap().activation();
    let n = next(&active, &[withdraw(&active, 101, 1, id)]);
    assert_eq!(status(&n, id), Status::Revoked);
    assert_eq!(n.state().record(id).unwrap().activation(), original);
    assert_eq!(n.state().active_count(), 1);
    assert_eq!(n.state().active_high_water(), 2);
    assert_eq!(n.state().total_activations(), 2);
    assert_eq!(n.state().record(id).unwrap().approval_count(), 1);
    rejects(&n, &[withdraw(&n, 101, 2, id)]);
}

#[test]
fn another_signature_cannot_withdraw_an_existing_membership() {
    let r = registry(2, 1);
    let target = member(&r, 2);
    rejects(&r, &[withdraw(&r, 1, 0, target)]);
    rejects(&r, &[withdraw(&r, 999, 0, target)]);
    assert_eq!(
        next(&r, &[withdraw(&r, 2, 0, target)])
            .state()
            .active_count(),
        1
    );
}

#[test]
fn active_target_rejects_later_approvals_without_consuming_sequence() {
    let r = registry(2, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    rejects(&n, &[approve(&n, 2, 0, id, 8)]);
    assert_eq!(n.state().record(member(&n, 2)).unwrap().next_sequence(), 0);
    assert_eq!(n.state().total_activations(), 3);
}

#[test]
fn approval_must_share_request_expiry_and_cannot_extend_it() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(&r, &[e]);
    for until in [7, 9] {
        let a = approve(&n, 1, 0, id, until);
        assert!(
            a.validate_for_parent(n.state().config(), &View::of(&n))
                .is_ok()
        );
        rejects(&n, &[a]);
    }
    assert_eq!(
        status(&next(&n, &[approve(&n, 1, 0, id, 8)]), id),
        Status::Active
    );
}

#[test]
fn approval_window_cannot_start_before_enrollment_window() {
    let r = next(&registry(1, 1), &[]);
    let e = enroll(&r, 101, 8); // request starts at height 2
    let id = e.enrollment_id().unwrap();
    let n = next(&r, &[e]);
    let mut h = header(&n, 1, 0, 8);
    h.anchor_height = 0;
    h.anchor_hash = block_hash(0);
    h.valid_from = 1;
    let a = Op::sign(
        h,
        Action::Approve {
            target_enrollment_id: id,
        },
        &key(1),
    )
    .unwrap();
    assert!(
        a.validate_for_parent(n.state().config(), &View::of(&n))
            .is_ok()
    );
    rejects(&n, &[a]);
}

#[test]
fn empty_blocks_expire_pending_only_after_inclusive_final_height() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 2);
    let id = e.enrollment_id().unwrap();
    let at_one = next(&r, &[e]);
    let at_two = next(&at_one, &[]);
    assert_eq!(status(&at_two, id), Status::Pending);
    let at_three = next(&at_two, &[]);
    assert_eq!(status(&at_three, id), Status::Expired);
    assert_eq!(at_three.state().pending_count(), 0);
    assert_eq!(at_three.state().record_count(), 2);
    assert_eq!(at_three.state().record(id).unwrap().next_sequence(), 1);
    rejects(&at_three, &[withdraw(&at_three, 101, 1, id)]);
}

#[test]
fn active_members_do_not_expire_with_their_original_requests() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 1);
    let id = e.enrollment_id().unwrap();
    let active = next(&r, &[e, approve(&r, 1, 0, id, 1)]);
    let n = next(&active, &[]);
    assert_eq!(status(&n, id), Status::Active);
    assert_eq!(n.state().active_count(), 2);
    assert_eq!(n.state().pending_count(), 0);
}

#[test]
fn expired_pending_frees_quota_without_freeing_key_or_evidence() {
    let r = registry_with(
        1,
        1,
        LimitsV2 {
            max_pending: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
    );
    let e = enroll(&r, 101, 1);
    let id = e.enrollment_id().unwrap();
    let pending = next(&r, &[e]);
    let n = next(&pending, &[enroll(&pending, 102, 8)]);
    assert_eq!(status(&n, id), Status::Expired);
    assert_eq!(n.state().pending_count(), 1);
    assert_eq!(n.state().record_count(), 3);
    rejects(&pending, &[enroll(&pending, 101, 8)]);
    let reused = signed(
        &pending,
        103,
        0,
        8,
        Action::Enroll {
            applicant_nonce: tag(5, 103),
            evidence_commitment: tag(6, 101),
            evidence_profile: pending.state().config().parameters().evidence_profile,
        },
    );
    rejects(&pending, &[reused]);
}

#[test]
fn request_key_and_evidence_reuse_rejects_after_each_lifecycle_outcome() {
    for terminal in [
        Status::Pending,
        Status::Active,
        Status::Revoked,
        Status::Expired,
    ] {
        let r = registry(1, 1);
        let until = if terminal == Status::Expired { 1 } else { 8 };
        let e = enroll(&r, 101, until);
        let id = e.enrollment_id().unwrap();
        let mut operations = vec![e];
        if terminal == Status::Active {
            operations.push(approve(&r, 1, 0, id, until));
        }
        if terminal == Status::Revoked {
            operations.push(withdraw(&r, 101, 1, id));
        }
        let mut n = next(&r, &operations);
        if terminal == Status::Expired {
            n = next(&n, &[]);
        }
        assert_eq!(status(&n, id), terminal);
        rejects(&n, &[e]);
        let changed_evidence = signed(
            &n,
            101,
            0,
            8,
            Action::Enroll {
                applicant_nonce: tag(5, 999),
                evidence_commitment: tag(6, 999),
                evidence_profile: n.state().config().parameters().evidence_profile,
            },
        );
        rejects(&n, &[changed_evidence]);
        let changed_key = signed(
            &n,
            102,
            0,
            8,
            Action::Enroll {
                applicant_nonce: tag(5, 102),
                evidence_commitment: tag(6, 101),
                evidence_profile: n.state().config().parameters().evidence_profile,
            },
        );
        rejects(&n, &[changed_key]);
    }
}

#[test]
fn bootstrap_keys_and_evidence_are_also_historical_reservations() {
    let r = registry(1, 1);
    rejects(&r, &[enroll(&r, 1, 8)]);
    let reused = signed(
        &r,
        101,
        0,
        8,
        Action::Enroll {
            applicant_nonce: tag(5, 101),
            evidence_commitment: tag(2, 1),
            evidence_profile: r.state().config().parameters().evidence_profile,
        },
    );
    rejects(&r, &[reused]);
    let withdrawn = next(&r, &[withdraw(&r, 1, 0, member(&r, 1))]);
    rejects(&withdrawn, &[enroll(&withdrawn, 1, 8)]);
}

#[test]
fn runtime_nonce_alone_is_not_a_global_identity_fingerprint() {
    let r = registry(1, 1);
    let a = enroll(&r, 101, 8);
    let b = signed(
        &r,
        102,
        0,
        8,
        Action::Enroll {
            applicant_nonce: tag(5, 101),
            evidence_commitment: tag(6, 102),
            evidence_profile: r.state().config().parameters().evidence_profile,
        },
    );
    assert_ne!(a.enrollment_id(), b.enrollment_id());
    assert_eq!(next(&r, &[a, b]).state().pending_count(), 2);
}

#[test]
fn same_block_reviewer_withdrawal_removes_earlier_approval_weight() {
    let r = registry(2, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(
        &r,
        &[
            e,
            approve(&r, 1, 0, id, 8),
            approve(&r, 2, 0, id, 8),
            withdraw(&r, 1, 1, member(&r, 1)),
        ],
    );
    assert_eq!(status(&n, id), Status::Pending);
    assert_eq!(n.state().record(id).unwrap().approval_count(), 2);
    assert_eq!(n.state().total_activations(), 2);
    assert_eq!(n.state().active_count(), 1);
    assert_eq!(n.state().active_high_water(), 2);
}

#[test]
fn approval_after_reviewer_withdrawal_rejects_the_whole_block() {
    let r = registry(2, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    rejects(
        &r,
        &[
            e,
            withdraw(&r, 1, 0, member(&r, 1)),
            approve(&r, 1, 1, id, 8),
        ],
    );
    assert_eq!(status(&r, member(&r, 1)), Status::Active);
    assert_eq!(r.state().record_count(), 2);
}

#[test]
fn earlier_block_approval_loses_weight_when_reviewer_withdraws() {
    let r = registry(3, 2);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let first = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    let second = next(
        &first,
        &[
            approve(&first, 2, 0, id, 8),
            withdraw(&first, 1, 1, member(&first, 1)),
        ],
    );
    assert_eq!(status(&second, id), Status::Pending);
    let third = next(&second, &[approve(&second, 3, 0, id, 8)]);
    assert_eq!(status(&third, id), Status::Active);
    assert_eq!(
        third
            .state()
            .record(id)
            .unwrap()
            .activation()
            .unwrap()
            .height,
        3
    );
}

#[test]
fn reviewer_withdrawal_never_cascade_revokes_already_active_members() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let active = next(&r, &[e, approve(&r, 1, 0, id, 8)]);
    let n = next(&active, &[withdraw(&active, 1, 1, member(&active, 1))]);
    assert_eq!(status(&n, id), Status::Active);
    assert_eq!(n.state().active_count(), 1);
    assert_eq!(n.state().total_activations(), 2);
}

#[test]
fn simultaneous_activations_follow_original_inclusion_not_approval_order() {
    let r = registry(1, 1);
    let a = enroll(&r, 101, 8);
    let aid = a.enrollment_id().unwrap();
    let first = next(&r, &[a]);
    let b = enroll(&first, 102, 8);
    let bid = b.enrollment_id().unwrap();
    let n = next(
        &first,
        &[
            b,
            approve(&first, 1, 0, bid, 8),
            approve(&first, 1, 1, aid, 8),
        ],
    );
    assert_eq!(
        n.state().record(aid).unwrap().activation().unwrap().ordinal,
        2
    );
    assert_eq!(
        n.state().record(bid).unwrap().activation().unwrap().ordinal,
        3
    );
    assert_eq!(
        n.state().record(aid).unwrap().activation().unwrap().height,
        2
    );
    assert_eq!(
        n.state().record(bid).unwrap().activation().unwrap().height,
        2
    );
}

#[test]
fn same_height_activation_order_is_operation_index_not_sorted_member_id() {
    let r = registry(1, 1);
    let a = enroll(&r, 101, 8);
    let b = enroll(&r, 102, 8);
    let (first, second) =
        if a.enrollment_id().unwrap().as_bytes() > b.enrollment_id().unwrap().as_bytes() {
            (a, b)
        } else {
            (b, a)
        };
    let first_id = first.enrollment_id().unwrap();
    let second_id = second.enrollment_id().unwrap();
    let n = next(
        &r,
        &[
            first,
            second,
            approve(&r, 1, 0, second_id, 8),
            approve(&r, 1, 1, first_id, 8),
        ],
    );
    assert_eq!(
        n.state()
            .record(first_id)
            .unwrap()
            .activation()
            .unwrap()
            .ordinal,
        2
    );
    assert_eq!(
        n.state()
            .record(second_id)
            .unwrap()
            .activation()
            .unwrap()
            .ordinal,
        3
    );
}

#[test]
fn high_water_uses_final_population_not_a_transient_early_activation() {
    let r = registry(2, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let n = next(
        &r,
        &[
            e,
            approve(&r, 2, 0, id, 8),
            withdraw(&r, 1, 0, member(&r, 1)),
        ],
    );
    assert_eq!(status(&n, id), Status::Active);
    assert_eq!(n.state().active_count(), 2);
    assert_eq!(n.state().active_high_water(), 2);
    assert_eq!(n.state().total_activations(), 3);
}

#[test]
fn invalid_last_operation_rolls_back_expiry_indexes_sequences_and_withdrawal() {
    let r = registry(2, 1);
    let expiring = enroll(&r, 101, 1);
    let expired_id = expiring.enrollment_id().unwrap();
    let parent = next(&r, &[expiring]);
    let fresh = enroll(&parent, 102, 8);
    let fresh_id = fresh.enrollment_id().unwrap();
    let prefix = [
        fresh,
        approve(&parent, 1, 0, fresh_id, 8),
        withdraw(&parent, 2, 0, member(&parent, 2)),
    ];
    let bad = approve(&parent, 999, 0, fresh_id, 8);
    let mut full = prefix.to_vec();
    full.push(bad);
    rejects(&parent, &full);
    assert_eq!(status(&parent, expired_id), Status::Pending);
    assert_eq!(parent.state().record(fresh_id), None);
    let retried = next(&parent, &prefix);
    assert_eq!(status(&retried, expired_id), Status::Expired);
    assert_eq!(status(&retried, fresh_id), Status::Active);
    assert_eq!(status(&retried, member(&retried, 2)), Status::Revoked);
    assert_eq!(retried.state().active_count(), 2);
    assert_eq!(retried.state().active_high_water(), 2);
}

#[test]
fn malformed_signature_cannot_leave_an_earlier_enrollment_or_consume_sequence() {
    let r = registry(1, 1);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let good = approve(&r, 1, 0, id, 8);
    let bad = Op::from_parts(
        *good.header(),
        *good.action(),
        SignatureBytes::from_bytes([0; 64]),
    );
    rejects(&r, &[e, bad]);
    assert_eq!(status(&next(&r, &[e, good]), id), Status::Active);
}

#[test]
fn independently_constructed_states_produce_identical_ordered_transitions() {
    let one = registry(2, 2);
    let two = registry(2, 2);
    let e = enroll(&one, 101, 8);
    let id = e.enrollment_id().unwrap();
    let ops = [e, approve(&one, 2, 0, id, 8), approve(&one, 1, 0, id, 8)];
    let a = next(&one, &ops);
    let b = next(&two, &ops);
    assert_eq!(a, b);
    let withdraw_op = withdraw(&a, 101, 1, id);
    assert_eq!(next(&a, &[withdraw_op]), next(&b, &[withdraw_op]));
}

#[test]
fn parent_context_height_and_exact_tip_must_match_even_for_empty_blocks() {
    let r = registry(1, 1);
    let before = r.clone();
    let mut view = View::of(&r);
    view.context = tag(30, 0);
    assert!(r.stage_block(&view, &[]).is_err());
    let mut view = View::of(&r);
    view.height = 1;
    assert!(r.stage_block(&view, &[]).is_err());
    let mut view = View::of(&r);
    view.hashes.insert(0, tag(30, 1));
    assert!(r.stage_block(&view, &[]).is_err());
    let mut view = View::of(&r);
    view.hashes.clear();
    assert!(r.stage_block(&view, &[]).is_err());
    assert_eq!(r, before);
}

#[test]
fn matching_tip_does_not_allow_a_wrong_earlier_operation_anchor() {
    let r = registry(1, 1);
    let old_request = enroll(&r, 101, 8);
    let parent = next(&r, &[]);
    let mut view = View::of(&parent);
    view.hashes.insert(0, tag(30, 2));
    assert!(parent.stage_block(&view, &[old_request]).is_err());
    assert_eq!(next(&parent, &[old_request]).state().pending_count(), 1);
}

#[test]
fn operation_count_cap_is_checked_before_ancestor_lookup_or_signature_work() {
    let r = registry_with(
        1,
        1,
        LimitsV2 {
            max_operations: 2,
            max_membership_operations: 2,
            max_new_enrollments: 2,
            ..LimitsV2::default()
        },
    );
    let a = enroll(&r, 101, 8);
    let b = enroll(&r, 102, 8);
    assert_eq!(next(&r, &[a, b]).state().pending_count(), 2);
    let invalid = Op::from_parts(
        *a.header(),
        *a.action(),
        SignatureBytes::from_bytes([0; 64]),
    );
    let view = View::of(&r);
    assert!(r.stage_block(&view, &[invalid; 3]).is_err());
    assert_eq!(view.lookups.get(), 0);
}

#[test]
fn membership_and_new_enrollment_caps_apply_to_direct_input() {
    let membership_limited = registry_with(
        1,
        1,
        LimitsV2 {
            max_operations: 4,
            max_membership_operations: 2,
            max_new_enrollments: 2,
            ..LimitsV2::default()
        },
    );
    let a = enroll(&membership_limited, 101, 8);
    let b = enroll(&membership_limited, 102, 8);
    let id = a.enrollment_id().unwrap();
    assert_eq!(
        next(&membership_limited, &[a, b]).state().pending_count(),
        2
    );
    rejects(
        &membership_limited,
        &[a, b, approve(&membership_limited, 1, 0, id, 8)],
    );

    let enrollment_limited = registry_with(
        1,
        1,
        LimitsV2 {
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
    );
    let first = enroll(&enrollment_limited, 101, 8);
    let second = enroll(&enrollment_limited, 102, 8);
    let view = View::of(&enrollment_limited);
    assert!(
        enrollment_limited
            .stage_block(&view, &[first, second])
            .is_err()
    );
    assert_eq!(view.lookups.get(), 0);
    let n = next(&enrollment_limited, &[first]);
    assert_eq!(next(&n, &[enroll(&n, 102, 8)]).state().pending_count(), 2);
}

#[test]
fn full_block_byte_budget_includes_header_count_and_each_length_prefix() {
    for (limit, permitted) in [(758, true), (757, false)] {
        let r = registry_with(
            1,
            1,
            LimitsV2 {
                max_block_bytes: limit,
                max_operation_bytes: 291,
                ..LimitsV2::default()
            },
        );
        let operations = [enroll(&r, 101, 8), enroll(&r, 102, 8)];
        // 168-byte empty MRB2 + 2 * (4-byte length + 291-byte operation) = 758.
        let view = View::of(&r);
        assert_eq!(r.stage_block(&view, &operations).is_ok(), permitted);
        if !permitted {
            assert_eq!(view.lookups.get(), 0);
        }
    }
}

#[test]
fn pending_capacity_is_freed_by_ordered_withdrawal_not_early_activation() {
    let r = registry_with(
        1,
        1,
        LimitsV2 {
            max_pending: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
    );
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let parent = next(&r, &[e]);
    let replacement = enroll(&parent, 102, 8);
    let withdrawal = withdraw(&parent, 101, 1, id);
    rejects(&parent, &[replacement, withdrawal]);
    rejects(&parent, &[approve(&parent, 1, 0, id, 8), replacement]);
    let n = next(&parent, &[withdrawal, replacement]);
    assert_eq!(status(&n, id), Status::Revoked);
    assert_eq!(n.state().pending_count(), 1);
    assert_eq!(n.state().record_count(), 3);
}

#[test]
fn record_capacity_includes_seeds_and_tombstones_but_allows_withdrawal() {
    let r = registry_with(
        1,
        1,
        LimitsV2 {
            max_records: 3,
            max_bootstrap: 1,
            max_pending: 2,
            max_new_enrollments: 2,
            ..LimitsV2::default()
        },
    );
    let a = enroll(&r, 101, 1);
    let b = enroll(&r, 102, 1);
    let full = next(&r, &[a, b]);
    assert_eq!(full.state().record_count(), 3);
    let expired = next(&full, &[]);
    assert_eq!(expired.state().pending_count(), 0);
    rejects(&expired, &[enroll(&expired, 103, 8)]);
    let n = next(&expired, &[withdraw(&expired, 1, 0, member(&expired, 1))]);
    assert_eq!(n.state().record_count(), 3);
    assert_eq!(n.state().active_count(), 0);
    assert_eq!(n.state().active_high_water(), 1);
}

#[test]
fn aggregate_approval_budget_retains_history_and_leaves_withdrawal_available() {
    let r = registry_with(
        2,
        2,
        LimitsV2 {
            max_records: 5,
            max_bootstrap: 2,
            max_pending: 3,
            max_new_enrollments: 3,
            ..LimitsV2::default()
        },
    );
    let a = enroll(&r, 101, 8);
    let b = enroll(&r, 102, 8);
    let c = enroll(&r, 103, 8);
    let aid = a.enrollment_id().unwrap();
    let bid = b.enrollment_id().unwrap();
    let cid = c.enrollment_id().unwrap();
    let n = next(
        &r,
        &[
            a,
            b,
            c,
            approve(&r, 1, 0, aid, 8),
            approve(&r, 2, 0, aid, 8),
            approve(&r, 1, 1, bid, 8),
            approve(&r, 2, 1, bid, 8),
            approve(&r, 1, 2, cid, 8),
        ],
    );
    assert_eq!(n.state().retained_approval_count(), 5);
    assert_eq!(status(&n, cid), Status::Pending);
    rejects(&n, &[approve(&n, 2, 2, cid, 8)]);
    let mut withdrawn = next(&n, &[withdraw(&n, 1, 3, member(&n, 1))]);
    while withdrawn.state().height() < 9 {
        withdrawn = next(&withdrawn, &[]);
    }
    assert_eq!(status(&withdrawn, cid), Status::Expired);
    assert_eq!(withdrawn.state().retained_approval_count(), 5);
    assert_eq!(withdrawn.state().record(cid).unwrap().approval_count(), 1);
}

#[test]
fn per_record_approval_budget_cannot_be_bypassed_across_blocks() {
    let r = registry(65, 64);
    let e = enroll(&r, 101, 8);
    let id = e.enrollment_id().unwrap();
    let mut operations = vec![e];
    for actor in 1..=64 {
        operations.push(approve(&r, actor, 0, id, 8));
    }
    operations.push(withdraw(&r, 64, 1, member(&r, 64)));
    let pending = next(&r, &operations);
    assert_eq!(status(&pending, id), Status::Pending);
    assert_eq!(pending.state().record(id).unwrap().approval_count(), 64);
    assert_eq!(pending.state().active_count(), 64);
    rejects(&pending, &[approve(&pending, 65, 0, id, 8)]);
    assert_eq!(
        pending
            .state()
            .record(member(&pending, 65))
            .unwrap()
            .next_sequence(),
        0
    );
}

#[test]
fn withdrawn_applicant_does_not_activate_or_reuse_an_ordinal() {
    let r = registry(1, 1);
    let a = enroll(&r, 101, 8);
    let aid = a.enrollment_id().unwrap();
    let n = next(
        &r,
        &[a, approve(&r, 1, 0, aid, 8), withdraw(&r, 101, 1, aid)],
    );
    assert_eq!(status(&n, aid), Status::Revoked);
    assert_eq!(n.state().record(aid).unwrap().activation(), None);
    assert_eq!(n.state().total_activations(), 1);
    let b = enroll(&n, 102, 8);
    let bid = b.enrollment_id().unwrap();
    let m = next(&n, &[b, approve(&n, 1, 1, bid, 8)]);
    assert_eq!(
        m.state().record(bid).unwrap().activation().unwrap().ordinal,
        2
    );
}

#[test]
fn declined_population_never_reuses_an_old_activation_ordinal() {
    let r = registry(1, 1);
    let a = enroll(&r, 101, 8);
    let aid = a.enrollment_id().unwrap();
    let active = next(&r, &[a, approve(&r, 1, 0, aid, 8)]);
    let withdrawn = next(&active, &[withdraw(&active, 101, 1, aid)]);
    let b = enroll(&withdrawn, 102, 8);
    let bid = b.enrollment_id().unwrap();
    let n = next(&withdrawn, &[b, approve(&withdrawn, 1, 1, bid, 8)]);
    assert_eq!(
        n.state().record(aid).unwrap().activation().unwrap().ordinal,
        2
    );
    assert_eq!(
        n.state().record(bid).unwrap().activation().unwrap().ordinal,
        3
    );
    assert_eq!(n.state().total_activations(), 3);
    assert_eq!(n.state().active_high_water(), 2);
}

#[test]
fn every_action_rechecks_context_ancestry_window_and_signature_before_execution() {
    let r = registry(1, 1);
    let request = enroll(&r, 101, 8);
    let id = request.enrollment_id().unwrap();
    let parent = next(&r, &[request]);
    for (actor, operation) in [
        (102, enroll(&parent, 102, 8)),
        (1, approve(&parent, 1, 0, id, 8)),
        (101, withdraw(&parent, 101, 1, id)),
    ] {
        // Each unmodified action is independently admissible from this parent.
        assert!(parent.stage_block(&View::of(&parent), &[operation]).is_ok());
        let mut context = *operation.header();
        context.config_hash = tag(30, 100);
        let mut ancestry = *operation.header();
        ancestry.anchor_hash = tag(30, 101);
        let mut future = *operation.header();
        future.valid_from = 3; // candidate is height 2
        let mut expired = *operation.header();
        expired.anchor_height = 0;
        expired.anchor_hash = block_hash(0);
        expired.valid_from = 1;
        expired.valid_until = 1;
        for bad_header in [context, ancestry, future, expired] {
            let bad = Op::sign(bad_header, *operation.action(), &key(actor)).unwrap();
            assert!(bad.verify_signature().is_ok());
            rejects(&parent, &[bad]);
        }
        rejects(
            &parent,
            &[Op::from_parts(
                *operation.header(),
                *operation.action(),
                SignatureBytes::from_bytes([0; 64]),
            )],
        );
    }
}
