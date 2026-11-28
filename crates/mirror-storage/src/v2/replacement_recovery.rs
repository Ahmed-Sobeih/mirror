//! Explicit recovery of one independently authorized profile-3 replacement.
//!
//! Both exact accepted histories remain caller-owned. Neither residual filenames,
//! checksums nor higher generations identify authority. Ordinary reopening stays
//! clean-only; this separate completion/retention API adds no persisted format.

use super::{
    BLOCKS_NAME, EncodedGeneration, LoadedSelection, MANIFEST_NAME, PENDING_NAME,
    REPLACEMENT_SELECTION_LEN_V2, ReadBudget, ReplacementBoundary, ReplacementCheckpointV2,
    ReplacementSelectionV2, ReplacementStorageErrorV2, ReplacementStoreV2, SELECTION_NAME,
    StorageErrorV2, StoreAccessV2, block_name, bounded_names, check_directory, check_identity,
    checked_root_path, compare_file, compare_generation, compare_selector, generation_name,
    load_generation, load_selected, prepare_replacement_v2, regular_metadata, require_names,
    require_platform, sync_directory, sync_regular, sync_selected,
};
use crate::v2::{lock::DirectoryLockV2, parse_block_name};
use mirror_chain::v2::{ChainV2, ManifestIdentityV2};
use std::{
    fmt,
    fs::{self, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

/// Explicit caller intent; never inferred from directory names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementRecoveryActionV2 {
    Complete,
    RetainOld,
}

/// Independently retained endpoints and BOTH complete accepted histories.
/// Immutable borrows preserve exact histories for checking staging and partial
/// old cleanup. Construction is pure; recovery checks again under its held lock
/// and a separately supplied network identity. No filesystem-derived constructor.
pub struct ReplacementRecoveryRequestV2<'a> {
    old: ReplacementCheckpointV2,
    candidate: ReplacementCheckpointV2,
    old_chain: &'a ChainV2,
    candidate_chain: &'a ChainV2,
}

impl<'a> ReplacementRecoveryRequestV2<'a> {
    pub fn new(
        old: ReplacementCheckpointV2,
        candidate: ReplacementCheckpointV2,
        old_chain: &'a ChainV2,
        candidate_chain: &'a ChainV2,
    ) -> Result<Self, ReplacementRecoveryErrorV2> {
        let request = Self {
            old,
            candidate,
            old_chain,
            candidate_chain,
        };
        request.selections(old_chain.manifest().identity())?;
        Ok(request)
    }
    pub const fn old(&self) -> ReplacementCheckpointV2 {
        self.old
    }
    pub const fn candidate(&self) -> ReplacementCheckpointV2 {
        self.candidate
    }
    pub const fn old_chain(&self) -> &'a ChainV2 {
        self.old_chain
    }
    pub const fn candidate_chain(&self) -> &'a ChainV2 {
        self.candidate_chain
    }

    fn selections(
        &self,
        identity: ManifestIdentityV2,
    ) -> Result<(ReplacementSelectionV2, ReplacementSelectionV2), ReplacementRecoveryErrorV2> {
        check_identity(identity, self.old)?;
        check_identity(identity, self.candidate)?;
        let old = ReplacementSelectionV2::from_chain(self.old_chain, self.old.generation)
            .map_err(ReplacementStorageErrorV2::from)?;
        let plan = prepare_replacement_v2(
            identity,
            self.old,
            &old,
            self.old_chain,
            self.candidate_chain,
        )
        .map_err(ReplacementStorageErrorV2::from)?;
        if plan.candidate_checkpoint() != self.candidate {
            return Err(ReplacementRecoveryErrorV2::InvalidRequest(
                "candidate pins must describe the exact preferred branch at the next generation",
            ));
        }
        Ok((old, *plan.candidate_selection()))
    }
}

impl fmt::Debug for ReplacementRecoveryRequestV2<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReplacementRecoveryRequestV2")
            .field("old", &self.old)
            .field("candidate", &self.candidate)
            .finish()
    }
}

impl ReplacementStoreV2 {
    /// Recover one authorized replacement under a new exclusive root lock.
    /// Success returns a fully replayed synchronized writer. Error returns no
    /// handle; unknown/corrupt artifacts are never implicitly repaired or deleted.
    pub fn recover_interrupted(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        request: &ReplacementRecoveryRequestV2<'_>,
        action: ReplacementRecoveryActionV2,
    ) -> Result<Self, ReplacementRecoveryErrorV2> {
        recover_open_inner(root.as_ref(), identity, request, action, |_| Ok(()))
    }
    /// Consume a writer (including a fenced one) while retaining its SAME lock.
    /// Retain both reference histories independently outside this handle.
    pub fn recover(
        self,
        request: &ReplacementRecoveryRequestV2<'_>,
        action: ReplacementRecoveryActionV2,
    ) -> Result<Self, ReplacementRecoveryErrorV2> {
        self.recover_inner(request, action, |_| Ok(()))
    }
    fn recover_inner(
        mut self,
        request: &ReplacementRecoveryRequestV2<'_>,
        action: ReplacementRecoveryActionV2,
        after: impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
    ) -> Result<Self, ReplacementRecoveryErrorV2> {
        self.require_writer()?;
        self.fenced = true;
        let retained = self.last_confirmed_checkpoint();
        if retained != request.old && retained != request.candidate {
            return Err(ReplacementRecoveryErrorV2::HandleCheckpointMismatch);
        }
        let selected = recover_locked(
            &self.root,
            &self.lock,
            self.chain.manifest().identity(),
            request,
            action,
            after,
        )?;
        self.chain = selected.chain;
        self.selection = selected.selection;
        self.fenced = false;
        Ok(self)
    }
}

fn recover_open_inner(
    input: &Path,
    identity: ManifestIdentityV2,
    request: &ReplacementRecoveryRequestV2<'_>,
    action: ReplacementRecoveryActionV2,
    after: impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
) -> Result<ReplacementStoreV2, ReplacementRecoveryErrorV2> {
    require_platform()?;
    request.selections(identity)?;
    let root = checked_root_path(input)?;
    let lock = DirectoryLockV2::acquire(&root, StoreAccessV2::Writer)?;
    let selected = recover_locked(&root, &lock, identity, request, action, after)?;
    Ok(ReplacementStoreV2 {
        root,
        chain: selected.chain,
        selection: selected.selection,
        access: StoreAccessV2::Writer,
        fenced: false,
        lock,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Selected {
    Old,
    Candidate,
}

/// Metadata bounded by accepted encodings; arbitrary surviving heights, never
/// an inferred ordinal write prefix. Missing and zero-length are distinct.
#[derive(Clone, Debug, PartialEq, Eq)]
struct GenerationShape {
    present: bool,
    manifest: Option<u64>,
    blocks: Option<Vec<Option<u64>>>,
}
impl GenerationShape {
    fn complete(&self, encoded: &EncodedGeneration) -> bool {
        self.present
            && self.manifest == Some(encoded.manifest.len() as u64)
            && self.blocks.as_ref().is_some_and(|lengths| {
                lengths.len() == encoded.blocks.len()
                    && lengths
                        .iter()
                        .zip(&encoded.blocks)
                        .all(|(len, bytes)| *len == Some(bytes.len() as u64))
            })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Layout {
    old: GenerationShape,
    candidate: GenerationShape,
    pending: Option<u64>,
}
struct PreparedRecovery {
    old: ReplacementSelectionV2,
    candidate: ReplacementSelectionV2,
    old_bytes: EncodedGeneration,
    candidate_bytes: EncodedGeneration,
    layout: Layout,
    selected: Selected,
}
impl PreparedRecovery {
    fn selected_record(&self) -> &ReplacementSelectionV2 {
        match self.selected {
            Selected::Old => &self.old,
            Selected::Candidate => &self.candidate,
        }
    }
    fn selected_bytes(&self) -> &EncodedGeneration {
        match self.selected {
            Selected::Old => &self.old_bytes,
            Selected::Candidate => &self.candidate_bytes,
        }
    }
}

fn allowed_length(
    actual: u64,
    maximum: u64,
    prefix: bool,
) -> Result<(), ReplacementRecoveryErrorV2> {
    if actual > maximum {
        return Err(StorageErrorV2::InputTooLarge {
            limit: maximum,
            actual,
        }
        .into());
    }
    if !prefix && actual != maximum {
        return Err(ReplacementRecoveryErrorV2::UnrecognizedArtifact(
            "partial old history file",
        ));
    }
    Ok(())
}

fn scan_generation(
    path: &Path,
    present: bool,
    encoded: &EncodedGeneration,
    prefix: bool,
) -> Result<GenerationShape, ReplacementRecoveryErrorV2> {
    if !present {
        return Ok(GenerationShape {
            present: false,
            manifest: None,
            blocks: None,
        });
    }
    check_directory(path)?;
    let names = bounded_names(path, 2)?;
    if names
        .iter()
        .any(|name| name != MANIFEST_NAME && name != BLOCKS_NAME)
    {
        return Err(ReplacementRecoveryErrorV2::InvalidResidualLayout(
            "unknown generation entry",
        ));
    }
    let manifest = if names.iter().any(|name| name == MANIFEST_NAME) {
        let len = regular_metadata(&path.join(MANIFEST_NAME))?.len();
        allowed_length(len, encoded.manifest.len() as u64, prefix)?;
        Some(len)
    } else {
        None
    };
    let blocks = if names.iter().any(|name| name == BLOCKS_NAME) {
        let block_root = path.join(BLOCKS_NAME);
        check_directory(&block_root)?;
        let names = bounded_names(&block_root, encoded.blocks.len())?;
        let mut lengths = vec![None; encoded.blocks.len()];
        for name in names {
            let height = parse_block_name(&name).ok_or(
                ReplacementRecoveryErrorV2::InvalidResidualLayout("noncanonical block name"),
            )?;
            let index = usize::try_from(height).map_err(|_| {
                ReplacementRecoveryErrorV2::InvalidResidualLayout("block height overflow")
            })?;
            let expected = encoded.blocks.get(index).ok_or(
                ReplacementRecoveryErrorV2::InvalidResidualLayout("unrequested block height"),
            )?;
            let len = regular_metadata(&block_root.join(name))?.len();
            allowed_length(len, expected.len() as u64, prefix)?;
            lengths[index] = Some(len);
        }
        Some(lengths)
    } else {
        None
    };
    if !prefix && blocks.is_some() && manifest.is_none() {
        return Err(ReplacementRecoveryErrorV2::InvalidResidualLayout(
            "old manifest cannot be removed before the blocks directory",
        ));
    }
    Ok(GenerationShape {
        present: true,
        manifest,
        blocks,
    })
}

fn scan_layout(
    root: &Path,
    prepared: &PreparedRecovery,
) -> Result<Layout, ReplacementRecoveryErrorV2> {
    let old_name = generation_name(prepared.old.generation());
    let candidate_name = generation_name(prepared.candidate.generation());
    let names = bounded_names(root, 4)?;
    if !names.iter().any(|name| name == SELECTION_NAME)
        || names.iter().any(|name| {
            name != SELECTION_NAME
                && name != PENDING_NAME
                && name != old_name.as_str()
                && name != candidate_name.as_str()
        })
    {
        return Err(ReplacementRecoveryErrorV2::InvalidResidualLayout(
            "unknown or missing root entry",
        ));
    }
    let old = scan_generation(
        &root.join(&old_name),
        names.iter().any(|name| name == old_name.as_str()),
        &prepared.old_bytes,
        false,
    )?;
    let candidate = scan_generation(
        &root.join(&candidate_name),
        names.iter().any(|name| name == candidate_name.as_str()),
        &prepared.candidate_bytes,
        true,
    )?;
    let pending = if names.iter().any(|name| name == PENDING_NAME) {
        let len = regular_metadata(&root.join(PENDING_NAME))?.len();
        allowed_length(len, REPLACEMENT_SELECTION_LEN_V2 as u64, true)?;
        Some(len)
    } else {
        None
    };
    if pending.is_some() && !candidate.complete(&prepared.candidate_bytes) {
        return Err(ReplacementRecoveryErrorV2::InvalidResidualLayout(
            "pending marker requires the complete candidate generation",
        ));
    }
    match prepared.selected {
        Selected::Old if !old.complete(&prepared.old_bytes) => {
            return Err(ReplacementRecoveryErrorV2::InvalidResidualLayout(
                "selected old history is incomplete",
            ));
        }
        Selected::Candidate
            if pending.is_some() || !candidate.complete(&prepared.candidate_bytes) =>
        {
            return Err(ReplacementRecoveryErrorV2::InvalidResidualLayout(
                "selected candidate must be complete without a pending marker",
            ));
        }
        _ => {}
    }
    Ok(Layout {
        old,
        candidate,
        pending,
    })
}

fn require_absent(path: &Path) -> Result<(), ReplacementRecoveryErrorV2> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
        Ok(_) => Err(ReplacementRecoveryErrorV2::FilesChanged),
    }
}

fn compare_prefix(
    path: &Path,
    observed: Option<u64>,
    expected: &[u8],
    budget: &mut ReadBudget,
) -> Result<(), ReplacementRecoveryErrorV2> {
    if let Some(len) = observed {
        allowed_length(len, expected.len() as u64, true)?;
        let actual = budget.read_exact(path, len)?;
        if !expected.starts_with(&actual) {
            return Err(ReplacementRecoveryErrorV2::UnrecognizedArtifact(
                "unauthorized file prefix",
            ));
        }
    } else {
        require_absent(path)?;
    }
    Ok(())
}

fn compare_residue(
    generation: &Path,
    shape: &GenerationShape,
    encoded: &EncodedGeneration,
    budget: &mut ReadBudget,
) -> Result<(), ReplacementRecoveryErrorV2> {
    if !shape.present {
        return require_absent(generation);
    }
    compare_prefix(
        &generation.join(MANIFEST_NAME),
        shape.manifest,
        &encoded.manifest,
        budget,
    )?;
    if let Some(lengths) = &shape.blocks {
        let blocks = generation.join(BLOCKS_NAME);
        for (height, (length, bytes)) in lengths.iter().zip(&encoded.blocks).enumerate() {
            compare_prefix(
                &blocks.join(block_name(height as u64)),
                *length,
                bytes,
                budget,
            )?;
        }
    } else {
        require_absent(&generation.join(BLOCKS_NAME))?;
    }
    Ok(())
}

fn compare_prepared(
    root: &Path,
    prepared: &PreparedRecovery,
    budget: &mut ReadBudget,
) -> Result<(), ReplacementRecoveryErrorV2> {
    if scan_layout(root, prepared)? != prepared.layout {
        return Err(ReplacementRecoveryErrorV2::FilesChanged);
    }
    compare_selector(root, prepared.selected_record(), budget)?;
    compare_residue(
        &root.join(generation_name(prepared.old.generation())),
        &prepared.layout.old,
        &prepared.old_bytes,
        budget,
    )?;
    compare_residue(
        &root.join(generation_name(prepared.candidate.generation())),
        &prepared.layout.candidate,
        &prepared.candidate_bytes,
        budget,
    )?;
    compare_prefix(
        &root.join(PENDING_NAME),
        prepared.layout.pending,
        &prepared.candidate.encode(),
        budget,
    )?;
    Ok(())
}

fn prepare(
    root: &Path,
    identity: ManifestIdentityV2,
    request: &ReplacementRecoveryRequestV2<'_>,
    budget: &mut ReadBudget,
) -> Result<PreparedRecovery, ReplacementRecoveryErrorV2> {
    let (old, candidate) = request.selections(identity)?;
    let old_bytes = EncodedGeneration::from_chain(request.old_chain)?;
    let candidate_bytes = EncodedGeneration::from_chain(request.candidate_chain)?;
    let selected_bytes = budget.read_exact(
        &root.join(SELECTION_NAME),
        REPLACEMENT_SELECTION_LEN_V2 as u64,
    )?;
    let selected = if selected_bytes == old.encode() {
        Selected::Old
    } else if selected_bytes == candidate.encode() {
        Selected::Candidate
    } else {
        return Err(ReplacementRecoveryErrorV2::UnrecognizedArtifact(
            "selected marker is not either exact authorized MSR2",
        ));
    };
    let empty = GenerationShape {
        present: false,
        manifest: None,
        blocks: None,
    };
    let mut prepared = PreparedRecovery {
        old,
        candidate,
        old_bytes,
        candidate_bytes,
        selected,
        layout: Layout {
            old: empty.clone(),
            candidate: empty,
            pending: None,
        },
    };
    prepared.layout = scan_layout(root, &prepared)?;
    let record = *prepared.selected_record();
    let loaded = load_generation(
        &root.join(generation_name(record.generation())),
        identity,
        record,
        budget,
    )?;
    let expected = match selected {
        Selected::Old => request.old_chain,
        Selected::Candidate => request.candidate_chain,
    };
    if &loaded.chain != expected {
        return Err(ReplacementRecoveryErrorV2::FilesChanged);
    }
    drop(loaded);
    let (other, shape, bytes) = match selected {
        Selected::Old => (
            candidate,
            &prepared.layout.candidate,
            &prepared.candidate_bytes,
        ),
        Selected::Candidate => (old, &prepared.layout.old, &prepared.old_bytes),
    };
    compare_residue(
        &root.join(generation_name(other.generation())),
        shape,
        bytes,
        budget,
    )?;
    compare_prefix(
        &root.join(PENDING_NAME),
        prepared.layout.pending,
        &candidate.encode(),
        budget,
    )?;
    Ok(prepared)
}

/// Reestablish durability of selected files, directories, marker and root before
/// any deletion. A selected-new layout may follow rename but precede root sync.
fn sync_selected_with_residue(
    root: &Path,
    lock: &DirectoryLockV2,
    prepared: &PreparedRecovery,
    after: &mut impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
) -> Result<(), ReplacementRecoveryErrorV2> {
    lock.verify_root(root)?;
    let record = prepared.selected_record();
    let encoded = prepared.selected_bytes();
    let generation = root.join(generation_name(record.generation()));
    sync_regular(
        &generation.join(MANIFEST_NAME),
        encoded.manifest.len() as u64,
    )?;
    after(RecoveryBoundary::SelectedSync(
        ReplacementBoundary::SelectedManifestSynced,
    ))?;
    let blocks = generation.join(BLOCKS_NAME);
    for (height, bytes) in encoded.blocks.iter().enumerate() {
        sync_regular(&blocks.join(block_name(height as u64)), bytes.len() as u64)?;
        after(RecoveryBoundary::SelectedSync(
            ReplacementBoundary::SelectedBlockSynced(height as u64),
        ))?;
    }
    sync_directory(&blocks)?;
    after(RecoveryBoundary::SelectedSync(
        ReplacementBoundary::SelectedBlocksDirectorySynced,
    ))?;
    sync_directory(&generation)?;
    after(RecoveryBoundary::SelectedSync(
        ReplacementBoundary::SelectedGenerationDirectorySynced,
    ))?;
    sync_regular(
        &root.join(SELECTION_NAME),
        REPLACEMENT_SELECTION_LEN_V2 as u64,
    )?;
    after(RecoveryBoundary::SelectedSync(
        ReplacementBoundary::SelectedSelectorSynced,
    ))?;
    lock.sync()?;
    after(RecoveryBoundary::SelectedSync(
        ReplacementBoundary::SelectedRootSynced,
    ))?;
    after(RecoveryBoundary::SelectedDurable)?;
    Ok(())
}

fn recover_locked(
    root: &Path,
    lock: &DirectoryLockV2,
    identity: ManifestIdentityV2,
    request: &ReplacementRecoveryRequestV2<'_>,
    action: ReplacementRecoveryActionV2,
    mut after: impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
) -> Result<LoadedSelection, ReplacementRecoveryErrorV2> {
    lock.verify_root(root)?;
    let mut budget = ReadBudget::new();
    let mut prepared = prepare(root, identity, request, &mut budget)?;
    if prepared.selected == Selected::Candidate && action == ReplacementRecoveryActionV2::RetainOld
    {
        return Err(ReplacementRecoveryErrorV2::CandidateAlreadySelected);
    }
    let mut publication_attempted = prepared.selected == Selected::Candidate;
    let operation = (|| {
        after(RecoveryBoundary::Validated)?;
        sync_selected_with_residue(root, lock, &prepared, &mut after)?;
        lock.verify_root(root)?;
        // Recheck all residue after validation/sync hooks, BEFORE first mutation.
        compare_prepared(root, &prepared, &mut budget)?;
        if prepared.selected == Selected::Old {
            match action {
                ReplacementRecoveryActionV2::Complete => {
                    stage_missing_generation(root, lock, &prepared, &mut budget, &mut after)?;
                    let next_root = root.join(generation_name(prepared.candidate.generation()));
                    let staged =
                        load_generation(&next_root, identity, prepared.candidate, &mut budget)?;
                    if staged.chain != *request.candidate_chain {
                        return Err(ReplacementRecoveryErrorV2::FilesChanged);
                    }
                    drop(staged);
                    after(RecoveryBoundary::CandidateValidated)?;
                    extend_prefix(
                        &root.join(PENDING_NAME),
                        prepared.layout.pending,
                        &prepared.candidate.encode(),
                        &mut budget,
                        &mut after,
                        FileKind::Pending,
                    )?;
                    publication_attempted = true;
                    after(RecoveryBoundary::PublicationAttempt)?;
                    lock.verify_root(root)?;
                    require_names(
                        root,
                        &[
                            SELECTION_NAME.into(),
                            PENDING_NAME.into(),
                            generation_name(prepared.old.generation()).into(),
                            generation_name(prepared.candidate.generation()).into(),
                        ],
                        4,
                    )?;
                    compare_selector(root, &prepared.old, &mut budget)?;
                    compare_file(
                        &root.join(PENDING_NAME),
                        &prepared.candidate.encode(),
                        &mut budget,
                    )?;
                    compare_generation(
                        &root.join(generation_name(prepared.old.generation())),
                        request.old_chain,
                        &prepared.old,
                        &prepared.old_bytes,
                        &mut budget,
                    )?;
                    compare_generation(
                        &next_root,
                        request.candidate_chain,
                        &prepared.candidate,
                        &prepared.candidate_bytes,
                        &mut budget,
                    )?;
                    fs::rename(root.join(PENDING_NAME), root.join(SELECTION_NAME))?;
                    after(RecoveryBoundary::SelectionPublished)?;
                    lock.sync()?;
                    after(RecoveryBoundary::PublishedRootSynced)?;
                    prepared.selected = Selected::Candidate;
                    prepared.layout = scan_layout(root, &prepared)?;
                    let selected =
                        load_generation(&next_root, identity, prepared.candidate, &mut budget)?;
                    if selected.chain != *request.candidate_chain {
                        return Err(ReplacementRecoveryErrorV2::FilesChanged);
                    }
                    drop(selected);
                    sync_selected_with_residue(root, lock, &prepared, &mut after)?;
                }
                ReplacementRecoveryActionV2::RetainOld => {
                    if prepared.layout.pending.is_some() {
                        after(RecoveryBoundary::BeforePendingRemoval)?;
                        lock.verify_root(root)?;
                        compare_prepared(root, &prepared, &mut budget)?;
                        fs::remove_file(root.join(PENDING_NAME))?;
                        after(RecoveryBoundary::PendingRemoved)?;
                        // Persist this absence BEFORE removing candidate bytes.
                        lock.sync()?;
                        after(RecoveryBoundary::PendingRemovalSynced)?;
                        prepared.layout.pending = None;
                    }
                    after(RecoveryBoundary::CleanupValidated)?;
                    lock.verify_root(root)?;
                    compare_prepared(root, &prepared, &mut budget)?;
                    retire_residue(
                        root,
                        lock,
                        prepared.candidate.generation(),
                        &prepared.layout.candidate,
                        &prepared.candidate_bytes,
                        &mut budget,
                        &mut after,
                    )?;
                }
            }
        }
        if action == ReplacementRecoveryActionV2::Complete {
            after(RecoveryBoundary::CleanupValidated)?;
            lock.verify_root(root)?;
            compare_prepared(root, &prepared, &mut budget)?;
            retire_residue(
                root,
                lock,
                prepared.old.generation(),
                &prepared.layout.old,
                &prepared.old_bytes,
                &mut budget,
                &mut after,
            )?;
        }
        let (checkpoint, expected_chain) = match action {
            ReplacementRecoveryActionV2::Complete => (request.candidate, request.candidate_chain),
            ReplacementRecoveryActionV2::RetainOld => (request.old, request.old_chain),
        };
        lock.verify_root(root)?;
        let selected = load_selected(root, identity, checkpoint, &mut budget)?;
        if &selected.chain != expected_chain {
            return Err(ReplacementRecoveryErrorV2::FilesChanged);
        }
        after(RecoveryBoundary::CleanValidated)?;
        let mut callback_error = None;
        let result = sync_selected(root, lock, &selected, &mut budget, &mut |boundary| {
            after(RecoveryBoundary::CleanSync(boundary)).map_err(|error| {
                callback_error = Some(error);
                ReplacementStorageErrorV2::from(io::Error::other(
                    "replacement recovery boundary interrupted",
                ))
            })
        });
        if let Some(error) = callback_error {
            return Err(error);
        }
        result?;
        Ok(selected)
    })();
    operation.map_err(|source| {
        if publication_attempted {
            ReplacementRecoveryErrorV2::PublicationUncertain {
                source: Box::new(source),
            }
        } else {
            ReplacementRecoveryErrorV2::RecoveryIncomplete {
                source: Box::new(source),
            }
        }
    })
}

fn stage_missing_generation(
    root: &Path,
    lock: &DirectoryLockV2,
    prepared: &PreparedRecovery,
    budget: &mut ReadBudget,
    after: &mut impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
) -> Result<(), ReplacementRecoveryErrorV2> {
    let shape = &prepared.layout.candidate;
    let encoded = &prepared.candidate_bytes;
    let generation = root.join(generation_name(prepared.candidate.generation()));
    lock.verify_root(root)?;
    if !shape.present {
        after(RecoveryBoundary::BeforeGenerationCreate)?;
        require_absent(&generation)?;
        fs::create_dir(&generation)?;
        after(RecoveryBoundary::GenerationCreated)?;
    }
    check_directory(&generation)?;
    let blocks = generation.join(BLOCKS_NAME);
    if shape.blocks.is_none() {
        require_absent(&blocks)?;
        fs::create_dir(&blocks)?;
        after(RecoveryBoundary::BlocksDirectoryCreated)?;
    }
    check_directory(&blocks)?;
    extend_prefix(
        &generation.join(MANIFEST_NAME),
        shape.manifest,
        &encoded.manifest,
        budget,
        after,
        FileKind::Manifest,
    )?;
    for (height, bytes) in encoded.blocks.iter().enumerate() {
        let observed = shape.blocks.as_ref().and_then(|lengths| lengths[height]);
        extend_prefix(
            &blocks.join(block_name(height as u64)),
            observed,
            bytes,
            budget,
            after,
            FileKind::Block(height as u64),
        )?;
    }
    sync_directory(&blocks)?;
    after(RecoveryBoundary::BlocksDirectorySynced)?;
    sync_directory(&generation)?;
    after(RecoveryBoundary::GenerationDirectorySynced)?;
    lock.sync()?;
    after(RecoveryBoundary::StagingRootSynced)?;
    Ok(())
}

#[derive(Clone, Copy)]
enum FileKind {
    Manifest,
    Block(u64),
    Pending,
}

/// Reuse one checked read/append descriptor. No truncation or overwrite occurs;
/// a failed write can leave only the authorized canonical prefix.
fn extend_prefix(
    path: &Path,
    observed: Option<u64>,
    intended: &[u8],
    budget: &mut ReadBudget,
    after: &mut impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
    kind: FileKind,
) -> Result<(), ReplacementRecoveryErrorV2> {
    let mut file = if let Some(len) = observed {
        allowed_length(len, intended.len() as u64, true)?;
        let charge = len
            .checked_add(1)
            .ok_or(ReplacementRecoveryErrorV2::FilesChanged)?;
        budget.charge(charge)?;
        let before = regular_metadata(path)?;
        if before.len() != len {
            return Err(ReplacementRecoveryErrorV2::FilesChanged);
        }
        let mut file = OpenOptions::new().read(true).append(true).open(path)?;
        same_file(&before, &file.metadata()?)?;
        let capacity = usize::try_from(charge).map_err(|_| StorageErrorV2::AllocationFailed)?;
        let mut actual = Vec::new();
        actual
            .try_reserve_exact(capacity)
            .map_err(|_| StorageErrorV2::AllocationFailed)?;
        (&mut file).take(charge).read_to_end(&mut actual)?;
        if actual.len() as u64 != len || !intended.starts_with(&actual) {
            return Err(ReplacementRecoveryErrorV2::FilesChanged);
        }
        same_file(&file.metadata()?, &regular_metadata(path)?)?;
        file
    } else {
        OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(path)?
    };
    after(match kind {
        FileKind::Manifest => RecoveryBoundary::ManifestOpened,
        FileKind::Block(h) => RecoveryBoundary::BlockOpened(h),
        FileKind::Pending => RecoveryBoundary::PendingOpened,
    })?;
    let length = observed.unwrap_or(0);
    let before = regular_metadata(path)?;
    same_file(&before, &file.metadata()?)?;
    if before.len() != length {
        return Err(ReplacementRecoveryErrorV2::FilesChanged);
    }
    file.write_all(&intended[length as usize..])?;
    after(match kind {
        FileKind::Manifest => RecoveryBoundary::ManifestWritten,
        FileKind::Block(h) => RecoveryBoundary::BlockWritten(h),
        FileKind::Pending => RecoveryBoundary::PendingWritten,
    })?;
    compare_file(path, intended, budget)?;
    same_file(&regular_metadata(path)?, &file.metadata()?)?;
    file.sync_all()?;
    after(match kind {
        FileKind::Manifest => RecoveryBoundary::ManifestSynced,
        FileKind::Block(h) => RecoveryBoundary::BlockSynced(h),
        FileKind::Pending => RecoveryBoundary::PendingSynced,
    })?;
    Ok(())
}

fn same_file(before: &Metadata, opened: &Metadata) -> Result<(), ReplacementRecoveryErrorV2> {
    if !before.is_file() || !opened.is_file() || before.len() != opened.len() {
        return Err(ReplacementRecoveryErrorV2::FilesChanged);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev()
            || before.ino() != opened.ino()
            || before.nlink() != 1
            || opened.nlink() != 1
        {
            return Err(ReplacementRecoveryErrorV2::FilesChanged);
        }
    }
    Ok(())
}

/// Delete exact named unselected artifacts only. Each unlink compares its bytes
/// again. Empty/missing prescribed directories are explicit, never recursive.
fn retire_residue(
    root: &Path,
    lock: &DirectoryLockV2,
    generation_number: u64,
    shape: &GenerationShape,
    encoded: &EncodedGeneration,
    budget: &mut ReadBudget,
    after: &mut impl FnMut(RecoveryBoundary) -> Result<(), ReplacementRecoveryErrorV2>,
) -> Result<(), ReplacementRecoveryErrorV2> {
    let generation = root.join(generation_name(generation_number));
    if shape.present {
        check_directory(&generation)?;
        if let Some(lengths) = &shape.blocks {
            let blocks = generation.join(BLOCKS_NAME);
            check_directory(&blocks)?;
            for (height, (length, bytes)) in lengths.iter().zip(&encoded.blocks).enumerate() {
                let path = blocks.join(block_name(height as u64));
                compare_prefix(&path, *length, bytes, budget)?;
                if length.is_some() {
                    lock.verify_root(root)?;
                    fs::remove_file(path)?;
                    after(RecoveryBoundary::RetiredBlock(height as u64))?;
                }
            }
            sync_directory(&blocks)?;
            after(RecoveryBoundary::RetiredBlocksDirectorySynced)?;
            require_names(&blocks, &[], 1)?;
            fs::remove_dir(&blocks)?;
            after(RecoveryBoundary::RetiredBlocksDirectoryRemoved)?;
        } else {
            require_absent(&generation.join(BLOCKS_NAME))?;
        }
        // Absence observed after a restart does not prove the earlier directory
        // removal was synchronized. Persist it even when this call did not
        // remove blocks/, before allowing manifest removal to become durable.
        sync_directory(&generation)?;
        after(RecoveryBoundary::RetiredGenerationAfterBlocksSynced)?;
        let manifest = generation.join(MANIFEST_NAME);
        compare_prefix(&manifest, shape.manifest, &encoded.manifest, budget)?;
        if shape.manifest.is_some() {
            fs::remove_file(&manifest)?;
            after(RecoveryBoundary::RetiredManifest)?;
        }
        sync_directory(&generation)?;
        after(RecoveryBoundary::RetiredGenerationSynced)?;
        require_names(&generation, &[], 1)?;
        fs::remove_dir(&generation)?;
        after(RecoveryBoundary::RetiredGenerationRemoved)?;
    } else {
        require_absent(&generation)?;
    }
    lock.verify_root(root)?;
    lock.sync()?;
    after(RecoveryBoundary::CleanupRootSynced)?;
    Ok(())
}

/// Private deterministic failure points; production supplies an infallible hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoveryBoundary {
    Validated,
    SelectedSync(ReplacementBoundary),
    SelectedDurable,
    BeforeGenerationCreate,
    GenerationCreated,
    BlocksDirectoryCreated,
    ManifestOpened,
    ManifestWritten,
    ManifestSynced,
    BlockOpened(u64),
    BlockWritten(u64),
    BlockSynced(u64),
    BlocksDirectorySynced,
    GenerationDirectorySynced,
    StagingRootSynced,
    CandidateValidated,
    PendingOpened,
    PendingWritten,
    PendingSynced,
    PublicationAttempt,
    SelectionPublished,
    PublishedRootSynced,
    CleanupValidated,
    BeforePendingRemoval,
    PendingRemoved,
    PendingRemovalSynced,
    RetiredBlock(u64),
    RetiredBlocksDirectorySynced,
    RetiredBlocksDirectoryRemoved,
    RetiredGenerationAfterBlocksSynced,
    RetiredManifest,
    RetiredGenerationSynced,
    RetiredGenerationRemoved,
    CleanupRootSynced,
    CleanValidated,
    CleanSync(ReplacementBoundary),
}

#[derive(Debug)]
pub enum ReplacementRecoveryErrorV2 {
    InvalidRequest(&'static str),
    InvalidResidualLayout(&'static str),
    UnrecognizedArtifact(&'static str),
    CandidateAlreadySelected,
    HandleCheckpointMismatch,
    FilesChanged,
    Storage(StorageErrorV2),
    Replacement(ReplacementStorageErrorV2),
    RecoveryIncomplete {
        source: Box<ReplacementRecoveryErrorV2>,
    },
    PublicationUncertain {
        source: Box<ReplacementRecoveryErrorV2>,
    },
}
impl fmt::Display for ReplacementRecoveryErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecoveryIncomplete { source } => write!(
                f,
                "V2 explicit replacement recovery incomplete; no writer returned: {source}"
            ),
            Self::PublicationUncertain { source } => write!(
                f,
                "V2 replacement recovery publication uncertain; no writer returned: {source}"
            ),
            other => write!(f, "V2 explicit replacement recovery: {other:?}"),
        }
    }
}
impl std::error::Error for ReplacementRecoveryErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Replacement(error) => Some(error),
            Self::RecoveryIncomplete { source } | Self::PublicationUncertain { source } => {
                Some(source)
            }
            _ => None,
        }
    }
}
impl From<StorageErrorV2> for ReplacementRecoveryErrorV2 {
    fn from(error: StorageErrorV2) -> Self {
        Self::Storage(error)
    }
}
impl From<ReplacementStorageErrorV2> for ReplacementRecoveryErrorV2 {
    fn from(error: ReplacementStorageErrorV2) -> Self {
        Self::Replacement(error)
    }
}
impl From<io::Error> for ReplacementRecoveryErrorV2 {
    fn from(error: io::Error) -> Self {
        Self::Storage(StorageErrorV2::Io(error))
    }
}

#[cfg(test)]
#[path = "replacement_recovery_tests.rs"]
mod tests;
