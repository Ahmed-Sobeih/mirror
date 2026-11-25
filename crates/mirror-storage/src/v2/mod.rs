//! Explicit, bounded persistence of one already accepted V2 chain selection.
//!
//! Initialization, exclusive reopening and shared read-only inspection are
//! separate operations. Every reopen requires independently selected network and
//! checkpoint pins and replays all actual blocks through `ChainV2`. `StoreV2`
//! preserves immutable profile 1, with generation zero and no update API.
//! `AppendStoreV2` supplies separately versioned profile 2 accepted-child updates
//! and checked clean-state reconciliation. See `docs/STORAGE_V2.md` and
//! `docs/STORAGE_APPEND_V2.md` for exact boundaries and filesystem assumptions.
//! Profile 3 adds only replacement metadata and pure accepted-chain preparation;
//! it has no store, publication or recovery API yet. See `docs/STORAGE_REPLACEMENT_V2.md`.

pub mod append;
pub mod append_selection;
mod lock;
pub mod replacement_plan;
pub mod replacement_selection;
pub mod selection;

use std::{
    ffi::OsString,
    fmt,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};

use mirror_chain::v2::{
    ChainReplayErrorV2, ChainReplayExpectationV2, ChainV2, MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2,
    ManifestErrorV2, ManifestIdentityV2, NetworkManifestV2, manifest::MAX_MANIFEST_BYTES_V2,
};
use mirror_core::v2::{BlockErrorV2, MIN_ENCODED_BLOCK_LEN_V2};

use lock::DirectoryLockV2;
pub use append::{AppendStorageErrorV2, AppendStoreV2};
pub use append::recovery::{
    AppendRecoveryActionV2, AppendRecoveryErrorV2, AppendRecoveryRequestV2,
    MAX_APPEND_RECOVERY_INPUT_BYTES_V2, MAX_APPEND_RECOVERY_READ_BYTES_V2,
};
pub use append_selection::{
    APPEND_SELECTION_LEN_V2, AppendCheckpointV2, AppendSelectionErrorV2, AppendSelectionV2,
};
pub use replacement_plan::{ReplacementPlanErrorV2, ReplacementPlanV2, prepare_replacement_v2};
pub use replacement_selection::{
    REPLACEMENT_SELECTION_LEN_V2, ReplacementCheckpointV2, ReplacementSelectionErrorV2,
    ReplacementSelectionV2,
};
pub use selection::{STORAGE_SELECTION_LEN_V2, SelectionErrorV2, SelectionRecordV2};

const MANIFEST_NAME: &str = "manifest.v2";
const BLOCKS_NAME: &str = "blocks";
const SELECTION_NAME: &str = "selection.v2";
const PENDING_NAME: &str = "selection.pending";
const MIN_BLOCK_BYTES: u64 = MIN_ENCODED_BLOCK_LEN_V2 as u64;

/// Lock ownership is local to cooperating processes, not consensus authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreAccessV2 {
    Writer,
    ReadOnly,
}

/// An immutable validated selection, with its directory lock held for life.
///
/// No raw file handle, mutable chain access, clone, append or recovery API is
/// exposed. Dropping closes this descriptor. The operating-system lock is released
/// after every descriptor for its open file description has closed, including
/// any temporarily inherited by a concurrent process fork before exec.
#[derive(Debug)]
pub struct StoreV2 {
    root: PathBuf,
    chain: ChainV2,
    selection: SelectionRecordV2,
    access: StoreAccessV2,
    _lock: DirectoryLockV2,
}

impl StoreV2 {
    /// Explicitly initialize an absent or empty directory from an accepted chain.
    /// The parent must already exist. No legacy or partial store is overwritten.
    pub fn initialize(root: impl AsRef<Path>, chain: &ChainV2) -> Result<Self, StorageErrorV2> {
        initialize_inner(root.as_ref(), chain, |_| Ok(()))
    }

    /// Reopen an existing selected history while holding exclusive ownership.
    /// Both expectations must come from the caller's independently chosen branch.
    pub fn open_writer(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: ChainReplayExpectationV2,
    ) -> Result<Self, StorageErrorV2> {
        open_selected(root.as_ref(), identity, expected, StoreAccessV2::Writer)
    }

    /// Read and validate without creating, writing, repairing or mining anything.
    /// A shared lock remains held until this handle is dropped. OS access-time
    /// updates are outside the application's no-write guarantee.
    pub fn inspect(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: ChainReplayExpectationV2,
    ) -> Result<Self, StorageErrorV2> {
        open_selected(root.as_ref(), identity, expected, StoreAccessV2::ReadOnly)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub const fn chain(&self) -> &ChainV2 {
        &self.chain
    }

    pub const fn selection(&self) -> &SelectionRecordV2 {
        &self.selection
    }

    pub const fn access(&self) -> StoreAccessV2 {
        self.access
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InitBoundary {
    RootReady,
    ManifestSynced,
    BlockSynced(u64),
    BlocksDirectorySynced,
    PreparedDirectorySynced,
    SelectionStaged,
    SelectionPublished,
    SelectionDirectorySynced,
}

fn initialize_inner(
    root: &Path,
    chain: &ChainV2,
    mut after: impl FnMut(InitBoundary) -> Result<(), StorageErrorV2>,
) -> Result<StoreV2, StorageErrorV2> {
    require_platform()?;
    // Serialize accepted, bounded source data before making filesystem changes.
    let selection = SelectionRecordV2::from_chain(chain)?;
    let manifest = chain.manifest().encode()?;
    let mut blocks = Vec::new();
    blocks
        .try_reserve_exact(chain.blocks().len())
        .map_err(|_| StorageErrorV2::AllocationFailed)?;
    for block in chain.blocks() {
        blocks.push(
            block
                .encode(chain.config())
                .map_err(StorageErrorV2::Block)?,
        );
    }
    let root = checked_root_path(root)?;
    let parent = root.parent().ok_or(StorageErrorV2::InvalidPath)?;
    match fs::symlink_metadata(&root) {
        Ok(metadata) if !metadata.is_dir() => return Err(StorageErrorV2::NotDirectory),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&root)?,
        Err(error) => return Err(error.into()),
    }
    let lock = DirectoryLockV2::acquire(&root, StoreAccessV2::Writer)?;
    let names = root_names(&root)?;
    if !names.is_empty() {
        return Err(uninitialized_error(&root, &names));
    }
    let mut publication_attempted = false;
    let result = (|| -> Result<(), StorageErrorV2> {
        // Persist the directory's own parent entry before publishing its marker.
        File::open(parent)?.sync_all()?;
        after(InitBoundary::RootReady)?;
        let blocks_root = root.join(BLOCKS_NAME);
        fs::create_dir(&blocks_root)?;
        write_new_synced(&root.join(MANIFEST_NAME), &manifest)?;
        after(InitBoundary::ManifestSynced)?;
        for (height, bytes) in blocks.iter().enumerate() {
            write_new_synced(&blocks_root.join(block_name(height as u64)), bytes)?;
            after(InitBoundary::BlockSynced(height as u64))?;
        }
        File::open(&blocks_root)?.sync_all()?;
        after(InitBoundary::BlocksDirectorySynced)?;
        lock.sync()?;
        after(InitBoundary::PreparedDirectorySynced)?;
        write_new_synced(&root.join(PENDING_NAME), &selection.encode())?;
        after(InitBoundary::SelectionStaged)?;
        // A failed rename may have an uncertain result on a faulty filesystem.
        // No further writes or returned ownership follow any such error.
        publication_attempted = true;
        fs::rename(root.join(PENDING_NAME), root.join(SELECTION_NAME))?;
        after(InitBoundary::SelectionPublished)?;
        lock.sync()?;
        after(InitBoundary::SelectionDirectorySynced)?;
        Ok(())
    })();
    if let Err(error) = result {
        return Err(if publication_attempted {
            StorageErrorV2::PublicationUncertain(Box::new(error))
        } else {
            StorageErrorV2::InitializationIncomplete(Box::new(error))
        });
    }
    Ok(StoreV2 {
        root,
        chain: chain.clone(),
        selection,
        access: StoreAccessV2::Writer,
        _lock: lock,
    })
}

fn open_selected(
    root: &Path,
    identity: ManifestIdentityV2,
    expected: ChainReplayExpectationV2,
    access: StoreAccessV2,
) -> Result<StoreV2, StorageErrorV2> {
    require_platform()?;
    if identity.config_hash != expected.config_hash
        || identity.genesis_hash != expected.genesis_hash
    {
        return Err(StorageErrorV2::IdentityMismatch);
    }
    let root = checked_root_path(root)?;
    let lock = DirectoryLockV2::acquire(&root, access)?;
    let names = root_names(&root)?;
    if !names.iter().any(|name| name == SELECTION_NAME) {
        return Err(uninitialized_error(&root, &names));
    }
    if names.len() != 3
        || ![SELECTION_NAME, MANIFEST_NAME, BLOCKS_NAME]
            .iter()
            .all(|required| names.iter().any(|name| name == required))
    {
        return Err(StorageErrorV2::InvalidLayout(
            "unexpected or missing root entry",
        ));
    }
    let record_bytes = read_regular_bounded(
        &root.join(SELECTION_NAME),
        STORAGE_SELECTION_LEN_V2 as u64,
        Some(STORAGE_SELECTION_LEN_V2 as u64),
    )?;
    let selection = SelectionRecordV2::decode_for(&record_bytes, expected)?;
    let manifest_bytes = read_regular_bounded(
        &root.join(MANIFEST_NAME),
        MAX_MANIFEST_BYTES_V2 as u64,
        Some(u64::from(selection.manifest_len())),
    )?;
    selection.verify_manifest_bytes(&manifest_bytes)?;
    let manifest = NetworkManifestV2::decode_for(&manifest_bytes, identity)?;
    let blocks_root = root.join(BLOCKS_NAME);
    check_directory(&blocks_root)?;
    let lengths = preflight_blocks(&blocks_root, &selection, &manifest)?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(lengths.len())
        .map_err(|_| StorageErrorV2::AllocationFailed)?;
    for (height, len) in lengths.iter().enumerate() {
        encoded.push(read_regular_bounded(
            &blocks_root.join(block_name(height as u64)),
            *len,
            Some(*len),
        )?);
    }
    let borrowed: Vec<_> = encoded.iter().map(Vec::as_slice).collect();
    let chain = ChainV2::replay_encoded(&manifest, &borrowed, expected)?;
    Ok(StoreV2 {
        root,
        chain,
        selection,
        access,
        _lock: lock,
    })
}

fn preflight_blocks(
    root: &Path,
    selection: &SelectionRecordV2,
    manifest: &NetworkManifestV2,
) -> Result<Vec<u64>, StorageErrorV2> {
    let count = selection.block_count() as usize;
    let mut lengths = vec![0; count];
    let mut entries = 0usize;
    let mut total = 0u64;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        entries += 1;
        if entries > count || entries > MAX_CHAIN_BLOCKS_V2 {
            return Err(StorageErrorV2::InvalidLayout("extra block entry"));
        }
        let name = entry.file_name();
        let height = parse_block_name(&name)
            .ok_or(StorageErrorV2::InvalidLayout("noncanonical block filename"))?;
        let index = usize::try_from(height)
            .map_err(|_| StorageErrorV2::InvalidLayout("block height overflow"))?;
        if index >= count {
            return Err(StorageErrorV2::InvalidLayout("unselected block height"));
        }
        let metadata = regular_metadata(&entry.path())?;
        let len = metadata.len();
        let limit = u64::from(manifest.config().limits().max_block_bytes);
        check_limit(len, limit)?;
        if len < MIN_BLOCK_BYTES {
            return Err(StorageErrorV2::LengthMismatch("truncated block file"));
        }
        total = total
            .checked_add(len)
            .ok_or(StorageErrorV2::InputTooLarge {
                limit: MAX_CHAIN_BYTES_V2 as u64,
                actual: u64::MAX,
            })?;
        check_limit(total, MAX_CHAIN_BYTES_V2 as u64)?;
        lengths[index] = len;
    }
    if entries != count || lengths.contains(&0) {
        return Err(StorageErrorV2::InvalidLayout("missing selected block"));
    }
    if total != selection.blocks_bytes() {
        return Err(StorageErrorV2::LengthMismatch("selected block byte total"));
    }
    Ok(lengths)
}

fn block_name(height: u64) -> String {
    format!("{height:020}.mrb2")
}

fn parse_block_name(name: &OsString) -> Option<u64> {
    let name = name.to_str()?;
    if name.len() != 25 || !name.ends_with(".mrb2") {
        return None;
    }
    let digits = name.get(..20)?;
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let height = digits.parse().ok()?;
    (block_name(height) == name).then_some(height)
}

fn root_names(root: &Path) -> Result<Vec<OsString>, StorageErrorV2> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        if names.len() == 4 {
            return Err(StorageErrorV2::InvalidLayout("too many root entries"));
        }
        names.push(entry?.file_name());
    }
    Ok(names)
}

fn uninitialized_error(root: &Path, names: &[OsString]) -> StorageErrorV2 {
    if names.is_empty() {
        return StorageErrorV2::EmptyDirectory;
    }
    if names.iter().any(|name| name == SELECTION_NAME) {
        return StorageErrorV2::AlreadyInitialized;
    }
    if names
        .iter()
        .any(|name| name == MANIFEST_NAME || name == PENDING_NAME)
    {
        return StorageErrorV2::PartialInitialization;
    }
    if names
        .iter()
        .any(|name| Path::new(name).extension().is_some_and(|ext| ext == "mrb"))
    {
        return StorageErrorV2::LegacyDirectory;
    }
    if names.iter().any(|name| name == BLOCKS_NAME) {
        let blocks = root.join(BLOCKS_NAME);
        // Classification alone never follows a symlink, opens block contents or
        // attempts legacy recovery. Strict selection opening checks all entries.
        if let Err(error) = check_directory(&blocks) {
            return error;
        }
        match fs::read_dir(&blocks) {
            Ok(entries) => {
                for (index, entry) in entries.enumerate() {
                    if index == MAX_CHAIN_BLOCKS_V2 {
                        return StorageErrorV2::InvalidLayout("too many unselected entries");
                    }
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(error) => return error.into(),
                    };
                    if entry.path().extension().is_some_and(|ext| ext == "mrb") {
                        return StorageErrorV2::LegacyDirectory;
                    }
                }
                return StorageErrorV2::PartialInitialization;
            }
            Err(error) => return error.into(),
        }
    }
    StorageErrorV2::ForeignDirectory
}

fn require_platform() -> Result<(), StorageErrorV2> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(StorageErrorV2::UnsupportedPlatform)
    }
}

/// Static component checks are defense in depth under a trusted parent. They
/// do not claim race-free traversal against an owner who swaps paths mid-call.
fn checked_root_path(input: &Path) -> Result<PathBuf, StorageErrorV2> {
    if input.as_os_str().is_empty() {
        return Err(StorageErrorV2::InvalidPath);
    }
    let absolute = if input.is_absolute() {
        input.to_path_buf()
    } else {
        std::env::current_dir()?.join(input)
    };
    let mut root = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir | Component::Normal(_) => root.push(component),
            Component::CurDir => {}
            _ => return Err(StorageErrorV2::InvalidPath),
        }
    }
    if root.parent().is_none() {
        return Err(StorageErrorV2::InvalidPath);
    }
    let mut path = PathBuf::new();
    for component in root.components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(StorageErrorV2::UnsafePath("symlink path component"));
                }
                if !metadata.is_dir() {
                    return Err(StorageErrorV2::NotDirectory);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && path == root => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(StorageErrorV2::MissingDirectory);
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(root)
}

fn check_directory(path: &Path) -> Result<(), StorageErrorV2> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(StorageErrorV2::UnsafePath("symlink directory"));
    }
    if !metadata.is_dir() {
        return Err(StorageErrorV2::NotDirectory);
    }
    Ok(())
}

fn regular_metadata(path: &Path) -> Result<Metadata, StorageErrorV2> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(StorageErrorV2::UnsafePath(
            "expected regular, non-symlink file",
        ));
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(StorageErrorV2::UnsafePath("hardlinked managed file"));
        }
    }
    Ok(metadata)
}

fn read_regular_bounded(
    path: &Path,
    limit: u64,
    exact: Option<u64>,
) -> Result<Vec<u8>, StorageErrorV2> {
    let before = regular_metadata(path)?;
    check_limit(before.len(), limit)?;
    if exact.is_some_and(|expected| expected != before.len()) {
        return Err(StorageErrorV2::LengthMismatch(
            "file length does not match selection",
        ));
    }
    let file = File::open(path)?;
    let opened = file.metadata()?;
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if !opened.is_file()
            || opened.dev() != before.dev()
            || opened.ino() != before.ino()
            || opened.nlink() != 1
        {
            return Err(StorageErrorV2::UnsafePath(
                "file identity changed during open",
            ));
        }
    }
    if opened.len() != before.len() {
        return Err(StorageErrorV2::LengthMismatch("file changed during open"));
    }
    // Use the checked exact length plus one sentinel byte, not a fresh unbounded
    // fs::read. Growth after metadata cannot make allocation escape the budget.
    let read_limit = before
        .len()
        .checked_add(1)
        .ok_or(StorageErrorV2::InvalidPath)?;
    let capacity = usize::try_from(read_limit).map_err(|_| StorageErrorV2::AllocationFailed)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| StorageErrorV2::AllocationFailed)?;
    file.take(read_limit).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != before.len() {
        return Err(StorageErrorV2::LengthMismatch("file changed while reading"));
    }
    Ok(bytes)
}

fn check_limit(actual: u64, limit: u64) -> Result<(), StorageErrorV2> {
    if actual > limit {
        Err(StorageErrorV2::InputTooLarge { limit, actual })
    } else {
        Ok(())
    }
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), StorageErrorV2> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

#[derive(Debug)]
pub enum StorageErrorV2 {
    UnsupportedPlatform,
    InvalidPath,
    MissingDirectory,
    NotDirectory,
    EmptyDirectory,
    LegacyDirectory,
    ForeignDirectory,
    PartialInitialization,
    AlreadyInitialized,
    Busy,
    UnsafePath(&'static str),
    InvalidLayout(&'static str),
    InputTooLarge {
        limit: u64,
        actual: u64,
    },
    LengthMismatch(&'static str),
    IdentityMismatch,
    AllocationFailed,
    Selection(SelectionErrorV2),
    Manifest(ManifestErrorV2),
    Block(BlockErrorV2),
    Replay(ChainReplayErrorV2),
    Io(io::Error),
    /// No selection publication was attempted; partial files remain untouched.
    InitializationIncomplete(Box<StorageErrorV2>),
    /// Publication may have happened; inspect against retained external pins.
    PublicationUncertain(Box<StorageErrorV2>),
}

impl fmt::Display for StorageErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "V2 storage I/O: {error}"),
            Self::InitializationIncomplete(error) => write!(
                formatter,
                "V2 initialization incomplete; retained files require inspection: {error}"
            ),
            Self::PublicationUncertain(error) => write!(
                formatter,
                "V2 selection publication uncertain; reconcile against the external checkpoint: {error}"
            ),
            other => write!(formatter, "V2 storage: {other:?}"),
        }
    }
}

impl std::error::Error for StorageErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Selection(error) => Some(error),
            Self::Manifest(error) => Some(error),
            Self::Block(error) => Some(error),
            Self::Replay(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InitializationIncomplete(error) | Self::PublicationUncertain(error) => {
                Some(error)
            }
            _ => None,
        }
    }
}

impl From<io::Error> for StorageErrorV2 {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SelectionErrorV2> for StorageErrorV2 {
    fn from(error: SelectionErrorV2) -> Self {
        Self::Selection(error)
    }
}

impl From<ManifestErrorV2> for StorageErrorV2 {
    fn from(error: ManifestErrorV2) -> Self {
        Self::Manifest(error)
    }
}

impl From<ChainReplayErrorV2> for StorageErrorV2 {
    fn from(error: ChainReplayErrorV2) -> Self {
        Self::Replay(error)
    }
}

#[cfg(test)]
mod tests;
