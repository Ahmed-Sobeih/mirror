//! Mirror blockchain orchestration.
//!
//! This crate links blocks into an ordered chain and advances the
//! deterministic account state only after complete validation.

use mirror_consensus::{BlockConsensusError, mine_block, validate_block};

use mirror_core::{Address, BLOCK_VERSION, Block, BlockError, SignedTransaction};

use mirror_crypto::Hash256;

use mirror_state::{
    BlockStateError, ChainState, StateError, execute_transactions, validate_block_state,
};

/// Configuration from which Mirror's deterministic genesis state and
/// genesis block are defined.
///
/// The permanent mainnet genesis configuration will be frozen later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisConfig {
    timestamp: u64,
    pow_bits: u32,
    allocations: Vec<(Address, u64)>,
}

impl GenesisConfig {
    pub fn new(timestamp: u64, pow_bits: u32, allocations: Vec<(Address, u64)>) -> Self {
        Self {
            timestamp,
            pow_bits,
            allocations,
        }
    }

    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub const fn pow_bits(&self) -> u32 {
        self.pow_bits
    }

    pub fn allocations(&self) -> &[(Address, u64)] {
        &self.allocations
    }
}

/// Fully validated in-memory Mirror blockchain.
///
/// Historical blocks are persisted separately by `mirror-storage`.
#[derive(Clone, Debug)]
pub struct Chain {
    blocks: Vec<Block>,
    state: ChainState,

    /// Current required Proof-of-Work target encoding.
    ///
    /// For now Mirror uses a fixed development difficulty.
    /// Difficulty adjustment rules will replace this later.
    pow_bits: u32,
}

impl Chain {
    /// Build and mine a new deterministic development genesis block.
    pub fn from_genesis(config: GenesisConfig) -> Result<Self, ChainError> {
        let state = ChainState::from_genesis_allocations(config.allocations)?;

        let mut genesis = Block::new(
            BLOCK_VERSION,
            Hash256::default(),
            state.state_root(),
            config.timestamp,
            config.pow_bits,
            Vec::new(),
        )?;

        mine_block(&mut genesis)?;

        validate_block(&genesis)?;
        validate_block_state(&state, &genesis)?;

        Ok(Self {
            blocks: vec![genesis],
            state,
            pow_bits: config.pow_bits,
        })
    }

    /// Reconstruct a complete chain from persisted blocks.
    ///
    /// Every block is independently revalidated. State is rebuilt from
    /// genesis allocations and transaction execution rather than trusted
    /// from disk.
    pub fn from_persisted_blocks(
        config: GenesisConfig,
        blocks: Vec<Block>,
    ) -> Result<Self, ChainError> {
        if blocks.is_empty() {
            return Err(ChainError::MissingGenesis);
        }

        let initial_state = ChainState::from_genesis_allocations(config.allocations)?;

        let genesis = &blocks[0];

        if genesis.header().version() != BLOCK_VERSION {
            return Err(ChainError::UnexpectedBlockVersion {
                expected: BLOCK_VERSION,
                got: genesis.header().version(),
            });
        }

        if genesis.header().previous_block_hash() != Hash256::default() {
            return Err(ChainError::InvalidGenesisPreviousHash);
        }

        if !genesis.transactions().is_empty() {
            return Err(ChainError::GenesisContainsTransactions);
        }

        if genesis.header().timestamp() != config.timestamp {
            return Err(ChainError::GenesisTimestampMismatch {
                expected: config.timestamp,
                got: genesis.header().timestamp(),
            });
        }

        if genesis.header().pow_bits() != config.pow_bits {
            return Err(ChainError::UnexpectedDifficulty {
                expected: config.pow_bits,
                got: genesis.header().pow_bits(),
            });
        }

        // Genesis PoW, Merkle root, signatures and state commitment
        // are not trusted merely because they came from local disk.
        validate_block(genesis)?;

        validate_block_state(&initial_state, genesis)?;

        let mut chain = Self {
            blocks: vec![genesis.clone()],
            state: initial_state,
            pow_bits: config.pow_bits,
        };

        for block in blocks.into_iter().skip(1) {
            chain.append_block(block)?;
        }

        Ok(chain)
    }

    /// Genesis is height zero.
    pub fn height(&self) -> u64 {
        (self.blocks.len() - 1) as u64
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn genesis(&self) -> &Block {
        &self.blocks[0]
    }

    pub fn tip(&self) -> &Block {
        self.blocks
            .last()
            .expect("Mirror chain always contains genesis")
    }

    pub fn tip_hash(&self) -> Hash256 {
        self.tip().hash()
    }

    pub const fn state(&self) -> &ChainState {
        &self.state
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub const fn pow_bits(&self) -> u32 {
        self.pow_bits
    }

    /// Construct and mine a candidate block extending the current tip.
    ///
    /// Difficulty comes from chain rules, never from the block producer.
    pub fn mine_next_block(
        &self,
        transactions: Vec<SignedTransaction>,
        timestamp: u64,
    ) -> Result<Block, ChainError> {
        let transition = execute_transactions(&self.state, &transactions)?;

        let mut block = Block::new(
            BLOCK_VERSION,
            self.tip_hash(),
            transition.state_root(),
            timestamp,
            self.pow_bits,
            transactions,
        )?;

        mine_block(&mut block)?;

        Ok(block)
    }

    /// Fully validate and append a block to the current chain tip.
    pub fn append_block(&mut self, block: Block) -> Result<(), ChainError> {
        let expected_previous = self.tip_hash();

        let got_previous = block.header().previous_block_hash();

        if got_previous != expected_previous {
            return Err(ChainError::PreviousBlockMismatch {
                expected: expected_previous,
                got: got_previous,
            });
        }

        if block.header().version() != BLOCK_VERSION {
            return Err(ChainError::UnexpectedBlockVersion {
                expected: BLOCK_VERSION,
                got: block.header().version(),
            });
        }

        // Consensus-critical:
        // the block author does not get to choose an easier target.
        if block.header().pow_bits() != self.pow_bits {
            return Err(ChainError::UnexpectedDifficulty {
                expected: self.pow_bits,
                got: block.header().pow_bits(),
            });
        }

        // Signatures + transaction Merkle root + PoW.
        validate_block(&block)?;

        // Re-execute against our current state and verify the
        // state root committed by the block header.
        let transition = validate_block_state(&self.state, &block)?;

        self.state = transition.into_post_state();

        self.blocks.push(block);

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChainError {
    Block(BlockError),
    Consensus(BlockConsensusError),
    State(StateError),
    BlockState(BlockStateError),

    MissingGenesis,

    InvalidGenesisPreviousHash,

    GenesisContainsTransactions,

    GenesisTimestampMismatch { expected: u64, got: u64 },

    UnexpectedBlockVersion { expected: u32, got: u32 },

    UnexpectedDifficulty { expected: u32, got: u32 },

    PreviousBlockMismatch { expected: Hash256, got: Hash256 },
}

impl core::fmt::Display for ChainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Block(error) => {
                write!(f, "block construction failed: {error}")
            }

            Self::Consensus(error) => {
                write!(f, "block consensus failed: {error}")
            }

            Self::State(error) => {
                write!(f, "state execution failed: {error}")
            }

            Self::BlockState(error) => {
                write!(f, "block state validation failed: {error}")
            }

            Self::MissingGenesis => {
                write!(f, "persisted chain contains no genesis block")
            }

            Self::InvalidGenesisPreviousHash => {
                write!(f, "genesis previous block hash must be zero")
            }

            Self::GenesisContainsTransactions => {
                write!(f, "Mirror genesis block must not contain transactions")
            }

            Self::GenesisTimestampMismatch { expected, got } => {
                write!(
                    f,
                    "genesis timestamp mismatch: expected {expected}, got {got}"
                )
            }

            Self::UnexpectedBlockVersion { expected, got } => {
                write!(
                    f,
                    "unexpected block version: expected {expected}, got {got}"
                )
            }

            Self::UnexpectedDifficulty { expected, got } => {
                write!(
                    f,
                    "unexpected proof-of-work difficulty: expected {expected:#010x}, got {got:#010x}"
                )
            }

            Self::PreviousBlockMismatch { expected, got } => {
                write!(f, "previous block mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for ChainError {}

impl From<BlockError> for ChainError {
    fn from(error: BlockError) -> Self {
        Self::Block(error)
    }
}

impl From<BlockConsensusError> for ChainError {
    fn from(error: BlockConsensusError) -> Self {
        Self::Consensus(error)
    }
}

impl From<StateError> for ChainError {
    fn from(error: StateError) -> Self {
        Self::State(error)
    }
}

impl From<BlockStateError> for ChainError {
    fn from(error: BlockStateError) -> Self {
        Self::BlockState(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mirror_consensus::{INITIAL_POW_BITS, mine_block};

    use mirror_core::{
        MIRROR_CHAIN_ID, NUSA_PER_MRY, TRANSACTION_KIND_TRANSFER, TRANSACTION_VERSION,
        TransactionBody,
    };

    use mirror_crypto::Keypair;

    fn key(secret: u8) -> Keypair {
        Keypair::from_secret_bytes([secret; 32])
    }

    fn address(keypair: &Keypair) -> Address {
        Address::from_public_key(&keypair.public_key())
    }

    fn config(alice: &Keypair) -> GenesisConfig {
        GenesisConfig::new(
            1_800_000_000,
            INITIAL_POW_BITS,
            vec![(address(alice), 100 * NUSA_PER_MRY)],
        )
    }

    fn test_chain(alice: &Keypair) -> Chain {
        Chain::from_genesis(config(alice)).unwrap()
    }

    fn transfer(alice: &Keypair, bob: Address, nonce: u64) -> SignedTransaction {
        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            MIRROR_CHAIN_ID,
            nonce,
            address(alice),
            bob,
            NUSA_PER_MRY,
            0,
            Vec::new(),
        );

        SignedTransaction::sign(body, alice).unwrap()
    }

    #[test]
    fn chain_starts_with_mined_genesis_block() {
        let alice = key(1);

        let chain = test_chain(&alice);

        assert_eq!(chain.height(), 0);
        assert_eq!(chain.len(), 1);

        assert_eq!(
            chain.genesis().header().previous_block_hash(),
            Hash256::default()
        );

        assert_eq!(validate_block(chain.genesis()), Ok(()));

        assert_eq!(
            chain.genesis().header().state_root(),
            chain.state().state_root()
        );
    }

    #[test]
    fn valid_next_block_links_and_updates_state() {
        let alice = key(1);
        let bob = key(2);

        let alice_address = address(&alice);

        let bob_address = address(&bob);

        let mut chain = test_chain(&alice);

        let genesis_hash = chain.tip_hash();

        let block = chain
            .mine_next_block(vec![transfer(&alice, bob_address, 0)], 1_800_000_001)
            .unwrap();

        assert_eq!(block.header().previous_block_hash(), genesis_hash);

        chain.append_block(block).unwrap();

        assert_eq!(chain.height(), 1);
        assert_eq!(chain.len(), 2);

        assert_eq!(
            chain.state().account(alice_address).balance(),
            99 * NUSA_PER_MRY
        );

        assert_eq!(chain.state().account(alice_address).nonce(), 1);

        assert_eq!(chain.state().account(bob_address).balance(), NUSA_PER_MRY);
    }

    #[test]
    fn block_with_wrong_previous_hash_is_rejected() {
        let alice = key(1);

        let mut chain = test_chain(&alice);

        let block = Block::new(
            BLOCK_VERSION,
            Hash256::from_bytes([0x99; 32]),
            chain.state().state_root(),
            1_800_000_001,
            INITIAL_POW_BITS,
            Vec::new(),
        )
        .unwrap();

        assert!(matches!(
            chain.append_block(block),
            Err(ChainError::PreviousBlockMismatch { .. })
        ));
    }

    #[test]
    fn replay_in_later_block_is_rejected() {
        let alice = key(1);
        let bob = key(2);

        let mut chain = test_chain(&alice);

        let first_transaction = transfer(&alice, address(&bob), 0);

        let block_one = chain
            .mine_next_block(vec![first_transaction.clone()], 1_800_000_001)
            .unwrap();

        chain.append_block(block_one).unwrap();

        let result = chain.mine_next_block(vec![first_transaction], 1_800_000_002);

        assert!(matches!(
            result,
            Err(ChainError::State(StateError::InvalidNonce {
                expected: 1,
                got: 0,
            }))
        ));
    }

    #[test]
    fn persisted_chain_rebuilds_state() {
        let alice = key(1);
        let bob = key(2);

        let alice_address = address(&alice);

        let bob_address = address(&bob);

        let mut original = test_chain(&alice);

        let block = original
            .mine_next_block(vec![transfer(&alice, bob_address, 0)], 1_800_000_001)
            .unwrap();

        original.append_block(block).unwrap();

        let persisted = original.blocks().to_vec();

        let restored = Chain::from_persisted_blocks(config(&alice), persisted).unwrap();

        assert_eq!(restored.height(), original.height());

        assert_eq!(restored.tip_hash(), original.tip_hash());

        assert_eq!(restored.state(), original.state());

        assert_eq!(restored.state().account(alice_address).nonce(), 1);

        assert_eq!(
            restored.state().account(bob_address).balance(),
            NUSA_PER_MRY
        );
    }

    #[test]
    fn self_selected_easier_difficulty_is_rejected() {
        let alice = key(1);

        let mut chain = test_chain(&alice);

        // Deliberately much easier than the chain rule.
        let easier_bits = 0x207f_ffff;

        let mut block = Block::new(
            BLOCK_VERSION,
            chain.tip_hash(),
            chain.state().state_root(),
            1_800_000_001,
            easier_bits,
            Vec::new(),
        )
        .unwrap();

        mine_block(&mut block).unwrap();

        assert_eq!(
            chain.append_block(block),
            Err(ChainError::UnexpectedDifficulty {
                expected: INITIAL_POW_BITS,
                got: easier_bits,
            })
        );
    }
}
