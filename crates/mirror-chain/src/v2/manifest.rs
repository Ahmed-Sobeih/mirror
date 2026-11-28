//! Explicit public synthetic network configuration and exact genesis pin.
//!
//! A manifest carries seed proofs, not administrative keys or authority to
//! change an already selected network. Decoding requires independently selected
//! configuration/genesis expectations. It does not validate a genesis block;
//! ChainV2 performs that separate check before accepting any ledger state.
use std::fmt;

use mirror_crypto::{Hash256, SignatureBytes};
use mirror_personhood::v2::{ConfigError, ConfigV2, MAX_CONFIG_BYTES, VerifiedBootstrap};

pub const MANIFEST_MAGIC_V2: [u8; 4] = *b"MNM2";
pub const MANIFEST_VERSION_V2: u16 = 2;
pub const MANIFEST_FIXED_LEN_V2: usize = 4 + 2 + 4 + 4 + 32;
// Matches the frozen MCF2 profile ceiling, not an eventual humanity limit.
const MAX_MANIFEST_PROOFS_V2: usize = 1_000;
pub const MAX_MANIFEST_BYTES_V2: usize =
    MANIFEST_FIXED_LEN_V2 + MAX_CONFIG_BYTES + 64 * MAX_MANIFEST_PROOFS_V2;

/// TRUSTED CALLER INPUT. Choose this outside the untrusted manifest/peer data.
/// Deriving it from the input being decoded would remove the network-pin check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManifestIdentityV2 {
    pub config_hash: Hash256,
    pub genesis_hash: Hash256,
}

/// Immutable configuration with every seed proof checked and a selected genesis
/// identity. This is not an accepted block, latest tip or proof of real people.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkManifestV2 {
    bootstrap: VerifiedBootstrap,
    seed_proofs: Vec<SignatureBytes>,
    genesis_hash: Hash256,
}

impl NetworkManifestV2 {
    /// Explicitly choose a synthetic network. All ordered seed proofs are
    /// verified under the complete config; the selected genesis must later be
    /// independently validated by ChainV2. Never substitute unchecked peer pins.
    pub fn new(
        config: &ConfigV2,
        seed_proofs: &[SignatureBytes],
        expected_genesis_hash: Hash256,
    ) -> Result<Self, ManifestErrorV2> {
        check_proof_count(config, seed_proofs.len())?;
        let bootstrap = config
            .verify_bootstrap(seed_proofs)
            .map_err(ManifestErrorV2::Config)?;
        let mut proofs = Vec::new();
        proofs
            .try_reserve_exact(seed_proofs.len())
            .map_err(|_| ManifestErrorV2::AllocationFailed)?;
        proofs.extend_from_slice(seed_proofs);
        Ok(Self {
            bootstrap,
            seed_proofs: proofs,
            genesis_hash: expected_genesis_hash,
        })
    }

    pub const fn config(&self) -> &ConfigV2 {
        self.bootstrap.config()
    }

    pub const fn bootstrap(&self) -> &VerifiedBootstrap {
        &self.bootstrap
    }

    pub const fn genesis_hash(&self) -> Hash256 {
        self.genesis_hash
    }

    pub const fn identity(&self) -> ManifestIdentityV2 {
        ManifestIdentityV2 {
            config_hash: self.config().context(),
            genesis_hash: self.genesis_hash,
        }
    }

    pub fn seed_proofs(&self) -> &[SignatureBytes] {
        &self.seed_proofs
    }

    /// Canonical MNM2 bytes. The metadata is public and contains no device data,
    /// private key, biometric archive or executable monetary policy.
    pub fn encode(&self) -> Result<Vec<u8>, ManifestErrorV2> {
        check_proof_count(self.config(), self.seed_proofs.len())?;
        let config_bytes = self.config().encode();
        let len = manifest_len(config_bytes.len(), self.seed_proofs.len())?;
        let config_len =
            u32::try_from(config_bytes.len()).map_err(|_| ManifestErrorV2::LengthOverflow)?;
        let proof_count =
            u32::try_from(self.seed_proofs.len()).map_err(|_| ManifestErrorV2::LengthOverflow)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(len)
            .map_err(|_| ManifestErrorV2::AllocationFailed)?;
        bytes.extend_from_slice(&MANIFEST_MAGIC_V2);
        bytes.extend_from_slice(&MANIFEST_VERSION_V2.to_le_bytes());
        bytes.extend_from_slice(&config_len.to_le_bytes());
        bytes.extend_from_slice(&config_bytes);
        bytes.extend_from_slice(&proof_count.to_le_bytes());
        for proof in &self.seed_proofs {
            bytes.extend_from_slice(proof.as_bytes());
        }
        bytes.extend_from_slice(self.genesis_hash.as_bytes());
        Ok(bytes)
    }

    /// Decode only for an independently selected configuration/genesis identity.
    /// Complete structural lengths/counts are preflighted before configuration
    /// or proof allocations and signature work. Every seed proof is then checked.
    /// No unpinned decoding API can choose the local network from arbitrary bytes.
    pub fn decode_for(bytes: &[u8], expected: ManifestIdentityV2) -> Result<Self, ManifestErrorV2> {
        if bytes.len() > MAX_MANIFEST_BYTES_V2 {
            return Err(ManifestErrorV2::TooLarge);
        }
        if bytes.len() < MANIFEST_FIXED_LEN_V2 {
            return Err(ManifestErrorV2::Truncated);
        }
        let mut input = Reader::new(bytes);
        if input.array::<4>()? != MANIFEST_MAGIC_V2 {
            return Err(ManifestErrorV2::InvalidMagic);
        }
        let version = u16::from_le_bytes(input.array()?);
        if version != MANIFEST_VERSION_V2 {
            return Err(ManifestErrorV2::UnsupportedVersion(version));
        }
        let config_len = u32::from_le_bytes(input.array()?) as usize;
        if config_len > MAX_CONFIG_BYTES {
            return Err(ManifestErrorV2::ConfigTooLarge);
        }
        let config_bytes = input.slice(config_len)?;
        let proof_count = u32::from_le_bytes(input.array()?) as usize;
        if proof_count > MAX_MANIFEST_PROOFS_V2 {
            return Err(ManifestErrorV2::TooManyProofs);
        }
        let expected_len = manifest_len(config_len, proof_count)?;
        if bytes.len() < expected_len {
            return Err(ManifestErrorV2::Truncated);
        }
        if bytes.len() > expected_len {
            return Err(ManifestErrorV2::TrailingBytes);
        }
        let proof_bytes = input.slice(
            proof_count
                .checked_mul(64)
                .ok_or(ManifestErrorV2::LengthOverflow)?,
        )?;
        let genesis_hash = Hash256::from_bytes(input.array()?);
        if genesis_hash != expected.genesis_hash {
            return Err(ManifestErrorV2::WrongGenesis);
        }
        let config = ConfigV2::decode(config_bytes).map_err(ManifestErrorV2::Config)?;
        if config.context() != expected.config_hash {
            return Err(ManifestErrorV2::WrongConfiguration);
        }
        check_proof_count(&config, proof_count)?;
        let mut proofs = Vec::new();
        proofs
            .try_reserve_exact(proof_count)
            .map_err(|_| ManifestErrorV2::AllocationFailed)?;
        for raw in proof_bytes.chunks_exact(64) {
            let mut bytes = [0; 64];
            bytes.copy_from_slice(raw);
            proofs.push(SignatureBytes::from_bytes(bytes));
        }
        Self::new(&config, &proofs, expected.genesis_hash)
    }
}

fn check_proof_count(config: &ConfigV2, count: usize) -> Result<(), ManifestErrorV2> {
    if count > MAX_MANIFEST_PROOFS_V2 {
        return Err(ManifestErrorV2::TooManyProofs);
    }
    if count != config.bootstrap().len() {
        return Err(ManifestErrorV2::WrongProofCount);
    }
    Ok(())
}

fn manifest_len(config_len: usize, proof_count: usize) -> Result<usize, ManifestErrorV2> {
    if config_len > MAX_CONFIG_BYTES {
        return Err(ManifestErrorV2::ConfigTooLarge);
    }
    if proof_count > MAX_MANIFEST_PROOFS_V2 {
        return Err(ManifestErrorV2::TooManyProofs);
    }
    let len = proof_count
        .checked_mul(64)
        .and_then(|n| n.checked_add(config_len))
        .and_then(|n| n.checked_add(MANIFEST_FIXED_LEN_V2))
        .ok_or(ManifestErrorV2::LengthOverflow)?;
    if len > MAX_MANIFEST_BYTES_V2 {
        return Err(ManifestErrorV2::TooLarge);
    }
    Ok(len)
}

struct Reader<'a> {
    remaining: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn slice(&mut self, len: usize) -> Result<&'a [u8], ManifestErrorV2> {
        if len > self.remaining.len() {
            return Err(ManifestErrorV2::Truncated);
        }
        let (prefix, rest) = self.remaining.split_at(len);
        self.remaining = rest;
        Ok(prefix)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], ManifestErrorV2> {
        let mut result = [0; N];
        result.copy_from_slice(self.slice(N)?);
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestErrorV2 {
    InvalidMagic,
    UnsupportedVersion(u16),
    Truncated,
    TrailingBytes,
    TooLarge,
    ConfigTooLarge,
    TooManyProofs,
    WrongProofCount,
    WrongConfiguration,
    WrongGenesis,
    Config(ConfigError),
    LengthOverflow,
    AllocationFailed,
}

impl fmt::Display for ManifestErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 network manifest rejected: {self:?}")
    }
}

impl std::error::Error for ManifestErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            _ => None,
        }
    }
}
