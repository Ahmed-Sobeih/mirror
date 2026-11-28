//! Bounded operator-owned control records and exact two-history restart intent.
//!
//! These are local control formats, not a peer protocol or a new consensus rule.
//! Their checksums detect corruption; authority comes from an independently chosen
//! network and the trusted operator-owned control directory. Replacing all local
//! records is outside that trust model. Neither decoder performs filesystem I/O.

use std::fmt;

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2, ManifestIdentityV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_network::v2::{HistoryErrorV2, HistoryFramesV2, HistoryReceiverV2};
use mirror_protocol::v2::{FRAME_HEADER_LEN_V2, MAX_HISTORY_WIRE_BYTES_V2, ProtocolErrorV2};
use mirror_storage::v2::{
    REPLACEMENT_SELECTION_LEN_V2, ReplacementCheckpointV2, ReplacementPlanErrorV2,
    ReplacementRecoveryErrorV2, ReplacementRecoveryRequestV2, ReplacementSelectionErrorV2,
    ReplacementSelectionV2, prepare_replacement_v2,
};

pub const CONTROL_RECORD_MAGIC_V2: [u8; 4] = *b"MNC2";
pub const CONTROL_RECORD_LEN_V2: usize = 464;
pub const INTENT_RECORD_MAGIC_V2: [u8; 4] = *b"MNI2";
pub const INTENT_HEADER_LEN_V2: usize = 952;
pub const INTENT_FIXED_LEN_V2: usize = INTENT_HEADER_LEN_V2 + 32;
/// Two independently bounded complete F1 histories, fixed metadata and checksum.
/// This is an encoded-input bound, not a resident-memory or elapsed-time promise.
pub const MAX_INTENT_BYTES_V2: usize = INTENT_FIXED_LEN_V2 + 2 * MAX_HISTORY_WIRE_BYTES_V2;

const VERSION: u16 = 2;
const KIND: u16 = 1;
const CONTROL_SELECTION_OFFSET: usize = 152;
const CONTROL_CHECKSUM_OFFSET: usize = CONTROL_SELECTION_OFFSET + REPLACEMENT_SELECTION_LEN_V2;
const INTENT_OLD_OFFSET: usize = 8;
const INTENT_CANDIDATE_OFFSET: usize = INTENT_OLD_OFFSET + CONTROL_RECORD_LEN_V2;
const INTENT_LENGTHS_OFFSET: usize = INTENT_CANDIDATE_OFFSET + CONTROL_RECORD_LEN_V2;
const CONTROL_DOMAIN: &[u8] = b"MIRROR_NODE_CONTROL_RECORD_V2";
const INTENT_DOMAIN: &[u8] = b"MIRROR_NODE_REPLACEMENT_INTENT_V2";

/// A checked local confirmed checkpoint with complete canonical MSR2 metadata.
///
/// Reading this object alone does not establish the current contents of a store.
/// A writer/status operation must independently open and replay that store against
/// this checkpoint. An unresolved intent must be handled explicitly first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlRecordV2 {
    selection: ReplacementSelectionV2,
}

impl ControlRecordV2 {
    pub fn from_chain(
        identity: ManifestIdentityV2,
        chain: &ChainV2,
        generation: u64,
    ) -> Result<Self, ControlRecordErrorV2> {
        if chain.manifest().identity() != identity {
            return Err(ControlRecordErrorV2::WrongIdentity);
        }
        Ok(Self {
            selection: ReplacementSelectionV2::from_chain(chain, generation)?,
        })
    }

    pub const fn identity(&self) -> ManifestIdentityV2 {
        let checkpoint = self.selection.checkpoint().chain;
        ManifestIdentityV2 {
            config_hash: checkpoint.config_hash,
            genesis_hash: checkpoint.genesis_hash,
        }
    }

    pub const fn checkpoint(&self) -> ReplacementCheckpointV2 {
        self.selection.checkpoint()
    }

    pub const fn selection(&self) -> &ReplacementSelectionV2 {
        &self.selection
    }

    /// Exact MNC2: header, explicit complete checkpoint, unchanged MSR2, checksum.
    /// The duplicated checkpoint is checked against the nested record on decode.
    pub fn encode(&self) -> [u8; CONTROL_RECORD_LEN_V2] {
        let mut bytes = [0; CONTROL_RECORD_LEN_V2];
        write_header(&mut bytes, CONTROL_RECORD_MAGIC_V2);
        write_checkpoint(&mut bytes[8..CONTROL_SELECTION_OFFSET], self.checkpoint());
        bytes[CONTROL_SELECTION_OFFSET..CONTROL_CHECKSUM_OFFSET]
            .copy_from_slice(&self.selection.encode());
        let hash = checksum(CONTROL_DOMAIN, &bytes[..CONTROL_CHECKSUM_OFFSET]);
        bytes[CONTROL_CHECKSUM_OFFSET..].copy_from_slice(hash.as_bytes());
        bytes
    }

    /// Decode only a fixed bounded record under independently selected C/genesis.
    /// The trusted local record supplies its checkpoint; a checksum is not proof
    /// of that trust, chain validity, current store contents or global freshness.
    pub fn decode_for(
        bytes: &[u8],
        identity: ManifestIdentityV2,
    ) -> Result<Self, ControlRecordErrorV2> {
        if bytes.len() != CONTROL_RECORD_LEN_V2 {
            return Err(ControlRecordErrorV2::InvalidLength);
        }
        check_header(bytes, CONTROL_RECORD_MAGIC_V2)?;
        check_checksum(CONTROL_DOMAIN, bytes, CONTROL_CHECKSUM_OFFSET)?;
        let expected = read_checkpoint(&bytes[8..CONTROL_SELECTION_OFFSET]);
        check_identity(identity, expected.chain)?;
        let selection = ReplacementSelectionV2::decode_for(
            &bytes[CONTROL_SELECTION_OFFSET..CONTROL_CHECKSUM_OFFSET],
            expected,
        )?;
        Ok(Self { selection })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmedSideV2 {
    Old,
    Candidate,
}

/// One validated local replacement intent, owning BOTH exact accepted histories.
///
/// There is no partial-history constructor and no ordinary writer here. Loading
/// executes both complete histories using the consuming F1 receiver, checks their
/// canonical metadata and repeats the existing strict replacement relationship.
/// The filesystem owner must persist/validate/sync this intent before publication.
pub struct IntentRecordV2 {
    old_record: ControlRecordV2,
    candidate_record: ControlRecordV2,
    old_chain: ChainV2,
    candidate_chain: ChainV2,
}

impl IntentRecordV2 {
    pub fn new(
        identity: ManifestIdentityV2,
        old: ReplacementCheckpointV2,
        old_chain: ChainV2,
        candidate_chain: ChainV2,
    ) -> Result<Self, ControlRecordErrorV2> {
        let old_record = ControlRecordV2::from_chain(identity, &old_chain, old.generation)?;
        if old_record.checkpoint() != old {
            return Err(ControlRecordErrorV2::WrongCurrentCheckpoint);
        }
        let plan = prepare_replacement_v2(
            identity,
            old,
            old_record.selection(),
            &old_chain,
            &candidate_chain,
        )?;
        let candidate_record = ControlRecordV2 {
            selection: *plan.candidate_selection(),
        };
        Ok(Self {
            old_record,
            candidate_record,
            old_chain,
            candidate_chain,
        })
    }

    pub const fn identity(&self) -> ManifestIdentityV2 {
        self.old_record.identity()
    }

    pub const fn old(&self) -> ReplacementCheckpointV2 {
        self.old_record.checkpoint()
    }

    pub const fn candidate(&self) -> ReplacementCheckpointV2 {
        self.candidate_record.checkpoint()
    }

    pub const fn old_record(&self) -> &ControlRecordV2 {
        &self.old_record
    }

    pub const fn candidate_record(&self) -> &ControlRecordV2 {
        &self.candidate_record
    }

    pub const fn old_chain(&self) -> &ChainV2 {
        &self.old_chain
    }

    pub const fn candidate_chain(&self) -> &ChainV2 {
        &self.candidate_chain
    }

    /// Only exact old or exact candidate metadata can coexist with this intent.
    /// A candidate-confirmed record does not permit retaining the older branch.
    /// The filesystem owner enforces that action rule and checks selected storage.
    pub fn validate_confirmed(
        &self,
        confirmed: &ControlRecordV2,
    ) -> Result<ConfirmedSideV2, ControlRecordErrorV2> {
        if *confirmed == self.old_record {
            Ok(ConfirmedSideV2::Old)
        } else if *confirmed == self.candidate_record {
            Ok(ConfirmedSideV2::Candidate)
        } else {
            Err(ControlRecordErrorV2::ConflictingConfirmed)
        }
    }

    pub fn request(&self) -> Result<ReplacementRecoveryRequestV2<'_>, ControlRecordErrorV2> {
        Ok(ReplacementRecoveryRequestV2::new(
            self.old(),
            self.candidate(),
            &self.old_chain,
            &self.candidate_chain,
        )?)
    }

    /// Exact local envelope with two unchanged, complete MPN2 frame streams.
    /// All declared stream/total lengths are bounded before reserving the output.
    pub fn encode(&self) -> Result<Vec<u8>, ControlRecordErrorV2> {
        let old_frames = HistoryFramesV2::new(&self.old_chain)?;
        let candidate_frames = HistoryFramesV2::new(&self.candidate_chain)?;
        let old_len = old_frames.summary().wire_len()?;
        let candidate_len = candidate_frames.summary().wire_len()?;
        let total = checked_intent_len(old_len as u64, candidate_len as u64)?;
        let mut bytes = Vec::with_capacity(total);
        bytes.resize(INTENT_HEADER_LEN_V2, 0);
        write_header(&mut bytes, INTENT_RECORD_MAGIC_V2);
        bytes[INTENT_OLD_OFFSET..INTENT_CANDIDATE_OFFSET]
            .copy_from_slice(&self.old_record.encode());
        bytes[INTENT_CANDIDATE_OFFSET..INTENT_LENGTHS_OFFSET]
            .copy_from_slice(&self.candidate_record.encode());
        bytes[INTENT_LENGTHS_OFFSET..INTENT_LENGTHS_OFFSET + 8]
            .copy_from_slice(&(old_len as u64).to_le_bytes());
        bytes[INTENT_LENGTHS_OFFSET + 8..INTENT_HEADER_LEN_V2]
            .copy_from_slice(&(candidate_len as u64).to_le_bytes());
        for frame in old_frames.chain(candidate_frames) {
            let frame = frame?;
            let next = bytes
                .len()
                .checked_add(frame.len())
                .ok_or(ControlRecordErrorV2::LengthOverflow)?;
            if next > total - 32 {
                return Err(ControlRecordErrorV2::InvalidLength);
            }
            bytes.extend_from_slice(&frame);
        }
        if bytes.len() != total - 32 {
            return Err(ControlRecordErrorV2::InvalidLength);
        }
        let hash = checksum(INTENT_DOMAIN, &bytes);
        bytes.extend_from_slice(hash.as_bytes());
        Ok(bytes)
    }

    /// Reject framing, caps and envelope corruption before executing histories.
    /// Header admission then precedes each bounded history payload's acceptance.
    /// Both complete accepted histories must match canonical nested records and
    /// the existing replacement rule before any intent is returned.
    pub fn decode_for(
        bytes: &[u8],
        identity: ManifestIdentityV2,
    ) -> Result<Self, ControlRecordErrorV2> {
        if !(INTENT_FIXED_LEN_V2..=MAX_INTENT_BYTES_V2).contains(&bytes.len()) {
            return Err(ControlRecordErrorV2::InvalidLength);
        }
        check_header(bytes, INTENT_RECORD_MAGIC_V2)?;
        let old_len = u64::from_le_bytes(array_at(bytes, INTENT_LENGTHS_OFFSET));
        let candidate_len = u64::from_le_bytes(array_at(bytes, INTENT_LENGTHS_OFFSET + 8));
        if checked_intent_len(old_len, candidate_len)? != bytes.len() {
            return Err(ControlRecordErrorV2::InvalidLength);
        }
        check_checksum(INTENT_DOMAIN, bytes, bytes.len() - 32)?;
        let old_record = ControlRecordV2::decode_for(
            &bytes[INTENT_OLD_OFFSET..INTENT_CANDIDATE_OFFSET],
            identity,
        )?;
        let candidate_record = ControlRecordV2::decode_for(
            &bytes[INTENT_CANDIDATE_OFFSET..INTENT_LENGTHS_OFFSET],
            identity,
        )?;
        let split = INTENT_HEADER_LEN_V2 + old_len as usize;
        let old_chain = decode_history(&bytes[INTENT_HEADER_LEN_V2..split], identity)?;
        let candidate_chain = decode_history(&bytes[split..bytes.len() - 32], identity)?;
        let intent = Self::new(
            identity,
            old_record.checkpoint(),
            old_chain,
            candidate_chain,
        )?;
        if intent.old_record != old_record || intent.candidate_record != candidate_record {
            return Err(ControlRecordErrorV2::NonCanonicalRecord);
        }
        Ok(intent)
    }
}

impl fmt::Debug for IntentRecordV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntentRecordV2")
            .field("old", &self.old())
            .field("candidate", &self.candidate())
            .finish()
    }
}

fn checked_intent_len(old: u64, candidate: u64) -> Result<usize, ControlRecordErrorV2> {
    if old == 0
        || candidate == 0
        || old > MAX_HISTORY_WIRE_BYTES_V2 as u64
        || candidate > MAX_HISTORY_WIRE_BYTES_V2 as u64
    {
        return Err(ControlRecordErrorV2::HistoryLengthOutOfBounds);
    }
    let total = old
        .checked_add(candidate)
        .and_then(|value| value.checked_add(INTENT_FIXED_LEN_V2 as u64))
        .ok_or(ControlRecordErrorV2::LengthOverflow)?;
    usize::try_from(total).map_err(|_| ControlRecordErrorV2::LengthOverflow)
}

fn decode_history(
    bytes: &[u8],
    identity: ManifestIdentityV2,
) -> Result<ChainV2, ControlRecordErrorV2> {
    let mut receiver = HistoryReceiverV2::new(identity);
    let mut offset = 0usize;
    while offset < bytes.len() {
        let header_end = offset
            .checked_add(FRAME_HEADER_LEN_V2)
            .filter(|end| *end <= bytes.len())
            .ok_or(ControlRecordErrorV2::InvalidLength)?;
        let pending = receiver.begin_frame(&bytes[offset..header_end])?;
        let payload_end = header_end
            .checked_add(pending.payload_len())
            .filter(|end| *end <= bytes.len())
            .ok_or(ControlRecordErrorV2::InvalidLength)?;
        receiver = pending.accept_payload(&bytes[header_end..payload_end])?;
        offset = payload_end;
    }
    Ok(receiver.finish()?.into_chain())
}

fn write_header(bytes: &mut [u8], magic: [u8; 4]) {
    bytes[..4].copy_from_slice(&magic);
    bytes[4..6].copy_from_slice(&VERSION.to_le_bytes());
    bytes[6..8].copy_from_slice(&KIND.to_le_bytes());
}

fn check_header(bytes: &[u8], magic: [u8; 4]) -> Result<(), ControlRecordErrorV2> {
    if bytes[..4] != magic {
        return Err(ControlRecordErrorV2::InvalidMagic);
    }
    let version = u16::from_le_bytes(array_at(bytes, 4));
    if version != VERSION {
        return Err(ControlRecordErrorV2::UnsupportedVersion(version));
    }
    let kind = u16::from_le_bytes(array_at(bytes, 6));
    if kind != KIND {
        return Err(ControlRecordErrorV2::UnsupportedKind(kind));
    }
    Ok(())
}

fn write_checkpoint(bytes: &mut [u8], checkpoint: ReplacementCheckpointV2) {
    bytes[..8].copy_from_slice(&checkpoint.generation.to_le_bytes());
    bytes[8..40].copy_from_slice(checkpoint.chain.config_hash.as_bytes());
    bytes[40..72].copy_from_slice(checkpoint.chain.genesis_hash.as_bytes());
    bytes[72..80].copy_from_slice(&checkpoint.chain.height.to_le_bytes());
    bytes[80..112].copy_from_slice(checkpoint.chain.tip_hash.as_bytes());
    bytes[112..144].copy_from_slice(checkpoint.chain.ledger_root.as_bytes());
}

fn read_checkpoint(bytes: &[u8]) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation: u64::from_le_bytes(array_at(bytes, 0)),
        chain: ChainReplayExpectationV2 {
            config_hash: Hash256::from_bytes(array_at(bytes, 8)),
            genesis_hash: Hash256::from_bytes(array_at(bytes, 40)),
            height: u64::from_le_bytes(array_at(bytes, 72)),
            tip_hash: Hash256::from_bytes(array_at(bytes, 80)),
            ledger_root: Hash256::from_bytes(array_at(bytes, 112)),
        },
    }
}

fn check_identity(
    identity: ManifestIdentityV2,
    checkpoint: ChainReplayExpectationV2,
) -> Result<(), ControlRecordErrorV2> {
    if checkpoint.config_hash != identity.config_hash
        || checkpoint.genesis_hash != identity.genesis_hash
    {
        return Err(ControlRecordErrorV2::WrongIdentity);
    }
    Ok(())
}

fn array_at<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    let mut value = [0; N];
    value.copy_from_slice(&bytes[offset..offset + N]);
    value
}

fn checksum(domain: &[u8], prefix: &[u8]) -> Hash256 {
    let mut input = Vec::with_capacity(domain.len() + prefix.len());
    input.extend_from_slice(domain);
    input.extend_from_slice(prefix);
    sha256d(&input)
}

fn check_checksum(domain: &[u8], bytes: &[u8], offset: usize) -> Result<(), ControlRecordErrorV2> {
    if checksum(domain, &bytes[..offset]).as_bytes() != &bytes[offset..] {
        return Err(ControlRecordErrorV2::ChecksumMismatch);
    }
    Ok(())
}

#[derive(Debug)]
pub enum ControlRecordErrorV2 {
    InvalidLength,
    InvalidMagic,
    UnsupportedVersion(u16),
    UnsupportedKind(u16),
    ChecksumMismatch,
    LengthOverflow,
    HistoryLengthOutOfBounds,
    WrongIdentity,
    WrongCurrentCheckpoint,
    NonCanonicalRecord,
    ConflictingConfirmed,
    Selection(ReplacementSelectionErrorV2),
    Plan(ReplacementPlanErrorV2),
    History(HistoryErrorV2),
    Protocol(ProtocolErrorV2),
    Recovery(ReplacementRecoveryErrorV2),
}

macro_rules! from_error {
    ($kind:ident, $ty:ty) => {
        impl From<$ty> for ControlRecordErrorV2 {
            fn from(error: $ty) -> Self {
                Self::$kind(error)
            }
        }
    };
}
from_error!(Selection, ReplacementSelectionErrorV2);
from_error!(Plan, ReplacementPlanErrorV2);
from_error!(History, HistoryErrorV2);
from_error!(Protocol, ProtocolErrorV2);
from_error!(Recovery, ReplacementRecoveryErrorV2);

impl fmt::Display for ControlRecordErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 node control record rejected: {self:?}")
    }
}

impl std::error::Error for ControlRecordErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Selection(error) => Some(error),
            Self::Plan(error) => Some(error),
            Self::History(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::Recovery(error) => Some(error),
            _ => None,
        }
    }
}
