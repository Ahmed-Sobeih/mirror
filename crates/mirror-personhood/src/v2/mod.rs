//! Explicitly versioned synthetic V2 components. B1 only: configuration and
//! proof-checked bootstrap. No V2 runtime enrollment, blocks, grants or mainnet.
//! Existing V1 types remain at the crate root, with unchanged encodings.
pub mod config;
mod encoding;
mod limits;

pub use config::{BootstrapDescriptor, ConfigError, ConfigParameters, ConfigV2,
    TestAllocation, VerifiedBootstrap, VerifiedSeed, CONFIG_FIXED_LEN,
    CONFIG_VERSION, MAX_APPROVAL_THRESHOLD, MAX_CONFIG_BYTES, SYNTHETIC_PROFILE_VERSION};
pub use limits::LimitsV2;
