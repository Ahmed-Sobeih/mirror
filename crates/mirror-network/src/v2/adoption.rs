//! Explicit adoption of a completely received, actually executed V2 history.
//!
//! Reception does not select the canonical branch. Preparation checks independent
//! network/current pins against an owned store and compares complete accepted
//! histories. Only a preferred candidate reaches the existing replacement store.
//! Both exact histories and local checkpoints are retained before publication, so
//! a failed attempt can request explicit recovery without trusting peer claims or
//! residual filenames. No storage format or recovery rule is changed here.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use mirror_chain::v2::{
    BranchRelationV2, ChainReplayExpectationV2, ChainV2, ForkChoiceErrorV2, ForkChoiceV2,
    ManifestIdentityV2, compare_branches_v2,
};
use mirror_storage::v2::{
    ReplacementCheckpointV2, ReplacementPlanErrorV2, ReplacementRecoveryActionV2,
    ReplacementRecoveryErrorV2, ReplacementRecoveryRequestV2, ReplacementSelectionV2,
    ReplacementStorageErrorV2, ReplacementStoreV2, StoreAccessV2, prepare_replacement_v2,
};

use super::ReceivedCandidateV2;

/// A locally computed decision, not a reusable write permit or global finality.
/// Before `publish` succeeds, `selected` is the planned local checkpoint only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdoptionReportV2 {
    pub choice: ForkChoiceV2,
    pub relation: BranchRelationV2,
    pub common_ancestor: ChainReplayExpectationV2,
    pub previous: ReplacementCheckpointV2,
    pub candidate: ChainReplayExpectationV2,
    pub selected: ReplacementCheckpointV2,
}

/// Immutable caller intent retained independently of the store's mutable state.
///
/// The old accepted history is cloned once after strict preference is established;
/// the received candidate is moved here without cloning. These in-memory copies
/// are not a durable intent journal and are not included in encoded-input caps.
#[derive(Debug)]
pub struct AdoptionRecoveryInputsV2 {
    identity: ManifestIdentityV2,
    old: ReplacementCheckpointV2,
    candidate: ReplacementCheckpointV2,
    old_chain: ChainV2,
    candidate_chain: ChainV2,
}

impl AdoptionRecoveryInputsV2 {
    pub const fn identity(&self) -> ManifestIdentityV2 {
        self.identity
    }
    pub const fn old(&self) -> ReplacementCheckpointV2 {
        self.old
    }
    pub const fn candidate(&self) -> ReplacementCheckpointV2 {
        self.candidate
    }
    pub const fn old_chain(&self) -> &ChainV2 {
        &self.old_chain
    }
    pub const fn candidate_chain(&self) -> &ChainV2 {
        &self.candidate_chain
    }

    /// Borrow the exact authorization for the existing explicit recovery API.
    /// It still revalidates every retained input and all disk residue under lock.
    pub fn request(&self) -> Result<ReplacementRecoveryRequestV2<'_>, ReplacementRecoveryErrorV2> {
        ReplacementRecoveryRequestV2::new(
            self.old,
            self.candidate,
            &self.old_chain,
            &self.candidate_chain,
        )
    }
}

enum PreparedHistory {
    Keep,
    Replace(AdoptionRecoveryInputsV2),
}

/// One prepared decision retaining exclusive store ownership until consumed.
///
/// No disk I/O occurs during preparation. Preferred publication repeats the
/// store's actual disk/identity/preference checks under the same lock. A losing
/// or tied candidate is discarded and publishing returns the unchanged writer.
///
/// ```compile_fail,E0382
/// use mirror_network::v2::adoption::AdoptionAttemptV2;
/// fn cannot_publish_twice(attempt: AdoptionAttemptV2) {
///     let _first = attempt.publish();
///     let _second = attempt.publish();
/// }
/// ```
pub struct AdoptionAttemptV2 {
    store: ReplacementStoreV2,
    report: AdoptionReportV2,
    history: PreparedHistory,
}

impl AdoptionAttemptV2 {
    /// `identity` and `expected_current` must be independently retained locally;
    /// never obtain them from peer summaries or untrusted selected-file bytes.
    /// Any rejection preserves both owned inputs and does not touch the store.
    pub fn prepare(
        store: ReplacementStoreV2,
        identity: ManifestIdentityV2,
        expected_current: ReplacementCheckpointV2,
        received: ReceivedCandidateV2,
    ) -> Result<Self, AdoptionPreparationFailureV2> {
        let checked = prepare(&store, identity, expected_current, &received);
        match checked {
            Err(error) => Err(AdoptionPreparationFailureV2 {
                error,
                store,
                received,
            }),
            Ok(report) => {
                let history = if report.choice == ForkChoiceV2::PreferCandidate {
                    // Checked above and immutable: no intervening store operation.
                    let old_chain = match store.chain() {
                        Ok(chain) => chain.clone(),
                        Err(error) => {
                            return Err(AdoptionPreparationFailureV2 {
                                error: AdoptionErrorV2::Storage(error),
                                store,
                                received,
                            });
                        }
                    };
                    PreparedHistory::Replace(AdoptionRecoveryInputsV2 {
                        identity,
                        old: expected_current,
                        candidate: report.selected,
                        old_chain,
                        candidate_chain: received.into_chain(),
                    })
                } else {
                    PreparedHistory::Keep
                };
                Ok(Self {
                    store,
                    report,
                    history,
                })
            }
        }
    }

    pub const fn choice(&self) -> ForkChoiceV2 {
        self.report.choice
    }
    pub const fn report(&self) -> AdoptionReportV2 {
        self.report
    }
    pub fn root(&self) -> &Path {
        self.store.root()
    }

    /// Present only for strict replacement. Both histories and next local pins
    /// already exist before any publication; neither is inferred after failure.
    pub const fn recovery_inputs(&self) -> Option<&AdoptionRecoveryInputsV2> {
        match &self.history {
            PreparedHistory::Keep => None,
            PreparedHistory::Replace(inputs) => Some(inputs),
        }
    }

    /// A keep decision makes no filesystem call. Replacement uses only the owned
    /// store API, which fully rechecks and synchronizes before reporting success.
    /// Failure retains the same lock and exact inputs but exposes no normal writer.
    pub fn publish(self) -> Result<AdoptionOutcomeV2, AdoptionPublicationFailureV2> {
        let Self {
            mut store,
            report,
            history,
        } = self;
        match history {
            PreparedHistory::Keep => Ok(AdoptionOutcomeV2 { store, report }),
            PreparedHistory::Replace(inputs) => {
                match store.replace(inputs.identity, inputs.old, &inputs.candidate_chain) {
                    Ok(selected) => {
                        // The store independently derives the new local checkpoint.
                        // The immutable accepted history and generation must agree.
                        debug_assert_eq!(selected, report.selected);
                        Ok(AdoptionOutcomeV2 {
                            store,
                            report: AdoptionReportV2 { selected, ..report },
                        })
                    }
                    Err(error) => Err(AdoptionPublicationFailureV2 {
                        error,
                        store,
                        inputs,
                    }),
                }
            }
        }
    }
}

fn prepare(
    store: &ReplacementStoreV2,
    identity: ManifestIdentityV2,
    expected_current: ReplacementCheckpointV2,
    received: &ReceivedCandidateV2,
) -> Result<AdoptionReportV2, AdoptionErrorV2> {
    if store.access() != StoreAccessV2::Writer {
        return Err(AdoptionErrorV2::Storage(
            ReplacementStorageErrorV2::ReadOnly,
        ));
    }
    let current = store.chain()?;
    let comparison = compare_branches_v2(identity, current, received.chain())?;
    if store.checkpoint()? != expected_current || current.checkpoint() != expected_current.chain {
        return Err(AdoptionErrorV2::WrongCurrentCheckpoint);
    }
    // Check full canonical metadata even when keeping current. This performs
    // bounded encoding only; no disk state is discovered or promoted here.
    let canonical = ReplacementSelectionV2::from_chain(current, expected_current.generation)
        .map_err(ReplacementStorageErrorV2::from)?;
    if store.selection()? != &canonical {
        return Err(AdoptionErrorV2::CurrentMetadataMismatch);
    }
    let selected = match comparison.choice() {
        ForkChoiceV2::KeepCurrent => expected_current,
        ForkChoiceV2::PreferCandidate => prepare_replacement_v2(
            identity,
            expected_current,
            store.selection()?,
            current,
            received.chain(),
        )?
        .candidate_checkpoint(),
    };
    Ok(AdoptionReportV2 {
        choice: comparison.choice(),
        relation: comparison.relation(),
        common_ancestor: comparison.common_ancestor(),
        previous: expected_current,
        candidate: received.chain().checkpoint(),
        selected,
    })
}

/// Successful adoption or unchanged keep-current selection. Normal writer access
/// is available only here, after publication reports success, or explicit recovery.
pub struct AdoptionOutcomeV2 {
    store: ReplacementStoreV2,
    report: AdoptionReportV2,
}

impl AdoptionOutcomeV2 {
    pub const fn report(&self) -> AdoptionReportV2 {
        self.report
    }
    pub const fn checkpoint(&self) -> ReplacementCheckpointV2 {
        self.report.selected
    }
    pub const fn store(&self) -> &ReplacementStoreV2 {
        &self.store
    }
    pub fn into_store(self) -> ReplacementStoreV2 {
        self.store
    }
}

/// Explicit fresh initialization only. The caller chooses this separate operation;
/// it is never a fallback after existing-store opening fails. Partial, legacy or
/// nonempty data remains subject to the unchanged store initialization contract.
/// The borrowed completed candidate remains available if initialization fails.
pub fn initialize_received_v2(
    root: impl AsRef<Path>,
    identity: ManifestIdentityV2,
    received: &ReceivedCandidateV2,
) -> Result<ReplacementStoreV2, AdoptionErrorV2> {
    compare_branches_v2(identity, received.chain(), received.chain())?;
    Ok(ReplacementStoreV2::initialize(root, received.chain())?)
}

/// Pure preparation rejected; ownership has not been silently lost or changed.
pub struct AdoptionPreparationFailureV2 {
    error: AdoptionErrorV2,
    store: ReplacementStoreV2,
    received: ReceivedCandidateV2,
}

impl AdoptionPreparationFailureV2 {
    pub const fn error(&self) -> &AdoptionErrorV2 {
        &self.error
    }
    pub fn into_inputs(self) -> (ReplacementStoreV2, ReceivedCandidateV2) {
        (self.store, self.received)
    }
}

/// A publication failure retains exclusive ownership and both exact histories.
/// There is no method returning an ordinary writer. Recovery is always explicit.
pub struct AdoptionPublicationFailureV2 {
    error: ReplacementStorageErrorV2,
    store: ReplacementStoreV2,
    inputs: AdoptionRecoveryInputsV2,
}

impl AdoptionPublicationFailureV2 {
    pub const fn error(&self) -> &ReplacementStorageErrorV2 {
        &self.error
    }
    pub const fn recovery_inputs(&self) -> &AdoptionRecoveryInputsV2 {
        &self.inputs
    }
    pub const fn is_fenced(&self) -> bool {
        self.store.is_fenced()
    }
    pub fn root(&self) -> &Path {
        self.store.root()
    }
    /// This is prior successful memory, never an assertion of current disk state.
    pub const fn last_confirmed_checkpoint(&self) -> ReplacementCheckpointV2 {
        self.store.last_confirmed_checkpoint()
    }

    /// Consume this failure and use storage recovery under its SAME held lock.
    /// An error returns no handle but keeps the exact input histories and root for
    /// a later explicitly requested retry. A selected candidate cannot roll back.
    pub fn recover(
        self,
        action: ReplacementRecoveryActionV2,
    ) -> Result<ReplacementStoreV2, AdoptionRecoveryFailureV2> {
        let Self { store, inputs, .. } = self;
        let root = store.root().to_path_buf();
        let result = inputs
            .request()
            .and_then(|request| store.recover(&request, action));
        result.map_err(|error| AdoptionRecoveryFailureV2 {
            error,
            root,
            inputs,
        })
    }
}

/// Explicit recovery failed and returned no owning handle. Exact independent
/// inputs remain available; `retry` acquires fresh ownership through the existing
/// recovery API. It is never invoked automatically or in an unbounded loop.
pub struct AdoptionRecoveryFailureV2 {
    error: ReplacementRecoveryErrorV2,
    root: PathBuf,
    inputs: AdoptionRecoveryInputsV2,
}

impl AdoptionRecoveryFailureV2 {
    pub const fn error(&self) -> &ReplacementRecoveryErrorV2 {
        &self.error
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub const fn recovery_inputs(&self) -> &AdoptionRecoveryInputsV2 {
        &self.inputs
    }

    pub fn retry(self, action: ReplacementRecoveryActionV2) -> Result<ReplacementStoreV2, Self> {
        let Self { root, inputs, .. } = self;
        let result = inputs.request().and_then(|request| {
            ReplacementStoreV2::recover_interrupted(&root, inputs.identity, &request, action)
        });
        result.map_err(|error| Self {
            error,
            root,
            inputs,
        })
    }
}

#[derive(Debug)]
pub enum AdoptionErrorV2 {
    Comparison(ForkChoiceErrorV2),
    Storage(ReplacementStorageErrorV2),
    Plan(ReplacementPlanErrorV2),
    WrongCurrentCheckpoint,
    CurrentMetadataMismatch,
}

impl From<ForkChoiceErrorV2> for AdoptionErrorV2 {
    fn from(error: ForkChoiceErrorV2) -> Self {
        Self::Comparison(error)
    }
}
impl From<ReplacementStorageErrorV2> for AdoptionErrorV2 {
    fn from(error: ReplacementStorageErrorV2) -> Self {
        Self::Storage(error)
    }
}
impl From<ReplacementPlanErrorV2> for AdoptionErrorV2 {
    fn from(error: ReplacementPlanErrorV2) -> Self {
        Self::Plan(error)
    }
}

impl fmt::Display for AdoptionErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 candidate adoption rejected: {self:?}")
    }
}
impl std::error::Error for AdoptionErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Comparison(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::Plan(error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Debug for AdoptionAttemptV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdoptionAttemptV2")
            .field("report", &self.report)
            .finish()
    }
}
impl fmt::Debug for AdoptionOutcomeV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdoptionOutcomeV2")
            .field("report", &self.report)
            .finish()
    }
}

macro_rules! failure_error {
    ($type:ty, $name:literal) => {
        impl fmt::Debug for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct($name).field("error", &self.error).finish()
            }
        }
        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}: {}", $name, self.error)
            }
        }
        impl std::error::Error for $type {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.error)
            }
        }
    };
}

failure_error!(
    AdoptionPreparationFailureV2,
    "V2 adoption preparation failed"
);
failure_error!(
    AdoptionPublicationFailureV2,
    "V2 adoption publication failed"
);
failure_error!(AdoptionRecoveryFailureV2, "V2 adoption recovery failed");
