use mirror_crypto::{Hash256, sha256d};

/// Calculate the Merkle root of an ordered list of hashes.
///
/// Mirror uses Bitcoin-style Merkle construction:
///
/// - each input is already a 256-bit transaction ID
/// - adjacent hashes are concatenated
/// - the concatenation is double-SHA256 hashed
/// - if a level has an odd number of hashes, the last hash is duplicated
/// - a single hash is its own Merkle root
///
/// The empty tree has a deterministic root equal to SHA256d(empty).
pub fn merkle_root(hashes: &[Hash256]) -> Hash256 {
    if hashes.is_empty() {
        return sha256d(&[]);
    }

    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut level = hashes.to_vec();

    while level.len() > 1 {
        if level.len() % 2 != 0 {
            let last = *level.last().expect("non-empty Merkle level");

            level.push(last);
        }

        let mut next = Vec::with_capacity(level.len() / 2);

        for pair in level.chunks_exact(2) {
            let mut bytes = [0u8; 64];

            bytes[..32].copy_from_slice(pair[0].as_bytes());

            bytes[32..].copy_from_slice(pair[1].as_bytes());

            next.push(sha256d(&bytes));
        }

        level = next;
    }

    level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn txid(name: &[u8]) -> Hash256 {
        sha256d(name)
    }

    #[test]
    fn empty_tree_has_known_root() {
        assert_eq!(
            merkle_root(&[]).to_hex(),
            "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"
        );
    }

    #[test]
    fn single_transaction_is_its_own_root() {
        let transaction = txid(b"tx1");

        assert_eq!(merkle_root(&[transaction]), transaction);
    }

    #[test]
    fn two_transactions_have_known_root() {
        let transactions = [txid(b"tx1"), txid(b"tx2")];

        assert_eq!(
            merkle_root(&transactions).to_hex(),
            "89c76c3eb6cab8c3a04c759a445fc64a8e09cf8b3035be4183244a4e714ee75b"
        );
    }

    #[test]
    fn odd_transaction_count_duplicates_last_hash() {
        let transactions = [txid(b"tx1"), txid(b"tx2"), txid(b"tx3")];

        assert_eq!(
            merkle_root(&transactions).to_hex(),
            "f4be66735d8fa90b4c8ba1d964cc8533cea95fa7bd273d57e32c813a4bc4d041"
        );
    }

    #[test]
    fn transaction_order_changes_root() {
        let first = [txid(b"tx1"), txid(b"tx2")];

        let second = [txid(b"tx2"), txid(b"tx1")];

        assert_ne!(merkle_root(&first), merkle_root(&second));
    }

    #[test]
    fn changing_one_transaction_changes_root() {
        let first = [txid(b"tx1"), txid(b"tx2"), txid(b"tx3")];

        let second = [txid(b"tx1"), txid(b"tx2"), txid(b"changed")];

        assert_ne!(merkle_root(&first), merkle_root(&second));
    }
}
