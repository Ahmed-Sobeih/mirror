//! Bounded diagnostic MPH2 history, never a source of chain authority.
//!
//! Replay requires independently trusted, immutable per-height metadata and a
//! caller-selected final checkpoint. A log cannot authenticate its own ancestry,
//! exact operations, state roots, or freshness. Real MRB2 validation is MB-02.
use std::fmt;

use mirror_crypto::{Hash256, sha256d};

use super::{
    ConfigV2, ENROLL_OPERATION_LEN_V2, MEMBERSHIP_BLOCK_BASE_BYTES_V2, MembershipActionV2,
    MembershipRegistryV2, OperationErrorV2, ParentChainViewV2, REVIEW_OPERATION_LEN_V2,
    SignedMembershipOperationV2, StateCodecErrorV2, StateErrorV2, VerifiedBootstrap,
};

pub const REPLAY_VERSION_V2: u16 = 2;
pub const REPLAY_PREFIX_LEN_V2: usize = 74;
pub const REPLAY_GROUP_BASE_LEN_V2: usize = 44;
/// Synthetic diagnostic bounds, not limits on the eventual live blockchain.
pub const MAX_REPLAY_BLOCKS_V2: usize = 4096;
/// Aggregate upper-bound state/configuration items across staged replay steps.
/// Includes cloned seed descriptors and test allocations, preventing a small log
/// of empty blocks from repeatedly copying/scanning large retained data.
pub const MAX_REPLAY_STATE_ITEMS_V2: u64 = 1_048_576;
const MAGIC: [u8; 4] = *b"MPH2";
const GROUP_DOMAIN: &[u8] = b"MIRROR_MEMBERSHIP_REPLAY_GROUP_V2";

/// An untrusted immutable diagnostic block group. A hash is a claim, not proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayGroupV2 {
    height: u64,
    block_hash: Hash256,
    operations: Vec<SignedMembershipOperationV2>,
}

impl ReplayGroupV2 {
    pub fn new(
        config: &ConfigV2,
        height: u64,
        block_hash: Hash256,
        operations: &[SignedMembershipOperationV2],
    ) -> Result<Self, ReplayErrorV2> {
        if height == 0 {
            return Err(ReplayErrorV2::NonContiguousHeight);
        }
        operation_bytes(config, operations)?;
        let mut owned = reserved_vec(operations.len())?;
        owned.extend_from_slice(operations);
        Ok(Self {
            height,
            block_hash,
            operations: owned,
        })
    }

    pub const fn height(&self) -> u64 {
        self.height
    }
    pub const fn block_hash(&self) -> Hash256 {
        self.block_hash
    }
    pub fn operations(&self) -> &[SignedMembershipOperationV2] {
        &self.operations
    }

    /// Exact diagnostic operation commitment, NOT the future MRB2 Merkle root.
    pub fn commitment(&self, config_hash: Hash256) -> Result<Hash256, ReplayErrorV2> {
        group_commitment(config_hash, self.height, &self.operations)
    }
}

/// Metadata supplied by an independently validated chain. For height zero the
/// group commitment is the empty group at height zero, not the bootstrap proofs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrustedReplayBlockV2 {
    pub block_hash: Hash256,
    pub group_commitment: Hash256,
    pub membership_root: Hash256,
}

/// TRUSTED CALLER BOUNDARY. This must describe one stable, locally validated
/// history, including pinned genesis and roots. Never implement it from MPH2
/// records or a peer's advertised map; doing so would make checks circular.
/// B4 caches each returned record once. MB-02/03 must supply real chain authority.
pub trait ReplayChainViewV2 {
    fn config_hash(&self) -> Hash256;
    fn block_at(&self, height: u64) -> Option<TrustedReplayBlockV2>;
}

/// Expected checkpoint chosen OUTSIDE the untrusted log, preventing acceptance
/// of an old valid prefix as an allegedly latest state. Does not discover latestness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayExpectationV2 {
    pub config_hash: Hash256,
    pub height: u64,
    pub tip_hash: Hash256,
    pub membership_root: Hash256,
}

/// A bounded full-from-genesis diagnostic transcript. Structural decoding alone
/// checks no signatures and creates no state or reviewer authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipReplayV2 {
    config: ConfigV2,
    genesis_hash: Hash256,
    groups: Vec<ReplayGroupV2>,
}

impl MembershipReplayV2 {
    pub fn new(
        config: &ConfigV2,
        genesis_hash: Hash256,
        groups: &[ReplayGroupV2],
    ) -> Result<Self, ReplayErrorV2> {
        transcript_size(config, groups)?;
        let mut owned = reserved_vec(groups.len())?;
        owned.extend_from_slice(groups);
        Ok(Self {
            config: config.clone(),
            genesis_hash,
            groups: owned,
        })
    }

    pub fn config_hash(&self) -> Hash256 {
        self.config.context()
    }
    pub const fn genesis_hash(&self) -> Hash256 {
        self.genesis_hash
    }
    pub fn groups(&self) -> &[ReplayGroupV2] {
        &self.groups
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReplayErrorV2> {
        let len = transcript_size(&self.config, &self.groups)?;
        let mut bytes = reserved_vec(len)?;
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&REPLAY_VERSION_V2.to_le_bytes());
        bytes.extend_from_slice(self.config_hash().as_bytes());
        bytes.extend_from_slice(self.genesis_hash.as_bytes());
        bytes.extend_from_slice(&(self.groups.len() as u32).to_le_bytes());
        for group in &self.groups {
            bytes.extend_from_slice(&group.height.to_le_bytes());
            bytes.extend_from_slice(group.block_hash.as_bytes());
            append_operations(&group.operations, &mut bytes);
        }
        debug_assert_eq!(bytes.len(), len);
        Ok(bytes)
    }

    pub fn decode(config: &ConfigV2, bytes: &[u8]) -> Result<Self, ReplayErrorV2> {
        if bytes.len() > config.limits().max_log_bytes as usize {
            return Err(ReplayErrorV2::LogBytesLimit);
        }
        let mut input = Input { bytes, position: 0 };
        if input.array::<4>()? != MAGIC {
            return Err(ReplayErrorV2::InvalidMagic);
        }
        if input.u16()? != REPLAY_VERSION_V2 {
            return Err(ReplayErrorV2::UnsupportedVersion);
        }
        if input.hash()? != config.context() {
            return Err(ReplayErrorV2::WrongContext);
        }
        let genesis_hash = input.hash()?;
        let count = input.u32()? as usize;
        if count > MAX_REPLAY_BLOCKS_V2 {
            return Err(ReplayErrorV2::GroupLimit);
        }
        // Require actual minimum group bytes before any count-sized allocation.
        input.require(
            count
                .checked_mul(REPLAY_GROUP_BASE_LEN_V2)
                .ok_or(ReplayErrorV2::LogBytesLimit)?,
        )?;
        let mut groups = reserved_vec(count)?;
        for index in 0..count {
            let height = input.u64()?;
            if height != index as u64 + 1 {
                return Err(ReplayErrorV2::NonContiguousHeight);
            }
            let block_hash = input.hash()?;
            let operation_count = input.u32()? as usize;
            check_operation_count(config, operation_count)?;
            let minimum = operation_count
                .checked_mul(4 + REVIEW_OPERATION_LEN_V2)
                .and_then(|n| n.checked_add((count - index - 1) * REPLAY_GROUP_BASE_LEN_V2))
                .ok_or(ReplayErrorV2::LogBytesLimit)?;
            input.require(minimum)?;
            // A declared count must also fit this block's minimum byte budget.
            let minimum_block = operation_count
                .checked_mul(4 + REVIEW_OPERATION_LEN_V2)
                .and_then(|n| n.checked_add(MEMBERSHIP_BLOCK_BASE_BYTES_V2))
                .ok_or(ReplayErrorV2::BlockBytesLimit)?;
            if minimum_block > config.limits().max_block_bytes as usize {
                return Err(ReplayErrorV2::BlockBytesLimit);
            }
            let mut operations = reserved_vec(operation_count)?;
            let mut block_bytes = MEMBERSHIP_BLOCK_BASE_BYTES_V2;
            let mut enrollments = 0usize;
            for _ in 0..operation_count {
                let len = input.u32()? as usize;
                if len > config.limits().max_operation_bytes as usize {
                    return Err(ReplayErrorV2::OperationBytesLimit);
                }
                if len != ENROLL_OPERATION_LEN_V2 && len != REVIEW_OPERATION_LEN_V2 {
                    return Err(ReplayErrorV2::InvalidOperationLength);
                }
                block_bytes = block_bytes
                    .checked_add(4 + len)
                    .ok_or(ReplayErrorV2::BlockBytesLimit)?;
                if block_bytes > config.limits().max_block_bytes as usize {
                    return Err(ReplayErrorV2::BlockBytesLimit);
                }
                let operation = SignedMembershipOperationV2::decode(config, input.take(len)?)
                    .map_err(ReplayErrorV2::Operation)?;
                if matches!(operation.action(), MembershipActionV2::Enroll { .. }) {
                    enrollments += 1;
                    if enrollments > config.limits().max_new_enrollments as usize {
                        return Err(ReplayErrorV2::EnrollmentLimit);
                    }
                }
                operations.push(operation);
            }
            groups.push(ReplayGroupV2 {
                height,
                block_hash,
                operations,
            });
        }
        if input.remaining() != 0 {
            return Err(ReplayErrorV2::TrailingBytes);
        }
        Ok(Self {
            config: config.clone(),
            genesis_hash,
            groups,
        })
    }

    /// Re-execute every group through B3. Nothing is published until all roots,
    /// authorization, group commitments and final checkpoint checks succeed.
    /// Supplied metadata is a trust boundary, not a cryptographic chain proof.
    pub fn replay<V: ReplayChainViewV2 + ?Sized>(
        &self,
        bootstrap: &VerifiedBootstrap,
        history: &V,
        expected: ReplayExpectationV2,
    ) -> Result<MembershipRegistryV2, ReplayErrorV2> {
        transcript_size(&self.config, &self.groups)?;
        let context = self.config_hash();
        if bootstrap.config() != &self.config || expected.config_hash != context {
            return Err(ReplayErrorV2::WrongContext);
        }
        // Do not iterate to an unbounded claimed expected height.
        if expected.height != self.groups.len() as u64 {
            return Err(ReplayErrorV2::UnexpectedHeight);
        }
        if history.config_hash() != context {
            return Err(ReplayErrorV2::WrongHistoryContext);
        }
        let mut metadata = reserved_vec(self.groups.len() + 1)?;
        for height in 0..=self.groups.len() {
            let height = height as u64;
            let trusted = history
                .block_at(height)
                .ok_or(ReplayErrorV2::MissingTrustedBlock { height })?;
            let (claimed_hash, digest) = if height == 0 {
                (
                    self.genesis_hash,
                    replay_group_commitment_v2(&self.config, 0, &[])?,
                )
            } else {
                let group = &self.groups[height as usize - 1];
                (group.block_hash, group.commitment(context)?)
            };
            if claimed_hash != trusted.block_hash {
                return Err(ReplayErrorV2::BlockHashMismatch { height });
            }
            if digest != trusted.group_commitment {
                return Err(ReplayErrorV2::GroupCommitmentMismatch { height });
            }
            metadata.push(trusted);
        }
        let last = metadata.last().ok_or(ReplayErrorV2::UnexpectedHeight)?;
        if expected.tip_hash != last.block_hash {
            return Err(ReplayErrorV2::UnexpectedTip);
        }
        if expected.membership_root != last.membership_root {
            return Err(ReplayErrorV2::UnexpectedRoot);
        }

        let mut work = ReplayWork { used: 0 };
        let config_items = (self.config.bootstrap().len() as u64)
            .checked_add(self.config.allocations().len() as u64)
            .ok_or(ReplayErrorV2::WorkLimit)?;
        work.charge(
            config_items
                .checked_add(bootstrap.seeds().len() as u64)
                .ok_or(ReplayErrorV2::WorkLimit)?,
        )?;
        let genesis = MembershipRegistryV2::bootstrap(bootstrap)
            .map_err(|error| ReplayErrorV2::State { height: 0, error })?;
        if genesis.state().root().map_err(ReplayErrorV2::Codec)? != metadata[0].membership_root {
            return Err(ReplayErrorV2::StateRootMismatch { height: 0 });
        }
        let mut registry = genesis.bind_to_validated_block(metadata[0].block_hash);
        for group in &self.groups {
            let incoming_items = group
                .operations
                .iter()
                .filter(|operation| {
                    matches!(
                        operation.action(),
                        MembershipActionV2::Enroll { .. } | MembershipActionV2::Approve { .. }
                    )
                })
                .count() as u64;
            let items = (registry.state().record_count() as u64)
                .checked_add(registry.state().retained_approval_count())
                .and_then(|n| n.checked_add(incoming_items))
                .and_then(|n| n.checked_add(config_items))
                .ok_or(ReplayErrorV2::WorkLimit)?;
            work.charge(items)?;
            let parent = ReplayParent {
                context,
                height: group.height - 1,
                metadata: &metadata,
            };
            let staged = registry
                .stage_block(&parent, &group.operations)
                .map_err(|error| ReplayErrorV2::State {
                    height: group.height,
                    error,
                })?;
            let trusted = metadata[group.height as usize];
            if staged.state().root().map_err(ReplayErrorV2::Codec)? != trusted.membership_root {
                return Err(ReplayErrorV2::StateRootMismatch {
                    height: group.height,
                });
            }
            registry = staged.bind_to_validated_block(trusted.block_hash);
        }
        // This is the only return of an authoritative type. It was rebuilt from
        // verified bootstrap and exact groups bound to the trusted checkpoint.
        Ok(registry)
    }
}

/// SHA256d(domain || C || height:u64 || count:u32 || repeated(len:u32, signed op)).
/// All integers little endian, domain has no terminator. Empty genesis is allowed.
pub fn replay_group_commitment_v2(
    config: &ConfigV2,
    height: u64,
    operations: &[SignedMembershipOperationV2],
) -> Result<Hash256, ReplayErrorV2> {
    operation_bytes(config, operations)?;
    group_commitment(config.context(), height, operations)
}

fn group_commitment(
    context: Hash256,
    height: u64,
    operations: &[SignedMembershipOperationV2],
) -> Result<Hash256, ReplayErrorV2> {
    let count = u32::try_from(operations.len()).map_err(|_| ReplayErrorV2::OperationLimit)?;
    let size = operations
        .iter()
        .try_fold(GROUP_DOMAIN.len() + 32 + 8 + 4, |n, operation| {
            n.checked_add(4 + operation.encoded_len())
                .ok_or(ReplayErrorV2::BlockBytesLimit)
        })?;
    let mut bytes = reserved_vec(size)?;
    bytes.extend_from_slice(GROUP_DOMAIN);
    bytes.extend_from_slice(context.as_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    for operation in operations {
        bytes.extend_from_slice(&(operation.encoded_len() as u32).to_le_bytes());
        bytes.extend_from_slice(&operation.encode());
    }
    Ok(sha256d(&bytes))
}

fn append_operations(operations: &[SignedMembershipOperationV2], bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(operations.len() as u32).to_le_bytes());
    for operation in operations {
        bytes.extend_from_slice(&(operation.encoded_len() as u32).to_le_bytes());
        bytes.extend_from_slice(&operation.encode());
    }
}

fn check_operation_count(config: &ConfigV2, count: usize) -> Result<(), ReplayErrorV2> {
    if count > config.limits().max_operations as usize {
        return Err(ReplayErrorV2::OperationLimit);
    }
    if count > config.limits().max_membership_operations as usize {
        return Err(ReplayErrorV2::MembershipOperationLimit);
    }
    Ok(())
}

fn operation_bytes(
    config: &ConfigV2,
    operations: &[SignedMembershipOperationV2],
) -> Result<usize, ReplayErrorV2> {
    check_operation_count(config, operations.len())?;
    let mut size = MEMBERSHIP_BLOCK_BASE_BYTES_V2;
    let mut enrollments = 0usize;
    for operation in operations {
        if operation.encoded_len() > config.limits().max_operation_bytes as usize {
            return Err(ReplayErrorV2::OperationBytesLimit);
        }
        size = size
            .checked_add(4 + operation.encoded_len())
            .ok_or(ReplayErrorV2::BlockBytesLimit)?;
        if size > config.limits().max_block_bytes as usize {
            return Err(ReplayErrorV2::BlockBytesLimit);
        }
        if matches!(operation.action(), MembershipActionV2::Enroll { .. }) {
            enrollments += 1;
            if enrollments > config.limits().max_new_enrollments as usize {
                return Err(ReplayErrorV2::EnrollmentLimit);
            }
        }
    }
    Ok(size - MEMBERSHIP_BLOCK_BASE_BYTES_V2)
}

fn transcript_size(config: &ConfigV2, groups: &[ReplayGroupV2]) -> Result<usize, ReplayErrorV2> {
    if groups.len() > MAX_REPLAY_BLOCKS_V2 {
        return Err(ReplayErrorV2::GroupLimit);
    }
    let mut size = REPLAY_PREFIX_LEN_V2;
    for (index, group) in groups.iter().enumerate() {
        if group.height != index as u64 + 1 {
            return Err(ReplayErrorV2::NonContiguousHeight);
        }
        let operation_size = operation_bytes(config, &group.operations)?;
        size = size
            .checked_add(REPLAY_GROUP_BASE_LEN_V2)
            .and_then(|n| n.checked_add(operation_size))
            .ok_or(ReplayErrorV2::LogBytesLimit)?;
        if size > config.limits().max_log_bytes as usize {
            return Err(ReplayErrorV2::LogBytesLimit);
        }
    }
    if size > config.limits().max_log_bytes as usize {
        return Err(ReplayErrorV2::LogBytesLimit);
    }
    Ok(size)
}

fn reserved_vec<T>(capacity: usize) -> Result<Vec<T>, ReplayErrorV2> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| ReplayErrorV2::AllocationFailed)?;
    Ok(values)
}

struct ReplayParent<'a> {
    context: Hash256,
    height: u64,
    metadata: &'a [TrustedReplayBlockV2],
}
impl ParentChainViewV2 for ReplayParent<'_> {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        self.height
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        if height > self.height {
            return None;
        }
        let index = usize::try_from(height).ok()?;
        self.metadata.get(index).map(|record| record.block_hash)
    }
}

struct ReplayWork {
    used: u64,
}
impl ReplayWork {
    fn charge(&mut self, items: u64) -> Result<(), ReplayErrorV2> {
        let next = self
            .used
            .checked_add(items)
            .ok_or(ReplayErrorV2::WorkLimit)?;
        if next > MAX_REPLAY_STATE_ITEMS_V2 {
            return Err(ReplayErrorV2::WorkLimit);
        }
        self.used = next;
        Ok(())
    }
}

struct Input<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Input<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
    fn require(&self, count: usize) -> Result<(), ReplayErrorV2> {
        if count > self.remaining() {
            Err(ReplayErrorV2::Truncated)
        } else {
            Ok(())
        }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], ReplayErrorV2> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(ReplayErrorV2::Truncated)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(ReplayErrorV2::Truncated)?;
        self.position = end;
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], ReplayErrorV2> {
        self.take(N)?
            .try_into()
            .map_err(|_| ReplayErrorV2::Truncated)
    }
    fn u16(&mut self) -> Result<u16, ReplayErrorV2> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32, ReplayErrorV2> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64, ReplayErrorV2> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    fn hash(&mut self) -> Result<Hash256, ReplayErrorV2> {
        Ok(Hash256::from_bytes(self.array()?))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayErrorV2 {
    InvalidMagic,
    UnsupportedVersion,
    WrongContext,
    WrongHistoryContext,
    Truncated,
    TrailingBytes,
    NonContiguousHeight,
    GroupLimit,
    OperationLimit,
    MembershipOperationLimit,
    EnrollmentLimit,
    OperationBytesLimit,
    BlockBytesLimit,
    LogBytesLimit,
    InvalidOperationLength,
    AllocationFailed,
    WorkLimit,
    UnexpectedHeight,
    UnexpectedTip,
    UnexpectedRoot,
    MissingTrustedBlock { height: u64 },
    BlockHashMismatch { height: u64 },
    GroupCommitmentMismatch { height: u64 },
    StateRootMismatch { height: u64 },
    Operation(OperationErrorV2),
    State { height: u64, error: StateErrorV2 },
    Codec(StateCodecErrorV2),
}
impl fmt::Display for ReplayErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 diagnostic replay rejected: {self:?}")
    }
}
impl std::error::Error for ReplayErrorV2 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_budget_accepts_exact_limit_and_rejects_next_item_without_mutation() {
        let mut work = ReplayWork { used: 0 };
        work.charge(MAX_REPLAY_STATE_ITEMS_V2 - 1).unwrap();
        work.charge(1).unwrap();
        assert_eq!(work.charge(1), Err(ReplayErrorV2::WorkLimit));
        assert_eq!(work.used, MAX_REPLAY_STATE_ITEMS_V2);
    }

    #[test]
    fn work_budget_never_wraps_or_consumes_a_rejected_charge() {
        let mut work = ReplayWork { used: 1 };
        assert_eq!(work.charge(u64::MAX), Err(ReplayErrorV2::WorkLimit));
        assert_eq!(work.used, 1);
        work.charge(1).unwrap();
        assert_eq!(work.used, 2);
    }
}
