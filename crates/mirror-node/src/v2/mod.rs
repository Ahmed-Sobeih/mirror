//! Durable local restart authority for one finite V2 node operation.
//!
//! The operator supplies separate trusted control/store directories and an
//! independently selected network identity. A complete two-history intent is
//! validated and synchronized before storage changes. Startup is clean-only;
//! unresolved intent requires an explicit recovery action. This module provides
//! no daemon, automatic recovery, peer discovery, signer, wallet or new consensus.

#[cfg(test)]
mod control_failure_tests;
#[cfg(test)]
mod control_process_tests;
mod files;
pub mod records;

use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use mirror_chain::v2::{
    ChainV2, ForkChoiceErrorV2, ForkChoiceV2, ManifestIdentityV2, compare_branches_v2,
};
use mirror_network::v2::ReceivedCandidateV2;
use mirror_storage::v2::{
    ReplacementCheckpointV2, ReplacementRecoveryActionV2, ReplacementRecoveryErrorV2,
    ReplacementStorageErrorV2, ReplacementStoreV2,
};

use files::{ControlLock, ReadBudget};
use records::{
    CONTROL_RECORD_LEN_V2, ConfirmedSideV2, ControlRecordErrorV2, ControlRecordV2, IntentRecordV2,
    MAX_INTENT_BYTES_V2,
};

const CONFIRMED_NAME: &str = "confirmed.v2";
const CONFIRMED_PENDING_NAME: &str = "confirmed.pending";
const INTENT_NAME: &str = "intent.v2";
const INTENT_PENDING_NAME: &str = "intent.pending";

/// Conservative cumulative allowance for control payload reads in one call,
/// including length-plus-one sentinels and repeated exact-byte comparisons.
/// Storage enforces its own separate read allowance. This is not an RSS cap.
pub const MAX_CONTROL_READ_BYTES_V2: u64 =
    8 * (MAX_INTENT_BYTES_V2 as u64 + CONTROL_RECORD_LEN_V2 as u64 + 2);

/// Read-only observation. Pending fields are trusted local intent, not an
/// assertion that the store currently selects either endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeStatusV2 {
    Missing,
    Clean {
        checkpoint: ReplacementCheckpointV2,
    },
    RecoveryRequired {
        confirmed: ReplacementCheckpointV2,
        old: ReplacementCheckpointV2,
        candidate: ReplacementCheckpointV2,
    },
}

/// Ready owner. Store ownership is released before the control lock on drop.
/// No raw store writer can be extracted to bypass intent persistence.
pub struct NodeControlV2 {
    store: ReplacementStoreV2,
    lock: ControlLock,
    control_root: PathBuf,
    store_root: PathBuf,
    identity: ManifestIdentityV2,
    confirmed: ControlRecordV2,
}

impl NodeControlV2 {
    /// Explicitly initialize control for an ALREADY initialized profile-3 store.
    /// The caller independently retains the full expected checkpoint. This never
    /// initializes, adopts or repairs storage after a failed open. An interrupted
    /// control initialization is preserved and rejected if it leaves staging.
    pub fn initialize(
        control_root: impl AsRef<Path>,
        store_root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        expected: ReplacementCheckpointV2,
    ) -> Result<Self, NodeControlErrorV2> {
        initialize_inner(
            control_root.as_ref(),
            store_root.as_ref(),
            identity,
            expected,
            |_| Ok(()),
        )
    }

    /// Clean-only startup. A valid pending intent is an explicit recovery state,
    /// never permission to complete, retain or initialize anything automatically.
    pub fn open_writer(
        control_root: impl AsRef<Path>,
        store_root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
    ) -> Result<Self, NodeControlErrorV2> {
        let (control_root, store_root) =
            files::checked_pair(control_root.as_ref(), store_root.as_ref())?;
        if !files::directory_exists(&control_root)? {
            return Err(NodeControlErrorV2::MissingControl);
        }
        let lock = ControlLock::acquire(&control_root, true)?;
        let mut budget = ReadBudget::new();
        let loaded = load_control(&control_root, &lock, identity, &mut budget)?
            .ok_or(NodeControlErrorV2::MissingControl)?;
        if loaded.intent.is_some() {
            return Err(NodeControlErrorV2::RecoveryRequired);
        }
        let store =
            ReplacementStoreV2::open_writer(&store_root, identity, loaded.confirmed.checkpoint())?;
        verify_store_record(&store, &loaded.confirmed)?;
        // An earlier intent unlink can be visible despite failed directory sync.
        // Reestablish confirmed durability and clean absence before another write.
        files::sync_exact(
            &control_root.join(CONFIRMED_NAME),
            &loaded.confirmed.encode(),
            &mut budget,
        )?;
        lock.sync(&control_root)?;
        files::sync_parent(&control_root)?;
        Ok(Self {
            store,
            lock,
            control_root,
            store_root,
            identity,
            confirmed: loaded.confirmed,
        })
    }

    /// No application writes, creation, synchronization, peer contact or repair.
    /// An absent/empty control directory is Missing, regardless of store contents.
    pub fn status(
        control_root: impl AsRef<Path>,
        store_root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
    ) -> Result<NodeStatusV2, NodeControlErrorV2> {
        let (control_root, store_root) =
            files::checked_pair(control_root.as_ref(), store_root.as_ref())?;
        if !files::directory_exists(&control_root)? {
            return Ok(NodeStatusV2::Missing);
        }
        let lock = ControlLock::acquire(&control_root, false)?;
        let mut budget = ReadBudget::new();
        let Some(loaded) = load_control(&control_root, &lock, identity, &mut budget)? else {
            return Ok(NodeStatusV2::Missing);
        };
        if let Some(intent) = loaded.intent {
            return Ok(NodeStatusV2::RecoveryRequired {
                confirmed: loaded.confirmed.checkpoint(),
                old: intent.old(),
                candidate: intent.candidate(),
            });
        }
        let store =
            ReplacementStoreV2::inspect(&store_root, identity, loaded.confirmed.checkpoint())?;
        verify_store_record(&store, &loaded.confirmed)?;
        lock.verify(&control_root)?;
        Ok(NodeStatusV2::Clean {
            checkpoint: loaded.confirmed.checkpoint(),
        })
    }

    /// Explicit recovery of the one fully replayed local intent. Unknown or
    /// incomplete intent staging is preserved and rejected. The two accepted
    /// histories come from this trusted separate directory, never store residue.
    pub fn recover(
        control_root: impl AsRef<Path>,
        store_root: impl AsRef<Path>,
        identity: ManifestIdentityV2,
        action: ReplacementRecoveryActionV2,
    ) -> Result<Self, NodeControlErrorV2> {
        recover_inner(
            control_root.as_ref(),
            store_root.as_ref(),
            identity,
            action,
            |_| Ok(()),
        )
    }

    pub fn control_root(&self) -> &Path {
        &self.control_root
    }
    pub fn store_root(&self) -> &Path {
        &self.store_root
    }
    pub const fn identity(&self) -> ManifestIdentityV2 {
        self.identity
    }
    pub fn checkpoint(&self) -> ReplacementCheckpointV2 {
        self.confirmed.checkpoint()
    }
    pub fn chain(&self) -> &ChainV2 {
        // A fenced or partially confirmed store is never returned in this owner.
        self.store
            .chain()
            .expect("ready node owns an unfenced replacement store")
    }

    /// Consume one ready owner and a complete peer candidate. Keep-current
    /// decisions perform no filesystem call. Every failure consumes the owner;
    /// after durable intent exists, explicit recovery can reacquire it.
    ///
    /// ```compile_fail,E0382
    /// use mirror_node::v2::NodeControlV2;
    /// use mirror_network::v2::ReceivedCandidateV2;
    /// fn no_second_adoption(node: NodeControlV2, a: ReceivedCandidateV2, b: ReceivedCandidateV2) {
    ///     let _first = node.adopt(a);
    ///     let _second = node.adopt(b);
    /// }
    /// ```
    pub fn adopt(
        self,
        received: ReceivedCandidateV2,
    ) -> Result<NodeAdoptionOutcomeV2, NodeControlErrorV2> {
        self.adopt_inner(received, |_| Ok(()))
    }

    fn adopt_inner(
        mut self,
        received: ReceivedCandidateV2,
        mut after: impl FnMut(ControlBoundaryV2) -> Result<(), NodeControlErrorV2>,
    ) -> Result<NodeAdoptionOutcomeV2, NodeControlErrorV2> {
        let previous = self.confirmed.checkpoint();
        verify_store_record(&self.store, &self.confirmed)?;
        let comparison = compare_branches_v2(self.identity, self.store.chain()?, received.chain())?;
        if comparison.choice() == ForkChoiceV2::KeepCurrent {
            return Ok(NodeAdoptionOutcomeV2 {
                control: self,
                report: NodeAdoptionReportV2 {
                    choice: ForkChoiceV2::KeepCurrent,
                    previous,
                    selected: previous,
                },
            });
        }
        let intent = IntentRecordV2::new(
            self.identity,
            previous,
            self.store.chain()?.clone(),
            received.into_chain(),
        )?;
        if intent.old_record() != &self.confirmed {
            return Err(NodeControlErrorV2::ChangedControl);
        }
        let intent_bytes = intent.encode()?;
        let mut budget = ReadBudget::new();
        require_names(&self.control_root, &self.lock, &[CONFIRMED_NAME])?;
        files::verify_exact(
            &self.control_root.join(CONFIRMED_NAME),
            &self.confirmed.encode(),
            &mut budget,
        )?;
        write_new(
            &self.control_root.join(INTENT_PENDING_NAME),
            &intent_bytes,
            WriteKind::Intent,
            &mut after,
        )?;
        let readback = files::read(
            &self.control_root.join(INTENT_PENDING_NAME),
            MAX_INTENT_BYTES_V2 as u64,
            Some(intent_bytes.len() as u64),
            &mut budget,
        )?;
        if readback != intent_bytes {
            return Err(NodeControlErrorV2::ChangedControl);
        }
        let validated = IntentRecordV2::decode_for(&readback, self.identity)?;
        validated.validate_confirmed(&self.confirmed)?;
        if validated.old_chain() != intent.old_chain()
            || validated.candidate_chain() != intent.candidate_chain()
        {
            return Err(NodeControlErrorV2::ChangedControl);
        }
        drop(readback);
        drop(intent);
        after(ControlBoundaryV2::IntentValidated)?;
        require_names(
            &self.control_root,
            &self.lock,
            &[CONFIRMED_NAME, INTENT_PENDING_NAME],
        )?;
        fs::rename(
            self.control_root.join(INTENT_PENDING_NAME),
            self.control_root.join(INTENT_NAME),
        )?;
        after(ControlBoundaryV2::IntentPublished)?;
        self.lock.sync(&self.control_root)?;
        after(ControlBoundaryV2::IntentDirectorySynced)?;
        // Recheck the exact published authority immediately before storage changes.
        require_names(
            &self.control_root,
            &self.lock,
            &[CONFIRMED_NAME, INTENT_NAME],
        )?;
        files::verify_exact(
            &self.control_root.join(CONFIRMED_NAME),
            &self.confirmed.encode(),
            &mut budget,
        )?;
        files::verify_exact(
            &self.control_root.join(INTENT_NAME),
            &intent_bytes,
            &mut budget,
        )?;
        self.lock.verify(&self.control_root)?;
        let selected = self
            .store
            .replace(self.identity, previous, validated.candidate_chain())?;
        if selected != validated.candidate() {
            return Err(NodeControlErrorV2::ChangedControl);
        }
        after(ControlBoundaryV2::StorePublicationCompleted)?;
        finish_confirmation(
            &self.control_root,
            &self.lock,
            &self.confirmed,
            &validated,
            &intent_bytes,
            None,
            selected,
            &mut budget,
            &mut after,
        )?;
        self.confirmed = *validated.candidate_record();
        Ok(NodeAdoptionOutcomeV2 {
            control: self,
            report: NodeAdoptionReportV2 {
                choice: ForkChoiceV2::PreferCandidate,
                previous,
                selected,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeAdoptionReportV2 {
    pub choice: ForkChoiceV2,
    pub previous: ReplacementCheckpointV2,
    pub selected: ReplacementCheckpointV2,
}

pub struct NodeAdoptionOutcomeV2 {
    control: NodeControlV2,
    report: NodeAdoptionReportV2,
}

impl NodeAdoptionOutcomeV2 {
    pub const fn report(&self) -> NodeAdoptionReportV2 {
        self.report
    }
    pub const fn control(&self) -> &NodeControlV2 {
        &self.control
    }
    pub fn into_control(self) -> NodeControlV2 {
        self.control
    }
}

fn initialize_inner(
    control_input: &Path,
    store_input: &Path,
    identity: ManifestIdentityV2,
    expected: ReplacementCheckpointV2,
    mut after: impl FnMut(ControlBoundaryV2) -> Result<(), NodeControlErrorV2>,
) -> Result<NodeControlV2, NodeControlErrorV2> {
    let (control_root, store_root) = files::checked_pair(control_input, store_input)?;
    if !files::directory_exists(&control_root)? {
        files::create_directory(&control_root)?;
    }
    let lock = ControlLock::acquire(&control_root, true)?;
    require_names(&control_root, &lock, &[])?;
    let store = ReplacementStoreV2::open_writer(&store_root, identity, expected)?;
    let confirmed = ControlRecordV2::from_chain(identity, store.chain()?, expected.generation)?;
    if confirmed.checkpoint() != expected {
        return Err(NodeControlErrorV2::ChangedControl);
    }
    verify_store_record(&store, &confirmed)?;
    let bytes = confirmed.encode();
    write_new(
        &control_root.join(CONFIRMED_PENDING_NAME),
        &bytes,
        WriteKind::Initialize,
        &mut after,
    )?;
    let mut budget = ReadBudget::new();
    files::verify_exact(
        &control_root.join(CONFIRMED_PENDING_NAME),
        &bytes,
        &mut budget,
    )?;
    ControlRecordV2::decode_for(&bytes, identity)?;
    require_names(&control_root, &lock, &[CONFIRMED_PENDING_NAME])?;
    fs::rename(
        control_root.join(CONFIRMED_PENDING_NAME),
        control_root.join(CONFIRMED_NAME),
    )?;
    after(ControlBoundaryV2::InitializedPublished)?;
    lock.sync(&control_root)?;
    files::sync_parent(&control_root)?;
    after(ControlBoundaryV2::InitializedDirectorySynced)?;
    require_names(&control_root, &lock, &[CONFIRMED_NAME])?;
    files::verify_exact(&control_root.join(CONFIRMED_NAME), &bytes, &mut budget)?;
    Ok(NodeControlV2 {
        store,
        lock,
        control_root,
        store_root,
        identity,
        confirmed,
    })
}

struct LoadedControl {
    confirmed: ControlRecordV2,
    intent: Option<IntentRecordV2>,
    intent_bytes: Vec<u8>,
    pending_confirmation: Option<Vec<u8>>,
}

fn load_control(
    root: &Path,
    lock: &ControlLock,
    identity: ManifestIdentityV2,
    budget: &mut ReadBudget,
) -> Result<Option<LoadedControl>, NodeControlErrorV2> {
    lock.verify(root)?;
    let names = files::names(root)?;
    if names.is_empty() {
        return Ok(None);
    }
    if names
        .iter()
        .any(|name| name != CONFIRMED_NAME && name != INTENT_NAME && name != CONFIRMED_PENDING_NAME)
    {
        return Err(NodeControlErrorV2::InvalidLayout(
            "unknown entry or incomplete intent staging",
        ));
    }
    if !names.iter().any(|name| name == CONFIRMED_NAME) {
        return Err(NodeControlErrorV2::InvalidLayout(
            "missing confirmed control record or incomplete initialization",
        ));
    }
    let has_intent = names.iter().any(|name| name == INTENT_NAME);
    let has_pending = names.iter().any(|name| name == CONFIRMED_PENDING_NAME);
    if has_pending && !has_intent {
        return Err(NodeControlErrorV2::InvalidLayout(
            "confirmation staging without complete intent",
        ));
    }
    let confirmed_bytes = files::read(
        &root.join(CONFIRMED_NAME),
        CONTROL_RECORD_LEN_V2 as u64,
        Some(CONTROL_RECORD_LEN_V2 as u64),
        budget,
    )?;
    let confirmed = ControlRecordV2::decode_for(&confirmed_bytes, identity)?;
    let mut loaded = LoadedControl {
        confirmed,
        intent: None,
        intent_bytes: Vec::new(),
        pending_confirmation: None,
    };
    if has_intent {
        let bytes = files::read(
            &root.join(INTENT_NAME),
            MAX_INTENT_BYTES_V2 as u64,
            None,
            budget,
        )?;
        let intent = IntentRecordV2::decode_for(&bytes, identity)?;
        let side = intent.validate_confirmed(&confirmed)?;
        if has_pending {
            if side != ConfirmedSideV2::Old {
                return Err(NodeControlErrorV2::InvalidLayout(
                    "already confirmed candidate also has confirmation staging",
                ));
            }
            let pending = files::read(
                &root.join(CONFIRMED_PENDING_NAME),
                CONTROL_RECORD_LEN_V2 as u64,
                None,
                budget,
            )?;
            if !intent.candidate_record().encode().starts_with(&pending) {
                return Err(NodeControlErrorV2::InvalidLayout(
                    "confirmation staging is not an exact candidate prefix",
                ));
            }
            loaded.pending_confirmation = Some(pending);
        }
        loaded.intent = Some(intent);
        loaded.intent_bytes = bytes;
    }
    lock.verify(root)?;
    Ok(Some(loaded))
}

fn recover_inner(
    control_input: &Path,
    store_input: &Path,
    identity: ManifestIdentityV2,
    action: ReplacementRecoveryActionV2,
    mut after: impl FnMut(ControlBoundaryV2) -> Result<(), NodeControlErrorV2>,
) -> Result<NodeControlV2, NodeControlErrorV2> {
    let (control_root, store_root) = files::checked_pair(control_input, store_input)?;
    if !files::directory_exists(&control_root)? {
        return Err(NodeControlErrorV2::MissingControl);
    }
    let lock = ControlLock::acquire(&control_root, true)?;
    let mut budget = ReadBudget::new();
    let loaded = load_control(&control_root, &lock, identity, &mut budget)?
        .ok_or(NodeControlErrorV2::MissingControl)?;
    let intent = loaded.intent.ok_or(NodeControlErrorV2::NoPendingIntent)?;
    let side = intent.validate_confirmed(&loaded.confirmed)?;
    let candidate_was_confirming =
        side == ConfirmedSideV2::Candidate || loaded.pending_confirmation.is_some();
    if candidate_was_confirming && action != ReplacementRecoveryActionV2::Complete {
        return Err(NodeControlErrorV2::CannotRetainConfirmedCandidate);
    }
    // A visible rename from a failed directory sync is not durable authority.
    // Reestablish exact intent durability BEFORE any mutating storage recovery.
    files::sync_exact(
        &control_root.join(INTENT_NAME),
        &loaded.intent_bytes,
        &mut budget,
    )?;
    lock.sync(&control_root)?;
    files::sync_parent(&control_root)?;
    after(ControlBoundaryV2::IntentResynchronized)?;
    let expected_names: &[&str] = if loaded.pending_confirmation.is_some() {
        &[CONFIRMED_NAME, INTENT_NAME, CONFIRMED_PENDING_NAME]
    } else {
        &[CONFIRMED_NAME, INTENT_NAME]
    };
    require_names(&control_root, &lock, expected_names)?;
    files::verify_exact(
        &control_root.join(CONFIRMED_NAME),
        &loaded.confirmed.encode(),
        &mut budget,
    )?;
    files::verify_exact(
        &control_root.join(INTENT_NAME),
        &loaded.intent_bytes,
        &mut budget,
    )?;
    if let Some(prefix) = loaded.pending_confirmation.as_deref() {
        files::verify_exact(
            &control_root.join(CONFIRMED_PENDING_NAME),
            prefix,
            &mut budget,
        )?;
    }
    let store = if candidate_was_confirming {
        // Confirmation staging is created only after storage returned clean.
        // An old selection or storage residue here is inconsistent, not a new
        // authorization to change storage under a stale confirmed checkpoint.
        ReplacementStoreV2::open_writer(&store_root, identity, intent.candidate())?
    } else {
        ReplacementStoreV2::recover_interrupted(&store_root, identity, &intent.request()?, action)?
    };
    let selected = store.checkpoint()?;
    let record = if selected == intent.old() {
        if candidate_was_confirming {
            return Err(NodeControlErrorV2::ChangedControl);
        }
        *intent.old_record()
    } else if selected == intent.candidate() {
        *intent.candidate_record()
    } else {
        return Err(NodeControlErrorV2::ChangedControl);
    };
    verify_store_record(&store, &record)?;
    after(ControlBoundaryV2::StorePublicationCompleted)?;
    finish_confirmation(
        &control_root,
        &lock,
        &loaded.confirmed,
        &intent,
        &loaded.intent_bytes,
        loaded.pending_confirmation.as_deref(),
        selected,
        &mut budget,
        &mut after,
    )?;
    Ok(NodeControlV2 {
        store,
        lock,
        control_root,
        store_root,
        identity,
        confirmed: record,
    })
}

fn finish_confirmation(
    root: &Path,
    lock: &ControlLock,
    confirmed: &ControlRecordV2,
    intent: &IntentRecordV2,
    intent_bytes: &[u8],
    pending: Option<&[u8]>,
    selected: ReplacementCheckpointV2,
    budget: &mut ReadBudget,
    after: &mut impl FnMut(ControlBoundaryV2) -> Result<(), NodeControlErrorV2>,
) -> Result<(), NodeControlErrorV2> {
    let record = if selected == intent.candidate() {
        intent.candidate_record()
    } else if selected == intent.old() {
        intent.old_record()
    } else {
        return Err(NodeControlErrorV2::ChangedControl);
    };
    let expected_names: &[&str] = if pending.is_some() {
        &[CONFIRMED_NAME, INTENT_NAME, CONFIRMED_PENDING_NAME]
    } else {
        &[CONFIRMED_NAME, INTENT_NAME]
    };
    require_names(root, lock, expected_names)?;
    files::verify_exact(&root.join(INTENT_NAME), intent_bytes, budget)?;
    files::verify_exact(&root.join(CONFIRMED_NAME), &confirmed.encode(), budget)?;
    if record != confirmed {
        if selected != intent.candidate() {
            return Err(NodeControlErrorV2::ChangedControl);
        }
        let bytes = record.encode();
        if let Some(prefix) = pending {
            if !bytes.starts_with(prefix) {
                return Err(NodeControlErrorV2::ChangedControl);
            }
            let mut file = files::append_file(&root.join(CONFIRMED_PENDING_NAME), prefix, budget)?;
            let suffix = &bytes[prefix.len()..];
            let midpoint = suffix.len() / 2;
            file.write_all(&suffix[..midpoint])?;
            after(ControlBoundaryV2::ConfirmedPrefixWritten)?;
            file.write_all(&suffix[midpoint..])?;
            after(ControlBoundaryV2::ConfirmedWritten)?;
            file.sync_all()?;
            after(ControlBoundaryV2::ConfirmedFileSynced)?;
        } else {
            write_new(
                &root.join(CONFIRMED_PENDING_NAME),
                &bytes,
                WriteKind::Confirm,
                after,
            )?;
        }
        files::verify_exact(&root.join(CONFIRMED_PENDING_NAME), &bytes, budget)?;
        files::verify_exact(&root.join(CONFIRMED_NAME), &confirmed.encode(), budget)?;
        require_names(
            root,
            lock,
            &[CONFIRMED_NAME, INTENT_NAME, CONFIRMED_PENDING_NAME],
        )?;
        fs::rename(root.join(CONFIRMED_PENDING_NAME), root.join(CONFIRMED_NAME))?;
        after(ControlBoundaryV2::ConfirmedPublished)?;
    } else if pending.is_some() {
        return Err(NodeControlErrorV2::InvalidLayout(
            "unexpected confirmation staging for selected record",
        ));
    }
    // Also required when a previous rename succeeded but its directory sync did
    // not: visible correct bytes do not justify retiring the only durable intent.
    files::sync_exact(&root.join(CONFIRMED_NAME), &record.encode(), budget)?;
    lock.sync(root)?;
    after(ControlBoundaryV2::ConfirmedDirectorySynced)?;
    require_names(root, lock, &[CONFIRMED_NAME, INTENT_NAME])?;
    files::verify_exact(&root.join(CONFIRMED_NAME), &record.encode(), budget)?;
    files::verify_exact(&root.join(INTENT_NAME), intent_bytes, budget)?;
    fs::remove_file(root.join(INTENT_NAME))?;
    after(ControlBoundaryV2::IntentRemoved)?;
    lock.sync(root)?;
    after(ControlBoundaryV2::IntentRetirementSynced)?;
    require_names(root, lock, &[CONFIRMED_NAME])?;
    files::verify_exact(&root.join(CONFIRMED_NAME), &record.encode(), budget)?;
    Ok(())
}

fn verify_store_record(
    store: &ReplacementStoreV2,
    record: &ControlRecordV2,
) -> Result<(), NodeControlErrorV2> {
    if store.checkpoint()? != record.checkpoint() || store.selection()? != record.selection() {
        return Err(NodeControlErrorV2::ChangedControl);
    }
    Ok(())
}

fn require_names(
    root: &Path,
    lock: &ControlLock,
    expected: &[&str],
) -> Result<(), NodeControlErrorV2> {
    lock.verify(root)?;
    let actual = files::names(root)?;
    if actual.len() != expected.len()
        || actual
            .iter()
            .any(|name| !expected.iter().any(|want| name == want))
    {
        return Err(NodeControlErrorV2::InvalidLayout(
            "unexpected control-directory entries",
        ));
    }
    Ok(())
}

enum WriteKind {
    Initialize,
    Intent,
    Confirm,
}

fn write_new(
    path: &Path,
    bytes: &[u8],
    kind: WriteKind,
    after: &mut impl FnMut(ControlBoundaryV2) -> Result<(), NodeControlErrorV2>,
) -> Result<(), NodeControlErrorV2> {
    let boundaries = match kind {
        WriteKind::Initialize => [
            ControlBoundaryV2::InitializedCreated,
            ControlBoundaryV2::InitializedPrefixWritten,
            ControlBoundaryV2::InitializedWritten,
            ControlBoundaryV2::InitializedFileSynced,
        ],
        WriteKind::Intent => [
            ControlBoundaryV2::IntentCreated,
            ControlBoundaryV2::IntentPrefixWritten,
            ControlBoundaryV2::IntentWritten,
            ControlBoundaryV2::IntentFileSynced,
        ],
        WriteKind::Confirm => [
            ControlBoundaryV2::ConfirmedCreated,
            ControlBoundaryV2::ConfirmedPrefixWritten,
            ControlBoundaryV2::ConfirmedWritten,
            ControlBoundaryV2::ConfirmedFileSynced,
        ],
    };
    let mut file = files::create_file(path)?;
    after(boundaries[0])?;
    let midpoint = bytes.len() / 2;
    file.write_all(&bytes[..midpoint])?;
    after(boundaries[1])?;
    file.write_all(&bytes[midpoint..])?;
    after(boundaries[2])?;
    file.sync_all()?;
    after(boundaries[3])?;
    Ok(())
}

/// Private deterministic test hooks; production always supplies a no-op callback.
/// No environment variable or public API can enable an interruption hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlBoundaryV2 {
    InitializedCreated,
    InitializedPrefixWritten,
    InitializedWritten,
    InitializedFileSynced,
    InitializedPublished,
    InitializedDirectorySynced,
    IntentCreated,
    IntentPrefixWritten,
    IntentWritten,
    IntentFileSynced,
    IntentValidated,
    IntentPublished,
    IntentDirectorySynced,
    IntentResynchronized,
    StorePublicationCompleted,
    ConfirmedCreated,
    ConfirmedPrefixWritten,
    ConfirmedWritten,
    ConfirmedFileSynced,
    ConfirmedPublished,
    ConfirmedDirectorySynced,
    IntentRemoved,
    IntentRetirementSynced,
}

#[derive(Debug)]
pub enum NodeControlErrorV2 {
    UnsupportedPlatform,
    InvalidPath,
    UnsafePath(&'static str),
    MissingControl,
    Busy,
    InvalidLayout(&'static str),
    RecoveryRequired,
    NoPendingIntent,
    CannotRetainConfirmedCandidate,
    ChangedControl,
    LengthMismatch,
    ReadBudgetExceeded,
    AllocationFailed,
    Io(io::Error),
    Record(ControlRecordErrorV2),
    Storage(ReplacementStorageErrorV2),
    Recovery(ReplacementRecoveryErrorV2),
    ForkChoice(ForkChoiceErrorV2),
}

impl fmt::Display for NodeControlErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("node control supports the verified Linux locking profile only")
            }
            Self::InvalidPath => f.write_str("invalid control/store path"),
            Self::UnsafePath(reason) => write!(f, "unsafe control/store path: {reason}"),
            Self::MissingControl => f.write_str("node control is not initialized"),
            Self::Busy => f.write_str("node control is already owned"),
            Self::InvalidLayout(reason) => write!(f, "invalid control layout: {reason}"),
            Self::RecoveryRequired => f.write_str("node control requires explicit recovery"),
            Self::NoPendingIntent => f.write_str("no complete pending intent authorizes recovery"),
            Self::CannotRetainConfirmedCandidate => {
                f.write_str("candidate confirmation requires completion; rollback is forbidden")
            }
            Self::ChangedControl => {
                f.write_str("trusted control or selected-store bytes/checkpoint changed")
            }
            Self::LengthMismatch => {
                f.write_str("control input exceeds or differs from its bounded length")
            }
            Self::ReadBudgetExceeded => f.write_str("control payload-read allowance exhausted"),
            Self::AllocationFailed => f.write_str("bounded control allocation failed"),
            Self::Io(error) => write!(f, "control I/O: {error}"),
            Self::Record(error) => write!(f, "control record: {error}"),
            Self::Storage(error) => write!(f, "selected store: {error}"),
            Self::Recovery(error) => write!(f, "explicit storage recovery: {error}"),
            Self::ForkChoice(error) => write!(f, "accepted branch comparison: {error}"),
        }
    }
}

impl std::error::Error for NodeControlErrorV2 {}
impl From<io::Error> for NodeControlErrorV2 {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<ControlRecordErrorV2> for NodeControlErrorV2 {
    fn from(e: ControlRecordErrorV2) -> Self {
        Self::Record(e)
    }
}
impl From<ReplacementStorageErrorV2> for NodeControlErrorV2 {
    fn from(e: ReplacementStorageErrorV2) -> Self {
        Self::Storage(e)
    }
}
impl From<ReplacementRecoveryErrorV2> for NodeControlErrorV2 {
    fn from(e: ReplacementRecoveryErrorV2) -> Self {
        Self::Recovery(e)
    }
}
impl From<ForkChoiceErrorV2> for NodeControlErrorV2 {
    fn from(e: ForkChoiceErrorV2) -> Self {
        Self::ForkChoice(e)
    }
}
