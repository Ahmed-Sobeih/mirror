//! Parent reconciliation and cumulative local work using accepted synthetic chains.

mod support_pool_v2;

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2, NetworkManifestV2};
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::{BlockHeaderV2, BlockV2, TransferErrorV2, UnifiedOperationErrorV2};
use mirror_crypto::Hash256;
use mirror_node::v2::pool::{
    PendingPoolErrorV2 as Error, PendingPoolV2 as Pool, PoolRemovalReasonV2 as Reason,
};
use mirror_personhood::v2::LimitsV2;
use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2};
use support_pool_v2::*;

/// Deliberately excludes the monotonic work meter: failed work can be charged
/// while every semantic retained-state field remains unchanged.
#[derive(Debug, PartialEq, Eq)]
struct RetainedState {
    parent: ChainReplayExpectationV2,
    revision: u64,
    operations: Vec<Operation>,
    ids: Vec<Hash256>,
    bytes: usize,
}

fn retained_state(pool: &Pool) -> RetainedState {
    RetainedState {
        parent: pool.parent(),
        revision: pool.revision(),
        operations: pool.operations().to_vec(),
        ids: pool.ids().to_vec(),
        bytes: pool.encoded_bytes(),
    }
}

fn retain(pool: &mut Pool, chain: &ChainV2, operation: Operation) {
    assert!(pool.admit(chain, &operation.encode()).is_ok());
    assert!(pool.ids().contains(&operation.id()));
}

fn sequence_error(expected: u64, got: u64) -> Error {
    Error::Ledger(LedgerErrorV2::Operation {
        index: 0,
        error: LedgerOperationErrorV2::AccountSequenceMismatch { expected, got },
    })
}

fn context(error: TransferErrorV2) -> Reason {
    Reason::Context(UnifiedOperationErrorV2::Transfer(error))
}

/// Another fully accepted genesis under the same configuration, not an arbitrary
/// parent view or an unchecked replacement of the first chain's manifest pin.
fn different_genesis(chain: &ChainV2) -> ChainV2 {
    let config = chain.config();
    let mut bytes = chain.genesis().header().encode();
    bytes[152..].copy_from_slice(&(chain.genesis().header().nonce() + 1).to_le_bytes());
    let header = BlockHeaderV2::decode(&bytes).unwrap();
    let result = mine_header_v2(header, config.parameters().pow_bits, 10_000).unwrap();
    let genesis = BlockV2::new(config, result.header, &[]).unwrap();
    let proofs = [
        config.sign_seed(0, &key(1)).unwrap(),
        config.sign_seed(1, &key(2)).unwrap(),
    ];
    let manifest = NetworkManifestV2::new(config, &proofs, genesis.hash()).unwrap();
    ChainV2::from_genesis(&manifest, genesis).unwrap()
}

#[test]
fn unchanged_parent_reconciliation_finds_inclusion_before_the_tip_block() {
    let mut chain = fresh();
    let included = transfer(&chain, 1, 2, 0, 10);
    append(&mut chain, &[included]);
    append(&mut chain, &[]);
    let accepted = chain.clone();
    let mut pool = Pool::new(&chain, pool_limits()).unwrap();

    // Admission is contextual. It deliberately has no global history index.
    retain(&mut pool, &chain, included);
    assert_eq!(pool.evaluate(&chain, 1).err(), Some(sequence_error(1, 0)));
    let revision = pool.revision();
    let report = pool.reconcile(&chain).unwrap();
    assert_eq!(report.previous, chain.checkpoint());
    assert_eq!(report.selected, chain.checkpoint());
    assert_eq!(report.revision, revision + 1);
    assert_eq!(report.removals.len(), 1);
    assert_eq!(report.removals[0].id, included.id());
    assert_eq!(report.removals[0].reason, Reason::Included);
    assert!(pool.is_empty());
    assert_eq!(pool.encoded_bytes(), 0);
    assert_eq!(chain, accepted);
}

#[test]
fn append_reconciliation_removes_exact_ids_and_retains_consumed_sequence_conflicts() {
    let original = fresh();
    let included = transfer(&original, 1, 2, 0, 10);
    let conflicting = transfer(&original, 1, 3, 0, 11);
    let mut pool = Pool::new(&original, pool_limits()).unwrap();
    retain(&mut pool, &original, included);
    retain(&mut pool, &original, conflicting);
    assert_eq!(pool.work_used(), 977); // 488 + 489, with one retained-ID comparison.
    let mut selected = original.clone();
    append(&mut selected, &[included]);
    let accepted = selected.clone();

    assert_eq!(
        pool.evaluate(&selected, 1).err(),
        Some(Error::ParentChanged)
    );
    assert_eq!(pool.work_used(), 977);
    let report = pool.reconcile(&selected).unwrap();
    assert_eq!(report.previous, original.checkpoint());
    assert_eq!(report.selected, selected.checkpoint());
    assert_eq!(report.removals.len(), 1);
    assert_eq!(report.removals[0].id, included.id());
    assert_eq!(report.removals[0].reason, Reason::Included);
    assert_eq!(pool.operations(), &[conflicting]);
    assert_eq!(pool.ids(), &[conflicting.id()]);
    assert_eq!(pool.encoded_bytes(), 243);
    // H=2, W=583, O=1, N=2, R=486: the full reserved charge is 1081.
    assert_eq!(pool.work_used(), 2_058);
    assert_eq!(
        pool.evaluate(&selected, 1).err(),
        Some(sequence_error(1, 0))
    );
    assert_eq!(selected, accepted);
    assert_eq!(original.height(), 0);
    assert_eq!(original.ledger().account(address(1)).balance(), 100);
}

#[test]
fn reconciliation_reports_fifo_removals_and_inclusion_takes_precedence_over_expiry() {
    let original = fresh();
    let expired = transfer_header(1, header(&original, 1, 9, 1), 3, 1, 0);
    let included = transfer_header(1, header(&original, 1, 0, 1), 2, 10, 0);
    let survivor = transfer(&original, 1, 3, 1, 5);
    let mut pool = Pool::new(&original, pool_limits()).unwrap();
    for operation in [expired, included, survivor] {
        retain(&mut pool, &original, operation);
    }
    let mut selected = original.clone();
    append(&mut selected, &[included]);
    let accepted = selected.clone();

    let report = pool.reconcile(&selected).unwrap();
    assert_eq!(report.removals.len(), 2);
    assert_eq!(report.removals[0].id, expired.id());
    assert_eq!(report.removals[0].reason, context(TransferErrorV2::Expired));
    assert_eq!(report.removals[1].id, included.id());
    assert_eq!(report.removals[1].reason, Reason::Included);
    assert_eq!(pool.operations(), &[survivor]);
    assert_eq!(
        pool.evaluate(&selected, 1).unwrap().operations(),
        &[survivor]
    );
    assert_eq!(selected, accepted);
}

#[test]
fn same_height_replacement_invalidates_the_exact_anchor_and_old_batch() {
    let genesis = fresh();
    let mut left = genesis.clone();
    let mut right = genesis.clone();
    left.append(next(&genesis, 20, &[])).unwrap();
    right.append(next(&genesis, 21, &[])).unwrap();
    assert_eq!(left.height(), right.height());
    assert_ne!(left.tip_hash(), right.tip_hash());
    let left_before = left.clone();
    let right_before = right.clone();
    let operation = transfer(&left, 1, 2, 0, 1);
    let mut pool = Pool::new(&left, pool_limits()).unwrap();
    retain(&mut pool, &left, operation);
    let batch = pool.evaluate(&left, 1).unwrap();

    assert!(!pool.batch_is_current(&right, &batch));
    assert_eq!(
        pool.admit(&right, &operation.encode()).err(),
        Some(Error::ParentChanged)
    );
    let report = pool.reconcile(&right).unwrap();
    assert_eq!(report.previous, left.checkpoint());
    assert_eq!(report.selected, right.checkpoint());
    assert_eq!(report.removals.len(), 1);
    assert_eq!(report.removals[0].id, operation.id());
    assert_eq!(
        report.removals[0].reason,
        context(TransferErrorV2::WrongAnchorHash)
    );
    assert!(!pool.batch_is_current(&left, &batch));
    assert!(!pool.batch_is_current(&right, &batch));
    assert_eq!(pool.parent(), right.checkpoint());
    assert!(pool.is_empty());
    assert_eq!(left, left_before);
    assert_eq!(right, right_before);
}

#[test]
fn shorter_replacement_drops_a_window_that_is_now_in_the_future() {
    let genesis = fresh();
    let mut longer = genesis.clone();
    append(&mut longer, &[]);
    append(&mut longer, &[]);
    let mut signed_header = header(&genesis, 1, 0, 8);
    signed_header.valid_from = 3;
    let operation = transfer_header(1, signed_header, 2, 1, 0);
    let mut pool = Pool::new(&longer, pool_limits()).unwrap();
    retain(&mut pool, &longer, operation);
    let before = longer.clone();

    let report = pool.reconcile(&genesis).unwrap();
    assert_eq!(report.previous, longer.checkpoint());
    assert_eq!(report.selected, genesis.checkpoint());
    assert_eq!(report.removals.len(), 1);
    assert_eq!(
        report.removals[0].reason,
        context(TransferErrorV2::NotYetValid)
    );
    assert!(pool.is_empty());
    assert_eq!(pool.parent(), genesis.checkpoint());
    assert_eq!(longer, before);
}

#[test]
fn unchanged_anchor_hash_does_not_exempt_an_overage_anchor() {
    let original = chain_with(
        LimitsV2 {
            max_anchor_age: 2,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let operation = transfer(&original, 1, 2, 0, 1);
    let mut pool = Pool::new(&original, pool_limits()).unwrap();
    retain(&mut pool, &original, operation);
    let mut selected = original.clone();
    append(&mut selected, &[]);
    append(&mut selected, &[]);
    let before = selected.clone();

    let report = pool.reconcile(&selected).unwrap();
    assert_eq!(report.removals.len(), 1);
    assert_eq!(
        report.removals[0].reason,
        context(TransferErrorV2::AnchorTooOld)
    );
    assert!(pool.is_empty());
    assert_eq!(selected, before);
}

fn assert_wrong_network_refuses_every_contextual_path(original: &ChainV2, other: &ChainV2) {
    let operation = transfer(original, 1, 2, 0, 1);
    let mut pool = Pool::new(original, pool_limits()).unwrap();
    retain(&mut pool, original, operation);
    let batch = pool.evaluate(original, 1).unwrap();
    let state = retained_state(&pool);
    let work = pool.work_used();
    let original_before = original.clone();
    let other_before = other.clone();

    assert_eq!(
        pool.admit(other, &operation.encode()).err(),
        Some(Error::WrongNetwork)
    );
    assert_eq!(pool.evaluate(other, 1).err(), Some(Error::WrongNetwork));
    assert_eq!(pool.reconcile(other).err(), Some(Error::WrongNetwork));
    assert!(!pool.batch_is_current(other, &batch));
    assert!(pool.batch_is_current(original, &batch));
    assert_eq!(retained_state(&pool), state);
    assert_eq!(pool.work_used(), work);
    assert_eq!(*original, original_before);
    assert_eq!(*other, other_before);
}

#[test]
fn another_configuration_is_not_a_reconciliation_target() {
    let original = fresh();
    let other = chain_with(LimitsV2::default(), &[(1, 100), (2, 0)], 1, 32);
    assert_ne!(
        original.checkpoint().config_hash,
        other.checkpoint().config_hash
    );
    assert_wrong_network_refuses_every_contextual_path(&original, &other);
}

#[test]
fn matching_configuration_does_not_authorize_another_genesis_pin() {
    let original = fresh();
    let other = different_genesis(&original);
    assert_eq!(
        original.checkpoint().config_hash,
        other.checkpoint().config_hash
    );
    assert_ne!(
        original.checkpoint().genesis_hash,
        other.checkpoint().genesis_hash
    );
    assert_wrong_network_refuses_every_contextual_path(&original, &other);
}

#[test]
fn abandoning_an_including_branch_never_resurrects_removed_input() {
    let original = fresh();
    let operation = transfer(&original, 1, 2, 0, 10);
    let mut pool = Pool::new(&original, pool_limits()).unwrap();
    retain(&mut pool, &original, operation);
    let mut included = original.clone();
    append(&mut included, &[operation]);
    let accepted = included.clone();
    assert_eq!(
        pool.reconcile(&included).unwrap().removals[0].reason,
        Reason::Included
    );
    let work = pool.work_used();

    let report = pool.reconcile(&original).unwrap();
    assert!(report.removals.is_empty());
    assert!(pool.is_empty());
    assert_eq!(pool.parent(), original.checkpoint());
    assert_eq!(pool.work_used(), work + 1); // Empty pool skips the history scan.
    retain(&mut pool, &original, operation); // Explicit caller resubmission only.
    assert_eq!(
        pool.evaluate(&original, 1).unwrap().operations(),
        &[operation]
    );
    assert_eq!(included, accepted);
}

#[test]
fn successful_unchanged_reconciliation_invalidates_an_owned_batch_observation() {
    let chain = fresh();
    let before = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 1);
    let mut pool = Pool::new(&chain, pool_limits()).unwrap();
    retain(&mut pool, &chain, operation);
    let batch = pool.evaluate(&chain, 1).unwrap();
    assert!(pool.batch_is_current(&chain, &batch));

    let report = pool.reconcile(&chain).unwrap();
    assert!(report.removals.is_empty());
    assert_eq!(report.previous, report.selected);
    assert_eq!(pool.operations(), batch.operations());
    assert_eq!(pool.revision(), batch.revision() + 1);
    assert!(!pool.batch_is_current(&chain, &batch));
    let new_batch = pool.evaluate(&chain, 1).unwrap();
    assert!(pool.batch_is_current(&chain, &new_batch));
    assert_eq!(chain, before);
}

#[test]
fn equal_parent_and_revision_cannot_make_a_different_pools_prefix_current() {
    let chain = fresh();
    let before = chain.clone();
    let first = transfer(&chain, 1, 2, 0, 1);
    let second = transfer(&chain, 1, 3, 0, 2);
    let mut left = Pool::new(&chain, pool_limits()).unwrap();
    let mut right = Pool::new(&chain, pool_limits()).unwrap();
    retain(&mut left, &chain, first);
    retain(&mut right, &chain, second);
    let batch = left.evaluate(&chain, 1).unwrap();

    assert_eq!(left.parent(), right.parent());
    assert_eq!(left.revision(), right.revision());
    assert!(left.batch_is_current(&chain, &batch));
    assert!(!right.batch_is_current(&chain, &batch));
    assert_eq!(right.work_used(), 488); // Observation is not validation work.
    assert_eq!(chain, before);
}

#[test]
fn reconciliation_budget_failure_keeps_old_parent_binding_and_all_retained_state() {
    let original = fresh();
    let operation = transfer(&original, 1, 2, 0, 1);
    let mut limits = pool_limits();
    limits.max_work_units = 1_072; // 488 admission + 585 reconciliation, minus one.
    let mut pool = Pool::new(&original, limits).unwrap();
    retain(&mut pool, &original, operation);
    let state = retained_state(&pool);
    let mut selected = original.clone();
    append(&mut selected, &[]);
    let before = selected.clone();

    assert_eq!(pool.reconcile(&selected).err(), Some(Error::WorkLimit));
    assert_eq!(retained_state(&pool), state);
    assert_eq!(pool.work_used(), 488);
    assert_eq!(
        pool.admit(&selected, &operation.encode()).err(),
        Some(Error::ParentChanged)
    );
    assert_eq!(
        pool.evaluate(&selected, 1).err(),
        Some(Error::ParentChanged)
    );
    assert_eq!(pool.work_used(), 488);
    assert!(pool.evaluate(&original, 1).is_ok());
    assert_eq!(pool.work_used(), 750);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(selected, before);
}

#[test]
fn identification_charge_is_retained_when_contextual_reservation_cannot_fit() {
    let chain = fresh();
    let before = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 1);
    let mut limits = pool_limits();
    limits.max_work_units = 487; // First identification fits, full admission does not.
    let mut pool = Pool::new(&chain, limits).unwrap();
    let state = retained_state(&pool);

    assert_eq!(
        pool.admit(&chain, &operation.encode()).err(),
        Some(Error::WorkLimit)
    );
    assert_eq!(pool.work_used(), 244);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(
        pool.admit(&chain, &operation.encode()).err(),
        Some(Error::WorkLimit)
    );
    assert_eq!(pool.work_used(), 244); // The next identification reservation cannot fit.
    assert_eq!(retained_state(&pool), state);
    assert_eq!(chain, before);
}

#[test]
fn rejected_signature_consumes_both_admission_charges_and_exhaustion_is_sticky() {
    let chain = fresh();
    let before = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 1);
    let mut bytes = operation.encode();
    bytes[242] ^= 0x80;
    let mut limits = pool_limits();
    limits.max_work_units = 488;
    let mut pool = Pool::new(&chain, limits).unwrap();
    let state = retained_state(&pool);

    assert!(matches!(
        pool.admit(&chain, &bytes),
        Err(Error::Operation(UnifiedOperationErrorV2::Transfer(
            TransferErrorV2::Crypto(_)
        )))
    ));
    assert_eq!(pool.work_used(), 488);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(pool.remove(operation.id()), Ok(false));
    assert_eq!(pool.clear(), Ok(0));
    assert_eq!(
        pool.admit(&chain, &operation.encode()).err(),
        Some(Error::WorkLimit)
    );
    assert_eq!(pool.work_used(), 488);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(chain, before);
}

#[test]
fn malformed_duplicate_and_full_refusals_charge_only_work_they_start() {
    let chain = fresh();
    let before = chain.clone();
    let first = transfer(&chain, 1, 2, 0, 1);
    let second = transfer(&chain, 1, 3, 0, 2);
    let mut limits = pool_limits();
    limits.max_entries = 1;
    limits.max_batch_operations = 1;
    let mut pool = Pool::new(&chain, limits).unwrap();
    retain(&mut pool, &chain, first);
    let state = retained_state(&pool);

    assert!(pool.admit(&chain, &first.encode()).is_ok());
    assert_eq!(pool.work_used(), 733); // Duplicate: 1 + 243 + one retained entry.
    assert_eq!(
        pool.admit(&chain, &second.encode()).err(),
        Some(Error::EntryLimit)
    );
    assert_eq!(pool.work_used(), 978); // Full refusal does not validate the signature.
    assert!(matches!(
        pool.admit(&chain, &[2, 0, 0]),
        Err(Error::Operation(_))
    ));
    assert_eq!(pool.work_used(), 983); // Malformed identification: 1 + 3 + 1.
    assert_eq!(
        pool.admit(&chain, &[0; 292]).err(),
        Some(Error::InputTooLarge)
    );
    assert_eq!(pool.work_used(), 983); // Size preflight precedes every reservation.
    assert_eq!(retained_state(&pool), state);
    assert_eq!(chain, before);
}

#[test]
fn successful_evaluation_fits_the_exact_work_limit_and_cannot_be_reused_for_free() {
    let chain = fresh();
    let before = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 1);
    let mut limits = pool_limits();
    limits.max_work_units = 750; // 488 admission + 262 one ordered stage.
    let mut pool = Pool::new(&chain, limits).unwrap();
    retain(&mut pool, &chain, operation);
    let state = retained_state(&pool);

    let batch = pool.evaluate(&chain, 1).unwrap();
    assert_eq!(batch.encoded_block_bytes(), 415);
    assert_eq!(pool.work_used(), 750);
    assert!(pool.batch_is_current(&chain, &batch));
    assert_eq!(pool.evaluate(&chain, 1).err(), Some(Error::WorkLimit));
    assert_eq!(pool.work_used(), 750);
    assert_eq!(retained_state(&pool), state);
    assert!(pool.batch_is_current(&chain, &batch));
    assert_eq!(chain, before);
}

#[test]
fn repeated_failed_execution_consumes_full_state_work_without_retained_mutation() {
    let chain = fresh();
    let before = chain.clone();
    let overspend = transfer(&chain, 1, 2, 0, 101);
    let mut limits = pool_limits();
    limits.max_work_units = 1_012; // 488 admission plus two complete 262-unit attempts.
    let mut pool = Pool::new(&chain, limits).unwrap();
    retain(&mut pool, &chain, overspend);
    let state = retained_state(&pool);
    let error = Error::Ledger(LedgerErrorV2::Operation {
        index: 0,
        error: LedgerOperationErrorV2::InsufficientBalance {
            available: 100,
            required: 101,
        },
    });

    assert_eq!(pool.evaluate(&chain, 1).err(), Some(error));
    assert_eq!(pool.work_used(), 750);
    assert_eq!(pool.evaluate(&chain, 1).err(), Some(error));
    assert_eq!(pool.work_used(), 1_012);
    assert_eq!(pool.evaluate(&chain, 1).err(), Some(Error::WorkLimit));
    assert_eq!(pool.work_used(), 1_012);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(chain, before);
}

#[test]
fn evaluation_charge_includes_actual_accounts_membership_indexes_and_retained_approvals() {
    let mut chain = fresh();
    let funding = transfer(&chain, 1, 3, 0, 10);
    let enrollment = enroll(&chain, 3, 8);
    let enrollment_id = match enrollment {
        Operation::Membership(operation) => operation.enrollment_id().unwrap(),
        _ => unreachable!(),
    };
    let approval = approve(&chain, 1, 0, enrollment_id, 8);
    append(&mut chain, &[funding, enrollment, approval]);
    assert_eq!(chain.config().bootstrap().len(), 2);
    assert_eq!(chain.config().allocations().len(), 2);
    assert_eq!(chain.ledger().account_count(), 3);
    assert_eq!(chain.ledger().membership().record_count(), 3);
    assert_eq!(chain.ledger().membership().retained_approval_count(), 1);
    let before = chain.clone();
    let operation = transfer(&chain, 1, 4, 1, 1);

    // S = 2 + 2 + 3 + 4*3 + 1 = 20; one stage therefore costs 268, not 262.
    let mut limits = pool_limits();
    limits.max_work_units = 755;
    let mut short = Pool::new(&chain, limits).unwrap();
    retain(&mut short, &chain, operation);
    assert_eq!(short.evaluate(&chain, 1).err(), Some(Error::WorkLimit));
    assert_eq!(short.work_used(), 488);
    limits.max_work_units = 756;
    let mut exact = Pool::new(&chain, limits).unwrap();
    retain(&mut exact, &chain, operation);
    assert!(exact.evaluate(&chain, 1).is_ok());
    assert_eq!(exact.work_used(), 756);
    assert_eq!(chain, before);
}

#[test]
fn every_unchanged_parent_revalidation_is_charged_and_advances_observation_revision() {
    let chain = fresh();
    let before = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 1);
    let mut limits = pool_limits();
    limits.max_work_units = 1_320; // 488 admission + two complete 416-unit reconciliations.
    let mut pool = Pool::new(&chain, limits).unwrap();
    retain(&mut pool, &chain, operation);

    for (expected_work, expected_revision) in [(904, 2), (1_320, 3)] {
        let report = pool.reconcile(&chain).unwrap();
        assert!(report.removals.is_empty());
        assert_eq!(report.revision, expected_revision);
        assert_eq!(pool.work_used(), expected_work);
        assert_eq!(pool.operations(), &[operation]);
    }
    let state = retained_state(&pool);
    assert_eq!(pool.reconcile(&chain).err(), Some(Error::WorkLimit));
    assert_eq!(pool.work_used(), 1_320);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(chain, before);
}

#[test]
fn empty_reconciliation_has_a_one_unit_charge_and_cannot_refill_an_exhausted_owner() {
    let original = fresh();
    let mut selected = original.clone();
    append(&mut selected, &[]);
    let before = selected.clone();
    let mut limits = pool_limits();
    limits.max_work_units = 1;
    let mut pool = Pool::new(&original, limits).unwrap();

    let report = pool.reconcile(&selected).unwrap();
    assert_eq!(report.previous, original.checkpoint());
    assert_eq!(report.selected, selected.checkpoint());
    assert!(report.removals.is_empty());
    assert_eq!(pool.work_used(), 1);
    assert_eq!(pool.revision(), 1);
    let state = retained_state(&pool);
    assert_eq!(pool.reconcile(&original).err(), Some(Error::WorkLimit));
    assert_eq!(retained_state(&pool), state);
    assert_eq!(pool.clear(), Ok(0));
    assert_eq!(pool.work_used(), 1);
    assert_eq!(selected, before);
}

#[test]
fn removing_and_clearing_retained_entries_never_refunds_validation_work_or_rebinds() {
    let chain = fresh();
    let before = chain.clone();
    let first = transfer(&chain, 1, 2, 0, 1);
    let second = transfer(&chain, 1, 3, 1, 2);
    let mut limits = pool_limits();
    limits.max_work_units = 977;
    let mut pool = Pool::new(&chain, limits).unwrap();
    retain(&mut pool, &chain, first);
    retain(&mut pool, &chain, second);
    assert_eq!(pool.work_used(), 977);

    assert_eq!(pool.remove(first.id()), Ok(true));
    assert_eq!(pool.work_used(), 977);
    assert_eq!(pool.operations(), &[second]);
    assert_eq!(pool.clear(), Ok(1));
    assert_eq!(pool.work_used(), 977);
    assert!(pool.is_empty());
    assert_eq!(pool.encoded_bytes(), 0);
    assert_eq!(pool.parent(), chain.checkpoint());
    let state = retained_state(&pool);
    assert_eq!(pool.remove(first.id()), Ok(false));
    assert_eq!(pool.clear(), Ok(0));
    assert_eq!(
        pool.admit(&chain, &first.encode()).err(),
        Some(Error::WorkLimit)
    );
    assert_eq!(pool.reconcile(&chain).err(), Some(Error::WorkLimit));
    assert_eq!(pool.work_used(), 977);
    assert_eq!(retained_state(&pool), state);
    assert_eq!(chain, before);
}
