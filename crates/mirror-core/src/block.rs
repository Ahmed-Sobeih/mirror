use mirror_crypto::{Hash256, sha256d};

/// The exact serialized size of a Mirror block header.
pub const BLOCK_HEADER_LEN: usize = 88;

/// Consensus-critical header of a Mirror block.
///
/// Binary layout:
///
/// - version:             4 bytes, little-endian
/// - previous_block_hash: 32 bytes
/// - transaction_root:    32 bytes
/// - timestamp:           8 bytes, little-endian
/// - pow_bits:            4 bytes, little-endian
/// - nonce:               8 bytes, little-endian
///
/// Total: 88 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    version: u32,
    previous_block_hash: Hash256,
    transaction_root: Hash256,
    timestamp: u64,
    pow_bits: u32,
    nonce: u64,
}

impl BlockHeader {
    pub const fn new(
        version: u32,
        previous_block_hash: Hash256,
        transaction_root: Hash256,
        timestamp: u64,
        pow_bits: u32,
        nonce: u64,
    ) -> Self {
        Self {
            version,
            previous_block_hash,
            transaction_root,
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

        bytes[68..76].copy_from_slice(&self.timestamp.to_le_bytes());

        bytes[76..80].copy_from_slice(&self.pow_bits.to_le_bytes());

        bytes[80..88].copy_from_slice(&self.nonce.to_le_bytes());

        bytes
    }

    /// Calculates the block hash:
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
        BlockHeader::new(1, Hash256::default(), Hash256::default(), 0, 0x207f_ffff, 0)
    }

    #[test]
    fn block_header_is_exactly_88_bytes() {
        let header = test_header();

        assert_eq!(header.encode().len(), BLOCK_HEADER_LEN);
        assert_eq!(BLOCK_HEADER_LEN, 88);
    }

    #[test]
    fn block_header_hash_matches_fixed_test_vector() {
        let header = test_header();

        assert_eq!(
            header.hash().to_hex(),
            "5faadd5964364ccd1ecb8e18afcf96d9aa39a7b5a73d04de3c1f9bc5cdf442f6"
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

        let header = BlockHeader::new(7, previous, transactions, 1_800_000_000, 0x207f_ffff, 42);

        assert_eq!(header.version(), 7);
        assert_eq!(header.previous_block_hash(), previous);
        assert_eq!(header.transaction_root(), transactions);
        assert_eq!(header.timestamp(), 1_800_000_000);
        assert_eq!(header.pow_bits(), 0x207f_ffff);
        assert_eq!(header.nonce(), 42);
    }
}
