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
    ffi::OsString,
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

        recover_interrupted_replacement(&root)?;

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

    /// Atomically replace the canonical block directory with a
    /// completely prepared chain.
    ///
    /// The caller must fully validate the candidate chain before calling
    /// this method. Storage itself only guarantees canonical persistence.
    ///
    /// The complete replacement is first written to a sibling staging
    /// directory. The old canonical directory is retained as a backup
    /// until the staged directory has been published.
    pub fn replace_chain(&self, blocks: &[Block]) -> Result<(), StorageError> {
        if blocks.is_empty() {
            return Err(StorageError::EmptyReplacementChain);
        }

        let paths = replacement_paths(&self.root)?;

        if paths.staging.exists() || paths.backup.exists() {
            return Err(StorageError::ReplacementArtifactsExist);
        }

        fs::create_dir(&paths.staging)?;

        let staging_store = Self {
            root: paths.staging.clone(),
        };

        let prepare_result = (|| -> Result<(), StorageError> {
            for (index, block) in blocks.iter().enumerate() {
                let height = u64::try_from(index).map_err(|_| StorageError::HeightOverflow)?;

                staging_store.write_block(height, block)?;
            }

            sync_directory(&paths.staging)?;

            sync_directory(&paths.parent)?;

            Ok(())
        })();

        if let Err(error) = prepare_result {
            let _ = fs::remove_dir_all(&paths.staging);

            return Err(error);
        }

        // From this point onward the replacement is recoverable:
        // if we crash after moving the old chain to backup but before
        // publishing staging, open() restores the backup.
        fs::rename(&self.root, &paths.backup)?;

        sync_directory(&paths.parent)?;

        if let Err(error) = fs::rename(&paths.staging, &self.root) {
            let _ = fs::rename(&paths.backup, &self.root);

            let _ = fs::remove_dir_all(&paths.staging);

            return Err(StorageError::Io(error));
        }

        sync_directory(&paths.parent)?;

        fs::remove_dir_all(&paths.backup)?;

        sync_directory(&paths.parent)?;

        Ok(())
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

#[derive(Debug)]
struct ReplacementPaths {
    parent: PathBuf,
    staging: PathBuf,
    backup: PathBuf,
}

fn replacement_paths(root: &Path) -> Result<ReplacementPaths, StorageError> {
    let name = root.file_name().ok_or(StorageError::InvalidStoreRoot)?;

    let parent = root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let mut staging_name = OsString::from(".");

    staging_name.push(name);
    staging_name.push(".mirror-reorg-staging");

    let mut backup_name = OsString::from(".");

    backup_name.push(name);
    backup_name.push(".mirror-reorg-backup");

    Ok(ReplacementPaths {
        staging: parent.join(staging_name),
        backup: parent.join(backup_name),
        parent,
    })
}

/// Recover a chain-directory swap interrupted by process or machine
/// failure.
///
/// Cases:
/// - canonical exists + backup exists: the new chain was published;
///   discard the old backup.
/// - canonical missing + backup exists: publication did not finish;
///   restore the previous canonical chain.
/// - stale staging without backup: replacement never reached commit;
///   discard staging.
fn recover_interrupted_replacement(root: &Path) -> Result<(), StorageError> {
    let paths = replacement_paths(root)?;

    let root_exists = root.is_dir();

    let backup_exists = paths.backup.is_dir();

    match (root_exists, backup_exists) {
        (true, true) => {
            fs::remove_dir_all(&paths.backup)?;

            if paths.staging.exists() {
                fs::remove_dir_all(&paths.staging)?;
            }

            sync_directory(&paths.parent)?;
        }

        (false, true) => {
            // The old canonical directory was moved aside, but the
            // candidate was never successfully published. Roll back.
            fs::rename(&paths.backup, root)?;

            if paths.staging.exists() {
                fs::remove_dir_all(&paths.staging)?;
            }

            sync_directory(&paths.parent)?;
        }

        (_, false) => {
            if paths.staging.exists() {
                fs::remove_dir_all(&paths.staging)?;

                sync_directory(&paths.parent)?;
            }
        }
    }

    Ok(())
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

    EmptyReplacementChain,
    InvalidStoreRoot,
    ReplacementArtifactsExist,

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

            Self::EmptyReplacementChain => {
                write!(f, "cannot replace canonical storage with an empty chain")
            }

            Self::InvalidStoreRoot => {
                write!(f, "block store root has no usable directory name")
            }

            Self::ReplacementArtifactsExist => {
                write!(
                    f,
                    "chain replacement staging or backup directory already exists"
                )
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

#[cfg(test)]
mod chain_replacement_tests {
    use super::*;

    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use mirror_crypto::Hash256;

    static NEXT_REORG_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct ReorgTestDirectory {
        path: PathBuf,
    }

    impl ReorgTestDirectory {
        fn new() -> Self {
            let unique = NEXT_REORG_DIRECTORY.fetch_add(1, Ordering::Relaxed);

            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let path = std::env::temp_dir().join(format!(
                "mirror-reorg-storage-test-{}-{nanos}-{unique}",
                std::process::id()
            ));

            fs::create_dir_all(&path).unwrap();

            Self { path }
        }

        fn block_root(&self) -> PathBuf {
            self.path.join("blocks")
        }
    }

    impl Drop for ReorgTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn block(previous: Hash256, marker: u8) -> Block {
        Block::new(
            1,
            previous,
            Hash256::from_bytes([marker; 32]),
            1_800_000_000 + u64::from(marker),
            0x1f0f_ffff,
            Vec::new(),
        )
        .unwrap()
    }

    #[test]
    fn complete_chain_can_replace_existing_chain() {
        let directory = ReorgTestDirectory::new();

        let store = BlockStore::open(directory.block_root()).unwrap();

        let old_zero = block(Hash256::default(), 1);

        let old_one = block(old_zero.hash(), 2);

        store.write_block(0, &old_zero).unwrap();

        store.write_block(1, &old_one).unwrap();

        let new_zero = block(Hash256::default(), 10);

        let new_one = block(new_zero.hash(), 11);

        let new_two = block(new_one.hash(), 12);

        let candidate = vec![new_zero.clone(), new_one.clone(), new_two.clone()];

        store.replace_chain(&candidate).unwrap();

        let loaded = store.read_contiguous_blocks().unwrap();

        assert_eq!(loaded, candidate);
    }

    #[test]
    fn interrupted_swap_restores_previous_chain() {
        let directory = ReorgTestDirectory::new();

        let root = directory.block_root();

        let store = BlockStore::open(&root).unwrap();

        let original = block(Hash256::default(), 20);

        store.write_block(0, &original).unwrap();

        let paths = replacement_paths(&root).unwrap();

        fs::create_dir(&paths.staging).unwrap();

        let staging_store = BlockStore {
            root: paths.staging.clone(),
        };

        staging_store
            .write_block(0, &block(Hash256::default(), 21))
            .unwrap();

        // Simulate a crash after old canonical storage was moved
        // to backup but before staging was published.
        fs::rename(&root, &paths.backup).unwrap();

        let recovered = BlockStore::open(&root).unwrap();

        let loaded = recovered.read_block(0).unwrap();

        assert_eq!(loaded, original);

        assert!(!paths.backup.exists());

        assert!(!paths.staging.exists());
    }

    #[test]
    fn empty_chain_replacement_is_rejected() {
        let directory = ReorgTestDirectory::new();

        let store = BlockStore::open(directory.block_root()).unwrap();

        assert!(matches!(
            store.replace_chain(&[]),
            Err(StorageError::EmptyReplacementChain)
        ));
    }
}
