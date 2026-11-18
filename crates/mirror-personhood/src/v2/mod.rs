//! Explicitly versioned synthetic V2 components: configuration/bootstrap and
//! signed membership authorization messages. No V2 state execution, blocks,
//! grants or mainnet. Existing V1 APIs and encodings remain unchanged.
pub mod config;
mod encoding;
mod limits;
pub mod operation;

pub use config::{BootstrapDescriptor, ConfigError, ConfigParameters, ConfigV2,
    TestAllocation, VerifiedBootstrap, VerifiedSeed, CONFIG_FIXED_LEN,
    CONFIG_VERSION, MAX_APPROVAL_THRESHOLD, MAX_CONFIG_BYTES, SYNTHETIC_PROFILE_VERSION};
pub use limits::LimitsV2;
pub use operation::{MembershipActionV2, OperationErrorV2, OperationHeaderV2,
    ParentChainViewV2, SignedMembershipOperationV2, ENROLL_OPERATION_LEN_V2,
    OPERATION_PREFIX_LEN_V2, OPERATION_VERSION_V2, REVIEW_OPERATION_LEN_V2};
