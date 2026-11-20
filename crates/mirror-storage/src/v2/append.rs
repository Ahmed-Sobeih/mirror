//! Profile 2: bounded one-block publication with explicit failure fencing.
//!
//! This is separate from the frozen immutable `StoreV2` profile. Reopening and
//! consuming reconciliation require an independently retained generation and
//! complete chain checkpoint. No pending file, checksum or larger generation
//! chooses authority, repairs staging, or establishes global freshness.

pub mod recovery;

use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use mirror_chain::v2::{ChainErrorV2, ChainV2, ManifestIdentityV2, NetworkManifestV2};
use mirror_core::v2::BlockV2;

use super::{
    BLOCKS_NAME, MANIFEST_NAME, PENDING_NAME, SELECTION_NAME, StorageErrorV2, StoreAccessV2,
    append_selection::{
        APPEND_SELECTION_LEN_V2, AppendCheckpointV2, AppendSelectionErrorV2, AppendSelectionV2,
    },
    block_name, check_directory, checked_root_path,
    lock::DirectoryLockV2,
    preflight_blocks, read_regular_bounded, regular_metadata, require_platform, root_names,
    uninitialized_error,
};

/// An exclusively appendable or shared inspection handle for storage profile 2.
///
/// Every successful append publishes one complete accepted child. On a disk
/// discrepancy or write failure, normal chain/selection access and all further
/// appends are fenced. `last_confirmed_checkpoint` then describes only the last
/// successful in-memory publication; disk may already select the candidate.
/// Consume the handle with `reconcile` or drop it and explicitly reopen with
/// independently retained pins. No mutable chain, raw descriptor or Clone exists.
#[derive(Debug)]
pub struct AppendStoreV2 {
    root: PathBuf,
    chain: ChainV2,
    selection: AppendSelectionV2,
    access: StoreAccessV2,
    fenced: bool,
    lock: DirectoryLockV2,
}

impl AppendStoreV2 {
    /// Create profile 2 only in an absent or empty final directory. No existing
    /// immutable/legacy store is adopted, migrated, overwritten or cleaned up.
    /// Generation starts at zero even if the supplied accepted chain is longer.
    pub fn initialize(
        root: impl AsRef<Path>,
        chain: &ChainV2,
    ) -> Result<Self, AppendStorageErrorV2> {
        initialize_inner(root.as_ref(), chain, |_| Ok(()))
    }

    /// Fully replay the externally selected history, then resynchronize its
    /// validated files and directories before permitting another append. This
    /// reestablishes the synchronization boundary after an uncertain prior call;
    /// it does not retrospectively certify an earlier failed synchronization.
    pub fn open_writer(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: AppendCheckpointV2,
    ) -> Result<Self, AppendStorageErrorV2> {
        open_inner(
            root.as_ref(),
            identity,
            expected,
            StoreAccessV2::Writer,
            |_| Ok(()),
        )
    }

    /// Shared, read-only full replay. Does not synchronize, initialize, repair,
    /// promote a pending selector or grant the ability to append.
    pub fn inspect(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: AppendCheckpointV2,
    ) -> Result<Self, AppendStorageErrorV2> {
        open_inner(
            root.as_ref(),
            identity,
            expected,
            StoreAccessV2::ReadOnly,
            |_| Ok(()),
        )
    }

    /// Validate a complete child through ChainV2, synchronize its bytes, publish
    /// the next selector and synchronize the unchanged root before advancing
    /// memory. The returned checkpoint is for the caller to retain independently.
    pub fn append(&mut self, block: BlockV2) -> Result<AppendCheckpointV2, AppendStorageErrorV2> {
        self.append_inner(block, |_| Ok(()))
    }

    /// Consume a writer while retaining the SAME directory lock, then fully
    /// replay and synchronize the independently selected complete on-disk state.
    /// Success returns an unfenced writer; any error drops ownership and returns
    /// no handle. Strict-clean-layout recovery rejects and preserves all pending,
    /// extra-tail, partial or unexplained files; it never promotes or deletes them.
    pub fn reconcile(self, expected: AppendCheckpointV2) -> Result<Self, AppendStorageErrorV2> {
        self.reconcile_inner(expected, |_| Ok(()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub const fn access(&self) -> StoreAccessV2 {
        self.access
    }

    pub const fn is_fenced(&self) -> bool {
        self.fenced
    }

    /// The last confirmed memory checkpoint, NOT a claim about selected disk
    /// state after a failed append. Its explicit name prevents silent fallback.
    pub fn last_confirmed_checkpoint(&self) -> AppendCheckpointV2 {
        self.selection.checkpoint()
    }

    pub fn chain(&self) -> Result<&ChainV2, AppendStorageErrorV2> {
        self.require_unfenced()?;
        Ok(&self.chain)
    }

    pub fn selection(&self) -> Result<&AppendSelectionV2, AppendStorageErrorV2> {
        self.require_unfenced()?;
        Ok(&self.selection)
    }

    pub fn checkpoint(&self) -> Result<AppendCheckpointV2, AppendStorageErrorV2> {
        self.require_unfenced()?;
        Ok(self.selection.checkpoint())
    }

    fn require_unfenced(&self) -> Result<(), AppendStorageErrorV2> {
        if self.fenced {
            Err(AppendStorageErrorV2::Fenced)
        } else {
            Ok(())
        }
    }

    fn require_writer(&self) -> Result<(), AppendStorageErrorV2> {
        if self.access != StoreAccessV2::Writer {
            Err(AppendStorageErrorV2::ReadOnly)
        } else {
            Ok(())
        }
    }

    fn append_inner(
        &mut self,
        block: BlockV2,
        mut after: impl FnMut(AppendBoundary) -> Result<(), AppendStorageErrorV2>,
    ) -> Result<AppendCheckpointV2, AppendStorageErrorV2> {
        self.require_writer()?;
        self.require_unfenced()?;

        // All candidate validation, budget checks and encoding finish before
        // touching disk. Rejection here keeps the original handle usable.
        let generation = self
            .selection
            .generation()
            .checked_add(1)
            .ok_or(AppendStorageErrorV2::GenerationOverflow)?;
        let mut candidate = self.chain.clone();
        candidate.append(block)?;
        let next = AppendSelectionV2::from_chain(&candidate, generation)?;
        let encoded_block = candidate
            .blocks()
            .last()
            .expect("accepted chain contains genesis")
            .encode(candidate.config())
            .map_err(StorageErrorV2::Block)?;
        let next_bytes = next.encode();
        let candidate_checkpoint = next.checkpoint();
        let block_path = self
            .root
            .join(BLOCKS_NAME)
            .join(block_name(candidate.height()));
        let pending_path = self.root.join(PENDING_NAME);
        let selected_path = self.root.join(SELECTION_NAME);

        // A held cooperative lock prevents another conforming writer, but old
        // files can still be damaged. Revalidate the entire selected disk history
        // against retained pins and compare the complete ledger/counters before
        // preparing a new generation. A discrepancy fences the live handle.
        self.fenced = true;
        self.lock.verify_root(&self.root)?;
        let disk = load_selected(
            &self.root,
            self.chain.manifest().identity(),
            self.selection.checkpoint(),
        )?;
        if disk.selection != self.selection || disk.chain != self.chain {
            return Err(AppendStorageErrorV2::SelectionChanged);
        }
        drop(disk);

        let mut publication_attempted = false;
        let result = (|| -> Result<(), AppendStorageErrorV2> {
            after(AppendBoundary::BeforeBlockCreate)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&block_path)?;
            after(AppendBoundary::BlockCreated)?;
            file.write_all(&encoded_block)?;
            after(AppendBoundary::BlockWritten)?;
            file.sync_all()?;
            after(AppendBoundary::BlockSynced)?;
            drop(file);
            sync_directory(&self.root.join(BLOCKS_NAME))?;
            after(AppendBoundary::BlocksDirectorySynced)?;

            let mut pending = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&pending_path)?;
            after(AppendBoundary::SelectorCreated)?;
            pending.write_all(&next_bytes)?;
            after(AppendBoundary::SelectorWritten)?;
            pending.sync_all()?;
            after(AppendBoundary::SelectorSynced)?;
            drop(pending);

            self.lock.verify_root(&self.root)?;
            compare_selector(&self.root, &self.selection)?;
            // Once publication may be attempted, errors cannot be reported as a
            // definite old selection. The fence stays set and memory stays old.
            publication_attempted = true;
            after(AppendBoundary::PublicationAttempt)?;
            fs::rename(&pending_path, &selected_path)?;
            after(AppendBoundary::SelectionPublished)?;
            self.lock.sync()?;
            after(AppendBoundary::RootSynced)?;
            Ok(())
        })();
        if let Err(error) = result {
            return Err(if publication_attempted {
                AppendStorageErrorV2::PublicationUncertain {
                    candidate: candidate_checkpoint,
                    source: Box::new(error),
                }
            } else {
                AppendStorageErrorV2::UpdateIncomplete {
                    candidate: candidate_checkpoint,
                    source: Box::new(error),
                }
            });
        }

        // No fallible operation remains. Publish both state and local generation
        // together only after the selected directory entry is synchronized.
        self.chain = candidate;
        self.selection = next;
        self.fenced = false;
        Ok(candidate_checkpoint)
    }

    fn reconcile_inner(
        mut self,
        expected: AppendCheckpointV2,
        mut after: impl FnMut(ReconcileBoundary) -> Result<(), AppendStorageErrorV2>,
    ) -> Result<Self, AppendStorageErrorV2> {
        self.require_writer()?;
        self.fenced = true;
        let result = (|| {
            check_identity(self.chain.manifest().identity(), expected)?;
            self.lock.verify_root(&self.root)?;
            let selected = load_selected(&self.root, self.chain.manifest().identity(), expected)?;
            after(ReconcileBoundary::Validated)?;
            sync_selected(&self.root, &self.lock, &selected, &mut after)?;
            Ok::<_, AppendStorageErrorV2>(selected)
        })();
        let selected =
            result.map_err(|error| AppendStorageErrorV2::ReconciliationFailed(Box::new(error)))?;
        self.chain = selected.chain;
        self.selection = selected.selection;
        self.fenced = false;
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppendBoundary {
    BeforeBlockCreate,
    BlockCreated,
    BlockWritten,
    BlockSynced,
    BlocksDirectorySynced,
    SelectorCreated,
    SelectorWritten,
    SelectorSynced,
    PublicationAttempt,
    SelectionPublished,
    RootSynced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReconcileBoundary {
    Validated,
    ManifestSynced,
    BlockSynced(u64),
    BlocksDirectorySynced,
    SelectionSynced,
    RootSynced,
    SelectionRechecked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InitializeBoundary {
    ParentSynced,
    ManifestSynced,
    BlockSynced(u64),
    BlocksDirectorySynced,
    RootPrepared,
    SelectionStaged,
    SelectionPublished,
    RootSynced,
}

fn initialize_inner(
    input: &Path,
    chain: &ChainV2,
    mut after: impl FnMut(InitializeBoundary) -> Result<(), AppendStorageErrorV2>,
) -> Result<AppendStoreV2, AppendStorageErrorV2> {
    require_platform()?;
    let selection = AppendSelectionV2::from_chain(chain, 0)?;
    let manifest = chain
        .manifest()
        .encode()
        .map_err(StorageErrorV2::Manifest)?;
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
    let root = checked_root_path(input)?;
    let parent = root.parent().ok_or(StorageErrorV2::InvalidPath)?;
    match fs::symlink_metadata(&root) {
        Ok(metadata) if !metadata.is_dir() => return Err(StorageErrorV2::NotDirectory.into()),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&root)?,
        Err(error) => return Err(error.into()),
    }
    let lock = DirectoryLockV2::acquire(&root, StoreAccessV2::Writer)?;
    let names = root_names(&root)?;
    if !names.is_empty() {
        return Err(uninitialized_error(&root, &names).into());
    }
    let mut publication_attempted = false;
    let result = (|| -> Result<(), AppendStorageErrorV2> {
        File::open(parent)?.sync_all()?;
        after(InitializeBoundary::ParentSynced)?;
        let blocks_root = root.join(BLOCKS_NAME);
        fs::create_dir(&blocks_root)?;
        super::write_new_synced(&root.join(MANIFEST_NAME), &manifest)?;
        after(InitializeBoundary::ManifestSynced)?;
        for (height, bytes) in blocks.iter().enumerate() {
            super::write_new_synced(&blocks_root.join(block_name(height as u64)), bytes)?;
            after(InitializeBoundary::BlockSynced(height as u64))?;
        }
        sync_directory(&blocks_root)?;
        after(InitializeBoundary::BlocksDirectorySynced)?;
        lock.sync()?;
        after(InitializeBoundary::RootPrepared)?;
        super::write_new_synced(&root.join(PENDING_NAME), &selection.encode())?;
        after(InitializeBoundary::SelectionStaged)?;
        publication_attempted = true;
        fs::rename(root.join(PENDING_NAME), root.join(SELECTION_NAME))?;
        after(InitializeBoundary::SelectionPublished)?;
        lock.sync()?;
        after(InitializeBoundary::RootSynced)?;
        Ok(())
    })();
    if let Err(error) = result {
        return Err(if publication_attempted {
            AppendStorageErrorV2::InitializationUncertain(Box::new(error))
        } else {
            AppendStorageErrorV2::InitializationIncomplete(Box::new(error))
        });
    }
    Ok(AppendStoreV2 {
        root,
        chain: chain.clone(),
        selection,
        access: StoreAccessV2::Writer,
        fenced: false,
        lock,
    })
}

fn open_inner(
    root: &Path,
    identity: ManifestIdentityV2,
    expected: AppendCheckpointV2,
    access: StoreAccessV2,
    mut after: impl FnMut(ReconcileBoundary) -> Result<(), AppendStorageErrorV2>,
) -> Result<AppendStoreV2, AppendStorageErrorV2> {
    require_platform()?;
    check_identity(identity, expected)?;
    let root = checked_root_path(root)?;
    let lock = DirectoryLockV2::acquire(&root, access)?;
    let selected = load_selected(&root, identity, expected)?;
    if access == StoreAccessV2::Writer {
        let result = (|| {
            after(ReconcileBoundary::Validated)?;
            sync_selected(&root, &lock, &selected, &mut after)
        })();
        result.map_err(|error| AppendStorageErrorV2::ReconciliationFailed(Box::new(error)))?;
    }
    Ok(AppendStoreV2 {
        root,
        chain: selected.chain,
        selection: selected.selection,
        access,
        fenced: false,
        lock,
    })
}

fn check_identity(
    identity: ManifestIdentityV2,
    expected: AppendCheckpointV2,
) -> Result<(), AppendStorageErrorV2> {
    if identity.config_hash != expected.chain.config_hash
        || identity.genesis_hash != expected.chain.genesis_hash
    {
        return Err(StorageErrorV2::IdentityMismatch.into());
    }
    Ok(())
}

struct LoadedSelection {
    chain: ChainV2,
    selection: AppendSelectionV2,
    lengths: Vec<u64>,
}

fn load_selected(
    root: &Path,
    identity: ManifestIdentityV2,
    expected: AppendCheckpointV2,
) -> Result<LoadedSelection, AppendStorageErrorV2> {
    check_identity(identity, expected)?;
    let names = root_names(root)?;
    if !names.iter().any(|name| name == SELECTION_NAME) {
        return Err(uninitialized_error(root, &names).into());
    }
    if names.len() != 3
        || ![SELECTION_NAME, MANIFEST_NAME, BLOCKS_NAME]
            .iter()
            .all(|required| names.iter().any(|name| name == required))
    {
        return Err(
            StorageErrorV2::InvalidLayout("unexpected or missing append-store root entry").into(),
        );
    }
    let record = read_regular_bounded(
        &root.join(SELECTION_NAME),
        APPEND_SELECTION_LEN_V2 as u64,
        Some(APPEND_SELECTION_LEN_V2 as u64),
    )?;
    let selection = AppendSelectionV2::decode_for(&record, expected)?;
    let snapshot = selection.snapshot();
    let manifest_bytes = read_regular_bounded(
        &root.join(MANIFEST_NAME),
        mirror_chain::v2::manifest::MAX_MANIFEST_BYTES_V2 as u64,
        Some(u64::from(snapshot.manifest_len())),
    )?;
    snapshot
        .verify_manifest_bytes(&manifest_bytes)
        .map_err(StorageErrorV2::Selection)?;
    let manifest = NetworkManifestV2::decode_for(&manifest_bytes, identity)
        .map_err(StorageErrorV2::Manifest)?;
    let blocks_root = root.join(BLOCKS_NAME);
    check_directory(&blocks_root)?;
    let lengths = preflight_blocks(&blocks_root, snapshot, &manifest)?;
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
    let chain = ChainV2::replay_encoded(&manifest, &borrowed, expected.chain)
        .map_err(StorageErrorV2::Replay)?;
    Ok(LoadedSelection {
        chain,
        selection,
        lengths,
    })
}

fn compare_selector(root: &Path, expected: &AppendSelectionV2) -> Result<(), AppendStorageErrorV2> {
    let bytes = read_regular_bounded(
        &root.join(SELECTION_NAME),
        APPEND_SELECTION_LEN_V2 as u64,
        Some(APPEND_SELECTION_LEN_V2 as u64),
    )?;
    if bytes.as_slice() != expected.encode() {
        return Err(AppendStorageErrorV2::SelectionChanged);
    }
    Ok(())
}

fn sync_selected(
    root: &Path,
    lock: &DirectoryLockV2,
    selected: &LoadedSelection,
    after: &mut impl FnMut(ReconcileBoundary) -> Result<(), AppendStorageErrorV2>,
) -> Result<(), AppendStorageErrorV2> {
    lock.verify_root(root)?;
    compare_selector(root, &selected.selection)?;
    sync_regular(
        &root.join(MANIFEST_NAME),
        u64::from(selected.selection.snapshot().manifest_len()),
    )?;
    after(ReconcileBoundary::ManifestSynced)?;
    for (height, length) in selected.lengths.iter().enumerate() {
        sync_regular(
            &root.join(BLOCKS_NAME).join(block_name(height as u64)),
            *length,
        )?;
        after(ReconcileBoundary::BlockSynced(height as u64))?;
    }
    sync_directory(&root.join(BLOCKS_NAME))?;
    after(ReconcileBoundary::BlocksDirectorySynced)?;
    sync_regular(&root.join(SELECTION_NAME), APPEND_SELECTION_LEN_V2 as u64)?;
    after(ReconcileBoundary::SelectionSynced)?;
    lock.sync()?;
    after(ReconcileBoundary::RootSynced)?;
    lock.verify_root(root)?;
    compare_selector(root, &selected.selection)?;
    after(ReconcileBoundary::SelectionRechecked)?;
    Ok(())
}

fn sync_regular(path: &Path, expected_len: u64) -> Result<(), AppendStorageErrorV2> {
    let before = regular_metadata(path)?;
    if before.len() != expected_len {
        return Err(StorageErrorV2::LengthMismatch(
            "validated file changed before synchronization",
        )
        .into());
    }
    let file = File::open(path)?;
    let opened = file.metadata()?;
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if !opened.is_file()
            || opened.nlink() != 1
            || before.dev() != opened.dev()
            || before.ino() != opened.ino()
        {
            return Err(StorageErrorV2::UnsafePath(
                "validated file replaced before synchronization",
            )
            .into());
        }
    }
    if opened.len() != expected_len {
        return Err(StorageErrorV2::LengthMismatch(
            "validated file changed while opening for synchronization",
        )
        .into());
    }
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), AppendStorageErrorV2> {
    check_directory(path)?;
    File::open(path)?.sync_all()?;
    Ok(())
}

#[derive(Debug)]
pub enum AppendStorageErrorV2 {
    ReadOnly,
    Fenced,
    GenerationOverflow,
    SelectionChanged,
    Storage(StorageErrorV2),
    Selection(AppendSelectionErrorV2),
    Chain(ChainErrorV2),
    InitializationIncomplete(Box<AppendStorageErrorV2>),
    InitializationUncertain(Box<AppendStorageErrorV2>),
    ReconciliationFailed(Box<AppendStorageErrorV2>),
    UpdateIncomplete {
        candidate: AppendCheckpointV2,
        source: Box<AppendStorageErrorV2>,
    },
    PublicationUncertain {
        candidate: AppendCheckpointV2,
        source: Box<AppendStorageErrorV2>,
    },
}

impl fmt::Display for AppendStorageErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fenced => write!(
                f,
                "V2 append handle fenced; independently selected reconciliation is required"
            ),
            Self::ReadOnly => write!(f, "V2 inspection handle cannot append or reconcile"),
            Self::UpdateIncomplete { source, .. } => write!(
                f,
                "V2 append incomplete; handle fenced and artifacts retained: {source}"
            ),
            Self::PublicationUncertain { source, .. } => write!(
                f,
                "V2 append publication uncertain; handle fenced and disk selection must be revalidated: {source}"
            ),
            Self::ReconciliationFailed(source) => write!(
                f,
                "V2 append reconciliation failed; no usable handle returned: {source}"
            ),
            other => write!(f, "V2 append storage: {other:?}"),
        }
    }
}

impl std::error::Error for AppendStorageErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Selection(error) => Some(error),
            Self::Chain(error) => Some(error),
            Self::InitializationIncomplete(error)
            | Self::InitializationUncertain(error)
            | Self::ReconciliationFailed(error) => Some(error),
            Self::UpdateIncomplete { source, .. } | Self::PublicationUncertain { source, .. } => {
                Some(source)
            }
            _ => None,
        }
    }
}

impl From<StorageErrorV2> for AppendStorageErrorV2 {
    fn from(error: StorageErrorV2) -> Self {
        Self::Storage(error)
    }
}

impl From<AppendSelectionErrorV2> for AppendStorageErrorV2 {
    fn from(error: AppendSelectionErrorV2) -> Self {
        Self::Selection(error)
    }
}

impl From<ChainErrorV2> for AppendStorageErrorV2 {
    fn from(error: ChainErrorV2) -> Self {
        Self::Chain(error)
    }
}

impl From<io::Error> for AppendStorageErrorV2 {
    fn from(error: io::Error) -> Self {
        Self::Storage(StorageErrorV2::Io(error))
    }
}

#[cfg(test)]
#[path = "append_tests.rs"]
mod tests;
