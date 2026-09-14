//! Core consensus data structures for the Mirror blockchain.

pub mod block;
pub mod merkle;
pub mod signed_transaction;
pub mod transaction;

pub use block::{BLOCK_HEADER_LEN, BlockHeader};

pub use signed_transaction::{
    SIGNED_TRANSACTION_AUTH_LEN, SignedTransaction, SignedTransactionError,
};

pub use transaction::{
    Address, MIRROR_CHAIN_ID, NUSA_PER_MRY, TRANSACTION_BODY_FIXED_LEN, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody, TransactionError,
};

pub use merkle::merkle_root;

pub use block::{Block, BlockError};

pub mod codec;

pub use codec::{
    BLOCK_ENCODING_MAGIC, CodecError, MIN_ENCODED_BLOCK_LEN, decode_block, encode_block,
};
