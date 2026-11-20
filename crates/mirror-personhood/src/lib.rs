//! Experimental Ant Kiss membership state machine, not deployed consensus.
//!
//! Checks signed requests and attestations. It does NOT perform biometric
//! matching, prove biological uniqueness, mint money, or provide anonymity.
//! Only opaque, randomized evidence commitments belong here; never raw or
//! encrypted biometric archives. See docs/PERSONHOOD_V0_1.md.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use mirror_crypto::{CryptoError, Hash256, Keypair, PublicKey, SignatureBytes, sha256d, verify_hash};

mod codec;
pub use codec::{LogError, MAX_LOG_BYTES, decode_log, encode_log};

/// Resource bounds for the experimental bootstrap/configuration, not mainnet policy.
pub const MAX_BOOTSTRAP_MEMBERS: usize = 1_000;
pub const MAX_APPROVAL_THRESHOLD: u16 = 64;

/// An enrollment identifier, NOT a biometric identifier or proof of uniqueness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberId([u8; 32]);

impl MemberId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self { Self(bytes) }
    pub const fn as_bytes(&self) -> &[u8; 32] { &self.0 }
}

impl fmt::Display for MemberId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 { write!(f, "{byte:02x}")?; }
        Ok(())
    }
}

/// Explicit development rules. All replaying nodes must use the same rules and
/// ordered bootstrap enrollments. There is no hidden owner/administrator key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryRules {
    context: Hash256,
    required_approvals: u16,
}

impl RegistryRules {
    pub fn new(network: Hash256, required_approvals: u16) -> Result<Self, RegistryError> {
        if network == Hash256::default() || required_approvals == 0
            || required_approvals > MAX_APPROVAL_THRESHOLD {
            return Err(RegistryError::InvalidRules);
        }
        let context = digest(b"MIRROR_PERSONHOOD_RULES_V1", &[
            network.as_bytes(), &required_approvals.to_le_bytes(),
        ]);
        Ok(Self { context, required_approvals })
    }

    pub const fn context(&self) -> Hash256 { self.context }
    pub const fn required_approvals(&self) -> u16 { self.required_approvals }
}

/// Applicant-signed request. The evidence commitment must refer to an off-chain
/// witnessed capture/review. It must not be an unsalted hash of personal data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enrollment {
    context: Hash256,
    public_key: PublicKey,
    nonce: Hash256,
    evidence: Hash256,
    signature: SignatureBytes,
}

impl Enrollment {
    pub fn sign(rules: &RegistryRules, key: &Keypair, nonce: Hash256, evidence: Hash256)
        -> Result<Self, RegistryError> {
        if nonce == Hash256::default() || evidence == Hash256::default() {
            return Err(RegistryError::MissingCommitment);
        }
        let mut request = Self {
            context: rules.context, public_key: key.public_key(), nonce, evidence,
            signature: SignatureBytes::from_bytes([0; 64]),
        };
        request.signature = key.sign_hash(&request.signing_hash());
        Ok(request)
    }

    pub fn member_id(&self) -> MemberId {
        MemberId(*digest(b"MIRROR_MEMBER_ID_V1", &[self.signing_hash().as_bytes()]).as_bytes())
    }
    pub const fn public_key(&self) -> PublicKey { self.public_key }
    pub const fn evidence_commitment(&self) -> Hash256 { self.evidence }

    fn signing_hash(&self) -> Hash256 {
        digest(b"MIRROR_ENROLLMENT_V1", &[
            self.context.as_bytes(), self.public_key.as_bytes(),
            self.nonce.as_bytes(), self.evidence.as_bytes(),
        ])
    }

    fn verify(&self, rules: &RegistryRules) -> Result<(), RegistryError> {
        check_context(self.context, rules)?;
        if self.nonce == Hash256::default() || self.evidence == Hash256::default() {
            return Err(RegistryError::MissingCommitment);
        }
        verify_hash(&self.public_key, &self.signing_hash(), &self.signature)?;
        Ok(())
    }
}

/// One active member attests to one exact enrollment request and rules context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approval {
    context: Hash256,
    applicant: MemberId,
    verifier: MemberId,
    signature: SignatureBytes,
}

impl Approval {
    pub fn sign(rules: &RegistryRules, applicant: MemberId, verifier: MemberId, key: &Keypair) -> Self {
        let mut approval = Self {
            context: rules.context, applicant, verifier,
            signature: SignatureBytes::from_bytes([0; 64]),
        };
        approval.signature = key.sign_hash(&approval.signing_hash());
        approval
    }
    fn signing_hash(&self) -> Hash256 {
        digest(b"MIRROR_APPROVAL_V1", &[
            self.context.as_bytes(), self.applicant.as_bytes(), self.verifier.as_bytes(),
        ])
    }
}

/// Voluntary, terminal self-revocation. Third-party revocation, death reports,
/// appeals and key recovery are deliberately not invented in this milestone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Withdrawal {
    context: Hash256,
    member: MemberId,
    signature: SignatureBytes,
}

impl Withdrawal {
    pub fn sign(rules: &RegistryRules, member: MemberId, key: &Keypair) -> Self {
        let mut withdrawal = Self {
            context: rules.context, member,
            signature: SignatureBytes::from_bytes([0; 64]),
        };
        withdrawal.signature = key.sign_hash(&withdrawal.signing_hash());
        withdrawal
    }
    fn signing_hash(&self) -> Hash256 {
        digest(b"MIRROR_WITHDRAWAL_V1", &[self.context.as_bytes(), self.member.as_bytes()])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MembershipEvent { Enroll(Enrollment), Approve(Approval), Withdraw(Withdrawal) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipStatus { Pending, Active, Revoked }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Membership {
    enrollment: Enrollment,
    status: MembershipStatus,
    approvals: BTreeSet<MemberId>,
    activation_order: Option<u64>,
}

impl Membership {
    pub fn id(&self) -> MemberId { self.enrollment.member_id() }
    pub const fn status(&self) -> MembershipStatus { self.status }
    pub const fn public_key(&self) -> PublicKey { self.enrollment.public_key }
    pub const fn activation_order(&self) -> Option<u64> { self.activation_order }
    pub fn approval_count(&self) -> usize { self.approvals.len() }
}

/// Deterministic registry. Exact key/evidence reuse is rejected permanently,
/// including after withdrawal. Fresh keys/fresh evidence from the SAME HUMAN
/// cannot be detected by this module alone: private deduplication is external.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipRegistry {
    rules: RegistryRules,
    members: BTreeMap<MemberId, Membership>,
    used_keys: BTreeSet<[u8; 32]>,
    used_evidence: BTreeSet<[u8; 32]>,
    active_count: u64,
    population_high_water: u64,
    total_activations: u64,
}

impl MembershipRegistry {
    /// The ordered bootstrap list is an explicit trusted starting assumption.
    /// Seed members have ordinary member rights after construction, no admin role.
    /// This function allocates NO currency and does NOT pick a founder key.
    pub fn new(rules: RegistryRules, bootstrap: &[Enrollment]) -> Result<Self, RegistryError> {
        if bootstrap.len() < usize::from(rules.required_approvals)
            || bootstrap.len() > MAX_BOOTSTRAP_MEMBERS {
            return Err(RegistryError::InvalidBootstrap);
        }
        let mut registry = Self {
            rules, members: BTreeMap::new(), used_keys: BTreeSet::new(),
            used_evidence: BTreeSet::new(), active_count: 0,
            population_high_water: 0, total_activations: 0,
        };
        for enrollment in bootstrap {
            registry.insert_pending(enrollment)?;
            let ordinal = registry.total_activations + 1;
            let member = registry.members.get_mut(&enrollment.member_id())
                .ok_or(RegistryError::UnknownMember)?;
            member.status = MembershipStatus::Active;
            member.activation_order = Some(ordinal);
            registry.total_activations = ordinal;
            registry.active_count += 1;
        }
        registry.population_high_water = registry.active_count;
        Ok(registry)
    }

    pub fn member(&self, id: MemberId) -> Option<&Membership> { self.members.get(&id) }
    pub const fn active_count(&self) -> u64 { self.active_count }
    pub const fn population_high_water(&self) -> u64 { self.population_high_water }
    pub const fn total_activations(&self) -> u64 { self.total_activations }
    pub const fn rules(&self) -> &RegistryRules { &self.rules }

    /// A failed individual event leaves the registry unchanged.
    pub fn apply(&mut self, event: &MembershipEvent) -> Result<(), RegistryError> {
        match event {
            MembershipEvent::Enroll(enrollment) => self.insert_pending(enrollment),
            MembershipEvent::Approve(approval) => self.approve(approval),
            MembershipEvent::Withdraw(withdrawal) => self.withdraw(withdrawal),
        }
    }

    /// Apply an ordered candidate batch atomically. Intended as a future ledger
    /// integration boundary; the existing PoW chain does not yet call this.
    pub fn apply_batch(&mut self, events: &[MembershipEvent]) -> Result<(), RegistryError> {
        let mut candidate = self.clone();
        for event in events { candidate.apply(event)?; }
        *self = candidate;
        Ok(())
    }

    /// Restore from the trusted bootstrap configuration and untrusted events;
    /// every request, authorization and state transition is checked again.
    pub fn replay(rules: RegistryRules, bootstrap: &[Enrollment], events: &[MembershipEvent])
        -> Result<Self, RegistryError> {
        let mut registry = Self::new(rules, bootstrap)?;
        for event in events { registry.apply(event)?; }
        Ok(registry)
    }

    /// Canonical development state commitment. Counts, rules, tombstones and
    /// approval sets are committed in sorted order; no raw biometric data.
    pub fn state_root(&self) -> Hash256 {
        let mut bytes = b"MIRROR_MEMBERSHIP_STATE_V1".to_vec();
        bytes.extend_from_slice(self.rules.context.as_bytes());
        bytes.extend_from_slice(&self.active_count.to_le_bytes());
        bytes.extend_from_slice(&self.population_high_water.to_le_bytes());
        bytes.extend_from_slice(&self.total_activations.to_le_bytes());
        bytes.extend_from_slice(&(self.members.len() as u64).to_le_bytes());
        for (id, member) in &self.members {
            bytes.extend_from_slice(id.as_bytes());
            bytes.extend_from_slice(member.enrollment.public_key.as_bytes());
            bytes.extend_from_slice(member.enrollment.nonce.as_bytes());
            bytes.extend_from_slice(member.enrollment.evidence.as_bytes());
            bytes.push(match member.status {
                MembershipStatus::Pending => 0,
                MembershipStatus::Active => 1,
                MembershipStatus::Revoked => 2,
            });
            bytes.extend_from_slice(&member.activation_order.unwrap_or(0).to_le_bytes());
            bytes.extend_from_slice(&(member.approvals.len() as u64).to_le_bytes());
            for verifier in &member.approvals { bytes.extend_from_slice(verifier.as_bytes()); }
        }
        sha256d(&bytes)
    }

    fn insert_pending(&mut self, enrollment: &Enrollment) -> Result<(), RegistryError> {
        enrollment.verify(&self.rules)?;
        let id = enrollment.member_id();
        if self.members.contains_key(&id) { return Err(RegistryError::DuplicateEnrollment); }
        if self.used_keys.contains(enrollment.public_key.as_bytes()) {
            return Err(RegistryError::PublicKeyAlreadyRegistered);
        }
        if self.used_evidence.contains(enrollment.evidence.as_bytes()) {
            return Err(RegistryError::EvidenceAlreadyUsed);
        }
        self.used_keys.insert(*enrollment.public_key.as_bytes());
        self.used_evidence.insert(*enrollment.evidence.as_bytes());
        self.members.insert(id, Membership {
            enrollment: enrollment.clone(), status: MembershipStatus::Pending,
            approvals: BTreeSet::new(), activation_order: None,
        });
        Ok(())
    }

    fn approve(&mut self, approval: &Approval) -> Result<(), RegistryError> {
        check_context(approval.context, &self.rules)?;
        if approval.applicant == approval.verifier { return Err(RegistryError::SelfApproval); }
        let verifier = self.members.get(&approval.verifier).ok_or(RegistryError::UnknownMember)?;
        if verifier.status != MembershipStatus::Active { return Err(RegistryError::InactiveVerifier); }
        verify_hash(&verifier.public_key(), &approval.signing_hash(), &approval.signature)?;
        let mut applicant = self.members.get(&approval.applicant)
            .ok_or(RegistryError::UnknownMember)?.clone();
        if applicant.status != MembershipStatus::Pending { return Err(RegistryError::NotPending); }
        // Old approvals from withdrawn verifiers no longer count toward activation.
        if applicant.approvals.contains(&approval.verifier) { return Err(RegistryError::DuplicateApproval); }
        applicant.approvals.retain(|id| self.members.get(id)
            .is_some_and(|member| member.status == MembershipStatus::Active));
        applicant.approvals.insert(approval.verifier);
        let activate = applicant.approvals.len() >= usize::from(self.rules.required_approvals);
        let (next_active, next_total) = if activate {
            (self.active_count.checked_add(1).ok_or(RegistryError::CounterOverflow)?,
             self.total_activations.checked_add(1).ok_or(RegistryError::CounterOverflow)?)
        } else { (self.active_count, self.total_activations) };
        if activate {
            applicant.status = MembershipStatus::Active;
            applicant.activation_order = Some(next_total);
        }
        self.members.insert(approval.applicant, applicant);
        self.active_count = next_active;
        self.total_activations = next_total;
        self.population_high_water = self.population_high_water.max(next_active);
        Ok(())
    }

    fn withdraw(&mut self, withdrawal: &Withdrawal) -> Result<(), RegistryError> {
        check_context(withdrawal.context, &self.rules)?;
        let member = self.members.get(&withdrawal.member).ok_or(RegistryError::UnknownMember)?;
        if member.status == MembershipStatus::Revoked { return Err(RegistryError::AlreadyRevoked); }
        verify_hash(&member.public_key(), &withdrawal.signing_hash(), &withdrawal.signature)?;
        let next_active = if member.status == MembershipStatus::Active {
            self.active_count.checked_sub(1).ok_or(RegistryError::CounterOverflow)?
        } else { self.active_count };
        self.members.get_mut(&withdrawal.member).ok_or(RegistryError::UnknownMember)?.status = MembershipStatus::Revoked;
        self.active_count = next_active;
        // High-water population and activation ordinal deliberately never decrease.
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Crypto(CryptoError), InvalidRules, InvalidBootstrap, WrongContext, MissingCommitment,
    DuplicateEnrollment, PublicKeyAlreadyRegistered, EvidenceAlreadyUsed, UnknownMember,
    SelfApproval, InactiveVerifier, DuplicateApproval, NotPending, AlreadyRevoked, CounterOverflow,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Crypto(error) => write!(f, "membership signature error: {error}"),
            other => write!(f, "membership transition rejected: {other:?}"),
        }
    }
}
impl std::error::Error for RegistryError {}
impl From<CryptoError> for RegistryError {
    fn from(error: CryptoError) -> Self { Self::Crypto(error) }
}

fn check_context(context: Hash256, rules: &RegistryRules) -> Result<(), RegistryError> {
    if context != rules.context { return Err(RegistryError::WrongContext); }
    Ok(())
}
fn digest(domain: &[u8], fields: &[&[u8]]) -> Hash256 {
    let mut bytes = domain.to_vec();
    for field in fields { bytes.extend_from_slice(field); }
    sha256d(&bytes)
}

#[cfg(test)]
mod tests;

pub mod v2;
