//! Deliberately static diagnostics: no raw path, argument, peer bytes or nested
//! error formatting can escape through the operator error record.

use std::{fmt, io};

use mirror_network::v2::{HistoryErrorV2, session::SessionErrorV2};
use mirror_node::v2::{NodeControlErrorV2, records::ControlRecordErrorV2};
use mirror_storage::v2::{
    ReplacementRecoveryErrorV2, ReplacementSelectionErrorV2, ReplacementStorageErrorV2,
    SelectionErrorV2, StorageErrorV2,
};

pub struct Failure {
    pub class: &'static str,
    pub next: &'static str,
}

impl Failure {
    pub(super) fn new(class: &'static str) -> Self {
        let next = match class {
            "recovery_required" => "explicit-recover",
            "invalid_path"
            | "unsafe_path"
            | "wrong_network"
            | "checkpoint_mismatch"
            | "missing_control"
            | "missing_store"
            | "uninitialized_store"
            | "invalid_control"
            | "invalid_store"
            | "invalid_recovery"
            | "peer_wrong_network"
            | "invalid_peer_request"
            | "listen_bind"
            | "lifecycle_limit"
            | "unsupported_platform" => "verify-inputs",
            _ => "inspect-status",
        };
        Self { class, next }
    }
}

impl From<fmt::Error> for Failure {
    fn from(_: fmt::Error) -> Self {
        Self::new("output_limit")
    }
}

impl From<NodeControlErrorV2> for Failure {
    fn from(error: NodeControlErrorV2) -> Self {
        Self::new(match &error {
            NodeControlErrorV2::UnsupportedPlatform => "unsupported_platform",
            NodeControlErrorV2::InvalidPath => "invalid_path",
            NodeControlErrorV2::UnsafePath(_) => "unsafe_path",
            NodeControlErrorV2::MissingControl => "missing_control",
            NodeControlErrorV2::Busy => "busy",
            NodeControlErrorV2::RecoveryRequired => "recovery_required",
            NodeControlErrorV2::NoPendingIntent => "no_pending_intent",
            NodeControlErrorV2::CannotRetainConfirmedCandidate => {
                "cannot_retain_confirmed_candidate"
            }
            NodeControlErrorV2::ChangedControl => "changed_control",
            NodeControlErrorV2::InvalidLayout(_) | NodeControlErrorV2::LengthMismatch => {
                "invalid_control"
            }
            NodeControlErrorV2::ReadBudgetExceeded => "read_budget_exceeded",
            NodeControlErrorV2::AllocationFailed => "allocation_failed",
            NodeControlErrorV2::Io(error) => local_io(error, "control_io"),
            NodeControlErrorV2::Record(error) => match error {
                ControlRecordErrorV2::WrongIdentity => "wrong_network",
                ControlRecordErrorV2::WrongCurrentCheckpoint => "checkpoint_mismatch",
                ControlRecordErrorV2::Selection(error) => {
                    selection_pin_error(error).unwrap_or("invalid_control")
                }
                _ => "invalid_control",
            },
            NodeControlErrorV2::Storage(error) => replacement(error),
            NodeControlErrorV2::Recovery(error) => recovery(error),
            NodeControlErrorV2::ForkChoice(_) => "invalid_branch_comparison",
        })
    }
}

impl From<SessionErrorV2> for Failure {
    fn from(error: SessionErrorV2) -> Self {
        Self::new(match &error {
            SessionErrorV2::InvalidLimits(_) => "invalid_session_limits",
            SessionErrorV2::InvalidRequest => "invalid_peer_request",
            SessionErrorV2::WrongNetwork
            | SessionErrorV2::History(HistoryErrorV2::WrongNetwork) => "peer_wrong_network",
            SessionErrorV2::AttemptsExhausted => "peer_attempt_budget",
            SessionErrorV2::WireBudgetExhausted => "peer_io_budget",
            SessionErrorV2::DeadlineExceeded => "peer_deadline",
            SessionErrorV2::EarlyClose => "peer_early_close",
            SessionErrorV2::TrailingData => "peer_trailing_data",
            SessionErrorV2::AllocationFailed => "allocation_failed",
            SessionErrorV2::Io(error) => match error.kind() {
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => "peer_deadline",
                io::ErrorKind::ConnectionRefused
                | io::ErrorKind::NetworkUnreachable
                | io::ErrorKind::HostUnreachable => "peer_unreachable",
                _ => "peer_io",
            },
            SessionErrorV2::History(_) => "invalid_peer_history",
        })
    }
}

fn local_io(error: &io::Error, otherwise: &'static str) -> &'static str {
    if error.kind() == io::ErrorKind::PermissionDenied {
        "permission_denied"
    } else {
        otherwise
    }
}

fn snapshot_pin_error(error: &SelectionErrorV2) -> Option<&'static str> {
    match error {
        SelectionErrorV2::WrongConfiguration | SelectionErrorV2::WrongGenesis => {
            Some("wrong_network")
        }
        SelectionErrorV2::WrongHeight
        | SelectionErrorV2::WrongTip
        | SelectionErrorV2::WrongLedgerRoot => Some("checkpoint_mismatch"),
        _ => None,
    }
}

fn selection_pin_error(error: &ReplacementSelectionErrorV2) -> Option<&'static str> {
    match error {
        ReplacementSelectionErrorV2::WrongGeneration => Some("checkpoint_mismatch"),
        ReplacementSelectionErrorV2::Snapshot(error) => snapshot_pin_error(error),
        _ => None,
    }
}

fn storage(error: &StorageErrorV2) -> &'static str {
    match error {
        StorageErrorV2::UnsupportedPlatform => "unsupported_platform",
        StorageErrorV2::InvalidPath | StorageErrorV2::NotDirectory => "invalid_path",
        StorageErrorV2::UnsafePath(_) => "unsafe_path",
        StorageErrorV2::MissingDirectory => "missing_store",
        StorageErrorV2::EmptyDirectory => "uninitialized_store",
        StorageErrorV2::Busy => "busy",
        StorageErrorV2::IdentityMismatch => "wrong_network",
        StorageErrorV2::AllocationFailed => "allocation_failed",
        StorageErrorV2::Selection(error) => snapshot_pin_error(error).unwrap_or("invalid_store"),
        StorageErrorV2::Io(error) => local_io(error, "store_io"),
        StorageErrorV2::InitializationIncomplete(_) => "initialization_incomplete",
        StorageErrorV2::PublicationUncertain(_) => "publication_uncertain",
        _ => "invalid_store",
    }
}

fn replacement(error: &ReplacementStorageErrorV2) -> &'static str {
    match error {
        ReplacementStorageErrorV2::Storage(error) => storage(error),
        ReplacementStorageErrorV2::Selection(error) => {
            selection_pin_error(error).unwrap_or("invalid_store")
        }
        ReplacementStorageErrorV2::ReadOnly => "read_only_store",
        ReplacementStorageErrorV2::Fenced => "store_fenced",
        ReplacementStorageErrorV2::SelectionChanged
        | ReplacementStorageErrorV2::CurrentChanged
        | ReplacementStorageErrorV2::CandidateChanged
        | ReplacementStorageErrorV2::FileChanged => "changed_store",
        ReplacementStorageErrorV2::ReadBudgetExceeded { .. } => "read_budget_exceeded",
        ReplacementStorageErrorV2::Plan(_) => "invalid_replacement",
        ReplacementStorageErrorV2::InitializationIncomplete(_) => "initialization_incomplete",
        ReplacementStorageErrorV2::InitializationUncertain(_)
        | ReplacementStorageErrorV2::PublicationUncertain { .. } => "publication_uncertain",
        ReplacementStorageErrorV2::ReconciliationFailed(_) => "reconciliation_failed",
        ReplacementStorageErrorV2::UpdateIncomplete { .. } => "update_incomplete",
    }
}

fn recovery(error: &ReplacementRecoveryErrorV2) -> &'static str {
    match error {
        ReplacementRecoveryErrorV2::Storage(error) => storage(error),
        ReplacementRecoveryErrorV2::Replacement(error) => replacement(error),
        ReplacementRecoveryErrorV2::CandidateAlreadySelected => "cannot_retain_selected_candidate",
        ReplacementRecoveryErrorV2::HandleCheckpointMismatch => "checkpoint_mismatch",
        ReplacementRecoveryErrorV2::FilesChanged => "changed_store",
        ReplacementRecoveryErrorV2::RecoveryIncomplete { .. } => "recovery_incomplete",
        ReplacementRecoveryErrorV2::PublicationUncertain { .. } => "publication_uncertain",
        _ => "invalid_recovery",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_identity_checkpoint_and_ownership_errors_remain_distinct() {
        for (error, class) in [
            (
                NodeControlErrorV2::Record(ControlRecordErrorV2::WrongIdentity),
                "wrong_network",
            ),
            (
                NodeControlErrorV2::Record(ControlRecordErrorV2::WrongCurrentCheckpoint),
                "checkpoint_mismatch",
            ),
            (
                NodeControlErrorV2::Storage(ReplacementStorageErrorV2::Storage(
                    StorageErrorV2::Busy,
                )),
                "busy",
            ),
            (NodeControlErrorV2::RecoveryRequired, "recovery_required"),
            (
                NodeControlErrorV2::CannotRetainConfirmedCandidate,
                "cannot_retain_confirmed_candidate",
            ),
        ] {
            assert_eq!(Failure::from(error).class, class);
        }
        assert_eq!(
            Failure::from(NodeControlErrorV2::RecoveryRequired).next,
            "explicit-recover"
        );
    }

    #[test]
    fn diagnostics_do_not_copy_untrusted_nested_error_text() {
        for error in [
            NodeControlErrorV2::UnsafePath("secret-path\nstatus=clean"),
            NodeControlErrorV2::InvalidLayout("secret-path\nstatus=clean"),
            NodeControlErrorV2::Io(io::Error::other("secret-path\nstatus=clean")),
        ] {
            let failure = Failure::from(error);
            let mut bytes = Vec::new();
            crate::output::diagnostic("status", failure.class, failure.next, None, None)
                .unwrap()
                .write_to(&mut bytes)
                .unwrap();
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("secret-path"));
            assert!(!text.contains("status=clean"));
        }
    }

    #[test]
    fn transport_budget_deadline_and_terminal_failures_remain_distinct() {
        for (error, class) in [
            (SessionErrorV2::WireBudgetExhausted, "peer_io_budget"),
            (SessionErrorV2::DeadlineExceeded, "peer_deadline"),
            (
                SessionErrorV2::Io(io::ErrorKind::TimedOut.into()),
                "peer_deadline",
            ),
            (SessionErrorV2::EarlyClose, "peer_early_close"),
            (SessionErrorV2::TrailingData, "peer_trailing_data"),
            (
                SessionErrorV2::History(HistoryErrorV2::WrongNetwork),
                "peer_wrong_network",
            ),
        ] {
            assert_eq!(Failure::from(error).class, class);
        }
    }
}
