//! V2 transfer and unified signed-operation values.
//!
//! All values produced by constructors or decoders are untrusted. Structural
//! decoding checks the fixed wire shape; `verify_signature` checks key possession;
//! `validate_for_parent` adds configuration, authorization lifetime and the
//! development transfer profile. None of these APIs reads or changes balances,
//! consumes a sequence, admits a member or accepts a block.
//!
//! Membership operations delegate to the existing B2 implementation unchanged.
//! Transfer encoding repeats the frozen common prefix explicitly so adding kind 0
//! does not broaden the membership-only decoder or alter existing signed bytes.

use std::fmt;

use mirror_crypto::{
    CryptoError, Hash256, Keypair, PublicKey, SignatureBytes, sha256d, verify_hash,
};
use mirror_personhood::v2::{
    ConfigV2, OPERATION_PREFIX_LEN_V2, OPERATION_VERSION_V2, OperationErrorV2, OperationHeaderV2,
    ParentChainViewV2, SignedMembershipOperationV2,
};

use crate::Address;

pub const TRANSFER_KIND_V2: u8 = 0;
pub const TRANSFER_OPERATION_LEN_V2: usize = OPERATION_PREFIX_LEN_V2 + 32 + 8 + 8 + 64;

/// Construction input. Amounts are integer Nusa; this is not a validated payment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferBodyV2 {
    pub recipient: Address,
    pub amount: u64,
    pub fee: u64,
}

/// A fixed-size, immutable, untrusted signed transfer. The actor sequence is an
/// ACCOUNT sequence; it is separate from that key's membership sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignedTransferV2 {
    header: OperationHeaderV2,
    body: TransferBodyV2,
    signature: SignatureBytes,
}

impl SignedTransferV2 {
    /// Copy received construction fields without granting authorization or funds.
    pub const fn from_parts(
        header: OperationHeaderV2,
        body: TransferBodyV2,
        signature: SignatureBytes,
    ) -> Self {
        Self {
            header,
            body,
            signature,
        }
    }

    /// Sign the exact supplied intent after checking the selected signing key.
    /// Invalid windows, zero amounts or unsupported fees can still be signed;
    /// contextual validation rejects them before the future ledger executor.
    pub fn sign(
        header: OperationHeaderV2,
        body: TransferBodyV2,
        key: &Keypair,
    ) -> Result<Self, TransferErrorV2> {
        if header.actor_public_key != key.public_key() {
            return Err(TransferErrorV2::WrongSigningKey);
        }
        let mut transfer = Self::from_parts(header, body, SignatureBytes::from_bytes([0; 64]));
        transfer.signature = key.sign_hash(&transfer.signing_hash());
        Ok(transfer)
    }

    pub const fn header(&self) -> &OperationHeaderV2 {
        &self.header
    }

    pub const fn body(&self) -> &TransferBodyV2 {
        &self.body
    }

    pub const fn signature(&self) -> SignatureBytes {
        self.signature
    }

    pub const fn encoded_len(&self) -> usize {
        TRANSFER_OPERATION_LEN_V2
    }

    /// Preserve the existing account-address derivation. This derives an address
    /// from the untrusted key; it does not verify that key or any account balance.
    pub fn sender(&self) -> Address {
        Address::from_public_key(&self.header.actor_public_key)
    }

    pub fn unsigned_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(TRANSFER_OPERATION_LEN_V2 - 64);
        bytes.extend_from_slice(&OPERATION_VERSION_V2.to_le_bytes());
        bytes.push(TRANSFER_KIND_V2);
        bytes.extend_from_slice(self.header.config_hash.as_bytes());
        bytes.extend_from_slice(&self.header.anchor_height.to_le_bytes());
        bytes.extend_from_slice(self.header.anchor_hash.as_bytes());
        bytes.extend_from_slice(&self.header.valid_from.to_le_bytes());
        bytes.extend_from_slice(&self.header.valid_until.to_le_bytes());
        bytes.extend_from_slice(self.header.actor_public_key.as_bytes());
        bytes.extend_from_slice(&self.header.actor_sequence.to_le_bytes());
        bytes.extend_from_slice(self.body.recipient.as_bytes());
        bytes.extend_from_slice(&self.body.amount.to_le_bytes());
        bytes.extend_from_slice(&self.body.fee.to_le_bytes());
        bytes
    }

    pub fn signing_hash(&self) -> Hash256 {
        domain_hash(b"MIRROR_OPERATION_V2", &self.unsigned_bytes())
    }

    pub fn id(&self) -> Hash256 {
        domain_hash(b"MIRROR_OPERATION_ID_V2", &self.encode())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = self.unsigned_bytes();
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes
    }

    /// Structural decoding only. Check cap/version/kind/exact length before any
    /// field copies; no attacker-controlled count or variable payload is allocated.
    pub fn decode(config: &ConfigV2, bytes: &[u8]) -> Result<Self, TransferErrorV2> {
        if bytes.len() > config.limits().max_operation_bytes as usize {
            return Err(TransferErrorV2::TooLarge);
        }
        if bytes.len() < 3 {
            return Err(TransferErrorV2::InvalidLength);
        }
        if u16::from_le_bytes([bytes[0], bytes[1]]) != OPERATION_VERSION_V2 {
            return Err(TransferErrorV2::UnsupportedVersion);
        }
        if bytes[2] != TRANSFER_KIND_V2 {
            return Err(TransferErrorV2::UnsupportedTransferKind);
        }
        if bytes.len() != TRANSFER_OPERATION_LEN_V2 {
            return Err(TransferErrorV2::InvalidLength);
        }
        let mut reader = Reader { bytes, position: 3 };
        let header = OperationHeaderV2 {
            config_hash: Hash256::from_bytes(reader.take()?),
            anchor_height: u64::from_le_bytes(reader.take()?),
            anchor_hash: Hash256::from_bytes(reader.take()?),
            valid_from: u64::from_le_bytes(reader.take()?),
            valid_until: u64::from_le_bytes(reader.take()?),
            actor_public_key: PublicKey::from_bytes(reader.take()?),
            actor_sequence: u64::from_le_bytes(reader.take()?),
        };
        let body = TransferBodyV2 {
            recipient: Address::from_bytes(reader.take()?),
            amount: u64::from_le_bytes(reader.take()?),
            fee: u64::from_le_bytes(reader.take()?),
        };
        let signature = SignatureBytes::from_bytes(reader.take()?);
        if reader.position != bytes.len() {
            return Err(TransferErrorV2::InvalidLength);
        }
        Ok(Self::from_parts(header, body, signature))
    }

    /// Strict Ed25519, including weak-public-key rejection. Key possession does
    /// not prove balance, current account nonce, membership or block acceptance.
    pub fn verify_signature(&self) -> Result<(), TransferErrorV2> {
        verify_hash(
            &self.header.actor_public_key,
            &self.signing_hash(),
            &self.signature,
        )
        .map_err(TransferErrorV2::Crypto)
    }

    /// Pure contextual authorization and development-profile check against the
    /// SAME trusted, immutable parent-chain interface as membership B2.
    ///
    /// Positive amount and zero fee are necessary conditions for this synthetic
    /// profile. No account state is consulted, and no sequence is consumed. A
    /// positive amount or sequence of `u64::MAX` may pass here; MB-02B must check
    /// current balance/nonce and arithmetic before publishing a ledger change.
    pub fn validate_for_parent<V: ParentChainViewV2 + ?Sized>(
        &self,
        config: &ConfigV2,
        parent: &V,
    ) -> Result<(), TransferErrorV2> {
        if self.encoded_len() > config.limits().max_operation_bytes as usize {
            return Err(TransferErrorV2::TooLarge);
        }
        let h = &self.header;
        if h.config_hash != config.context() {
            return Err(TransferErrorV2::WrongContext);
        }
        if parent.config_hash() != config.context() {
            return Err(TransferErrorV2::WrongParentContext);
        }
        let height = parent
            .parent_height()
            .checked_add(1)
            .ok_or(TransferErrorV2::HeightOverflow)?;
        let age = height
            .checked_sub(h.anchor_height)
            .filter(|age| *age > 0)
            .ok_or(TransferErrorV2::AnchorNotPast)?;
        if age > config.limits().max_anchor_age {
            return Err(TransferErrorV2::AnchorTooOld);
        }
        let span = h
            .valid_until
            .checked_sub(h.valid_from)
            .ok_or(TransferErrorV2::InvertedWindow)?
            .checked_add(1)
            .ok_or(TransferErrorV2::WindowOverflow)?;
        if span > config.limits().max_validity_blocks {
            return Err(TransferErrorV2::WindowTooLong);
        }
        if h.anchor_height >= h.valid_from {
            return Err(TransferErrorV2::AnchorNotBeforeWindow);
        }
        if height < h.valid_from {
            return Err(TransferErrorV2::NotYetValid);
        }
        if height > h.valid_until {
            return Err(TransferErrorV2::Expired);
        }
        if self.body.amount == 0 {
            return Err(TransferErrorV2::ZeroAmount);
        }
        if self.body.fee != 0 {
            return Err(TransferErrorV2::UnsupportedFee);
        }
        let expected_hash = parent
            .canonical_hash_at(h.anchor_height)
            .ok_or(TransferErrorV2::UnknownAnchor)?;
        if expected_hash != h.anchor_hash {
            return Err(TransferErrorV2::WrongAnchorHash);
        }
        self.verify_signature()
    }
}

/// One ordered V2 operation, preserving complete membership B2 bytes. This enum
/// carries an untrusted signed value, not a validated ledger operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignedOperationV2 {
    Transfer(SignedTransferV2),
    Membership(SignedMembershipOperationV2),
}

impl SignedOperationV2 {
    pub const fn header(&self) -> &OperationHeaderV2 {
        match self {
            Self::Transfer(operation) => operation.header(),
            Self::Membership(operation) => operation.header(),
        }
    }

    pub const fn kind(&self) -> u8 {
        match self {
            Self::Transfer(_) => TRANSFER_KIND_V2,
            Self::Membership(operation) => operation.action().kind(),
        }
    }

    pub const fn encoded_len(&self) -> usize {
        match self {
            Self::Transfer(operation) => operation.encoded_len(),
            Self::Membership(operation) => operation.encoded_len(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Transfer(operation) => operation.encode(),
            Self::Membership(operation) => operation.encode(),
        }
    }

    pub fn signing_hash(&self) -> Hash256 {
        match self {
            Self::Transfer(operation) => operation.signing_hash(),
            Self::Membership(operation) => operation.signing_hash(),
        }
    }

    pub fn id(&self) -> Hash256 {
        match self {
            Self::Transfer(operation) => operation.id(),
            Self::Membership(operation) => operation.id(),
        }
    }

    /// Dispatch only exact known V2 kinds. In particular, no legacy fallback or
    /// generic application payload exists. Delegated decoders remain structural.
    pub fn decode(config: &ConfigV2, bytes: &[u8]) -> Result<Self, UnifiedOperationErrorV2> {
        if bytes.len() > config.limits().max_operation_bytes as usize {
            return Err(UnifiedOperationErrorV2::TooLarge);
        }
        if bytes.len() < 3 {
            return Err(UnifiedOperationErrorV2::InvalidLength);
        }
        if u16::from_le_bytes([bytes[0], bytes[1]]) != OPERATION_VERSION_V2 {
            return Err(UnifiedOperationErrorV2::UnsupportedVersion);
        }
        match bytes[2] {
            TRANSFER_KIND_V2 => SignedTransferV2::decode(config, bytes)
                .map(Self::Transfer)
                .map_err(UnifiedOperationErrorV2::Transfer),
            1..=3 => SignedMembershipOperationV2::decode(config, bytes)
                .map(Self::Membership)
                .map_err(UnifiedOperationErrorV2::Membership),
            _ => Err(UnifiedOperationErrorV2::UnsupportedKind),
        }
    }

    pub fn verify_signature(&self) -> Result<(), UnifiedOperationErrorV2> {
        match self {
            Self::Transfer(operation) => operation
                .verify_signature()
                .map_err(UnifiedOperationErrorV2::Transfer),
            Self::Membership(operation) => operation
                .verify_signature()
                .map_err(UnifiedOperationErrorV2::Membership),
        }
    }

    /// Common contextual dispatch, not combined accounts/membership execution.
    pub fn validate_for_parent<V: ParentChainViewV2 + ?Sized>(
        &self,
        config: &ConfigV2,
        parent: &V,
    ) -> Result<(), UnifiedOperationErrorV2> {
        match self {
            Self::Transfer(operation) => operation
                .validate_for_parent(config, parent)
                .map_err(UnifiedOperationErrorV2::Transfer),
            Self::Membership(operation) => operation
                .validate_for_parent(config, parent)
                .map_err(UnifiedOperationErrorV2::Membership),
        }
    }
}

impl From<SignedTransferV2> for SignedOperationV2 {
    fn from(value: SignedTransferV2) -> Self {
        Self::Transfer(value)
    }
}

impl From<SignedMembershipOperationV2> for SignedOperationV2 {
    fn from(value: SignedMembershipOperationV2) -> Self {
        Self::Membership(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferErrorV2 {
    WrongSigningKey,
    TooLarge,
    InvalidLength,
    UnsupportedVersion,
    UnsupportedTransferKind,
    WrongContext,
    WrongParentContext,
    HeightOverflow,
    AnchorNotPast,
    AnchorTooOld,
    InvertedWindow,
    WindowOverflow,
    WindowTooLong,
    AnchorNotBeforeWindow,
    NotYetValid,
    Expired,
    ZeroAmount,
    UnsupportedFee,
    UnknownAnchor,
    WrongAnchorHash,
    Crypto(CryptoError),
}

impl fmt::Display for TransferErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 transfer rejected: {self:?}")
    }
}

impl std::error::Error for TransferErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Crypto(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnifiedOperationErrorV2 {
    TooLarge,
    InvalidLength,
    UnsupportedVersion,
    UnsupportedKind,
    Transfer(TransferErrorV2),
    Membership(OperationErrorV2),
}

impl fmt::Display for UnifiedOperationErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transfer(error) => write!(f, "{error}"),
            Self::Membership(error) => write!(f, "{error}"),
            _ => write!(f, "V2 operation rejected: {self:?}"),
        }
    }
}

impl std::error::Error for UnifiedOperationErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transfer(error) => Some(error),
            Self::Membership(error) => Some(error),
            _ => None,
        }
    }
}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> Hash256 {
    let mut preimage = Vec::with_capacity(domain.len() + bytes.len());
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(bytes);
    sha256d(&preimage)
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], TransferErrorV2> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(TransferErrorV2::InvalidLength)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(TransferErrorV2::InvalidLength)?
            .try_into()
            .map_err(|_| TransferErrorV2::InvalidLength)?;
        self.position = end;
        Ok(value)
    }
}
