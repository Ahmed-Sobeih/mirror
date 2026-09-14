//! Consensus rules for the Mirror blockchain.
//!
//! Mirror begins with target-based Proof of Work.
//! Proof of Humanity will later constrain how human participants
//! interact with consensus.

use core::fmt;

use mirror_core::BlockHeader;
use mirror_crypto::Hash256;

/// Easy initial development/mainnet-bootstrap PoW target.
///
/// This corresponds to approximately 1 successful hash in 4096
/// for uniformly distributed SHA-256 hashes.
pub const INITIAL_POW_BITS: u32 = 0x1f0f_ffff;

/// A 256-bit Proof-of-Work target.
///
/// The bytes are stored in big-endian numeric order so that ordinary
/// lexicographic comparison is also numeric comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowTarget([u8; 32]);

impl PowTarget {
    /// Decode Mirror's compact target representation.
    ///
    /// Layout follows the Bitcoin-style "compact" idea:
    ///
    /// ```text
    /// bits = exponent || coefficient
    /// ```
    ///
    /// The upper byte is the exponent and the lower 23 bits contain
    /// the coefficient.
    pub fn from_compact(bits: u32) -> Result<Self, PowError> {
        let exponent = (bits >> 24) as usize;
        let negative = (bits & 0x0080_0000) != 0;
        let coefficient = bits & 0x007f_ffff;

        if negative {
            return Err(PowError::NegativeTarget);
        }

        if coefficient == 0 {
            return Err(PowError::ZeroTarget);
        }

        if exponent > 32 {
            return Err(PowError::TargetOverflow);
        }

        let mut target = [0u8; 32];

        if exponent <= 3 {
            let shift = 8 * (3 - exponent);
            let value = coefficient >> shift;

            if value == 0 {
                return Err(PowError::ZeroTarget);
            }

            let value_bytes = value.to_be_bytes();

            if exponent > 0 {
                target[32 - exponent..].copy_from_slice(&value_bytes[4 - exponent..]);
            }
        } else {
            let coefficient_bytes = [
                ((coefficient >> 16) & 0xff) as u8,
                ((coefficient >> 8) & 0xff) as u8,
                (coefficient & 0xff) as u8,
            ];

            let start = 32 - exponent;

            target[start..start + 3].copy_from_slice(&coefficient_bytes);
        }

        Ok(Self(target))
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns true when the hash satisfies this target.
    ///
    /// Mirror consensus interprets block hashes as big-endian
    /// 256-bit integers for Proof-of-Work comparison.
    pub fn accepts(&self, hash: Hash256) -> bool {
        hash.as_bytes() <= &self.0
    }

    pub fn to_hex(&self) -> String {
        let mut output = String::with_capacity(64);

        for byte in self.0 {
            use core::fmt::Write;

            write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
        }

        output
    }
}

/// Result returned after successfully mining a block header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MineResult {
    pub nonce: u64,
    pub hash: Hash256,
    pub attempts: u64,
}

/// Proof-of-Work validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowError {
    ZeroTarget,
    NegativeTarget,
    TargetOverflow,
    HashAboveTarget,
    NonceExhausted,
}

impl fmt::Display for PowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroTarget => write!(f, "proof-of-work target is zero"),
            Self::NegativeTarget => {
                write!(f, "proof-of-work target cannot be negative")
            }
            Self::TargetOverflow => {
                write!(f, "proof-of-work target exceeds 256 bits")
            }
            Self::HashAboveTarget => {
                write!(f, "block hash is above the proof-of-work target")
            }
            Self::NonceExhausted => {
                write!(f, "all nonce values were exhausted")
            }
        }
    }
}

impl std::error::Error for PowError {}

/// Validate the Proof of Work contained in a block header.
pub fn validate_pow(header: &BlockHeader) -> Result<(), PowError> {
    let target = PowTarget::from_compact(header.pow_bits())?;
    let hash = header.hash();

    if target.accepts(hash) {
        Ok(())
    } else {
        Err(PowError::HashAboveTarget)
    }
}

/// Mine a block header by incrementing its nonce.
///
/// Mining starts from whatever nonce is already present in the header.
pub fn mine(header: &mut BlockHeader) -> Result<MineResult, PowError> {
    let target = PowTarget::from_compact(header.pow_bits())?;

    let mut nonce = header.nonce();
    let mut attempts = 0u64;

    loop {
        header.set_nonce(nonce);

        let hash = header.hash();
        attempts = attempts.saturating_add(1);

        if target.accepts(hash) {
            return Ok(MineResult {
                nonce,
                hash,
                attempts,
            });
        }

        if nonce == u64::MAX {
            return Err(PowError::NonceExhausted);
        }

        nonce += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirror_crypto::Hash256;

    fn mining_header() -> BlockHeader {
        BlockHeader::new(
            1,
            Hash256::default(),
            Hash256::default(),
            Hash256::default(),
            0,
            INITIAL_POW_BITS,
            0,
        )
    }

    #[test]
    fn compact_target_decodes_correctly() {
        let target = PowTarget::from_compact(INITIAL_POW_BITS).unwrap();

        assert_eq!(
            target.to_hex(),
            concat!(
                "000fffff",
                "00000000000000000000000000000000000000000000000000000000"
            )
        );
    }

    #[test]
    fn negative_target_is_rejected() {
        let result = PowTarget::from_compact(0x1f8f_ffff);

        assert_eq!(result, Err(PowError::NegativeTarget));
    }

    #[test]
    fn zero_target_is_rejected() {
        let result = PowTarget::from_compact(0x1f00_0000);

        assert_eq!(result, Err(PowError::ZeroTarget));
    }

    #[test]
    fn mining_finds_known_nonce() {
        let mut header = mining_header();

        let result = mine(&mut header).unwrap();

        assert_eq!(result.nonce, 4_481);
        assert_eq!(result.attempts, 4_482);

        assert_eq!(
            result.hash.to_hex(),
            "0000d7a59dd226fff21dba990f9f48aed3bfe39d62ad033f0b8736b6695db686"
        );

        assert_eq!(header.nonce(), 4_481);
    }

    #[test]
    fn mined_header_passes_validation() {
        let mut header = mining_header();

        mine(&mut header).unwrap();

        assert_eq!(validate_pow(&header), Ok(()));
    }

    #[test]
    fn unmined_header_fails_validation() {
        let header = mining_header();

        assert_eq!(validate_pow(&header), Err(PowError::HashAboveTarget));
    }
}
