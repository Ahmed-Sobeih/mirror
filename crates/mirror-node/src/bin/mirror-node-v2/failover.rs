//! An ordered, finite acquisition prefix followed by the unchanged B2 lifecycle.
//! Only an explicitly reviewed typed failure may advance to another entry.

use std::{io, net::SocketAddr, time::Duration};

use mirror_chain::v2::{ChainErrorV2, ManifestErrorV2, ManifestIdentityV2};
use mirror_consensus::v2::PowErrorV2;
use mirror_core::v2::BlockErrorV2;
use mirror_network::v2::{
    HistoryErrorV2, ReceivedCandidateV2,
    session::{PeerSessionsV2, SessionErrorV2, SessionLimitsV2, SessionStatsV2},
};
use mirror_node::v2::NodeControlV2;
use mirror_protocol::v2::ProtocolErrorV2;

use super::{
    args::{Common, MAX_FAILOVER_PEERS},
    errors::Failure,
    rejoin,
    signals::Signals,
};

/// Bounded, sanitized evidence about acquisition only. No endpoint or owner is
/// retained, and later service cannot alter or replenish these snapshots.
#[derive(Clone, Copy, Debug)]
pub(super) struct Acquisition {
    pub peer_count: u8,
    pub validated_peer: u8,
    pub failed_receives: u8,
    pub last_receive_error: &'static str,
    pub stop: &'static str,
    pub results: [&'static str; MAX_FAILOVER_PEERS],
    pub stats: SessionStatsV2,
}

impl Acquisition {
    pub(super) fn new(peer_count: u8) -> Self {
        Self {
            peer_count,
            validated_peer: 0,
            failed_receives: 0,
            last_receive_error: "none",
            stop: "startup",
            results: ["not-attempted"; MAX_FAILOVER_PEERS],
            stats: SessionStatsV2::default(),
        }
    }

    /// Validate an ordered prefix without trusting a copied count to stand in
    /// for the actual charged owner. Used before admitting a candidate and before
    /// returning normal output; unresolved errors do not assert selected state.
    pub(super) fn consistent(&self) -> bool {
        let count = usize::from(self.peer_count);
        let attempts = usize::from(self.stats.attempts);
        if count == 0
            || count > MAX_FAILOVER_PEERS
            || attempts > count
            || self
                .failed_receives
                .checked_add(u8::from(self.validated_peer != 0))
                != Some(self.stats.attempts)
            || (self.validated_peer != 0 && self.validated_peer != self.stats.attempts)
        {
            return false;
        }
        for (index, result) in self.results.iter().enumerate() {
            if index >= attempts {
                if *result != "not-attempted" {
                    return false;
                }
            } else if self.validated_peer != 0 && index + 1 == attempts {
                if *result != "validated" {
                    return false;
                }
            } else if matches!(*result, "not-attempted" | "validated" | "none") {
                return false;
            }
        }
        if self.failed_receives == 0 {
            self.last_receive_error == "none"
        } else {
            // Every failure precedes the sole optional validated result.
            self.results[usize::from(self.failed_receives) - 1] == self.last_receive_error
        }
    }
}

pub(super) struct Report {
    pub lifecycle: rejoin::Report,
    pub acquisition: Acquisition,
}

pub(super) fn run(
    common: &Common,
    peers: &[SocketAddr],
    listen: SocketAddr,
    limits: SessionLimitsV2,
    admission: Duration,
) -> Result<Report, Failure> {
    // Argument parsing already supplies 1..=4 peers. Keep this private boundary
    // fail-closed as well, without indexing an invalid list or creating effects.
    let count = u8::try_from(peers.len())
        .ok()
        .filter(|count| *count != 0 && usize::from(*count) <= MAX_FAILOVER_PEERS);
    let mut acquisition = Acquisition::new(count.unwrap_or(0));
    let result = if count.is_some() {
        run_owned(common, peers, listen, limits, admission, &mut acquisition)
    } else {
        acquisition.stop = "fatal";
        Err(Failure::new("lifecycle_limit"))
    };

    // run_owned has returned: every socket, listener, directory owner and signal
    // guard has already dropped before either success or error reaches formatting.
    match result {
        Ok(lifecycle) => {
            if !acquisition.consistent() || lifecycle.receive != acquisition.stats {
                let failure = Failure::new("lifecycle_limit");
                let failure = if lifecycle.choice.is_some() {
                    failure.after_adoption()
                } else {
                    failure
                };
                return Err(failure.with_acquisition(acquisition));
            }
            Ok(Report {
                lifecycle,
                acquisition,
            })
        }
        Err(failure) => Err(failure.with_acquisition(acquisition)),
    }
}

fn run_owned(
    common: &Common,
    peers: &[SocketAddr],
    listen: SocketAddr,
    limits: SessionLimitsV2,
    admission: Duration,
    acquisition: &mut Acquisition,
) -> Result<rejoin::Report, Failure> {
    let signals = Signals::install()?;
    let ready = NodeControlV2::open_writer(&common.control, &common.store, common.identity)?;
    let previous = ready.checkpoint();
    let mut sessions = PeerSessionsV2::new(limits);
    match acquire(peers, common.identity, &mut sessions, &signals, acquisition)? {
        Some(received) => rejoin::finish_received(
            ready,
            received,
            common.identity,
            listen,
            admission,
            &mut sessions,
            &signals,
        ),
        None => {
            drop(ready);
            Ok(rejoin::stopped(previous, sessions.stats()))
        }
    }
}

/// Acquire one ordered prefix using an already retained cumulative owner.
/// Only the trace is fresh per round; actual attempts and bytes never reset.
/// None represents shutdown before any request in this acquisition phase.
pub(super) fn acquire(
    peers: &[SocketAddr],
    identity: ManifestIdentityV2,
    sessions: &mut PeerSessionsV2,
    signals: &Signals,
    acquisition: &mut Acquisition,
) -> Result<Option<ReceivedCandidateV2>, Failure> {
    if usize::from(acquisition.peer_count) != peers.len()
        || acquisition.stop != "startup"
        || acquisition.stats != SessionStatsV2::default()
        || !acquisition.consistent()
    {
        acquisition.stop = "fatal";
        return Err(Failure::new("lifecycle_limit"));
    }
    let base = sessions.stats();
    let limits = sessions.limits();
    let mut last_failure = None;

    for (index, peer) in peers.iter().enumerate() {
        // A signal between failed requests preserves the actual latest failure.
        // Before this round's first request it is a normal incomplete shutdown.
        if signals.requested() {
            acquisition.stop = "shutdown";
            return match last_failure {
                Some(failure) => Err(failure),
                None => Ok(None),
            };
        }
        if let Some(reason) = exhausted(sessions.stats(), limits) {
            acquisition.stop = reason;
            return Err(last_failure.unwrap_or_else(|| {
                Failure::new(if reason == "wire-limit" {
                    "peer_io_budget"
                } else {
                    "peer_attempt_budget"
                })
            }));
        }

        let before = sessions.stats();
        let result = sessions.request(*peer, identity);
        let after = sessions.stats();
        let Some(round_stats) = usage_since(after, base) else {
            acquisition.stop = "fatal";
            return Err(Failure::new("lifecycle_limit"));
        };
        acquisition.stats = round_stats;
        let charged = before.attempts.checked_add(1) == Some(after.attempts);
        if after.wire_bytes < before.wire_bytes
            || after.wire_bytes > limits.max_wire_bytes()
            || after.attempts > limits.max_attempts()
            || before.attempts.checked_sub(base.attempts).map(usize::from) != Some(index)
            || (!charged && after != before)
        {
            acquisition.stop = "fatal";
            return Err(Failure::new("lifecycle_limit"));
        }

        match result {
            Ok(received) => {
                if !charged {
                    acquisition.stop = "fatal";
                    return Err(Failure::new("lifecycle_limit"));
                }
                acquisition.results[index] = "validated";
                acquisition.validated_peer = round_stats.attempts;
                acquisition.stop = "validated";
                if !acquisition.consistent() {
                    acquisition.stop = "fatal";
                    return Err(Failure::new("lifecycle_limit"));
                }
                return Ok(Some(received));
            }
            Err(error) => {
                // Preserve the typed distinction before the output class merges
                // malformed inputs with allocation or defensive failures.
                let may_advance = charged && advance_allowed(&error);
                let fatal_reason = match &error {
                    SessionErrorV2::WireBudgetExhausted => "wire-limit",
                    SessionErrorV2::AttemptsExhausted => "attempt-limit",
                    _ => "fatal",
                };
                let failure = Failure::from(error);
                if charged {
                    acquisition.results[index] = failure.class;
                    acquisition.last_receive_error = failure.class;
                    acquisition.failed_receives = match acquisition.failed_receives.checked_add(1) {
                        Some(count) => count,
                        None => {
                            acquisition.stop = "fatal";
                            return Err(Failure::new("lifecycle_limit"));
                        }
                    };
                }
                if !may_advance {
                    acquisition.stop = fatal_reason;
                    return Err(failure);
                }
                // A failed request is never converted into normal shutdown or
                // hidden by a later budget check. The terminal label supplements
                // the original failure; it does not invent an extra request.
                let reason = if signals.requested() {
                    Some("shutdown")
                } else if let Some(reason) = exhausted(after, limits) {
                    Some(reason)
                } else if index + 1 == peers.len() {
                    Some("peers-exhausted")
                } else {
                    None
                };
                if let Some(reason) = reason {
                    acquisition.stop = reason;
                    return Err(failure);
                }
                last_failure = Some(failure);
            }
        }
    }
    // A nonempty list either validates or returns its final failure above.
    acquisition.stop = "fatal";
    Err(Failure::new("lifecycle_limit"))
}

fn usage_since(after: SessionStatsV2, base: SessionStatsV2) -> Option<SessionStatsV2> {
    Some(SessionStatsV2 {
        attempts: after.attempts.checked_sub(base.attempts)?,
        wire_bytes: after.wire_bytes.checked_sub(base.wire_bytes)?,
    })
}

pub(super) fn exhausted(stats: SessionStatsV2, limits: SessionLimitsV2) -> Option<&'static str> {
    if stats.wire_bytes >= limits.max_wire_bytes() {
        Some("wire-limit")
    } else if stats.attempts >= limits.max_attempts() {
        Some("attempt-limit")
    } else {
        None
    }
}

/// Closed policy from NODE_FAILOVER_V2.md. Newly introduced or ambiguous error
/// variants remain fatal until their precise requester provenance is reviewed.
fn advance_allowed(error: &SessionErrorV2) -> bool {
    match error {
        SessionErrorV2::DeadlineExceeded
        | SessionErrorV2::EarlyClose
        | SessionErrorV2::TrailingData => true,
        SessionErrorV2::Io(error) => matches!(
            error.kind(),
            io::ErrorKind::ConnectionRefused
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::NetworkUnreachable
                | io::ErrorKind::HostUnreachable
                | io::ErrorKind::BrokenPipe
                | io::ErrorKind::NotConnected
                | io::ErrorKind::TimedOut
                | io::ErrorKind::WouldBlock
                | io::ErrorKind::WriteZero
        ),
        SessionErrorV2::History(error) => history_advance_allowed(error),
        _ => false,
    }
}

fn history_advance_allowed(error: &HistoryErrorV2) -> bool {
    match error {
        HistoryErrorV2::WrongNetwork
        | HistoryErrorV2::UnexpectedFrame { .. }
        | HistoryErrorV2::WrongPayloadLength { .. }
        | HistoryErrorV2::WrongBlockHeight { .. }
        | HistoryErrorV2::BlockTooLarge { .. }
        | HistoryErrorV2::BlockBytesMismatch
        | HistoryErrorV2::EndpointMismatch
        | HistoryErrorV2::ChangedSummary => true,
        HistoryErrorV2::Protocol(error) => matches!(
            error,
            ProtocolErrorV2::InvalidMagic
                | ProtocolErrorV2::UnsupportedVersion(_)
                | ProtocolErrorV2::UnknownKind(_)
                | ProtocolErrorV2::InvalidPayloadLength { .. }
                | ProtocolErrorV2::InvalidBlockCount(_)
                | ProtocolErrorV2::InvalidHeight { .. }
                | ProtocolErrorV2::InvalidManifestLength(_)
                | ProtocolErrorV2::InvalidBlockBytes(_)
                | ProtocolErrorV2::BlockHeightOutOfRange(_)
        ),
        HistoryErrorV2::Manifest(error) => matches!(
            error,
            ManifestErrorV2::InvalidMagic
                | ManifestErrorV2::UnsupportedVersion(_)
                | ManifestErrorV2::Truncated
                | ManifestErrorV2::TrailingBytes
                | ManifestErrorV2::TooLarge
                | ManifestErrorV2::ConfigTooLarge
                | ManifestErrorV2::TooManyProofs
                | ManifestErrorV2::WrongProofCount
                | ManifestErrorV2::WrongConfiguration
                | ManifestErrorV2::WrongGenesis
        ),
        HistoryErrorV2::Block(error) => block_advance_allowed(error),
        HistoryErrorV2::Chain(error) => match error {
            ChainErrorV2::WrongConfiguration
            | ChainErrorV2::WrongHeight { .. }
            | ChainErrorV2::WrongParent
            | ChainErrorV2::InvalidTimestamp { .. }
            | ChainErrorV2::WrongGenesisTimestamp { .. }
            | ChainErrorV2::GenesisHasOperations
            | ChainErrorV2::WrongGenesisHash
            | ChainErrorV2::WrongOperationsRoot
            | ChainErrorV2::WrongLedgerRoot => true,
            ChainErrorV2::Body(error) => block_advance_allowed(error),
            ChainErrorV2::Work(error) => matches!(
                error,
                PowErrorV2::WrongBits { .. } | PowErrorV2::HashAboveTarget
            ),
            _ => false,
        },
        _ => false,
    }
}

fn block_advance_allowed(error: &BlockErrorV2) -> bool {
    matches!(
        error,
        BlockErrorV2::UnsupportedVersion(_)
            | BlockErrorV2::InvalidMagic
            | BlockErrorV2::Truncated
            | BlockErrorV2::TrailingBytes
            | BlockErrorV2::TooLarge
            | BlockErrorV2::TooManyOperations
            | BlockErrorV2::TooManyMembershipOperations
            | BlockErrorV2::TooManyEnrollments
            | BlockErrorV2::OperationTooLarge { .. }
            | BlockErrorV2::WrongContext
            | BlockErrorV2::OperationContext { .. }
            | BlockErrorV2::OperationsRootMismatch
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use mirror_consensus::PowError;
    use mirror_core::v2::{TransferErrorV2, UnifiedOperationErrorV2};
    use mirror_personhood::v2::{ConfigError, StateCodecErrorV2, StateErrorV2};
    use mirror_protocol::v2::FrameKindV2;
    use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2};

    fn allowed(error: SessionErrorV2) {
        assert!(
            advance_allowed(&error),
            "permitted failure became fatal: {error:?}"
        );
    }

    fn fatal(error: SessionErrorV2) {
        assert!(
            !advance_allowed(&error),
            "fatal failure permitted advancement: {error:?}"
        );
    }

    #[test]
    fn acquisition_advances_only_for_named_transport_failures() {
        for error in [
            SessionErrorV2::DeadlineExceeded,
            SessionErrorV2::EarlyClose,
            SessionErrorV2::TrailingData,
        ] {
            allowed(error);
        }
        for kind in [
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionAborted,
            io::ErrorKind::NetworkUnreachable,
            io::ErrorKind::HostUnreachable,
            io::ErrorKind::BrokenPipe,
            io::ErrorKind::NotConnected,
            io::ErrorKind::TimedOut,
            io::ErrorKind::WouldBlock,
            io::ErrorKind::WriteZero,
        ] {
            allowed(SessionErrorV2::Io(kind.into()));
        }
        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::AddrInUse,
            io::ErrorKind::AddrNotAvailable,
            io::ErrorKind::NetworkDown,
            io::ErrorKind::Interrupted,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::InvalidData,
            io::ErrorKind::Unsupported,
            io::ErrorKind::OutOfMemory,
            io::ErrorKind::UnexpectedEof,
            io::ErrorKind::Other,
        ] {
            fatal(SessionErrorV2::Io(kind.into()));
        }
        // Even text identical to a permitted category is not authority.
        fatal(SessionErrorV2::Io(io::Error::other("peer_unreachable")));
    }

    #[test]
    fn acquisition_keeps_session_authority_and_budget_failures_fatal() {
        for error in [
            SessionErrorV2::InvalidLimits("deadline overflow"),
            SessionErrorV2::InvalidRequest,
            SessionErrorV2::WrongNetwork,
            SessionErrorV2::AttemptsExhausted,
            SessionErrorV2::WireBudgetExhausted,
            SessionErrorV2::AllocationFailed,
        ] {
            fatal(error);
        }
        allowed(SessionErrorV2::History(HistoryErrorV2::WrongNetwork));
        // The output category is intentionally coarser than the typed policy.
        assert_eq!(
            Failure::from(SessionErrorV2::WrongNetwork).class,
            Failure::from(SessionErrorV2::History(HistoryErrorV2::WrongNetwork)).class
        );
    }

    #[test]
    fn acquisition_distinguishes_peer_claims_from_receiver_invariants() {
        for error in [
            HistoryErrorV2::WrongNetwork,
            HistoryErrorV2::UnexpectedFrame {
                expected: FrameKindV2::Begin,
                got: FrameKindV2::End,
            },
            HistoryErrorV2::WrongPayloadLength {
                expected: 152,
                got: 153,
            },
            HistoryErrorV2::WrongBlockHeight {
                expected: 1,
                got: 2,
            },
            HistoryErrorV2::BlockTooLarge {
                len: 1000,
                max: 999,
            },
            HistoryErrorV2::BlockBytesMismatch,
            HistoryErrorV2::EndpointMismatch,
            HistoryErrorV2::ChangedSummary,
        ] {
            allowed(SessionErrorV2::History(error));
        }
        for error in [
            HistoryErrorV2::AlreadyComplete,
            HistoryErrorV2::IncompleteFrame,
            HistoryErrorV2::IncompleteHistory,
            HistoryErrorV2::FrameBudget,
            HistoryErrorV2::WireBudget,
            HistoryErrorV2::AccountingMismatch,
            HistoryErrorV2::UnexpectedPhase,
            HistoryErrorV2::LengthOverflow,
        ] {
            fatal(SessionErrorV2::History(error));
        }
        assert_eq!(
            Failure::from(SessionErrorV2::History(HistoryErrorV2::ChangedSummary)).class,
            Failure::from(SessionErrorV2::History(HistoryErrorV2::AccountingMismatch)).class
        );
    }

    #[test]
    fn acquisition_protocol_policy_excludes_local_lengths_and_allocation() {
        for error in [
            ProtocolErrorV2::InvalidMagic,
            ProtocolErrorV2::UnsupportedVersion(1),
            ProtocolErrorV2::UnknownKind(9),
            ProtocolErrorV2::InvalidPayloadLength {
                kind: FrameKindV2::Begin,
                len: 153,
                min: 152,
                max: 152,
            },
            ProtocolErrorV2::InvalidBlockCount(0),
            ProtocolErrorV2::InvalidHeight {
                height: 1,
                block_count: 1,
            },
            ProtocolErrorV2::InvalidManifestLength(0),
            ProtocolErrorV2::InvalidBlockBytes(0),
            ProtocolErrorV2::BlockHeightOutOfRange(4096),
        ] {
            allowed(SessionErrorV2::History(HistoryErrorV2::Protocol(error)));
        }
        for error in [
            ProtocolErrorV2::InvalidHeaderLength(11),
            ProtocolErrorV2::InvalidSummaryLength(151),
            ProtocolErrorV2::TruncatedFrame,
            ProtocolErrorV2::TrailingBytes,
            ProtocolErrorV2::LengthOverflow,
            ProtocolErrorV2::AllocationFailed,
        ] {
            fatal(SessionErrorV2::History(HistoryErrorV2::Protocol(error)));
        }
    }

    #[test]
    fn acquisition_manifest_policy_keeps_nested_configuration_errors_fatal() {
        for error in [
            ManifestErrorV2::InvalidMagic,
            ManifestErrorV2::UnsupportedVersion(1),
            ManifestErrorV2::Truncated,
            ManifestErrorV2::TrailingBytes,
            ManifestErrorV2::TooLarge,
            ManifestErrorV2::ConfigTooLarge,
            ManifestErrorV2::TooManyProofs,
            ManifestErrorV2::WrongProofCount,
            ManifestErrorV2::WrongConfiguration,
            ManifestErrorV2::WrongGenesis,
        ] {
            allowed(SessionErrorV2::History(HistoryErrorV2::Manifest(error)));
        }
        for error in [
            ManifestErrorV2::Config(ConfigError::InvalidLimits),
            ManifestErrorV2::Config(ConfigError::WrongProofCount),
            ManifestErrorV2::Config(ConfigError::SupplyOverflow),
            ManifestErrorV2::LengthOverflow,
            ManifestErrorV2::AllocationFailed,
        ] {
            fatal(SessionErrorV2::History(HistoryErrorV2::Manifest(error)));
        }
    }

    #[test]
    fn acquisition_block_policy_is_identical_directly_and_inside_chain_errors() {
        for error in [
            BlockErrorV2::UnsupportedVersion(1),
            BlockErrorV2::InvalidMagic,
            BlockErrorV2::Truncated,
            BlockErrorV2::TrailingBytes,
            BlockErrorV2::TooLarge,
            BlockErrorV2::TooManyOperations,
            BlockErrorV2::TooManyMembershipOperations,
            BlockErrorV2::TooManyEnrollments,
            BlockErrorV2::OperationTooLarge { index: 0 },
            BlockErrorV2::WrongContext,
            BlockErrorV2::OperationContext { index: 0 },
            BlockErrorV2::OperationsRootMismatch,
        ] {
            allowed(SessionErrorV2::History(HistoryErrorV2::Block(error)));
            allowed(SessionErrorV2::History(HistoryErrorV2::Chain(
                ChainErrorV2::Body(error),
            )));
        }
        for error in [
            BlockErrorV2::InvalidHeaderLength,
            BlockErrorV2::LengthOverflow,
            BlockErrorV2::Operation {
                index: 0,
                error: UnifiedOperationErrorV2::InvalidLength,
            },
            BlockErrorV2::Operation {
                index: 1,
                error: UnifiedOperationErrorV2::Transfer(TransferErrorV2::WindowOverflow),
            },
        ] {
            fatal(SessionErrorV2::History(HistoryErrorV2::Block(error)));
            fatal(SessionErrorV2::History(HistoryErrorV2::Chain(
                ChainErrorV2::Body(error),
            )));
        }
    }

    #[test]
    fn acquisition_chain_policy_separates_peer_block_claims_from_work_limits() {
        for error in [
            ChainErrorV2::WrongConfiguration,
            ChainErrorV2::WrongHeight {
                expected: 1,
                got: 2,
            },
            ChainErrorV2::WrongParent,
            ChainErrorV2::InvalidTimestamp { parent: 1, got: 0 },
            ChainErrorV2::WrongGenesisTimestamp {
                expected: 1,
                got: 2,
            },
            ChainErrorV2::GenesisHasOperations,
            ChainErrorV2::WrongGenesisHash,
            ChainErrorV2::WrongOperationsRoot,
            ChainErrorV2::WrongLedgerRoot,
            ChainErrorV2::Work(PowErrorV2::WrongBits {
                expected: 1,
                got: 2,
            }),
            ChainErrorV2::Work(PowErrorV2::HashAboveTarget),
        ] {
            allowed(SessionErrorV2::History(HistoryErrorV2::Chain(error)));
        }
        for error in [
            ChainErrorV2::HeightOverflow,
            ChainErrorV2::EmptyHistory,
            ChainErrorV2::TooManyBlocks,
            ChainErrorV2::TooManyBytes,
            ChainErrorV2::StateItemLimit,
            ChainErrorV2::LengthOverflow,
            ChainErrorV2::AllocationFailed,
            ChainErrorV2::Work(PowErrorV2::Target(PowError::ZeroTarget)),
            ChainErrorV2::Work(PowErrorV2::InvalidAttemptBudget {
                requested: 0,
                maximum: 1,
            }),
            ChainErrorV2::Work(PowErrorV2::AttemptsExhausted { attempts: 1 }),
            ChainErrorV2::Work(PowErrorV2::NonceExhausted { attempts: 1 }),
        ] {
            fatal(SessionErrorV2::History(HistoryErrorV2::Chain(error)));
        }
    }

    #[test]
    fn acquisition_never_downgrades_nested_ledger_allocation_or_state_failures() {
        for error in [
            LedgerErrorV2::AllocationFailed,
            LedgerErrorV2::LengthOverflow,
            LedgerErrorV2::BalanceSumOverflow,
            LedgerErrorV2::AccountLimit,
            LedgerErrorV2::SupplyMismatch {
                expected: 1,
                got: 0,
            },
            LedgerErrorV2::Limits(BlockErrorV2::LengthOverflow),
            LedgerErrorV2::Membership(StateErrorV2::InconsistentState("synthetic invariant")),
            LedgerErrorV2::Membership(StateErrorV2::ApprovalTotalLimit),
            LedgerErrorV2::MembershipCodec(StateCodecErrorV2::AllocationFailed),
            LedgerErrorV2::MembershipCodec(StateCodecErrorV2::Inconsistent("synthetic invariant")),
            LedgerErrorV2::Operation {
                index: 0,
                error: LedgerOperationErrorV2::Membership(StateErrorV2::CounterOverflow),
            },
            LedgerErrorV2::Operation {
                index: 1,
                error: LedgerOperationErrorV2::InsufficientBalance {
                    available: 0,
                    required: 1,
                },
            },
        ] {
            fatal(SessionErrorV2::History(HistoryErrorV2::Chain(
                ChainErrorV2::Ledger(error),
            )));
        }
    }
}
