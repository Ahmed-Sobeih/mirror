//! Bounded framing for one finite V2 candidate-history transfer.
//!
//! Every field is untrusted peer data. This codec checks only canonical framing
//! and absolute development-profile bounds. It does not verify a manifest, a
//! block, a signature, an endpoint, network identity, freshness or fork choice.
//! A receiver must independently select the network and execute the full history.
//! Legacy MIRR/version-1 frames remain a separate protocol.

use std::fmt;

use mirror_crypto::Hash256;

pub const HISTORY_MAGIC_V2: [u8; 4] = *b"MPN2";
pub const HISTORY_PROTOCOL_VERSION_V2: u16 = 2;
pub const FRAME_HEADER_LEN_V2: usize = 12;
pub const HISTORY_SUMMARY_LEN_V2: usize = 152;
pub const BLOCK_HEIGHT_LEN_V2: usize = 8;
pub const MAX_HISTORY_BLOCKS_V2: usize = 4_096;
pub const MAX_HISTORY_BLOCK_BYTES_V2: usize = 16_777_216;
pub const MAX_MANIFEST_BYTES_V2: usize = 2_781_636;
pub const MAX_BLOCK_BYTES_V2: usize = 1_048_576;
pub const MIN_BLOCK_BYTES_V2: usize = 168;
pub const MIN_MANIFEST_BYTES_V2: usize = 46;
pub const MAX_HISTORY_FRAMES_V2: usize = MAX_HISTORY_BLOCKS_V2 + 3;
/// Two summaries, one manifest, every complete MRB2, headers and height prefixes.
/// This is an encoded transfer bound, not resident memory or elapsed time.
pub const MAX_HISTORY_WIRE_BYTES_V2: usize = 19_641_112;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum FrameKindV2 {
    Begin = 1,
    Manifest = 2,
    Block = 3,
    End = 4,
}

impl FrameKindV2 {
    fn decode(value: u16) -> Result<Self, ProtocolErrorV2> {
        match value {
            1 => Ok(Self::Begin),
            2 => Ok(Self::Manifest),
            3 => Ok(Self::Block),
            4 => Ok(Self::End),
            other => Err(ProtocolErrorV2::UnknownKind(other)),
        }
    }

    const fn payload_bounds(self) -> (usize, usize) {
        match self {
            Self::Begin | Self::End => (HISTORY_SUMMARY_LEN_V2, HISTORY_SUMMARY_LEN_V2),
            Self::Manifest => (MIN_MANIFEST_BYTES_V2, MAX_MANIFEST_BYTES_V2),
            Self::Block => (
                BLOCK_HEIGHT_LEN_V2 + MIN_BLOCK_BYTES_V2,
                BLOCK_HEIGHT_LEN_V2 + MAX_BLOCK_BYTES_V2,
            ),
        }
    }
}

/// UNTRUSTED transfer-consistency claims, never a selected replay checkpoint.
/// Matching these fields does not establish latestness, finality or adoption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistorySummaryV2 {
    pub config_hash: Hash256,
    pub genesis_hash: Hash256,
    pub height: u64,
    pub tip_hash: Hash256,
    pub ledger_root: Hash256,
    pub block_count: u32,
    pub block_bytes: u64,
    pub manifest_bytes: u32,
}

impl HistorySummaryV2 {
    /// Check only the absolute finite transfer shape. Network and actual block
    /// validation remain mandatory downstream, including configured lower caps.
    pub fn validate(&self) -> Result<(), ProtocolErrorV2> {
        if self.block_count == 0 || u64::from(self.block_count) > MAX_HISTORY_BLOCKS_V2 as u64 {
            return Err(ProtocolErrorV2::InvalidBlockCount(self.block_count));
        }
        if self.height.checked_add(1) != Some(u64::from(self.block_count)) {
            return Err(ProtocolErrorV2::InvalidHeight {
                height: self.height,
                block_count: self.block_count,
            });
        }
        if u64::from(self.manifest_bytes) < MIN_MANIFEST_BYTES_V2 as u64
            || u64::from(self.manifest_bytes) > MAX_MANIFEST_BYTES_V2 as u64
        {
            return Err(ProtocolErrorV2::InvalidManifestLength(self.manifest_bytes));
        }
        let count = u64::from(self.block_count);
        let minimum = count
            .checked_mul(MIN_BLOCK_BYTES_V2 as u64)
            .ok_or(ProtocolErrorV2::LengthOverflow)?;
        let maximum = count
            .checked_mul(MAX_BLOCK_BYTES_V2 as u64)
            .ok_or(ProtocolErrorV2::LengthOverflow)?
            .min(MAX_HISTORY_BLOCK_BYTES_V2 as u64);
        if self.block_bytes < minimum || self.block_bytes > maximum {
            return Err(ProtocolErrorV2::InvalidBlockBytes(self.block_bytes));
        }
        Ok(())
    }

    /// Exact claimed transfer length, including Begin/End and every frame header.
    pub fn wire_len(&self) -> Result<usize, ProtocolErrorV2> {
        self.validate()?;
        let blocks =
            usize::try_from(self.block_count).map_err(|_| ProtocolErrorV2::LengthOverflow)?;
        let block_bytes =
            usize::try_from(self.block_bytes).map_err(|_| ProtocolErrorV2::LengthOverflow)?;
        let manifest_bytes =
            usize::try_from(self.manifest_bytes).map_err(|_| ProtocolErrorV2::LengthOverflow)?;
        let result = blocks
            .checked_mul(FRAME_HEADER_LEN_V2 + BLOCK_HEIGHT_LEN_V2)
            .and_then(|n| n.checked_add(3 * FRAME_HEADER_LEN_V2 + 2 * HISTORY_SUMMARY_LEN_V2))
            .and_then(|n| n.checked_add(block_bytes))
            .and_then(|n| n.checked_add(manifest_bytes))
            .ok_or(ProtocolErrorV2::LengthOverflow)?;
        if result > MAX_HISTORY_WIRE_BYTES_V2 {
            return Err(ProtocolErrorV2::LengthOverflow);
        }
        Ok(result)
    }

    pub fn encode(&self) -> Result<[u8; HISTORY_SUMMARY_LEN_V2], ProtocolErrorV2> {
        self.validate()?;
        let mut bytes = [0; HISTORY_SUMMARY_LEN_V2];
        bytes[0..32].copy_from_slice(self.config_hash.as_bytes());
        bytes[32..64].copy_from_slice(self.genesis_hash.as_bytes());
        bytes[64..72].copy_from_slice(&self.height.to_le_bytes());
        bytes[72..104].copy_from_slice(self.tip_hash.as_bytes());
        bytes[104..136].copy_from_slice(self.ledger_root.as_bytes());
        bytes[136..140].copy_from_slice(&self.block_count.to_le_bytes());
        bytes[140..148].copy_from_slice(&self.block_bytes.to_le_bytes());
        bytes[148..152].copy_from_slice(&self.manifest_bytes.to_le_bytes());
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolErrorV2> {
        if bytes.len() != HISTORY_SUMMARY_LEN_V2 {
            return Err(ProtocolErrorV2::InvalidSummaryLength(bytes.len()));
        }
        let result = Self {
            config_hash: Hash256::from_bytes(array(&bytes[0..32])?),
            genesis_hash: Hash256::from_bytes(array(&bytes[32..64])?),
            height: u64::from_le_bytes(array(&bytes[64..72])?),
            tip_hash: Hash256::from_bytes(array(&bytes[72..104])?),
            ledger_root: Hash256::from_bytes(array(&bytes[104..136])?),
            block_count: u32::from_le_bytes(array(&bytes[136..140])?),
            block_bytes: u64::from_le_bytes(array(&bytes[140..148])?),
            manifest_bytes: u32::from_le_bytes(array(&bytes[148..152])?),
        };
        result.validate()?;
        Ok(result)
    }
}

/// A structurally checked header. Its private fields prevent bypassing per-kind
/// absolute payload limits. The receiver still enforces its current phase and
/// configured/remaining budgets before allocating a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameHeaderV2 {
    kind: FrameKindV2,
    payload_len: u32,
}

impl FrameHeaderV2 {
    pub fn new(kind: FrameKindV2, payload_len: usize) -> Result<Self, ProtocolErrorV2> {
        let (min, max) = kind.payload_bounds();
        if !(min..=max).contains(&payload_len) {
            return Err(ProtocolErrorV2::InvalidPayloadLength {
                kind,
                len: payload_len,
                min,
                max,
            });
        }
        Ok(Self {
            kind,
            payload_len: u32::try_from(payload_len).map_err(|_| ProtocolErrorV2::LengthOverflow)?,
        })
    }

    pub const fn kind(&self) -> FrameKindV2 {
        self.kind
    }

    pub const fn payload_len(&self) -> usize {
        self.payload_len as usize
    }

    pub fn encode(&self) -> [u8; FRAME_HEADER_LEN_V2] {
        let mut bytes = [0; FRAME_HEADER_LEN_V2];
        bytes[0..4].copy_from_slice(&HISTORY_MAGIC_V2);
        bytes[4..6].copy_from_slice(&HISTORY_PROTOCOL_VERSION_V2.to_le_bytes());
        bytes[6..8].copy_from_slice(&(self.kind as u16).to_le_bytes());
        bytes[8..12].copy_from_slice(&self.payload_len.to_le_bytes());
        bytes
    }

    /// Check the complete fixed header before reading or allocating its payload.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolErrorV2> {
        if bytes.len() != FRAME_HEADER_LEN_V2 {
            return Err(ProtocolErrorV2::InvalidHeaderLength(bytes.len()));
        }
        if bytes[0..4] != HISTORY_MAGIC_V2 {
            return Err(ProtocolErrorV2::InvalidMagic);
        }
        let version = u16::from_le_bytes(array(&bytes[4..6])?);
        if version != HISTORY_PROTOCOL_VERSION_V2 {
            return Err(ProtocolErrorV2::UnsupportedVersion(version));
        }
        let kind = FrameKindV2::decode(u16::from_le_bytes(array(&bytes[6..8])?))?;
        let len = usize::try_from(u32::from_le_bytes(array(&bytes[8..12])?))
            .map_err(|_| ProtocolErrorV2::LengthOverflow)?;
        Self::new(kind, len)
    }
}

/// A borrowed, structurally decoded frame. Manifest and block bytes remain
/// unverified. No input-sized allocation is performed by either decoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameRefV2<'a> {
    Begin(HistorySummaryV2),
    Manifest(&'a [u8]),
    Block { height: u64, bytes: &'a [u8] },
    End(HistorySummaryV2),
}

pub fn encode_frame_v2(frame: FrameRefV2<'_>) -> Result<Vec<u8>, ProtocolErrorV2> {
    let (kind, len) = match frame {
        FrameRefV2::Begin(summary) => {
            summary.validate()?;
            (FrameKindV2::Begin, HISTORY_SUMMARY_LEN_V2)
        }
        FrameRefV2::End(summary) => {
            summary.validate()?;
            (FrameKindV2::End, HISTORY_SUMMARY_LEN_V2)
        }
        FrameRefV2::Manifest(bytes) => (FrameKindV2::Manifest, bytes.len()),
        FrameRefV2::Block { height, bytes } => {
            check_block_height(height)?;
            (
                FrameKindV2::Block,
                BLOCK_HEIGHT_LEN_V2
                    .checked_add(bytes.len())
                    .ok_or(ProtocolErrorV2::LengthOverflow)?,
            )
        }
    };
    let header = FrameHeaderV2::new(kind, len)?;
    let total = FRAME_HEADER_LEN_V2
        .checked_add(len)
        .ok_or(ProtocolErrorV2::LengthOverflow)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(total)
        .map_err(|_| ProtocolErrorV2::AllocationFailed)?;
    result.extend_from_slice(&header.encode());
    match frame {
        FrameRefV2::Begin(summary) | FrameRefV2::End(summary) => {
            result.extend_from_slice(&summary.encode()?)
        }
        FrameRefV2::Manifest(bytes) => result.extend_from_slice(bytes),
        FrameRefV2::Block { height, bytes } => {
            result.extend_from_slice(&height.to_le_bytes());
            result.extend_from_slice(bytes);
        }
    }
    Ok(result)
}

/// Decode exactly one complete frame, rejecting both truncation and trailing data.
pub fn decode_frame_v2(bytes: &[u8]) -> Result<FrameRefV2<'_>, ProtocolErrorV2> {
    if bytes.len() < FRAME_HEADER_LEN_V2 {
        return Err(ProtocolErrorV2::TruncatedFrame);
    }
    let header = FrameHeaderV2::decode(&bytes[..FRAME_HEADER_LEN_V2])?;
    decode_payload_v2(header, &bytes[FRAME_HEADER_LEN_V2..])
}

/// Decode exactly the payload described by a previously checked header.
pub fn decode_payload_v2(
    header: FrameHeaderV2,
    payload: &[u8],
) -> Result<FrameRefV2<'_>, ProtocolErrorV2> {
    if payload.len() < header.payload_len() {
        return Err(ProtocolErrorV2::TruncatedFrame);
    }
    if payload.len() > header.payload_len() {
        return Err(ProtocolErrorV2::TrailingBytes);
    }
    match header.kind() {
        FrameKindV2::Begin => Ok(FrameRefV2::Begin(HistorySummaryV2::decode(payload)?)),
        FrameKindV2::End => Ok(FrameRefV2::End(HistorySummaryV2::decode(payload)?)),
        FrameKindV2::Manifest => Ok(FrameRefV2::Manifest(payload)),
        FrameKindV2::Block => {
            let height = u64::from_le_bytes(array(&payload[..BLOCK_HEIGHT_LEN_V2])?);
            check_block_height(height)?;
            Ok(FrameRefV2::Block {
                height,
                bytes: &payload[BLOCK_HEIGHT_LEN_V2..],
            })
        }
    }
}

fn check_block_height(height: u64) -> Result<(), ProtocolErrorV2> {
    if height >= MAX_HISTORY_BLOCKS_V2 as u64 {
        return Err(ProtocolErrorV2::BlockHeightOutOfRange(height));
    }
    Ok(())
}

fn array<const N: usize>(bytes: &[u8]) -> Result<[u8; N], ProtocolErrorV2> {
    bytes
        .try_into()
        .map_err(|_| ProtocolErrorV2::TruncatedFrame)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolErrorV2 {
    InvalidMagic,
    UnsupportedVersion(u16),
    UnknownKind(u16),
    InvalidHeaderLength(usize),
    InvalidSummaryLength(usize),
    InvalidPayloadLength {
        kind: FrameKindV2,
        len: usize,
        min: usize,
        max: usize,
    },
    InvalidBlockCount(u32),
    InvalidHeight {
        height: u64,
        block_count: u32,
    },
    InvalidManifestLength(u32),
    InvalidBlockBytes(u64),
    BlockHeightOutOfRange(u64),
    TruncatedFrame,
    TrailingBytes,
    LengthOverflow,
    AllocationFailed,
}

impl fmt::Display for ProtocolErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 peer history frame rejected: {self:?}")
    }
}

impl std::error::Error for ProtocolErrorV2 {}
