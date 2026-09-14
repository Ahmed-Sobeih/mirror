//! Durable block storage for Mirror.
//!
//! Each block is stored as its canonical encoded bytes in a separate file:
//!
//! ```text
//! 00000000000000000000.mrb
//! 00000000000000000001.mrb
//! 00000000000000000002.mrb
//! ...
//! ```
//!
//! Writes first go to a temporary file, are flushed to disk, and are then
//! renamed into place.

use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use mirror_core::{Block, CodecError, decode_block, encode_block};

const BLOCK_EXTENSION: &str = "mrb";
const TEMP_ATTEMPTS: u32 = 100;

/// Filesystem-backed canonical Mirror block store.
///
/// This initial implementation assumes a single writer.
/// Multi-process locking will be added before multiple node processes
/// are allowed to share the same data directory.
#[derive(Clone, Debug)]
pub struct BlockStore {
    root: PathBuf,
}

impl BlockStore {
    /// Open or create a Mirror block directory.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, StorageError> {
        let root = root.as_ref().to_path_buf();

        fs::create_dir_all(&root)?;

        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn contains(&self, height: u64) -> bool {
        self.block_path(height).is_file()
    }

    /// Persist one canonical block.
    ///
    /// Existing block heights are never silently overwritten.
    pub fn write_block(&self, height: u64, block: &Block) -> Result<PathBuf, StorageError> {
        let destination = self.block_path(height);

        if destination.exists() {
            return Err(StorageError::BlockAlreadyExists(height));
        }

        let bytes = encode_block(block)?;

        let (temporary, mut file) = self.create_temporary_file(height)?;

        let result = (|| -> Result<(), StorageError> {
            file.write_all(&bytes)?;

            // Ensure block bytes reach the filesystem before publication.
            file.sync_all()?;

            drop(file);

            // Single-writer protection against accidental replacement.
            if destination.exists() {
                return Err(StorageError::BlockAlreadyExists(height));
            }

            fs::rename(&temporary, &destination)?;

            sync_directory(&self.root)?;

            Ok(())
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }

        result?;

        Ok(destination)
    }

    /// Read and structurally decode a block.
    ///
    /// Consensus/state validation is intentionally performed by higher
    /// layers after decoding.
    pub fn read_block(&self, height: u64) -> Result<Block, StorageError> {
        let path = self.block_path(height);

        if !path.is_file() {
            return Err(StorageError::BlockNotFound(height));
        }

        let bytes = fs::read(path)?;

        Ok(decode_block(&bytes)?)
    }

    /// Return all stored block heights in ascending order.
    pub fn stored_heights(&self) -> Result<Vec<u64>, StorageError> {
        let mut heights = Vec::new();

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let name = entry.file_name();

            let Some(name) = name.to_str() else {
                continue;
            };

            let Some(height) = parse_block_filename(name) else {
                continue;
            };

            heights.push(height);
        }

        heights.sort_unstable();

        Ok(heights)
    }

    /// Load every stored block from height zero upward.
    ///
    /// The persisted blockchain must contain every height with no gaps.
    pub fn read_contiguous_blocks(&self) -> Result<Vec<Block>, StorageError> {
        let heights = self.stored_heights()?;

        let mut blocks = Vec::with_capacity(heights.len());

        let mut expected = 0u64;

        for height in heights {
            if height != expected {
                return Err(StorageError::NonContiguousChain {
                    expected,
                    got: height,
                });
            }

            blocks.push(self.read_block(height)?);

            expected = expected
                .checked_add(1)
                .ok_or(StorageError::HeightOverflow)?;
        }

        Ok(blocks)
    }

    pub fn block_path(&self, height: u64) -> PathBuf {
        self.root.join(format!("{height:020}.{BLOCK_EXTENSION}"))
    }

    fn create_temporary_file(&self, height: u64) -> Result<(PathBuf, File), StorageError> {
        let process = std::process::id();

        for attempt in 0..TEMP_ATTEMPTS {
            let path = self
                .root
                .join(format!(".{height:020}.{process}.{attempt}.tmp"));

            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    return Ok((path, file));
                }

                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    continue;
                }

                Err(error) => {
                    return Err(StorageError::Io(error));
                }
            }
        }

        Err(StorageError::TemporaryFileExhausted)
    }
}

fn parse_block_filename(name: &str) -> Option<u64> {
    let suffix = format!(".{BLOCK_EXTENSION}");

    let number = name.strip_suffix(&suffix)?;

    if number.len() != 20 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    number.parse().ok()
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), StorageError> {
    File::open(path)?.sync_all()?;

    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), StorageError> {
    Ok(())
}

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Codec(CodecError),

    BlockAlreadyExists(u64),
    BlockNotFound(u64),

    NonContiguousChain { expected: u64, got: u64 },

    HeightOverflow,
    TemporaryFileExhausted,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => {
                write!(f, "storage I/O error: {error}")
            }

            Self::Codec(error) => {
                write!(f, "stored block decode error: {error}")
            }

            Self::BlockAlreadyExists(height) => {
                write!(f, "block already exists at height {height}")
            }

            Self::BlockNotFound(height) => {
                write!(f, "block not found at height {height}")
            }

            Self::NonContiguousChain { expected, got } => {
                write!(
                    f,
                    "non-contiguous stored chain: expected height {expected}, found {got}"
                )
            }

            Self::HeightOverflow => {
                write!(f, "stored blockchain height overflow")
            }

            Self::TemporaryFileExhausted => {
                write!(f, "unable to allocate temporary block file")
            }
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<CodecError> for StorageError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use mirror_core::{
        Address, MIRROR_CHAIN_ID, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
        TRANSACTION_VERSION, TransactionBody,
    };

    use mirror_crypto::{Hash256, Keypair};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);

            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let path = std::env::temp_dir().join(format!(
                "mirror-storage-test-{}-{nanos}-{unique}",
                std::process::id()
            ));

            fs::create_dir_all(&path).unwrap();

            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn test_block(previous: Hash256, nonce: u64) -> Block {
        let alice = Keypair::from_secret_bytes([1u8; 32]);

        let bob = Keypair::from_secret_bytes([2u8; 32]);

        let body = TransactionBody::new(
            TRANSACTION_VERSION,
            TRANSACTION_KIND_TRANSFER,
            MIRROR_CHAIN_ID,
            0,
            Address::from_public_key(&alice.public_key()),
            Address::from_public_key(&bob.public_key()),
            NUSA_PER_MRY,
            0,
            Vec::new(),
        );

        let transaction = SignedTransaction::sign(body, &alice).unwrap();

        let mut block = Block::new(
            1,
            previous,
            Hash256::from_bytes([0x33; 32]),
            1_800_000_001,
            0x1f0f_ffff,
            vec![transaction],
        )
        .unwrap();

        block.set_nonce(nonce);

        block
    }

    #[test]
    fn block_round_trip_through_disk() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        let original = test_block(Hash256::default(), 42);

        store.write_block(0, &original).unwrap();

        let loaded = store.read_block(0).unwrap();

        assert_eq!(loaded, original);
        assert_eq!(loaded.hash(), original.hash());
    }

    #[test]
    fn existing_height_is_not_overwritten() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        let first = test_block(Hash256::default(), 1);

        let second = test_block(Hash256::default(), 2);

        store.write_block(0, &first).unwrap();

        assert!(matches!(
            store.write_block(0, &second),
            Err(StorageError::BlockAlreadyExists(0))
        ));

        assert_eq!(store.read_block(0).unwrap(), first);
    }

    #[test]
    fn stored_heights_are_sorted() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        store
            .write_block(2, &test_block(Hash256::from_bytes([2; 32]), 2))
            .unwrap();

        store
            .write_block(0, &test_block(Hash256::default(), 0))
            .unwrap();

        store
            .write_block(1, &test_block(Hash256::from_bytes([1; 32]), 1))
            .unwrap();

        assert_eq!(store.stored_heights().unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn corrupted_block_file_is_rejected() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        let block = test_block(Hash256::default(), 42);

        let path = store.write_block(0, &block).unwrap();

        let mut bytes = fs::read(&path).unwrap();

        bytes[0] ^= 0xff;

        fs::write(&path, bytes).unwrap();

        assert!(matches!(
            store.read_block(0),
            Err(StorageError::Codec(CodecError::InvalidBlockMagic))
        ));
    }

    #[test]
    fn missing_block_is_reported() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        assert!(matches!(
            store.read_block(7),
            Err(StorageError::BlockNotFound(7))
        ));
    }
}

#[cfg(test)]
mod contiguous_chain_tests {
    use super::*;

    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use mirror_crypto::Hash256;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);

            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let path = std::env::temp_dir().join(format!(
                "mirror-contiguous-test-{}-{nanos}-{unique}",
                std::process::id()
            ));

            fs::create_dir_all(&path).unwrap();

            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn empty_block(previous: Hash256, nonce: u64) -> Block {
        let mut block = Block::new(
            1,
            previous,
            Hash256::from_bytes([0x44; 32]),
            1_800_000_000,
            0x1f0f_ffff,
            Vec::new(),
        )
        .unwrap();

        block.set_nonce(nonce);

        block
    }

    #[test]
    fn contiguous_blocks_load_in_height_order() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        let block_zero = empty_block(Hash256::default(), 10);

        let block_one = empty_block(block_zero.hash(), 20);

        store.write_block(0, &block_zero).unwrap();

        store.write_block(1, &block_one).unwrap();

        let loaded = store.read_contiguous_blocks().unwrap();

        assert_eq!(loaded, vec![block_zero, block_one]);
    }

    #[test]
    fn gap_in_stored_chain_is_rejected() {
        let directory = TestDirectory::new();

        let store = BlockStore::open(&directory.path).unwrap();

        store
            .write_block(0, &empty_block(Hash256::default(), 10))
            .unwrap();

        store
            .write_block(2, &empty_block(Hash256::from_bytes([0x22; 32]), 30))
            .unwrap();

        assert!(matches!(
            store.read_contiguous_blocks(),
            Err(StorageError::NonContiguousChain {
                expected: 1,
                got: 2,
            })
        ));
    }
}
