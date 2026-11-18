//! Synthetic B2 authorization demonstration; no database, actual people or MRY.
use mirror_crypto::{Hash256, Keypair};
use mirror_personhood::v2::{BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2,
    MembershipActionV2, OperationHeaderV2, OperationErrorV2, ParentChainViewV2,
    SignedMembershipOperationV2};

struct SyntheticParent { config: Hash256, height: u64, genesis: Hash256 }
impl ParentChainViewV2 for SyntheticParent {
    fn config_hash(&self) -> Hash256 { self.config }
    fn parent_height(&self) -> u64 { self.height }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height == 0).then_some(self.genesis)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("B2 SYNTHETIC AUTHORIZATION DEMO — no membership activation or currency");
    let key = Keypair::from_secret_bytes([1;32]);
    let profile = Hash256::from_bytes([2;32]);
    let config = ConfigV2::new(ConfigParameters { format_version: 2, profile_version: 1,
        network: Hash256::from_bytes([3;32]), pow_bits: 0x207f_ffff,
        genesis_timestamp: 0, required_approvals: 1, evidence_profile: profile }, LimitsV2::default(),
        &[BootstrapDescriptor { public_key: key.public_key(), nonce: Hash256::from_bytes([4;32]),
            evidence: Hash256::from_bytes([5;32]) }], &[])?;
    let mut parent = SyntheticParent { config: config.context(), height: 0, genesis: Hash256::from_bytes([6;32]) };
    let header = OperationHeaderV2 { config_hash: config.context(), anchor_height: 0,
        anchor_hash: parent.genesis, valid_from: 1, valid_until: 8,
        actor_public_key: key.public_key(), actor_sequence: 0 };
    let action = MembershipActionV2::Enroll { applicant_nonce: Hash256::from_bytes([7;32]),
        evidence_commitment: Hash256::from_bytes([8;32]), evidence_profile: profile };
    let operation = SignedMembershipOperationV2::sign(header, action, &key)?;
    let decoded = SignedMembershipOperationV2::decode(&config, &operation.encode())?;
    decoded.validate_for_parent(&config, &parent)?;
    println!("Signed enrollment: {} bytes; candidate height 1: accepted by B2 checks", decoded.encoded_len());
    parent.height = 8;
    if decoded.validate_for_parent(&config, &parent) != Err(OperationErrorV2::Expired) {
        return Err("expired message was not rejected".into());
    }
    println!("Same message at candidate height 9: EXPIRED (correctly rejected)");
    println!("No actor rights, sequence consumption, biometric match or block acceptance claimed.");
    Ok(())
}
