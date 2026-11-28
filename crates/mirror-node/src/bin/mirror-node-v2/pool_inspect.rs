//! One finite input snapshot and ephemeral C1 owner under one clean A1 writer.
//! Reports copy only observations; all input, pool and authority owners drop
//! before the caller can format or write output.

use std::path::Path;

use mirror_core::v2::{BlockErrorV2, MIN_ENCODED_BLOCK_LEN_V2};
use mirror_node::v2::{
    NodeControlV2,
    pool::{MAX_POOL_OPERATION_BYTES_V2, PendingPoolErrorV2, PendingPoolV2, PoolAdmissionV2},
};
use mirror_personhood::v2::{StateCodecErrorV2, StateErrorV2};
use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2};
use mirror_storage::v2::ReplacementCheckpointV2;

use super::{
    args::{Common, PoolInspectOptions},
    errors::Failure,
    pool_input,
    signals::Signals,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BlockedReason {
    InvalidCount,
    BatchByteLimit,
    WorkLimit,
    Operation,
    Body,
    Ledger,
}

impl BlockedReason {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidCount => "invalid-count",
            Self::BatchByteLimit => "batch-byte-limit",
            Self::WorkLimit => "work-limit",
            Self::Operation => "operation",
            Self::Body => "body",
            Self::Ledger => "ledger",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Evaluation {
    NotRequested,
    Evaluated {
        count: usize,
        block_bytes: usize,
    },
    Blocked {
        reason: BlockedReason,
        index: Option<usize>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Report {
    pub checkpoint: ReplacementCheckpointV2,
    pub options: PoolInspectOptions,
    pub input_bytes: usize,
    pub input_count: usize,
    pub retained_count: usize,
    pub duplicate_count: usize,
    pub retained_encoded_bytes: usize,
    pub work_used: u64,
    pub evaluation: Evaluation,
}

impl Report {
    pub(super) const fn exit_code(&self) -> u8 {
        if matches!(self.evaluation, Evaluation::Blocked { .. }) {
            1
        } else {
            0
        }
    }
}

pub(super) fn run(
    common: &Common,
    input: &Path,
    options: PoolInspectOptions,
) -> Result<Report, Failure> {
    // The parser already performs this effect-free validation. Keep the private
    // coordinator's boundary defensive without widening A1 or C1's public API.
    options
        .validate()
        .map_err(|_| Failure::new("pool_invariant"))?;
    let signals = Signals::install()?;
    let snapshot = pool_input::capture(
        input,
        options.max_input_bytes,
        options.max_input_count,
        &signals,
    )?;
    check_stop(&signals)?;
    let ready = NodeControlV2::open_writer(&common.control, &common.store, common.identity)?;
    check_stop(&signals)?;
    let checkpoint = ready.checkpoint();
    if checkpoint.chain != ready.chain().checkpoint()
        || checkpoint.chain.config_hash != common.identity.config_hash
        || checkpoint.chain.genesis_hash != common.identity.genesis_hash
    {
        return Err(Failure::new("pool_invariant"));
    }
    let mut pool = PendingPoolV2::new(ready.chain(), options.limits).map_err(admission_error)?;
    let mut duplicate_count = 0usize;
    let mut decoded = [0; MAX_POOL_OPERATION_BYTES_V2];
    for index in 0..snapshot.len() {
        check_stop(&signals)?;
        let length = snapshot.decode(index, &mut decoded)?;
        let admitted = pool
            .admit(ready.chain(), &decoded[..length])
            .map_err(admission_error)?;
        // A real admission error above wins over a stop set during that call.
        check_stop(&signals)?;
        if matches!(admitted, PoolAdmissionV2::Duplicate { .. }) {
            duplicate_count = duplicate_count
                .checked_add(1)
                .ok_or_else(|| Failure::new("arithmetic_overflow"))?;
        }
    }
    let evaluation = if options.evaluate_count == 0 {
        Evaluation::NotRequested
    } else {
        check_stop(&signals)?;
        match pool.evaluate(ready.chain(), options.evaluate_count) {
            Ok(batch) => {
                let count = batch.operations().len();
                let block_bytes = batch.encoded_block_bytes();
                let expected_bytes = batch
                    .operations()
                    .iter()
                    .try_fold(MIN_ENCODED_BLOCK_LEN_V2, |total, operation| {
                        total
                            .checked_add(4)
                            .and_then(|value| value.checked_add(operation.encoded_len()))
                    })
                    .ok_or_else(|| Failure::new("arithmetic_overflow"))?;
                if count != options.evaluate_count
                    || batch.ids().len() != count
                    || batch.parent() != checkpoint.chain
                    || !pool.batch_is_current(ready.chain(), &batch)
                    || block_bytes != expected_bytes
                    || block_bytes > options.limits.max_batch_bytes
                {
                    return Err(Failure::new("pool_invariant"));
                }
                drop(batch);
                check_stop(&signals)?;
                Evaluation::Evaluated { count, block_bytes }
            }
            Err(error) => evaluation_error(error, options.evaluate_count)?,
        }
    };
    if pool.parent() != checkpoint.chain
        || ready.checkpoint() != checkpoint
        || snapshot.len()
            != pool
                .len()
                .checked_add(duplicate_count)
                .ok_or_else(|| Failure::new("arithmetic_overflow"))?
    {
        return Err(Failure::new("pool_invariant"));
    }
    // A blocked evaluation is a real returned outcome. Do not relabel it as
    // shutdown if the flag was set while the failed evaluation was admitted.
    if !matches!(evaluation, Evaluation::Blocked { .. }) {
        check_stop(&signals)?;
    }
    let report = Report {
        checkpoint,
        options,
        input_bytes: snapshot.byte_len(),
        input_count: snapshot.len(),
        retained_count: pool.len(),
        duplicate_count,
        retained_encoded_bytes: pool.encoded_bytes(),
        work_used: pool.work_used(),
        evaluation,
    };
    drop(pool);
    drop(ready);
    drop(snapshot);
    drop(signals);
    Ok(report)
}

fn check_stop(signals: &Signals) -> Result<(), Failure> {
    if signals.requested() {
        Err(Failure::new("pool_shutdown"))
    } else {
        Ok(())
    }
}

fn admission_error(error: PendingPoolErrorV2) -> Failure {
    Failure::new(match error {
        PendingPoolErrorV2::InputTooLarge | PendingPoolErrorV2::Operation(_) => "pool_operation",
        PendingPoolErrorV2::EntryLimit => "pool_entry_limit",
        PendingPoolErrorV2::EncodedByteLimit => "pool_encoded_byte_limit",
        PendingPoolErrorV2::WorkLimit => "pool_work_limit",
        PendingPoolErrorV2::ArithmeticOverflow => "arithmetic_overflow",
        PendingPoolErrorV2::AllocationFailed => "allocation_failed",
        PendingPoolErrorV2::InvalidLimits
        | PendingPoolErrorV2::WrongNetwork
        | PendingPoolErrorV2::ParentChanged
        | PendingPoolErrorV2::InvalidBatchCount
        | PendingPoolErrorV2::BatchByteLimit
        | PendingPoolErrorV2::Body(_)
        | PendingPoolErrorV2::Ledger(_) => "pool_invariant",
    })
}

fn evaluation_error(error: PendingPoolErrorV2, count: usize) -> Result<Evaluation, Failure> {
    match error {
        PendingPoolErrorV2::InvalidBatchCount => blocked(BlockedReason::InvalidCount, None, count),
        PendingPoolErrorV2::BatchByteLimit => blocked(BlockedReason::BatchByteLimit, None, count),
        PendingPoolErrorV2::WorkLimit => blocked(BlockedReason::WorkLimit, None, count),
        PendingPoolErrorV2::Operation(_) => blocked(BlockedReason::Operation, None, count),
        PendingPoolErrorV2::Body(error) => body_error(error, count),
        PendingPoolErrorV2::Ledger(error) => ledger_error(error, count),
        PendingPoolErrorV2::ArithmeticOverflow => Err(Failure::new("arithmetic_overflow")),
        PendingPoolErrorV2::AllocationFailed => Err(Failure::new("allocation_failed")),
        PendingPoolErrorV2::InvalidLimits
        | PendingPoolErrorV2::WrongNetwork
        | PendingPoolErrorV2::ParentChanged
        | PendingPoolErrorV2::InputTooLarge
        | PendingPoolErrorV2::EntryLimit
        | PendingPoolErrorV2::EncodedByteLimit => Err(Failure::new("pool_invariant")),
    }
}

fn body_error(error: BlockErrorV2, count: usize) -> Result<Evaluation, Failure> {
    match error {
        BlockErrorV2::LengthOverflow => Err(Failure::new("arithmetic_overflow")),
        BlockErrorV2::OperationTooLarge { index } => {
            blocked(BlockedReason::Body, Some(index), count)
        }
        BlockErrorV2::TooLarge
        | BlockErrorV2::TooManyOperations
        | BlockErrorV2::TooManyMembershipOperations
        | BlockErrorV2::TooManyEnrollments => blocked(BlockedReason::Body, None, count),
        // C1 preflight receives retained typed operations. It neither decodes
        // a block nor checks a supplied header/root/context in this call.
        BlockErrorV2::InvalidHeaderLength
        | BlockErrorV2::UnsupportedVersion(_)
        | BlockErrorV2::InvalidMagic
        | BlockErrorV2::Truncated
        | BlockErrorV2::TrailingBytes
        | BlockErrorV2::Operation { .. }
        | BlockErrorV2::OperationContext { .. }
        | BlockErrorV2::WrongContext
        | BlockErrorV2::OperationsRootMismatch => Err(Failure::new("pool_invariant")),
    }
}

fn ledger_error(error: LedgerErrorV2, count: usize) -> Result<Evaluation, Failure> {
    match error {
        LedgerErrorV2::Limits(error) => body_error(error, count),
        LedgerErrorV2::Membership(error) => membership_error(error, count),
        LedgerErrorV2::Operation { index, error } => {
            match error {
                LedgerOperationErrorV2::Membership(error) => {
                    let nested = membership_error(error, count)?;
                    if let Evaluation::Blocked {
                        index: Some(nested_index),
                        ..
                    } = nested
                    {
                        if usize::try_from(index).ok() != Some(nested_index) {
                            return Err(Failure::new("pool_invariant"));
                        }
                    }
                }
                LedgerOperationErrorV2::Transfer(_)
                | LedgerOperationErrorV2::AccountSequenceMismatch { .. }
                | LedgerOperationErrorV2::AccountSequenceOverflow
                | LedgerOperationErrorV2::InsufficientBalance { .. }
                | LedgerOperationErrorV2::RecipientBalanceOverflow
                | LedgerOperationErrorV2::AccountLimit => {}
            }
            blocked(BlockedReason::Ledger, Some(index), count)
        }
        LedgerErrorV2::MembershipCodec(error) => Err(codec_error(error)),
        LedgerErrorV2::AccountLimit => blocked(BlockedReason::Ledger, None, count),
        LedgerErrorV2::BalanceSumOverflow | LedgerErrorV2::LengthOverflow => {
            Err(Failure::new("arithmetic_overflow"))
        }
        LedgerErrorV2::SupplyMismatch { .. } => Err(Failure::new("pool_invariant")),
        LedgerErrorV2::AllocationFailed => Err(Failure::new("allocation_failed")),
    }
}

/// Semantic membership refusals remain inspectable. Structural parent/index/
/// stage errors are impossible for this accepted-chain integration; counters
/// and lengths must never be downgraded to a normal blocked report.
fn membership_error(error: StateErrorV2, count: usize) -> Result<Evaluation, Failure> {
    match error {
        StateErrorV2::HeightOverflow | StateErrorV2::CounterOverflow => {
            Err(Failure::new("arithmetic_overflow"))
        }
        StateErrorV2::MembershipCountExceedsTotal
        | StateErrorV2::OperationIndexOutOfRange { .. }
        | StateErrorV2::OperationIndicesNotIncreasing { .. }
        | StateErrorV2::UnexpectedOperationIndex { .. }
        | StateErrorV2::NoRemainingMembershipOperations
        | StateErrorV2::IncompleteMembershipStage { .. }
        | StateErrorV2::WrongParentContext
        | StateErrorV2::WrongParentHeight
        | StateErrorV2::UnknownParent
        | StateErrorV2::WrongParentHash
        | StateErrorV2::InconsistentState(_) => Err(Failure::new("pool_invariant")),
        StateErrorV2::Operation { index, .. } => blocked(BlockedReason::Ledger, Some(index), count),
        StateErrorV2::OperationLimit
        | StateErrorV2::MembershipOperationLimit
        | StateErrorV2::EnrollmentLimit
        | StateErrorV2::OperationBytesLimit
        | StateErrorV2::BlockBytesLimit
        | StateErrorV2::DuplicateId
        | StateErrorV2::DuplicateKey
        | StateErrorV2::DuplicateEvidence
        | StateErrorV2::RecordLimit
        | StateErrorV2::PendingLimit
        | StateErrorV2::UnknownActor
        | StateErrorV2::UnknownTarget
        | StateErrorV2::SelfApproval
        | StateErrorV2::ReviewerNotActive
        | StateErrorV2::TargetNotPending
        | StateErrorV2::ApprovalWindowMismatch
        | StateErrorV2::DuplicateApproval
        | StateErrorV2::ApprovalRecordLimit
        | StateErrorV2::ApprovalTotalLimit
        | StateErrorV2::SequenceMismatch { .. }
        | StateErrorV2::SequenceOverflow
        | StateErrorV2::NotOwner
        | StateErrorV2::TerminalMembership => blocked(BlockedReason::Ledger, None, count),
    }
}

fn codec_error(error: StateCodecErrorV2) -> Failure {
    match error {
        StateCodecErrorV2::AllocationFailed => Failure::new("allocation_failed"),
        StateCodecErrorV2::State(StateErrorV2::HeightOverflow | StateErrorV2::CounterOverflow) => {
            Failure::new("arithmetic_overflow")
        }
        StateCodecErrorV2::TooLarge
        | StateCodecErrorV2::Truncated
        | StateCodecErrorV2::TrailingBytes
        | StateCodecErrorV2::InvalidMagic
        | StateCodecErrorV2::UnsupportedVersion
        | StateCodecErrorV2::WrongContext
        | StateCodecErrorV2::RecordLimit
        | StateCodecErrorV2::ApprovalLimit
        | StateCodecErrorV2::InvalidTag { .. }
        | StateCodecErrorV2::RecordOrder
        | StateCodecErrorV2::ApprovalOrder
        | StateCodecErrorV2::Inconsistent(_)
        | StateCodecErrorV2::State(_)
        | StateCodecErrorV2::Operation(_)
        | StateCodecErrorV2::SnapshotMismatch => Failure::new("pool_invariant"),
    }
}

fn blocked(reason: BlockedReason, index: Option<u32>, count: usize) -> Result<Evaluation, Failure> {
    let index = index
        .map(|index| usize::try_from(index).map_err(|_| Failure::new("arithmetic_overflow")))
        .transpose()?;
    if index.is_some_and(|index| index >= count) {
        return Err(Failure::new("pool_invariant"));
    }
    Ok(Evaluation::Blocked { reason, index })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirror_core::v2::{TransferErrorV2, UnifiedOperationErrorV2};
    use mirror_personhood::v2::OperationErrorV2;

    fn failure(result: Result<Evaluation, Failure>) -> &'static str {
        match result {
            Ok(_) => panic!("expected operational failure"),
            Err(error) => error.class,
        }
    }

    fn evaluation(result: Result<Evaluation, Failure>) -> Evaluation {
        result.unwrap_or_else(|error| panic!("unexpected static failure: {}", error.class))
    }

    #[test]
    fn closed_blocked_reasons_have_exact_contract_spellings() {
        for (reason, text) in [
            (BlockedReason::InvalidCount, "invalid-count"),
            (BlockedReason::BatchByteLimit, "batch-byte-limit"),
            (BlockedReason::WorkLimit, "work-limit"),
            (BlockedReason::Operation, "operation"),
            (BlockedReason::Body, "body"),
            (BlockedReason::Ledger, "ledger"),
        ] {
            assert_eq!(reason.as_str(), text);
        }
    }

    #[test]
    fn admission_refusals_never_become_partial_normal_reports() {
        for (error, class) in [
            (PendingPoolErrorV2::InputTooLarge, "pool_operation"),
            (
                PendingPoolErrorV2::Operation(UnifiedOperationErrorV2::InvalidLength),
                "pool_operation",
            ),
            (PendingPoolErrorV2::EntryLimit, "pool_entry_limit"),
            (
                PendingPoolErrorV2::EncodedByteLimit,
                "pool_encoded_byte_limit",
            ),
            (PendingPoolErrorV2::WorkLimit, "pool_work_limit"),
            (
                PendingPoolErrorV2::ArithmeticOverflow,
                "arithmetic_overflow",
            ),
            (PendingPoolErrorV2::AllocationFailed, "allocation_failed"),
            (PendingPoolErrorV2::InvalidLimits, "pool_invariant"),
            (PendingPoolErrorV2::WrongNetwork, "pool_invariant"),
            (PendingPoolErrorV2::ParentChanged, "pool_invariant"),
            (PendingPoolErrorV2::InvalidBatchCount, "pool_invariant"),
            (PendingPoolErrorV2::BatchByteLimit, "pool_invariant"),
            (
                PendingPoolErrorV2::Body(BlockErrorV2::TooLarge),
                "pool_invariant",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::AccountLimit),
                "pool_invariant",
            ),
        ] {
            assert_eq!(admission_error(error).class, class);
        }
    }

    #[test]
    fn expected_evaluation_refusals_are_bounded_closed_results() {
        for (error, reason) in [
            (
                PendingPoolErrorV2::InvalidBatchCount,
                BlockedReason::InvalidCount,
            ),
            (
                PendingPoolErrorV2::BatchByteLimit,
                BlockedReason::BatchByteLimit,
            ),
            (PendingPoolErrorV2::WorkLimit, BlockedReason::WorkLimit),
            (
                PendingPoolErrorV2::Operation(UnifiedOperationErrorV2::UnsupportedKind),
                BlockedReason::Operation,
            ),
            (
                PendingPoolErrorV2::Body(BlockErrorV2::TooManyEnrollments),
                BlockedReason::Body,
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::AccountLimit),
                BlockedReason::Ledger,
            ),
        ] {
            assert_eq!(
                evaluation(evaluation_error(error, 4)),
                Evaluation::Blocked {
                    reason,
                    index: None
                }
            );
        }
    }

    #[test]
    fn body_quota_refusals_are_distinct_from_impossible_representation_failures() {
        for error in [
            BlockErrorV2::TooLarge,
            BlockErrorV2::TooManyOperations,
            BlockErrorV2::TooManyMembershipOperations,
            BlockErrorV2::TooManyEnrollments,
        ] {
            let expected = Evaluation::Blocked {
                reason: BlockedReason::Body,
                index: None,
            };
            assert_eq!(
                evaluation(evaluation_error(PendingPoolErrorV2::Body(error), 4)),
                expected
            );
            assert_eq!(
                evaluation(ledger_error(LedgerErrorV2::Limits(error), 4)),
                expected
            );
        }
        assert_eq!(
            evaluation(body_error(BlockErrorV2::OperationTooLarge { index: 2 }, 4)),
            Evaluation::Blocked {
                reason: BlockedReason::Body,
                index: Some(2)
            }
        );
        assert_eq!(
            failure(body_error(BlockErrorV2::LengthOverflow, 4)),
            "arithmetic_overflow"
        );
        for error in [
            BlockErrorV2::InvalidHeaderLength,
            BlockErrorV2::UnsupportedVersion(1),
            BlockErrorV2::InvalidMagic,
            BlockErrorV2::Truncated,
            BlockErrorV2::TrailingBytes,
            BlockErrorV2::Operation {
                index: 2,
                error: UnifiedOperationErrorV2::InvalidLength,
            },
            BlockErrorV2::OperationContext { index: 2 },
            BlockErrorV2::WrongContext,
            BlockErrorV2::OperationsRootMismatch,
        ] {
            assert_eq!(
                failure(evaluation_error(PendingPoolErrorV2::Body(error), 4)),
                "pool_invariant"
            );
            assert_eq!(
                failure(ledger_error(LedgerErrorV2::Limits(error), 4)),
                "pool_invariant"
            );
        }
    }

    #[test]
    fn ledger_operation_domain_errors_preserve_the_exact_fifo_index() {
        for error in [
            LedgerOperationErrorV2::Transfer(TransferErrorV2::Expired),
            LedgerOperationErrorV2::AccountSequenceMismatch {
                expected: 4,
                got: 6,
            },
            LedgerOperationErrorV2::AccountSequenceOverflow,
            LedgerOperationErrorV2::InsufficientBalance {
                available: 1,
                required: 2,
            },
            LedgerOperationErrorV2::RecipientBalanceOverflow,
            LedgerOperationErrorV2::AccountLimit,
            LedgerOperationErrorV2::Membership(StateErrorV2::UnknownTarget),
            LedgerOperationErrorV2::Membership(StateErrorV2::DuplicateApproval),
            LedgerOperationErrorV2::Membership(StateErrorV2::SequenceOverflow),
            LedgerOperationErrorV2::Membership(StateErrorV2::Operation {
                index: 3,
                error: OperationErrorV2::Expired,
            }),
        ] {
            assert_eq!(
                evaluation(ledger_error(
                    LedgerErrorV2::Operation { index: 3, error },
                    4
                )),
                Evaluation::Blocked {
                    reason: BlockedReason::Ledger,
                    index: Some(3)
                }
            );
        }
    }

    #[test]
    fn impossible_operation_indices_are_operational_invariants() {
        for error in [
            LedgerErrorV2::Operation {
                index: 4,
                error: LedgerOperationErrorV2::AccountLimit,
            },
            LedgerErrorV2::Membership(StateErrorV2::Operation {
                index: 4,
                error: OperationErrorV2::Expired,
            }),
            LedgerErrorV2::Operation {
                index: 3,
                error: LedgerOperationErrorV2::Membership(StateErrorV2::Operation {
                    index: 2,
                    error: OperationErrorV2::Expired,
                }),
            },
        ] {
            assert_eq!(failure(ledger_error(error, 4)), "pool_invariant");
        }
        assert_eq!(
            failure(body_error(BlockErrorV2::OperationContext { index: 4 }, 4)),
            "pool_invariant"
        );
        assert_eq!(
            evaluation(body_error(BlockErrorV2::OperationTooLarge { index: 3 }, 4)),
            Evaluation::Blocked {
                reason: BlockedReason::Body,
                index: Some(3)
            }
        );
    }

    #[test]
    fn nested_membership_bookkeeping_failures_are_never_domain_blockage() {
        for error in [
            StateErrorV2::MembershipCountExceedsTotal,
            StateErrorV2::OperationIndexOutOfRange { index: 1 },
            StateErrorV2::OperationIndicesNotIncreasing {
                previous: 1,
                got: 0,
            },
            StateErrorV2::UnexpectedOperationIndex {
                expected: 0,
                got: 1,
            },
            StateErrorV2::NoRemainingMembershipOperations,
            StateErrorV2::IncompleteMembershipStage { remaining: 1 },
            StateErrorV2::WrongParentContext,
            StateErrorV2::WrongParentHeight,
            StateErrorV2::UnknownParent,
            StateErrorV2::WrongParentHash,
            StateErrorV2::InconsistentState("untrusted\nstatus=clean"),
        ] {
            assert_eq!(
                failure(ledger_error(LedgerErrorV2::Membership(error), 4)),
                "pool_invariant"
            );
            assert_eq!(
                failure(ledger_error(
                    LedgerErrorV2::Operation {
                        index: 2,
                        error: LedgerOperationErrorV2::Membership(error)
                    },
                    4
                )),
                "pool_invariant"
            );
            assert_eq!(
                failure(ledger_error(
                    LedgerErrorV2::MembershipCodec(StateCodecErrorV2::State(error)),
                    4
                )),
                "pool_invariant"
            );
        }
    }

    #[test]
    fn allocation_arithmetic_supply_and_codec_failures_stay_operational() {
        for (error, class) in [
            (PendingPoolErrorV2::AllocationFailed, "allocation_failed"),
            (
                PendingPoolErrorV2::ArithmeticOverflow,
                "arithmetic_overflow",
            ),
            (
                PendingPoolErrorV2::Body(BlockErrorV2::LengthOverflow),
                "arithmetic_overflow",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::Limits(BlockErrorV2::LengthOverflow)),
                "arithmetic_overflow",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::AllocationFailed),
                "allocation_failed",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::LengthOverflow),
                "arithmetic_overflow",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::BalanceSumOverflow),
                "arithmetic_overflow",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::SupplyMismatch {
                    expected: 1,
                    got: 2,
                }),
                "pool_invariant",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::MembershipCodec(
                    StateCodecErrorV2::AllocationFailed,
                )),
                "allocation_failed",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::MembershipCodec(
                    StateCodecErrorV2::Inconsistent("do not echo"),
                )),
                "pool_invariant",
            ),
            (
                PendingPoolErrorV2::Ledger(LedgerErrorV2::MembershipCodec(
                    StateCodecErrorV2::State(StateErrorV2::UnknownTarget),
                )),
                "pool_invariant",
            ),
        ] {
            assert_eq!(failure(evaluation_error(error, 4)), class);
        }
        for error in [StateErrorV2::HeightOverflow, StateErrorV2::CounterOverflow] {
            for nested in [
                LedgerErrorV2::Membership(error),
                LedgerErrorV2::Operation {
                    index: 2,
                    error: LedgerOperationErrorV2::Membership(error),
                },
                LedgerErrorV2::MembershipCodec(StateCodecErrorV2::State(error)),
            ] {
                assert_eq!(failure(ledger_error(nested, 4)), "arithmetic_overflow");
            }
        }
    }

    #[test]
    fn unexpected_evaluation_owner_or_admission_errors_are_invariants() {
        for error in [
            PendingPoolErrorV2::InvalidLimits,
            PendingPoolErrorV2::WrongNetwork,
            PendingPoolErrorV2::ParentChanged,
            PendingPoolErrorV2::InputTooLarge,
            PendingPoolErrorV2::EntryLimit,
            PendingPoolErrorV2::EncodedByteLimit,
        ] {
            assert_eq!(failure(evaluation_error(error, 1)), "pool_invariant");
        }
    }
}
