//! Explicit recovery of one independently authorized profile-2 append.
//!
//! No new on-disk format or implicit repair is introduced. A caller retains both
//! checkpoints and the exact intended block outside the damaged store. That
//! request, not a checksum or prefix match, authorizes these specific paths.
//! Prefix equality constrains bytes; it cannot prove who created an empty or
//! common prefix. See `docs/STORAGE_RECOVERY_V2.md` for the finite state machine.

use std::{
    fmt,
    fs::{self, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use mirror_chain::v2::{
    ChainErrorV2, ChainV2, MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2, ManifestIdentityV2,
    NetworkManifestV2, manifest::MAX_MANIFEST_BYTES_V2,
};
use mirror_core::v2::BlockV2;

use super::{
    APPEND_SELECTION_LEN_V2, AppendCheckpointV2, AppendSelectionV2, AppendStorageErrorV2,
    AppendStoreV2, LoadedSelection, ReconcileBoundary, check_identity, load_selected,
    sync_directory, sync_regular, sync_selected,
};
use crate::v2::{
    BLOCKS_NAME, MANIFEST_NAME, MIN_BLOCK_BYTES, PENDING_NAME, SELECTION_NAME, StorageErrorV2,
    StoreAccessV2, block_name, check_directory, checked_root_path, lock::DirectoryLockV2,
    parse_block_name, read_regular_bounded, regular_metadata, require_platform, root_names,
};

/// Manifest + complete parent/candidate history + old and pending selectors.
/// This is an encoded-input bound, not resident memory or a total I/O bound.
pub const MAX_APPEND_RECOVERY_INPUT_BYTES_V2: usize =
    MAX_MANIFEST_BYTES_V2 + MAX_CHAIN_BYTES_V2 + 2 * APPEND_SELECTION_LEN_V2;

/// Conservative, enforced per-call allowance for payload reads, including all
/// comparisons, final replay and one-byte growth sentinels. Directory metadata,
/// bounded enumeration, writes and synchronization are accounted separately.
pub const MAX_APPEND_RECOVERY_READ_BYTES_V2: usize =
    8 * (MAX_APPEND_RECOVERY_INPUT_BYTES_V2 + MAX_CHAIN_BLOCKS_V2 + 4);

/// Explicit local intent, never inferred from the files being repaired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendRecoveryActionV2 {
    Complete,
    RetainParent,
}

/// TRUSTED CALLER SELECTION of one intended append. Construction checks the
/// relationship between pins and header, not block validity. Recovery still
/// replays the complete parent and accepts this exact child through ChainV2.
/// No serialization, filesystem-derived constructor or automatic intent exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppendRecoveryRequestV2 {
    parent: AppendCheckpointV2,
    candidate: AppendCheckpointV2,
    block: BlockV2,
}

impl AppendRecoveryRequestV2 {
    pub fn new(
        parent: AppendCheckpointV2,
        candidate: AppendCheckpointV2,
        block: BlockV2,
    ) -> Result<Self, AppendRecoveryErrorV2> {
        let height =
            parent
                .chain
                .height
                .checked_add(1)
                .ok_or(AppendRecoveryErrorV2::InvalidRequest(
                    "parent height overflow",
                ))?;
        let generation = parent
            .generation
            .checked_add(1)
            .ok_or(AppendRecoveryErrorV2::InvalidRequest("generation overflow"))?;
        if height >= MAX_CHAIN_BLOCKS_V2 as u64
            || parent.generation > parent.chain.height
            || candidate.chain.height != height
            || candidate.generation != generation
            || candidate.chain.config_hash != parent.chain.config_hash
            || candidate.chain.genesis_hash != parent.chain.genesis_hash
        {
            return Err(AppendRecoveryErrorV2::InvalidRequest(
                "checkpoints must describe exactly one bounded child append",
            ));
        }
        let header = block.header();
        if header.height() != height
            || header.config_hash() != candidate.chain.config_hash
            || header.previous_hash() != parent.chain.tip_hash
            || block.hash() != candidate.chain.tip_hash
            || header.ledger_root() != candidate.chain.ledger_root
            || block.encoded_len() > MAX_CHAIN_BYTES_V2
        {
            return Err(AppendRecoveryErrorV2::InvalidRequest(
                "exact candidate header does not match the independent checkpoints",
            ));
        }
        Ok(Self {
            parent,
            candidate,
            block,
        })
    }

    pub const fn parent(&self) -> AppendCheckpointV2 {
        self.parent
    }

    pub const fn candidate(&self) -> AppendCheckpointV2 {
        self.candidate
    }

    pub const fn block(&self) -> &BlockV2 {
        &self.block
    }
}

impl AppendStoreV2 {
    /// Explicitly recover the narrowly authorized append under a NEW exclusive
    /// lock. Ordinary open/inspect/reconcile remain strict and never call this.
    /// Success returns a synchronized, fully replayed writer. Failure returns no
    /// handle and performs no automatic rollback or deletion of unknown bytes.
    pub fn recover_interrupted(
        root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        request: &AppendRecoveryRequestV2,
        action: AppendRecoveryActionV2,
    ) -> Result<Self, AppendRecoveryErrorV2> {
        recover_open_inner(root.as_ref(), identity, request, action, |_| Ok(()))
    }

    /// Consume a writer without releasing/reacquiring its existing lock. It may
    /// be fenced after append failure. The retained memory checkpoint must match
    /// one of the independently supplied endpoints. No handle escapes on error.
    pub fn recover(
        self,
        request: &AppendRecoveryRequestV2,
        action: AppendRecoveryActionV2,
    ) -> Result<Self, AppendRecoveryErrorV2> {
        self.recover_inner(request, action, |_| Ok(()))
    }

    fn recover_inner(
        mut self,
        request: &AppendRecoveryRequestV2,
        action: AppendRecoveryActionV2,
        after: impl FnMut(RecoveryBoundary) -> Result<(), AppendRecoveryErrorV2>,
    ) -> Result<Self, AppendRecoveryErrorV2> {
        self.require_writer()?;
        self.fenced = true;
        let retained = self.last_confirmed_checkpoint();
        if retained != request.parent && retained != request.candidate {
            return Err(AppendRecoveryErrorV2::HandleCheckpointMismatch);
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
    request: &AppendRecoveryRequestV2,
    action: AppendRecoveryActionV2,
    after: impl FnMut(RecoveryBoundary) -> Result<(), AppendRecoveryErrorV2>,
) -> Result<AppendStoreV2, AppendRecoveryErrorV2> {
    require_platform()?;
    check_identity(identity, request.parent)?;
    check_identity(identity, request.candidate)?;
    let root = checked_root_path(input)?;
    let lock = DirectoryLockV2::acquire(&root, StoreAccessV2::Writer)?;
    let selected = recover_locked(&root, &lock, identity, request, action, after)?;
    Ok(AppendStoreV2 {
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
    Parent,
    Candidate,
}

struct PreparedRecovery {
    parent: LoadedSelection,
    candidate: ChainV2,
    next: AppendSelectionV2,
    selected: Selected,
    child_bytes: Vec<u8>,
    tail: Option<Vec<u8>>,
    pending: Option<Vec<u8>>,
}

impl PreparedRecovery {
    fn selected_record(&self) -> &AppendSelectionV2 {
        match self.selected {
            Selected::Parent => &self.parent.selection,
            Selected::Candidate => &self.next,
        }
    }
}

struct ReadBudget {
    remaining: usize,
}

impl ReadBudget {
    fn new() -> Self {
        Self {
            remaining: MAX_APPEND_RECOVERY_READ_BYTES_V2,
        }
    }

    fn charge(&mut self, bytes: usize) -> Result<(), AppendRecoveryErrorV2> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(AppendRecoveryErrorV2::ReadBudgetExceeded)?;
        Ok(())
    }

    fn read(
        &mut self,
        path: &Path,
        limit: u64,
        exact: Option<u64>,
    ) -> Result<Vec<u8>, AppendRecoveryErrorV2> {
        let charged = usize::try_from(limit)
            .ok()
            .and_then(|limit| limit.checked_add(1))
            .ok_or(AppendRecoveryErrorV2::ReadBudgetExceeded)?;
        self.charge(charged)?;
        Ok(read_regular_bounded(path, limit, exact)?)
    }
}

/// Read only known names, with all counts/sizes checked before block payloads.
fn prepare(
    root: &Path,
    identity: ManifestIdentityV2,
    request: &AppendRecoveryRequestV2,
    budget: &mut ReadBudget,
) -> Result<PreparedRecovery, AppendRecoveryErrorV2> {
    check_identity(identity, request.parent)?;
    check_identity(identity, request.candidate)?;
    let names = root_names(root)?;
    if !(names.len() == 3 || names.len() == 4)
        || ![MANIFEST_NAME, BLOCKS_NAME, SELECTION_NAME]
            .iter()
            .all(|required| names.iter().any(|name| name == required))
        || names.iter().any(|name| {
            name != MANIFEST_NAME
                && name != BLOCKS_NAME
                && name != SELECTION_NAME
                && name != PENDING_NAME
        })
    {
        return Err(AppendRecoveryErrorV2::InvalidResidualLayout(
            "recovery requires a selected append store and only its named pending marker",
        ));
    }
    let has_pending = names.iter().any(|name| name == PENDING_NAME);
    let selected_bytes = budget.read(
        &root.join(SELECTION_NAME),
        APPEND_SELECTION_LEN_V2 as u64,
        Some(APPEND_SELECTION_LEN_V2 as u64),
    )?;
    let (selected, record) = match AppendSelectionV2::decode_for(&selected_bytes, request.parent) {
        Ok(record) => (Selected::Parent, record),
        Err(_) => (
            Selected::Candidate,
            AppendSelectionV2::decode_for(&selected_bytes, request.candidate).map_err(|_| {
                AppendRecoveryErrorV2::UnrecognizedArtifact(
                    "selected marker matches neither independent endpoint",
                )
            })?,
        ),
    };
    let manifest_bytes = budget.read(
        &root.join(MANIFEST_NAME),
        MAX_MANIFEST_BYTES_V2 as u64,
        Some(u64::from(record.snapshot().manifest_len())),
    )?;
    record
        .verify_manifest_bytes(&manifest_bytes)
        .map_err(AppendStorageErrorV2::from)?;
    let manifest = NetworkManifestV2::decode_for(&manifest_bytes, identity)
        .map_err(StorageErrorV2::Manifest)?;
    let child_bytes = request
        .block
        .encode(manifest.config())
        .map_err(StorageErrorV2::Block)?;
    let parent_count = usize::try_from(request.parent.chain.height + 1)
        .map_err(|_| AppendRecoveryErrorV2::InvalidRequest("parent count overflow"))?;
    // The request constructor establishes parent_count + 1 <= MAX_CHAIN_BLOCKS.
    let blocks_root = root.join(BLOCKS_NAME);
    check_directory(&blocks_root)?;
    let mut lengths = vec![0u64; parent_count];
    let mut tail_len = None;
    let mut total = child_bytes.len() as u64;
    for (index, entry) in fs::read_dir(&blocks_root)?.enumerate() {
        if index >= parent_count + 1 {
            return Err(AppendRecoveryErrorV2::InvalidResidualLayout(
                "extra block entry",
            ));
        }
        let entry = entry?;
        let height = parse_block_name(&entry.file_name()).ok_or(
            AppendRecoveryErrorV2::InvalidResidualLayout("noncanonical block filename"),
        )?;
        let len = regular_metadata(&entry.path())?.len();
        if height == parent_count as u64 {
            limit(len, child_bytes.len() as u64)?;
            tail_len = Some(len);
        } else if height < parent_count as u64 {
            limit(len, u64::from(manifest.config().limits().max_block_bytes))?;
            if len < MIN_BLOCK_BYTES {
                return Err(
                    StorageErrorV2::LengthMismatch("truncated selected parent block").into(),
                );
            }
            lengths[height as usize] = len;
            total = total
                .checked_add(len)
                .ok_or(AppendRecoveryErrorV2::ReadBudgetExceeded)?;
            limit(total, MAX_CHAIN_BYTES_V2 as u64)?;
        } else {
            return Err(AppendRecoveryErrorV2::InvalidResidualLayout(
                "unrequested block height",
            ));
        }
    }
    if lengths.contains(&0) {
        return Err(AppendRecoveryErrorV2::InvalidResidualLayout(
            "missing parent block",
        ));
    }
    limit(total, MAX_CHAIN_BYTES_V2 as u64)?;
    let pending_len = if has_pending {
        let len = regular_metadata(&root.join(PENDING_NAME))?.len();
        limit(len, APPEND_SELECTION_LEN_V2 as u64)?;
        Some(len)
    } else {
        None
    };
    if has_pending && tail_len != Some(child_bytes.len() as u64) {
        return Err(AppendRecoveryErrorV2::InvalidResidualLayout(
            "pending selector requires the complete intended candidate file",
        ));
    }
    if selected == Selected::Candidate
        && (has_pending || tail_len != Some(child_bytes.len() as u64))
    {
        return Err(AppendRecoveryErrorV2::InvalidResidualLayout(
            "selected candidate must be complete with no pending marker",
        ));
    }
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(parent_count)
        .map_err(|_| StorageErrorV2::AllocationFailed)?;
    for (height, len) in lengths.iter().enumerate() {
        encoded.push(budget.read(
            &blocks_root.join(block_name(height as u64)),
            *len,
            Some(*len),
        )?);
    }
    let slices: Vec<_> = encoded.iter().map(Vec::as_slice).collect();
    let parent_chain = ChainV2::replay_encoded(&manifest, &slices, request.parent.chain)
        .map_err(StorageErrorV2::Replay)?;
    drop(encoded);
    let parent_record = AppendSelectionV2::from_chain(&parent_chain, request.parent.generation)
        .map_err(AppendStorageErrorV2::from)?;
    let mut candidate = parent_chain.clone();
    candidate.append(request.block.clone())?;
    let next = AppendSelectionV2::from_chain(&candidate, request.candidate.generation)
        .map_err(AppendStorageErrorV2::from)?;
    if next.checkpoint() != request.candidate {
        return Err(AppendRecoveryErrorV2::InvalidRequest(
            "executed candidate does not match pins",
        ));
    }
    let exact_record = match selected {
        Selected::Parent => &parent_record,
        Selected::Candidate => &next,
    };
    if selected_bytes.as_slice() != exact_record.encode() {
        return Err(AppendRecoveryErrorV2::UnrecognizedArtifact(
            "selected metadata differs from the fully executed history",
        ));
    }
    let tail = tail_len
        .map(|len| {
            budget.read(
                &blocks_root.join(block_name(parent_count as u64)),
                len,
                Some(len),
            )
        })
        .transpose()?;
    let pending = pending_len
        .map(|len| budget.read(&root.join(PENDING_NAME), len, Some(len)))
        .transpose()?;
    if tail
        .as_ref()
        .is_some_and(|bytes| !child_bytes.starts_with(bytes))
    {
        return Err(AppendRecoveryErrorV2::UnrecognizedArtifact(
            "candidate file is not an authorized prefix",
        ));
    }
    if pending
        .as_ref()
        .is_some_and(|bytes| !next.encode().starts_with(bytes))
    {
        return Err(AppendRecoveryErrorV2::UnrecognizedArtifact(
            "pending marker is not an authorized prefix",
        ));
    }
    Ok(PreparedRecovery {
        parent: LoadedSelection {
            chain: parent_chain,
            selection: parent_record,
            lengths,
        },
        candidate,
        next,
        selected,
        child_bytes,
        tail,
        pending,
    })
}

fn limit(actual: u64, allowed: u64) -> Result<(), AppendRecoveryErrorV2> {
    if actual > allowed {
        Err(StorageErrorV2::InputTooLarge {
            limit: allowed,
            actual,
        }
        .into())
    } else {
        Ok(())
    }
}

fn recover_locked(
    root: &Path,
    lock: &DirectoryLockV2,
    identity: ManifestIdentityV2,
    request: &AppendRecoveryRequestV2,
    action: AppendRecoveryActionV2,
    mut after: impl FnMut(RecoveryBoundary) -> Result<(), AppendRecoveryErrorV2>,
) -> Result<LoadedSelection, AppendRecoveryErrorV2> {
    lock.verify_root(root)?;
    let mut budget = ReadBudget::new();
    let prepared = prepare(root, identity, request, &mut budget)?;
    if prepared.selected == Selected::Candidate && action == AppendRecoveryActionV2::RetainParent {
        return Err(AppendRecoveryErrorV2::CandidateAlreadySelected);
    }
    let mut publication_attempted = prepared.selected == Selected::Candidate;
    let operation = (|| {
        after(RecoveryBoundary::Validated)?;
        lock.verify_root(root)?;
        compare_layout(root, &prepared)?;
        compare_marker(root, prepared.selected_record(), &mut budget)?;
        // Reestablish durability of the validated prefix before publishing or
        // removing anything. A previous failed sync is never assumed successful.
        sync_regular(
            &root.join(MANIFEST_NAME),
            u64::from(prepared.parent.selection.snapshot().manifest_len()),
        )?;
        after(RecoveryBoundary::ParentManifestSynced)?;
        for (height, len) in prepared.parent.lengths.iter().enumerate() {
            sync_regular(
                &root.join(BLOCKS_NAME).join(block_name(height as u64)),
                *len,
            )?;
            after(RecoveryBoundary::ParentBlockSynced(height as u64))?;
        }
        sync_directory(&root.join(BLOCKS_NAME))?;
        after(RecoveryBoundary::ParentBlocksDirectorySynced)?;
        sync_regular(&root.join(SELECTION_NAME), APPEND_SELECTION_LEN_V2 as u64)?;
        after(RecoveryBoundary::SelectedMarkerSynced)?;
        lock.sync()?;
        after(RecoveryBoundary::ParentRootSynced)?;
        lock.verify_root(root)?;
        compare_layout(root, &prepared)?;
        compare_marker(root, prepared.selected_record(), &mut budget)?;

        let tail_path = root
            .join(BLOCKS_NAME)
            .join(block_name(request.candidate.chain.height));
        let pending_path = root.join(PENDING_NAME);
        if prepared.selected == Selected::Parent {
            match action {
                AppendRecoveryActionV2::Complete => {
                    after(RecoveryBoundary::BeforeTailWrite)?;
                    lock.verify_root(root)?;
                    compare_marker(root, &prepared.parent.selection, &mut budget)?;
                    compare_layout(root, &prepared)?;
                    extend_prefix(
                        &tail_path,
                        prepared.tail.as_deref(),
                        &prepared.child_bytes,
                        &mut budget,
                        &mut after,
                        false,
                    )?;
                    sync_directory(&root.join(BLOCKS_NAME))?;
                    after(RecoveryBoundary::TailDirectorySynced)?;
                    after(RecoveryBoundary::BeforePendingWrite)?;
                    lock.verify_root(root)?;
                    compare_marker(root, &prepared.parent.selection, &mut budget)?;
                    extend_prefix(
                        &pending_path,
                        prepared.pending.as_deref(),
                        &prepared.next.encode(),
                        &mut budget,
                        &mut after,
                        true,
                    )?;
                    lock.verify_root(root)?;
                    compare_marker(root, &prepared.parent.selection, &mut budget)?;
                    // Compare complete artifacts again before selecting them.
                    compare_artifact(&tail_path, Some(&prepared.child_bytes), &mut budget)?;
                    compare_artifact(&pending_path, Some(&prepared.next.encode()), &mut budget)?;
                    publication_attempted = true;
                    after(RecoveryBoundary::PublicationAttempt)?;
                    lock.verify_root(root)?;
                    compare_marker(root, &prepared.parent.selection, &mut budget)?;
                    compare_artifact(&tail_path, Some(&prepared.child_bytes), &mut budget)?;
                    compare_artifact(&pending_path, Some(&prepared.next.encode()), &mut budget)?;
                    fs::rename(&pending_path, root.join(SELECTION_NAME))?;
                    after(RecoveryBoundary::SelectionPublished)?;
                    lock.sync()?;
                    after(RecoveryBoundary::RootSynced)?;
                }
                AppendRecoveryActionV2::RetainParent => {
                    // Pending removal must be durable BEFORE tail removal. This
                    // preserves a recognized old-selected shape after any retry.
                    if let Some(bytes) = prepared.pending.as_deref() {
                        after(RecoveryBoundary::BeforePendingRemoval)?;
                        lock.verify_root(root)?;
                        compare_marker(root, &prepared.parent.selection, &mut budget)?;
                        compare_artifact(&tail_path, prepared.tail.as_deref(), &mut budget)?;
                        compare_artifact(&pending_path, Some(bytes), &mut budget)?;
                        fs::remove_file(&pending_path)?;
                        after(RecoveryBoundary::PendingRemoved)?;
                        lock.sync()?;
                        after(RecoveryBoundary::PendingRemovalSynced)?;
                    }
                    if let Some(bytes) = prepared.tail.as_deref() {
                        after(RecoveryBoundary::BeforeTailRemoval)?;
                        lock.verify_root(root)?;
                        compare_marker(root, &prepared.parent.selection, &mut budget)?;
                        compare_artifact(&pending_path, None, &mut budget)?;
                        compare_artifact(&tail_path, Some(bytes), &mut budget)?;
                        fs::remove_file(&tail_path)?;
                        after(RecoveryBoundary::TailRemoved)?;
                        sync_directory(&root.join(BLOCKS_NAME))?;
                        after(RecoveryBoundary::TailRemovalSynced)?;
                    }
                }
            }
        }
        let (expected, expected_chain) = match action {
            AppendRecoveryActionV2::Complete => (request.candidate, &prepared.candidate),
            AppendRecoveryActionV2::RetainParent => (request.parent, &prepared.parent.chain),
        };
        lock.verify_root(root)?;
        // Charge the old strict loader's entire worst-case read phase before it
        // runs, including a sentinel for each bounded manifest/selector/block.
        budget.charge(
            MAX_MANIFEST_BYTES_V2
                + MAX_CHAIN_BYTES_V2
                + APPEND_SELECTION_LEN_V2
                + MAX_CHAIN_BLOCKS_V2
                + 2,
        )?;
        let loaded = load_selected(root, identity, expected)?;
        if &loaded.chain != expected_chain {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
        after(RecoveryBoundary::CleanValidated)?;
        // sync_selected reads the selected marker exactly twice, with sentinels.
        budget.charge(2 * (APPEND_SELECTION_LEN_V2 + 1))?;
        let mut callback_error = None;
        let sync_result = sync_selected(root, lock, &loaded, &mut |boundary| {
            after(RecoveryBoundary::CleanSync(boundary)).map_err(|error| {
                callback_error = Some(error);
                AppendStorageErrorV2::from(io::Error::other("recovery boundary interrupted"))
            })
        });
        if let Some(error) = callback_error {
            return Err(error);
        }
        sync_result?;
        Ok(loaded)
    })();
    operation.map_err(|source| {
        if publication_attempted {
            AppendRecoveryErrorV2::PublicationUncertain {
                source: Box::new(source),
            }
        } else {
            AppendRecoveryErrorV2::RecoveryIncomplete {
                source: Box::new(source),
            }
        }
    })
}

fn compare_marker(
    root: &Path,
    expected: &AppendSelectionV2,
    budget: &mut ReadBudget,
) -> Result<(), AppendRecoveryErrorV2> {
    compare_artifact(&root.join(SELECTION_NAME), Some(&expected.encode()), budget)
}

/// Recheck the bounded observed name/length shape before the first mutation.
/// This is defense in depth, not a hostile-owner traversal guarantee.
fn compare_layout(root: &Path, prepared: &PreparedRecovery) -> Result<(), AppendRecoveryErrorV2> {
    let names = root_names(root)?;
    if names.len() != 3 + usize::from(prepared.pending.is_some())
        || ![MANIFEST_NAME, BLOCKS_NAME, SELECTION_NAME]
            .iter()
            .all(|required| names.iter().any(|name| name == required))
        || names.iter().any(|name| {
            name != MANIFEST_NAME
                && name != BLOCKS_NAME
                && name != SELECTION_NAME
                && !(name == PENDING_NAME && prepared.pending.is_some())
        })
    {
        return Err(AppendRecoveryErrorV2::FilesChanged);
    }
    check_directory(&root.join(BLOCKS_NAME))?;
    let count = prepared.parent.lengths.len();
    let expected_count = count + usize::from(prepared.tail.is_some());
    let mut seen = 0;
    for (index, entry) in fs::read_dir(root.join(BLOCKS_NAME))?.enumerate() {
        if index >= expected_count {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
        let entry = entry?;
        let height =
            parse_block_name(&entry.file_name()).ok_or(AppendRecoveryErrorV2::FilesChanged)?;
        let expected_len = if height < count as u64 {
            prepared.parent.lengths[height as usize]
        } else if height == count as u64 {
            prepared
                .tail
                .as_ref()
                .ok_or(AppendRecoveryErrorV2::FilesChanged)?
                .len() as u64
        } else {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        };
        if regular_metadata(&entry.path())?.len() != expected_len {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
        seen += 1;
    }
    if seen != expected_count {
        return Err(AppendRecoveryErrorV2::FilesChanged);
    }
    if let Some(bytes) = prepared.pending.as_ref() {
        if regular_metadata(&root.join(PENDING_NAME))?.len() != bytes.len() as u64 {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
    }
    Ok(())
}

fn compare_artifact(
    path: &Path,
    expected: Option<&[u8]>,
    budget: &mut ReadBudget,
) -> Result<(), AppendRecoveryErrorV2> {
    match expected {
        Some(bytes) => {
            if budget.read(path, bytes.len() as u64, Some(bytes.len() as u64))? != bytes {
                return Err(AppendRecoveryErrorV2::FilesChanged);
            }
        }
        None => match fs::symlink_metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => return Err(AppendRecoveryErrorV2::FilesChanged),
        },
    }
    Ok(())
}

/// Complete only the missing suffix, with no truncation or overwrite. For an
/// existing prefix, check the actual read/append descriptor against its path,
/// length and exact observed bytes before writing through that same descriptor.
fn extend_prefix(
    path: &Path,
    observed: Option<&[u8]>,
    intended: &[u8],
    budget: &mut ReadBudget,
    after: &mut impl FnMut(RecoveryBoundary) -> Result<(), AppendRecoveryErrorV2>,
    marker: bool,
) -> Result<(), AppendRecoveryErrorV2> {
    let mut file = if let Some(prefix) = observed {
        let before = regular_metadata(path)?;
        if before.len() != prefix.len() as u64 {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
        let mut file = OpenOptions::new().read(true).append(true).open(path)?;
        same_file(&before, &file.metadata()?)?;
        budget.charge(prefix.len() + 1)?;
        let mut actual = Vec::new();
        actual
            .try_reserve_exact(prefix.len() + 1)
            .map_err(|_| StorageErrorV2::AllocationFailed)?;
        (&mut file)
            .take(prefix.len() as u64 + 1)
            .read_to_end(&mut actual)?;
        if actual != prefix || !intended.starts_with(&actual) {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
        same_file(&file.metadata()?, &regular_metadata(path)?)?;
        file
    } else {
        OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(path)?
    };
    after(if marker {
        RecoveryBoundary::PendingOpened
    } else {
        RecoveryBoundary::TailOpened
    })?;
    let prefix_len = observed.map_or(0, <[u8]>::len);
    let metadata = regular_metadata(path)?;
    same_file(&metadata, &file.metadata()?)?;
    if metadata.len() != prefix_len as u64 {
        return Err(AppendRecoveryErrorV2::FilesChanged);
    }
    file.write_all(&intended[prefix_len..])?;
    after(if marker {
        RecoveryBoundary::PendingWritten
    } else {
        RecoveryBoundary::TailWritten
    })?;
    compare_artifact(path, Some(intended), budget)?;
    same_file(&regular_metadata(path)?, &file.metadata()?)?;
    file.sync_all()?;
    after(if marker {
        RecoveryBoundary::PendingSynced
    } else {
        RecoveryBoundary::TailSynced
    })?;
    Ok(())
}

fn same_file(before: &Metadata, opened: &Metadata) -> Result<(), AppendRecoveryErrorV2> {
    if !before.is_file() || !opened.is_file() || before.len() != opened.len() {
        return Err(AppendRecoveryErrorV2::FilesChanged);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev()
            || before.ino() != opened.ino()
            || before.nlink() != 1
            || opened.nlink() != 1
        {
            return Err(AppendRecoveryErrorV2::FilesChanged);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoveryBoundary {
    Validated,
    ParentManifestSynced,
    ParentBlockSynced(u64),
    ParentBlocksDirectorySynced,
    SelectedMarkerSynced,
    ParentRootSynced,
    BeforeTailWrite,
    TailOpened,
    TailWritten,
    TailSynced,
    TailDirectorySynced,
    BeforePendingWrite,
    PendingOpened,
    PendingWritten,
    PendingSynced,
    PublicationAttempt,
    SelectionPublished,
    RootSynced,
    BeforePendingRemoval,
    PendingRemoved,
    PendingRemovalSynced,
    BeforeTailRemoval,
    TailRemoved,
    TailRemovalSynced,
    CleanValidated,
    CleanSync(ReconcileBoundary),
}

#[derive(Debug)]
pub enum AppendRecoveryErrorV2 {
    InvalidRequest(&'static str),
    UnrecognizedArtifact(&'static str),
    InvalidResidualLayout(&'static str),
    CandidateAlreadySelected,
    HandleCheckpointMismatch,
    FilesChanged,
    ReadBudgetExceeded,
    Storage(StorageErrorV2),
    Append(AppendStorageErrorV2),
    /// Cleanup or preparation may be incomplete. No usable handle is returned.
    RecoveryIncomplete {
        source: Box<AppendRecoveryErrorV2>,
    },
    /// The candidate may be selected already. Reuse independent request pins.
    PublicationUncertain {
        source: Box<AppendRecoveryErrorV2>,
    },
}

impl fmt::Display for AppendRecoveryErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecoveryIncomplete { source } => write!(
                f,
                "V2 explicit recovery incomplete; no handle returned: {source}"
            ),
            Self::PublicationUncertain { source } => write!(
                f,
                "V2 recovery publication uncertain; no handle returned: {source}"
            ),
            other => write!(f, "V2 explicit append recovery: {other:?}"),
        }
    }
}

impl std::error::Error for AppendRecoveryErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Append(error) => Some(error),
            Self::RecoveryIncomplete { source } | Self::PublicationUncertain { source } => {
                Some(source)
            }
            _ => None,
        }
    }
}

impl From<StorageErrorV2> for AppendRecoveryErrorV2 {
    fn from(error: StorageErrorV2) -> Self {
        Self::Storage(error)
    }
}

impl From<AppendStorageErrorV2> for AppendRecoveryErrorV2 {
    fn from(error: AppendStorageErrorV2) -> Self {
        Self::Append(error)
    }
}

impl From<io::Error> for AppendRecoveryErrorV2 {
    fn from(error: io::Error) -> Self {
        Self::Storage(StorageErrorV2::Io(error))
    }
}

impl From<ChainErrorV2> for AppendRecoveryErrorV2 {
    fn from(error: ChainErrorV2) -> Self {
        Self::Append(AppendStorageErrorV2::Chain(error))
    }
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
