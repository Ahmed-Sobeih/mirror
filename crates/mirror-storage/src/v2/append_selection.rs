//! Exact, independently pinned selection metadata for append storage profile 2.
//!
//! MSU2 wraps one unchanged MSS2 profile-1 snapshot and an explicit local append
//! generation. Neither checksum supplies authority or freshness. A decoded record
//! still requires independent network selection and complete actual-chain replay.
//! In particular, the outer generation is not a population or consensus weight.

use std::fmt;

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2};
use mirror_crypto::{Hash256, sha256d};

use super::selection::{STORAGE_SELECTION_LEN_V2, SelectionErrorV2, SelectionRecordV2};

pub const APPEND_SELECTION_MAGIC_V2: [u8; 4] = *b"MSU2";
pub const APPEND_SELECTION_LEN_V2: usize = 280;
pub const APPEND_SELECTION_VERSION_V2: u16 = 2;
pub const APPEND_SELECTION_PROFILE_V2: u16 = 2;

const SNAPSHOT_OFFSET: usize = 16;
const CHECKSUM_OFFSET: usize = SNAPSHOT_OFFSET + STORAGE_SELECTION_LEN_V2;
const CHECKSUM_DOMAIN: &[u8] = b"MIRROR_STORAGE_APPEND_SELECTION_V2";

/// TRUSTED CALLER INPUT, retained independently of the record being checked.
///
/// The generation identifies a local append publication, not global latestness.
/// Reading either field from untrusted disk or peer metadata removes the matching
/// protection. All five complete chain checkpoint fields remain mandatory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppendCheckpointV2 {
    pub generation: u64,
    pub chain: ChainReplayExpectationV2,
}

/// Bounded append metadata with an unchanged embedded profile-1 snapshot.
///
/// Generation zero can initialize an already accepted history at any height.
/// One successful append increments generation and height by one; therefore a
/// structurally valid generation never exceeds the selected height. This codec
/// checks that bound and external equality, not publication transition history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppendSelectionV2 {
    generation: u64,
    snapshot: SelectionRecordV2,
}

impl AppendSelectionV2 {
    /// Derive metadata from an accepted chain and a selected local generation.
    /// This does not publish storage or establish that any append occurred.
    pub fn from_chain(chain: &ChainV2, generation: u64) -> Result<Self, AppendSelectionErrorV2> {
        let record = Self {
            generation,
            snapshot: SelectionRecordV2::from_chain(chain)?,
        };
        record.validate_generation()?;
        Ok(record)
    }

    pub const fn checkpoint(&self) -> AppendCheckpointV2 {
        AppendCheckpointV2 {
            generation: self.generation,
            chain: self.snapshot.checkpoint(),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn snapshot(&self) -> &SelectionRecordV2 {
        &self.snapshot
    }

    /// Canonical MSU2 bytes: a 16-byte wrapper, exact 232-byte MSS2 snapshot,
    /// then SHA256d of the append-selection domain and the complete prefix.
    pub fn encode(&self) -> [u8; APPEND_SELECTION_LEN_V2] {
        let mut bytes = [0; APPEND_SELECTION_LEN_V2];
        bytes[..4].copy_from_slice(&APPEND_SELECTION_MAGIC_V2);
        bytes[4..6].copy_from_slice(&APPEND_SELECTION_VERSION_V2.to_le_bytes());
        bytes[6..8].copy_from_slice(&APPEND_SELECTION_PROFILE_V2.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.generation.to_le_bytes());
        bytes[SNAPSHOT_OFFSET..CHECKSUM_OFFSET].copy_from_slice(&self.snapshot.encode());
        let checksum = selection_checksum(&bytes[..CHECKSUM_OFFSET]);
        bytes[CHECKSUM_OFFSET..].copy_from_slice(checksum.as_bytes());
        bytes
    }

    /// Check exact framing/integrity, the frozen embedded snapshot contract,
    /// generation bounds and the caller's independently selected generation.
    /// No input-sized allocation, manifest decoding or chain execution occurs.
    pub fn decode_for(
        bytes: &[u8],
        expected: AppendCheckpointV2,
    ) -> Result<Self, AppendSelectionErrorV2> {
        if bytes.len() != APPEND_SELECTION_LEN_V2 {
            return Err(AppendSelectionErrorV2::InvalidLength);
        }
        if bytes[..4] != APPEND_SELECTION_MAGIC_V2 {
            return Err(AppendSelectionErrorV2::InvalidMagic);
        }
        let version = u16::from_le_bytes(array_at(bytes, 4));
        if version != APPEND_SELECTION_VERSION_V2 {
            return Err(AppendSelectionErrorV2::UnsupportedVersion(version));
        }
        let profile = u16::from_le_bytes(array_at(bytes, 6));
        if profile != APPEND_SELECTION_PROFILE_V2 {
            return Err(AppendSelectionErrorV2::UnsupportedProfile(profile));
        }
        let checksum = Hash256::from_bytes(array_at(bytes, CHECKSUM_OFFSET));
        if selection_checksum(&bytes[..CHECKSUM_OFFSET]) != checksum {
            return Err(AppendSelectionErrorV2::ChecksumMismatch);
        }
        let record = Self {
            generation: u64::from_le_bytes(array_at(bytes, 8)),
            snapshot: SelectionRecordV2::decode_for(
                &bytes[SNAPSHOT_OFFSET..CHECKSUM_OFFSET],
                expected.chain,
            )?,
        };
        record.validate_generation()?;
        if record.generation != expected.generation {
            return Err(AppendSelectionErrorV2::WrongGeneration);
        }
        Ok(record)
    }

    /// Check exact manifest bytes under the embedded snapshot contract.
    /// The caller still must verify MNM2 seed proofs and replay all MRB2 blocks.
    pub fn verify_manifest_bytes(&self, bytes: &[u8]) -> Result<(), AppendSelectionErrorV2> {
        self.snapshot.verify_manifest_bytes(bytes)?;
        Ok(())
    }

    fn validate_generation(&self) -> Result<(), AppendSelectionErrorV2> {
        if self.generation > self.snapshot.checkpoint().height {
            return Err(AppendSelectionErrorV2::GenerationExceedsHeight);
        }
        Ok(())
    }
}

// All call sites follow the exact-length guard and use statically bounded offsets.
fn array_at<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    let mut result = [0; N];
    result.copy_from_slice(&bytes[offset..offset + N]);
    result
}

fn selection_checksum(prefix: &[u8]) -> Hash256 {
    let mut input = [0; CHECKSUM_DOMAIN.len() + CHECKSUM_OFFSET];
    input[..CHECKSUM_DOMAIN.len()].copy_from_slice(CHECKSUM_DOMAIN);
    input[CHECKSUM_DOMAIN.len()..].copy_from_slice(prefix);
    sha256d(&input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendSelectionErrorV2 {
    InvalidLength,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnsupportedProfile(u16),
    ChecksumMismatch,
    GenerationExceedsHeight,
    WrongGeneration,
    Snapshot(SelectionErrorV2),
}

impl From<SelectionErrorV2> for AppendSelectionErrorV2 {
    fn from(error: SelectionErrorV2) -> Self {
        Self::Snapshot(error)
    }
}

impl fmt::Display for AppendSelectionErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "V2 append selection rejected: {self:?}")
    }
}

impl std::error::Error for AppendSelectionErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Snapshot(error) => Some(error),
            _ => None,
        }
    }
}
