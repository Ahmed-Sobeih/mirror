//! Local FIFO admission and ordered execution under actual accepted V2 chains.
//! All keys, allocations and evidence are public synthetic development fixtures.
mod support_pool_v2;

use mirror_chain::v2::{ChainReplayExpectationV2, ChainV2};
use mirror_core::v2::{BlockErrorV2, TransferErrorV2, UnifiedOperationErrorV2};
use mirror_crypto::Hash256;
use mirror_node::v2::pool::{
    PendingPoolErrorV2 as Error, PendingPoolLimitsV2, PendingPoolV2 as Pool,
    PoolAdmissionV2 as Admission,
};
use mirror_personhood::v2::{
    LimitsV2, MembershipActionV2, MembershipStatusV2, OperationErrorV2,
    SignedMembershipOperationV2, StateErrorV2,
};
use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2};
use support_pool_v2::*;

#[derive(Debug, PartialEq, Eq)]
struct Retained {
    parent: ChainReplayExpectationV2,
    revision: u64,
    bytes: usize,
    operations: Vec<Operation>,
    ids: Vec<Hash256>,
}

fn retained(pool: &Pool) -> Retained {
    assert_eq!(pool.len(), pool.operations().len());
    assert_eq!(pool.len(), pool.ids().len());
    assert_eq!(pool.is_empty(), pool.len() == 0);
    Retained {
        parent: pool.parent(),
        revision: pool.revision(),
        bytes: pool.encoded_bytes(),
        operations: pool.operations().to_vec(),
        ids: pool.ids().to_vec(),
    }
}

fn new_pool(chain: &ChainV2) -> Pool {
    Pool::new(chain, pool_limits()).unwrap()
}

fn retain(pool: &mut Pool, chain: &ChainV2, operations: &[Operation]) {
    for operation in operations {
        assert_eq!(
            pool.admit(chain, &operation.encode()),
            Ok(Admission::Retained { id: operation.id() })
        );
    }
}

fn evaluation_error(pool: &mut Pool, chain: &ChainV2, count: usize, expected: Error) {
    let before = retained(pool);
    let accepted = chain.clone();
    assert_eq!(pool.evaluate(chain, count).unwrap_err(), expected);
    assert_eq!(retained(pool), before);
    assert_eq!(chain, &accepted);
}

fn ledger_error(index: u32, error: LedgerOperationErrorV2) -> Error {
    Error::Ledger(LedgerErrorV2::Operation { index, error })
}

#[test]
fn constructor_checks_every_local_range_and_starts_with_no_validation_work() {
    let chain = fresh();
    let before = chain.clone();
    let maximum = pool_limits();
    let minimum = PendingPoolLimitsV2 {
        max_entries: 1,
        max_encoded_bytes: 1,
        max_batch_operations: 1,
        max_batch_bytes: 168,
        max_work_units: 1,
    };
    for limits in [minimum, maximum] {
        let pool = Pool::new(&chain, limits).unwrap();
        assert_eq!(pool.limits(), limits);
        assert_eq!(pool.parent(), chain.checkpoint());
        assert_eq!(pool.revision(), 0);
        assert_eq!(pool.work_used(), 0);
        assert_eq!(pool.encoded_bytes(), 0);
        assert!(pool.is_empty());
        assert!(pool.operations().is_empty());
        assert!(pool.ids().is_empty());
    }
    let invalid = [
        PendingPoolLimitsV2 {
            max_entries: 0,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_entries: 257,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_entries: usize::MAX,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_encoded_bytes: 0,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_encoded_bytes: 74_497,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_batch_operations: 0,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_batch_operations: 65,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_entries: 1,
            max_batch_operations: 2,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_batch_bytes: 167,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_batch_bytes: 19_049,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_work_units: 0,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_work_units: 67_108_865,
            ..maximum
        },
        PendingPoolLimitsV2 {
            max_work_units: u64::MAX,
            ..maximum
        },
    ];
    for limits in invalid {
        assert_eq!(Pool::new(&chain, limits).unwrap_err(), Error::InvalidLimits);
    }
    assert_eq!(chain, before);
}

#[test]
fn all_four_operation_kinds_retain_exact_signed_bytes_and_ids_in_arrival_order() {
    let chain = fresh();
    let before = chain.clone();
    let operations = [
        transfer(&chain, 1, 2, 0, 10),
        enroll(&chain, 3, 8),
        approve(&chain, 1, 0, tag(90), 8),
        withdraw(&chain, 2, 0),
    ];
    assert_eq!(
        operations.map(|operation| operation.encoded_len()),
        [243, 291, 227, 227]
    );
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &operations);
    assert_eq!(pool.operations(), operations);
    assert_eq!(pool.ids(), operations.map(|operation| operation.id()));
    for (stored, supplied) in pool.operations().iter().zip(operations) {
        assert_eq!(stored.encode(), supplied.encode());
    }
    assert_eq!(pool.encoded_bytes(), 988);
    assert_eq!(pool.revision(), 4);
    assert_eq!(chain, before);
}

#[test]
fn input_size_preflight_precedes_decode_and_an_insufficient_work_allowance() {
    let chain = fresh();
    let before = chain.clone();
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_work_units: 1,
            ..pool_limits()
        },
    )
    .unwrap();
    let state = retained(&pool);
    for bytes in [vec![0; 292], vec![2; 65_536]] {
        assert_eq!(pool.admit(&chain, &bytes), Err(Error::InputTooLarge));
        assert_eq!(pool.work_used(), 0);
        assert_eq!(retained(&pool), state);
    }
    assert_eq!(chain, before);
}

#[test]
fn structural_rejections_do_not_change_existing_entries_or_order() {
    let chain = fresh();
    let accepted = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 10);
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[operation]);
    let before = retained(&pool);
    let mut trailing = operation.encode();
    trailing.push(0);
    let cases = [
        (Vec::new(), UnifiedOperationErrorV2::InvalidLength),
        (vec![1, 0, 0], UnifiedOperationErrorV2::UnsupportedVersion),
        (vec![2, 0, 4], UnifiedOperationErrorV2::UnsupportedKind),
        (
            vec![2, 0, 0],
            UnifiedOperationErrorV2::Transfer(TransferErrorV2::InvalidLength),
        ),
        (
            trailing,
            UnifiedOperationErrorV2::Transfer(TransferErrorV2::InvalidLength),
        ),
        (
            vec![2, 0, 1],
            UnifiedOperationErrorV2::Membership(OperationErrorV2::InvalidLength),
        ),
    ];
    for (bytes, expected) in cases {
        assert_eq!(pool.admit(&chain, &bytes), Err(Error::Operation(expected)));
        assert_eq!(retained(&pool), before);
    }
    assert_eq!(chain, accepted);
}

#[test]
fn invalid_transfer_and_membership_signatures_never_retain_a_new_id() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[transfer(&chain, 1, 2, 0, 10)]);
    let before = retained(&pool);
    for operation in [transfer(&chain, 1, 2, 1, 11), enroll(&chain, 3, 8)] {
        let mut bytes = operation.encode();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(matches!(
            pool.admit(&chain, &bytes),
            Err(Error::Operation(UnifiedOperationErrorV2::Transfer(
                TransferErrorV2::Crypto(_)
            ))) | Err(Error::Operation(UnifiedOperationErrorV2::Membership(
                OperationErrorV2::Crypto(_)
            )))
        ));
        assert_eq!(retained(&pool), before);
    }
    assert_eq!(chain, accepted);
}

#[test]
fn transfer_context_rejects_future_wrong_anchor_wrong_context_and_unsupported_money() {
    let chain = fresh();
    let accepted = chain.clone();
    let base = header(&chain, 1, 0, 8);
    let mut future = base;
    future.valid_from = 2;
    let mut wrong_anchor = base;
    wrong_anchor.anchor_hash = tag(99);
    let mut not_past = base;
    not_past.anchor_height = 1;
    let mut wrong_context = base;
    wrong_context.config_hash = tag(99);
    let mut too_long = base;
    too_long.valid_until = 257;
    let mut inverted = base;
    inverted.valid_from = 3;
    inverted.valid_until = 2;
    let cases = [
        (
            transfer_header(1, future, 2, 1, 0),
            TransferErrorV2::NotYetValid,
        ),
        (
            transfer_header(1, wrong_anchor, 2, 1, 0),
            TransferErrorV2::WrongAnchorHash,
        ),
        (
            transfer_header(1, not_past, 2, 1, 0),
            TransferErrorV2::AnchorNotPast,
        ),
        (
            transfer_header(1, wrong_context, 2, 1, 0),
            TransferErrorV2::WrongContext,
        ),
        (
            transfer_header(1, too_long, 2, 1, 0),
            TransferErrorV2::WindowTooLong,
        ),
        (
            transfer_header(1, inverted, 2, 1, 0),
            TransferErrorV2::InvertedWindow,
        ),
        (
            transfer_header(1, base, 2, 0, 0),
            TransferErrorV2::ZeroAmount,
        ),
        (
            transfer_header(1, base, 2, 1, 1),
            TransferErrorV2::UnsupportedFee,
        ),
    ];
    let mut pool = new_pool(&chain);
    for (operation, expected) in cases {
        assert_eq!(
            pool.admit(&chain, &operation.encode()),
            Err(Error::Operation(UnifiedOperationErrorV2::Transfer(
                expected
            )))
        );
        assert!(pool.is_empty());
        assert_eq!(pool.encoded_bytes(), 0);
        assert_eq!(pool.revision(), 0);
    }
    assert_eq!(chain, accepted);
}

#[test]
fn admission_uses_selected_next_height_for_expiry_and_anchor_age() {
    let mut chain = fresh();
    let expired = transfer_header(1, header(&chain, 1, 0, 1), 2, 1, 0);
    append(&mut chain, &[]);
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    assert_eq!(
        pool.admit(&chain, &expired.encode()),
        Err(Error::Operation(UnifiedOperationErrorV2::Transfer(
            TransferErrorV2::Expired
        )))
    );
    assert!(pool.is_empty());
    assert_eq!(chain, accepted);

    let mut chain = chain_with(
        LimitsV2 {
            max_anchor_age: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let old_anchor = transfer(&chain, 1, 2, 0, 1);
    append(&mut chain, &[]);
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    assert_eq!(
        pool.admit(&chain, &old_anchor.encode()),
        Err(Error::Operation(UnifiedOperationErrorV2::Transfer(
            TransferErrorV2::AnchorTooOld
        )))
    );
    assert!(pool.is_empty());
    assert_eq!(chain, accepted);
}

#[test]
fn membership_admission_preserves_existing_context_checks() {
    let chain = fresh();
    let accepted = chain.clone();
    let base = header(&chain, 3, 0, 8);
    let action = MembershipActionV2::Enroll {
        applicant_nonce: tag(43),
        evidence_commitment: tag(53),
        evidence_profile: chain.config().parameters().evidence_profile,
    };
    let mut wrong_sequence = base;
    wrong_sequence.actor_sequence = 1;
    let mut future = base;
    future.valid_from = 2;
    let cases = [
        (
            wrong_sequence,
            action,
            OperationErrorV2::InvalidEnrollmentSequence,
        ),
        (future, action, OperationErrorV2::NotYetValid),
        (
            base,
            MembershipActionV2::Enroll {
                applicant_nonce: tag(43),
                evidence_commitment: tag(53),
                evidence_profile: tag(99),
            },
            OperationErrorV2::WrongEvidenceProfile,
        ),
        (
            base,
            MembershipActionV2::Enroll {
                applicant_nonce: Hash256::default(),
                evidence_commitment: tag(53),
                evidence_profile: chain.config().parameters().evidence_profile,
            },
            OperationErrorV2::ZeroCommitment,
        ),
    ];
    let mut pool = new_pool(&chain);
    for (header, action, expected) in cases {
        let operation = SignedMembershipOperationV2::sign(header, action, &key(3)).unwrap();
        assert_eq!(
            pool.admit(&chain, &operation.encode()),
            Err(Error::Operation(UnifiedOperationErrorV2::Membership(
                expected
            )))
        );
        assert!(pool.is_empty());
        assert_eq!(pool.revision(), 0);
    }
    assert_eq!(chain, accepted);
}

#[test]
fn exact_duplicate_is_recognized_at_entry_capacity_without_reordering() {
    let chain = fresh();
    let accepted = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 10);
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_entries: 1,
            max_batch_operations: 1,
            ..pool_limits()
        },
    )
    .unwrap();
    retain(&mut pool, &chain, &[operation]);
    let before = retained(&pool);
    let used = pool.work_used();
    assert_eq!(
        pool.admit(&chain, &operation.encode()),
        Ok(Admission::Duplicate { id: operation.id() })
    );
    assert_eq!(retained(&pool), before);
    assert_eq!(pool.work_used() - used, 1 + 243 + 1);
    let mut invalid_signature = transfer(&chain, 1, 2, 1, 11).encode();
    *invalid_signature.last_mut().unwrap() ^= 1;
    assert_eq!(
        pool.admit(&chain, &invalid_signature),
        Err(Error::EntryLimit)
    );
    assert_eq!(retained(&pool), before);
    assert_eq!(chain, accepted);
}

#[test]
fn retained_byte_capacity_is_exact_and_recognizes_duplicates_when_full() {
    let chain = fresh();
    let accepted = chain.clone();
    let operations = [transfer(&chain, 1, 2, 0, 10), withdraw(&chain, 2, 0)];
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_entries: 3,
            max_batch_operations: 3,
            max_encoded_bytes: 470,
            ..pool_limits()
        },
    )
    .unwrap();
    retain(&mut pool, &chain, &operations);
    assert_eq!(pool.encoded_bytes(), 470);
    let before = retained(&pool);
    assert_eq!(
        pool.admit(&chain, &operations[0].encode()),
        Ok(Admission::Duplicate {
            id: operations[0].id()
        })
    );
    assert_eq!(
        pool.admit(&chain, &transfer(&chain, 1, 2, 1, 1).encode()),
        Err(Error::EncodedByteLimit)
    );
    assert_eq!(retained(&pool), before);
    assert_eq!(chain, accepted);
}

#[test]
fn a_positive_tiny_retention_limit_is_valid_but_cannot_admit_an_operation() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_encoded_bytes: 1,
            ..pool_limits()
        },
    )
    .unwrap();
    let before = retained(&pool);
    assert_eq!(
        pool.admit(&chain, &transfer(&chain, 1, 2, 0, 1).encode()),
        Err(Error::EncodedByteLimit)
    );
    assert_eq!(retained(&pool), before);
    assert_eq!(chain, accepted);
}

#[test]
fn removal_reuses_capacity_preserves_survivor_order_and_clear_is_explicit() {
    let chain = fresh();
    let accepted = chain.clone();
    let operations: Vec<_> = (0..4)
        .map(|sequence| transfer(&chain, 1, 2, sequence, 1))
        .collect();
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_entries: 3,
            max_batch_operations: 3,
            max_encoded_bytes: 3 * 243,
            ..pool_limits()
        },
    )
    .unwrap();
    retain(&mut pool, &chain, &operations[..3]);
    assert!(pool.remove(operations[1].id()).unwrap());
    assert_eq!(pool.encoded_bytes(), 2 * 243);
    retain(&mut pool, &chain, &operations[3..]);
    assert_eq!(
        pool.operations(),
        [operations[0], operations[2], operations[3]]
    );
    let before = retained(&pool);
    assert!(!pool.remove(operations[1].id()).unwrap());
    assert_eq!(retained(&pool), before);
    assert_eq!(pool.clear().unwrap(), 3);
    assert_eq!(pool.encoded_bytes(), 0);
    assert!(pool.operations().is_empty());
    assert!(pool.ids().is_empty());
    let empty = retained(&pool);
    assert_eq!(pool.clear().unwrap(), 0);
    assert!(!pool.remove(operations[0].id()).unwrap());
    assert_eq!(retained(&pool), empty);
    assert_eq!(chain, accepted);
}

#[test]
fn canonical_pending_membership_capacity_does_not_limit_retained_input() {
    let chain = chain_with(
        LimitsV2 {
            max_pending: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let accepted = chain.clone();
    let operations = [
        enroll(&chain, 3, 8),
        enroll(&chain, 4, 8),
        transfer(&chain, 1, 2, 0, 1),
    ];
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &operations);
    assert_eq!(pool.len(), 3);
    assert_eq!(chain.ledger().membership().pending_count(), 0);
    assert_eq!(chain, accepted);
}

#[test]
fn evaluated_batch_is_an_exact_prefix_and_does_not_remove_or_consume_input() {
    let chain = fresh();
    let accepted = chain.clone();
    let operations = [
        transfer(&chain, 1, 2, 0, 10),
        transfer(&chain, 1, 2, 1, 100),
    ];
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &operations);
    let before = retained(&pool);
    let first = pool.evaluate(&chain, 1).unwrap();
    assert_eq!(first.parent(), chain.checkpoint());
    assert_eq!(first.revision(), pool.revision());
    assert_eq!(first.operations(), &operations[..1]);
    assert_eq!(first.ids(), [operations[0].id()]);
    assert_eq!(first.encoded_block_bytes(), 415);
    let again = pool.evaluate(&chain, 1).unwrap();
    assert_eq!(again.operations(), first.operations());
    assert_eq!(again.ids(), first.ids());
    assert_eq!(retained(&pool), before);
    evaluation_error(
        &mut pool,
        &chain,
        2,
        ledger_error(
            1,
            LedgerOperationErrorV2::InsufficientBalance {
                available: 90,
                required: 100,
            },
        ),
    );
    assert!(pool.batch_is_current(&chain, &first));
    assert_eq!(chain, accepted);
}

#[test]
fn invalid_batch_counts_do_not_reserve_work_or_silently_shorten_the_prefix() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_batch_operations: 1,
            ..pool_limits()
        },
    )
    .unwrap();
    retain(
        &mut pool,
        &chain,
        &[transfer(&chain, 1, 2, 0, 1), transfer(&chain, 1, 2, 1, 1)],
    );
    let used = pool.work_used();
    for count in [0, 2, 3, usize::MAX] {
        evaluation_error(&mut pool, &chain, count, Error::InvalidBatchCount);
        assert_eq!(pool.work_used(), used);
    }
    assert_eq!(pool.evaluate(&chain, 1).unwrap().operations().len(), 1);
    assert_eq!(chain, accepted);
}

#[test]
fn batch_byte_limit_includes_full_framing_and_refuses_before_staging_work() {
    let chain = fresh();
    let accepted = chain.clone();
    let operation = transfer(&chain, 1, 2, 0, 1);
    for cap in [168, 414] {
        let mut pool = Pool::new(
            &chain,
            PendingPoolLimitsV2 {
                max_batch_bytes: cap,
                ..pool_limits()
            },
        )
        .unwrap();
        retain(&mut pool, &chain, &[operation]);
        let used = pool.work_used();
        evaluation_error(&mut pool, &chain, 1, Error::BatchByteLimit);
        assert_eq!(pool.work_used(), used);
    }
    let mut pool = Pool::new(
        &chain,
        PendingPoolLimitsV2 {
            max_batch_bytes: 415,
            ..pool_limits()
        },
    )
    .unwrap();
    retain(
        &mut pool,
        &chain,
        &[operation, transfer(&chain, 1, 2, 1, 1)],
    );
    assert_eq!(pool.evaluate(&chain, 1).unwrap().encoded_block_bytes(), 415);
    let used = pool.work_used();
    evaluation_error(&mut pool, &chain, 2, Error::BatchByteLimit);
    assert_eq!(pool.work_used(), used);
    assert_eq!(chain, accepted);
}

#[test]
fn canonical_total_operation_limit_remains_independent_of_pool_limits() {
    let chain = chain_with(
        LimitsV2 {
            max_operations: 1,
            max_membership_operations: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[transfer(&chain, 1, 2, 0, 1), transfer(&chain, 1, 2, 1, 1)],
    );
    let used = pool.work_used();
    evaluation_error(
        &mut pool,
        &chain,
        2,
        Error::Body(BlockErrorV2::TooManyOperations),
    );
    assert_eq!(pool.work_used(), used);
}

#[test]
fn canonical_membership_and_enrollment_quotas_are_checked_before_evaluation_work() {
    let chain = chain_with(
        LimitsV2 {
            max_operations: 3,
            max_membership_operations: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[withdraw(&chain, 1, 0), withdraw(&chain, 2, 0)],
    );
    let used = pool.work_used();
    evaluation_error(
        &mut pool,
        &chain,
        2,
        Error::Body(BlockErrorV2::TooManyMembershipOperations),
    );
    assert_eq!(pool.work_used(), used);

    let chain = chain_with(
        LimitsV2 {
            max_operations: 2,
            max_membership_operations: 2,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[enroll(&chain, 3, 8), enroll(&chain, 4, 8)],
    );
    let used = pool.work_used();
    evaluation_error(
        &mut pool,
        &chain,
        2,
        Error::Body(BlockErrorV2::TooManyEnrollments),
    );
    assert_eq!(pool.work_used(), used);
}

#[test]
fn canonical_block_bytes_are_checked_even_when_the_local_batch_fits() {
    let chain = chain_with(
        LimitsV2 {
            max_operation_bytes: 291,
            max_block_bytes: 463,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[transfer(&chain, 1, 2, 0, 1), transfer(&chain, 1, 2, 1, 1)],
    );
    assert_eq!(pool.evaluate(&chain, 1).unwrap().encoded_block_bytes(), 415);
    let used = pool.work_used();
    evaluation_error(&mut pool, &chain, 2, Error::Body(BlockErrorV2::TooLarge));
    assert_eq!(pool.work_used(), used);
}

#[test]
fn actual_maximum_entry_count_and_batch_count_are_bounded_independently() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    for sequence in 0..256 {
        retain(&mut pool, &chain, &[transfer(&chain, 1, 1, sequence, 1)]);
    }
    assert_eq!(pool.len(), 256);
    assert_eq!(pool.encoded_bytes(), 256 * 243);
    let before = retained(&pool);
    assert_eq!(
        pool.admit(&chain, &transfer(&chain, 1, 1, 256, 1).encode()),
        Err(Error::EntryLimit)
    );
    assert_eq!(retained(&pool), before);
    let batch = pool.evaluate(&chain, 64).unwrap();
    assert_eq!(batch.operations().len(), 64);
    assert_eq!(batch.encoded_block_bytes(), 168 + 64 * (4 + 243));
    evaluation_error(&mut pool, &chain, 65, Error::InvalidBatchCount);
    assert_eq!(retained(&pool), before);
    assert_eq!(chain, accepted);
}

#[test]
fn contextual_admission_of_an_overspend_does_not_claim_spendability() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[transfer(&chain, 1, 2, 0, 101)]);
    assert_eq!(pool.len(), 1);
    evaluation_error(
        &mut pool,
        &chain,
        1,
        ledger_error(
            0,
            LedgerOperationErrorV2::InsufficientBalance {
                available: 100,
                required: 101,
            },
        ),
    );
    assert_eq!(chain, accepted);
}

#[test]
fn aggregate_spends_are_checked_in_one_ordered_ledger_transition() {
    let chain = fresh();
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[transfer(&chain, 1, 2, 0, 60), transfer(&chain, 1, 2, 1, 50)],
    );
    evaluation_error(
        &mut pool,
        &chain,
        2,
        ledger_error(
            1,
            LedgerOperationErrorV2::InsufficientBalance {
                available: 40,
                required: 50,
            },
        ),
    );
    assert_eq!(pool.len(), 2);
}

#[test]
fn a_sequence_gap_blocks_the_exact_prefix_until_explicit_removal_and_resubmission() {
    let chain = fresh();
    let accepted = chain.clone();
    let later = transfer(&chain, 1, 2, 1, 1);
    let first = transfer(&chain, 1, 2, 0, 1);
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[later, first]);
    evaluation_error(
        &mut pool,
        &chain,
        2,
        ledger_error(
            0,
            LedgerOperationErrorV2::AccountSequenceMismatch {
                expected: 0,
                got: 1,
            },
        ),
    );
    assert_eq!(pool.operations(), [later, first]);
    assert!(pool.remove(later.id()).unwrap());
    assert_eq!(pool.evaluate(&chain, 1).unwrap().operations(), [first]);
    retain(&mut pool, &chain, &[later]);
    assert_eq!(
        pool.evaluate(&chain, 2).unwrap().operations(),
        [first, later]
    );
    assert_eq!(chain, accepted);
}

#[test]
fn distinct_ids_with_one_account_sequence_are_retained_without_replacement() {
    let chain = fresh();
    let accepted = chain.clone();
    let first = transfer(&chain, 1, 2, 0, 10);
    let competing = transfer(&chain, 1, 2, 0, 20);
    assert_ne!(first.id(), competing.id());
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[first, competing]);
    evaluation_error(
        &mut pool,
        &chain,
        2,
        ledger_error(
            1,
            LedgerOperationErrorV2::AccountSequenceMismatch {
                expected: 1,
                got: 0,
            },
        ),
    );
    assert_eq!(pool.operations(), [first, competing]);
    assert!(pool.remove(first.id()).unwrap());
    assert_eq!(pool.evaluate(&chain, 1).unwrap().operations(), [competing]);
    assert_eq!(chain, accepted);
}

#[test]
fn an_earlier_receive_can_fund_a_later_spend_but_reversed_order_fails() {
    let chain = fresh();
    let accepted = chain.clone();
    let receive = transfer(&chain, 1, 2, 0, 10);
    let spend = transfer(&chain, 2, 3, 0, 10);
    let mut forward = new_pool(&chain);
    retain(&mut forward, &chain, &[receive, spend]);
    let batch = forward.evaluate(&chain, 2).unwrap();
    assert_eq!(batch.operations(), [receive, spend]);
    assert_eq!(chain, accepted);
    let mut explicitly_accepted = chain.clone();
    append(&mut explicitly_accepted, batch.operations());
    assert_eq!(
        explicitly_accepted.ledger().account(address(2)).balance(),
        0
    );
    assert_eq!(
        explicitly_accepted.ledger().account(address(3)).balance(),
        10
    );
    let mut reverse = new_pool(&chain);
    retain(&mut reverse, &chain, &[spend, receive]);
    evaluation_error(
        &mut reverse,
        &chain,
        2,
        ledger_error(
            0,
            LedgerOperationErrorV2::InsufficientBalance {
                available: 0,
                required: 10,
            },
        ),
    );
    assert_eq!(chain, accepted);
}

#[test]
fn transfer_account_and_membership_record_sequences_are_separate_domains() {
    let chain = fresh();
    let accepted = chain.clone();
    let enrollment = enroll(&chain, 3, 8);
    let Operation::Membership(member) = enrollment else {
        unreachable!()
    };
    let enrollment_id = member.enrollment_id().unwrap();
    let operations = [
        transfer(&chain, 1, 2, 0, 10),
        enrollment,
        approve(&chain, 1, 0, enrollment_id, 8),
        transfer(&chain, 1, 2, 1, 10),
    ];
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &operations);
    let batch = pool.evaluate(&chain, 4).unwrap();
    assert_eq!(batch.operations(), operations);
    assert_eq!(chain, accepted);
    let mut explicitly_accepted = chain.clone();
    append(&mut explicitly_accepted, batch.operations());
    assert_eq!(
        explicitly_accepted
            .ledger()
            .account(address(1))
            .next_sequence(),
        2
    );
    assert_eq!(
        explicitly_accepted
            .ledger()
            .membership()
            .record_by_key(key(1).public_key())
            .unwrap()
            .next_sequence(),
        1
    );
    assert_eq!(
        explicitly_accepted
            .ledger()
            .membership()
            .record(enrollment_id)
            .unwrap()
            .status(),
        MembershipStatusV2::Active
    );
    assert_eq!(explicitly_accepted.ledger().initial_test_supply(), 100);
    assert_eq!(chain, accepted);
}

#[test]
fn late_transfer_failure_discards_earlier_mixed_effects_without_dropping_entries() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[
            transfer(&chain, 1, 2, 0, 10),
            enroll(&chain, 3, 8),
            transfer(&chain, 1, 2, 1, 100),
        ],
    );
    evaluation_error(
        &mut pool,
        &chain,
        3,
        ledger_error(
            2,
            LedgerOperationErrorV2::InsufficientBalance {
                available: 90,
                required: 100,
            },
        ),
    );
    assert_eq!(pool.len(), 3);
    assert!(
        chain
            .ledger()
            .membership()
            .record_by_key(key(3).public_key())
            .is_none()
    );
    assert_eq!(chain, accepted);
}

#[test]
fn late_membership_failure_discards_earlier_transfer_and_enrollment_effects() {
    let chain = fresh();
    let accepted = chain.clone();
    let mut pool = new_pool(&chain);
    retain(
        &mut pool,
        &chain,
        &[
            transfer(&chain, 1, 2, 0, 10),
            enroll(&chain, 3, 8),
            approve(&chain, 1, 0, tag(99), 8),
        ],
    );
    evaluation_error(
        &mut pool,
        &chain,
        3,
        ledger_error(
            2,
            LedgerOperationErrorV2::Membership(StateErrorV2::UnknownTarget),
        ),
    );
    assert_eq!(pool.len(), 3);
    assert_eq!(chain.ledger().account(address(1)).balance(), 100);
    assert_eq!(chain, accepted);
}

#[test]
fn original_mixed_order_selects_the_first_stateful_failure() {
    let chain = fresh();
    let accepted = chain.clone();
    let overspend = transfer(&chain, 1, 2, 0, 101);
    let unknown = approve(&chain, 1, 0, tag(99), 8);
    for (operations, expected) in [
        (
            [overspend, unknown],
            LedgerOperationErrorV2::InsufficientBalance {
                available: 100,
                required: 101,
            },
        ),
        (
            [unknown, overspend],
            LedgerOperationErrorV2::Membership(StateErrorV2::UnknownTarget),
        ),
    ] {
        let mut pool = new_pool(&chain);
        retain(&mut pool, &chain, &operations);
        evaluation_error(&mut pool, &chain, 2, ledger_error(0, expected));
    }
    assert_eq!(chain, accepted);
}

#[test]
fn retained_enrollment_still_obeys_canonical_pending_state_during_execution() {
    let mut chain = chain_with(
        LimitsV2 {
            max_pending: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let existing = enroll(&chain, 3, 8);
    append(&mut chain, &[existing]);
    let accepted = chain.clone();
    assert_eq!(chain.ledger().membership().pending_count(), 1);
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[enroll(&chain, 4, 9)]);
    evaluation_error(
        &mut pool,
        &chain,
        1,
        ledger_error(
            0,
            LedgerOperationErrorV2::Membership(StateErrorV2::PendingLimit),
        ),
    );
    assert_eq!(pool.len(), 1);
    assert_eq!(chain, accepted);
}

#[test]
fn admission_removal_and_clear_invalidate_batches_but_no_ops_and_refusals_do_not() {
    let chain = fresh();
    let accepted = chain.clone();
    let first = transfer(&chain, 1, 2, 0, 1);
    let second = transfer(&chain, 1, 2, 1, 1);
    let mut pool = new_pool(&chain);
    retain(&mut pool, &chain, &[first]);
    let batch = pool.evaluate(&chain, 1).unwrap();
    assert!(pool.batch_is_current(&chain, &batch));
    assert_eq!(
        pool.admit(&chain, &first.encode()),
        Ok(Admission::Duplicate { id: first.id() })
    );
    assert!(!pool.remove(tag(99)).unwrap());
    assert!(
        pool.admit(&chain, &transfer(&chain, 1, 2, 1, 0).encode())
            .is_err()
    );
    evaluation_error(&mut pool, &chain, 0, Error::InvalidBatchCount);
    assert!(pool.batch_is_current(&chain, &batch));
    retain(&mut pool, &chain, &[second]);
    assert!(!pool.batch_is_current(&chain, &batch));
    let batch = pool.evaluate(&chain, 2).unwrap();
    assert!(pool.batch_is_current(&chain, &batch));
    assert!(pool.remove(second.id()).unwrap());
    assert!(!pool.batch_is_current(&chain, &batch));
    let batch = pool.evaluate(&chain, 1).unwrap();
    assert!(pool.batch_is_current(&chain, &batch));
    assert_eq!(pool.clear().unwrap(), 1);
    assert!(!pool.batch_is_current(&chain, &batch));
    let revision = pool.revision();
    assert_eq!(pool.clear().unwrap(), 0);
    assert_eq!(pool.revision(), revision);
    assert_eq!(chain, accepted);
}
