//! Bounded in-memory acceptance of complete synthetic V2 block histories.
//!
//! Every ledger is bound only after validating an actual block against a pinned
//! manifest and the immutable accepted ancestry of this chain. Independent valid
//! branches are possible. The separate fixed-target development comparator
//! chooses between accepted branches without publication or finality. This module
//! supplies no network service, monetary issuance or proof of real humanity.

pub mod fork_choice;
pub mod manifest;

pub use fork_choice::{
    BranchComparisonV2, BranchRelationV2, BranchSideV2, ForkChoiceErrorV2, ForkChoiceV2,
    compare_branches_v2,
};
pub use manifest::{ManifestErrorV2, ManifestIdentityV2, NetworkManifestV2};

use std::fmt;

use mirror_consensus::v2::{PowErrorV2, validate_header_pow_v2};
use mirror_core::v2::{
    BLOCK_VERSION_V2, BlockErrorV2, BlockHeaderV2, BlockV2, SignedOperationV2, operations_root_v2,
    validate_operation_limits_v2,
};
use mirror_crypto::Hash256;
use mirror_personhood::v2::{ConfigV2, MembershipActionV2, ParentChainViewV2, VerifiedBootstrap};
use mirror_state::v2::{LedgerErrorV2, LedgerV2};

/// Local synthetic profile ceiling, including the genesis block.
pub const MAX_CHAIN_BLOCKS_V2: usize = 4_096;
/// Sum of the complete canonical MRB2 sizes, including genesis and empty blocks.
pub const MAX_CHAIN_BYTES_V2: usize = 16 * 1024 * 1024;
/// Cumulative abstract state/configuration work charge; not a population limit.
pub const MAX_CHAIN_STATE_ITEMS_V2: u64 = 1_048_576;

/// TRUSTED CALLER INPUT. Select this checkpoint independently of replay bytes.
/// It prevents accepting an unintended old prefix or branch; it does not discover
/// the latest tip or make the selected branch globally final.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainReplayExpectationV2 {
    pub config_hash: Hash256,
    pub genesis_hash: Hash256,
    pub height: u64,
    pub tip_hash: Hash256,
    pub ledger_root: Hash256,
}

/// A fully validated finite branch and its executed accounts/membership ledger.
/// Fields and ancestry are private; callers cannot attach arbitrary parent views
/// or replace the accepted tip independently of its ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainV2 {
    manifest: NetworkManifestV2,
    blocks: Vec<BlockV2>,
    ledger: LedgerV2,
    encoded_bytes: usize,
    state_items_used: u64,
}

impl ChainV2 {
    /// Validate the exact configured genesis before making any state available.
    /// A manifest pin is necessary but insufficient: even a pinned malformed
    /// genesis must pass work, empty-body and independently executed root checks.
    pub fn from_genesis(
        manifest: &NetworkManifestV2,
        genesis: BlockV2,
    ) -> Result<Self, ChainErrorV2> {
        let config = manifest.config();
        let length = validate_operation_limits_v2(config, genesis.operations())?;
        let resources = Resources::default().next(length, genesis_charge(config)?)?;
        let header = genesis.header();
        check_context(config, header)?;
        check_height(header, 0)?;
        if header.previous_hash() != Hash256::default() {
            return Err(ChainErrorV2::WrongParent);
        }
        let expected_timestamp = config.parameters().genesis_timestamp;
        if header.timestamp() != expected_timestamp {
            return Err(ChainErrorV2::WrongGenesisTimestamp {
                expected: expected_timestamp,
                got: header.timestamp(),
            });
        }
        if !genesis.operations().is_empty() {
            return Err(ChainErrorV2::GenesisHasOperations);
        }
        validate_header_pow_v2(header, config.parameters().pow_bits)?;
        let hash = genesis.hash();
        if hash != manifest.genesis_hash() {
            return Err(ChainErrorV2::WrongGenesisHash);
        }
        check_operations_root(config, &genesis)?;
        let staged = LedgerV2::bootstrap(manifest.bootstrap())?;
        if staged.root()? != header.ledger_root() {
            return Err(ChainErrorV2::WrongLedgerRoot);
        }
        let mut blocks = Vec::new();
        blocks
            .try_reserve_exact(1)
            .map_err(|_| ChainErrorV2::AllocationFailed)?;
        blocks.push(genesis);
        Ok(Self {
            manifest: manifest.clone(),
            blocks,
            ledger: staged.bind_to_validated_block(hash),
            encoded_bytes: resources.bytes,
            state_items_used: resources.items,
        })
    }

    /// Accept one actual child, with all rejection paths leaving this entire
    /// branch unchanged. The full original body is executed once in order; there
    /// is no signature prepass that could mask an earlier execution failure.
    pub fn append(&mut self, block: BlockV2) -> Result<(), ChainErrorV2> {
        let length = validate_operation_limits_v2(self.config(), block.operations())?;
        let resources = self
            .resources()
            .next(length, self.child_charge(block.operations())?)?;
        let height = self
            .height()
            .checked_add(1)
            .ok_or(ChainErrorV2::HeightOverflow)?;
        check_context(self.config(), block.header())?;
        check_height(block.header(), height)?;
        if block.header().previous_hash() != self.tip_hash() {
            return Err(ChainErrorV2::WrongParent);
        }
        self.check_timestamp(block.header().timestamp())?;
        validate_header_pow_v2(block.header(), self.config().parameters().pow_bits)?;
        check_operations_root(self.config(), &block)?;
        let staged = self.ledger.stage_operations(self, block.operations())?;
        if staged.root()? != block.header().ledger_root() {
            return Err(ChainErrorV2::WrongLedgerRoot);
        }
        // No fallible validation or allocation remains after reserving the slot.
        // Capacity itself is not exposed and is not semantic chain state.
        self.blocks
            .try_reserve(1)
            .map_err(|_| ChainErrorV2::AllocationFailed)?;
        let ledger = staged.bind_to_validated_block(block.hash());
        self.blocks.push(block);
        self.ledger = ledger;
        self.encoded_bytes = resources.bytes;
        self.state_items_used = resources.items;
        Ok(())
    }

    /// Build a nonce-zero proposal from the accepted parent without mutating it.
    /// Body execution and budgets are checked, but the result is not accepted.
    /// After any work search, append still validates the complete actual block.
    pub fn prepare_candidate(
        &self,
        timestamp: u64,
        operations: &[SignedOperationV2],
    ) -> Result<BlockV2, ChainErrorV2> {
        let length = validate_operation_limits_v2(self.config(), operations)?;
        self.resources()
            .next(length, self.child_charge(operations)?)?;
        let height = self
            .height()
            .checked_add(1)
            .ok_or(ChainErrorV2::HeightOverflow)?;
        self.check_timestamp(timestamp)?;
        let staged = self.ledger.stage_operations(self, operations)?;
        let header = BlockHeaderV2::new(
            self.config().context(),
            height,
            self.tip_hash(),
            operations_root_v2(self.config(), operations)?,
            staged.root()?,
            timestamp,
            self.config().parameters().pow_bits,
            0,
        );
        Ok(BlockV2::new(self.config(), header, operations)?)
    }

    /// Rebuild complete typed blocks using the same actual-block acceptance path.
    /// All aggregate count/byte budgets are checked before input-sized cloning
    /// or execution. State-work charges are checked before every transition.
    pub fn replay(
        manifest: &NetworkManifestV2,
        blocks: &[BlockV2],
        expected: ChainReplayExpectationV2,
    ) -> Result<Self, ChainReplayErrorV2> {
        preflight_count(blocks.len())?;
        let mut total = 0usize;
        for (index, block) in blocks.iter().enumerate() {
            let size = validate_operation_limits_v2(manifest.config(), block.operations())
                .map_err(|error| ChainReplayErrorV2::Block {
                    index,
                    error: ChainErrorV2::Body(error),
                })?;
            total = add_replay_bytes(total, size)?;
        }
        check_replay_selection(manifest, blocks.len(), expected)?;
        let mut chain = Self::from_genesis(manifest, blocks[0].clone())
            .map_err(|error| ChainReplayErrorV2::Block { index: 0, error })?;
        for (index, block) in blocks.iter().enumerate().skip(1) {
            chain
                .append(block.clone())
                .map_err(|error| ChainReplayErrorV2::Block { index, error })?;
        }
        chain.check_replayed_tip(expected)?;
        Ok(chain)
    }

    /// Rebuild complete canonical MRB2 blocks, never a cache or filtered event log.
    /// Count and all declared input slice lengths are bounded before decoding any
    /// block, allocating its operations, or checking any operation signature.
    pub fn replay_encoded(
        manifest: &NetworkManifestV2,
        blocks: &[&[u8]],
        expected: ChainReplayExpectationV2,
    ) -> Result<Self, ChainReplayErrorV2> {
        preflight_count(blocks.len())?;
        let mut total = 0usize;
        for (index, bytes) in blocks.iter().enumerate() {
            if bytes.len() > manifest.config().limits().max_block_bytes as usize {
                return Err(ChainReplayErrorV2::Block {
                    index,
                    error: ChainErrorV2::Body(BlockErrorV2::TooLarge),
                });
            }
            total = add_replay_bytes(total, bytes.len())?;
        }
        check_replay_selection(manifest, blocks.len(), expected)?;
        let decode = |index: usize| {
            BlockV2::decode(manifest.config(), blocks[index]).map_err(|error| {
                ChainReplayErrorV2::Block {
                    index,
                    error: ChainErrorV2::Body(error),
                }
            })
        };
        let mut chain = Self::from_genesis(manifest, decode(0)?)
            .map_err(|error| ChainReplayErrorV2::Block { index: 0, error })?;
        for index in 1..blocks.len() {
            chain
                .append(decode(index)?)
                .map_err(|error| ChainReplayErrorV2::Block { index, error })?;
        }
        chain.check_replayed_tip(expected)?;
        Ok(chain)
    }

    pub const fn manifest(&self) -> &NetworkManifestV2 {
        &self.manifest
    }

    pub const fn config(&self) -> &ConfigV2 {
        self.manifest.config()
    }

    pub const fn ledger(&self) -> &LedgerV2 {
        &self.ledger
    }

    pub const fn height(&self) -> u64 {
        self.ledger.height()
    }

    pub const fn tip_hash(&self) -> Hash256 {
        self.ledger.tip_hash()
    }

    pub fn genesis(&self) -> &BlockV2 {
        &self.blocks[0] // Construction always accepts exactly one genesis first.
    }

    pub fn blocks(&self) -> &[BlockV2] {
        &self.blocks
    }

    pub fn block_at(&self, height: u64) -> Option<&BlockV2> {
        self.blocks.get(usize::try_from(height).ok()?)
    }

    pub fn checkpoint(&self) -> ChainReplayExpectationV2 {
        ChainReplayExpectationV2 {
            config_hash: self.config().context(),
            genesis_hash: self.manifest.genesis_hash(),
            height: self.height(),
            tip_hash: self.tip_hash(),
            // This field was compared with the complete executed ledger root
            // before the block became accessible through this immutable view.
            ledger_root: self.blocks[self.blocks.len() - 1].header().ledger_root(),
        }
    }

    pub const fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    pub const fn state_items_used(&self) -> u64 {
        self.state_items_used
    }

    fn resources(&self) -> Resources {
        Resources {
            blocks: self.blocks.len(),
            bytes: self.encoded_bytes,
            items: self.state_items_used,
        }
    }

    fn check_timestamp(&self, timestamp: u64) -> Result<(), ChainErrorV2> {
        let parent = self.blocks[self.blocks.len() - 1].header().timestamp();
        if timestamp <= parent {
            return Err(ChainErrorV2::InvalidTimestamp {
                parent,
                got: timestamp,
            });
        }
        Ok(())
    }

    /// Charge config seed/allocation entries, current accounts, membership
    /// records and retained approvals, plus one potential item per incoming
    /// transfer/enrollment/approval. Withdrawals add no incoming item. This is
    /// conservative abstract accounting, not a count of actual allocations.
    fn child_charge(&self, operations: &[SignedOperationV2]) -> Result<u64, ChainErrorV2> {
        let incoming = operations
            .iter()
            .filter(|operation| {
                !matches!(
                    operation,
                    SignedOperationV2::Membership(operation)
                        if matches!(operation.action(), MembershipActionV2::Withdraw { .. })
                )
            })
            .count();
        let mut charge = config_items(self.config())?;
        for items in [
            self.ledger.account_count(),
            self.ledger.membership().record_count(),
            incoming,
        ] {
            charge = charge
                .checked_add(u64::try_from(items).map_err(|_| ChainErrorV2::LengthOverflow)?)
                .ok_or(ChainErrorV2::LengthOverflow)?;
        }
        charge
            .checked_add(self.ledger.membership().retained_approval_count())
            .ok_or(ChainErrorV2::LengthOverflow)
    }

    fn check_replayed_tip(
        &self,
        expected: ChainReplayExpectationV2,
    ) -> Result<(), ChainReplayErrorV2> {
        if self.tip_hash() != expected.tip_hash {
            return Err(ChainReplayErrorV2::WrongExpectedTip);
        }
        if self.checkpoint().ledger_root != expected.ledger_root {
            return Err(ChainReplayErrorV2::WrongExpectedLedgerRoot);
        }
        Ok(())
    }
}

// Stable accepted ancestry is the only view used by append/prepare/replay.
// This public trait implementation permits read-only downstream use but cannot
// supply or override the chain's own acceptance oracle.
impl ParentChainViewV2 for ChainV2 {
    fn config_hash(&self) -> Hash256 {
        self.config().context()
    }

    fn parent_height(&self) -> u64 {
        self.height()
    }

    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        self.block_at(height).map(BlockV2::hash)
    }
}

/// Exact nonce-zero genesis proposal from verified public synthetic bootstrap.
/// Work search, independent manifest selection and from_genesis acceptance remain
/// separate steps. This helper does not create an accepted chain or trusted tip.
pub fn genesis_template_v2(bootstrap: &VerifiedBootstrap) -> Result<BlockV2, ChainErrorV2> {
    let config = bootstrap.config();
    let length = validate_operation_limits_v2(config, &[])?;
    Resources::default().next(length, genesis_charge(config)?)?;
    let staged = LedgerV2::bootstrap(bootstrap)?;
    let header = BlockHeaderV2::new(
        config.context(),
        0,
        Hash256::default(),
        operations_root_v2(config, &[])?,
        staged.root()?,
        config.parameters().genesis_timestamp,
        config.parameters().pow_bits,
        0,
    );
    Ok(BlockV2::new(config, header, &[])?)
}

fn check_context(config: &ConfigV2, header: &BlockHeaderV2) -> Result<(), ChainErrorV2> {
    if header.version() != BLOCK_VERSION_V2 {
        return Err(ChainErrorV2::Body(BlockErrorV2::UnsupportedVersion(
            header.version(),
        )));
    }
    if header.config_hash() != config.context() {
        return Err(ChainErrorV2::WrongConfiguration);
    }
    Ok(())
}

fn check_height(header: &BlockHeaderV2, expected: u64) -> Result<(), ChainErrorV2> {
    if header.height() != expected {
        return Err(ChainErrorV2::WrongHeight {
            expected,
            got: header.height(),
        });
    }
    Ok(())
}

fn check_operations_root(config: &ConfigV2, block: &BlockV2) -> Result<(), ChainErrorV2> {
    if operations_root_v2(config, block.operations())? != block.header().operations_root() {
        return Err(ChainErrorV2::WrongOperationsRoot);
    }
    Ok(())
}

fn config_items(config: &ConfigV2) -> Result<u64, ChainErrorV2> {
    config
        .bootstrap()
        .len()
        .checked_add(config.allocations().len())
        .and_then(|count| u64::try_from(count).ok())
        .ok_or(ChainErrorV2::LengthOverflow)
}

fn genesis_charge(config: &ConfigV2) -> Result<u64, ChainErrorV2> {
    config_items(config)?
        .checked_mul(2)
        .ok_or(ChainErrorV2::LengthOverflow)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Resources {
    blocks: usize,
    bytes: usize,
    items: u64,
}

impl Resources {
    fn next(self, bytes: usize, items: u64) -> Result<Self, ChainErrorV2> {
        let next = Self {
            blocks: self
                .blocks
                .checked_add(1)
                .ok_or(ChainErrorV2::LengthOverflow)?,
            bytes: self
                .bytes
                .checked_add(bytes)
                .ok_or(ChainErrorV2::LengthOverflow)?,
            items: self
                .items
                .checked_add(items)
                .ok_or(ChainErrorV2::LengthOverflow)?,
        };
        if next.blocks > MAX_CHAIN_BLOCKS_V2 {
            return Err(ChainErrorV2::TooManyBlocks);
        }
        if next.bytes > MAX_CHAIN_BYTES_V2 {
            return Err(ChainErrorV2::TooManyBytes);
        }
        if next.items > MAX_CHAIN_STATE_ITEMS_V2 {
            return Err(ChainErrorV2::StateItemLimit);
        }
        Ok(next)
    }
}

fn preflight_count(count: usize) -> Result<(), ChainReplayErrorV2> {
    if count == 0 {
        return Err(ChainReplayErrorV2::Input(ChainErrorV2::EmptyHistory));
    }
    if count > MAX_CHAIN_BLOCKS_V2 {
        return Err(ChainReplayErrorV2::Input(ChainErrorV2::TooManyBlocks));
    }
    Ok(())
}

fn add_replay_bytes(total: usize, size: usize) -> Result<usize, ChainReplayErrorV2> {
    let next = total
        .checked_add(size)
        .ok_or(ChainReplayErrorV2::Input(ChainErrorV2::LengthOverflow))?;
    if next > MAX_CHAIN_BYTES_V2 {
        return Err(ChainReplayErrorV2::Input(ChainErrorV2::TooManyBytes));
    }
    Ok(next)
}

fn check_replay_selection(
    manifest: &NetworkManifestV2,
    count: usize,
    expected: ChainReplayExpectationV2,
) -> Result<(), ChainReplayErrorV2> {
    if manifest.config().context() != expected.config_hash {
        return Err(ChainReplayErrorV2::WrongExpectedConfiguration);
    }
    if manifest.genesis_hash() != expected.genesis_hash {
        return Err(ChainReplayErrorV2::WrongExpectedGenesis);
    }
    let got = u64::try_from(count - 1)
        .map_err(|_| ChainReplayErrorV2::Input(ChainErrorV2::LengthOverflow))?;
    if got != expected.height {
        return Err(ChainReplayErrorV2::WrongExpectedHeight {
            expected: expected.height,
            got,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainErrorV2 {
    Body(BlockErrorV2),
    Work(PowErrorV2),
    Ledger(LedgerErrorV2),
    WrongConfiguration,
    WrongHeight { expected: u64, got: u64 },
    WrongParent,
    InvalidTimestamp { parent: u64, got: u64 },
    WrongGenesisTimestamp { expected: u64, got: u64 },
    GenesisHasOperations,
    WrongGenesisHash,
    WrongOperationsRoot,
    WrongLedgerRoot,
    HeightOverflow,
    EmptyHistory,
    TooManyBlocks,
    TooManyBytes,
    StateItemLimit,
    LengthOverflow,
    AllocationFailed,
}

impl From<BlockErrorV2> for ChainErrorV2 {
    fn from(error: BlockErrorV2) -> Self {
        Self::Body(error)
    }
}

impl From<PowErrorV2> for ChainErrorV2 {
    fn from(error: PowErrorV2) -> Self {
        Self::Work(error)
    }
}

impl From<LedgerErrorV2> for ChainErrorV2 {
    fn from(error: LedgerErrorV2) -> Self {
        Self::Ledger(error)
    }
}

impl fmt::Display for ChainErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 chain rejected block: {self:?}")
    }
}

impl std::error::Error for ChainErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            Self::Work(error) => Some(error),
            Self::Ledger(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainReplayErrorV2 {
    Input(ChainErrorV2),
    Block { index: usize, error: ChainErrorV2 },
    WrongExpectedConfiguration,
    WrongExpectedGenesis,
    WrongExpectedHeight { expected: u64, got: u64 },
    WrongExpectedTip,
    WrongExpectedLedgerRoot,
}

impl fmt::Display for ChainReplayErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 chain replay rejected: {self:?}")
    }
}

impl std::error::Error for ChainReplayErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(error) | Self::Block { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
