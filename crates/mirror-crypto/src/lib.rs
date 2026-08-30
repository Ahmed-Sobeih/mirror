//! Cryptographic primitives used by the Mirror protocol.
//!
//! Consensus-critical code belongs here. Changes to hashing rules must be
//! treated as protocol changes once the network is live.

use core::fmt;
use sha2::{Digest, Sha256};

/// A 256-bit cryptographic hash used throughout Mirror.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    /// Creates a hash directly from its 32 raw bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw 32-byte representation.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Converts the hash to lowercase hexadecimal.
    pub fn to_hex(&self) -> String {
        let mut output = String::with_capacity(64);

        for byte in self.0 {
            use core::fmt::Write;
            write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
        }

        output
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256({self})")
    }
}

/// Computes Bitcoin-style double SHA-256:
///
/// SHA256(SHA256(data))
pub fn sha256d(data: &[u8]) -> Hash256 {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);

    Hash256::from_bytes(second.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256d_empty_bytes_matches_known_value() {
        let hash = sha256d(b"");

        assert_eq!(
            hash.to_hex(),
            "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"
        );
    }

    #[test]
    fn sha256d_abc_matches_known_value() {
        let hash = sha256d(b"abc");

        assert_eq!(
            hash.to_hex(),
            "4f8b42c22dd3729b519ba6f68d2da7cc5b2d606d05daed5ad5128cc03e6c6358"
        );
    }

    #[test]
    fn same_input_always_produces_same_hash() {
        let a = sha256d(b"Mirror");
        let b = sha256d(b"Mirror");

        assert_eq!(a, b);
    }

    #[test]
    fn different_input_produces_different_hash() {
        let a = sha256d(b"Mirror");
        let b = sha256d(b"mirror");

        assert_ne!(a, b);
    }

    #[test]
    fn hash_is_32_bytes() {
        let hash = sha256d(b"Mirror");

        assert_eq!(hash.as_bytes().len(), 32);
    }
}
