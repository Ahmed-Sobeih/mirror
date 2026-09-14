//! Mirror blockchain orchestration.
//!
//! This crate links blocks into an ordered chain and advances the
//! deterministic account state only after complete validation.

use mirror_consensus::{BlockConsensusError, mine_block, validate_block};

use mirror_core::{Address, Block, BlockError, SignedTransaction};

use mirror_crypto::Hash256;

use mirror_state::{
    BlockStateError, ChainState, StateError, execute_transactions, validate_block_state,
};

/// Configuration from which the deterministic genesis block is built.
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

/// An in-memory validated Mirror blockchain.
///
/// Persistent storage will later move historical blocks and state
/// into `mirror-storage`.
#[derive(Clone, Debug)]
pub struct Chain {
    blocks: Vec<Block>,
    state: ChainState,
}

impl Chain {
    /// Build and mine Mirror's deterministic genesis block.
    pub fn from_genesis(config: GenesisConfig) -> Result<Self, ChainError> {
        let state = ChainState::from_genesis_allocations(config.allocations)?;

        let mut genesis = Block::new(
            1,
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
        })
    }

    /// Genesis is height 0, so height is blocks.len() - 1.
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

    /// Construct and mine a candidate block extending the current tip.
    ///
    /// This does not append it. The candidate must still go through
    /// `append_block`, exactly like a block received from another peer.
    pub fn mine_next_block(
        &self,
        transactions: Vec<SignedTransaction>,
        timestamp: u64,
        pow_bits: u32,
    ) -> Result<Block, ChainError> {
        let transition = execute_transactions(&self.state, &transactions)?;

        let mut block = Block::new(
            1,
            self.tip_hash(),
            transition.state_root(),
            timestamp,
            pow_bits,
            transactions,
        )?;

        mine_block(&mut block)?;

        Ok(block)
    }

    /// Validate and append a block to the current chain tip.
    pub fn append_block(&mut self, block: Block) -> Result<(), ChainError> {
        let expected = self.tip_hash();
        let got = block.header().previous_block_hash();

        if got != expected {
            return Err(ChainError::PreviousBlockMismatch { expected, got });
        }

        // Validate signatures, Merkle root and PoW.
        validate_block(&block)?;

        // Independently execute transactions against our current state
        // and verify the committed post-state root.
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

    use mirror_consensus::INITIAL_POW_BITS;

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

    fn test_chain(alice: &Keypair) -> Chain {
        Chain::from_genesis(GenesisConfig::new(
            1_800_000_000,
            INITIAL_POW_BITS,
            vec![(address(alice), 100 * NUSA_PER_MRY)],
        ))
        .unwrap()
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
            .mine_next_block(
                vec![transfer(&alice, bob_address, 0)],
                1_800_000_001,
                INITIAL_POW_BITS,
            )
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
            1,
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
            .mine_next_block(
                vec![first_transaction.clone()],
                1_800_000_001,
                INITIAL_POW_BITS,
            )
            .unwrap();

        chain.append_block(block_one).unwrap();

        let result =
            chain.mine_next_block(vec![first_transaction], 1_800_000_002, INITIAL_POW_BITS);

        assert!(matches!(
            result,
            Err(ChainError::State(StateError::InvalidNonce {
                expected: 1,
                got: 0,
            }))
        ));
    }
}
