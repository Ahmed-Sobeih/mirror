//! Finite V2 operator commands composed from accepted A1 control and F2 sessions.
//! Serving has one fixed history and explicit admission/shutdown limits.

mod args;
mod errors;
mod output;
mod provider;
mod rejoin;
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
            let uncertain = failure
                .may_have_completed
                .or_else(|| (failure.class == "output_limit").then_some(may_have_effects));
            emit_diagnostic(output::diagnostic(
                operation,
                failure.class,
                failure.next,
                None,
                uncertain,
            ));
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
