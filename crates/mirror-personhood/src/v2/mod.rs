//! Explicitly versioned synthetic V2 components: configuration/bootstrap and
//! signed messages and staged membership execution. No live V2 blocks, grants
//! or mainnet. Existing V1 APIs and encodings remain unchanged.
pub mod config;
mod encoding;
mod limits;
pub mod operation;
pub mod replay;
pub mod state;

pub use config::{
    BootstrapDescriptor, CONFIG_FIXED_LEN, CONFIG_VERSION, ConfigError, ConfigParameters, ConfigV2,
    MAX_APPROVAL_THRESHOLD, MAX_CONFIG_BYTES, SYNTHETIC_PROFILE_VERSION, TestAllocation,
    VerifiedBootstrap, VerifiedSeed,
};
pub use limits::LimitsV2;
pub use operation::{
    ENROLL_OPERATION_LEN_V2, MembershipActionV2, OPERATION_PREFIX_LEN_V2, OPERATION_VERSION_V2,
    OperationErrorV2, OperationHeaderV2, ParentChainViewV2, REVIEW_OPERATION_LEN_V2,
    SignedMembershipOperationV2,
};

pub use state::{
    ActivationV2, ApprovalRecordV2, InclusionPositionV2, IndexedMembershipStageV2,
    MAX_APPROVALS_PER_RECORD_V2, MEMBERSHIP_BLOCK_BASE_BYTES_V2, MembershipOriginV2,
    MembershipRecordV2, MembershipRegistryV2, MembershipStateV2, MembershipStatusV2,
    StagedMembershipStateV2, StateErrorV2,
};

pub use replay::{
    MAX_REPLAY_BLOCKS_V2, MAX_REPLAY_STATE_ITEMS_V2, MembershipReplayV2, REPLAY_GROUP_BASE_LEN_V2,
    REPLAY_PREFIX_LEN_V2, REPLAY_VERSION_V2, ReplayChainViewV2, ReplayErrorV2, ReplayExpectationV2,
    ReplayGroupV2, TrustedReplayBlockV2, replay_group_commitment_v2,
};
pub use state::{StateCodecErrorV2, UntrustedMembershipSnapshotV2};
