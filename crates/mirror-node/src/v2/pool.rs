//! MB-04C1: bounded local retention and evaluation of signed V2 operations.
//!
//! Retention establishes contextual validity at one accepted parent, not funds,
//! sequence availability or canonical inclusion. Evaluation stages one exact FIFO
//! prefix once and discards the staged ledger. Only explicit reconciliation reports
//! exact inclusion in an immutable accepted chain. No pool API grants publication
//! authority, signs, mines, persists, or mutates the chain or node controller.
//!
//! The cumulative work meter is conservative abstract accounting for this owner,
//! not a CPU-time, RSS or process-wide rate guarantee. See
//! `docs/PENDING_POOL_V2.md` for the selected scope and exact charge formulas.

use std::fmt;

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2};
use mirror_core::v2::{
    BlockErrorV2, MIN_ENCODED_BLOCK_LEN_V2, SignedOperationV2, UnifiedOperationErrorV2,
    validate_operation_limits_v2,
};
use mirror_crypto::Hash256;
use mirror_state::v2::LedgerErrorV2;

/// Independent local retention ceiling, not canonical membership `max_pending`.
pub const MAX_POOL_ENTRIES_V2: usize = 256;
/// Largest existing canonical signed V2 operation (an enrollment).
pub const MAX_POOL_OPERATION_BYTES_V2: usize = 291;
/// Sum of retained operation encodings; excludes allocator and block overhead.
pub const MAX_POOL_ENCODED_BYTES_V2: usize = MAX_POOL_ENTRIES_V2 * MAX_POOL_OPERATION_BYTES_V2;
/// A single requested FIFO prefix never exceeds this local operation count.
pub const MAX_POOL_BATCH_OPERATIONS_V2: usize = 64;
/// Complete MRB2 size, including its fixed header/count and per-operation lengths.
pub const MAX_POOL_BATCH_BYTES_V2: usize =
    MIN_ENCODED_BLOCK_LEN_V2 + MAX_POOL_BATCH_OPERATIONS_V2 * (4 + MAX_POOL_OPERATION_BYTES_V2);
/// Cumulative abstract validation/revalidation allowance for one local owner.
pub const MAX_POOL_WORK_UNITS_V2: u64 = 67_108_864;

/// Explicit immutable local policy. All fields must be positive and within their
/// independent hard ceilings; batch bytes must also fit an empty MRB2 encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingPoolLimitsV2 {
    pub max_entries: usize,
    pub max_encoded_bytes: usize,
    pub max_batch_operations: usize,
    pub max_batch_bytes: usize,
    pub max_work_units: u64,
}

impl PendingPoolLimitsV2 {
    fn validate(self) -> Result<(), PendingPoolErrorV2> {
        if self.max_entries == 0
            || self.max_entries > MAX_POOL_ENTRIES_V2
            || self.max_encoded_bytes == 0
            || self.max_encoded_bytes > MAX_POOL_ENCODED_BYTES_V2
            || self.max_batch_operations == 0
            || self.max_batch_operations > MAX_POOL_BATCH_OPERATIONS_V2
            || self.max_batch_operations > self.max_entries
            || self.max_batch_bytes < MIN_ENCODED_BLOCK_LEN_V2
            || self.max_batch_bytes > MAX_POOL_BATCH_BYTES_V2
            || self.max_work_units == 0
            || self.max_work_units > MAX_POOL_WORK_UNITS_V2
        {
            return Err(PendingPoolErrorV2::InvalidLimits);
        }
        Ok(())
    }
}

/// One retained-input observation, without a spendability or inclusion promise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolAdmissionV2 {
    Retained { id: Hash256 },
    Duplicate { id: Hash256 },
}

/// Historical local evidence that these exact operations executed in this order
/// against the stated accepted parent. It contains no staged state or authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluatedBatchV2 {
    parent: ChainReplayExpectationV2,
    revision: u64,
    operations: Vec<SignedOperationV2>,
    ids: Vec<Hash256>,
    encoded_block_bytes: usize,
}

impl EvaluatedBatchV2 {
    pub const fn parent(&self) -> ChainReplayExpectationV2 {
        self.parent
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn operations(&self) -> &[SignedOperationV2] {
        &self.operations
    }

    pub fn ids(&self) -> &[Hash256] {
        &self.ids
    }

    pub const fn encoded_block_bytes(&self) -> usize {
        self.encoded_block_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolRemovalReasonV2 {
    Included,
    Context(UnifiedOperationErrorV2),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolRemovalV2 {
    pub id: Hash256,
    pub reason: PoolRemovalReasonV2,
}

/// FIFO removal observations from one complete reconciliation. The report does
/// not choose a chain or restore operations removed during an earlier call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolReconciliationV2 {
    pub previous: ChainReplayExpectationV2,
    pub selected: ChainReplayExpectationV2,
    pub revision: u64,
    pub removals: Vec<PoolRemovalV2>,
}

/// A bounded owner whose work allowance is never replenished by housekeeping,
/// failed calls or reconciliation. It deliberately does not implement `Clone`.
#[derive(Debug)]
pub struct PendingPoolV2 {
    limits: PendingPoolLimitsV2,
    parent: ChainReplayExpectationV2,
    revision: u64,
    operations: Vec<SignedOperationV2>,
    ids: Vec<Hash256>,
    encoded_bytes: usize,
    work_used: u64,
}

impl PendingPoolV2 {
    /// Bind an empty owner to an actual accepted parent. Reading its checkpoint
    /// is constant-time and performs no charged validation or history scan.
    pub fn new(chain: &ChainV2, limits: PendingPoolLimitsV2) -> Result<Self, PendingPoolErrorV2> {
        limits.validate()?;
        Ok(Self {
            limits,
            parent: chain.checkpoint(),
            revision: 0,
            operations: Vec::new(),
            ids: Vec::new(),
            encoded_bytes: 0,
            work_used: 0,
        })
    }

    pub const fn limits(&self) -> PendingPoolLimitsV2 {
        self.limits
    }

    pub const fn parent(&self) -> ChainReplayExpectationV2 {
        self.parent
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub const fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    pub const fn work_used(&self) -> u64 {
        self.work_used
    }

    pub fn operations(&self) -> &[SignedOperationV2] {
        &self.operations
    }

    pub fn ids(&self) -> &[Hash256] {
        &self.ids
    }

    /// Retain one structurally decoded, contextually valid signed operation.
    /// Deduplication is only against retained signed IDs, including when full.
    /// Already included operations may be retained until explicit reconciliation.
    pub fn admit(
        &mut self,
        chain: &ChainV2,
        bytes: &[u8],
    ) -> Result<PoolAdmissionV2, PendingPoolErrorV2> {
        self.check_parent(chain)?;
        if bytes.len() > MAX_POOL_OPERATION_BYTES_V2
            || bytes.len() > chain.config().limits().max_operation_bytes as usize
        {
            return Err(PendingPoolErrorV2::InputTooLarge);
        }

        let length = count_u64(bytes.len())?;
        let identify = add(add(1, length)?, count_u64(self.len())?)?;
        self.reserve_work(identify)?;
        let operation = SignedOperationV2::decode(chain.config(), bytes)
            .map_err(PendingPoolErrorV2::Operation)?;
        let id = operation.id();
        if self.ids.contains(&id) {
            return Ok(PoolAdmissionV2::Duplicate { id });
        }

        if self.len() >= self.limits.max_entries {
            return Err(PendingPoolErrorV2::EntryLimit);
        }
        let encoded_bytes = self
            .encoded_bytes
            .checked_add(operation.encoded_len())
            .ok_or(PendingPoolErrorV2::ArithmeticOverflow)?;
        if encoded_bytes > self.limits.max_encoded_bytes {
            return Err(PendingPoolErrorV2::EncodedByteLimit);
        }
        let revision = self.next_revision()?;
        self.reserve_work(add(1, length)?)?;
        operation
            .validate_for_parent(chain.config(), chain)
            .map_err(PendingPoolErrorV2::Operation)?;

        self.operations
            .try_reserve_exact(1)
            .map_err(|_| PendingPoolErrorV2::AllocationFailed)?;
        self.ids
            .try_reserve_exact(1)
            .map_err(|_| PendingPoolErrorV2::AllocationFailed)?;
        self.operations.push(operation);
        self.ids.push(id);
        self.encoded_bytes = encoded_bytes;
        self.revision = revision;
        Ok(PoolAdmissionV2::Retained { id })
    }

    /// Execute exactly one selected FIFO prefix once against this accepted
    /// parent, then discard the staged state. Failures retain their original
    /// mixed-operation index and do not remove or reorder any input.
    pub fn evaluate(
        &mut self,
        chain: &ChainV2,
        count: usize,
    ) -> Result<EvaluatedBatchV2, PendingPoolErrorV2> {
        self.check_parent(chain)?;
        if count == 0 || count > self.len() || count > self.limits.max_batch_operations {
            return Err(PendingPoolErrorV2::InvalidBatchCount);
        }
        let prefix = &self.operations[..count];
        let encoded_operations = prefix.iter().try_fold(0usize, |total, operation| {
            total
                .checked_add(operation.encoded_len())
                .ok_or(PendingPoolErrorV2::ArithmeticOverflow)
        })?;
        let local_block_bytes = count
            .checked_mul(4)
            .and_then(|lengths| lengths.checked_add(MIN_ENCODED_BLOCK_LEN_V2))
            .and_then(|base| base.checked_add(encoded_operations))
            .ok_or(PendingPoolErrorV2::ArithmeticOverflow)?;
        if local_block_bytes > self.limits.max_batch_bytes {
            return Err(PendingPoolErrorV2::BatchByteLimit);
        }
        let encoded_block_bytes = validate_operation_limits_v2(chain.config(), prefix)
            .map_err(PendingPoolErrorV2::Body)?;

        let work = add(
            add(
                add(1, state_weight(chain)?)?,
                multiply(4, count_u64(count)?)?,
            )?,
            count_u64(encoded_operations)?,
        )?;
        self.reserve_work(work)?;
        let operations = copy_slice(&self.operations[..count])?;
        let ids = copy_slice(&self.ids[..count])?;
        let staged = chain
            .ledger()
            .stage_operations(chain, &operations)
            .map_err(PendingPoolErrorV2::Ledger)?;
        drop(staged);
        Ok(EvaluatedBatchV2 {
            parent: self.parent,
            revision: self.revision,
            operations,
            ids,
            encoded_block_bytes,
        })
    }

    /// Check whether this historical batch still matches this pool's exact
    /// selected prefix and actual accepted parent. This is never an append permit.
    pub fn batch_is_current(&self, chain: &ChainV2, batch: &EvaluatedBatchV2) -> bool {
        chain.checkpoint() == self.parent
            && batch.parent == self.parent
            && batch.revision == self.revision
            && self.operations.get(..batch.operations.len()) == Some(batch.operations.as_slice())
            && self.ids.get(..batch.ids.len()) == Some(batch.ids.as_slice())
    }

    /// Atomically reconcile against a selected accepted parent of the same
    /// network, including the unchanged parent. Charge the complete scan first;
    /// exact historical inclusion takes precedence over contextual removal.
    pub fn reconcile(
        &mut self,
        chain: &ChainV2,
    ) -> Result<PoolReconciliationV2, PendingPoolErrorV2> {
        let selected = chain.checkpoint();
        self.check_network(selected)?;
        let revision = self.next_revision()?;
        let work = self.reconciliation_work(chain)?;
        self.reserve_work(work)?;

        let count = self.len();
        let mut operations = reserved_vec(count)?;
        let mut ids = reserved_vec(count)?;
        let mut removals = reserved_vec(count)?;
        let mut included = [false; MAX_POOL_ENTRIES_V2];
        if count != 0 {
            for block in chain.blocks() {
                for operation in block.operations() {
                    let id = operation.id();
                    for (index, retained_id) in self.ids.iter().enumerate() {
                        if id == *retained_id {
                            included[index] = true;
                        }
                    }
                }
            }
        }

        let mut encoded_bytes = 0usize;
        for (index, (&operation, &id)) in self.operations.iter().zip(&self.ids).enumerate() {
            let reason = if included[index] {
                Some(PoolRemovalReasonV2::Included)
            } else {
                operation
                    .validate_for_parent(chain.config(), chain)
                    .err()
                    .map(PoolRemovalReasonV2::Context)
            };
            if let Some(reason) = reason {
                removals.push(PoolRemovalV2 { id, reason });
            } else {
                encoded_bytes = encoded_bytes
                    .checked_add(operation.encoded_len())
                    .ok_or(PendingPoolErrorV2::ArithmeticOverflow)?;
                operations.push(operation);
                ids.push(id);
            }
        }

        let previous = self.parent;
        self.operations = operations;
        self.ids = ids;
        self.encoded_bytes = encoded_bytes;
        self.parent = selected;
        self.revision = revision;
        Ok(PoolReconciliationV2 {
            previous,
            selected,
            revision,
            removals,
        })
    }

    /// Remove exactly one retained signed ID, preserving survivor order and all
    /// consumed work. An absent ID does not change the revision.
    pub fn remove(&mut self, id: Hash256) -> Result<bool, PendingPoolErrorV2> {
        let Some(index) = self.ids.iter().position(|retained| *retained == id) else {
            return Ok(false);
        };
        let revision = self.next_revision()?;
        let encoded_bytes = self
            .encoded_bytes
            .checked_sub(self.operations[index].encoded_len())
            .ok_or(PendingPoolErrorV2::ArithmeticOverflow)?;
        self.operations.remove(index);
        self.ids.remove(index);
        self.encoded_bytes = encoded_bytes;
        self.revision = revision;
        Ok(true)
    }

    /// Remove all retained inputs without replenishing work or rebinding the
    /// selected parent. An already-empty owner is unchanged.
    pub fn clear(&mut self) -> Result<usize, PendingPoolErrorV2> {
        let count = self.len();
        if count == 0 {
            return Ok(0);
        }
        let revision = self.next_revision()?;
        self.operations.clear();
        self.ids.clear();
        self.encoded_bytes = 0;
        self.revision = revision;
        Ok(count)
    }

    fn check_network(&self, selected: ChainReplayExpectationV2) -> Result<(), PendingPoolErrorV2> {
        if selected.config_hash != self.parent.config_hash
            || selected.genesis_hash != self.parent.genesis_hash
        {
            return Err(PendingPoolErrorV2::WrongNetwork);
        }
        Ok(())
    }

    fn check_parent(&self, chain: &ChainV2) -> Result<(), PendingPoolErrorV2> {
        let selected = chain.checkpoint();
        self.check_network(selected)?;
        if selected != self.parent {
            return Err(PendingPoolErrorV2::ParentChanged);
        }
        Ok(())
    }

    fn next_revision(&self) -> Result<u64, PendingPoolErrorV2> {
        self.revision
            .checked_add(1)
            .ok_or(PendingPoolErrorV2::ArithmeticOverflow)
    }

    fn reserve_work(&mut self, work: u64) -> Result<(), PendingPoolErrorV2> {
        let used = add(self.work_used, work)?;
        if used > self.limits.max_work_units {
            return Err(PendingPoolErrorV2::WorkLimit);
        }
        self.work_used = used;
        Ok(())
    }

    fn reconciliation_work(&self, chain: &ChainV2) -> Result<u64, PendingPoolErrorV2> {
        if self.is_empty() {
            return Ok(1);
        }
        let history_operations = chain.blocks().iter().try_fold(0u64, |total, block| {
            add(total, count_u64(block.operations().len())?)
        })?;
        let count = count_u64(self.len())?;
        let history = add(
            add(1, count_u64(chain.blocks().len())?)?,
            count_u64(chain.encoded_bytes())?,
        )?;
        let comparisons = multiply(history_operations, add(count, 1)?)?;
        add(
            add(add(history, comparisons)?, multiply(3, count)?)?,
            count_u64(self.encoded_bytes)?,
        )
    }
}

fn state_weight(chain: &ChainV2) -> Result<u64, PendingPoolErrorV2> {
    let config = chain.config();
    let ledger = chain.ledger();
    let mut weight = add(
        count_u64(config.bootstrap().len())?,
        count_u64(config.allocations().len())?,
    )?;
    weight = add(weight, count_u64(ledger.account_count())?)?;
    weight = add(
        weight,
        multiply(4, count_u64(ledger.membership().record_count())?)?,
    )?;
    add(weight, ledger.membership().retained_approval_count())
}

fn count_u64(count: usize) -> Result<u64, PendingPoolErrorV2> {
    u64::try_from(count).map_err(|_| PendingPoolErrorV2::ArithmeticOverflow)
}

fn add(left: u64, right: u64) -> Result<u64, PendingPoolErrorV2> {
    left.checked_add(right)
        .ok_or(PendingPoolErrorV2::ArithmeticOverflow)
}

fn multiply(left: u64, right: u64) -> Result<u64, PendingPoolErrorV2> {
    left.checked_mul(right)
        .ok_or(PendingPoolErrorV2::ArithmeticOverflow)
}

fn reserved_vec<T>(count: usize) -> Result<Vec<T>, PendingPoolErrorV2> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| PendingPoolErrorV2::AllocationFailed)?;
    Ok(values)
}

fn copy_slice<T: Copy>(values: &[T]) -> Result<Vec<T>, PendingPoolErrorV2> {
    let mut copy = reserved_vec(values.len())?;
    copy.extend_from_slice(values);
    Ok(copy)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingPoolErrorV2 {
    InvalidLimits,
    WrongNetwork,
    ParentChanged,
    InputTooLarge,
    EntryLimit,
    EncodedByteLimit,
    InvalidBatchCount,
    BatchByteLimit,
    WorkLimit,
    ArithmeticOverflow,
    AllocationFailed,
    Operation(UnifiedOperationErrorV2),
    Body(BlockErrorV2),
    Ledger(LedgerErrorV2),
}

impl fmt::Display for PendingPoolErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operation(error) => write!(f, "V2 pending input rejected: {error}"),
            Self::Body(error) => write!(f, "V2 pending batch rejected: {error}"),
            Self::Ledger(error) => write!(f, "V2 pending execution rejected: {error}"),
            _ => write!(f, "V2 pending pool rejected: {self:?}"),
        }
    }
}

impl std::error::Error for PendingPoolErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Operation(error) => Some(error),
            Self::Body(error) => Some(error),
            Self::Ledger(error) => Some(error),
            _ => None,
        }
    }
}
