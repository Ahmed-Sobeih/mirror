//! Canonical binary encoding and decoding for complete Mirror blocks.
//!
//! This format is suitable for disk persistence and, later, P2P transport.
//! It does not change the existing transaction IDs or block hashes.

use core::fmt;

use mirror_crypto::{PublicKey, SignatureBytes};

use crate::{
    Address, BLOCK_HEADER_LEN, Block, BlockHeader, SIGNED_TRANSACTION_AUTH_LEN, SignedTransaction,
    SignedTransactionError, TRANSACTION_BODY_FIXED_LEN, TransactionBody,
};

/// Versioned marker for a canonically encoded Mirror block.
pub const BLOCK_ENCODING_MAGIC: [u8; 4] = *b"MRB1";

/// Smallest possible canonical block:
///
/// magic + header + transaction_count
pub const MIN_ENCODED_BLOCK_LEN: usize = BLOCK_ENCODING_MAGIC.len() + BLOCK_HEADER_LEN + 4;

/// Smallest possible signed transaction.
///
/// This is a transaction body with an empty payload plus its
/// public key and signature.
pub const MIN_SIGNED_TRANSACTION_LEN: usize =
    TRANSACTION_BODY_FIXED_LEN + SIGNED_TRANSACTION_AUTH_LEN;

/// Encode a complete Mirror block.
///
/// Layout:
///
/// ```text
/// magic                  4 bytes  "MRB1"
/// header               120 bytes
/// transaction_count      4 bytes  u32 little-endian
///
/// repeated transaction_count times:
///     transaction_len    4 bytes  u32 little-endian
///     transaction        variable canonical SignedTransaction bytes
/// ```
pub fn encode_block(block: &Block) -> Result<Vec<u8>, CodecError> {
    let transaction_count: u32 = block
        .transactions()
        .len()
        .try_into()
        .map_err(|_| CodecError::TooManyTransactions)?;

    let mut encoded_transactions = Vec::with_capacity(block.transactions().len());

    let mut total_len = MIN_ENCODED_BLOCK_LEN;

    for transaction in block.transactions() {
        let encoded = transaction.encode()?;

        let _: u32 = encoded
            .len()
            .try_into()
            .map_err(|_| CodecError::TransactionTooLarge)?;

        total_len = total_len
            .checked_add(4)
            .and_then(|length| length.checked_add(encoded.len()))
            .ok_or(CodecError::LengthOverflow)?;

        encoded_transactions.push(encoded);
    }

    let mut bytes = Vec::with_capacity(total_len);

    bytes.extend_from_slice(&BLOCK_ENCODING_MAGIC);
    bytes.extend_from_slice(&block.header().encode());
    bytes.extend_from_slice(&transaction_count.to_le_bytes());

    for transaction in encoded_transactions {
        let transaction_len =
            u32::try_from(transaction.len()).map_err(|_| CodecError::TransactionTooLarge)?;

        bytes.extend_from_slice(&transaction_len.to_le_bytes());

        bytes.extend_from_slice(&transaction);
    }

    Ok(bytes)
}

/// Decode one complete canonical Mirror block.
///
/// This performs structural decoding only. A decoded block must still
/// pass block integrity, consensus, and state validation before a node
/// accepts it.
pub fn decode_block(bytes: &[u8]) -> Result<Block, CodecError> {
    let mut decoder = Decoder::new(bytes);

    let magic = decoder.take_array::<4>()?;

    if magic != BLOCK_ENCODING_MAGIC {
        return Err(CodecError::InvalidBlockMagic);
    }

    let header = decode_header(&mut decoder)?;

    let transaction_count = decoder.read_u32()? as usize;

    // Do not trust an externally supplied transaction count when
    // reserving memory. Capacity is bounded by bytes actually present.
    let maximum_possible = decoder.remaining() / (4 + MIN_SIGNED_TRANSACTION_LEN);

    let mut transactions = Vec::with_capacity(transaction_count.min(maximum_possible));

    for _ in 0..transaction_count {
        let transaction_len = decoder.read_u32()? as usize;

        if transaction_len < MIN_SIGNED_TRANSACTION_LEN {
            return Err(CodecError::InvalidTransactionLength);
        }

        let transaction_bytes = decoder.take_slice(transaction_len)?;

        transactions.push(decode_signed_transaction(transaction_bytes)?);
    }

    if decoder.remaining() != 0 {
        return Err(CodecError::TrailingBytes);
    }

    Ok(Block::from_parts(header, transactions))
}

fn decode_header(decoder: &mut Decoder<'_>) -> Result<BlockHeader, CodecError> {
    let version = decoder.read_u32()?;

    let previous_block_hash = mirror_crypto::Hash256::from_bytes(decoder.take_array::<32>()?);

    let transaction_root = mirror_crypto::Hash256::from_bytes(decoder.take_array::<32>()?);

    let state_root = mirror_crypto::Hash256::from_bytes(decoder.take_array::<32>()?);

    let timestamp = decoder.read_u64()?;
    let pow_bits = decoder.read_u32()?;
    let nonce = decoder.read_u64()?;

    Ok(BlockHeader::new(
        version,
        previous_block_hash,
        transaction_root,
        state_root,
        timestamp,
        pow_bits,
        nonce,
    ))
}

fn decode_signed_transaction(bytes: &[u8]) -> Result<SignedTransaction, CodecError> {
    if bytes.len() < MIN_SIGNED_TRANSACTION_LEN {
        return Err(CodecError::InvalidTransactionLength);
    }

    let mut decoder = Decoder::new(bytes);

    let version = decoder.read_u16()?;
    let kind = decoder.read_u16()?;
    let chain_id = decoder.read_u32()?;
    let nonce = decoder.read_u64()?;

    let sender = Address::from_bytes(decoder.take_array::<32>()?);

    let recipient = Address::from_bytes(decoder.take_array::<32>()?);

    let amount = decoder.read_u64()?;
    let fee = decoder.read_u64()?;

    let payload_len = decoder.read_u32()? as usize;

    let required_remaining = payload_len
        .checked_add(SIGNED_TRANSACTION_AUTH_LEN)
        .ok_or(CodecError::LengthOverflow)?;

    if decoder.remaining() != required_remaining {
        return Err(CodecError::InvalidTransactionLength);
    }

    let payload = decoder.take_slice(payload_len)?.to_vec();

    let public_key = PublicKey::from_bytes(decoder.take_array::<32>()?);

    let signature = SignatureBytes::from_bytes(decoder.take_array::<64>()?);

    if decoder.remaining() != 0 {
        return Err(CodecError::TrailingBytes);
    }

    let body = TransactionBody::new(
        version, kind, chain_id, nonce, sender, recipient, amount, fee, payload,
    );

    Ok(SignedTransaction::from_parts(body, public_key, signature))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEnd,
    InvalidBlockMagic,
    InvalidTransactionLength,
    TrailingBytes,
    TooManyTransactions,
    TransactionTooLarge,
    LengthOverflow,
    Transaction(SignedTransactionError),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => {
                write!(f, "unexpected end of encoded data")
            }

            Self::InvalidBlockMagic => {
                write!(f, "invalid Mirror block encoding magic")
            }

            Self::InvalidTransactionLength => {
                write!(f, "invalid encoded transaction length")
            }

            Self::TrailingBytes => {
                write!(f, "unexpected trailing bytes")
            }

            Self::TooManyTransactions => {
                write!(f, "too many transactions to encode")
            }

            Self::TransactionTooLarge => {
                write!(f, "encoded transaction is too large")
            }

            Self::LengthOverflow => {
                write!(f, "encoded length overflow")
            }

            Self::Transaction(error) => {
                write!(f, "transaction encoding failed: {error}")
            }
        }
    }
}

impl std::error::Error for CodecError {}

impl From<SignedTransactionError> for CodecError {
    fn from(error: SignedTransactionError) -> Self {
        Self::Transaction(error)
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn take_slice(&mut self, len: usize) -> Result<&'a [u8], CodecError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(CodecError::LengthOverflow)?;

        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(CodecError::UnexpectedEnd)?;

        self.offset = end;

        Ok(value)
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], CodecError> {
        let bytes = self.take_slice(N)?;

        bytes.try_into().map_err(|_| CodecError::UnexpectedEnd)
    }

    fn read_u16(&mut self) -> Result<u16, CodecError> {
        Ok(u16::from_le_bytes(self.take_array::<2>()?))
    }

    fn read_u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_le_bytes(self.take_array::<4>()?))
    }

    fn read_u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_le_bytes(self.take_array::<8>()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mirror_crypto::{Hash256, Keypair};

    use crate::{MIRROR_CHAIN_ID, NUSA_PER_MRY, TRANSACTION_KIND_TRANSFER, TRANSACTION_VERSION};

    fn signed_transaction() -> SignedTransaction {
        let alice = Keypair::from_secret_bytes([1u8; 32]);

        let bob = Keypair::from_secret_bytes([2u8; 32]);

        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            MIRROR_CHAIN_ID,
            0,
            Address::from_public_key(&alice.public_key()),
            Address::from_public_key(&bob.public_key()),
            NUSA_PER_MRY,
            0,
            Vec::new(),
        );

        SignedTransaction::sign(body, &alice).unwrap()
    }

    fn test_block() -> Block {
        let mut block = Block::new(
            1,
            Hash256::from_bytes([0x11; 32]),
            Hash256::from_bytes([0x22; 32]),
            1_800_000_001,
            0x1f0f_ffff,
            vec![signed_transaction()],
        )
        .unwrap();

        block.set_nonce(42);

        block
    }

    #[test]
    fn empty_block_has_expected_encoded_size() {
        let block = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            Vec::new(),
        )
        .unwrap();

        assert_eq!(encode_block(&block).unwrap().len(), MIN_ENCODED_BLOCK_LEN);

        assert_eq!(MIN_ENCODED_BLOCK_LEN, 128);
    }

    #[test]
    fn block_round_trip_preserves_exact_value() {
        let original = test_block();

        let bytes = encode_block(&original).unwrap();

        let decoded = decode_block(&bytes).unwrap();

        assert_eq!(decoded, original);
        assert_eq!(decoded.hash(), original.hash());
    }

    #[test]
    fn reencoding_decoded_block_is_identical() {
        let original = encode_block(&test_block()).unwrap();

        let decoded = decode_block(&original).unwrap();

        let reencoded = encode_block(&decoded).unwrap();

        assert_eq!(reencoded, original);
    }

    #[test]
    fn truncated_block_is_rejected() {
        let mut bytes = encode_block(&test_block()).unwrap();

        bytes.pop();

        assert_eq!(decode_block(&bytes), Err(CodecError::UnexpectedEnd));
    }

    #[test]
    fn invalid_magic_is_rejected() {
        let mut bytes = encode_block(&test_block()).unwrap();

        bytes[0] ^= 0xff;

        assert_eq!(decode_block(&bytes), Err(CodecError::InvalidBlockMagic));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut bytes = encode_block(&test_block()).unwrap();

        bytes.push(0);

        assert_eq!(decode_block(&bytes), Err(CodecError::TrailingBytes));
    }

    #[test]
    fn corrupted_signature_decodes_but_fails_integrity() {
        let mut bytes = encode_block(&test_block()).unwrap();

        let last = bytes.len() - 1;

        bytes[last] ^= 0x01;

        let decoded = decode_block(&bytes).unwrap();

        assert!(decoded.validate_integrity().is_err());
    }
}
