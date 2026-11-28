//! Immutable, validated configuration and proof-checked bootstrap for synthetic V2.
//! No genesis is mined, no registry is activated and no currency is issued here.
use std::{collections::BTreeSet, fmt};
use mirror_crypto::{sha256d, verify_hash, CryptoError, Hash256, Keypair, PublicKey, SignatureBytes};
use super::{encoding, LimitsV2};

pub const CONFIG_VERSION: u16 = 2;
pub const SYNTHETIC_PROFILE_VERSION: u16 = 1;
pub const MAX_APPROVAL_THRESHOLD: u16 = 64;
pub const CONFIG_FIXED_LEN: usize = 150;
pub const MAX_CONFIG_BYTES: usize = CONFIG_FIXED_LEN + 1_000 * 96 + 65_536 * 40;

/// Construction input, never an already-validated configuration or peer authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigParameters {
    pub format_version: u16,
    pub profile_version: u16,
    pub network: Hash256,
    pub pow_bits: u32,
    pub genesis_timestamp: u64,
    pub required_approvals: u16,
    pub evidence_profile: Hash256,
}

impl ConfigParameters {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        if self.format_version != CONFIG_VERSION { return Err(ConfigError::UnsupportedVersion); }
        if self.profile_version != SYNTHETIC_PROFILE_VERSION { return Err(ConfigError::UnsupportedProfile); }
        if self.network == Hash256::default() || self.evidence_profile == Hash256::default() {
            return Err(ConfigError::ZeroDomain);
        }
        if self.required_approvals == 0 || self.required_approvals > MAX_APPROVAL_THRESHOLD {
            return Err(ConfigError::InvalidThreshold);
        }
        // Same representable range as legacy PowTarget::from_compact, without a
        // personhood -> consensus -> core dependency. This does not mine or verify work.
        let exponent = self.pow_bits >> 24;
        let coefficient = self.pow_bits & 0x007f_ffff;
        if self.pow_bits & 0x0080_0000 != 0 || coefficient == 0 || exponent > 32 {
            return Err(ConfigError::InvalidPowTarget);
        }
        if exponent <= 3 && coefficient >> (8 * (3 - exponent)) == 0 {
            return Err(ConfigError::InvalidPowTarget);
        }
        Ok(())
    }
}

/// Opaque synthetic enrollment references, not proof of biological uniqueness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapDescriptor {
    pub public_key: PublicKey,
    pub nonce: Hash256,
    pub evidence: Hash256,
}

impl BootstrapDescriptor {
    pub fn encode(&self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[..32].copy_from_slice(self.public_key.as_bytes());
        bytes[32..64].copy_from_slice(self.nonce.as_bytes());
        bytes[64..].copy_from_slice(self.evidence.as_bytes());
        bytes
    }
}

/// Explicit nonvaluable test balances. No founder/marketing defaults or mint API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TestAllocation {
    pub address: [u8; 32],
    pub balance_nusa: u64,
}

/// Fields are private; callers cannot substitute a claimed context or mutate
/// rules after their hash has been computed. Decoding reuses all constructor checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigV2 {
    pub(super) parameters: ConfigParameters,
    pub(super) limits: LimitsV2,
    pub(super) bootstrap: Vec<BootstrapDescriptor>,
    pub(super) allocations: Vec<TestAllocation>,
    context: Hash256,
    initial_test_supply: u64,
}

impl ConfigV2 {
    pub fn new(parameters: ConfigParameters, limits: LimitsV2,
        bootstrap: &[BootstrapDescriptor], allocations: &[TestAllocation]) -> Result<Self, ConfigError> {
        parameters.validate()?;
        limits.validate()?;
        // Counts are checked before any input-sized cloning/set allocation.
        if bootstrap.is_empty() || bootstrap.len() > limits.max_bootstrap as usize
            || bootstrap.len() < usize::from(parameters.required_approvals) {
            return Err(ConfigError::InvalidBootstrapCount);
        }
        if allocations.len() > limits.max_accounts as usize { return Err(ConfigError::TooManyAllocations); }
        let mut keys = BTreeSet::new();
        let mut nonces = BTreeSet::new();
        let mut evidence = BTreeSet::new();
        for seed in bootstrap {
            if seed.nonce == Hash256::default() || seed.evidence == Hash256::default() {
                return Err(ConfigError::ZeroCommitment);
            }
            if !keys.insert(*seed.public_key.as_bytes()) { return Err(ConfigError::DuplicateKey); }
            if !nonces.insert(*seed.nonce.as_bytes()) { return Err(ConfigError::DuplicateNonce); }
            if !evidence.insert(*seed.evidence.as_bytes()) { return Err(ConfigError::DuplicateEvidence); }
        }
        let mut previous = None;
        let mut initial_test_supply = 0u64;
        for allocation in allocations {
            if previous.is_some_and(|address| address >= allocation.address) {
                return Err(ConfigError::AllocationsNotStrictlySorted);
            }
            previous = Some(allocation.address);
            initial_test_supply = initial_test_supply.checked_add(allocation.balance_nusa)
                .ok_or(ConfigError::SupplyOverflow)?;
        }
        let mut config = Self { parameters, limits, bootstrap: bootstrap.to_vec(),
            allocations: allocations.to_vec(), context: Hash256::default(), initial_test_supply };
        let mut preimage = b"MIRROR_CONFIG_V2".to_vec();
        preimage.extend_from_slice(&config.encode());
        config.context = sha256d(&preimage);
        Ok(config)
    }

    pub const fn parameters(&self) -> ConfigParameters { self.parameters }
    pub const fn limits(&self) -> &LimitsV2 { &self.limits }
    pub const fn context(&self) -> Hash256 { self.context }
    pub const fn initial_test_supply(&self) -> u64 { self.initial_test_supply }
    pub fn bootstrap(&self) -> &[BootstrapDescriptor] { &self.bootstrap }
    pub fn allocations(&self) -> &[TestAllocation] { &self.allocations }
    pub fn encode(&self) -> Vec<u8> { encoding::encode(self) }
    pub fn decode(bytes: &[u8]) -> Result<Self, ConfigError> { encoding::decode(bytes) }

    fn seed_hash(&self, domain: &[u8], index: u32) -> Result<Hash256, ConfigError> {
        let seed = self.bootstrap.get(index as usize).ok_or(ConfigError::UnknownSeed)?;
        let mut bytes = domain.to_vec();
        bytes.extend_from_slice(self.context.as_bytes());
        bytes.extend_from_slice(&index.to_le_bytes());
        bytes.extend_from_slice(&seed.encode());
        Ok(sha256d(&bytes))
    }

    /// Zero-based seed position is signed. It is not a reward/activation ordinal.
    pub fn seed_signing_hash(&self, index: u32) -> Result<Hash256, ConfigError> {
        self.seed_hash(b"MIRROR_SEED_V2", index)
    }
    pub fn seed_id(&self, index: u32) -> Result<Hash256, ConfigError> {
        self.seed_hash(b"MIRROR_SEED_ID_V2", index)
    }
    pub fn sign_seed(&self, index: u32, key: &Keypair) -> Result<SignatureBytes, ConfigError> {
        let seed = self.bootstrap.get(index as usize).ok_or(ConfigError::UnknownSeed)?;
        if seed.public_key != key.public_key() { return Err(ConfigError::WrongSeedKey); }
        Ok(key.sign_hash(&self.seed_signing_hash(index)?))
    }

    /// Every ordered seed must prove possession under THIS complete config.
    /// No majority/first-seed shortcut. No partially verified bootstrap escapes.
    /// Config validity alone does not validate its public keys; this boundary uses
    /// mirror-crypto's strict verification (including weak-key rejection).
    pub fn verify_bootstrap(&self, proofs: &[SignatureBytes]) -> Result<VerifiedBootstrap, ConfigError> {
        if proofs.len() != self.bootstrap.len() { return Err(ConfigError::WrongProofCount); }
        for (index, (seed, proof)) in self.bootstrap.iter().zip(proofs).enumerate() {
            verify_hash(&seed.public_key, &self.seed_signing_hash(index as u32)?, proof)
                .map_err(|error| ConfigError::InvalidSeedProof { index: index as u32, error })?;
        }
        let seeds = self.bootstrap.iter().enumerate().map(|(index, seed)| {
            Ok(VerifiedSeed { id: self.seed_id(index as u32)?, public_key: seed.public_key,
                ordinal: index as u64 + 1 })
        }).collect::<Result<Vec<_>, ConfigError>>()?;
        Ok(VerifiedBootstrap { config: self.clone(), seeds })
    }
}

/// Verified cryptographic possession only; actual seed people are still trusted
/// initial admissions. No V2 runtime membership rights exist in B1 yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedBootstrap { config: ConfigV2, seeds: Vec<VerifiedSeed> }
impl VerifiedBootstrap {
    pub const fn config(&self) -> &ConfigV2 { &self.config }
    pub fn seeds(&self) -> &[VerifiedSeed] { &self.seeds }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifiedSeed { id: Hash256, public_key: PublicKey, ordinal: u64 }
impl VerifiedSeed {
    pub const fn id(&self) -> Hash256 { self.id }
    pub const fn public_key(&self) -> PublicKey { self.public_key }
    pub const fn ordinal(&self) -> u64 { self.ordinal }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    UnsupportedVersion, UnsupportedProfile, ZeroDomain, InvalidThreshold,
    InvalidPowTarget, InvalidLimits, InvalidBootstrapCount, TooManyAllocations,
    ZeroCommitment, DuplicateKey, DuplicateNonce, DuplicateEvidence,
    AllocationsNotStrictlySorted, SupplyOverflow, InvalidMagic, Truncated,
    TrailingBytes, TooLarge, UnknownSeed, WrongSeedKey, WrongProofCount,
    InvalidSeedProof { index: u32, error: CryptoError },
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 configuration rejected: {self:?}")
    }
}
impl std::error::Error for ConfigError {}
