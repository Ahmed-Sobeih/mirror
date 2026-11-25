//! Comparison of already accepted branches in the fixed-target development profile.
//!
//! Network identity is an independent caller input. Every accepted block in that
//! network has the same configured target, so accepted height is an exact
//! normalized ordering of accumulated work. It is not a literal hash-work total,
//! human voting weight, storage generation, global latestness or finality.
//!
//! Comparison only borrows complete accepted chains. It neither publishes a
//! selection nor modifies a store, and adds no serialized selection certificate.

use core::fmt;

use super::{ChainReplayExpectationV2, ChainV2, ManifestIdentityV2};

/// Which accepted input failed the independently supplied network identity check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchSideV2 {
    Current,
    Candidate,
}

/// Relationship of the complete accepted histories, separate from preference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchRelationV2 {
    Identical,
    CandidateExtendsCurrent,
    CurrentExtendsCandidate,
    Diverged,
}

/// Development preference for these ordered inputs, not a publication permit.
///
/// Equal normalized scores retain the caller's current branch. Two peers with
/// different equal-score current branches can therefore retain different tips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForkChoiceV2 {
    KeepCurrent,
    PreferCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForkChoiceErrorV2 {
    WrongConfiguration { branch: BranchSideV2 },
    WrongGenesis { branch: BranchSideV2 },
}

impl fmt::Display for ForkChoiceErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongConfiguration { branch } => {
                write!(
                    f,
                    "V2 fork choice: {branch:?} branch has the wrong configuration"
                )
            }
            Self::WrongGenesis { branch } => {
                write!(f, "V2 fork choice: {branch:?} branch has the wrong genesis")
            }
        }
    }
}

impl std::error::Error for ForkChoiceErrorV2 {}

/// An immutable comparison bound to both complete accepted input branches.
///
/// Only [`compare_branches_v2`] constructs this value. `selected_chain()` returns
/// the entire preferred branch, keeping its blocks, manifest, accounts,
/// membership, permanent records, approvals and resource counters together.
/// Copied choices/checkpoints are diagnostic data: after the borrow ends they can
/// become stale. A later durable publication API must independently recheck its
/// own selected state and network under its store lock.
///
/// A current branch cannot be changed while its comparison remains in use:
///
/// ```compile_fail,E0502
/// use mirror_chain::v2::{compare_branches_v2, ChainV2, ManifestIdentityV2};
/// use mirror_core::v2::BlockV2;
///
/// fn cannot_change_current(
///     identity: ManifestIdentityV2,
///     current: &mut ChainV2,
///     candidate: &ChainV2,
///     child: BlockV2,
/// ) {
///     let comparison = compare_branches_v2(identity, current, candidate).unwrap();
///     current.append(child).unwrap();
///     let _selected = comparison.selected_chain();
/// }
/// ```
///
/// The candidate is held immutable too, including when the current branch wins:
///
/// ```compile_fail,E0502
/// use mirror_chain::v2::{compare_branches_v2, ChainV2, ManifestIdentityV2};
/// use mirror_core::v2::BlockV2;
///
/// fn cannot_change_candidate(
///     identity: ManifestIdentityV2,
///     current: &ChainV2,
///     candidate: &mut ChainV2,
///     child: BlockV2,
/// ) {
///     let comparison = compare_branches_v2(identity, current, candidate).unwrap();
///     candidate.append(child).unwrap();
///     let _selected = comparison.selected_chain();
/// }
/// ```
pub struct BranchComparisonV2<'a> {
    identity: ManifestIdentityV2,
    current: &'a ChainV2,
    candidate: &'a ChainV2,
    common_ancestor: ChainReplayExpectationV2,
    relation: BranchRelationV2,
    choice: ForkChoiceV2,
}

impl<'a> BranchComparisonV2<'a> {
    pub const fn identity(&self) -> ManifestIdentityV2 {
        self.identity
    }

    pub const fn relation(&self) -> BranchRelationV2 {
        self.relation
    }

    pub const fn choice(&self) -> ForkChoiceV2 {
        self.choice
    }

    /// Full checkpoint of the last shared accepted block, including its root.
    pub const fn common_ancestor(&self) -> ChainReplayExpectationV2 {
        self.common_ancestor
    }

    /// Accepted post-genesis block count, meaningful only within this identity.
    /// The existing finite chain profile bounds it to `0..=4095`.
    pub const fn current_score(&self) -> u64 {
        self.current.height()
    }

    /// Accepted post-genesis block count under the same fixed-target identity.
    pub const fn candidate_score(&self) -> u64 {
        self.candidate.height()
    }

    pub fn current_checkpoint(&self) -> ChainReplayExpectationV2 {
        self.current.checkpoint()
    }

    pub fn candidate_checkpoint(&self) -> ChainReplayExpectationV2 {
        self.candidate.checkpoint()
    }

    /// Borrow the whole preferred chain without copying or grafting its state.
    /// This does not mutate either input or publish a canonical disk selection.
    pub const fn selected_chain(&self) -> &'a ChainV2 {
        match self.choice {
            ForkChoiceV2::KeepCurrent => self.current,
            ForkChoiceV2::PreferCandidate => self.candidate,
        }
    }
}

impl fmt::Debug for BranchComparisonV2<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BranchComparisonV2")
            .field("identity", &self.identity)
            .field("current_checkpoint", &self.current_checkpoint())
            .field("candidate_checkpoint", &self.candidate_checkpoint())
            .field("common_ancestor", &self.common_ancestor)
            .field("relation", &self.relation)
            .field("choice", &self.choice)
            .finish()
    }
}

/// Compare two fully accepted branches against an independently chosen network.
///
/// Checks current configuration/genesis, then candidate configuration/genesis.
/// The candidate wins exactly when its accepted height is greater; all ties keep
/// current. No score, block, cache or peer advertisement can be supplied instead
/// of an accepted [`ChainV2`]. For example, an unaccepted block is the wrong type:
///
/// ```compile_fail,E0308
/// use mirror_chain::v2::{compare_branches_v2, ChainV2, ManifestIdentityV2};
/// use mirror_core::v2::BlockV2;
///
/// fn cannot_compare_unaccepted_block(
///     identity: ManifestIdentityV2,
///     current: &ChainV2,
///     proposal: &BlockV2,
/// ) {
///     let _ = compare_branches_v2(identity, current, proposal);
/// }
/// ```
///
/// Shared ancestry is found by comparing actual accepted header hashes in order
/// until the first mismatch. The existing profile permits at most 4,096 pairs
/// (8,192 fixed-header hashes). There is no allocation, body cloning, disk I/O or
/// additional ledger execution. Construction/append/replay already established
/// the private chain invariants and all finite resource limits.
pub fn compare_branches_v2<'a>(
    expected: ManifestIdentityV2,
    current: &'a ChainV2,
    candidate: &'a ChainV2,
) -> Result<BranchComparisonV2<'a>, ForkChoiceErrorV2> {
    check_identity(expected, current, BranchSideV2::Current)?;
    check_identity(expected, candidate, BranchSideV2::Candidate)?;

    // Both accepted histories have the exact independently pinned genesis.
    // Collision-resistant parent links make the matching prefix their ancestry.
    let mut ancestor = current.genesis();
    let mut ancestor_hash = expected.genesis_hash;
    for (left, right) in current.blocks().iter().zip(candidate.blocks()) {
        let left_hash = left.hash();
        if left_hash != right.hash() {
            break;
        }
        ancestor = left;
        ancestor_hash = left_hash;
    }
    let common_ancestor = ChainReplayExpectationV2 {
        config_hash: expected.config_hash,
        genesis_hash: expected.genesis_hash,
        height: ancestor.header().height(),
        tip_hash: ancestor_hash,
        ledger_root: ancestor.header().ledger_root(),
    };
    let current_is_prefix = common_ancestor.height == current.height();
    let candidate_is_prefix = common_ancestor.height == candidate.height();
    let relation = match (current_is_prefix, candidate_is_prefix) {
        (true, true) => BranchRelationV2::Identical,
        (true, false) => BranchRelationV2::CandidateExtendsCurrent,
        (false, true) => BranchRelationV2::CurrentExtendsCandidate,
        (false, false) => BranchRelationV2::Diverged,
    };
    // Same configuration means the identical positive per-block work constant.
    // Comparing normalized counts avoids any multiplication or accumulation.
    let choice = if candidate.height() > current.height() {
        ForkChoiceV2::PreferCandidate
    } else {
        ForkChoiceV2::KeepCurrent
    };
    Ok(BranchComparisonV2 {
        identity: expected,
        current,
        candidate,
        common_ancestor,
        relation,
        choice,
    })
}

fn check_identity(
    expected: ManifestIdentityV2,
    chain: &ChainV2,
    branch: BranchSideV2,
) -> Result<(), ForkChoiceErrorV2> {
    if chain.config().context() != expected.config_hash {
        return Err(ForkChoiceErrorV2::WrongConfiguration { branch });
    }
    if chain.manifest().genesis_hash() != expected.genesis_hash {
        return Err(ForkChoiceErrorV2::WrongGenesis { branch });
    }
    Ok(())
}
