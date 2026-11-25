//! Development fork choice over fully accepted public synthetic histories.
//! Expected network identity and complete block vectors predate this comparator.

use std::collections::BTreeMap;

use mirror_chain::v2::{
    BranchComparisonV2, BranchRelationV2 as Relation, BranchSideV2 as Side, ChainErrorV2,
    ChainReplayExpectationV2, ChainV2, ForkChoiceErrorV2 as Error, ForkChoiceV2 as Choice,
    MAX_CHAIN_BLOCKS_V2, ManifestIdentityV2, NetworkManifestV2, compare_branches_v2,
    genesis_template_v2,
};
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::BlockV2;
use mirror_crypto::{Hash256, Keypair};
use mirror_personhood::v2::{ConfigV2, MembershipStatusV2 as Status};

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

fn hash(value: &str) -> Hash256 {
    Hash256::from_bytes(unhex(value).try_into().unwrap())
}

fn identity() -> ManifestIdentityV2 {
    // Independent pins for the existing public Python-derived vector network.
    // No incoming manifest, selector or branch supplies these expectations.
    ManifestIdentityV2 {
        config_hash: hash("9abab0c437d2883d70c68084a3f2dc843f3f2ab7ccb69a80db3fbf1ebff27e5e"),
        genesis_hash: hash("4415cf74a204f2df610aa1e4ed737df220d93e1ad8a931e9533c00cf624c14ea"),
    }
}

struct Fixture {
    values: BTreeMap<&'static str, &'static str>,
    manifest: NetworkManifestV2,
}

impl Fixture {
    fn new() -> Self {
        let values: BTreeMap<_, _> = include_str!("vectors/chain_v2.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_once('=').unwrap())
            .collect();
        let manifest =
            NetworkManifestV2::decode_for(&unhex(values["manifest_hex"]), identity()).unwrap();
        Self { values, manifest }
    }

    fn hash(&self, field: &str) -> Hash256 {
        hash(self.values[field])
    }

    fn block(&self, name: &str) -> BlockV2 {
        let bytes = unhex(self.values[format!("{name}_block").as_str()]);
        BlockV2::decode(self.manifest.config(), &bytes).unwrap()
    }

    fn main(&self, height: u64) -> ChainV2 {
        assert!(height <= 4);
        let mut chain = ChainV2::from_genesis(&self.manifest, self.block("block0")).unwrap();
        for next in 1..=height {
            chain.append(self.block(&format!("block{next}"))).unwrap();
        }
        chain
    }

    fn fork(&self, height: u64) -> ChainV2 {
        assert!((2..=3).contains(&height));
        let mut chain = self.main(1);
        for next in 2..=height {
            chain.append(self.block(&format!("fork{next}"))).unwrap();
        }
        chain
    }

    fn alternate_genesis(&self) -> ChainV2 {
        let genesis = self.block("alternate_genesis");
        let manifest = NetworkManifestV2::new(
            self.manifest.config(),
            self.manifest.seed_proofs(),
            genesis.hash(),
        )
        .unwrap();
        ChainV2::from_genesis(&manifest, genesis).unwrap()
    }

    fn different_config(&self) -> ChainV2 {
        let source = self.manifest.config();
        let mut parameters = source.parameters();
        parameters.network = Hash256::from_bytes([0xe1; 32]);
        let config = ConfigV2::new(
            parameters,
            *source.limits(),
            source.bootstrap(),
            source.allocations(),
        )
        .unwrap();
        let proofs: Vec<_> = (0u32..2)
            .map(|index| {
                config
                    .sign_seed(index, &Keypair::from_secret_bytes([index as u8 + 1; 32]))
                    .unwrap()
            })
            .collect();
        let bootstrap = config.verify_bootstrap(&proofs).unwrap();
        let genesis = mined(&config, &genesis_template_v2(&bootstrap).unwrap());
        let manifest = NetworkManifestV2::new(&config, &proofs, genesis.hash()).unwrap();
        ChainV2::from_genesis(&manifest, genesis).unwrap()
    }
}

fn mined(config: &ConfigV2, block: &BlockV2) -> BlockV2 {
    let result = mine_header_v2(*block.header(), config.parameters().pow_bits, 10_000).unwrap();
    BlockV2::new(config, result.header, block.operations()).unwrap()
}

fn append_empty_at(chain: &mut ChainV2, timestamp: u64) {
    let block = mined(
        chain.config(),
        &chain.prepare_candidate(timestamp, &[]).unwrap(),
    );
    chain.append(block).unwrap();
}

fn append_empty(chain: &mut ChainV2) {
    let timestamp = chain.blocks().last().unwrap().header().timestamp() + 1;
    append_empty_at(chain, timestamp);
}

fn assert_comparison<'a>(
    current: &'a ChainV2,
    candidate: &'a ChainV2,
    relation: Relation,
    choice: Choice,
    ancestor_height: u64,
) -> BranchComparisonV2<'a> {
    let before_current = current.clone();
    let before_candidate = candidate.clone();
    let result = compare_branches_v2(identity(), current, candidate).unwrap();
    assert_eq!(result.identity(), identity());
    assert_eq!(result.relation(), relation);
    assert_eq!(result.choice(), choice);
    assert_eq!(result.current_score(), current.height());
    assert_eq!(result.candidate_score(), candidate.height());
    assert_eq!(result.current_checkpoint(), current.checkpoint());
    assert_eq!(result.candidate_checkpoint(), candidate.checkpoint());
    let ancestor = current.block_at(ancestor_height).unwrap();
    assert_eq!(candidate.block_at(ancestor_height), Some(ancestor));
    assert_eq!(
        result.common_ancestor(),
        ChainReplayExpectationV2 {
            config_hash: identity().config_hash,
            genesis_hash: identity().genesis_hash,
            height: ancestor_height,
            tip_hash: ancestor.hash(),
            ledger_root: ancestor.header().ledger_root(),
        }
    );
    let selected = match choice {
        Choice::KeepCurrent => current,
        Choice::PreferCandidate => candidate,
    };
    assert!(std::ptr::eq(result.selected_chain(), selected));
    assert_eq!(result.selected_chain(), selected);
    assert_eq!(current, &before_current);
    assert_eq!(candidate, &before_candidate);
    result
}

fn assert_rejected(
    expected: ManifestIdentityV2,
    current: &ChainV2,
    candidate: &ChainV2,
    error: Error,
) {
    let before_current = current.clone();
    let before_candidate = candidate.clone();
    assert_eq!(
        compare_branches_v2(expected, current, candidate).err(),
        Some(error)
    );
    assert_eq!(current, &before_current);
    assert_eq!(candidate, &before_candidate);
}

#[test]
fn identical_genesis_has_zero_score_and_keeps_current() {
    let fixture = Fixture::new();
    let current = fixture.main(0);
    let candidate = fixture.main(0);
    assert_comparison(
        &current,
        &candidate,
        Relation::Identical,
        Choice::KeepCurrent,
        0,
    );
}

#[test]
fn independently_replayed_identical_history_uses_its_tip_as_common_ancestor() {
    let fixture = Fixture::new();
    let current = fixture.main(4);
    let candidate =
        ChainV2::replay(&fixture.manifest, current.blocks(), current.checkpoint()).unwrap();
    assert_comparison(
        &current,
        &candidate,
        Relation::Identical,
        Choice::KeepCurrent,
        4,
    );
}

#[test]
fn same_chain_reference_is_an_identical_comparison() {
    let current = Fixture::new().main(3);
    assert_comparison(
        &current,
        &current,
        Relation::Identical,
        Choice::KeepCurrent,
        3,
    );
}

#[test]
fn one_child_extension_is_preferred_without_mutating_current() {
    let fixture = Fixture::new();
    assert_comparison(
        &fixture.main(2),
        &fixture.main(3),
        Relation::CandidateExtendsCurrent,
        Choice::PreferCandidate,
        2,
    );
}

#[test]
fn multi_child_extension_preserves_complete_candidate_ledger() {
    let fixture = Fixture::new();
    let current = fixture.main(1);
    let candidate = fixture.main(4);
    let result = assert_comparison(
        &current,
        &candidate,
        Relation::CandidateExtendsCurrent,
        Choice::PreferCandidate,
        1,
    );
    let selected = result.selected_chain();
    assert_eq!(
        selected.ledger().root().unwrap(),
        fixture.hash("state4_ledger_root")
    );
    assert_eq!(
        selected
            .ledger()
            .membership()
            .record(fixture.hash("c_member_id"))
            .unwrap()
            .status(),
        Status::Revoked
    );
    assert_eq!(
        selected
            .ledger()
            .membership()
            .record(fixture.hash("seed0_id"))
            .unwrap()
            .status(),
        Status::Revoked
    );
    assert_eq!(selected.state_items_used(), candidate.state_items_used());
    assert_eq!(selected.encoded_bytes(), candidate.encoded_bytes());
}

#[test]
fn authentic_shorter_prefix_keeps_current() {
    let fixture = Fixture::new();
    assert_comparison(
        &fixture.main(4),
        &fixture.main(1),
        Relation::CurrentExtendsCandidate,
        Choice::KeepCurrent,
        1,
    );
}

#[test]
fn genesis_candidate_cannot_replace_a_nonempty_history() {
    let fixture = Fixture::new();
    assert_comparison(
        &fixture.main(4),
        &fixture.main(0),
        Relation::CurrentExtendsCandidate,
        Choice::KeepCurrent,
        0,
    );
}

#[test]
fn longer_divergent_membership_branch_is_preferred() {
    let fixture = Fixture::new();
    assert_comparison(
        &fixture.main(2),
        &fixture.fork(3),
        Relation::Diverged,
        Choice::PreferCandidate,
        1,
    );
}

#[test]
fn shorter_divergent_membership_branch_keeps_current() {
    let fixture = Fixture::new();
    assert_comparison(
        &fixture.main(4),
        &fixture.fork(3),
        Relation::Diverged,
        Choice::KeepCurrent,
        1,
    );
}

#[test]
fn equal_score_divergence_keeps_current_even_when_candidate_hash_is_lower() {
    let fixture = Fixture::new();
    let current = fixture.main(3);
    let candidate = fixture.fork(3);
    assert!(candidate.tip_hash().as_bytes() < current.tip_hash().as_bytes());
    assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::KeepCurrent,
        1,
    );
}

#[test]
fn reversed_equal_score_divergence_also_keeps_its_current_branch() {
    let fixture = Fixture::new();
    let current = fixture.fork(3);
    let candidate = fixture.main(3);
    assert!(candidate.tip_hash().as_bytes() > current.tip_hash().as_bytes());
    assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::KeepCurrent,
        1,
    );
}

#[test]
fn identical_accounts_do_not_make_different_membership_branches_identical() {
    let fixture = Fixture::new();
    let current = fixture.main(3);
    let candidate = fixture.fork(3);
    assert_eq!(
        current.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        current.ledger().membership().root().unwrap(),
        candidate.ledger().membership().root().unwrap()
    );
    assert_ne!(
        current.ledger().root().unwrap(),
        candidate.ledger().root().unwrap()
    );
    let result = assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::KeepCurrent,
        1,
    );
    assert_eq!(
        result
            .selected_chain()
            .ledger()
            .membership()
            .record(fixture.hash("c_member_id"))
            .unwrap()
            .status(),
        Status::Active
    );
}

#[test]
fn preferred_fork_retains_its_own_membership_approvals_and_population_counters() {
    let fixture = Fixture::new();
    let current = fixture.main(3);
    let mut candidate = fixture.fork(3);
    append_empty(&mut candidate);
    assert_eq!(
        current.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert!(
        current.ledger().membership().active_count()
            > candidate.ledger().membership().active_count()
    );
    assert!(
        current.ledger().membership().active_high_water()
            > candidate.ledger().membership().active_high_water()
    );
    let result = assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::PreferCandidate,
        1,
    );
    let selected = result.selected_chain();
    assert_eq!(
        selected.ledger().membership(),
        candidate.ledger().membership()
    );
    assert_eq!(
        selected
            .ledger()
            .membership()
            .record(fixture.hash("c_member_id"))
            .unwrap()
            .status(),
        Status::Pending
    );
    assert_eq!(
        selected
            .ledger()
            .membership()
            .record(fixture.hash("seed0_id"))
            .unwrap()
            .status(),
        Status::Revoked
    );
    assert_eq!(
        selected.ledger().membership().retained_approval_count(),
        candidate.ledger().membership().retained_approval_count()
    );
    assert_eq!(
        selected.ledger().membership().total_activations(),
        candidate.ledger().membership().total_activations()
    );
    assert_eq!(selected.state_items_used(), candidate.state_items_used());
    assert_eq!(selected.encoded_bytes(), candidate.encoded_bytes());
}

#[test]
fn empty_blocks_count_as_work_and_expiry_stays_with_the_selected_branch() {
    let fixture = Fixture::new();
    let current = fixture.main(3);
    let mut candidate = fixture.fork(3);
    for _ in 0..3 {
        append_empty(&mut candidate);
    }
    assert_eq!(candidate.height(), 6);
    assert_eq!(
        candidate
            .ledger()
            .membership()
            .record(fixture.hash("c_member_id"))
            .unwrap()
            .status(),
        Status::Expired
    );
    let result = assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::PreferCandidate,
        1,
    );
    assert_eq!(result.candidate_score(), 6);
    assert_eq!(result.selected_chain().ledger(), candidate.ledger());
    assert_eq!(
        result.selected_chain().ledger().membership().record_count(),
        candidate.ledger().membership().record_count()
    );
}

#[test]
fn divergence_at_first_child_has_genesis_as_common_ancestor() {
    let fixture = Fixture::new();
    let current = fixture.main(1);
    let mut candidate = fixture.main(0);
    append_empty_at(&mut candidate, 2);
    append_empty(&mut candidate);
    assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::PreferCandidate,
        0,
    );
}

#[test]
fn common_ancestor_is_latest_matching_block_in_a_deeper_history() {
    let fixture = Fixture::new();
    let mut common = fixture.main(3);
    for _ in 0..7 {
        append_empty(&mut common);
    }
    let mut current = common.clone();
    let mut candidate = common.clone();
    append_empty_at(&mut current, 11);
    append_empty_at(&mut candidate, 12);
    append_empty(&mut candidate);
    assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::PreferCandidate,
        10,
    );
}

#[test]
fn timestamp_magnitude_does_not_break_a_work_tie() {
    let fixture = Fixture::new();
    let mut current = fixture.main(0);
    let mut candidate = fixture.main(0);
    append_empty_at(&mut current, 1);
    append_empty_at(&mut candidate, u64::MAX);
    assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::KeepCurrent,
        0,
    );
    assert_comparison(
        &candidate,
        &current,
        Relation::Diverged,
        Choice::KeepCurrent,
        0,
    );
}

#[test]
fn wrong_independent_configuration_rejects_even_identical_inputs() {
    let chain = Fixture::new().main(3);
    assert_rejected(
        ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([0xf1; 32]),
            ..identity()
        },
        &chain,
        &chain,
        Error::WrongConfiguration {
            branch: Side::Current,
        },
    );
}

#[test]
fn wrong_independent_genesis_rejects_even_identical_inputs() {
    let chain = Fixture::new().main(3);
    assert_rejected(
        ManifestIdentityV2 {
            genesis_hash: Hash256::from_bytes([0xf2; 32]),
            ..identity()
        },
        &chain,
        &chain,
        Error::WrongGenesis {
            branch: Side::Current,
        },
    );
}

#[test]
fn candidate_with_same_config_but_other_valid_genesis_is_rejected() {
    let fixture = Fixture::new();
    let current = fixture.main(0);
    let candidate = fixture.alternate_genesis();
    assert_eq!(current.config(), candidate.config());
    assert_ne!(current.genesis().hash(), candidate.genesis().hash());
    assert_rejected(
        identity(),
        &current,
        &candidate,
        Error::WrongGenesis {
            branch: Side::Candidate,
        },
    );
}

#[test]
fn current_with_other_valid_genesis_is_rejected_before_candidate() {
    let fixture = Fixture::new();
    assert_rejected(
        identity(),
        &fixture.alternate_genesis(),
        &fixture.main(4),
        Error::WrongGenesis {
            branch: Side::Current,
        },
    );
}

#[test]
fn candidate_with_different_valid_configuration_is_rejected() {
    let fixture = Fixture::new();
    let current = fixture.main(0);
    let mut candidate = fixture.different_config();
    append_empty(&mut candidate);
    assert!(candidate.height() > current.height());
    assert_rejected(
        identity(),
        &current,
        &candidate,
        Error::WrongConfiguration {
            branch: Side::Candidate,
        },
    );
}

#[test]
fn current_with_different_configuration_is_rejected_before_candidate() {
    let fixture = Fixture::new();
    assert_rejected(
        identity(),
        &fixture.different_config(),
        &fixture.main(4),
        Error::WrongConfiguration {
            branch: Side::Current,
        },
    );
}

#[test]
fn configuration_error_precedes_genesis_error_for_the_same_side() {
    let fixture = Fixture::new();
    let chain = fixture.main(0);
    assert_rejected(
        ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([0xf1; 32]),
            genesis_hash: Hash256::from_bytes([0xf2; 32]),
        },
        &chain,
        &fixture.different_config(),
        Error::WrongConfiguration {
            branch: Side::Current,
        },
    );
}

#[test]
fn current_genesis_error_precedes_candidate_configuration_error() {
    let fixture = Fixture::new();
    assert_rejected(
        identity(),
        &fixture.alternate_genesis(),
        &fixture.different_config(),
        Error::WrongGenesis {
            branch: Side::Current,
        },
    );
}

#[test]
fn comparison_can_explicitly_select_another_accepted_network_identity() {
    let fixture = Fixture::new();
    let current = fixture.alternate_genesis();
    let mut candidate = current.clone();
    append_empty(&mut candidate);
    let selected_identity = current.manifest().identity();
    let result = compare_branches_v2(selected_identity, &current, &candidate).unwrap();
    assert_eq!(result.identity(), selected_identity);
    assert_eq!(result.choice(), Choice::PreferCandidate);
    assert_eq!(result.relation(), Relation::CandidateExtendsCurrent);
    assert_eq!(result.common_ancestor(), current.checkpoint());
    assert!(std::ptr::eq(result.selected_chain(), &candidate));
}

#[test]
fn invalid_peer_block_cannot_increase_the_comparable_accepted_score() {
    let fixture = Fixture::new();
    let mut current = fixture.main(0);
    let before = current.clone();
    assert_eq!(
        current.append(fixture.block("wrong_root1")),
        Err(ChainErrorV2::WrongLedgerRoot)
    );
    assert_eq!(current, before);
    let candidate = fixture.main(1);
    let result = assert_comparison(
        &current,
        &candidate,
        Relation::CandidateExtendsCurrent,
        Choice::PreferCandidate,
        0,
    );
    assert_eq!(result.current_score(), 0);
    assert_eq!(result.candidate_score(), 1);
}

#[test]
fn maximum_accepted_history_compares_without_score_overflow_or_extra_blocks() {
    let fixture = Fixture::new();
    let mut prefix = fixture.main(0);
    for _ in 0..MAX_CHAIN_BLOCKS_V2 - 2 {
        append_empty(&mut prefix);
    }
    let mut current = prefix.clone();
    let mut candidate = prefix.clone();
    append_empty(&mut current);
    append_empty_at(
        &mut candidate,
        current.blocks().last().unwrap().header().timestamp() + 1,
    );
    assert_eq!(current.blocks().len(), MAX_CHAIN_BLOCKS_V2);
    assert_eq!(candidate.blocks().len(), MAX_CHAIN_BLOCKS_V2);
    let result = assert_comparison(
        &current,
        &candidate,
        Relation::Diverged,
        Choice::KeepCurrent,
        (MAX_CHAIN_BLOCKS_V2 - 2) as u64,
    );
    assert_eq!(result.current_score(), (MAX_CHAIN_BLOCKS_V2 - 1) as u64);
    assert_eq!(result.candidate_score(), (MAX_CHAIN_BLOCKS_V2 - 1) as u64);
    assert_eq!(
        candidate.prepare_candidate(u64::MAX, &[]),
        Err(ChainErrorV2::TooManyBlocks)
    );
    assert_comparison(
        &prefix,
        &current,
        Relation::CandidateExtendsCurrent,
        Choice::PreferCandidate,
        prefix.height(),
    );
}
