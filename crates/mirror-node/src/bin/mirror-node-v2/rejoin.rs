//! One explicit receive, admitted A1 keep/adopt, then the shared finite provider.
//! The controller, sticky signal guard and F2 allowance retain one ownership path.

use std::{net::SocketAddr, time::Duration};

use mirror_chain::v2::{ForkChoiceV2, ManifestIdentityV2};
use mirror_network::v2::{
    ReceivedCandidateV2,
    session::{PeerSessionsV2, SessionLimitsV2, SessionStatsV2},
};
use mirror_node::v2::NodeControlV2;
use mirror_storage::v2::ReplacementCheckpointV2;

use super::{args::Common, errors::Failure, provider, signals::Signals};

pub(super) struct Report {
    pub previous: ReplacementCheckpointV2,
    pub choice: Option<ForkChoiceV2>,
    pub receive: SessionStatsV2,
    pub service: provider::Report,
    pub serve_stats: SessionStatsV2,
}

pub(super) fn run(
    common: &Common,
    peer: SocketAddr,
    listen: SocketAddr,
    limits: SessionLimitsV2,
    admission: Duration,
) -> Result<Report, Failure> {
    // Install once before opening. Every return drops ownership before this
    // guard restores the prior dispositions and before the caller formats output.
    let signals = Signals::install()?;
    let ready = NodeControlV2::open_writer(&common.control, &common.store, common.identity)?;
    let previous = ready.checkpoint();
    let mut sessions = PeerSessionsV2::new(limits);
    if signals.requested() {
        drop(ready);
        return Ok(stopped(previous, sessions.stats()));
    }

    // F2 closes the sole stream before returning a complete candidate or error.
    // A concurrent stop never hides a failed receive or starts another attempt.
    let received = sessions.request(peer, common.identity)?;
    finish_received(
        ready,
        received,
        common.identity,
        listen,
        admission,
        &mut sessions,
        &signals,
    )
}

/// Shared B2/B3 adoption admission and fixed-history serving. The caller retains
/// the original signal guard and transport owner, including earlier failures.
pub(super) fn finish_received(
    ready: NodeControlV2,
    received: ReceivedCandidateV2,
    identity: ManifestIdentityV2,
    listen: SocketAddr,
    admission: Duration,
    sessions: &mut PeerSessionsV2,
    signals: &Signals,
) -> Result<Report, Failure> {
    let previous = ready.checkpoint();
    let receive = sessions.stats();
    if signals.requested() {
        drop(received);
        drop(ready);
        return Ok(stopped(previous, receive));
    }

    // This stop check is adoption admission. A later signal cannot cancel A1's
    // synchronous intent/publication/confirmation sequence or imply rollback.
    let outcome = ready
        .adopt(received)
        .map_err(|error| Failure::from(error).after_adoption())?;
    let adoption = outcome.report();
    let ready = outcome.into_control();
    let service = provider::serve_ready(ready, identity, listen, admission, sessions, signals)
        .map_err(Failure::after_adoption)?;

    // Statistics are snapshots, never a replacement allowance. Refuse an
    // inconsistent normal record rather than wrapping either phase delta.
    let serve_stats = SessionStatsV2 {
        attempts: service
            .stats
            .attempts
            .checked_sub(receive.attempts)
            .ok_or_else(|| Failure::new("lifecycle_limit").after_adoption())?,
        wire_bytes: service
            .stats
            .wire_bytes
            .checked_sub(receive.wire_bytes)
            .ok_or_else(|| Failure::new("lifecycle_limit").after_adoption())?,
    };
    if service.completed.checked_add(service.rejected) != Some(serve_stats.attempts) {
        return Err(Failure::new("lifecycle_limit").after_adoption());
    }
    Ok(Report {
        previous,
        choice: Some(adoption.choice),
        receive,
        service,
        serve_stats,
    })
}

/// A stop before adoption reports the verified old selection and any completed
/// receive. No candidate was compared, and no service attempt or bind occurred.
pub(super) fn stopped(previous: ReplacementCheckpointV2, receive: SessionStatsV2) -> Report {
    Report {
        previous,
        choice: None,
        receive,
        service: provider::Report {
            checkpoint: previous,
            reason: "shutdown",
            stats: receive,
            completed: 0,
            rejected: 0,
            last_session_error: "none",
        },
        serve_stats: SessionStatsV2::default(),
    }
}
