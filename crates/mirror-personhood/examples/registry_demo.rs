//! Pure development demonstration: synthetic evidence, no biometric matching,
//! no real enrollments, no currency and no changes to mirror-data.
use mirror_crypto::{Hash256, Keypair, sha256d};
use mirror_personhood::{
    Approval, Enrollment, MembershipEvent, MembershipRegistry, RegistryRules,
    decode_log, encode_log,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("EXPERIMENTAL MEMBERSHIP DEMO — synthetic attestations; no MRY issued");
    let rules = RegistryRules::new(sha256d(b"mirror-personhood-demo-only"), 1)?;
    let first = Keypair::from_secret_bytes([1; 32]);
    let applicant = Keypair::from_secret_bytes([2; 32]);
    let seed = Enrollment::sign(&rules, &first, Hash256::from_bytes([1; 32]), sha256d(b"synthetic seed evidence"))?;
    let request = Enrollment::sign(&rules, &applicant, Hash256::from_bytes([2; 32]), sha256d(b"synthetic applicant evidence"))?;
    let id = request.member_id();
    let approval = Approval::sign(&rules, id, seed.member_id(), &first);
    let mut registry = MembershipRegistry::new(rules.clone(), std::slice::from_ref(&seed))?;
    let events = vec![MembershipEvent::Enroll(request), MembershipEvent::Approve(approval)];
    registry.apply(&events[0])?;
    println!("After request: {:?}", registry.member(id).ok_or("missing pending membership")?.status());
    registry.apply(&events[1])?;
    println!("After approval: {:?}", registry.member(id).ok_or("missing active membership")?.status());
    let bytes = encode_log(&events)?;
    let restored = MembershipRegistry::replay(rules, &[seed], &decode_log(&bytes)?)?;
    if restored != registry { return Err("replayed registry differs".into()); }
    println!("Active memberships: {}", restored.active_count());
    println!("Population high-water: {}", restored.population_high_water());
    println!("State root: {}", restored.state_root());
    println!("Canonical log round-trip and signature revalidation: PASS");
    println!("Existing chain/database: untouched");
    Ok(())
}
