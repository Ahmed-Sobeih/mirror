use core::fmt;

use mirror_crypto::{Hash256, sha256d};

/// One Maraya (MRY) consists of ten million Nusa.
///
/// Consensus code must always use integer Nusa.
/// Floating-point currency values are forbidden.
pub const NUSA_PER_MRY: u64 = 10_000_000;

/// Current Mirror transaction format version.
pub const TRANSACTION_VERSION: u16 = 1;

/// Normal MRY transfer.
pub const TRANSACTION_KIND_TRANSFER: u16 = 0;

/// Size of the fixed portion of a transaction body.
///
/// Payload bytes are appended after these 100 bytes.
pub const TRANSACTION_BODY_FIXED_LEN: usize = 100;

/// A 256-bit Mirror account address.
///
/// Public-key-to-address derivation will be defined when
/// we implement Mirror signatures.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Address([u8; 32]);

impl Address {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        let mut output = String::with_capacity(64);

        for byte in self.0 {
            use core::fmt::Write;

            write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
        }

        output
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({self})")
    }
}

/// The consensus-critical part of a Mirror transaction.
///
/// This exact encoding will later be signed by the sender.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionBody {
    version: u16,
    kind: u16,
    chain_id: u32,
    nonce: u64,
    sender: Address,
    recipient: Address,
    amount: u64,
    fee: u64,
    payload: Vec<u8>,
}

impl TransactionBody {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u16,
        kind: u16,
        chain_id: u32,
        nonce: u64,
        sender: Address,
        recipient: Address,
        amount: u64,
        fee: u64,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            version,
            kind,
            chain_id,
            nonce,
            sender,
            recipient,
            amount,
            fee,
            payload,
        }
    }

    pub const fn version(&self) -> u16 {
        self.version
    }

    pub const fn kind(&self) -> u16 {
        self.kind
    }

    pub const fn chain_id(&self) -> u32 {
        self.chain_id
    }

    pub const fn nonce(&self) -> u64 {
        self.nonce
    }

    pub const fn sender(&self) -> Address {
        self.sender
    }

    pub const fn recipient(&self) -> Address {
        self.recipient
    }

    pub const fn amount(&self) -> u64 {
        self.amount
    }

    pub const fn fee(&self) -> u64 {
        self.fee
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Canonical Mirror transaction encoding.
    ///
    /// Layout:
    ///
    /// ```text
    /// version        u16   little-endian
    /// kind           u16   little-endian
    /// chain_id       u32   little-endian
    /// nonce          u64   little-endian
    /// sender         32 bytes
    /// recipient      32 bytes
    /// amount         u64   little-endian
    /// fee            u64   little-endian
    /// payload_length u32   little-endian
    /// payload        variable bytes
    /// ```
    pub fn encode(&self) -> Result<Vec<u8>, TransactionError> {
        let payload_len: u32 = self
            .payload
            .len()
            .try_into()
            .map_err(|_| TransactionError::PayloadTooLarge)?;

        let mut bytes = Vec::with_capacity(TRANSACTION_BODY_FIXED_LEN + self.payload.len());

        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.kind.to_le_bytes());
        bytes.extend_from_slice(&self.chain_id.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());

        bytes.extend_from_slice(self.sender.as_bytes());
        bytes.extend_from_slice(self.recipient.as_bytes());

        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.fee.to_le_bytes());

        bytes.extend_from_slice(&payload_len.to_le_bytes());
        bytes.extend_from_slice(&self.payload);

        Ok(bytes)
    }

    /// Hash of the transaction body.
    ///
    /// This is the value the sender will later sign.
    pub fn signing_hash(&self) -> Result<Hash256, TransactionError> {
        Ok(sha256d(&self.encode()?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionError {
    PayloadTooLarge,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge => {
                write!(f, "transaction payload is too large")
            }
        }
    }
}

impl std::error::Error for TransactionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_transaction() -> TransactionBody {
        TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            1,
            7,
            Address::from_bytes([0x11; 32]),
            Address::from_bytes([0x22; 32]),
            1_500_000_000,
            25_000,
            Vec::new(),
        )
    }

    #[test]
    fn empty_payload_transaction_is_100_bytes() {
        let transaction = test_transaction();

        assert_eq!(
            transaction.encode().unwrap().len(),
            TRANSACTION_BODY_FIXED_LEN
        );

        assert_eq!(TRANSACTION_BODY_FIXED_LEN, 100);
    }

    #[test]
    fn transaction_encoding_is_deterministic() {
        let transaction = test_transaction();

        assert_eq!(transaction.encode().unwrap(), transaction.encode().unwrap());
    }

    #[test]
    fn signing_hash_matches_fixed_test_vector() {
        let transaction = test_transaction();

        assert_eq!(
            transaction.signing_hash().unwrap().to_hex(),
            "8694066f38a49ea014bbe19a3c1e9774094993b581e38c27719749450645f2fe"
        );
    }

    #[test]
    fn changing_nonce_changes_signing_hash() {
        let first = test_transaction();

        let second = TransactionBody::new(
            first.version(),
            first.kind(),
            first.chain_id(),
            first.nonce() + 1,
            first.sender(),
            first.recipient(),
            first.amount(),
            first.fee(),
            first.payload().to_vec(),
        );

        assert_ne!(
            first.signing_hash().unwrap(),
            second.signing_hash().unwrap()
        );
    }

    #[test]
    fn payload_changes_signing_hash() {
        let first = test_transaction();

        let second = TransactionBody::new(
            first.version(),
            first.kind(),
            first.chain_id(),
            first.nonce(),
            first.sender(),
            first.recipient(),
            first.amount(),
            first.fee(),
            b"Mirror".to_vec(),
        );

        assert_ne!(
            first.signing_hash().unwrap(),
            second.signing_hash().unwrap()
        );
    }

    #[test]
    fn mry_denomination_is_exact() {
        assert_eq!(NUSA_PER_MRY, 10_000_000);
        assert_eq!(5 * NUSA_PER_MRY, 50_000_000);
    }
}
