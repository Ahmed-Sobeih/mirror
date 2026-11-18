//! Private fault injection for otherwise unreachable u64 exhaustion and damaged
//! internal indexes. These constructors/mutations do not exist in the public API.
use super::*;
use crate::v2::{ConfigParameters, LimitsV2, OperationHeaderV2};
use mirror_crypto::Keypair;

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

struct Parent {
    context: Hash256,
    height: u64,
    tip: Hash256,
}
impl ParentChainViewV2 for Parent {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        self.height
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height == self.height).then_some(self.tip)
    }
}

fn setup() -> (Keypair, MembershipRegistryV2, Parent) {
    let key = Keypair::from_secret_bytes([1; 32]);
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(2),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 1,
            evidence_profile: hash(3),
        },
        LimitsV2::default(),
        &[BootstrapDescriptor {
            public_key: key.public_key(),
            nonce: hash(4),
            evidence: hash(5),
        }],
        &[],
    )
    .unwrap();
    let verified = config
        .verify_bootstrap(&[config.sign_seed(0, &key).unwrap()])
        .unwrap();
    let registry = MembershipRegistryV2::bootstrap(&verified)
        .unwrap()
        .bind_to_validated_block(hash(6));
    let parent = Parent {
        context: config.context(),
        height: 0,
        tip: hash(6),
    };
    (key, registry, parent)
}

fn signed(
    registry: &MembershipRegistryV2,
    key: &Keypair,
    sequence: u64,
    action: MembershipActionV2,
) -> SignedMembershipOperationV2 {
    SignedMembershipOperationV2::sign(
        OperationHeaderV2 {
            config_hash: registry.state.config.context(),
            anchor_height: 0,
            anchor_hash: hash(6),
            valid_from: 1,
            valid_until: 8,
            actor_public_key: key.public_key(),
            actor_sequence: sequence,
        },
        action,
        key,
    )
    .unwrap()
}

#[test]
fn sequence_exhaustion_rejects_without_wrapping_or_mutation() {
    let (key, mut registry, parent) = setup();
    let id = registry.state.record_by_key(key.public_key()).unwrap().id();
    registry
        .state
        .records
        .get_mut(id.as_bytes())
        .unwrap()
        .next_sequence = u64::MAX;
    assert!(registry.state.check_invariants().is_ok());
    let operation = signed(
        &registry,
        &key,
        u64::MAX,
        MembershipActionV2::Withdraw { own_member_id: id },
    );
    let before = registry.clone();
    assert_eq!(
        registry.stage_block(&parent, &[operation]),
        Err(StateErrorV2::SequenceOverflow)
    );
    assert_eq!(registry, before);
}

#[test]
fn parent_height_exhaustion_rejects_empty_block_without_wrap() {
    let (_, mut registry, mut parent) = setup();
    registry.state.height = u64::MAX;
    parent.height = u64::MAX;
    assert!(registry.state.check_invariants().is_ok());
    let before = registry.clone();
    assert_eq!(
        registry.stage_block(&parent, &[]),
        Err(StateErrorV2::HeightOverflow)
    );
    assert_eq!(registry, before);
}

#[test]
fn activation_ordinal_exhaustion_discards_enrollment_and_approval() {
    let (seed, mut registry, parent) = setup();
    // Impossible through the bounded public API; force the arithmetic edge to
    // make release-mode wraparound regressions detectable.
    registry.state.total_activations = u64::MAX;
    let newcomer = Keypair::from_secret_bytes([2; 32]);
    let enroll = signed(
        &registry,
        &newcomer,
        0,
        MembershipActionV2::Enroll {
            applicant_nonce: hash(7),
            evidence_commitment: hash(8),
            evidence_profile: hash(3),
        },
    );
    let approve = signed(
        &registry,
        &seed,
        0,
        MembershipActionV2::Approve {
            target_enrollment_id: enroll.enrollment_id().unwrap(),
        },
    );
    let before = registry.clone();
    assert_eq!(
        registry.stage_block(&parent, &[enroll, approve]),
        Err(StateErrorV2::CounterOverflow)
    );
    assert_eq!(registry, before);
}

#[test]
fn invariant_audit_rejects_damaged_key_and_evidence_indexes() {
    let (_, registry, _) = setup();
    let mut wrong_key = registry.state.clone();
    *wrong_key.used_keys.values_mut().next().unwrap() = [0; 32];
    assert!(wrong_key.check_invariants().is_err());
    let mut wrong_evidence = registry.state.clone();
    *wrong_evidence.used_evidence.values_mut().next().unwrap() = [0; 32];
    assert!(wrong_evidence.check_invariants().is_err());
    let mut missing_index = registry.state.clone();
    missing_index.used_keys.clear();
    assert!(missing_index.check_invariants().is_err());
}

#[test]
fn invariant_audit_derives_counts_and_activation_ordinals_from_records() {
    let (_, registry, _) = setup();
    let mut wrong_active = registry.state.clone();
    wrong_active.active_count = 0;
    assert!(wrong_active.check_invariants().is_err());
    let mut wrong_pending = registry.state.clone();
    wrong_pending.pending_count = 1;
    assert!(wrong_pending.check_invariants().is_err());
    let mut wrong_total = registry.state.clone();
    wrong_total.total_activations = 2;
    assert!(wrong_total.check_invariants().is_err());
    let mut wrong_high_water = registry.state.clone();
    wrong_high_water.active_high_water = 0;
    assert!(wrong_high_water.check_invariants().is_err());
    let mut wrong_approvals = registry.state.clone();
    wrong_approvals.retained_approval_count = 1;
    assert!(wrong_approvals.check_invariants().is_err());
    let mut missing_activation = registry.state.clone();
    missing_activation
        .records
        .values_mut()
        .next()
        .unwrap()
        .activation = None;
    assert!(missing_activation.check_invariants().is_err());
}

#[test]
fn block_tag_is_external_to_canonical_membership_state() {
    let (_, registry, _) = setup();
    let stage = StagedMembershipStateV2 {
        state: registry.state.clone(),
    };
    let first = stage.clone().bind_to_validated_block(hash(9));
    let second = stage.bind_to_validated_block(hash(10));
    assert_eq!(first.state(), second.state());
    assert_ne!(first.tip_hash(), second.tip_hash());
    let parent = Parent {
        context: first.state.config.context(),
        height: 0,
        tip: first.tip_hash(),
    };
    assert!(first.stage_block(&parent, &[]).is_ok());
    assert_eq!(
        second.stage_block(&parent, &[]),
        Err(StateErrorV2::WrongParentHash)
    );
}
