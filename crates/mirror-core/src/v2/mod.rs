//! MB-02A: typed V2 operations and bounded block encodings/commitments.
//!
//! These are untrusted data formats, not accepted ledger state. Decoding,
//! body integrity and contextual authorization are separate checks. Combined
//! accounts/membership execution and actual chain/genesis/PoW validation follow
//! in MB-02B/C. All legacy types and wire encodings remain unchanged.

pub mod block;
pub mod operation;

pub use block::{
    BLOCK_ENCODING_MAGIC_V2, BLOCK_HEADER_LEN_V2, BLOCK_VERSION_V2, BlockErrorV2, BlockHeaderV2,
    BlockV2, MIN_ENCODED_BLOCK_LEN_V2, operations_root_v2,
};
pub use operation::{
    SignedOperationV2, SignedTransferV2, TRANSFER_KIND_V2, TRANSFER_OPERATION_LEN_V2,
    TransferBodyV2, TransferErrorV2, UnifiedOperationErrorV2,
};
