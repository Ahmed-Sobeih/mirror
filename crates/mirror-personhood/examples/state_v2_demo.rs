//! Synthetic B3 transitions only: no block production, devices, files or currency.
use mirror_crypto::{Hash256, Keypair};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    MembershipRegistryV2, MembershipStatusV2, OperationHeaderV2, ParentChainViewV2,
    SignedMembershipOperationV2,
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
    println!("B3 SYNTHETIC MEMBERSHIP DEMO — no real people, biometrics or MRY");
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
    registry = registry
        .stage_block(
            &parent,
            &[
                enroll,
                sign(&config, &first, 0, approval)?,
                sign(&config, &second, 0, approval)?,
            ],
        )?
        .bind_to_validated_block(hash(101));
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
    registry = registry
        .stage_block(
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
        )?
        .bind_to_validated_block(hash(102));
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
    registry = registry
        .stage_block(&parent, &[good])?
        .bind_to_validated_block(hash(103));
    assert_eq!(registry.state().active_count(), 3);
    assert_eq!(registry.state().total_activations(), 4);
    assert_eq!(registry.state().active_high_water(), 3);
    println!(
        "Height 3 retry: ordinary surviving reviewer activates applicant; active=3, total first activations=4"
    );
    println!(
        "No founder override, grants, minting, biometric matching or accepted chain block claimed."
    );
    Ok(())
}
