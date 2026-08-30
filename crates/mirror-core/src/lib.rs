//! Core consensus data structures for the Mirror blockchain.

pub mod block;
pub mod transaction;

pub use block::{BLOCK_HEADER_LEN, BlockHeader};

pub use transaction::{
    Address, NUSA_PER_MRY, TRANSACTION_BODY_FIXED_LEN, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody, TransactionError,
};
