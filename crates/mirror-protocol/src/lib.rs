//! Mirror peer-to-peer wire protocol.
//!
//! This crate defines the exact binary messages exchanged between
//! independent Mirror nodes. Transport itself lives in `mirror-network`.

use core::fmt;

use mirror_crypto::Hash256;

/// Identifies Mirror network traffic.
///
/// A random TCP client sending arbitrary bytes should not accidentally
/// be interpreted as a Mirror peer.
pub const NETWORK_MAGIC: [u8; 4] = *b"MIRR";

/// Current Mirror P2P protocol version.
pub const PROTOCOL_VERSION: u16 = 1;

/// Fixed wire-frame header size.
///
/// ```text
/// magic             4 bytes
/// protocol_version  2 bytes LE
/// message_kind      2 bytes LE
/// payload_length    4 bytes LE
/// --------------------------
/// total            12 bytes
/// ```
pub const FRAME_HEADER_LEN: usize = 12;

/// Defensive maximum for one P2P message payload.
///
/// Block-size policy will eventually have its own consensus limit.
/// This is currently a transport safety bound.
pub const MAX_FRAME_PAYLOAD_LEN: usize = 4 * 1024 * 1024;

pub const MESSAGE_KIND_HELLO: u16 = 1;

/// Exact encoded Hello payload length.
///
/// ```text
/// chain_id       4 bytes
/// genesis_hash  32 bytes
/// best_height    8 bytes
/// tip_hash      32 bytes
/// --------------------
/// total         76 bytes
/// ```
pub const HELLO_PAYLOAD_LEN: usize = 76;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelloMessage {
    chain_id: u32,
    genesis_hash: Hash256,
    best_height: u64,
    tip_hash: Hash256,
}

impl HelloMessage {
    pub const fn new(
        chain_id: u32,
        genesis_hash: Hash256,
        best_height: u64,
        tip_hash: Hash256,
    ) -> Self {
        Self {
            chain_id,
            genesis_hash,
            best_height,
            tip_hash,
        }
    }

    pub const fn chain_id(&self) -> u32 {
        self.chain_id
    }

    pub const fn genesis_hash(&self) -> Hash256 {
        self.genesis_hash
    }

    pub const fn best_height(&self) -> u64 {
        self.best_height
    }

    pub const fn tip_hash(&self) -> Hash256 {
        self.tip_hash
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireMessage {
    Hello(HelloMessage),
}

/// Encode one complete Mirror P2P message.
pub fn encode_message(message: &WireMessage) -> Result<Vec<u8>, ProtocolError> {
    let (kind, payload) = match message {
        WireMessage::Hello(hello) => (MESSAGE_KIND_HELLO, encode_hello(hello)),
    };

    if payload.len() > MAX_FRAME_PAYLOAD_LEN {
        return Err(ProtocolError::PayloadTooLarge {
            len: payload.len(),
            max: MAX_FRAME_PAYLOAD_LEN,
        });
    }

    let payload_len = u32::try_from(payload.len()).map_err(|_| ProtocolError::LengthOverflow)?;

    let mut bytes = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());

    bytes.extend_from_slice(&NETWORK_MAGIC);

    bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());

    bytes.extend_from_slice(&kind.to_le_bytes());

    bytes.extend_from_slice(&payload_len.to_le_bytes());

    bytes.extend_from_slice(&payload);

    Ok(bytes)
}

/// Decode one complete Mirror P2P message.
///
/// The input must contain exactly one frame.
pub fn decode_message(bytes: &[u8]) -> Result<WireMessage, ProtocolError> {
    if bytes.len() < FRAME_HEADER_LEN {
        return Err(ProtocolError::UnexpectedEnd);
    }

    let magic: [u8; 4] = bytes[0..4]
        .try_into()
        .map_err(|_| ProtocolError::UnexpectedEnd)?;

    if magic != NETWORK_MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }

    let protocol_version = u16::from_le_bytes(
        bytes[4..6]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    );

    if protocol_version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedProtocolVersion {
            expected: PROTOCOL_VERSION,
            got: protocol_version,
        });
    }

    let message_kind = u16::from_le_bytes(
        bytes[6..8]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    );

    let payload_len = u32::from_le_bytes(
        bytes[8..12]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    ) as usize;

    if payload_len > MAX_FRAME_PAYLOAD_LEN {
        return Err(ProtocolError::PayloadTooLarge {
            len: payload_len,
            max: MAX_FRAME_PAYLOAD_LEN,
        });
    }

    let expected_len = FRAME_HEADER_LEN
        .checked_add(payload_len)
        .ok_or(ProtocolError::LengthOverflow)?;

    if bytes.len() < expected_len {
        return Err(ProtocolError::UnexpectedEnd);
    }

    if bytes.len() > expected_len {
        return Err(ProtocolError::TrailingBytes);
    }

    let payload = &bytes[FRAME_HEADER_LEN..];

    match message_kind {
        MESSAGE_KIND_HELLO => Ok(WireMessage::Hello(decode_hello(payload)?)),

        other => Err(ProtocolError::UnknownMessageKind(other)),
    }
}

fn encode_hello(hello: &HelloMessage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HELLO_PAYLOAD_LEN);

    bytes.extend_from_slice(&hello.chain_id.to_le_bytes());

    bytes.extend_from_slice(hello.genesis_hash.as_bytes());

    bytes.extend_from_slice(&hello.best_height.to_le_bytes());

    bytes.extend_from_slice(hello.tip_hash.as_bytes());

    debug_assert_eq!(bytes.len(), HELLO_PAYLOAD_LEN);

    bytes
}

fn decode_hello(payload: &[u8]) -> Result<HelloMessage, ProtocolError> {
    if payload.len() != HELLO_PAYLOAD_LEN {
        return Err(ProtocolError::InvalidPayloadLength {
            kind: MESSAGE_KIND_HELLO,
            expected: HELLO_PAYLOAD_LEN,
            got: payload.len(),
        });
    }

    let chain_id = u32::from_le_bytes(
        payload[0..4]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    );

    let genesis_hash = Hash256::from_bytes(
        payload[4..36]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    );

    let best_height = u64::from_le_bytes(
        payload[36..44]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    );

    let tip_hash = Hash256::from_bytes(
        payload[44..76]
            .try_into()
            .map_err(|_| ProtocolError::UnexpectedEnd)?,
    );

    Ok(HelloMessage::new(
        chain_id,
        genesis_hash,
        best_height,
        tip_hash,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    UnexpectedEnd,
    InvalidMagic,

    UnsupportedProtocolVersion {
        expected: u16,
        got: u16,
    },

    UnknownMessageKind(u16),

    InvalidPayloadLength {
        kind: u16,
        expected: usize,
        got: usize,
    },

    PayloadTooLarge {
        len: usize,
        max: usize,
    },

    TrailingBytes,
    LengthOverflow,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => {
                write!(f, "unexpected end of network message")
            }

            Self::InvalidMagic => {
                write!(f, "invalid Mirror network magic")
            }

            Self::UnsupportedProtocolVersion { expected, got } => {
                write!(
                    f,
                    "unsupported protocol version: expected {expected}, got {got}"
                )
            }

            Self::UnknownMessageKind(kind) => {
                write!(f, "unknown Mirror message kind {kind}")
            }

            Self::InvalidPayloadLength {
                kind,
                expected,
                got,
            } => {
                write!(
                    f,
                    "invalid payload length for message {kind}: expected {expected}, got {got}"
                )
            }

            Self::PayloadTooLarge { len, max } => {
                write!(f, "network payload too large: {len} bytes, maximum {max}")
            }

            Self::TrailingBytes => {
                write!(f, "unexpected bytes after network frame")
            }

            Self::LengthOverflow => {
                write!(f, "network message length overflow")
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello() -> WireMessage {
        WireMessage::Hello(HelloMessage::new(
            1,
            Hash256::from_bytes([0x11; 32]),
            42,
            Hash256::from_bytes([0x22; 32]),
        ))
    }

    #[test]
    fn hello_round_trip() {
        let original = hello();

        let encoded = encode_message(&original).unwrap();

        let decoded = decode_message(&encoded).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn hello_frame_has_known_size() {
        let encoded = encode_message(&hello()).unwrap();

        assert_eq!(encoded.len(), FRAME_HEADER_LEN + HELLO_PAYLOAD_LEN);

        assert_eq!(encoded.len(), 88);
    }

    #[test]
    fn invalid_magic_is_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        encoded[0] ^= 0xff;

        assert_eq!(decode_message(&encoded), Err(ProtocolError::InvalidMagic));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        encoded[4..6].copy_from_slice(&999u16.to_le_bytes());

        assert_eq!(
            decode_message(&encoded),
            Err(ProtocolError::UnsupportedProtocolVersion {
                expected: PROTOCOL_VERSION,
                got: 999,
            })
        );
    }

    #[test]
    fn unknown_message_kind_is_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        encoded[6..8].copy_from_slice(&999u16.to_le_bytes());

        assert_eq!(
            decode_message(&encoded),
            Err(ProtocolError::UnknownMessageKind(999))
        );
    }

    #[test]
    fn truncated_frame_is_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        encoded.pop();

        assert_eq!(decode_message(&encoded), Err(ProtocolError::UnexpectedEnd));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        encoded.push(0);

        assert_eq!(decode_message(&encoded), Err(ProtocolError::TrailingBytes));
    }

    #[test]
    fn oversized_declared_payload_is_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        let oversized = (MAX_FRAME_PAYLOAD_LEN as u32) + 1;

        encoded[8..12].copy_from_slice(&oversized.to_le_bytes());

        assert_eq!(
            decode_message(&encoded),
            Err(ProtocolError::PayloadTooLarge {
                len: MAX_FRAME_PAYLOAD_LEN + 1,
                max: MAX_FRAME_PAYLOAD_LEN,
            })
        );
    }

    #[test]
    fn wrong_hello_payload_length_is_rejected() {
        let mut encoded = encode_message(&hello()).unwrap();

        let smaller = (HELLO_PAYLOAD_LEN - 1) as u32;

        encoded[8..12].copy_from_slice(&smaller.to_le_bytes());

        encoded.pop();

        assert_eq!(
            decode_message(&encoded),
            Err(ProtocolError::InvalidPayloadLength {
                kind: MESSAGE_KIND_HELLO,
                expected: HELLO_PAYLOAD_LEN,
                got: HELLO_PAYLOAD_LEN - 1,
            })
        );
    }
}
