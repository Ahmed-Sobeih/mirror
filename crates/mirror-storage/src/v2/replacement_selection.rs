//! Exact, independently pinned selection metadata for replacement profile 3.
//!
//! MSR2 wraps one unchanged MSS2 snapshot and a local publication generation.
//! Profile 3 is restricted to the current fixed-target rule: each replacement
//! must select a strictly higher accepted branch. This codec checks metadata,
//! not that a replacement occurred or was permitted. Checksums establish neither
//! authority nor freshness; complete accepted-chain and transition checks remain
//! separate. No filesystem layout, publication or recovery is implemented here.

use std::fmt;

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2};
use mirror_crypto::{Hash256, sha256d};

use super::selection::{STORAGE_SELECTION_LEN_V2, SelectionErrorV2, SelectionRecordV2};

pub const REPLACEMENT_SELECTION_MAGIC_V2: [u8; 4] = *b"MSR2";
pub const REPLACEMENT_SELECTION_LEN_V2: usize = 280;
pub const REPLACEMENT_SELECTION_VERSION_V2: u16 = 2;
pub const REPLACEMENT_SELECTION_PROFILE_V2: u16 = 3;

const SNAPSHOT_OFFSET: usize = 16;
const CHECKSUM_OFFSET: usize = SNAPSHOT_OFFSET + STORAGE_SELECTION_LEN_V2;
const CHECKSUM_DOMAIN: &[u8] = b"MIRROR_STORAGE_REPLACEMENT_SELECTION_V2";

/// TRUSTED CALLER INPUT, selected independently of the record being checked.
///
/// Generation counts local successful publications, not chain work, people,
/// rewards or global latestness. All five complete chain checkpoint fields are
/// mandatory. Reading these expectations from the record or an untrusted peer
/// removes the selection protection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplacementCheckpointV2 {
    pub generation: u64,
    pub chain: ChainReplayExpectationV2,
}

/// Bounded profile-3 metadata, not an accepted chain or durable commit permit.
///
/// Initialization starts at generation zero at any accepted height. A subsequent
/// strictly preferred replacement increases generation by one and selected
/// height by at least one, so generation cannot exceed selected height. Decoding
/// validates that necessary bound, not the actual history of publications.
/// Equal-height replacement or future human-weighted choice needs a separately
/// reviewed profile rather than changing this invariant implicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplacementSelectionV2 {
    generation: u64,
    snapshot: SelectionRecordV2,
}

impl ReplacementSelectionV2 {
    /// Derive canonical metadata from a complete accepted chain.
    /// This does not prove that the selected local generation was published.
    pub fn from_chain(
        chain: &ChainV2,
        generation: u64,
    ) -> Result<Self, ReplacementSelectionErrorV2> {
        let record = Self {
            generation,
            snapshot: SelectionRecordV2::from_chain(chain)?,
        };
        record.validate_generation()?;
        Ok(record)
    }

    pub const fn checkpoint(&self) -> ReplacementCheckpointV2 {
        ReplacementCheckpointV2 {
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

    /// Canonical MSR2 bytes: a 16-byte wrapper, unchanged 232-byte MSS2 snapshot,
    /// then SHA256d of the replacement-selection domain and complete prefix.
    pub fn encode(&self) -> [u8; REPLACEMENT_SELECTION_LEN_V2] {
        let mut bytes = [0; REPLACEMENT_SELECTION_LEN_V2];
        bytes[..4].copy_from_slice(&REPLACEMENT_SELECTION_MAGIC_V2);
        bytes[4..6].copy_from_slice(&REPLACEMENT_SELECTION_VERSION_V2.to_le_bytes());
        bytes[6..8].copy_from_slice(&REPLACEMENT_SELECTION_PROFILE_V2.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.generation.to_le_bytes());
        bytes[SNAPSHOT_OFFSET..CHECKSUM_OFFSET].copy_from_slice(&self.snapshot.encode());
        let checksum = selection_checksum(&bytes[..CHECKSUM_OFFSET]);
        bytes[CHECKSUM_OFFSET..].copy_from_slice(checksum.as_bytes());
        bytes
    }

    /// Check exact framing, outer integrity, the unchanged nested MSS2 contract,
    /// the generation bound and independent generation equality, in that order.
    /// No input-sized allocation, manifest decoding or chain replay occurs.
    pub fn decode_for(
        bytes: &[u8],
        expected: ReplacementCheckpointV2,
    ) -> Result<Self, ReplacementSelectionErrorV2> {
        if bytes.len() != REPLACEMENT_SELECTION_LEN_V2 {
            return Err(ReplacementSelectionErrorV2::InvalidLength);
        }
        if bytes[..4] != REPLACEMENT_SELECTION_MAGIC_V2 {
            return Err(ReplacementSelectionErrorV2::InvalidMagic);
        }
        let version = u16::from_le_bytes(array_at(bytes, 4));
        if version != REPLACEMENT_SELECTION_VERSION_V2 {
            return Err(ReplacementSelectionErrorV2::UnsupportedVersion(version));
        }
        let profile = u16::from_le_bytes(array_at(bytes, 6));
        if profile != REPLACEMENT_SELECTION_PROFILE_V2 {
            return Err(ReplacementSelectionErrorV2::UnsupportedProfile(profile));
        }
        let checksum = Hash256::from_bytes(array_at(bytes, CHECKSUM_OFFSET));
        if selection_checksum(&bytes[..CHECKSUM_OFFSET]) != checksum {
            return Err(ReplacementSelectionErrorV2::ChecksumMismatch);
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
            return Err(ReplacementSelectionErrorV2::WrongGeneration);
        }
        Ok(record)
    }

    /// Check the exact manifest bytes described by the embedded snapshot.
    /// MNM2 seed-proof validation and complete MRB2 replay are still required.
    pub fn verify_manifest_bytes(&self, bytes: &[u8]) -> Result<(), ReplacementSelectionErrorV2> {
        self.snapshot.verify_manifest_bytes(bytes)?;
        Ok(())
    }

    fn validate_generation(&self) -> Result<(), ReplacementSelectionErrorV2> {
        if self.generation > self.snapshot.checkpoint().height {
            return Err(ReplacementSelectionErrorV2::GenerationExceedsHeight);
        }
        Ok(())
    }
}

// Callers establish the exact length before these statically bounded offsets.
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
pub enum ReplacementSelectionErrorV2 {
    InvalidLength,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnsupportedProfile(u16),
    ChecksumMismatch,
    GenerationExceedsHeight,
    WrongGeneration,
    Snapshot(SelectionErrorV2),
}

impl From<SelectionErrorV2> for ReplacementSelectionErrorV2 {
    fn from(error: SelectionErrorV2) -> Self {
        Self::Snapshot(error)
    }
}

impl fmt::Display for ReplacementSelectionErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "V2 replacement selection rejected: {self:?}")
    }
}

impl std::error::Error for ReplacementSelectionErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Snapshot(error) => Some(error),
            _ => None,
        }
    }
}
