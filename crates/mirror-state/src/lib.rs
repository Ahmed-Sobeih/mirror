//! Consensus state-transition engine for the Mirror blockchain.
//!
//! This crate determines whether transactions are economically valid
//! and calculates the state commitment placed into each block header.

use std::collections::BTreeMap;

use mirror_core::{
    Address, MIRROR_CHAIN_ID, SignedTransaction, SignedTransactionError, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, merkle_root,
};

use mirror_crypto::{Hash256, sha256d};

const ACCOUNT_DOMAIN: &[u8] = b"MIRROR_ACCOUNT_V1";
const STATE_ROOT_DOMAIN: &[u8] = b"MIRROR_STATE_ROOT_V1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Account {
    balance: u64,
    nonce: u64,
}

impl Account {
    pub const fn new(balance: u64, nonce: u64) -> Self {
        Self { balance, nonce }
    }

    pub const fn balance(&self) -> u64 {
        self.balance
    }

    pub const fn nonce(&self) -> u64 {
        self.nonce
    }
}

/// Complete account state of the Mirror chain.
///
/// BTreeMap is intentional: addresses always iterate in canonical
/// byte order, ensuring all nodes calculate the same state root.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChainState {
    accounts: BTreeMap<Address, Account>,
}

impl ChainState {
    pub const fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    /// Construct the initial state from explicit genesis allocations.
    ///
    /// This does not itself decide what Mirror's genesis allocations are.
    /// The future genesis specification will provide those values.
    pub fn from_genesis_allocations<I>(allocations: I) -> Result<Self, StateError>
    where
        I: IntoIterator<Item = (Address, u64)>,
    {
        let mut state = Self::new();

        for (address, balance) in allocations {
            if state.accounts.contains_key(&address) {
                return Err(StateError::DuplicateGenesisAddress);
            }

            if balance != 0 {
                state.accounts.insert(address, Account::new(balance, 0));
            }
        }

        Ok(state)
    }

    /// Return an account.
    ///
    /// An address that has never appeared on-chain behaves as an
    /// account with zero balance and nonce zero.
    pub fn account(&self, address: Address) -> Account {
        self.accounts.get(&address).copied().unwrap_or_default()
    }

    /// Compute the deterministic commitment to the complete account state.
    ///
    /// Each account leaf commits to:
    ///
    /// address || balance || nonce
    ///
    /// Accounts are sorted by raw address bytes because ChainState uses
    /// a BTreeMap.
    pub fn state_root(&self) -> Hash256 {
        let mut leaves = Vec::with_capacity(self.accounts.len());

        for (address, account) in &self.accounts {
            let mut bytes = Vec::with_capacity(ACCOUNT_DOMAIN.len() + 32 + 8 + 8);

            bytes.extend_from_slice(ACCOUNT_DOMAIN);
            bytes.extend_from_slice(address.as_bytes());
            bytes.extend_from_slice(&account.balance.to_le_bytes());
            bytes.extend_from_slice(&account.nonce.to_le_bytes());

            leaves.push(sha256d(&bytes));
        }

        let account_tree_root = merkle_root(&leaves);

        let mut commitment = Vec::with_capacity(STATE_ROOT_DOMAIN.len() + 32);

        commitment.extend_from_slice(STATE_ROOT_DOMAIN);
        commitment.extend_from_slice(account_tree_root.as_bytes());

        sha256d(&commitment)
    }

    /// Apply one signed transaction to chain state.
    ///
    /// Returns the transaction fee in Nusa.
    pub fn apply_transaction(
        &mut self,
        transaction: &SignedTransaction,
    ) -> Result<u64, StateError> {
        transaction.verify()?;

        let body = transaction.body();

        if body.version() != TRANSACTION_VERSION {
            return Err(StateError::UnsupportedVersion {
                got: body.version(),
            });
        }

        if body.kind() != TRANSACTION_KIND_TRANSFER {
            return Err(StateError::UnsupportedTransactionKind { got: body.kind() });
        }

        if body.chain_id() != MIRROR_CHAIN_ID {
            return Err(StateError::WrongChainId {
                expected: MIRROR_CHAIN_ID,
                got: body.chain_id(),
            });
        }

        if body.amount() == 0 {
            return Err(StateError::ZeroTransferAmount);
        }

        let sender_address = body.sender();
        let recipient_address = body.recipient();

        let sender = self.account(sender_address);

        if body.nonce() != sender.nonce {
            return Err(StateError::InvalidNonce {
                expected: sender.nonce,
                got: body.nonce(),
            });
        }

        let required = body
            .amount()
            .checked_add(body.fee())
            .ok_or(StateError::AmountOverflow)?;

        if sender.balance < required {
            return Err(StateError::InsufficientBalance {
                balance: sender.balance,
                required,
            });
        }

        let next_nonce = sender
            .nonce
            .checked_add(1)
            .ok_or(StateError::NonceOverflow)?;

        // Self-transfer:
        //
        // Value returns to the sender, so only the fee is lost.
        // We still require balance >= amount + fee before execution.
        if sender_address == recipient_address {
            let next_balance = sender
                .balance
                .checked_sub(body.fee())
                .ok_or(StateError::AmountOverflow)?;

            self.accounts
                .insert(sender_address, Account::new(next_balance, next_nonce));

            return Ok(body.fee());
        }

        let recipient = self.account(recipient_address);

        let sender_balance = sender
            .balance
            .checked_sub(required)
            .ok_or(StateError::AmountOverflow)?;

        let recipient_balance = recipient
            .balance
            .checked_add(body.amount())
            .ok_or(StateError::AmountOverflow)?;

        self.accounts
            .insert(sender_address, Account::new(sender_balance, next_nonce));

        self.accounts.insert(
            recipient_address,
            Account::new(recipient_balance, recipient.nonce),
        );

        Ok(body.fee())
    }

    /// Apply an ordered transaction batch atomically.
    ///
    /// If any transaction fails, no transaction from the batch
    /// modifies the original state.
    pub fn apply_transactions(
        &mut self,
        transactions: &[SignedTransaction],
    ) -> Result<u64, StateError> {
        let mut candidate = self.clone();
        let mut total_fees = 0u64;

        for transaction in transactions {
            let fee = candidate.apply_transaction(transaction)?;

            total_fees = total_fees
                .checked_add(fee)
                .ok_or(StateError::AmountOverflow)?;
        }

        *self = candidate;

        Ok(total_fees)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateError {
    Transaction(SignedTransactionError),

    UnsupportedVersion { got: u16 },

    UnsupportedTransactionKind { got: u16 },

    WrongChainId { expected: u32, got: u32 },

    InvalidNonce { expected: u64, got: u64 },

    InsufficientBalance { balance: u64, required: u64 },

    ZeroTransferAmount,
    AmountOverflow,
    NonceOverflow,
    DuplicateGenesisAddress,
}

impl core::fmt::Display for StateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transaction(error) => {
                write!(f, "transaction verification failed: {error}")
            }

            Self::UnsupportedVersion { got } => {
                write!(f, "unsupported transaction version {got}")
            }

            Self::UnsupportedTransactionKind { got } => {
                write!(f, "unsupported transaction kind {got}")
            }

            Self::WrongChainId { expected, got } => {
                write!(f, "wrong chain id: expected {expected}, got {got}")
            }

            Self::InvalidNonce { expected, got } => {
                write!(f, "invalid account nonce: expected {expected}, got {got}")
            }

            Self::InsufficientBalance { balance, required } => {
                write!(f, "insufficient balance: have {balance}, need {required}")
            }

            Self::ZeroTransferAmount => {
                write!(f, "transfer amount cannot be zero")
            }

            Self::AmountOverflow => {
                write!(f, "monetary amount overflow")
            }

            Self::NonceOverflow => {
                write!(f, "account nonce overflow")
            }

            Self::DuplicateGenesisAddress => {
                write!(f, "duplicate address in genesis allocations")
            }
        }
    }
}

impl std::error::Error for StateError {}

impl From<SignedTransactionError> for StateError {
    fn from(error: SignedTransactionError) -> Self {
        Self::Transaction(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirror_core::NUSA_PER_MRY;

    use mirror_core::TransactionBody;
    use mirror_crypto::Keypair;

    fn key(secret: u8) -> Keypair {
        Keypair::from_secret_bytes([secret; 32])
    }

    fn address(keypair: &Keypair) -> Address {
        Address::from_public_key(&keypair.public_key())
    }

    fn transfer(
        sender: &Keypair,
        recipient: Address,
        nonce: u64,
        amount: u64,
        fee: u64,
        chain_id: u32,
    ) -> SignedTransaction {
        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            chain_id,
            nonce,
            address(sender),
            recipient,
            amount,
            fee,
            Vec::new(),
        );

        SignedTransaction::sign(body, sender).expect("test transaction must sign")
    }

    #[test]
    fn genesis_state_root_matches_fixed_vector() {
        let state = ChainState::from_genesis_allocations([(
            Address::from_bytes([0x11; 32]),
            100 * NUSA_PER_MRY,
        )])
        .unwrap();

        assert_eq!(
            state.state_root().to_hex(),
            "96633fd3194d3cb499bde7d2802e911ba5cede7a0f5faacfe8f8c13c9c8347c9"
        );
    }

    #[test]
    fn valid_transfer_updates_balances_and_nonce() {
        let alice = key(1);
        let bob = key(2);

        let alice_address = address(&alice);
        let bob_address = address(&bob);

        let mut state =
            ChainState::from_genesis_allocations([(alice_address, 100 * NUSA_PER_MRY)]).unwrap();

        let old_root = state.state_root();

        let transaction = transfer(&alice, bob_address, 0, NUSA_PER_MRY, 1_000, MIRROR_CHAIN_ID);

        let fee = state.apply_transaction(&transaction).unwrap();

        assert_eq!(fee, 1_000);

        assert_eq!(
            state.account(alice_address),
            Account::new(99 * NUSA_PER_MRY - 1_000, 1)
        );

        assert_eq!(state.account(bob_address), Account::new(NUSA_PER_MRY, 0));

        assert_ne!(state.state_root(), old_root);
    }

    #[test]
    fn insufficient_balance_is_rejected_without_mutation() {
        let alice = key(1);
        let bob = key(2);

        let alice_address = address(&alice);

        let mut state =
            ChainState::from_genesis_allocations([(alice_address, NUSA_PER_MRY)]).unwrap();

        let original = state.clone();

        let transaction = transfer(
            &alice,
            address(&bob),
            0,
            2 * NUSA_PER_MRY,
            1_000,
            MIRROR_CHAIN_ID,
        );

        assert!(matches!(
            state.apply_transaction(&transaction),
            Err(StateError::InsufficientBalance { .. })
        ));

        assert_eq!(state, original);
    }

    #[test]
    fn replayed_transaction_is_rejected_by_nonce() {
        let alice = key(1);
        let bob = key(2);

        let mut state =
            ChainState::from_genesis_allocations([(address(&alice), 100 * NUSA_PER_MRY)]).unwrap();

        let transaction = transfer(
            &alice,
            address(&bob),
            0,
            NUSA_PER_MRY,
            1_000,
            MIRROR_CHAIN_ID,
        );

        state.apply_transaction(&transaction).unwrap();

        assert_eq!(
            state.apply_transaction(&transaction),
            Err(StateError::InvalidNonce {
                expected: 1,
                got: 0,
            })
        );
    }

    #[test]
    fn wrong_chain_id_is_rejected() {
        let alice = key(1);
        let bob = key(2);

        let mut state =
            ChainState::from_genesis_allocations([(address(&alice), 100 * NUSA_PER_MRY)]).unwrap();

        let transaction = transfer(
            &alice,
            address(&bob),
            0,
            NUSA_PER_MRY,
            1_000,
            MIRROR_CHAIN_ID + 1,
        );

        assert_eq!(
            state.apply_transaction(&transaction),
            Err(StateError::WrongChainId {
                expected: MIRROR_CHAIN_ID,
                got: MIRROR_CHAIN_ID + 1,
            })
        );
    }

    #[test]
    fn transaction_batch_is_atomic() {
        let alice = key(1);
        let bob = key(2);

        let mut state =
            ChainState::from_genesis_allocations([(address(&alice), 100 * NUSA_PER_MRY)]).unwrap();

        let before = state.clone();

        let first = transfer(
            &alice,
            address(&bob),
            0,
            NUSA_PER_MRY,
            1_000,
            MIRROR_CHAIN_ID,
        );

        // Invalid: after the first transaction the expected nonce
        // would be 1, not 2.
        let second = transfer(
            &alice,
            address(&bob),
            2,
            NUSA_PER_MRY,
            1_000,
            MIRROR_CHAIN_ID,
        );

        assert_eq!(
            state.apply_transactions(&[first, second]),
            Err(StateError::InvalidNonce {
                expected: 1,
                got: 2,
            })
        );

        assert_eq!(state, before);
    }
}

use mirror_core::Block;

/// Result of deterministically executing an ordered transaction list
/// against a specific pre-state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateTransition {
    post_state: ChainState,
    state_root: Hash256,
    total_fees: u64,
}

impl StateTransition {
    pub const fn post_state(&self) -> &ChainState {
        &self.post_state
    }

    pub const fn state_root(&self) -> Hash256 {
        self.state_root
    }

    pub const fn total_fees(&self) -> u64 {
        self.total_fees
    }

    pub fn into_post_state(self) -> ChainState {
        self.post_state
    }
}

/// Execute an ordered transaction list without modifying `pre_state`.
///
/// Every node given the same pre-state and transactions must produce
/// exactly the same resulting state and state root.
pub fn execute_transactions(
    pre_state: &ChainState,
    transactions: &[SignedTransaction],
) -> Result<StateTransition, StateError> {
    let mut post_state = pre_state.clone();

    let total_fees = post_state.apply_transactions(transactions)?;

    let state_root = post_state.state_root();

    Ok(StateTransition {
        post_state,
        state_root,
        total_fees,
    })
}

/// State-level validation errors for a complete block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockStateError {
    State(StateError),

    StateRootMismatch {
        committed: Hash256,
        calculated: Hash256,
    },
}

impl core::fmt::Display for BlockStateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::State(error) => {
                write!(f, "state transition failed: {error}")
            }

            Self::StateRootMismatch {
                committed,
                calculated,
            } => {
                write!(
                    f,
                    "state root mismatch: block commits to {committed}, calculated {calculated}"
                )
            }
        }
    }
}

impl std::error::Error for BlockStateError {}

impl From<StateError> for BlockStateError {
    fn from(error: StateError) -> Self {
        Self::State(error)
    }
}

/// Re-execute a block from a known pre-state and verify that the state
/// commitment in its header is correct.
pub fn validate_block_state(
    pre_state: &ChainState,
    block: &Block,
) -> Result<StateTransition, BlockStateError> {
    let transition = execute_transactions(pre_state, block.transactions())?;

    let committed = block.header().state_root();
    let calculated = transition.state_root();

    if committed != calculated {
        return Err(BlockStateError::StateRootMismatch {
            committed,
            calculated,
        });
    }

    Ok(transition)
}

#[cfg(test)]
mod block_state_tests {
    use super::*;
    use mirror_core::NUSA_PER_MRY;

    use mirror_core::{
        Address, Block, SignedTransaction, TRANSACTION_KIND_TRANSFER, TRANSACTION_VERSION,
        TransactionBody,
    };

    use mirror_crypto::Keypair;

    fn key(secret: u8) -> Keypair {
        Keypair::from_secret_bytes([secret; 32])
    }

    fn address(keypair: &Keypair) -> Address {
        Address::from_public_key(&keypair.public_key())
    }

    fn transaction(alice: &Keypair, bob: Address) -> SignedTransaction {
        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            MIRROR_CHAIN_ID,
            0,
            address(alice),
            bob,
            NUSA_PER_MRY,
            0,
            Vec::new(),
        );

        SignedTransaction::sign(body, alice).expect("test transaction must sign")
    }

    #[test]
    fn block_state_root_can_be_reproduced() {
        let alice = key(1);
        let bob = key(2);

        let pre_state =
            ChainState::from_genesis_allocations([(address(&alice), 100 * NUSA_PER_MRY)]).unwrap();

        let tx = transaction(&alice, address(&bob));

        let transition = execute_transactions(&pre_state, std::slice::from_ref(&tx)).unwrap();

        let block = Block::new(
            1,
            Hash256::default(),
            transition.state_root(),
            1_800_000_000,
            0x1f0f_ffff,
            vec![tx],
        )
        .unwrap();

        let validated = validate_block_state(&pre_state, &block).unwrap();

        assert_eq!(validated.state_root(), transition.state_root());

        assert_eq!(validated.post_state(), transition.post_state());
    }

    #[test]
    fn incorrect_block_state_root_is_rejected() {
        let alice = key(1);
        let bob = key(2);

        let pre_state =
            ChainState::from_genesis_allocations([(address(&alice), 100 * NUSA_PER_MRY)]).unwrap();

        let tx = transaction(&alice, address(&bob));

        let block = Block::new(
            1,
            Hash256::default(),
            Hash256::default(),
            1_800_000_000,
            0x1f0f_ffff,
            vec![tx],
        )
        .unwrap();

        assert!(matches!(
            validate_block_state(&pre_state, &block),
            Err(BlockStateError::StateRootMismatch { .. })
        ));
    }
}
