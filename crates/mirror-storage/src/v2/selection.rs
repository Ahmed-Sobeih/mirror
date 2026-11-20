//! Exact, bounded metadata for an explicitly selected immutable V2 store.
//!
//! MSS2 checksums detect damaged bytes, not malicious replacement or rollback.
//! Decoding requires an independently retained full checkpoint. Even a matching
//! record does not establish block or ledger validity: the storage caller must
//! verify exact MNM2 bytes and replay every complete MRB2 through `ChainV2`.
//! Existing network, block, operation and state encodings remain unchanged.

use std::fmt;

use mirror_chain::v2::{
    ChainReplayExpectationV2, ChainV2, MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2, ManifestErrorV2,
    manifest::{MANIFEST_FIXED_LEN_V2, MAX_MANIFEST_BYTES_V2},
};
use mirror_core::v2::MIN_ENCODED_BLOCK_LEN_V2;
use mirror_crypto::{Hash256, sha256d};

pub const STORAGE_SELECTION_MAGIC_V2: [u8; 4] = *b"MSS2";
pub const STORAGE_SELECTION_LEN_V2: usize = 232;
pub const STORAGE_SELECTION_VERSION_V2: u16 = 2;
/// Immutable initial publication only. Later generation updates need a reviewed
/// storage contract; accepting their number alone would not define recovery.
pub const STORAGE_SELECTION_PROFILE_V2: u16 = 1;
pub const MAX_STORAGE_READ_BYTES_V2: usize =
    STORAGE_SELECTION_LEN_V2 + MAX_MANIFEST_BYTES_V2 + MAX_CHAIN_BYTES_V2;

const CHECKSUM_OFFSET: usize = 200;
const CHECKSUM_DOMAIN: &[u8] = b"MIRROR_STORAGE_SELECTION_V2";

/// Bounded selected metadata, not an accepted chain or a freshness certificate.
///
/// Construction derives values from an already accepted `ChainV2`. Decoding
/// checks external selection and metadata consistency only; complete actual
/// block replay is still mandatory. No setter or unpinned decoder is exposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionRecordV2 {
    checkpoint: ChainReplayExpectationV2,
    generation: u64,
    block_count: u32,
    manifest_len: u32,
    blocks_bytes: u64,
    manifest_checksum: Hash256,
}

impl SelectionRecordV2 {
    /// Derive generation-zero metadata from a privately validated finite branch.
    /// This does not publish the store or select a globally canonical branch.
    pub fn from_chain(chain: &ChainV2) -> Result<Self, SelectionErrorV2> {
        let manifest_bytes = chain
            .manifest()
            .encode()
            .map_err(SelectionErrorV2::Manifest)?;
        let record = Self {
            checkpoint: chain.checkpoint(),
            generation: 0,
            block_count: u32::try_from(chain.blocks().len())
                .map_err(|_| SelectionErrorV2::LengthOverflow)?,
            manifest_len: u32::try_from(manifest_bytes.len())
                .map_err(|_| SelectionErrorV2::LengthOverflow)?,
            blocks_bytes: u64::try_from(chain.encoded_bytes())
                .map_err(|_| SelectionErrorV2::LengthOverflow)?,
            manifest_checksum: sha256d(&manifest_bytes),
        };
        record.validate_bounds()?;
        Ok(record)
    }

    pub const fn checkpoint(&self) -> ChainReplayExpectationV2 {
        self.checkpoint
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn block_count(&self) -> u32 {
        self.block_count
    }

    pub const fn manifest_len(&self) -> u32 {
        self.manifest_len
    }

    pub const fn blocks_bytes(&self) -> u64 {
        self.blocks_bytes
    }

    /// Integrity checksum of exact MNM2 bytes, not a new network identity.
    pub const fn manifest_checksum(&self) -> Hash256 {
        self.manifest_checksum
    }

    /// Canonical fixed-width MSS2 bytes; unsigned integers are little-endian.
    /// The trailing checksum covers the domain and the complete first 200 bytes.
    pub fn encode(&self) -> [u8; STORAGE_SELECTION_LEN_V2] {
        let mut bytes = [0; STORAGE_SELECTION_LEN_V2];
        bytes[0..4].copy_from_slice(&STORAGE_SELECTION_MAGIC_V2);
        bytes[4..6].copy_from_slice(&STORAGE_SELECTION_VERSION_V2.to_le_bytes());
        bytes[6..8].copy_from_slice(&STORAGE_SELECTION_PROFILE_V2.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.generation.to_le_bytes());
        bytes[16..48].copy_from_slice(self.checkpoint.config_hash.as_bytes());
        bytes[48..80].copy_from_slice(self.checkpoint.genesis_hash.as_bytes());
        bytes[80..88].copy_from_slice(&self.checkpoint.height.to_le_bytes());
        bytes[88..120].copy_from_slice(self.checkpoint.tip_hash.as_bytes());
        bytes[120..152].copy_from_slice(self.checkpoint.ledger_root.as_bytes());
        bytes[152..156].copy_from_slice(&self.block_count.to_le_bytes());
        bytes[156..160].copy_from_slice(&self.manifest_len.to_le_bytes());
        bytes[160..168].copy_from_slice(&self.blocks_bytes.to_le_bytes());
        bytes[168..200].copy_from_slice(self.manifest_checksum.as_bytes());
        let checksum = selection_checksum(&bytes[..CHECKSUM_OFFSET]);
        bytes[CHECKSUM_OFFSET..].copy_from_slice(checksum.as_bytes());
        bytes
    }

    /// Decode only against an independently retained full branch checkpoint.
    ///
    /// Reading `expected` from this same record or an untrusted peer removes the
    /// selection protection. Fixed length, known framing, integrity and semantic
    /// budgets are checked before matching each checkpoint field. No allocation,
    /// manifest decoding, signature verification or block replay occurs here.
    pub fn decode_for(
        bytes: &[u8],
        expected: ChainReplayExpectationV2,
    ) -> Result<Self, SelectionErrorV2> {
        if bytes.len() != STORAGE_SELECTION_LEN_V2 {
            return Err(SelectionErrorV2::InvalidLength);
        }
        if bytes[..4] != STORAGE_SELECTION_MAGIC_V2 {
            return Err(SelectionErrorV2::InvalidMagic);
        }
        let version = u16::from_le_bytes(array_at(bytes, 4));
        if version != STORAGE_SELECTION_VERSION_V2 {
            return Err(SelectionErrorV2::UnsupportedVersion(version));
        }
        let profile = u16::from_le_bytes(array_at(bytes, 6));
        if profile != STORAGE_SELECTION_PROFILE_V2 {
            return Err(SelectionErrorV2::UnsupportedProfile(profile));
        }
        let checksum = Hash256::from_bytes(array_at(bytes, CHECKSUM_OFFSET));
        if selection_checksum(&bytes[..CHECKSUM_OFFSET]) != checksum {
            return Err(SelectionErrorV2::ChecksumMismatch);
        }
        let record = Self {
            checkpoint: ChainReplayExpectationV2 {
                config_hash: Hash256::from_bytes(array_at(bytes, 16)),
                genesis_hash: Hash256::from_bytes(array_at(bytes, 48)),
                height: u64::from_le_bytes(array_at(bytes, 80)),
                tip_hash: Hash256::from_bytes(array_at(bytes, 88)),
                ledger_root: Hash256::from_bytes(array_at(bytes, 120)),
            },
            generation: u64::from_le_bytes(array_at(bytes, 8)),
            block_count: u32::from_le_bytes(array_at(bytes, 152)),
            manifest_len: u32::from_le_bytes(array_at(bytes, 156)),
            blocks_bytes: u64::from_le_bytes(array_at(bytes, 160)),
            manifest_checksum: Hash256::from_bytes(array_at(bytes, 168)),
        };
        record.validate_bounds()?;
        record.check_checkpoint(expected)?;
        Ok(record)
    }

    /// Check exact manifest byte length and integrity before MNM2 decoding.
    ///
    /// Success does not verify seed proofs, configuration or genesis. The caller
    /// must still use `NetworkManifestV2::decode_for` with independent network
    /// pins, then replay all actual blocks against the external full checkpoint.
    pub fn verify_manifest_bytes(&self, bytes: &[u8]) -> Result<(), SelectionErrorV2> {
        if bytes.len() != self.manifest_len as usize {
            return Err(SelectionErrorV2::ManifestLengthMismatch);
        }
        if sha256d(bytes) != self.manifest_checksum {
            return Err(SelectionErrorV2::ManifestChecksumMismatch);
        }
        Ok(())
    }

    fn validate_bounds(&self) -> Result<(), SelectionErrorV2> {
        if self.generation != 0 {
            return Err(SelectionErrorV2::UnsupportedGeneration(self.generation));
        }
        if self.block_count == 0 || u64::from(self.block_count) > MAX_CHAIN_BLOCKS_V2 as u64 {
            return Err(SelectionErrorV2::InvalidBlockCount);
        }
        // Validate count first, avoiding height + 1 overflow on arbitrary input.
        if self.checkpoint.height != u64::from(self.block_count) - 1 {
            return Err(SelectionErrorV2::HeightCountMismatch);
        }
        if u64::from(self.manifest_len) < MANIFEST_FIXED_LEN_V2 as u64
            || u64::from(self.manifest_len) > MAX_MANIFEST_BYTES_V2 as u64
        {
            return Err(SelectionErrorV2::InvalidManifestLength);
        }
        let minimum_bytes = u64::from(self.block_count)
            .checked_mul(MIN_ENCODED_BLOCK_LEN_V2 as u64)
            .ok_or(SelectionErrorV2::LengthOverflow)?;
        if self.blocks_bytes < minimum_bytes || self.blocks_bytes > MAX_CHAIN_BYTES_V2 as u64 {
            return Err(SelectionErrorV2::InvalidBlocksBytes);
        }
        Ok(())
    }

    fn check_checkpoint(&self, expected: ChainReplayExpectationV2) -> Result<(), SelectionErrorV2> {
        if self.checkpoint.config_hash != expected.config_hash {
            return Err(SelectionErrorV2::WrongConfiguration);
        }
        if self.checkpoint.genesis_hash != expected.genesis_hash {
            return Err(SelectionErrorV2::WrongGenesis);
        }
        if self.checkpoint.height != expected.height {
            return Err(SelectionErrorV2::WrongHeight);
        }
        if self.checkpoint.tip_hash != expected.tip_hash {
            return Err(SelectionErrorV2::WrongTip);
        }
        if self.checkpoint.ledger_root != expected.ledger_root {
            return Err(SelectionErrorV2::WrongLedgerRoot);
        }
        Ok(())
    }
}

// Used only at fixed, statically bounded offsets after the exact-length guard.
fn array_at<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    let mut result = [0; N];
    result.copy_from_slice(&bytes[offset..offset + N]);
    result
}

fn selection_checksum(prefix: &[u8]) -> Hash256 {
    let mut preimage = [0; CHECKSUM_DOMAIN.len() + CHECKSUM_OFFSET];
    preimage[..CHECKSUM_DOMAIN.len()].copy_from_slice(CHECKSUM_DOMAIN);
    preimage[CHECKSUM_DOMAIN.len()..].copy_from_slice(prefix);
    sha256d(&preimage)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionErrorV2 {
    InvalidLength,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnsupportedProfile(u16),
    UnsupportedGeneration(u64),
    ChecksumMismatch,
    InvalidBlockCount,
    HeightCountMismatch,
    InvalidManifestLength,
    InvalidBlocksBytes,
    WrongConfiguration,
    WrongGenesis,
    WrongHeight,
    WrongTip,
    WrongLedgerRoot,
    ManifestLengthMismatch,
    ManifestChecksumMismatch,
    LengthOverflow,
    Manifest(ManifestErrorV2),
}

impl fmt::Display for SelectionErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 storage selection rejected: {self:?}")
    }
}

impl std::error::Error for SelectionErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Manifest(error) => Some(error),
            _ => None,
        }
    }
}
