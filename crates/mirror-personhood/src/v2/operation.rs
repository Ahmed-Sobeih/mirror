//! B2: fixed-size signed membership operations. No registry mutation or money.
//!
//! Decoding is structural. Signature validation establishes key possession, not
//! reviewer eligibility, a current sequence, biological uniqueness or acceptance.
//! `validate_for_parent` additionally checks authorization context and lifetime
//! against a TRUSTED parent-chain view. B3 must still enforce all stateful rules.

use std::fmt;
use mirror_crypto::{sha256d, verify_hash, CryptoError, Hash256, Keypair, PublicKey, SignatureBytes};
use super::ConfigV2;

pub const OPERATION_VERSION_V2: u16 = 2;
pub const OPERATION_PREFIX_LEN_V2: usize = 131;
pub const ENROLL_OPERATION_LEN_V2: usize = 291;
pub const REVIEW_OPERATION_LEN_V2: usize = 227;

/// Unsigned construction data, not a validation certificate. Fields are copied
/// into an immutable signed operation; caller changes cannot alter it afterward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationHeaderV2 {
    pub config_hash: Hash256,
    pub anchor_height: u64,
    pub anchor_hash: Hash256,
    pub valid_from: u64,
    pub valid_until: u64,
    pub actor_public_key: PublicKey,
    pub actor_sequence: u64,
}

impl OperationHeaderV2 {
    // This module owns the shared V2 prefix. MB-02 may reuse it for Transfer=0,
    // but must not change membership bytes or accept Transfer in this decoder.
    pub(crate) fn append_prefix(&self, kind: u8, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&OPERATION_VERSION_V2.to_le_bytes());
        bytes.push(kind);
        bytes.extend_from_slice(self.config_hash.as_bytes());
        bytes.extend_from_slice(&self.anchor_height.to_le_bytes());
        bytes.extend_from_slice(self.anchor_hash.as_bytes());
        bytes.extend_from_slice(&self.valid_from.to_le_bytes());
        bytes.extend_from_slice(&self.valid_until.to_le_bytes());
        bytes.extend_from_slice(self.actor_public_key.as_bytes());
        bytes.extend_from_slice(&self.actor_sequence.to_le_bytes());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipActionV2 {
    Enroll { applicant_nonce: Hash256, evidence_commitment: Hash256, evidence_profile: Hash256 },
    Approve { target_enrollment_id: Hash256 },
    Withdraw { own_member_id: Hash256 },
}

impl MembershipActionV2 {
    pub const fn kind(&self) -> u8 {
        match self { Self::Enroll { .. } => 1, Self::Approve { .. } => 2, Self::Withdraw { .. } => 3 }
    }
    pub const fn signed_len(&self) -> usize {
        match self { Self::Enroll { .. } => ENROLL_OPERATION_LEN_V2, _ => REVIEW_OPERATION_LEN_V2 }
    }
    fn append_body(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Enroll { applicant_nonce, evidence_commitment, evidence_profile } => {
                bytes.extend_from_slice(applicant_nonce.as_bytes());
                bytes.extend_from_slice(evidence_commitment.as_bytes());
                bytes.extend_from_slice(evidence_profile.as_bytes());
            }
            Self::Approve { target_enrollment_id } => bytes.extend_from_slice(target_enrollment_id.as_bytes()),
            Self::Withdraw { own_member_id } => bytes.extend_from_slice(own_member_id.as_bytes()),
        }
    }
}

/// A fixed-size untrusted signed value, not an accepted membership transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignedMembershipOperationV2 {
    header: OperationHeaderV2,
    action: MembershipActionV2,
    signature: SignatureBytes,
}

impl SignedMembershipOperationV2 {
    /// Construct an untrusted received value. Call validation before using it.
    pub const fn from_parts(header: OperationHeaderV2, action: MembershipActionV2,
        signature: SignatureBytes) -> Self { Self { header, action, signature } }

    /// Sign exactly the requested fields. This verifies the selected signing key,
    /// not whether the request is currently admissible. In particular signing an
    /// expired window does NOT make it valid; validation requires a parent view.
    pub fn sign(header: OperationHeaderV2, action: MembershipActionV2, key: &Keypair)
        -> Result<Self, OperationErrorV2> {
        if header.actor_public_key != key.public_key() { return Err(OperationErrorV2::WrongSigningKey); }
        let mut operation = Self::from_parts(header, action, SignatureBytes::from_bytes([0; 64]));
        operation.signature = key.sign_hash(&operation.signing_hash());
        Ok(operation)
    }

    pub const fn header(&self) -> &OperationHeaderV2 { &self.header }
    pub const fn action(&self) -> &MembershipActionV2 { &self.action }
    pub const fn signature(&self) -> SignatureBytes { self.signature }
    pub const fn encoded_len(&self) -> usize { self.action.signed_len() }

    pub fn unsigned_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_len() - 64);
        self.header.append_prefix(self.action.kind(), &mut bytes);
        self.action.append_body(&mut bytes);
        bytes
    }
    pub fn signing_hash(&self) -> Hash256 {
        domain_hash(b"MIRROR_OPERATION_V2", &self.unsigned_bytes())
    }
    pub fn id(&self) -> Hash256 { domain_hash(b"MIRROR_OPERATION_ID_V2", &self.encode()) }
    /// The signature is intentionally excluded; ALL unsigned enrollment fields,
    /// including signer, sequence, ancestry and lifetime, are included.
    pub fn enrollment_id(&self) -> Option<Hash256> {
        match self.action {
            MembershipActionV2::Enroll { .. } => Some(domain_hash(b"MIRROR_ENROLLMENT_ID_V2", &self.unsigned_bytes())),
            _ => None,
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = self.unsigned_bytes();
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes
    }

    /// Structural decoder only. Kind 0 (Transfer) is deliberately NOT supported
    /// here. Check total cap, version, kind and exact size before fixed-size reads.
    /// No attacker-controlled vector count or variable payload can be allocated.
    pub fn decode(config: &ConfigV2, bytes: &[u8]) -> Result<Self, OperationErrorV2> {
        if bytes.len() > config.limits().max_operation_bytes as usize { return Err(OperationErrorV2::TooLarge); }
        if bytes.len() < 3 { return Err(OperationErrorV2::InvalidLength); }
        if u16::from_le_bytes([bytes[0], bytes[1]]) != OPERATION_VERSION_V2 {
            return Err(OperationErrorV2::UnsupportedVersion);
        }
        let kind = bytes[2];
        let expected = match kind {
            1 => ENROLL_OPERATION_LEN_V2,
            2 | 3 => REVIEW_OPERATION_LEN_V2,
            _ => return Err(OperationErrorV2::UnsupportedMembershipKind),
        };
        if bytes.len() != expected { return Err(OperationErrorV2::InvalidLength); }
        let mut input = Reader { bytes, position: 3 };
        let header = OperationHeaderV2 {
            config_hash: Hash256::from_bytes(input.take()?),
            anchor_height: u64::from_le_bytes(input.take()?),
            anchor_hash: Hash256::from_bytes(input.take()?),
            valid_from: u64::from_le_bytes(input.take()?),
            valid_until: u64::from_le_bytes(input.take()?),
            actor_public_key: PublicKey::from_bytes(input.take()?),
            actor_sequence: u64::from_le_bytes(input.take()?),
        };
        let action = match kind {
            1 => MembershipActionV2::Enroll {
                applicant_nonce: Hash256::from_bytes(input.take()?),
                evidence_commitment: Hash256::from_bytes(input.take()?),
                evidence_profile: Hash256::from_bytes(input.take()?),
            },
            2 => MembershipActionV2::Approve { target_enrollment_id: Hash256::from_bytes(input.take()?) },
            3 => MembershipActionV2::Withdraw { own_member_id: Hash256::from_bytes(input.take()?) },
            _ => return Err(OperationErrorV2::UnsupportedMembershipKind),
        };
        let signature = SignatureBytes::from_bytes(input.take()?);
        if input.position != bytes.len() { return Err(OperationErrorV2::InvalidLength); }
        Ok(Self::from_parts(header, action, signature))
    }

    /// Uses mirror-crypto's strict verifier, including weak-public-key rejection.
    /// Success says NOTHING about actor eligibility or whether the evidence is true.
    pub fn verify_signature(&self) -> Result<(), OperationErrorV2> {
        verify_hash(&self.header.actor_public_key, &self.signing_hash(), &self.signature)
            .map_err(OperationErrorV2::Crypto)
    }

    /// Pure contextual authorization check. The candidate height is derived from
    /// the supplied trusted parent, not a peer's advertised height or local clock.
    /// Membership state, sequence consumption, target existence/window alignment,
    /// reviewer assignment and final block acceptance remain B3/MB-02 checks.
    pub fn validate_for_parent<V: ParentChainViewV2 + ?Sized>(&self, config: &ConfigV2,
        parent: &V) -> Result<(), OperationErrorV2> {
        if self.encoded_len() > config.limits().max_operation_bytes as usize {
            return Err(OperationErrorV2::TooLarge);
        }
        let h = &self.header;
        if h.config_hash != config.context() { return Err(OperationErrorV2::WrongContext); }
        if parent.config_hash() != config.context() { return Err(OperationErrorV2::WrongParentContext); }
        let height = parent.parent_height().checked_add(1).ok_or(OperationErrorV2::HeightOverflow)?;
        let age = height.checked_sub(h.anchor_height).filter(|age| *age > 0)
            .ok_or(OperationErrorV2::AnchorNotPast)?;
        if age > config.limits().max_anchor_age { return Err(OperationErrorV2::AnchorTooOld); }
        let span = h.valid_until.checked_sub(h.valid_from)
            .ok_or(OperationErrorV2::InvertedWindow)?
            .checked_add(1).ok_or(OperationErrorV2::WindowOverflow)?;
        if span > config.limits().max_validity_blocks { return Err(OperationErrorV2::WindowTooLong); }
        if h.anchor_height >= h.valid_from { return Err(OperationErrorV2::AnchorNotBeforeWindow); }
        if height < h.valid_from { return Err(OperationErrorV2::NotYetValid); }
        if height > h.valid_until { return Err(OperationErrorV2::Expired); }
        if let MembershipActionV2::Enroll { applicant_nonce, evidence_commitment, evidence_profile } = self.action {
            if applicant_nonce == Hash256::default() || evidence_commitment == Hash256::default() {
                return Err(OperationErrorV2::ZeroCommitment);
            }
            if evidence_profile != config.parameters().evidence_profile { return Err(OperationErrorV2::WrongEvidenceProfile); }
            if h.actor_sequence != 0 { return Err(OperationErrorV2::InvalidEnrollmentSequence); }
        }
        let expected_hash = parent.canonical_hash_at(h.anchor_height).ok_or(OperationErrorV2::UnknownAnchor)?;
        if expected_hash != h.anchor_hash { return Err(OperationErrorV2::WrongAnchorHash); }
        self.verify_signature()
    }
}

/// TRUST BOUNDARY, not an oracle. Implement this on the locally validated,
/// immutable candidate-parent ancestry under the pinned config/genesis. Do not
/// implement it using an applicant/peer's unchecked hash map or advertised height.
/// The same stable view must serve the entire validation call/block. During reorg
/// use the candidate branch, never the old canonical chain. B2 tests use synthetic
/// views; a live-chain implementation is deliberately deferred to MB-02/03.
pub trait ParentChainViewV2 {
    fn config_hash(&self) -> Hash256;
    fn parent_height(&self) -> u64;
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationErrorV2 {
    WrongSigningKey, TooLarge, InvalidLength, UnsupportedVersion, UnsupportedMembershipKind,
    WrongContext, WrongParentContext, HeightOverflow, AnchorNotPast, AnchorTooOld,
    InvertedWindow, WindowOverflow, WindowTooLong, AnchorNotBeforeWindow, NotYetValid,
    Expired, ZeroCommitment, WrongEvidenceProfile, InvalidEnrollmentSequence,
    UnknownAnchor, WrongAnchorHash, Crypto(CryptoError),
}
impl fmt::Display for OperationErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 membership operation rejected: {self:?}")
    }
}
impl std::error::Error for OperationErrorV2 {}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> Hash256 {
    let mut preimage = Vec::with_capacity(domain.len() + bytes.len());
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(bytes);
    sha256d(&preimage)
}
struct Reader<'a> { bytes: &'a [u8], position: usize }
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], OperationErrorV2> {
        let end = self.position.checked_add(N).ok_or(OperationErrorV2::InvalidLength)?;
        let value = self.bytes.get(self.position..end).ok_or(OperationErrorV2::InvalidLength)?;
        let value = value.try_into().map_err(|_| OperationErrorV2::InvalidLength)?;
        self.position = end;
        Ok(value)
    }
}
