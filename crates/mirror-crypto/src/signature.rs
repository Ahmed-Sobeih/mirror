use core::fmt;

use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, VerifyingKey};

use crate::Hash256;

pub const PUBLIC_KEY_LEN: usize = 32;
pub const SIGNATURE_LEN: usize = 64;

/// Public Ed25519 key used to verify Mirror transaction signatures.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; PUBLIC_KEY_LEN]);

impl PublicKey {
    pub const fn from_bytes(bytes: [u8; PUBLIC_KEY_LEN]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LEN] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({self})")
    }
}

/// Raw 64-byte Ed25519 signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SignatureBytes([u8; SIGNATURE_LEN]);

impl SignatureBytes {
    pub const fn from_bytes(bytes: [u8; SIGNATURE_LEN]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SIGNATURE_LEN] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        bytes_to_hex(&self.0)
    }
}

impl fmt::Display for SignatureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

impl fmt::Debug for SignatureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignatureBytes({self})")
    }
}

/// Mirror signing keypair.
///
/// The secret key is intentionally private and Debug is not implemented.
/// We do not expose secret-key bytes from this type at this stage.
pub struct Keypair {
    signing_key: SigningKey,
}

impl Keypair {
    /// Generate a new keypair using the operating system CSPRNG.
    pub fn generate() -> Result<Self, CryptoError> {
        let mut secret = [0u8; 32];

        getrandom::fill(&mut secret).map_err(|_| CryptoError::RandomnessUnavailable)?;

        Ok(Self::from_secret_bytes(secret))
    }

    /// Construct a keypair from deterministic secret bytes.
    ///
    /// Primarily useful for reproducible tests and, later,
    /// secure wallet restoration.
    pub fn from_secret_bytes(secret: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&secret),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_bytes(self.signing_key.verifying_key().to_bytes())
    }

    /// Sign an already-computed Mirror 256-bit signing hash.
    pub fn sign_hash(&self, hash: &Hash256) -> SignatureBytes {
        let signature: DalekSignature = self.signing_key.sign(hash.as_bytes());

        SignatureBytes::from_bytes(signature.to_bytes())
    }
}

/// Strictly verify a signature over a Mirror hash.
pub fn verify_hash(
    public_key: &PublicKey,
    hash: &Hash256,
    signature: &SignatureBytes,
) -> Result<(), CryptoError> {
    let verifying_key = VerifyingKey::from_bytes(public_key.as_bytes())
        .map_err(|_| CryptoError::InvalidPublicKey)?;

    if verifying_key.is_weak() {
        return Err(CryptoError::WeakPublicKey);
    }

    let signature = DalekSignature::from_bytes(signature.as_bytes());

    verifying_key
        .verify_strict(hash.as_bytes(), &signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CryptoError {
    RandomnessUnavailable,
    InvalidPublicKey,
    WeakPublicKey,
    InvalidSignature,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RandomnessUnavailable => {
                write!(f, "secure operating-system randomness unavailable")
            }
            Self::InvalidPublicKey => {
                write!(f, "invalid Ed25519 public key")
            }
            Self::WeakPublicKey => {
                write!(f, "weak Ed25519 public key rejected")
            }
            Self::InvalidSignature => {
                write!(f, "invalid Ed25519 signature")
            }
        }
    }
}

impl std::error::Error for CryptoError {}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        use core::fmt::Write;

        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sha256d;

    fn test_keypair() -> Keypair {
        Keypair::from_secret_bytes([1u8; 32])
    }

    #[test]
    fn fixed_secret_produces_known_public_key() {
        let keypair = test_keypair();

        assert_eq!(
            keypair.public_key().to_hex(),
            "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c"
        );
    }

    #[test]
    fn signature_matches_fixed_test_vector() {
        let keypair = test_keypair();
        let hash = sha256d(b"Mirror");
        let signature = keypair.sign_hash(&hash);

        assert_eq!(
            hash.to_hex(),
            "1faf357140a758364f6f3e613f0bb0c06cdd5cc1278cd71716c73097402d141a"
        );

        assert_eq!(
            signature.to_hex(),
            concat!(
                "f513624ff1d970c0bfd0b336534fb82c",
                "6b5cc0f22059346fa2acaa389e66275d",
                "bbed63eacae39a27ae09acd5a75d3664",
                "1e59a646b3c5282c8a5822e78927670e"
            )
        );
    }

    #[test]
    fn valid_signature_verifies() {
        let keypair = test_keypair();
        let hash = sha256d(b"Mirror");
        let signature = keypair.sign_hash(&hash);

        assert_eq!(
            verify_hash(&keypair.public_key(), &hash, &signature),
            Ok(())
        );
    }

    #[test]
    fn modified_message_is_rejected() {
        let keypair = test_keypair();

        let original = sha256d(b"Mirror");
        let modified = sha256d(b"Mirror!");

        let signature = keypair.sign_hash(&original);

        assert_eq!(
            verify_hash(&keypair.public_key(), &modified, &signature),
            Err(CryptoError::InvalidSignature)
        );
    }

    #[test]
    fn wrong_public_key_is_rejected() {
        let alice = Keypair::from_secret_bytes([1u8; 32]);

        let bob = Keypair::from_secret_bytes([2u8; 32]);

        let hash = sha256d(b"Mirror");
        let signature = alice.sign_hash(&hash);

        assert_eq!(
            verify_hash(&bob.public_key(), &hash, &signature),
            Err(CryptoError::InvalidSignature)
        );
    }

    #[test]
    fn generated_keypair_can_sign_and_verify() {
        let keypair = Keypair::generate().unwrap();

        let hash = sha256d(b"Mirror random wallet");
        let signature = keypair.sign_hash(&hash);

        assert_eq!(
            verify_hash(&keypair.public_key(), &hash, &signature),
            Ok(())
        );
    }
}
