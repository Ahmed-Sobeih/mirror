//! Actual accepted histories, independent fixture pins and malicious metadata.

mod support_v2;

use mirror_chain::v2::{
    BranchRelationV2, BranchSideV2, ChainErrorV2, ChainV2, ForkChoiceErrorV2, NetworkManifestV2,
};
use mirror_core::v2::{BlockHeaderV2, BlockV2};
use mirror_crypto::{Hash256, sha256d};
use mirror_storage::v2::{
    ReplacementCheckpointV2, ReplacementPlanErrorV2 as Error, ReplacementSelectionV2,
    prepare_replacement_v2,
};
use support_v2::{
    fixture_bytes, fixture_hash, fixture_identity, fork_chain, fork_expectation, main_chain,
    main_expectation, manifest,
};

fn selection(chain: &ChainV2, generation: u64) -> ReplacementSelectionV2 {
    ReplacementSelectionV2::from_chain(chain, generation).unwrap()
}

fn main_pin(height: u64, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: main_expectation(height),
    }
}

fn alternate_genesis() -> ChainV2 {
    let base = manifest();
    let genesis =
        BlockV2::decode(base.config(), &fixture_bytes("alternate_genesis_block")).unwrap();
    let alternative =
        NetworkManifestV2::new(base.config(), base.seed_proofs(), genesis.hash()).unwrap();
    ChainV2::from_genesis(&alternative, genesis).unwrap()
}

fn append_empty(chain: &mut ChainV2) {
    let timestamp = chain
        .blocks()
        .last()
        .unwrap()
        .header()
        .timestamp()
        .checked_add(1)
        .unwrap();
    let proposal = chain.prepare_candidate(timestamp, &[]).unwrap();
    let header = proposal.header();
    // Use complete real acceptance as the mining predicate; no mock work checker.
    for nonce in 0..10_000 {
        let candidate = BlockHeaderV2::new(
            header.config_hash(),
            header.height(),
            header.previous_hash(),
            header.operations_root(),
            header.ledger_root(),
            header.timestamp(),
            header.pow_bits(),
            nonce,
        );
        let block = BlockV2::new(chain.config(), candidate, proposal.operations()).unwrap();
        match chain.append(block) {
            Ok(()) => return,
            Err(ChainErrorV2::Work(_)) => {}
            Err(error) => panic!("unexpected empty-child rejection: {error}"),
        }
    }
    panic!("synthetic easy-target search exhausted");
}

fn rechecksum(bytes: &mut [u8; 280]) {
    let mut inner = b"MIRROR_STORAGE_SELECTION_V2".to_vec();
    inner.extend_from_slice(&bytes[16..216]);
    bytes[216..248].copy_from_slice(sha256d(&inner).as_bytes());
    let mut outer = b"MIRROR_STORAGE_REPLACEMENT_SELECTION_V2".to_vec();
    outer.extend_from_slice(&bytes[..248]);
    bytes[248..].copy_from_slice(sha256d(&outer).as_bytes());
}

#[test]
fn preferred_divergent_branch_keeps_complete_candidate_and_independent_pins() {
    let current = fork_chain();
    let candidate = main_chain(4);
    let current_before = current.clone();
    let candidate_before = candidate.clone();
    let old = selection(&current, 0);
    let expected = ReplacementCheckpointV2 {
        generation: 0,
        chain: fork_expectation(),
    };
    let plan =
        prepare_replacement_v2(fixture_identity(), expected, &old, &current, &candidate).unwrap();
    assert_eq!(plan.identity(), fixture_identity());
    assert_eq!(plan.relation(), BranchRelationV2::Diverged);
    assert_eq!(plan.common_ancestor(), main_expectation(1));
    assert_eq!(plan.current_checkpoint(), expected);
    assert_eq!(plan.candidate_checkpoint(), main_pin(4, 1));
    assert_eq!(plan.current_selection(), &old);
    assert_eq!(plan.candidate_selection(), &selection(&candidate, 1));
    assert!(std::ptr::eq(plan.current_chain(), &current));
    assert!(std::ptr::eq(plan.candidate_chain(), &candidate));
    assert_eq!(plan.candidate_chain().ledger(), candidate.ledger());
    assert_eq!(
        plan.candidate_chain().state_items_used(),
        candidate.state_items_used()
    );
    assert_eq!(
        plan.candidate_chain().encoded_bytes(),
        candidate.encoded_bytes()
    );
    assert_eq!(current, current_before);
    assert_eq!(candidate, candidate_before);
}

#[test]
fn equal_account_bytes_cannot_graft_membership_from_current_branch() {
    let current = main_chain(3);
    let mut candidate = fork_chain();
    append_empty(&mut candidate);
    assert_eq!(
        current.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        current.ledger().membership().root().unwrap(),
        candidate.ledger().membership().root().unwrap()
    );
    assert!(
        current.ledger().membership().active_count()
            > candidate.ledger().membership().active_count()
    );
    assert!(
        current.ledger().membership().active_high_water()
            > candidate.ledger().membership().active_high_water()
    );
    let old = selection(&current, 2);
    let plan = prepare_replacement_v2(
        fixture_identity(),
        main_pin(3, 2),
        &old,
        &current,
        &candidate,
    )
    .unwrap();
    let selected = plan.candidate_chain();
    assert!(std::ptr::eq(selected, &candidate));
    assert_eq!(
        selected.ledger().membership(),
        candidate.ledger().membership()
    );
    for id in [fixture_hash("c_member_id"), fixture_hash("seed0_id")] {
        assert_eq!(
            selected.ledger().membership().record(id),
            candidate.ledger().membership().record(id)
        );
        assert_ne!(
            selected.ledger().membership().record(id),
            current.ledger().membership().record(id)
        );
    }
    assert_eq!(
        selected.ledger().membership().retained_approval_count(),
        candidate.ledger().membership().retained_approval_count()
    );
    assert_eq!(
        selected.ledger().membership().total_activations(),
        candidate.ledger().membership().total_activations()
    );
    assert_eq!(plan.candidate_checkpoint().generation, 3);
    assert_eq!(plan.common_ancestor(), main_expectation(1));
}

#[test]
fn multi_block_extension_increments_generation_once() {
    let current = main_chain(0);
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    let plan = prepare_replacement_v2(
        fixture_identity(),
        main_pin(0, 0),
        &old,
        &current,
        &candidate,
    )
    .unwrap();
    assert_eq!(plan.relation(), BranchRelationV2::CandidateExtendsCurrent);
    assert_eq!(plan.common_ancestor(), main_expectation(0));
    assert_eq!(plan.candidate_checkpoint(), main_pin(4, 1));
}

#[test]
fn one_child_extension_advances_existing_generation() {
    let current = main_chain(1);
    let candidate = main_chain(2);
    let old = selection(&current, 1);
    let plan = prepare_replacement_v2(
        fixture_identity(),
        main_pin(1, 1),
        &old,
        &current,
        &candidate,
    )
    .unwrap();
    assert_eq!(plan.candidate_checkpoint(), main_pin(2, 2));
    assert_eq!(plan.common_ancestor(), main_expectation(1));
}

#[test]
fn initialization_above_genesis_keeps_generation_independent_of_height() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    let plan = prepare_replacement_v2(
        fixture_identity(),
        main_pin(3, 0),
        &old,
        &current,
        &candidate,
    )
    .unwrap();
    assert_eq!(plan.current_checkpoint(), main_pin(3, 0));
    assert_eq!(plan.candidate_checkpoint(), main_pin(4, 1));
}

#[test]
fn generation_can_reach_but_not_exceed_new_height() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 3);
    let plan = prepare_replacement_v2(
        fixture_identity(),
        main_pin(3, 3),
        &old,
        &current,
        &candidate,
    )
    .unwrap();
    assert_eq!(plan.candidate_checkpoint(), main_pin(4, 4));
}

fn assert_not_preferred(current: &ChainV2, candidate: &ChainV2) {
    let before = current.clone();
    let candidate_before = candidate.clone();
    let old = selection(current, 0);
    assert_eq!(
        prepare_replacement_v2(
            fixture_identity(),
            old.checkpoint(),
            &old,
            current,
            candidate
        )
        .unwrap_err(),
        Error::CandidateNotPreferred
    );
    assert_eq!(current, &before);
    assert_eq!(candidate, &candidate_before);
    assert_eq!(old, selection(current, 0));
}

#[test]
fn identical_history_is_not_a_new_publication() {
    let current = main_chain(3);
    assert_not_preferred(&current, &current);
}

#[test]
fn divergent_ties_reject_in_both_directions() {
    let current = main_chain(3);
    let candidate = fork_chain();
    assert_not_preferred(&current, &candidate);
    assert_not_preferred(&candidate, &current);
}

#[test]
fn shorter_prefix_cannot_roll_back_current() {
    assert_not_preferred(&main_chain(4), &main_chain(1));
}

#[test]
fn shorter_divergent_branch_cannot_roll_back_current() {
    assert_not_preferred(&main_chain(4), &fork_chain());
}

#[test]
fn every_independent_current_checkpoint_field_is_required() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    let expected = main_pin(3, 0);
    let mut mismatches = [expected; 5];
    mismatches[0].chain.config_hash = Hash256::default();
    mismatches[1].chain.genesis_hash = Hash256::default();
    mismatches[2].chain.height = 2;
    mismatches[3].chain.tip_hash = Hash256::default();
    mismatches[4].chain.ledger_root = Hash256::default();
    for wrong in mismatches {
        assert_eq!(
            prepare_replacement_v2(fixture_identity(), wrong, &old, &current, &candidate)
                .unwrap_err(),
            Error::WrongCurrentCheckpoint
        );
    }
}

#[test]
fn selector_from_different_accepted_branch_is_rejected() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let foreign = selection(&fork_chain(), 0);
    assert_eq!(
        prepare_replacement_v2(
            fixture_identity(),
            main_pin(3, 0),
            &foreign,
            &current,
            &candidate
        )
        .unwrap_err(),
        Error::WrongCurrentSelection
    );
}

#[test]
fn independently_retained_generation_is_required() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 1);
    for generation in [0, 2, u64::MAX] {
        assert_eq!(
            prepare_replacement_v2(
                fixture_identity(),
                main_pin(3, generation),
                &old,
                &current,
                &candidate
            )
            .unwrap_err(),
            Error::WrongCurrentSelection
        );
    }
}

#[test]
fn wrong_independent_network_configuration_is_rejected_first() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    let mut identity = fixture_identity();
    identity.config_hash = Hash256::default();
    assert_eq!(
        prepare_replacement_v2(identity, main_pin(2, 0), &old, &current, &candidate).unwrap_err(),
        Error::Comparison(ForkChoiceErrorV2::WrongConfiguration {
            branch: BranchSideV2::Current
        })
    );
}

#[test]
fn wrong_independent_network_genesis_is_rejected_first() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    let mut identity = fixture_identity();
    identity.genesis_hash = Hash256::default();
    assert_eq!(
        prepare_replacement_v2(identity, main_pin(2, 0), &old, &current, &candidate).unwrap_err(),
        Error::Comparison(ForkChoiceErrorV2::WrongGenesis {
            branch: BranchSideV2::Current
        })
    );
}

#[test]
fn foreign_candidate_genesis_rejects_before_current_pin_or_score() {
    let current = main_chain(3);
    let candidate = alternate_genesis();
    assert!(candidate.height() < current.height());
    let old = selection(&current, 0);
    assert_eq!(
        prepare_replacement_v2(
            fixture_identity(),
            main_pin(2, 0),
            &old,
            &current,
            &candidate
        )
        .unwrap_err(),
        Error::Comparison(ForkChoiceErrorV2::WrongGenesis {
            branch: BranchSideV2::Candidate
        })
    );
}

#[test]
fn foreign_current_genesis_is_rejected() {
    let current = alternate_genesis();
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    assert_eq!(
        prepare_replacement_v2(
            fixture_identity(),
            old.checkpoint(),
            &old,
            &current,
            &candidate
        )
        .unwrap_err(),
        Error::Comparison(ForkChoiceErrorV2::WrongGenesis {
            branch: BranchSideV2::Current
        })
    );
}

#[test]
fn current_endpoint_is_checked_before_wrong_selector_generation() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let old = selection(&current, 1);
    assert_eq!(
        prepare_replacement_v2(
            fixture_identity(),
            main_pin(2, 0),
            &old,
            &current,
            &candidate
        )
        .unwrap_err(),
        Error::WrongCurrentCheckpoint
    );
}

#[test]
fn checksum_correct_metadata_must_match_complete_accepted_current() {
    let current = main_chain(3);
    let candidate = main_chain(4);
    let expected = main_pin(3, 0);
    let old = selection(&current, 0);
    let mut variants = [old.encode(); 3];
    let length = old.snapshot().manifest_len() + 1;
    variants[0][172..176].copy_from_slice(&length.to_le_bytes());
    let bytes = old.snapshot().blocks_bytes() + 1;
    variants[1][176..184].copy_from_slice(&bytes.to_le_bytes());
    variants[2][184] ^= 1;
    for mut bytes in variants {
        rechecksum(&mut bytes);
        // Structural decoding and all external endpoint pins can still pass.
        let forged = ReplacementSelectionV2::decode_for(&bytes, expected).unwrap();
        assert_eq!(forged.checkpoint(), old.checkpoint());
        assert_ne!(forged, old);
        assert_eq!(
            prepare_replacement_v2(fixture_identity(), expected, &forged, &current, &candidate)
                .unwrap_err(),
            Error::CurrentMetadataMismatch
        );
    }
}

#[test]
fn metadata_mismatch_is_not_hidden_by_nonpreferred_candidate() {
    let current = main_chain(3);
    let expected = main_pin(3, 0);
    let mut bytes = selection(&current, 0).encode();
    bytes[184] ^= 1;
    rechecksum(&mut bytes);
    let forged = ReplacementSelectionV2::decode_for(&bytes, expected).unwrap();
    assert_eq!(
        prepare_replacement_v2(fixture_identity(), expected, &forged, &current, &current)
            .unwrap_err(),
        Error::CurrentMetadataMismatch
    );
}

#[test]
fn prepared_records_round_trip_only_with_their_independent_checkpoints() {
    let current = main_chain(1);
    let candidate = main_chain(4);
    let old = selection(&current, 0);
    let plan = prepare_replacement_v2(
        fixture_identity(),
        main_pin(1, 0),
        &old,
        &current,
        &candidate,
    )
    .unwrap();
    let next = plan.candidate_selection();
    assert_eq!(
        ReplacementSelectionV2::decode_for(&next.encode(), main_pin(4, 1)).unwrap(),
        *next
    );
    assert!(ReplacementSelectionV2::decode_for(&next.encode(), main_pin(1, 0)).is_err());
    next.verify_manifest_bytes(&candidate.manifest().encode().unwrap())
        .unwrap();
}
