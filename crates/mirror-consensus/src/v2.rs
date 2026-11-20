//! Fixed-target work checks for the explicit synthetic V2 chain.
//!
//! These functions check only the work predicate over the exact domain-separated
//! 160-byte V2 header. The caller supplies the target from trusted configuration.
//! They establish no block-body, ledger, genesis, ancestry or finality authority.
//! Fixed easy work is a development rule, not equal-human consensus.

use core::fmt;

use mirror_core::v2::BlockHeaderV2;
use mirror_crypto::Hash256;

use crate::{PowError, PowTarget};

/// A finite local mining-call budget, not a consensus limit or difficulty rule.
/// Callers may resume a search with a new header/nonce and another explicit budget.
pub const MAX_MINING_ATTEMPTS_V2: u64 = 1_000_000;

/// A local work-search result. Public fields are data, not a block certificate;
/// an acceptance layer must independently validate the complete block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MineResultV2 {
    pub header: BlockHeaderV2,
    pub hash: Hash256,
    pub attempts: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowErrorV2 {
    WrongBits { expected: u32, got: u32 },
    Target(PowError),
    HashAboveTarget,
    InvalidAttemptBudget { requested: u64, maximum: u64 },
    AttemptsExhausted { attempts: u64 },
    NonceExhausted { attempts: u64 },
}

impl fmt::Display for PowErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongBits { expected, got } => write!(
                f,
                "V2 work target differs from configuration: expected {expected:#010x}, got {got:#010x}"
            ),
            Self::Target(error) => write!(f, "invalid V2 work target: {error}"),
            Self::HashAboveTarget => {
                write!(f, "V2 header hash is above the configured work target")
            }
            Self::InvalidAttemptBudget { requested, maximum } => write!(
                f,
                "V2 mining budget must be in 1..={maximum}, got {requested}"
            ),
            Self::AttemptsExhausted { attempts } => {
                write!(f, "V2 mining budget exhausted after {attempts} attempts")
            }
            Self::NonceExhausted { attempts } => {
                write!(f, "V2 nonce space exhausted after {attempts} attempts")
            }
        }
    }
}

impl std::error::Error for PowErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Target(error) => Some(error),
            _ => None,
        }
    }
}

fn configured_target(header: &BlockHeaderV2, expected_bits: u32) -> Result<PowTarget, PowErrorV2> {
    // Exact configured bytes are required, including when two compact values
    // decode to the same numeric target. Do not infer authority from the header.
    if header.pow_bits() != expected_bits {
        return Err(PowErrorV2::WrongBits {
            expected: expected_bits,
            got: header.pow_bits(),
        });
    }
    // Preserve Mirror's existing compact semantics, including accepted positive
    // noncanonical forms. ConfigV2 validates this same representable range.
    PowTarget::from_compact(expected_bits).map_err(PowErrorV2::Target)
}

/// Check exact configured target bits and the V2 header's work predicate.
///
/// `expected_bits` must come from the pinned local configuration, never from an
/// untrusted peer/header. SHA256d("MIRROR_BLOCK_V2" || header_bytes) is interpreted
/// as a big-endian 256-bit integer; equality with the target is accepted. No digest
/// reversal is applied. All other block and ledger rules remain the caller's job.
pub fn validate_header_pow_v2(
    header: &BlockHeaderV2,
    expected_bits: u32,
) -> Result<(), PowErrorV2> {
    let target = configured_target(header, expected_bits)?;
    if target.accepts(header.hash()) {
        Ok(())
    } else {
        Err(PowErrorV2::HashAboveTarget)
    }
}

/// Search a bounded consecutive nonce range without mutating the input header.
///
/// The initial nonce is included and only nonce changes. The local attempt budget
/// is validated before target checks/hash work. A solution on the final permitted
/// attempt or at u64::MAX succeeds. After a failed u64::MAX attempt, nonce exhaustion
/// takes precedence over budget exhaustion; nonce never wraps to zero.
pub fn mine_header_v2(
    header: BlockHeaderV2,
    expected_bits: u32,
    max_attempts: u64,
) -> Result<MineResultV2, PowErrorV2> {
    if max_attempts == 0 || max_attempts > MAX_MINING_ATTEMPTS_V2 {
        return Err(PowErrorV2::InvalidAttemptBudget {
            requested: max_attempts,
            maximum: MAX_MINING_ATTEMPTS_V2,
        });
    }
    let target = configured_target(&header, expected_bits)?;
    let mut nonce = header.nonce();
    for attempts in 1..=max_attempts {
        let candidate = BlockHeaderV2::new(
            header.config_hash(),
            header.height(),
            header.previous_hash(),
            header.operations_root(),
            header.ledger_root(),
            header.timestamp(),
            header.pow_bits(),
            nonce,
        );
        let hash = candidate.hash();
        if target.accepts(hash) {
            return Ok(MineResultV2 {
                header: candidate,
                hash,
                attempts,
            });
        }
        nonce = nonce
            .checked_add(1)
            .ok_or(PowErrorV2::NonceExhausted { attempts })?;
    }
    Err(PowErrorV2::AttemptsExhausted {
        attempts: max_attempts,
    })
}
