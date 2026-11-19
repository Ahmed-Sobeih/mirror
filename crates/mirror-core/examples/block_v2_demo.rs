//! Synthetic format demonstration only. No ledger execution or accepted block.
use mirror_core::{
    Address, NUSA_PER_MRY,
    v2::{
        BlockHeaderV2, BlockV2, SignedOperationV2, SignedTransferV2, TransferBodyV2,
        operations_root_v2,
    },
};
use mirror_crypto::{Hash256, Keypair, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, CONFIG_VERSION, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    OperationHeaderV2, ParentChainViewV2, SYNTHETIC_PROFILE_VERSION, SignedMembershipOperationV2,
};

struct SyntheticParent {
    context: Hash256,
    hash: Hash256,
}
impl ParentChainViewV2 for SyntheticParent {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        0
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height == 0).then_some(self.hash)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let alice = Keypair::from_secret_bytes([1; 32]);
    let bob = Keypair::from_secret_bytes([2; 32]);
    let applicant = Keypair::from_secret_bytes([3; 32]);
    let profile = sha256d(b"Mirror V2 example: synthetic evidence only");
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: CONFIG_VERSION,
            profile_version: SYNTHETIC_PROFILE_VERSION,
            network: sha256d(b"Mirror V2 format demonstration"),
            pow_bits: 0x1f0fffff,
            genesis_timestamp: 1_800_000_000,
            required_approvals: 1,
            evidence_profile: profile,
        },
        LimitsV2::default(),
        &[BootstrapDescriptor {
            public_key: alice.public_key(),
            nonce: sha256d(b"synthetic seed nonce"),
            evidence: sha256d(b"synthetic seed evidence"),
        }],
        &[],
    )?;
    let parent = SyntheticParent {
        context: config.context(),
        hash: sha256d(b"synthetic parent; no accepted V2 genesis exists here"),
    };
    let header = |key: &Keypair| OperationHeaderV2 {
        config_hash: config.context(),
        anchor_height: 0,
        anchor_hash: parent.hash,
        valid_from: 1,
        valid_until: 10,
        actor_public_key: key.public_key(),
        actor_sequence: 0,
    };
    let transfer = SignedTransferV2::sign(
        header(&alice),
        TransferBodyV2 {
            recipient: Address::from_public_key(&bob.public_key()),
            amount: NUSA_PER_MRY,
            fee: 0,
        },
        &alice,
    )?;
    let enroll = SignedMembershipOperationV2::sign(
        header(&applicant),
        MembershipActionV2::Enroll {
            applicant_nonce: sha256d(b"synthetic applicant nonce"),
            evidence_commitment: sha256d(b"synthetic applicant evidence"),
            evidence_profile: profile,
        },
        &applicant,
    )?;
    let approval = SignedMembershipOperationV2::sign(
        header(&alice),
        MembershipActionV2::Approve {
            target_enrollment_id: enroll.enrollment_id().expect("Enroll has an ID"),
        },
        &alice,
    )?;
    let operations = [
        SignedOperationV2::from(transfer),
        SignedOperationV2::from(enroll),
        SignedOperationV2::from(approval),
    ];
    for operation in &operations {
        operation.validate_for_parent(&config, &parent)?;
    }
    let root = operations_root_v2(&config, &operations)?;
    let block = BlockV2::new(
        &config,
        BlockHeaderV2::new(
            config.context(),
            1,
            parent.hash,
            root,
            sha256d(b"UNEXECUTED claimed ledger root for encoding demonstration"),
            1_800_000_001,
            config.parameters().pow_bits,
            0,
        ),
        &operations,
    )?;
    let bytes = block.encode(&config)?;
    let decoded = BlockV2::decode(&config, &bytes)?;
    assert_eq!(decoded, block);
    decoded.verify_integrity(&config)?;
    assert_eq!(decoded.operations()[0], operations[0]);
    assert_eq!(decoded.operations()[1], operations[1]);
    assert_eq!(decoded.operations()[2], operations[2]);

    println!("Mirror V2 operation/block format demonstration");
    println!("Configuration: {}", config.context());
    println!("Original positions: 0 Transfer, 1 Enroll, 2 Approve");
    println!("Signed operation bytes: 243 / 291 / 227");
    println!("Encoded block bytes: {}", bytes.len());
    println!("Operations root: {root}");
    println!("Candidate header hash: {}", block.hash());
    println!("Exact round-trip, body commitment and signatures: PASS");
    println!("Operation authorization against synthetic parent: PASS");

    let mut altered = bytes;
    *altered.last_mut().expect("nonempty block") ^= 1;
    let decoded_altered = BlockV2::decode(&config, &altered)?;
    assert!(decoded_altered.verify_integrity(&config).is_err());
    println!("Altered body rejected by integrity validation: PASS");
    println!("No balance was checked or changed; no membership was activated.");
    println!("Claimed ledger root and PoW are unvalidated; this is not an accepted block.");
    Ok(())
}
