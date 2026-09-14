use mirror_crypto::{Hash256, sha256d};

/// The exact serialized size of a Mirror block header.
pub const BLOCK_HEADER_LEN: usize = 120;

/// Current Mirror block format version.
pub const BLOCK_VERSION: u32 = 1;

/// Consensus-critical header of a Mirror block.
///
/// Binary layout:
///
/// ```text
/// version:              4 bytes, little-endian
/// previous_block_hash: 32 bytes
/// transaction_root:    32 bytes
/// state_root:          32 bytes
/// timestamp:            8 bytes, little-endian
/// pow_bits:             4 bytes, little-endian
/// nonce:                8 bytes, little-endian
/// ```
///
/// Total: 120 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    version: u32,
    previous_block_hash: Hash256,
    transaction_root: Hash256,
    state_root: Hash256,
    timestamp: u64,
    pow_bits: u32,
    nonce: u64,
}

impl BlockHeader {
    pub const fn new(
        version: u32,
        previous_block_hash: Hash256,
        transaction_root: Hash256,
        state_root: Hash256,
        timestamp: u64,
        pow_bits: u32,
        nonce: u64,
    ) -> Self {
        Self {
            version,
            previous_block_hash,
            transaction_root,
            state_root,
            timestamp,
            pow_bits,
            nonce,
        }
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn previous_block_hash(&self) -> Hash256 {
        self.previous_block_hash
    }

    pub const fn transaction_root(&self) -> Hash256 {
        self.transaction_root
    }

    pub const fn state_root(&self) -> Hash256 {
        self.state_root
    }

    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub const fn pow_bits(&self) -> u32 {
        self.pow_bits
    }

    pub const fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }

    /// Serializes the header into the exact byte representation
    /// used by Mirror consensus.
    pub fn encode(&self) -> [u8; BLOCK_HEADER_LEN] {
        let mut bytes = [0u8; BLOCK_HEADER_LEN];

        bytes[0..4].copy_from_slice(&self.version.to_le_bytes());

        bytes[4..36].copy_from_slice(self.previous_block_hash.as_bytes());

        bytes[36..68].copy_from_slice(self.transaction_root.as_bytes());

        bytes[68..100].copy_from_slice(self.state_root.as_bytes());

        bytes[100..108].copy_from_slice(&self.timestamp.to_le_bytes());

        bytes[108..112].copy_from_slice(&self.pow_bits.to_le_bytes());

        bytes[112..120].copy_from_slice(&self.nonce.to_le_bytes());

        bytes
    }

    /// Calculates the Mirror block hash:
    ///
    /// SHA256(SHA256(serialized_header))
    pub fn hash(&self) -> Hash256 {
        sha256d(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_header() -> BlockHeader {
        BlockHeader::new(
            1,
            Hash256::default(),
            Hash256::default(),
            Hash256::default(),
            0,
            0x207f_ffff,
            0,
        )
    }

    #[test]
    fn block_header_is_exactly_120_bytes() {
        let header = test_header();

        assert_eq!(header.encode().len(), BLOCK_HEADER_LEN);
        assert_eq!(BLOCK_HEADER_LEN, 120);
    }

    #[test]
    fn block_header_hash_matches_fixed_test_vector() {
        let header = test_header();

        assert_eq!(
            header.hash().to_hex(),
            "0de7cfa1a666c5ed1a2af99cbdc718ddda4dcc0a1c9dea71df6f01b51e8d779e"
        );
    }

    #[test]
    fn changing_nonce_changes_hash() {
        let mut header = test_header();

        let first = header.hash();

        header.set_nonce(1);

        let second = header.hash();

        assert_ne!(first, second);
    }

    #[test]
    fn encoding_is_deterministic() {
        let header = test_header();

        assert_eq!(header.encode(), header.encode());
    }

    #[test]
    fn fields_round_trip_through_accessors() {
        let previous = Hash256::from_bytes([1u8; 32]);
        let transactions = Hash256::from_bytes([2u8; 32]);
        let state = Hash256::from_bytes([3u8; 32]);

        let header = BlockHeader::new(
            7,
            previous,
            transactions,
            state,
            1_800_000_000,
            0x207f_ffff,
            42,
        );

        assert_eq!(header.version(), 7);
        assert_eq!(header.previous_block_hash(), previous);
        assert_eq!(header.transaction_root(), transactions);
        assert_eq!(header.state_root(), state);
        assert_eq!(header.timestamp(), 1_800_000_000);
        assert_eq!(header.pow_bits(), 0x207f_ffff);
        assert_eq!(header.nonce(), 42);
    }

    #[test]
    fn changing_state_root_changes_block_hash() {
        let first = test_header();

        let second = BlockHeader::new(
            first.version(),
            first.previous_block_hash(),
            first.transaction_root(),
            Hash256::from_bytes([1u8; 32]),
            first.timestamp(),
            first.pow_bits(),
            first.nonce(),
        );

        assert_ne!(first.hash(), second.hash());
    }
}

use crate::merkle::merkle_root;
use crate::signed_transaction::{SignedTransaction, SignedTransactionError};

/// A complete Mirror block.
///
/// The header commits cryptographically to the ordered transaction list
/// through `transaction_root`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    header: BlockHeader,
    transactions: Vec<SignedTransaction>,
}

impl Block {
    /// Construct a new unmined block.
    ///
    /// Every transaction is cryptographically verified before the block
    /// is created. The transaction Merkle root is calculated automatically.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u32,
        previous_block_hash: Hash256,
        state_root: Hash256,
        timestamp: u64,
        pow_bits: u32,
        transactions: Vec<SignedTransaction>,
    ) -> Result<Self, BlockError> {
        let transaction_root = Self::calculate_transaction_root(&transactions)?;

        let header = BlockHeader::new(
            version,
            previous_block_hash,
            transaction_root,
            state_root,
            timestamp,
            pow_bits,
            0,
        );

        Ok(Self {
            header,
            transactions,
        })
    }

    /// Construct a block from existing components.
    ///
    /// This is intentionally unchecked because blocks received from the
    /// network will first need to be decoded and then validated.
    pub const fn from_parts(header: BlockHeader, transactions: Vec<SignedTransaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    pub const fn header(&self) -> &BlockHeader {
        &self.header
    }

    pub fn transactions(&self) -> &[SignedTransaction] {
        &self.transactions
    }

    pub fn hash(&self) -> Hash256 {
        self.header.hash()
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.header.set_nonce(nonce);
    }

    /// Recalculate the transaction root from this block's transactions.
    pub fn calculated_transaction_root(&self) -> Result<Hash256, BlockError> {
        Self::calculate_transaction_root(&self.transactions)
    }

    /// Validate the cryptographic integrity of the block's transaction list.
    ///
    /// This currently verifies:
    ///
    /// - every transaction signature
    /// - every sender/public-key relationship
    /// - every transaction ID
    /// - the Merkle root committed in the header
    ///
    /// Account balances, nonces, fees and state transitions will be checked
    /// later by the state-transition layer.
    pub fn validate_integrity(&self) -> Result<(), BlockError> {
        let calculated = self.calculated_transaction_root()?;

        if calculated != self.header.transaction_root() {
            return Err(BlockError::TransactionRootMismatch);
        }

        Ok(())
    }

    fn calculate_transaction_root(
        transactions: &[SignedTransaction],
    ) -> Result<Hash256, BlockError> {
        let mut txids = Vec::with_capacity(transactions.len());

        for transaction in transactions {
            transaction.verify()?;
            txids.push(transaction.txid()?);
        }

        Ok(merkle_root(&txids))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockError {
    Transaction(SignedTransactionError),
    TransactionRootMismatch,
}

impl core::fmt::Display for BlockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transaction(error) => {
                write!(f, "invalid transaction: {error}")
            }

            Self::TransactionRootMismatch => {
                write!(f, "block transaction root does not match transactions")
            }
        }
    }
}

impl std::error::Error for BlockError {}

impl From<SignedTransactionError> for BlockError {
    fn from(error: SignedTransactionError) -> Self {
        Self::Transaction(error)
    }
}

#[cfg(test)]
mod full_block_tests {
    use super::*;

    use crate::{
        Address, SignedTransaction, TRANSACTION_KIND_TRANSFER, TRANSACTION_VERSION, TransactionBody,
    };

    use mirror_crypto::Keypair;

    fn signed_transaction(secret: u8, nonce: u64, recipient: u8) -> SignedTransaction {
        let keypair = Keypair::from_secret_bytes([secret; 32]);

        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            1,
            nonce,
            Address::from_public_key(&keypair.public_key()),
            Address::from_bytes([recipient; 32]),
            10_000_000,
            1_000,
            Vec::new(),
        );

        SignedTransaction::sign(body, &keypair).expect("test transaction must sign")
    }

    fn test_transactions() -> Vec<SignedTransaction> {
        vec![
            signed_transaction(1, 0, 0x22),
            signed_transaction(2, 0, 0x33),
        ]
    }

    #[test]
    fn block_commits_to_transaction_merkle_root() {
        let transactions = test_transactions();

        let txids = transactions
            .iter()
            .map(|tx| tx.txid().unwrap())
            .collect::<Vec<_>>();

        let expected_root = merkle_root(&txids);

        let block = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            transactions,
        )
        .unwrap();

        assert_eq!(block.header().transaction_root(), expected_root);
    }

    #[test]
    fn valid_block_integrity_passes() {
        let block = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            test_transactions(),
        )
        .unwrap();

        assert_eq!(block.validate_integrity(), Ok(()));
    }

    #[test]
    fn transaction_order_changes_block_root() {
        let first_transactions = test_transactions();

        let mut second_transactions = first_transactions.clone();

        second_transactions.reverse();

        let first = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            first_transactions,
        )
        .unwrap();

        let second = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            second_transactions,
        )
        .unwrap();

        assert_ne!(
            first.header().transaction_root(),
            second.header().transaction_root()
        );
    }

    #[test]
    fn incorrect_transaction_root_is_rejected() {
        let transactions = test_transactions();

        let header = BlockHeader::new(
            1,
            Hash256::default(),
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            0,
        );

        let block = Block::from_parts(header, transactions);

        assert_eq!(
            block.validate_integrity(),
            Err(BlockError::TransactionRootMismatch)
        );
    }

    #[test]
    fn empty_block_uses_empty_merkle_root() {
        let block = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            Vec::new(),
        )
        .unwrap();

        assert_eq!(block.header().transaction_root(), merkle_root(&[]));
    }
}
