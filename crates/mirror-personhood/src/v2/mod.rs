//! Explicitly versioned synthetic V2 components: configuration/bootstrap and
//! signed messages and staged membership execution. No live V2 blocks, grants
//! or mainnet. Existing V1 APIs and encodings remain unchanged.
pub mod config;
mod encoding;
mod limits;
pub mod operation;
pub mod state;

pub use config::{BootstrapDescriptor, ConfigError, ConfigParameters, ConfigV2,
    TestAllocation, VerifiedBootstrap, VerifiedSeed, CONFIG_FIXED_LEN,
    CONFIG_VERSION, MAX_APPROVAL_THRESHOLD, MAX_CONFIG_BYTES, SYNTHETIC_PROFILE_VERSION};
pub use limits::LimitsV2;
pub use operation::{MembershipActionV2, OperationErrorV2, OperationHeaderV2,
    ParentChainViewV2, SignedMembershipOperationV2, ENROLL_OPERATION_LEN_V2,
    OPERATION_PREFIX_LEN_V2, OPERATION_VERSION_V2, REVIEW_OPERATION_LEN_V2};

pub use state::{ActivationV2, ApprovalRecordV2, InclusionPositionV2,
    MembershipOriginV2, MembershipRecordV2, MembershipRegistryV2, MembershipStateV2,
    MembershipStatusV2, StagedMembershipStateV2, StateErrorV2,
    MAX_APPROVALS_PER_RECORD_V2, MEMBERSHIP_BLOCK_BASE_BYTES_V2};
