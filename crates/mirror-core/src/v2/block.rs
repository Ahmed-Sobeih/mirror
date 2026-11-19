//! MB-02A: immutable, bounded V2 block representations and body commitments.
//!
//! A constructed or decoded block is UNTRUSTED. Integrity verification establishes
//! configuration agreement, the exact ordered-operation commitment and strict
//! signatures only. It does not execute balances or membership, check sequences,
//! transfer policy/windows/ancestry, validate proof of work or ledger roots, pin
//! genesis, select a chain, or establish finality. Those are MB-02B/C boundaries.

use std::fmt;

use mirror_crypto::{Hash256, sha256d};
use mirror_personhood::v2::ConfigV2;

use super::operation::{SignedOperationV2, UnifiedOperationErrorV2};

pub const BLOCK_VERSION_V2: u32 = 2;
pub const BLOCK_HEADER_LEN_V2: usize = 160;
pub const BLOCK_ENCODING_MAGIC_V2: [u8; 4] = *b"MRB2";
pub const MIN_ENCODED_BLOCK_LEN_V2: usize = 168;

const HEADER_DOMAIN: &[u8] = b"MIRROR_BLOCK_V2";
const LEAF_DOMAIN: &[u8] = b"MIRROR_OP_LEAF_V2";
const ROOT_DOMAIN: &[u8] = b"MIRROR_OP_ROOT_V2";
const MIN_SIGNED_OPERATION_LEN_V2: usize = 227;

/// An immutable, untrusted V2 header. A hash is not proof of header validity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockHeaderV2 {
    config_hash: Hash256,
    height: u64,
    previous_hash: Hash256,
    operations_root: Hash256,
    ledger_root: Hash256,
    timestamp: u64,
    pow_bits: u32,
    nonce: u64,
}

impl BlockHeaderV2 {
    /// Copy fixed-size construction inputs. Version is exactly 2. This does not
    /// establish context, ancestry, work, ledger correctness or genesis identity.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        config_hash: Hash256,
        height: u64,
        previous_hash: Hash256,
        operations_root: Hash256,
        ledger_root: Hash256,
        timestamp: u64,
        pow_bits: u32,
        nonce: u64,
    ) -> Self {
        Self {
            config_hash,
            height,
            previous_hash,
            operations_root,
            ledger_root,
            timestamp,
            pow_bits,
            nonce,
        }
    }

    pub const fn version(&self) -> u32 {
        BLOCK_VERSION_V2
    }
    pub const fn config_hash(&self) -> Hash256 {
        self.config_hash
    }
    pub const fn height(&self) -> u64 {
        self.height
    }
    pub const fn previous_hash(&self) -> Hash256 {
        self.previous_hash
    }
    pub const fn operations_root(&self) -> Hash256 {
        self.operations_root
    }
    pub const fn ledger_root(&self) -> Hash256 {
        self.ledger_root
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

    /// Canonical fixed-size header. All integers are unsigned little-endian.
    pub fn encode(&self) -> [u8; BLOCK_HEADER_LEN_V2] {
        let mut bytes = [0; BLOCK_HEADER_LEN_V2];
        bytes[0..4].copy_from_slice(&BLOCK_VERSION_V2.to_le_bytes());
        bytes[4..36].copy_from_slice(self.config_hash.as_bytes());
        bytes[36..44].copy_from_slice(&self.height.to_le_bytes());
        bytes[44..76].copy_from_slice(self.previous_hash.as_bytes());
        bytes[76..108].copy_from_slice(self.operations_root.as_bytes());
        bytes[108..140].copy_from_slice(self.ledger_root.as_bytes());
        bytes[140..148].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[148..152].copy_from_slice(&self.pow_bits.to_le_bytes());
        bytes[152..160].copy_from_slice(&self.nonce.to_le_bytes());
        bytes
    }

    /// Structural decoding only, requiring exactly 160 bytes and version 2.
    pub fn decode(bytes: &[u8]) -> Result<Self, BlockErrorV2> {
        if bytes.len() != BLOCK_HEADER_LEN_V2 {
            return Err(BlockErrorV2::InvalidHeaderLength);
        }
        let mut reader = Reader::new(bytes);
        let version = u32::from_le_bytes(reader.take()?);
        if version != BLOCK_VERSION_V2 {
            return Err(BlockErrorV2::UnsupportedVersion(version));
        }
        Ok(Self::new(
            Hash256::from_bytes(reader.take()?),
            u64::from_le_bytes(reader.take()?),
            Hash256::from_bytes(reader.take()?),
            Hash256::from_bytes(reader.take()?),
            Hash256::from_bytes(reader.take()?),
            u64::from_le_bytes(reader.take()?),
            u32::from_le_bytes(reader.take()?),
            u64::from_le_bytes(reader.take()?),
        ))
    }

    /// SHA256d("MIRROR_BLOCK_V2" || canonical_header), without a terminating NUL.
    pub fn hash(&self) -> Hash256 {
        let mut preimage = [0; HEADER_DOMAIN.len() + BLOCK_HEADER_LEN_V2];
        preimage[..HEADER_DOMAIN.len()].copy_from_slice(HEADER_DOMAIN);
        preimage[HEADER_DOMAIN.len()..].copy_from_slice(&self.encode());
        sha256d(&preimage)
    }
}

/// A bounded, immutable representation, NOT an accepted blockchain block.
/// Original full-block operation order and positions are preserved, including
/// transfers. Never filter this list into membership-only execution/replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockV2 {
    header: BlockHeaderV2,
    operations: Vec<SignedOperationV2>,
    encoded_len: usize,
}

impl BlockV2 {
    /// Check all body budgets before copying inputs. Construction is structural:
    /// signatures, context and operation commitment may still be invalid.
    pub fn new(
        config: &ConfigV2,
        header: BlockHeaderV2,
        operations: &[SignedOperationV2],
    ) -> Result<Self, BlockErrorV2> {
        let encoded_len = check_operations(config, operations)?;
        Ok(Self {
            header,
            operations: operations.to_vec(),
            encoded_len,
        })
    }

    pub const fn header(&self) -> &BlockHeaderV2 {
        &self.header
    }
    pub fn operations(&self) -> &[SignedOperationV2] {
        &self.operations
    }
    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }
    pub fn hash(&self) -> Hash256 {
        self.header.hash()
    }

    /// Recheck the supplied configuration's budgets before output allocation.
    /// Successful encoding is not signature, policy or block validation.
    pub fn encode(&self, config: &ConfigV2) -> Result<Vec<u8>, BlockErrorV2> {
        let len = check_operations(config, &self.operations)?;
        let count = operation_count(config, self.operations.len())?;
        let mut bytes = Vec::with_capacity(len);
        bytes.extend_from_slice(&BLOCK_ENCODING_MAGIC_V2);
        bytes.extend_from_slice(&self.header.encode());
        bytes.extend_from_slice(&count.to_le_bytes());
        for operation in &self.operations {
            let encoded = operation.encode();
            let record_len =
                u32::try_from(encoded.len()).map_err(|_| BlockErrorV2::LengthOverflow)?;
            bytes.extend_from_slice(&record_len.to_le_bytes());
            bytes.extend_from_slice(&encoded);
        }
        Ok(bytes)
    }

    /// Structural MRB2 decoder. Performs an allocation-free complete preflight:
    /// total bytes, count against actual bytes, each exact kind/length/version,
    /// membership/enrollment subcounts and trailing data. Only after that pass
    /// does it reserve the bounded operation vector. Signatures are not verified.
    pub fn decode(config: &ConfigV2, bytes: &[u8]) -> Result<Self, BlockErrorV2> {
        if bytes.len() > config.limits().max_block_bytes as usize {
            return Err(BlockErrorV2::TooLarge);
        }
        if bytes.len() < MIN_ENCODED_BLOCK_LEN_V2 {
            return Err(BlockErrorV2::Truncated);
        }
        let mut input = Reader::new(bytes);
        let magic: [u8; 4] = input.take()?;
        if magic != BLOCK_ENCODING_MAGIC_V2 {
            return Err(BlockErrorV2::InvalidMagic);
        }
        let header = BlockHeaderV2::decode(input.slice(BLOCK_HEADER_LEN_V2)?)?;
        let count = u32::from_le_bytes(input.take()?);
        if count > config.limits().max_operations {
            return Err(BlockErrorV2::TooManyOperations);
        }

        // Even the shortest permitted signed operation needs its length prefix.
        // Compare by division so a malicious count cannot overflow multiplication.
        if count as usize > input.remaining() / (4 + MIN_SIGNED_OPERATION_LEN_V2) {
            return Err(BlockErrorV2::Truncated);
        }

        let mut budget = BodyBudget::new();
        for index in 0..count {
            let len = u32::from_le_bytes(input.take()?) as usize;
            if len > config.limits().max_operation_bytes as usize {
                return Err(BlockErrorV2::OperationTooLarge { index });
            }
            let record = input.slice(len)?;
            let operation = SignedOperationV2::decode(config, record)
                .map_err(|error| BlockErrorV2::Operation { index, error })?;
            budget.add(config, &operation, index)?;
        }
        if input.remaining() != 0 {
            return Err(BlockErrorV2::TrailingBytes);
        }
        // Parsing exact lengths means budget bytes equal the input size. The
        // second pass performs the same fixed-size decoding, with no I/O or
        // mutable context. Private immutable operation fields cannot change.
        let mut input = Reader::new(&bytes[MIN_ENCODED_BLOCK_LEN_V2..]);
        let mut operations = Vec::with_capacity(count as usize);
        for index in 0..count {
            let len = u32::from_le_bytes(input.take()?) as usize;
            let record = input.slice(len)?;
            operations.push(
                SignedOperationV2::decode(config, record)
                    .map_err(|error| BlockErrorV2::Operation { index, error })?,
            );
        }
        Ok(Self {
            header,
            operations,
            encoded_len: budget.bytes,
        })
    }

    /// Check budgets, common configuration, exact ordered body commitment and
    /// every strict signature. This is NOT block acceptance: account/member
    /// sequences, balances, transfer policy, authorization windows/ancestry,
    /// ledger root, PoW, timestamps, genesis and fork choice are not checked.
    /// A caller must execute/validate those MB-02B/C rules before publishing state.
    pub fn verify_integrity(&self, config: &ConfigV2) -> Result<(), BlockErrorV2> {
        check_operations(config, &self.operations)?;
        if self.header.config_hash() != config.context() {
            return Err(BlockErrorV2::WrongContext);
        }
        for (index, operation) in self.operations.iter().enumerate() {
            if operation.header().config_hash != config.context() {
                return Err(BlockErrorV2::OperationContext {
                    index: index as u32,
                });
            }
        }
        let root = operations_root_v2(config, &self.operations)?;
        if root != self.header.operations_root() {
            return Err(BlockErrorV2::OperationsRootMismatch);
        }
        for (index, operation) in self.operations.iter().enumerate() {
            operation
                .verify_signature()
                .map_err(|error| BlockErrorV2::Operation {
                    index: index as u32,
                    error,
                })?;
        }
        Ok(())
    }
}

/// Commit to the exact ordered signed operations AND their count. Input budgets
/// are checked before allocating leaves. This is the MRB2 operation commitment,
/// distinct from the membership-only MPH2 diagnostic group digest.
///
/// leaf = SHA256d("MIRROR_OP_LEAF_V2" || len:u32 LE || complete_signed_operation)
/// root = SHA256d("MIRROR_OP_ROOT_V2" || count:u32 LE || merkle_root(leaves))
///
/// The inner Merkle tree preserves Mirror's existing convention: SHA256d(empty)
/// for no leaves, identity for one leaf, pairwise SHA256d and duplication of an
/// odd last leaf. The outer count prevents the duplicate-last length ambiguity.
pub fn operations_root_v2(
    config: &ConfigV2,
    operations: &[SignedOperationV2],
) -> Result<Hash256, BlockErrorV2> {
    check_operations(config, operations)?;
    let count = operation_count(config, operations.len())?;
    let mut leaves = Vec::with_capacity(operations.len());
    for operation in operations {
        let bytes = operation.encode();
        let len = u32::try_from(bytes.len()).map_err(|_| BlockErrorV2::LengthOverflow)?;
        let mut preimage = Vec::with_capacity(LEAF_DOMAIN.len() + 4 + bytes.len());
        preimage.extend_from_slice(LEAF_DOMAIN);
        preimage.extend_from_slice(&len.to_le_bytes());
        preimage.extend_from_slice(&bytes);
        leaves.push(sha256d(&preimage));
    }
    let inner = crate::merkle_root(&leaves);
    let mut preimage = [0; ROOT_DOMAIN.len() + 4 + 32];
    preimage[..ROOT_DOMAIN.len()].copy_from_slice(ROOT_DOMAIN);
    preimage[ROOT_DOMAIN.len()..ROOT_DOMAIN.len() + 4].copy_from_slice(&count.to_le_bytes());
    preimage[ROOT_DOMAIN.len() + 4..].copy_from_slice(inner.as_bytes());
    Ok(sha256d(&preimage))
}

fn operation_count(config: &ConfigV2, len: usize) -> Result<u32, BlockErrorV2> {
    let count = u32::try_from(len).map_err(|_| BlockErrorV2::TooManyOperations)?;
    if count > config.limits().max_operations {
        return Err(BlockErrorV2::TooManyOperations);
    }
    Ok(count)
}

/// Preflight the complete MRB2 operation/count/byte budgets without allocating,
/// hashing or verifying signatures. Returns the complete encoded block length.
/// This is shared with the mixed-ledger executor; success grants no authority.
pub fn validate_operation_limits_v2(
    config: &ConfigV2,
    operations: &[SignedOperationV2],
) -> Result<usize, BlockErrorV2> {
    check_operations(config, operations)
}

fn check_operations(
    config: &ConfigV2,
    operations: &[SignedOperationV2],
) -> Result<usize, BlockErrorV2> {
    operation_count(config, operations.len())?;
    let mut budget = BodyBudget::new();
    for (index, operation) in operations.iter().enumerate() {
        budget.add(config, operation, index as u32)?;
    }
    if budget.bytes > config.limits().max_block_bytes as usize {
        return Err(BlockErrorV2::TooLarge);
    }
    Ok(budget.bytes)
}

struct BodyBudget {
    bytes: usize,
    membership: u32,
    enrollments: u32,
}

impl BodyBudget {
    const fn new() -> Self {
        Self {
            bytes: MIN_ENCODED_BLOCK_LEN_V2,
            membership: 0,
            enrollments: 0,
        }
    }

    fn add(
        &mut self,
        config: &ConfigV2,
        operation: &SignedOperationV2,
        index: u32,
    ) -> Result<(), BlockErrorV2> {
        let limits = config.limits();
        let len = operation.encoded_len();
        if len > limits.max_operation_bytes as usize {
            return Err(BlockErrorV2::OperationTooLarge { index });
        }
        if operation.kind() != 0 {
            self.membership = self
                .membership
                .checked_add(1)
                .ok_or(BlockErrorV2::LengthOverflow)?;
            if self.membership > limits.max_membership_operations {
                return Err(BlockErrorV2::TooManyMembershipOperations);
            }
        }
        if operation.kind() == 1 {
            self.enrollments = self
                .enrollments
                .checked_add(1)
                .ok_or(BlockErrorV2::LengthOverflow)?;
            if self.enrollments > limits.max_new_enrollments {
                return Err(BlockErrorV2::TooManyEnrollments);
            }
        }
        self.bytes = self
            .bytes
            .checked_add(4)
            .and_then(|bytes| bytes.checked_add(len))
            .ok_or(BlockErrorV2::LengthOverflow)?;
        if self.bytes > limits.max_block_bytes as usize {
            return Err(BlockErrorV2::TooLarge);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockErrorV2 {
    InvalidHeaderLength,
    UnsupportedVersion(u32),
    InvalidMagic,
    Truncated,
    TrailingBytes,
    TooLarge,
    TooManyOperations,
    TooManyMembershipOperations,
    TooManyEnrollments,
    OperationTooLarge {
        index: u32,
    },
    Operation {
        index: u32,
        error: UnifiedOperationErrorV2,
    },
    LengthOverflow,
    WrongContext,
    OperationContext {
        index: u32,
    },
    OperationsRootMismatch,
}

impl fmt::Display for BlockErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 block representation rejected: {self:?}")
    }
}
impl std::error::Error for BlockErrorV2 {}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
    fn slice(&mut self, len: usize) -> Result<&'a [u8], BlockErrorV2> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(BlockErrorV2::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(BlockErrorV2::Truncated)?;
        self.position = end;
        Ok(value)
    }
    fn take<const N: usize>(&mut self) -> Result<[u8; N], BlockErrorV2> {
        self.slice(N)?
            .try_into()
            .map_err(|_| BlockErrorV2::Truncated)
    }
}
