//! Synthetic B3 transitions and B4 commitments/replay. No real blocks or currency.
use mirror_crypto::{Hash256, Keypair};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    MembershipRegistryV2, MembershipReplayV2, MembershipStatusV2, OperationHeaderV2,
    ParentChainViewV2, ReplayChainViewV2, ReplayExpectationV2, ReplayGroupV2,
    SignedMembershipOperationV2, TrustedReplayBlockV2, UntrustedMembershipSnapshotV2,
    replay_group_commitment_v2,
};

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

struct SyntheticParent {
    context: Hash256,
    height: u64,
}
impl ParentChainViewV2 for SyntheticParent {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        self.height
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height <= self.height).then(|| hash(100 + height as u8))
    }
}

// Captured from a separate direct execution of THIS SYNTHETIC fixture, never
// constructed by trusting the decoded log. MB-02/03 must provide actual locally
// validated chain metadata and a durable expected checkpoint in a real node.
struct SyntheticHistory {
    context: Hash256,
    blocks: Vec<TrustedReplayBlockV2>,
}
impl ReplayChainViewV2 for SyntheticHistory {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn block_at(&self, height: u64) -> Option<TrustedReplayBlockV2> {
        self.blocks.get(usize::try_from(height).ok()?).copied()
    }
}

fn advance(
    registry: &MembershipRegistryV2,
    parent: &SyntheticParent,
    operations: &[SignedMembershipOperationV2],
    groups: &mut Vec<ReplayGroupV2>,
    history: &mut SyntheticHistory,
) -> Result<MembershipRegistryV2, Box<dyn std::error::Error>> {
    let staged = registry.stage_block(parent, operations)?;
    let height = staged.state().height();
    let block_hash = hash(100 + u8::try_from(height)?);
    let group = ReplayGroupV2::new(registry.state().config(), height, block_hash, operations)?;
    history.blocks.push(TrustedReplayBlockV2 {
        block_hash,
        group_commitment: group.commitment(history.context)?,
        membership_root: staged.state().root()?,
    });
    groups.push(group);
    Ok(staged.bind_to_validated_block(block_hash))
}

fn sign(
    config: &ConfigV2,
    key: &Keypair,
    sequence: u64,
    action: MembershipActionV2,
) -> Result<SignedMembershipOperationV2, Box<dyn std::error::Error>> {
    Ok(SignedMembershipOperationV2::sign(
        OperationHeaderV2 {
            config_hash: config.context(),
            anchor_height: 0,
            anchor_hash: hash(100),
            valid_from: 1,
            valid_until: 8,
            actor_public_key: key.public_key(),
            actor_sequence: sequence,
        },
        action,
        key,
    )?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("B4 SYNTHETIC MEMBERSHIP DEMO — no real people, biometrics or MRY");
    let first = Keypair::from_secret_bytes([1; 32]);
    let second = Keypair::from_secret_bytes([2; 32]);
    let third = Keypair::from_secret_bytes([3; 32]);
    let fourth = Keypair::from_secret_bytes([4; 32]);
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(10),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile: hash(11),
        },
        LimitsV2::default(),
        &[
            BootstrapDescriptor {
                public_key: first.public_key(),
                nonce: hash(12),
                evidence: hash(13),
            },
            BootstrapDescriptor {
                public_key: second.public_key(),
                nonce: hash(14),
                evidence: hash(15),
            },
        ],
        &[],
    )?;
    let bootstrap =
        config.verify_bootstrap(&[config.sign_seed(0, &first)?, config.sign_seed(1, &second)?])?;
    // These are labeled synthetic ancestry tags, NOT hashes of real valid blocks.
    let mut registry =
        MembershipRegistryV2::bootstrap(&bootstrap)?.bind_to_validated_block(hash(100));
    let mut groups = Vec::new();
    let mut history = SyntheticHistory {
        context: config.context(),
        blocks: vec![TrustedReplayBlockV2 {
            block_hash: hash(100),
            group_commitment: replay_group_commitment_v2(&config, 0, &[])?,
            membership_root: registry.state().root()?,
        }],
    };
    let mut parent = SyntheticParent {
        context: config.context(),
        height: 0,
    };
    let enroll = sign(
        &config,
        &third,
        0,
        MembershipActionV2::Enroll {
            applicant_nonce: hash(16),
            evidence_commitment: hash(17),
            evidence_profile: hash(11),
        },
    )?;
    let third_id = enroll.enrollment_id().ok_or("missing enrollment ID")?;
    let approval = MembershipActionV2::Approve {
        target_enrollment_id: third_id,
    };
    registry = advance(
        &registry,
        &parent,
        &[
            enroll,
            sign(&config, &first, 0, approval)?,
            sign(&config, &second, 0, approval)?,
        ],
        &mut groups,
        &mut history,
    )?;
    assert_eq!(registry.state().active_count(), 3);
    println!("Height 1: two distinct approvals activate applicant; active=3, ordinal=3");

    parent.height = 1;
    let enroll = sign(
        &config,
        &fourth,
        0,
        MembershipActionV2::Enroll {
            applicant_nonce: hash(18),
            evidence_commitment: hash(19),
            evidence_profile: hash(11),
        },
    )?;
    let fourth_id = enroll.enrollment_id().ok_or("missing enrollment ID")?;
    let approval = MembershipActionV2::Approve {
        target_enrollment_id: fourth_id,
    };
    registry = advance(
        &registry,
        &parent,
        &[
            enroll,
            sign(&config, &first, 1, approval)?,
            sign(&config, &third, 1, approval)?,
            sign(
                &config,
                &first,
                2,
                MembershipActionV2::Withdraw {
                    own_member_id: config.seed_id(0)?,
                },
            )?,
        ],
        &mut groups,
        &mut history,
    )?;
    assert_eq!(
        registry
            .state()
            .record(fourth_id)
            .ok_or("missing applicant")?
            .status(),
        MembershipStatusV2::Pending
    );
    assert_eq!(registry.state().active_count(), 2);
    assert_eq!(registry.state().active_high_water(), 3);
    println!(
        "Height 2: withdrawing reviewer's earlier approval cannot activate applicant; active=2, pending=1, high-water=3"
    );

    parent.height = 2;
    let good = sign(&config, &second, 1, approval)?;
    let bad = sign(
        &config,
        &third,
        2,
        MembershipActionV2::Withdraw {
            own_member_id: config.seed_id(1)?,
        },
    )?;
    let before = registry.clone();
    if registry.stage_block(&parent, &[good, bad]).is_ok() {
        return Err("unauthorized withdrawal accepted".into());
    }
    assert_eq!(registry, before);
    println!(
        "Invalid final operation: entire candidate rejected; original state and sequences unchanged"
    );
    registry = advance(&registry, &parent, &[good], &mut groups, &mut history)?;
    assert_eq!(registry.state().active_count(), 3);
    assert_eq!(registry.state().total_activations(), 4);
    assert_eq!(registry.state().active_high_water(), 3);
    println!(
        "Height 3 retry: ordinary surviving reviewer activates applicant; active=3, total first activations=4"
    );
    parent.height = 3;
    registry = advance(&registry, &parent, &[], &mut groups, &mut history)?;
    println!("Height 4: empty block retained; registry height is part of its commitment");
    let expected = ReplayExpectationV2 {
        config_hash: config.context(),
        height: registry.state().height(),
        tip_hash: registry.tip_hash(),
        membership_root: registry.state().root()?,
    };
    let log = MembershipReplayV2::new(&config, hash(100), &groups)?;
    let bytes = log.encode()?;
    let recovered =
        MembershipReplayV2::decode(&config, &bytes)?.replay(&bootstrap, &history, expected)?;
    assert_eq!(recovered, registry);
    let cache_bytes = registry.state().encode()?;
    let cache = UntrustedMembershipSnapshotV2::decode(&config, &cache_bytes)?;
    cache.verify_against(recovered.state())?;
    println!("MPH2 replay: all 4 groups and complete state match direct execution");
    println!("Membership root: {}", recovered.state().root()?);
    println!("MPS2 cache: matches reconstructed state; decoding alone grants no rights");
    let old = MembershipReplayV2::new(&config, hash(100), &groups[..3])?;
    assert!(old.replay(&bootstrap, &history, expected).is_err());
    let mut tampered = bytes.clone();
    tampered[82] ^= 1; // First group's claimed block hash, not a signature field.
    assert!(
        MembershipReplayV2::decode(&config, &tampered)?
            .replay(&bootstrap, &history, expected)
            .is_err()
    );
    println!("Old valid prefix and altered history: rejected against trusted checkpoint");
    println!(
        "No founder override, grants, minting, biometric matching or accepted chain block claimed."
    );
    Ok(())
}
