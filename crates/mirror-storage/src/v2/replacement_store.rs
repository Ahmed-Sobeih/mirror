//! Profile 3: complete accepted-branch publication under stable root ownership.
//!
//! Only a clean independently selected generation can be opened or reconciled.
//! Failed staging or retirement remains fenced and preserved; only the separate
//! explicit recovery API completes or retains authorized residual layouts. See
//! `docs/STORAGE_REPLACEMENT_STORE_V2.md` for the ordering and filesystem contract.

use std::{
    ffi::OsString,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use mirror_chain::v2::{
    ChainV2, MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2, ManifestIdentityV2, NetworkManifestV2,
    manifest::MAX_MANIFEST_BYTES_V2,
};

use super::{
    BLOCKS_NAME, MANIFEST_NAME, PENDING_NAME, SELECTION_NAME, StorageErrorV2, StoreAccessV2,
    block_name, check_directory, checked_root_path,
    lock::DirectoryLockV2,
    preflight_blocks, read_regular_bounded, regular_metadata,
    replacement_plan::{ReplacementPlanErrorV2, prepare_replacement_v2},
    replacement_selection::{
        REPLACEMENT_SELECTION_LEN_V2, ReplacementCheckpointV2, ReplacementSelectionErrorV2,
        ReplacementSelectionV2,
    },
    require_platform, uninitialized_error,
};

#[path = "replacement_recovery.rs"]
mod recovery;
pub use recovery::{
    ReplacementRecoveryActionV2, ReplacementRecoveryErrorV2, ReplacementRecoveryRequestV2,
};

/// Maximum encoded payload for one complete selected generation and its MSR2.
/// This excludes in-memory structures, allocation overhead and growth sentinels.
pub const MAX_REPLACEMENT_SELECTED_BYTES_V2: u64 =
    MAX_MANIFEST_BYTES_V2 as u64 + MAX_CHAIN_BYTES_V2 as u64 + REPLACEMENT_SELECTION_LEN_V2 as u64;

/// Maximum distinct encoded payload in a recognized two-generation operation.
/// Two complete histories/manifests and at most two MSR2 records can coexist.
pub const MAX_REPLACEMENT_INPUT_BYTES_V2: u64 = 2 * MAX_REPLACEMENT_SELECTED_BYTES_V2;

/// Conservative payload-read budget per call, including repeated verification
/// and a growth sentinel for every permitted read. This is not an RSS limit.
pub const MAX_REPLACEMENT_READ_BYTES_V2: u64 =
    8 * (MAX_REPLACEMENT_INPUT_BYTES_V2 + 2 * MAX_CHAIN_BLOCKS_V2 as u64 + 4);

/// An owned profile-3 selection, reconstructed from actual complete history.
///
/// Writers retain exclusive cooperative ownership of the same root inode for
/// their lifetime; inspectors retain shared ownership. No mutable chain, raw
/// descriptor or clone is exposed. A failed disk operation fences ordinary state
/// access. `last_confirmed_checkpoint` then describes memory only, never a claim
/// that disk still selects the old history.
#[derive(Debug)]
pub struct ReplacementStoreV2 {
    root: PathBuf,
    chain: ChainV2,
    selection: ReplacementSelectionV2,
    access: StoreAccessV2,
    fenced: bool,
    lock: DirectoryLockV2,
}

impl ReplacementStoreV2 {
    /// Initialize only an absent or empty final directory with an existing
    /// parent. Any accepted height starts at local generation zero. Existing
    /// legacy, immutable, append or partial stores are never adopted or removed.
    pub fn initialize(
        root: impl AsRef<Path>,
        chain: &ChainV2,
    ) -> Result<Self, ReplacementStorageErrorV2> {
        initialize_inner(root.as_ref(), chain, |_| Ok(()))
    }

    /// Replay the independently pinned clean selected generation, synchronize
    /// its validated contents and retain exclusive root ownership.
    pub fn open_writer(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: ReplacementCheckpointV2,
    ) -> Result<Self, ReplacementStorageErrorV2> {
        open_inner(
            root.as_ref(),
            identity,
            expected,
            StoreAccessV2::Writer,
            |_| Ok(()),
        )
    }

    /// Fully replay under shared ownership without application writes, repair,
    /// synchronization or promotion of pending data. OS access-time changes are
    /// outside the application's no-write guarantee.
    pub fn inspect(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: ReplacementCheckpointV2,
    ) -> Result<Self, ReplacementStorageErrorV2> {
        open_inner(
            root.as_ref(),
            identity,
            expected,
            StoreAccessV2::ReadOnly,
            |_| Ok(()),
        )
    }

    /// Publish one strictly preferred complete accepted branch. The network and
    /// current endpoint/generation must be independently retained by the caller.
    /// Success retires only the exact unselected old copy and returns the new
    /// checkpoint for independent retention. No automatic branch archive exists.
    /// Retain, or be able to independently reconstruct, BOTH exact accepted
    /// histories before calling: checkpoints alone cannot authorize later cleanup
    /// of a partially removed old copy. Use explicit recovery for authorized
    /// interrupted layouts; clean reconciliation does not repair that residue.
    pub fn replace(
        &mut self,
        identity: ManifestIdentityV2,
        expected_current: ReplacementCheckpointV2,
        candidate: &ChainV2,
    ) -> Result<ReplacementCheckpointV2, ReplacementStorageErrorV2> {
        self.replace_inner(identity, expected_current, candidate, |_| Ok(()))
    }

    /// Consume a writer and retain its SAME lock while replaying and syncing a
    /// clean independently pinned selection. Pending selectors, extra generations,
    /// partial cleanup and unknown artifacts reject unchanged. On error no writer
    /// is returned. This API does not resume or undo an interrupted replacement.
    pub fn reconcile(
        self,
        expected: ReplacementCheckpointV2,
    ) -> Result<Self, ReplacementStorageErrorV2> {
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

    /// Last successful memory publication, not a selected-disk assertion after
    /// an incomplete or uncertain replacement.
    pub const fn last_confirmed_checkpoint(&self) -> ReplacementCheckpointV2 {
        self.selection.checkpoint()
    }

    pub fn chain(&self) -> Result<&ChainV2, ReplacementStorageErrorV2> {
        self.require_unfenced()?;
        Ok(&self.chain)
    }

    pub fn selection(&self) -> Result<&ReplacementSelectionV2, ReplacementStorageErrorV2> {
        self.require_unfenced()?;
        Ok(&self.selection)
    }

    pub fn checkpoint(&self) -> Result<ReplacementCheckpointV2, ReplacementStorageErrorV2> {
        self.require_unfenced()?;
        Ok(self.selection.checkpoint())
    }

    fn require_writer(&self) -> Result<(), ReplacementStorageErrorV2> {
        if self.access == StoreAccessV2::Writer {
            Ok(())
        } else {
            Err(ReplacementStorageErrorV2::ReadOnly)
        }
    }

    fn require_unfenced(&self) -> Result<(), ReplacementStorageErrorV2> {
        if self.fenced {
            Err(ReplacementStorageErrorV2::Fenced)
        } else {
            Ok(())
        }
    }

    fn replace_inner(
        &mut self,
        identity: ManifestIdentityV2,
        expected_current: ReplacementCheckpointV2,
        candidate: &ChainV2,
        mut after: impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
    ) -> Result<ReplacementCheckpointV2, ReplacementStorageErrorV2> {
        self.require_writer()?;
        self.require_unfenced()?;

        // Candidate/pin rejections occur before any disk access or fencing. The
        // private accepted type retains the original complete ledger and limits.
        let next = *prepare_replacement_v2(
            identity,
            expected_current,
            &self.selection,
            &self.chain,
            candidate,
        )?
        .candidate_selection();
        let encoded_candidate = EncodedGeneration::from_chain(candidate)?;
        let encoded_current = EncodedGeneration::from_chain(&self.chain)?;
        let next_checkpoint = next.checkpoint();
        let old_generation = self.root.join(generation_name(self.selection.generation()));
        let next_generation = self.root.join(generation_name(next.generation()));
        let mut budget = ReadBudget::new();

        self.fenced = true;
        let mut publication_attempted = false;
        let result = (|| {
            self.lock.verify_root(&self.root)?;
            let disk = load_selected(&self.root, identity, expected_current, &mut budget)?;
            if disk.selection != self.selection || disk.chain != self.chain {
                return Err(ReplacementStorageErrorV2::CurrentChanged);
            }
            // A previously prepared plan is not a durable permit. Repeat actual
            // branch preference and exact metadata checks against reloaded disk.
            let checked = prepare_replacement_v2(
                identity,
                expected_current,
                &disk.selection,
                &disk.chain,
                candidate,
            )?;
            if *checked.candidate_selection() != next {
                return Err(ReplacementStorageErrorV2::CandidateChanged);
            }
            drop(checked);
            drop(disk);
            after(ReplacementBoundary::CurrentValidated)?;

            stage_generation(
                &self.root,
                &self.lock,
                next.generation(),
                &encoded_candidate,
                &mut after,
            )?;
            let staged = load_generation(&next_generation, identity, next, &mut budget)?;
            if staged.chain != *candidate {
                return Err(ReplacementStorageErrorV2::CandidateChanged);
            }
            drop(staged);
            after(ReplacementBoundary::CandidateValidated)?;
            stage_selector(&self.root, &next, &mut after)?;

            self.lock.verify_root(&self.root)?;
            check_operation_layout(
                &self.root,
                self.selection.generation(),
                next.generation(),
                true,
            )?;
            compare_selector(&self.root, &self.selection, &mut budget)?;
            compare_file(&self.root.join(PENDING_NAME), &next.encode(), &mut budget)?;
            // Check exact prepared bytes after the pending marker exists, too.
            compare_generation(
                &next_generation,
                candidate,
                &next,
                &encoded_candidate,
                &mut budget,
            )?;
            publication_attempted = true;
            after(ReplacementBoundary::PublicationAttempt)?;
            fs::rename(self.root.join(PENDING_NAME), self.root.join(SELECTION_NAME))?;
            after(ReplacementBoundary::SelectionPublished)?;
            self.lock.sync()?;
            after(ReplacementBoundary::PublishedRootSynced)?;

            // No old byte may be deleted until the selected new history is
            // complete, accepted and synchronized, and the entire old generation
            // has been preflighted and compared with the exact accepted source.
            self.lock.verify_root(&self.root)?;
            check_operation_layout(
                &self.root,
                self.selection.generation(),
                next.generation(),
                false,
            )?;
            compare_selector(&self.root, &next, &mut budget)?;
            let selected_candidate =
                load_generation(&next_generation, identity, next, &mut budget)?;
            if selected_candidate.chain != *candidate {
                return Err(ReplacementStorageErrorV2::CandidateChanged);
            }
            drop(selected_candidate);
            let old = load_generation(&old_generation, identity, self.selection, &mut budget)?;
            if old.chain != self.chain {
                return Err(ReplacementStorageErrorV2::CurrentChanged);
            }
            drop(old);
            compare_generation(
                &old_generation,
                &self.chain,
                &self.selection,
                &encoded_current,
                &mut budget,
            )?;
            after(ReplacementBoundary::OldGenerationValidated)?;
            retire_generation(&old_generation, &encoded_current, &mut budget, &mut after)?;
            self.lock.sync()?;
            after(ReplacementBoundary::CleanupRootSynced)?;

            let selected = load_selected(&self.root, identity, next_checkpoint, &mut budget)?;
            if selected.chain != *candidate || selected.selection != next {
                return Err(ReplacementStorageErrorV2::CandidateChanged);
            }
            after(ReplacementBoundary::SelectedValidated)?;
            sync_selected(&self.root, &self.lock, &selected, &mut budget, &mut after)?;
            Ok::<_, ReplacementStorageErrorV2>(selected)
        })();

        let selected = result.map_err(|source| {
            if publication_attempted {
                ReplacementStorageErrorV2::PublicationUncertain {
                    candidate: next_checkpoint,
                    source: Box::new(source),
                }
            } else {
                ReplacementStorageErrorV2::UpdateIncomplete {
                    candidate: next_checkpoint,
                    source: Box::new(source),
                }
            }
        })?;

        // All fallible work has completed while the same root remained owned.
        // Move the fully replayed candidate ledger and selector together.
        self.chain = selected.chain;
        self.selection = selected.selection;
        self.fenced = false;
        Ok(next_checkpoint)
    }

    fn reconcile_inner(
        mut self,
        expected: ReplacementCheckpointV2,
        mut after: impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
    ) -> Result<Self, ReplacementStorageErrorV2> {
        self.require_writer()?;
        self.fenced = true;
        let mut budget = ReadBudget::new();
        let result = (|| {
            let identity = self.chain.manifest().identity();
            check_identity(identity, expected)?;
            self.lock.verify_root(&self.root)?;
            let selected = load_selected(&self.root, identity, expected, &mut budget)?;
            after(ReplacementBoundary::SelectedValidated)?;
            sync_selected(&self.root, &self.lock, &selected, &mut budget, &mut after)?;
            Ok::<_, ReplacementStorageErrorV2>(selected)
        })();
        let selected = result
            .map_err(|source| ReplacementStorageErrorV2::ReconciliationFailed(Box::new(source)))?;
        self.chain = selected.chain;
        self.selection = selected.selection;
        self.fenced = false;
        Ok(self)
    }
}

/// Private deterministic test boundaries. They are not a production retry API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplacementBoundary {
    ParentSynced,
    CurrentValidated,
    BeforeGenerationCreate,
    GenerationCreated,
    BlocksDirectoryCreated,
    ManifestCreated,
    ManifestWritten,
    ManifestSynced,
    BlockCreated(u64),
    BlockWritten(u64),
    BlockSynced(u64),
    BlocksDirectorySynced,
    GenerationDirectorySynced,
    StagingRootSynced,
    CandidateValidated,
    SelectorCreated,
    SelectorWritten,
    SelectorSynced,
    PublicationAttempt,
    SelectionPublished,
    PublishedRootSynced,
    OldGenerationValidated,
    OldBlockRemoved(u64),
    OldBlocksDirectorySynced,
    OldBlocksDirectoryRemoved,
    OldGenerationAfterBlocksSynced,
    OldManifestRemoved,
    OldGenerationDirectorySynced,
    OldGenerationDirectoryRemoved,
    CleanupRootSynced,
    SelectedValidated,
    SelectedManifestSynced,
    SelectedBlockSynced(u64),
    SelectedBlocksDirectorySynced,
    SelectedGenerationDirectorySynced,
    SelectedSelectorSynced,
    SelectedRootSynced,
    SelectionRechecked,
}

fn initialize_inner(
    input: &Path,
    chain: &ChainV2,
    mut after: impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
) -> Result<ReplacementStoreV2, ReplacementStorageErrorV2> {
    require_platform()?;
    let selection = ReplacementSelectionV2::from_chain(chain, 0)?;
    let encoded = EncodedGeneration::from_chain(chain)?;
    let root = checked_root_path(input)?;
    let parent = root.parent().ok_or(StorageErrorV2::InvalidPath)?;
    match fs::symlink_metadata(&root) {
        Ok(metadata) if !metadata.is_dir() => return Err(StorageErrorV2::NotDirectory.into()),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&root)?,
        Err(error) => return Err(error.into()),
    }
    let lock = DirectoryLockV2::acquire(&root, StoreAccessV2::Writer)?;
    let names = bounded_names(&root, 4)?;
    if !names.is_empty() {
        return Err(uninitialized_error(&root, &names).into());
    }
    let mut budget = ReadBudget::new();
    let mut publication_attempted = false;
    let result = (|| {
        File::open(parent)?.sync_all()?;
        after(ReplacementBoundary::ParentSynced)?;
        stage_generation(&root, &lock, 0, &encoded, &mut after)?;
        let generation = root.join(generation_name(0));
        let staged = load_generation(
            &generation,
            chain.manifest().identity(),
            selection,
            &mut budget,
        )?;
        if staged.chain != *chain {
            return Err(ReplacementStorageErrorV2::CandidateChanged);
        }
        drop(staged);
        after(ReplacementBoundary::CandidateValidated)?;
        stage_selector(&root, &selection, &mut after)?;
        lock.verify_root(&root)?;
        require_names(
            &root,
            &[
                OsString::from(PENDING_NAME),
                OsString::from(generation_name(0)),
            ],
            4,
        )?;
        compare_file(&root.join(PENDING_NAME), &selection.encode(), &mut budget)?;
        compare_generation(&generation, chain, &selection, &encoded, &mut budget)?;
        publication_attempted = true;
        after(ReplacementBoundary::PublicationAttempt)?;
        fs::rename(root.join(PENDING_NAME), root.join(SELECTION_NAME))?;
        after(ReplacementBoundary::SelectionPublished)?;
        lock.sync()?;
        after(ReplacementBoundary::PublishedRootSynced)?;
        let selected = load_selected(
            &root,
            chain.manifest().identity(),
            selection.checkpoint(),
            &mut budget,
        )?;
        if selected.chain != *chain {
            return Err(ReplacementStorageErrorV2::CandidateChanged);
        }
        after(ReplacementBoundary::SelectedValidated)?;
        sync_selected(&root, &lock, &selected, &mut budget, &mut after)?;
        Ok::<_, ReplacementStorageErrorV2>(selected)
    })();
    let selected = result.map_err(|source| {
        if publication_attempted {
            ReplacementStorageErrorV2::InitializationUncertain(Box::new(source))
        } else {
            ReplacementStorageErrorV2::InitializationIncomplete(Box::new(source))
        }
    })?;
    Ok(ReplacementStoreV2 {
        root,
        chain: selected.chain,
        selection: selected.selection,
        access: StoreAccessV2::Writer,
        fenced: false,
        lock,
    })
}

fn open_inner(
    input: &Path,
    identity: ManifestIdentityV2,
    expected: ReplacementCheckpointV2,
    access: StoreAccessV2,
    mut after: impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
) -> Result<ReplacementStoreV2, ReplacementStorageErrorV2> {
    require_platform()?;
    check_identity(identity, expected)?;
    let root = checked_root_path(input)?;
    let lock = DirectoryLockV2::acquire(&root, access)?;
    let mut budget = ReadBudget::new();
    let selected = load_selected(&root, identity, expected, &mut budget)?;
    if access == StoreAccessV2::Writer {
        let result = (|| {
            after(ReplacementBoundary::SelectedValidated)?;
            sync_selected(&root, &lock, &selected, &mut budget, &mut after)
        })();
        result
            .map_err(|source| ReplacementStorageErrorV2::ReconciliationFailed(Box::new(source)))?;
    }
    Ok(ReplacementStoreV2 {
        root,
        chain: selected.chain,
        selection: selected.selection,
        access,
        fenced: false,
        lock,
    })
}

fn generation_name(generation: u64) -> String {
    format!("generation-{generation:020}")
}

fn check_identity(
    identity: ManifestIdentityV2,
    expected: ReplacementCheckpointV2,
) -> Result<(), ReplacementStorageErrorV2> {
    if identity.config_hash != expected.chain.config_hash
        || identity.genesis_hash != expected.chain.genesis_hash
    {
        return Err(StorageErrorV2::IdentityMismatch.into());
    }
    Ok(())
}

/// Enumeration is bounded independently of any payload allocation. The one
/// extra entry proves rejection; unknown files are never traversed or removed.
fn bounded_names(root: &Path, maximum: usize) -> Result<Vec<OsString>, ReplacementStorageErrorV2> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        if names.len() == maximum {
            return Err(StorageErrorV2::InvalidLayout("too many replacement-store entries").into());
        }
        names.push(entry?.file_name());
    }
    Ok(names)
}

fn require_names(
    root: &Path,
    expected: &[OsString],
    maximum: usize,
) -> Result<(), ReplacementStorageErrorV2> {
    let names = bounded_names(root, maximum)?;
    if names.len() != expected.len() || !expected.iter().all(|name| names.contains(name)) {
        return Err(
            StorageErrorV2::InvalidLayout("unexpected or missing replacement-store entry").into(),
        );
    }
    Ok(())
}

fn check_clean_layout(root: &Path, generation: u64) -> Result<(), ReplacementStorageErrorV2> {
    require_names(
        root,
        &[
            OsString::from(SELECTION_NAME),
            OsString::from(generation_name(generation)),
        ],
        4,
    )
}

fn check_operation_layout(
    root: &Path,
    old: u64,
    next: u64,
    pending: bool,
) -> Result<(), ReplacementStorageErrorV2> {
    let mut names = vec![
        OsString::from(SELECTION_NAME),
        OsString::from(generation_name(old)),
        OsString::from(generation_name(next)),
    ];
    if pending {
        names.push(OsString::from(PENDING_NAME));
    }
    require_names(root, &names, 4)
}

struct EncodedGeneration {
    manifest: Vec<u8>,
    blocks: Vec<Vec<u8>>,
}

impl EncodedGeneration {
    fn from_chain(chain: &ChainV2) -> Result<Self, ReplacementStorageErrorV2> {
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
        Ok(Self { manifest, blocks })
    }
}

struct LoadedSelection {
    chain: ChainV2,
    selection: ReplacementSelectionV2,
    lengths: Vec<u64>,
}

fn load_selected(
    root: &Path,
    identity: ManifestIdentityV2,
    expected: ReplacementCheckpointV2,
    budget: &mut ReadBudget,
) -> Result<LoadedSelection, ReplacementStorageErrorV2> {
    check_identity(identity, expected)?;
    let names = bounded_names(root, 4)?;
    if !names.iter().any(|name| name == SELECTION_NAME) {
        return Err(uninitialized_error(root, &names).into());
    }
    check_clean_layout(root, expected.generation)?;
    let record = budget.read_exact(
        &root.join(SELECTION_NAME),
        REPLACEMENT_SELECTION_LEN_V2 as u64,
    )?;
    let selection = ReplacementSelectionV2::decode_for(&record, expected)?;
    load_generation(
        &root.join(generation_name(expected.generation)),
        identity,
        selection,
        budget,
    )
}

fn load_generation(
    generation: &Path,
    identity: ManifestIdentityV2,
    selection: ReplacementSelectionV2,
    budget: &mut ReadBudget,
) -> Result<LoadedSelection, ReplacementStorageErrorV2> {
    check_identity(identity, selection.checkpoint())?;
    check_directory(generation)?;
    require_names(
        generation,
        &[OsString::from(MANIFEST_NAME), OsString::from(BLOCKS_NAME)],
        2,
    )?;
    let manifest_bytes = budget.read_exact(
        &generation.join(MANIFEST_NAME),
        u64::from(selection.snapshot().manifest_len()),
    )?;
    selection.verify_manifest_bytes(&manifest_bytes)?;
    let manifest = NetworkManifestV2::decode_for(&manifest_bytes, identity)
        .map_err(StorageErrorV2::Manifest)?;
    let blocks_root = generation.join(BLOCKS_NAME);
    check_directory(&blocks_root)?;
    let lengths = preflight_blocks(&blocks_root, selection.snapshot(), &manifest)?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(lengths.len())
        .map_err(|_| StorageErrorV2::AllocationFailed)?;
    for (height, length) in lengths.iter().enumerate() {
        encoded.push(budget.read_exact(&blocks_root.join(block_name(height as u64)), *length)?);
    }
    let borrowed: Vec<_> = encoded.iter().map(Vec::as_slice).collect();
    let chain = ChainV2::replay_encoded(&manifest, &borrowed, selection.checkpoint().chain)
        .map_err(StorageErrorV2::Replay)?;
    if ReplacementSelectionV2::from_chain(&chain, selection.generation())? != selection {
        return Err(ReplacementStorageErrorV2::SelectionChanged);
    }
    Ok(LoadedSelection {
        chain,
        selection,
        lengths,
    })
}

fn stage_generation(
    root: &Path,
    lock: &DirectoryLockV2,
    generation: u64,
    encoded: &EncodedGeneration,
    after: &mut impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
) -> Result<(), ReplacementStorageErrorV2> {
    lock.verify_root(root)?;
    after(ReplacementBoundary::BeforeGenerationCreate)?;
    let generation = root.join(generation_name(generation));
    fs::create_dir(&generation)?;
    after(ReplacementBoundary::GenerationCreated)?;
    let blocks = generation.join(BLOCKS_NAME);
    fs::create_dir(&blocks)?;
    after(ReplacementBoundary::BlocksDirectoryCreated)?;
    let mut manifest = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(generation.join(MANIFEST_NAME))?;
    after(ReplacementBoundary::ManifestCreated)?;
    manifest.write_all(&encoded.manifest)?;
    after(ReplacementBoundary::ManifestWritten)?;
    manifest.sync_all()?;
    after(ReplacementBoundary::ManifestSynced)?;
    drop(manifest);
    for (height, bytes) in encoded.blocks.iter().enumerate() {
        let height = height as u64;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(blocks.join(block_name(height)))?;
        after(ReplacementBoundary::BlockCreated(height))?;
        file.write_all(bytes)?;
        after(ReplacementBoundary::BlockWritten(height))?;
        file.sync_all()?;
        after(ReplacementBoundary::BlockSynced(height))?;
    }
    sync_directory(&blocks)?;
    after(ReplacementBoundary::BlocksDirectorySynced)?;
    sync_directory(&generation)?;
    after(ReplacementBoundary::GenerationDirectorySynced)?;
    lock.sync()?;
    after(ReplacementBoundary::StagingRootSynced)?;
    Ok(())
}

fn stage_selector(
    root: &Path,
    selection: &ReplacementSelectionV2,
    after: &mut impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
) -> Result<(), ReplacementStorageErrorV2> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(PENDING_NAME))?;
    after(ReplacementBoundary::SelectorCreated)?;
    file.write_all(&selection.encode())?;
    after(ReplacementBoundary::SelectorWritten)?;
    file.sync_all()?;
    after(ReplacementBoundary::SelectorSynced)?;
    Ok(())
}

fn compare_selector(
    root: &Path,
    selection: &ReplacementSelectionV2,
    budget: &mut ReadBudget,
) -> Result<(), ReplacementStorageErrorV2> {
    let bytes = budget.read_exact(
        &root.join(SELECTION_NAME),
        REPLACEMENT_SELECTION_LEN_V2 as u64,
    )?;
    if bytes != selection.encode() {
        return Err(ReplacementStorageErrorV2::SelectionChanged);
    }
    Ok(())
}

fn compare_file(
    path: &Path,
    expected: &[u8],
    budget: &mut ReadBudget,
) -> Result<(), ReplacementStorageErrorV2> {
    let bytes = budget.read_exact(path, expected.len() as u64)?;
    if bytes != expected {
        return Err(ReplacementStorageErrorV2::FileChanged);
    }
    Ok(())
}

fn compare_generation(
    generation: &Path,
    chain: &ChainV2,
    selection: &ReplacementSelectionV2,
    encoded: &EncodedGeneration,
    budget: &mut ReadBudget,
) -> Result<(), ReplacementStorageErrorV2> {
    check_directory(generation)?;
    require_names(
        generation,
        &[OsString::from(MANIFEST_NAME), OsString::from(BLOCKS_NAME)],
        2,
    )?;
    let blocks = generation.join(BLOCKS_NAME);
    check_directory(&blocks)?;
    // Enumerate/preflight the entire tree before comparing any payload. The
    // selected manifest and accepted chain supply limits, never an arbitrary file.
    preflight_blocks(&blocks, selection.snapshot(), chain.manifest())?;
    compare_file(&generation.join(MANIFEST_NAME), &encoded.manifest, budget)?;
    for (height, bytes) in encoded.blocks.iter().enumerate() {
        compare_file(&blocks.join(block_name(height as u64)), bytes, budget)?;
    }
    Ok(())
}

fn retire_generation(
    generation: &Path,
    encoded: &EncodedGeneration,
    budget: &mut ReadBudget,
    after: &mut impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
) -> Result<(), ReplacementStorageErrorV2> {
    // The caller already verified the whole old directory. Each named file is
    // checked again at removal; no wildcard or recursive deletion is permitted.
    let blocks = generation.join(BLOCKS_NAME);
    for (height, bytes) in encoded.blocks.iter().enumerate() {
        let height = height as u64;
        let path = blocks.join(block_name(height));
        compare_file(&path, bytes, budget)?;
        fs::remove_file(&path)?;
        after(ReplacementBoundary::OldBlockRemoved(height))?;
    }
    sync_directory(&blocks)?;
    after(ReplacementBoundary::OldBlocksDirectorySynced)?;
    check_directory(&blocks)?;
    fs::remove_dir(&blocks)?;
    after(ReplacementBoundary::OldBlocksDirectoryRemoved)?;
    sync_directory(generation)?;
    after(ReplacementBoundary::OldGenerationAfterBlocksSynced)?;
    let manifest = generation.join(MANIFEST_NAME);
    compare_file(&manifest, &encoded.manifest, budget)?;
    fs::remove_file(&manifest)?;
    after(ReplacementBoundary::OldManifestRemoved)?;
    sync_directory(generation)?;
    after(ReplacementBoundary::OldGenerationDirectorySynced)?;
    check_directory(generation)?;
    fs::remove_dir(generation)?;
    after(ReplacementBoundary::OldGenerationDirectoryRemoved)?;
    Ok(())
}

fn sync_selected(
    root: &Path,
    lock: &DirectoryLockV2,
    selected: &LoadedSelection,
    budget: &mut ReadBudget,
    after: &mut impl FnMut(ReplacementBoundary) -> Result<(), ReplacementStorageErrorV2>,
) -> Result<(), ReplacementStorageErrorV2> {
    lock.verify_root(root)?;
    check_clean_layout(root, selected.selection.generation())?;
    compare_selector(root, &selected.selection, budget)?;
    let generation = root.join(generation_name(selected.selection.generation()));
    check_directory(&generation)?;
    sync_regular(
        &generation.join(MANIFEST_NAME),
        u64::from(selected.selection.snapshot().manifest_len()),
    )?;
    after(ReplacementBoundary::SelectedManifestSynced)?;
    let blocks = generation.join(BLOCKS_NAME);
    check_directory(&blocks)?;
    for (height, length) in selected.lengths.iter().enumerate() {
        sync_regular(&blocks.join(block_name(height as u64)), *length)?;
        after(ReplacementBoundary::SelectedBlockSynced(height as u64))?;
    }
    sync_directory(&blocks)?;
    after(ReplacementBoundary::SelectedBlocksDirectorySynced)?;
    sync_directory(&generation)?;
    after(ReplacementBoundary::SelectedGenerationDirectorySynced)?;
    sync_regular(
        &root.join(SELECTION_NAME),
        REPLACEMENT_SELECTION_LEN_V2 as u64,
    )?;
    after(ReplacementBoundary::SelectedSelectorSynced)?;
    lock.sync()?;
    after(ReplacementBoundary::SelectedRootSynced)?;
    lock.verify_root(root)?;
    check_clean_layout(root, selected.selection.generation())?;
    compare_selector(root, &selected.selection, budget)?;
    let encoded = EncodedGeneration::from_chain(&selected.chain)?;
    compare_generation(
        &generation,
        &selected.chain,
        &selected.selection,
        &encoded,
        budget,
    )?;
    after(ReplacementBoundary::SelectionRechecked)?;
    Ok(())
}

fn sync_regular(path: &Path, expected_len: u64) -> Result<(), ReplacementStorageErrorV2> {
    let before = regular_metadata(path)?;
    if before.len() != expected_len {
        return Err(StorageErrorV2::LengthMismatch(
            "replacement file changed before synchronization",
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
                "replacement file changed during synchronization open",
            )
            .into());
        }
    }
    if opened.len() != expected_len {
        return Err(StorageErrorV2::LengthMismatch(
            "replacement file changed during synchronization open",
        )
        .into());
    }
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), ReplacementStorageErrorV2> {
    check_directory(path)?;
    File::open(path)?.sync_all()?;
    Ok(())
}

struct ReadBudget {
    used: u64,
}

impl ReadBudget {
    const fn new() -> Self {
        Self { used: 0 }
    }

    fn charge(&mut self, amount: u64) -> Result<(), ReplacementStorageErrorV2> {
        let attempted = self.used.checked_add(amount).unwrap_or(u64::MAX);
        if attempted > MAX_REPLACEMENT_READ_BYTES_V2 {
            return Err(ReplacementStorageErrorV2::ReadBudgetExceeded {
                limit: MAX_REPLACEMENT_READ_BYTES_V2,
                attempted,
            });
        }
        self.used = attempted;
        Ok(())
    }

    fn read_exact(&mut self, path: &Path, len: u64) -> Result<Vec<u8>, ReplacementStorageErrorV2> {
        // All callers obtain len from bounded fixed metadata, preflight or an
        // accepted encoding. This charge happens before even opening the payload.
        let charge = len
            .checked_add(1)
            .ok_or(ReplacementStorageErrorV2::ReadBudgetExceeded {
                limit: MAX_REPLACEMENT_READ_BYTES_V2,
                attempted: u64::MAX,
            })?;
        self.charge(charge)?;
        Ok(read_regular_bounded(path, len, Some(len))?)
    }
}

#[derive(Debug)]
pub enum ReplacementStorageErrorV2 {
    ReadOnly,
    Fenced,
    SelectionChanged,
    CurrentChanged,
    CandidateChanged,
    FileChanged,
    ReadBudgetExceeded {
        limit: u64,
        attempted: u64,
    },
    Storage(StorageErrorV2),
    Selection(ReplacementSelectionErrorV2),
    Plan(ReplacementPlanErrorV2),
    InitializationIncomplete(Box<ReplacementStorageErrorV2>),
    InitializationUncertain(Box<ReplacementStorageErrorV2>),
    ReconciliationFailed(Box<ReplacementStorageErrorV2>),
    UpdateIncomplete {
        candidate: ReplacementCheckpointV2,
        source: Box<ReplacementStorageErrorV2>,
    },
    PublicationUncertain {
        candidate: ReplacementCheckpointV2,
        source: Box<ReplacementStorageErrorV2>,
    },
}

impl fmt::Display for ReplacementStorageErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "V2 inspection handle cannot replace or reconcile"),
            Self::Fenced => write!(
                f,
                "V2 replacement handle fenced; independently pinned reconciliation required"
            ),
            Self::UpdateIncomplete { source, .. } => write!(
                f,
                "V2 replacement incomplete; handle fenced and artifacts preserved: {source}"
            ),
            Self::PublicationUncertain { source, .. } => write!(
                f,
                "V2 replacement completion uncertain; disk may select candidate while memory remains fenced: {source}"
            ),
            Self::ReconciliationFailed(source) => write!(
                f,
                "V2 replacement reconciliation failed; no writer returned: {source}"
            ),
            other => write!(f, "V2 replacement storage: {other:?}"),
        }
    }
}

impl std::error::Error for ReplacementStorageErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Selection(error) => Some(error),
            Self::Plan(error) => Some(error),
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

impl From<StorageErrorV2> for ReplacementStorageErrorV2 {
    fn from(error: StorageErrorV2) -> Self {
        Self::Storage(error)
    }
}

impl From<ReplacementSelectionErrorV2> for ReplacementStorageErrorV2 {
    fn from(error: ReplacementSelectionErrorV2) -> Self {
        Self::Selection(error)
    }
}

impl From<ReplacementPlanErrorV2> for ReplacementStorageErrorV2 {
    fn from(error: ReplacementPlanErrorV2) -> Self {
        Self::Plan(error)
    }
}

impl From<io::Error> for ReplacementStorageErrorV2 {
    fn from(error: io::Error) -> Self {
        Self::Storage(StorageErrorV2::Io(error))
    }
}

#[cfg(test)]
#[path = "replacement_store_tests.rs"]
mod tests;
