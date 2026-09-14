use core::fmt;

use mirror_crypto::{
    CryptoError, Hash256, Keypair, PublicKey, SignatureBytes, sha256d, verify_hash,
};

use crate::transaction::{Address, TransactionBody, TransactionError};

/// Domain separator used when deriving Mirror addresses.
///
/// Domain separation ensures that a raw public key hashed for another
/// protocol does not automatically produce the same address namespace.
const ADDRESS_DOMAIN: &[u8] = b"MIRROR_ADDRESS_V1";

/// Ed25519 authentication data:
///
/// 32-byte public key + 64-byte signature.
pub const SIGNED_TRANSACTION_AUTH_LEN: usize = 96;

impl Address {
    /// Derive a Mirror v1 address from an Ed25519 public key.
    ///
    /// address = SHA256d(
    ///     "MIRROR_ADDRESS_V1" || public_key
    /// )
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        let mut data = Vec::with_capacity(ADDRESS_DOMAIN.len() + public_key.as_bytes().len());

        data.extend_from_slice(ADDRESS_DOMAIN);
        data.extend_from_slice(public_key.as_bytes());

        let hash = sha256d(&data);

        Self::from_bytes(*hash.as_bytes())
    }
}

/// A cryptographically authorized Mirror transaction.
///
/// The body contains the transaction intent.
/// The public key identifies the signing key.
/// The signature proves authorization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    body: TransactionBody,
    public_key: PublicKey,
    signature: SignatureBytes,
}

impl SignedTransaction {
    /// Sign a transaction body.
    ///
    /// The body's sender address must already correspond to the
    /// signing keypair.
    pub fn sign(body: TransactionBody, keypair: &Keypair) -> Result<Self, SignedTransactionError> {
        let public_key = keypair.public_key();
        let expected_sender = Address::from_public_key(&public_key);

        if body.sender() != expected_sender {
            return Err(SignedTransactionError::SenderPublicKeyMismatch);
        }

        let signing_hash = body.signing_hash()?;
        let signature = keypair.sign_hash(&signing_hash);

        Ok(Self {
            body,
            public_key,
            signature,
        })
    }

    /// Construct from already existing components.
    ///
    /// This does not imply validity. Call `verify()` before accepting it.
    pub const fn from_parts(
        body: TransactionBody,
        public_key: PublicKey,
        signature: SignatureBytes,
    ) -> Self {
        Self {
            body,
            public_key,
            signature,
        }
    }

    pub const fn body(&self) -> &TransactionBody {
        &self.body
    }

    pub const fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub const fn signature(&self) -> SignatureBytes {
        self.signature
    }

    /// Verify authorization of this transaction.
    pub fn verify(&self) -> Result<(), SignedTransactionError> {
        let expected_sender = Address::from_public_key(&self.public_key);

        if self.body.sender() != expected_sender {
            return Err(SignedTransactionError::SenderPublicKeyMismatch);
        }

        let signing_hash = self.body.signing_hash()?;

        verify_hash(&self.public_key, &signing_hash, &self.signature)?;

        Ok(())
    }

    /// Canonical encoding of a fully signed Mirror transaction.
    ///
    /// Layout:
    ///
    /// ```text
    /// transaction_body   variable bytes
    /// public_key         32 bytes
    /// signature          64 bytes
    /// ```
    pub fn encode(&self) -> Result<Vec<u8>, SignedTransactionError> {
        let body = self.body.encode()?;

        let mut bytes = Vec::with_capacity(body.len() + SIGNED_TRANSACTION_AUTH_LEN);

        bytes.extend_from_slice(&body);
        bytes.extend_from_slice(self.public_key.as_bytes());
        bytes.extend_from_slice(self.signature.as_bytes());

        Ok(bytes)
    }

    /// Unique transaction identifier.
    ///
    /// txid = SHA256d(canonical_signed_transaction)
    pub fn txid(&self) -> Result<Hash256, SignedTransactionError> {
        Ok(sha256d(&self.encode()?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignedTransactionError {
    Transaction(TransactionError),
    Crypto(CryptoError),
    SenderPublicKeyMismatch,
}

impl fmt::Display for SignedTransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transaction(error) => {
                write!(f, "transaction error: {error}")
            }

            Self::Crypto(error) => {
                write!(f, "cryptographic error: {error}")
            }

            Self::SenderPublicKeyMismatch => {
                write!(f, "sender address does not match signing public key")
            }
        }
    }
}

impl std::error::Error for SignedTransactionError {}

impl From<TransactionError> for SignedTransactionError {
    fn from(error: TransactionError) -> Self {
        Self::Transaction(error)
    }
}

impl From<CryptoError> for SignedTransactionError {
    fn from(error: CryptoError) -> Self {
        Self::Crypto(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::transaction::{TRANSACTION_KIND_TRANSFER, TRANSACTION_VERSION};

    fn alice() -> Keypair {
        Keypair::from_secret_bytes([1u8; 32])
    }

    fn alice_body() -> TransactionBody {
        let alice = alice();

        TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            1,
            7,
            Address::from_public_key(&alice.public_key()),
            Address::from_bytes([0x22; 32]),
            1_500_000_000,
            25_000,
            Vec::new(),
        )
    }

    #[test]
    fn fixed_public_key_derives_known_address() {
        let alice = alice();

        let address = Address::from_public_key(&alice.public_key());

        assert_eq!(
            address.to_hex(),
            "dcec2c244ec397bf30033d66011b95d2519276d864a48a54182042f7ce6d4e24"
        );
    }

    #[test]
    fn valid_signed_transaction_verifies() {
        let alice = alice();

        let transaction = SignedTransaction::sign(alice_body(), &alice).unwrap();

        assert_eq!(transaction.verify(), Ok(()));
    }

    #[test]
    fn wrong_sender_cannot_be_signed() {
        let alice = alice();

        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            1,
            7,
            Address::from_bytes([0x99; 32]),
            Address::from_bytes([0x22; 32]),
            1_500_000_000,
            25_000,
            Vec::new(),
        );

        assert_eq!(
            SignedTransaction::sign(body, &alice),
            Err(SignedTransactionError::SenderPublicKeyMismatch)
        );
    }

    #[test]
    fn modifying_signed_transaction_breaks_signature() {
        let alice = alice();

        let original = SignedTransaction::sign(alice_body(), &alice).unwrap();

        let body = TransactionBody::new(
            original.body().version(),
            original.body().kind(),
            original.body().chain_id(),
            original.body().nonce(),
            original.body().sender(),
            original.body().recipient(),
            original.body().amount() + 1,
            original.body().fee(),
            original.body().payload().to_vec(),
        );

        let modified =
            SignedTransaction::from_parts(body, original.public_key(), original.signature());

        assert_eq!(
            modified.verify(),
            Err(SignedTransactionError::Crypto(
                CryptoError::InvalidSignature
            ))
        );
    }

    #[test]
    fn signed_transaction_has_expected_size() {
        let alice = alice();

        let transaction = SignedTransaction::sign(alice_body(), &alice).unwrap();

        assert_eq!(transaction.encode().unwrap().len(), 196);
    }

    #[test]
    fn transaction_id_matches_fixed_test_vector() {
        let alice = alice();

        let transaction = SignedTransaction::sign(alice_body(), &alice).unwrap();

        assert_eq!(
            transaction.txid().unwrap().to_hex(),
            "696e296daece6e61f93f22156a433e9c72488495571163fff659bb43ccb5450e"
        );
    }
}
