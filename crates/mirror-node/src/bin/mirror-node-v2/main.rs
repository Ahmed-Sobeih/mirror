//! Finite V2 operator commands composed from accepted A1 control and F2 sessions.
//! Serving has one fixed history and explicit admission/shutdown limits.

mod args;
mod errors;
mod failover;
mod output;
mod provider;
mod rejoin;
mod runner;
mod signals;

use std::{
    env, fmt,
    io::{self, Write},
    process::ExitCode,
};

use mirror_chain::v2::ForkChoiceV2;
use mirror_network::v2::session::PeerSessionsV2;
use mirror_node::v2::{NodeControlV2, NodeStatusV2};
use mirror_storage::v2::ReplacementRecoveryActionV2;

use args::Command;
use errors::Failure;
use output::Output;

fn main() -> ExitCode {
    // Admission and every scalar/session-limit validation complete before any
    // control/store lookup, lock acquisition, synchronization or socket operation.
    let command = match args::parse(env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            emit_diagnostic(output::diagnostic(
                "parse",
                "usage",
                "help",
                Some(error.code()),
                None,
            ));
            return ExitCode::from(2);
        }
    };
    let (operation, may_have_effects) = match &command {
        Command::Help => ("help", false),
        Command::Init { .. } => ("init", true),
        Command::Status(_) => ("status", false),
        Command::Sync { .. } => ("sync-once", true),
        Command::Serve { .. } => ("serve", true),
        Command::RejoinServe { .. } => ("rejoin-serve", true),
        Command::RejoinFailoverServe { .. } => ("rejoin-failover-serve", true),
        Command::Run { .. } => ("run", true),
        Command::Recover { .. } => ("recover", true),
    };
    match execute(command) {
        Ok((output, exit)) => {
            if output.write_to(&mut io::stdout().lock()).is_err() {
                // The operation is never repeated because its output was lost.
                emit_diagnostic(output::diagnostic(
                    operation,
                    "output_io",
                    "inspect-status",
                    None,
                    Some(may_have_effects),
                ));
                ExitCode::from(1)
            } else {
                ExitCode::from(exit)
            }
        }
        Err(failure) => {
            emit_diagnostic(failure_diagnostic(operation, &failure, may_have_effects));
            ExitCode::from(1)
        }
    }
}

/// No success record reaches stdout until the whole requested operation succeeds.
/// Output errors preserve the operation's uncertainty and never trigger a retry.
fn execute(command: Command) -> Result<(Output, u8), Failure> {
    match command {
        Command::Help => Ok((output::help()?, 0)),
        Command::Init { common, expected } => {
            let ready = NodeControlV2::initialize(
                &common.control,
                &common.store,
                common.identity,
                expected,
            )?;
            let mut output = Output::state("init", "clean", common.identity, "verified")?;
            output.field("mode", "attach-existing")?;
            output.checkpoint("selected", ready.checkpoint())?;
            Ok((output, 0))
        }
        Command::Status(common) => {
            // Status must not open a writer, contact a peer or attempt recovery.
            match NodeControlV2::status(&common.control, &common.store, common.identity)? {
                NodeStatusV2::Missing => Ok((
                    Output::state("status", "missing", common.identity, "uninspected")?,
                    3,
                )),
                NodeStatusV2::Clean { checkpoint } => {
                    let mut output = Output::state("status", "clean", common.identity, "verified")?;
                    output.checkpoint("selected", checkpoint)?;
                    Ok((output, 0))
                }
                NodeStatusV2::RecoveryRequired {
                    confirmed,
                    old,
                    candidate,
                } => {
                    let mut output = Output::state(
                        "status",
                        "recovery-required",
                        common.identity,
                        "uninspected",
                    )?;
                    output.checkpoint("confirmed", confirmed)?;
                    output.checkpoint("old", old)?;
                    output.checkpoint("candidate", candidate)?;
                    Ok((output, 4))
                }
            }
        }
        Command::Sync {
            common,
            peer,
            limits,
        } => {
            // Clean-only readiness comes first. This same owner keeps both locks
            // throughout exactly one request and consuming adoption. The identity
            // is independently supplied by the operator, never by decoded data.
            let ready =
                NodeControlV2::open_writer(&common.control, &common.store, common.identity)?;
            let mut sessions = PeerSessionsV2::new(limits);
            let received = sessions.request(peer, common.identity)?;
            let outcome = ready.adopt(received)?;
            let report = outcome.report();
            let stats = sessions.stats();
            let mut output = Output::state("sync-once", "clean", common.identity, "verified")?;
            output.field(
                "choice",
                match report.choice {
                    ForkChoiceV2::KeepCurrent => "keep-current",
                    ForkChoiceV2::PreferCandidate => "prefer-candidate",
                },
            )?;
            output.checkpoint("previous", report.previous)?;
            output.checkpoint("selected", report.selected)?;
            output.field("attempts", stats.attempts)?;
            output.field("wire_bytes", stats.wire_bytes)?;
            Ok((output, 0))
        }
        Command::Serve {
            common,
            listen,
            limits,
            admission,
        } => {
            // The provider releases every socket and both directory owners
            // before this result is formatted or written to a possibly blocked
            // output consumer. Serving is never repeated after output loss.
            let report = provider::run(&common, listen, limits, admission)?;
            let mut output = Output::state("serve", "stopped", common.identity, "verified")?;
            output.checkpoint("selected", report.checkpoint)?;
            output.field("reason", report.reason)?;
            output.field("attempts", report.stats.attempts)?;
            output.field("completed_sessions", report.completed)?;
            output.field("rejected_sessions", report.rejected)?;
            output.field("wire_bytes", report.stats.wire_bytes)?;
            output.field("last_session_error", report.last_session_error)?;
            Ok((output, 0))
        }
        Command::RejoinServe {
            common,
            peer,
            listen,
            limits,
            admission,
        } => {
            // The finite lifecycle owns all resources. Its return releases them
            // before any formatting or output, including a blocked stdout.
            let report = rejoin::run(&common, peer, listen, limits, admission)?;
            let mut output = Output::state("rejoin-serve", "stopped", common.identity, "verified")?;
            output.checkpoint("previous", report.previous)?;
            output.checkpoint("selected", report.service.checkpoint)?;
            output.field(
                "receive_status",
                if report.receive.attempts == 0 {
                    "not-started"
                } else {
                    "validated"
                },
            )?;
            output.field(
                "choice",
                match report.choice {
                    None => "not-attempted",
                    Some(ForkChoiceV2::KeepCurrent) => "keep-current",
                    Some(ForkChoiceV2::PreferCandidate) => "prefer-candidate",
                },
            )?;
            output.field("reason", report.service.reason)?;
            output.field("attempts", report.service.stats.attempts)?;
            output.field("receive_attempts", report.receive.attempts)?;
            output.field("serve_attempts", report.serve_stats.attempts)?;
            output.field("completed_sessions", report.service.completed)?;
            output.field("rejected_sessions", report.service.rejected)?;
            output.field("wire_bytes", report.service.stats.wire_bytes)?;
            output.field("receive_wire_bytes", report.receive.wire_bytes)?;
            output.field("serve_wire_bytes", report.serve_stats.wire_bytes)?;
            output.field("last_session_error", report.service.last_session_error)?;
            Ok((output, 0))
        }
        Command::RejoinFailoverServe {
            common,
            peers,
            listen,
            limits,
            admission,
        } => {
            let report = failover::run(&common, peers.as_slice(), listen, limits, admission)?;
            let lifecycle = report.lifecycle;
            if lifecycle.receive != report.acquisition.stats {
                return Err(Failure::new("lifecycle_limit")
                    .after_adoption()
                    .with_acquisition(report.acquisition));
            }
            // The lifecycle has released ownership and sockets before either
            // the successful report or a bounded diagnostic is formatted.
            let mut output = Output::state(
                "rejoin-failover-serve",
                "stopped",
                common.identity,
                "verified",
            )?;
            output.checkpoint("previous", lifecycle.previous)?;
            output.checkpoint("selected", lifecycle.service.checkpoint)?;
            output.field(
                "receive_status",
                if lifecycle.receive.attempts == 0 {
                    "not-started"
                } else {
                    "validated"
                },
            )?;
            output.field(
                "choice",
                match lifecycle.choice {
                    None => "not-attempted",
                    Some(ForkChoiceV2::KeepCurrent) => "keep-current",
                    Some(ForkChoiceV2::PreferCandidate) => "prefer-candidate",
                },
            )?;
            output.field("reason", lifecycle.service.reason)?;
            output.field("attempts", lifecycle.service.stats.attempts)?;
            output.field("receive_attempts", lifecycle.receive.attempts)?;
            output.field("serve_attempts", lifecycle.serve_stats.attempts)?;
            output.field("completed_sessions", lifecycle.service.completed)?;
            output.field("rejected_sessions", lifecycle.service.rejected)?;
            output.field("wire_bytes", lifecycle.service.stats.wire_bytes)?;
            output.field("receive_wire_bytes", lifecycle.receive.wire_bytes)?;
            output.field("serve_wire_bytes", lifecycle.serve_stats.wire_bytes)?;
            output.field("last_session_error", lifecycle.service.last_session_error)?;
            output::acquisition_context(&mut output, &report.acquisition, false)?;
            Ok((output, 0))
        }
        Command::Run {
            common,
            peers,
            listen,
            limits,
            admission,
            max_rounds,
            round_wait,
        } => {
            let report = runner::run(
                &common,
                peers.as_slice(),
                listen,
                limits,
                admission,
                max_rounds,
                round_wait,
            )?;
            // The runner has released its one ownership path before bounded
            // validation, formatting and potentially blocked output begin.
            Ok((output::run_report(common.identity, &report)?, 0))
        }
        Command::Recover { common, action } => {
            let ready =
                NodeControlV2::recover(&common.control, &common.store, common.identity, action)?;
            let mut output = Output::state("recover", "clean", common.identity, "verified")?;
            output.field(
                "action",
                match action {
                    ReplacementRecoveryActionV2::Complete => "complete",
                    ReplacementRecoveryActionV2::RetainOld => "retain-old",
                },
            )?;
            output.checkpoint("selected", ready.checkpoint())?;
            Ok((output, 0))
        }
    }
}

fn failure_diagnostic(
    operation: &'static str,
    failure: &Failure,
    may_have_effects: bool,
) -> Result<Output, fmt::Error> {
    let uncertain = failure
        .may_have_completed
        .or_else(|| (failure.class == "output_limit").then_some(may_have_effects));
    let base = || output::diagnostic(operation, failure.class, failure.next, None, uncertain);
    let mut record = base()?;
    if let Some(run) = failure.run {
        if output::run_context(&mut record, &run).is_err() {
            record = base()?;
        }
    } else if let Some(acquisition) = failure.acquisition {
        if output::acquisition_context(&mut record, &acquisition, true).is_err() {
            // Discard any partial context, retaining the original safe class,
            // next action and possible-completion warning. Internal reporting
            // failure must not erase uncertainty after admitted adoption.
            record = base()?;
        }
    }
    Ok(record)
}

fn emit_diagnostic(record: Result<Output, fmt::Error>) {
    let mut stderr = io::stderr().lock();
    if let Ok(record) = record {
        let _ = record.write_to(&mut stderr);
    } else {
        // This static fallback is also bounded. A failed error channel cannot be
        // repaired here; the original failure exit remains authoritative.
        let _ = stderr.write_all(
            b"format=mirror-node-v2-1\noperation=output\nerror=output_limit\nnext=inspect-status\n",
        );
        let _ = stderr.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirror_network::v2::session::SessionStatsV2;

    #[test]
    fn malformed_failover_evidence_cannot_erase_possible_completion_or_old_diagnostics() {
        let malformed = failover::Acquisition {
            peer_count: 4,
            validated_peer: 2,
            failed_receives: 0,
            last_receive_error: "none",
            stop: "validated",
            results: ["not-attempted"; 4],
            stats: SessionStatsV2::default(),
        };
        let failure = Failure::new("listen_bind")
            .after_adoption()
            .with_acquisition(malformed);
        let mut bytes = Vec::new();
        failure_diagnostic("rejoin-failover-serve", &failure, true)
            .unwrap()
            .write_to(&mut bytes)
            .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("error=listen_bind\n"));
        assert!(text.contains("operation_may_have_completed=true\n"));
        assert!(text.contains("next=inspect-status\n"));
        assert!(!text.contains("acquisition_"));
        assert!(!text.contains("validated_peer="));
        assert!(!text.contains("selected_"));
        let mut bytes = Vec::new();
        failure_diagnostic("status", &Failure::new("busy"), false)
            .unwrap()
            .write_to(&mut bytes)
            .unwrap();
        assert_eq!(
            bytes,
            b"format=mirror-node-v2-1\noperation=status\nerror=busy\nnext=inspect-status\n"
        );
    }

    #[test]
    fn malformed_run_evidence_retains_safe_diagnostic_and_never_leaks_partial_context() {
        let acquisition = failover::Acquisition {
            peer_count: 4,
            validated_peer: 2,
            failed_receives: 0,
            last_receive_error: "none",
            stop: "validated",
            results: ["not-attempted"; 4],
            stats: SessionStatsV2::default(),
        };
        let trace = runner::RunTrace {
            round_limit: 4,
            rounds_started: 2,
            rounds_completed: 1,
            peer_count: 4,
            phase: "adoption-service",
            stats: SessionStatsV2 {
                attempts: 3,
                wire_bytes: 2000,
            },
            receive: SessionStatsV2 {
                attempts: 2,
                wire_bytes: 1000,
            },
            failed_receives: 0,
            last_receive_error: "none",
            last_completed_service_error: "none",
            acquisition: Some(acquisition),
            adoption_admitted: true,
        };
        let failure = Failure::new("listen_bind")
            .with_acquisition(acquisition)
            .with_run(trace);
        let mut bytes = Vec::new();
        failure_diagnostic("run", &failure, true)
            .unwrap()
            .write_to(&mut bytes)
            .unwrap();
        assert_eq!(bytes, b"format=mirror-node-v2-1\noperation=run\nerror=listen_bind\noperation_may_have_completed=true\nnext=inspect-status\n");
        let mut bytes = Vec::new();
        failure_diagnostic("status", &Failure::new("busy"), false)
            .unwrap()
            .write_to(&mut bytes)
            .unwrap();
        assert_eq!(
            bytes,
            b"format=mirror-node-v2-1\noperation=status\nerror=busy\nnext=inspect-status\n"
        );
    }
}
