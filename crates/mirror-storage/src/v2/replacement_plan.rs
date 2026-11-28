//! Pure preparation of a strictly preferred accepted branch for storage profile 3.
//!
//! This module performs no filesystem operation and acquires no store lock. It
//! checks independent network/current pins, the complete current metadata and
//! actual accepted branch preference. Its result cannot publish or recover a
//! store. A future writer must repeat these checks against its selected bytes
//! while retaining exclusive ownership of the same root directory inode.

use std::fmt;

use mirror_chain::v2::{
    BranchComparisonV2, BranchRelationV2, ChainReplayExpectationV2, ChainV2, ForkChoiceErrorV2,
    ForkChoiceV2, ManifestIdentityV2, compare_branches_v2,
};

use super::replacement_selection::{
    ReplacementCheckpointV2, ReplacementSelectionErrorV2, ReplacementSelectionV2,
};

/// Checked in-memory intent, borrowing both complete accepted histories.
///
/// The current selector and candidate selector are bounded metadata, not durable
/// commit permits. Neither a plan nor a copied checkpoint says which files are
/// selected, whether any publication happened, or which branch is globally final.
/// There is no public constructor, decoder or filesystem mutation method.
///
/// A candidate cannot change while the plan still refers to it:
///
/// ```compile_fail,E0502
/// use mirror_chain::v2::{ChainV2, ManifestIdentityV2};
/// use mirror_core::v2::BlockV2;
/// use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementSelectionV2,
///     prepare_replacement_v2};
/// fn cannot_change_candidate(identity: ManifestIdentityV2,
///     expected: ReplacementCheckpointV2, selection: &ReplacementSelectionV2,
///     current: &ChainV2, candidate: &mut ChainV2, child: BlockV2) {
///     let plan = prepare_replacement_v2(identity, expected, selection, current, candidate).unwrap();
///     candidate.append(child).unwrap();
///     let _selected = plan.candidate_chain();
/// }
/// ```
///
/// The accepted current history is held immutable as well:
///
/// ```compile_fail,E0502
/// use mirror_chain::v2::{ChainV2, ManifestIdentityV2};
/// use mirror_core::v2::BlockV2;
/// use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementSelectionV2,
///     prepare_replacement_v2};
/// fn cannot_change_current(identity: ManifestIdentityV2,
///     expected: ReplacementCheckpointV2, selection: &ReplacementSelectionV2,
///     current: &mut ChainV2, candidate: &ChainV2, child: BlockV2) {
///     let plan = prepare_replacement_v2(identity, expected, selection, current, candidate).unwrap();
///     current.append(child).unwrap();
///     let _previous = plan.current_chain();
/// }
/// ```
pub struct ReplacementPlanV2<'a> {
    comparison: BranchComparisonV2<'a>,
    current: &'a ChainV2,
    current_selection: ReplacementSelectionV2,
    candidate_selection: ReplacementSelectionV2,
}

impl<'a> ReplacementPlanV2<'a> {
    pub const fn identity(&self) -> ManifestIdentityV2 {
        self.comparison.identity()
    }

    pub const fn relation(&self) -> BranchRelationV2 {
        self.comparison.relation()
    }

    pub const fn common_ancestor(&self) -> ChainReplayExpectationV2 {
        self.comparison.common_ancestor()
    }

    pub const fn current_selection(&self) -> &ReplacementSelectionV2 {
        &self.current_selection
    }

    pub const fn candidate_selection(&self) -> &ReplacementSelectionV2 {
        &self.candidate_selection
    }

    pub const fn current_checkpoint(&self) -> ReplacementCheckpointV2 {
        self.current_selection.checkpoint()
    }

    pub const fn candidate_checkpoint(&self) -> ReplacementCheckpointV2 {
        self.candidate_selection.checkpoint()
    }

    pub const fn current_chain(&self) -> &'a ChainV2 {
        self.current
    }

    /// The exact whole candidate; no state is copied or grafted from the old fork.
    pub const fn candidate_chain(&self) -> &'a ChainV2 {
        self.comparison.selected_chain()
    }
}

impl fmt::Debug for ReplacementPlanV2<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReplacementPlanV2")
            .field("identity", &self.identity())
            .field("relation", &self.relation())
            .field("common_ancestor", &self.common_ancestor())
            .field("current_checkpoint", &self.current_checkpoint())
            .field("candidate_checkpoint", &self.candidate_checkpoint())
            .finish()
    }
}

/// Prepare metadata for one strictly preferred complete accepted branch.
///
/// `identity` and `expected_current` are independent caller selections, not values
/// learned from the selector or an untrusted peer. A decoded selector is checked
/// against the complete current chain, including manifest checksum/length and
/// encoded block-byte totals, rather than only its endpoint fields.
///
/// Error order: comparator network checks (current then candidate); current chain
/// endpoint; current selector endpoint/generation; exact derived current metadata;
/// strict candidate preference; generation increment; derived candidate metadata.
/// The existing comparator also computes real common ancestry and fixed-target
/// scores. Every tie, shorter candidate and identical branch is rejected without
/// incrementing generation. A multi-block extension advances generation only once.
///
/// Two bounded manifest encodings are needed on success. No chain is cloned or
/// reexecuted, but unlike pure fork comparison this function is not allocation-free.
/// Success does not change any file or either input. There is no durable-replacement
/// API accepting this plan as an authorization token.
pub fn prepare_replacement_v2<'a>(
    identity: ManifestIdentityV2,
    expected_current: ReplacementCheckpointV2,
    current_selection: &ReplacementSelectionV2,
    current: &'a ChainV2,
    candidate: &'a ChainV2,
) -> Result<ReplacementPlanV2<'a>, ReplacementPlanErrorV2> {
    let comparison = compare_branches_v2(identity, current, candidate)?;
    if current.checkpoint() != expected_current.chain {
        return Err(ReplacementPlanErrorV2::WrongCurrentCheckpoint);
    }
    if current_selection.checkpoint() != expected_current {
        return Err(ReplacementPlanErrorV2::WrongCurrentSelection);
    }
    let canonical_current =
        ReplacementSelectionV2::from_chain(current, expected_current.generation)?;
    if *current_selection != canonical_current {
        return Err(ReplacementPlanErrorV2::CurrentMetadataMismatch);
    }
    if comparison.choice() != ForkChoiceV2::PreferCandidate {
        return Err(ReplacementPlanErrorV2::CandidateNotPreferred);
    }
    let generation = expected_current
        .generation
        .checked_add(1)
        .ok_or(ReplacementPlanErrorV2::GenerationOverflow)?;
    let candidate_selection = ReplacementSelectionV2::from_chain(candidate, generation)?;
    Ok(ReplacementPlanV2 {
        comparison,
        current,
        current_selection: canonical_current,
        candidate_selection,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementPlanErrorV2 {
    Comparison(ForkChoiceErrorV2),
    WrongCurrentCheckpoint,
    WrongCurrentSelection,
    CurrentMetadataMismatch,
    CandidateNotPreferred,
    GenerationOverflow,
    Selection(ReplacementSelectionErrorV2),
}

impl From<ForkChoiceErrorV2> for ReplacementPlanErrorV2 {
    fn from(error: ForkChoiceErrorV2) -> Self {
        Self::Comparison(error)
    }
}

impl From<ReplacementSelectionErrorV2> for ReplacementPlanErrorV2 {
    fn from(error: ReplacementSelectionErrorV2) -> Self {
        Self::Selection(error)
    }
}

impl fmt::Display for ReplacementPlanErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "V2 replacement preparation rejected: {self:?}")
    }
}

impl std::error::Error for ReplacementPlanErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Comparison(error) => Some(error),
            Self::Selection(error) => Some(error),
            _ => None,
        }
    }
}
