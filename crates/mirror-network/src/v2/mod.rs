//! Bounded V2 history framing and complete in-memory candidate reconstruction.
//!
//! The caller independently chooses configuration/genesis. A peer's endpoint is
//! a transfer-consistency claim, never canonical selection or freshness authority.
//! Every block is actually accepted from genesis before a completed candidate is
//! returned. Partial candidates are private, and every receiving error consumes
//! continuation authority. This component opens no sockets and changes no store.

use std::{fmt, iter::FusedIterator};

use mirror_chain::v2::{
    ChainErrorV2, ChainV2, ManifestErrorV2, ManifestIdentityV2, NetworkManifestV2,
};
use mirror_core::v2::{BlockErrorV2, BlockV2};
use mirror_protocol::v2::{
    BLOCK_HEIGHT_LEN_V2, FRAME_HEADER_LEN_V2, FrameHeaderV2, FrameKindV2, FrameRefV2,
    HistorySummaryV2, MAX_HISTORY_FRAMES_V2, MAX_HISTORY_WIRE_BYTES_V2, MIN_BLOCK_BYTES_V2,
    ProtocolErrorV2, decode_payload_v2, encode_frame_v2,
};

/// One transfer, with no public access to the partially executed chain.
///
/// The header/payload split permits a transport to reject a length or phase before
/// allocating a payload. Callers must retain the returned owner on each success.
/// A rejected header, payload or frame destroys this attempt, including its partial
/// candidate. A transport must close/discard a failed attempt; no implicit retry or
/// resynchronization scans arbitrary incoming bytes for a new magic value.
///
/// ```compile_fail
/// use mirror_chain::v2::ManifestIdentityV2;
/// use mirror_network::v2::HistoryReceiverV2;
/// let identity = ManifestIdentityV2 {
///     config_hash: Default::default(), genesis_hash: Default::default(),
/// };
/// let receiver = HistoryReceiverV2::new(identity);
/// let _failed_attempt = receiver.accept_frame(&[]);
/// // Even a rejected frame consumes continuation; there is no old receiver.
/// let _partial = receiver.finish();
/// ```
#[derive(Debug)]
pub struct HistoryReceiverV2 {
    identity: ManifestIdentityV2,
    phase: Phase,
    frame_count: usize,
    wire_bytes: usize,
}

#[derive(Debug)]
enum Phase {
    Begin,
    Manifest(HistorySummaryV2),
    Genesis {
        summary: HistorySummaryV2,
        manifest: NetworkManifestV2,
    },
    Blocks {
        summary: HistorySummaryV2,
        chain: ChainV2,
    },
    End {
        summary: HistorySummaryV2,
        chain: ChainV2,
    },
    Complete {
        summary: HistorySummaryV2,
        chain: ChainV2,
    },
}

impl Phase {
    fn expected_kind(&self) -> Result<FrameKindV2, HistoryErrorV2> {
        match self {
            Self::Begin => Ok(FrameKindV2::Begin),
            Self::Manifest(_) => Ok(FrameKindV2::Manifest),
            Self::Genesis { .. } | Self::Blocks { .. } => Ok(FrameKindV2::Block),
            Self::End { .. } => Ok(FrameKindV2::End),
            Self::Complete { .. } => Err(HistoryErrorV2::AlreadyComplete),
        }
    }

    fn summary(&self) -> Option<HistorySummaryV2> {
        match self {
            Self::Begin => None,
            Self::Manifest(summary)
            | Self::Genesis { summary, .. }
            | Self::Blocks { summary, .. }
            | Self::End { summary, .. }
            | Self::Complete { summary, .. } => Some(*summary),
        }
    }
}

/// Private continuation paired with a checked fixed header. The length exposed
/// here has passed version/kind, phase, configured and remaining-transfer bounds.
/// An I/O failure may simply drop this value; no partial chain is returned.
#[derive(Debug)]
pub struct PendingHistoryFrameV2 {
    receiver: HistoryReceiverV2,
    header: FrameHeaderV2,
}

/// A complete, actually executed candidate plus transfer accounting.
/// This is not a canonical choice, global latest tip or finality certificate.
#[derive(Debug)]
pub struct ReceivedCandidateV2 {
    chain: ChainV2,
    summary: HistorySummaryV2,
    frame_count: usize,
    wire_bytes: usize,
}

impl ReceivedCandidateV2 {
    pub const fn chain(&self) -> &ChainV2 {
        &self.chain
    }

    pub fn into_chain(self) -> ChainV2 {
        self.chain
    }

    /// The peer's captured claim, now checked against actual complete execution.
    pub const fn summary(&self) -> HistorySummaryV2 {
        self.summary
    }

    pub const fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Exactly one accepted Begin, manifest, all blocks and End, including headers.
    pub const fn wire_bytes(&self) -> usize {
        self.wire_bytes
    }
}

impl HistoryReceiverV2 {
    /// Independently select the network; never derive this from peer frames.
    pub const fn new(identity: ManifestIdentityV2) -> Self {
        Self {
            identity,
            phase: Phase::Begin,
            frame_count: 0,
            wire_bytes: 0,
        }
    }

    /// Consume one exact fixed header before a transport allocates its payload.
    /// Header refusal, like payload refusal, consumes the entire attempt.
    pub fn begin_frame(mut self, bytes: &[u8]) -> Result<PendingHistoryFrameV2, HistoryErrorV2> {
        let expected = self.phase.expected_kind()?;
        let header = FrameHeaderV2::decode(bytes)?;
        if header.kind() != expected {
            return Err(HistoryErrorV2::UnexpectedFrame {
                expected,
                got: header.kind(),
            });
        }
        let len = header.payload_len();
        match &self.phase {
            Phase::Manifest(summary) => {
                check_length(len, summary.manifest_bytes as usize)?;
            }
            Phase::Genesis { summary, manifest } => {
                // Accepted genesis is exactly the empty canonical MRB2 block.
                check_length(len, BLOCK_HEIGHT_LEN_V2 + MIN_BLOCK_BYTES_V2)?;
                check_block_budget(*summary, manifest, 0, 0, len)?;
            }
            Phase::Blocks { summary, chain } => {
                check_block_budget(
                    *summary,
                    chain.manifest(),
                    chain.blocks().len(),
                    chain.encoded_bytes(),
                    len,
                )?;
            }
            _ => {}
        }
        let frames = self
            .frame_count
            .checked_add(1)
            .ok_or(HistoryErrorV2::LengthOverflow)?;
        let frame_bytes = FRAME_HEADER_LEN_V2
            .checked_add(len)
            .ok_or(HistoryErrorV2::LengthOverflow)?;
        let wire_bytes = self
            .wire_bytes
            .checked_add(frame_bytes)
            .ok_or(HistoryErrorV2::LengthOverflow)?;
        if frames > MAX_HISTORY_FRAMES_V2 {
            return Err(HistoryErrorV2::FrameBudget);
        }
        if wire_bytes > MAX_HISTORY_WIRE_BYTES_V2 {
            return Err(HistoryErrorV2::WireBudget);
        }
        if let Some(summary) = self.phase.summary() {
            if frames > summary.block_count as usize + 3 {
                return Err(HistoryErrorV2::FrameBudget);
            }
            if wire_bytes > summary.wire_len()? {
                return Err(HistoryErrorV2::WireBudget);
            }
        }
        self.frame_count = frames;
        self.wire_bytes = wire_bytes;
        Ok(PendingHistoryFrameV2 {
            receiver: self,
            header,
        })
    }

    /// Convenience for an already bounded complete frame. Socket adapters must
    /// use begin_frame before allocating/reading an attacker-declared payload.
    pub fn accept_frame(self, bytes: &[u8]) -> Result<Self, HistoryErrorV2> {
        let (header, payload) = bytes
            .split_at_checked(FRAME_HEADER_LEN_V2)
            .ok_or(HistoryErrorV2::IncompleteFrame)?;
        self.begin_frame(header)?.accept_payload(payload)
    }

    /// Expose only a terminally confirmed, completely executed candidate.
    pub fn finish(self) -> Result<ReceivedCandidateV2, HistoryErrorV2> {
        match self.phase {
            Phase::Complete { summary, chain } => Ok(ReceivedCandidateV2 {
                chain,
                summary,
                frame_count: self.frame_count,
                wire_bytes: self.wire_bytes,
            }),
            _ => Err(HistoryErrorV2::IncompleteHistory),
        }
    }
}

impl PendingHistoryFrameV2 {
    pub const fn payload_len(&self) -> usize {
        self.header.payload_len()
    }

    /// Decode exact payload bytes without copying them, then execute the authorized
    /// next phase. No malformed frame can be skipped while retaining this attempt.
    pub fn accept_payload(self, bytes: &[u8]) -> Result<HistoryReceiverV2, HistoryErrorV2> {
        let frame = decode_payload_v2(self.header, bytes)?;
        let mut receiver = self.receiver;
        receiver.phase = match (receiver.phase, frame) {
            (Phase::Begin, FrameRefV2::Begin(summary)) => {
                if summary.config_hash != receiver.identity.config_hash
                    || summary.genesis_hash != receiver.identity.genesis_hash
                {
                    return Err(HistoryErrorV2::WrongNetwork);
                }
                Phase::Manifest(summary)
            }
            (Phase::Manifest(summary), FrameRefV2::Manifest(bytes)) => {
                let manifest = NetworkManifestV2::decode_for(bytes, receiver.identity)?;
                // Genesis is fixed-size. Future children must fit this actual
                // configuration, not merely the looser absolute wire ceiling.
                let max = u64::from(summary.block_count - 1)
                    .checked_mul(u64::from(manifest.config().limits().max_block_bytes))
                    .and_then(|n| n.checked_add(MIN_BLOCK_BYTES_V2 as u64))
                    .ok_or(HistoryErrorV2::LengthOverflow)?;
                if summary.block_bytes > max {
                    return Err(HistoryErrorV2::BlockBytesMismatch);
                }
                Phase::Genesis { summary, manifest }
            }
            (Phase::Genesis { summary, manifest }, FrameRefV2::Block { height, bytes }) => {
                check_height(height, 0)?;
                let block = BlockV2::decode(manifest.config(), bytes)?;
                check_height(block.header().height(), height)?;
                let chain = ChainV2::from_genesis(&manifest, block)?;
                after_block(summary, chain)?
            }
            (Phase::Blocks { summary, mut chain }, FrameRefV2::Block { height, bytes }) => {
                check_height(height, chain.blocks().len() as u64)?;
                let block = BlockV2::decode(chain.config(), bytes)?;
                check_height(block.header().height(), height)?;
                chain.append(block)?;
                after_block(summary, chain)?
            }
            (Phase::End { summary, chain }, FrameRefV2::End(end)) => {
                if end != summary {
                    return Err(HistoryErrorV2::ChangedSummary);
                }
                check_complete_claim(&chain, summary)?;
                if receiver.frame_count != summary.block_count as usize + 3
                    || receiver.wire_bytes != summary.wire_len()?
                {
                    return Err(HistoryErrorV2::AccountingMismatch);
                }
                Phase::Complete { summary, chain }
            }
            // The private pending header is bound to the exact allowed kind.
            _ => return Err(HistoryErrorV2::UnexpectedPhase),
        };
        Ok(receiver)
    }
}

fn check_length(got: usize, expected: usize) -> Result<(), HistoryErrorV2> {
    if got == expected {
        Ok(())
    } else {
        Err(HistoryErrorV2::WrongPayloadLength { expected, got })
    }
}

fn check_height(got: u64, expected: u64) -> Result<(), HistoryErrorV2> {
    if got == expected {
        Ok(())
    } else {
        Err(HistoryErrorV2::WrongBlockHeight { expected, got })
    }
}

fn check_block_budget(
    summary: HistorySummaryV2,
    manifest: &NetworkManifestV2,
    accepted_count: usize,
    accepted_bytes: usize,
    payload_len: usize,
) -> Result<(), HistoryErrorV2> {
    let len = payload_len
        .checked_sub(BLOCK_HEIGHT_LEN_V2)
        .ok_or(HistoryErrorV2::LengthOverflow)?;
    let max = manifest.config().limits().max_block_bytes as usize;
    if len > max {
        return Err(HistoryErrorV2::BlockTooLarge { len, max });
    }
    let remaining = (summary.block_count as usize)
        .checked_sub(accepted_count + 1)
        .ok_or(HistoryErrorV2::FrameBudget)?;
    // Use u64: the product can exceed 32-bit usize before the aggregate cap.
    let prospective = (accepted_bytes as u64)
        .checked_add(len as u64)
        .ok_or(HistoryErrorV2::LengthOverflow)?;
    let minimum = prospective
        .checked_add(
            (remaining as u64)
                .checked_mul(MIN_BLOCK_BYTES_V2 as u64)
                .ok_or(HistoryErrorV2::LengthOverflow)?,
        )
        .ok_or(HistoryErrorV2::LengthOverflow)?;
    let maximum = prospective
        .checked_add(
            (remaining as u64)
                .checked_mul(max as u64)
                .ok_or(HistoryErrorV2::LengthOverflow)?,
        )
        .ok_or(HistoryErrorV2::LengthOverflow)?;
    if summary.block_bytes < minimum || summary.block_bytes > maximum {
        return Err(HistoryErrorV2::BlockBytesMismatch);
    }
    Ok(())
}

fn after_block(summary: HistorySummaryV2, chain: ChainV2) -> Result<Phase, HistoryErrorV2> {
    if chain.blocks().len() == summary.block_count as usize {
        // Reject a false advertised terminal as soon as the last block is known.
        check_complete_claim(&chain, summary)?;
        Ok(Phase::End { summary, chain })
    } else {
        Ok(Phase::Blocks { summary, chain })
    }
}

fn check_complete_claim(chain: &ChainV2, summary: HistorySummaryV2) -> Result<(), HistoryErrorV2> {
    let computed = chain.checkpoint();
    if chain.blocks().len() != summary.block_count as usize
        || chain.encoded_bytes() as u64 != summary.block_bytes
    {
        return Err(HistoryErrorV2::BlockBytesMismatch);
    }
    if computed.config_hash != summary.config_hash
        || computed.genesis_hash != summary.genesis_hash
        || computed.height != summary.height
        || computed.tip_hash != summary.tip_hash
        || computed.ledger_root != summary.ledger_root
    {
        return Err(HistoryErrorV2::EndpointMismatch);
    }
    Ok(())
}

/// A frame-at-a-time sender borrowing one immutable accepted branch.
/// At most one emitted frame and temporary encoded block are produced per next().
/// No socket, selected-store mutation, unbounded output buffer or new fork choice.
#[derive(Debug)]
pub struct HistoryFramesV2<'a> {
    chain: &'a ChainV2,
    manifest: Vec<u8>,
    summary: HistorySummaryV2,
    next: usize,
    failed: bool,
}

impl<'a> HistoryFramesV2<'a> {
    pub fn new(chain: &'a ChainV2) -> Result<Self, HistoryErrorV2> {
        let manifest = chain.manifest().encode()?;
        let checkpoint = chain.checkpoint();
        let summary = HistorySummaryV2 {
            config_hash: checkpoint.config_hash,
            genesis_hash: checkpoint.genesis_hash,
            height: checkpoint.height,
            tip_hash: checkpoint.tip_hash,
            ledger_root: checkpoint.ledger_root,
            block_count: u32::try_from(chain.blocks().len())
                .map_err(|_| HistoryErrorV2::LengthOverflow)?,
            block_bytes: u64::try_from(chain.encoded_bytes())
                .map_err(|_| HistoryErrorV2::LengthOverflow)?,
            manifest_bytes: u32::try_from(manifest.len())
                .map_err(|_| HistoryErrorV2::LengthOverflow)?,
        };
        summary.validate()?;
        Ok(Self {
            chain,
            manifest,
            summary,
            next: 0,
            failed: false,
        })
    }

    pub const fn summary(&self) -> HistorySummaryV2 {
        self.summary
    }

    fn encode_next(&self) -> Result<Vec<u8>, HistoryErrorV2> {
        match self.next {
            0 => Ok(encode_frame_v2(FrameRefV2::Begin(self.summary))?),
            1 => Ok(encode_frame_v2(FrameRefV2::Manifest(&self.manifest))?),
            index if index == self.chain.blocks().len() + 2 => {
                Ok(encode_frame_v2(FrameRefV2::End(self.summary))?)
            }
            index => {
                let block = &self.chain.blocks()[index - 2];
                let bytes = block.encode(self.chain.config())?;
                Ok(encode_frame_v2(FrameRefV2::Block {
                    height: block.header().height(),
                    bytes: &bytes,
                })?)
            }
        }
    }
}

impl Iterator for HistoryFramesV2<'_> {
    type Item = Result<Vec<u8>, HistoryErrorV2>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.next >= self.chain.blocks().len() + 3 {
            return None;
        }
        let result = self.encode_next();
        if result.is_err() {
            self.failed = true;
        } else {
            self.next += 1;
        }
        Some(result)
    }
}

impl FusedIterator for HistoryFramesV2<'_> {}

#[derive(Debug)]
pub enum HistoryErrorV2 {
    Protocol(ProtocolErrorV2),
    Manifest(ManifestErrorV2),
    Block(BlockErrorV2),
    Chain(ChainErrorV2),
    WrongNetwork,
    UnexpectedFrame {
        expected: FrameKindV2,
        got: FrameKindV2,
    },
    WrongPayloadLength {
        expected: usize,
        got: usize,
    },
    WrongBlockHeight {
        expected: u64,
        got: u64,
    },
    BlockTooLarge {
        len: usize,
        max: usize,
    },
    BlockBytesMismatch,
    EndpointMismatch,
    ChangedSummary,
    AlreadyComplete,
    IncompleteFrame,
    IncompleteHistory,
    FrameBudget,
    WireBudget,
    AccountingMismatch,
    UnexpectedPhase,
    LengthOverflow,
}

impl fmt::Display for HistoryErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 candidate history rejected: {self:?}")
    }
}

impl std::error::Error for HistoryErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            Self::Manifest(error) => Some(error),
            Self::Block(error) => Some(error),
            Self::Chain(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ProtocolErrorV2> for HistoryErrorV2 {
    fn from(error: ProtocolErrorV2) -> Self {
        Self::Protocol(error)
    }
}
impl From<ManifestErrorV2> for HistoryErrorV2 {
    fn from(error: ManifestErrorV2) -> Self {
        Self::Manifest(error)
    }
}
impl From<BlockErrorV2> for HistoryErrorV2 {
    fn from(error: BlockErrorV2) -> Self {
        Self::Block(error)
    }
}
impl From<ChainErrorV2> for HistoryErrorV2 {
    fn from(error: ChainErrorV2) -> Self {
        Self::Chain(error)
    }
}
