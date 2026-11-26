//! One fixed-history provider with one admission loop and one F2 budget owner.
//! No output is performed while sockets or directory ownership remain live.

use std::{
    io,
    net::{Shutdown, SocketAddr, TcpListener},
    thread,
    time::{Duration, Instant},
};

use mirror_network::v2::session::{
    PeerSessionsV2, SessionErrorV2, SessionLimitsV2, SessionStatsV2,
};
use mirror_node::v2::NodeControlV2;
use mirror_storage::v2::ReplacementCheckpointV2;

use super::{args::Common, errors::Failure, signals::Signals};

pub(super) struct Report {
    pub checkpoint: ReplacementCheckpointV2,
    pub reason: &'static str,
    pub stats: SessionStatsV2,
    pub completed: u8,
    pub rejected: u8,
    pub last_session_error: &'static str,
}

pub(super) fn run(
    common: &Common,
    listen: SocketAddr,
    limits: SessionLimitsV2,
    admission: Duration,
) -> Result<Report, Failure> {
    // Signal setup happens only after complete effect-free command parsing.
    // The guard restores prior dispositions after all owned resources drop.
    let signals = Signals::install()?;
    let ready = NodeControlV2::open_writer(&common.control, &common.store, common.identity)?;
    let checkpoint = ready.checkpoint();
    let mut sessions = PeerSessionsV2::new(limits);
    let mut completed = 0;
    let mut rejected = 0;
    let mut last_session_error = "none";

    // Opening/replay is separate from the finite admission window. A stop
    // delivered during opening is honored before binding any listening socket.
    if signals.requested() {
        return Ok(Report {
            checkpoint,
            reason: "shutdown",
            stats: sessions.stats(),
            completed,
            rejected,
            last_session_error,
        });
    }
    let end = Instant::now()
        .checked_add(admission)
        .ok_or_else(|| Failure::new("lifecycle_limit"))?;
    let listener = TcpListener::bind(listen).map_err(|_| Failure::new("listen_bind"))?;
    listener
        .set_nonblocking(true)
        .map_err(|_| Failure::new("listen_io"))?;

    let reason = loop {
        if let Some(reason) = stop_reason(signals.requested(), Instant::now(), end, &sessions) {
            break reason;
        }
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(
                    end.saturating_duration_since(Instant::now())
                        .min(Duration::from_millis(1)),
                );
                continue;
            }
            Err(_) => return Err(Failure::new("listen_io")),
        };
        // A successful TCP handshake or this transient owned stream is not
        // an admitted F2 session. Reject a stop/time race before request I/O.
        if signals.requested() || Instant::now() >= end {
            let _ = stream.shutdown(Shutdown::Both);
            drop(stream);
            continue;
        }

        // No other accept can run while this consuming call is active. Its
        // original absolute timeout is never refreshed by a signal or by the
        // admission deadline. At most one admitted connection drains afterward.
        // A1 already established ready.chain().identity() == common.identity;
        // a WrongNetwork result here therefore concerns the untrusted request.
        match sessions.serve_connection(stream, common.identity, ready.chain()) {
            Ok(_) => completed += 1,
            Err(error) => {
                if matches!(
                    &error,
                    SessionErrorV2::InvalidLimits(_)
                        | SessionErrorV2::AttemptsExhausted
                        | SessionErrorV2::AllocationFailed
                        | SessionErrorV2::History(_)
                ) {
                    return Err(error.into());
                }
                rejected += 1;
                last_session_error = Failure::from(error).class;
                // Peer rejection never creates a replacement owner or refunds
                // bytes. Exhaustion and any concurrent stop/time are resolved
                // by the same priority at the head of the loop.
            }
        }
    };

    let report = Report {
        checkpoint,
        reason,
        stats: sessions.stats(),
        completed,
        rejected,
        last_session_error,
    };
    // Keep resource-release ordering independent of report/output construction.
    // A1 itself drops store ownership before control ownership.
    drop(listener);
    drop(ready);
    drop(signals);
    Ok(report)
}

fn stop_reason(
    shutdown: bool,
    now: Instant,
    end: Instant,
    sessions: &PeerSessionsV2,
) -> Option<&'static str> {
    let stats = sessions.stats();
    let limits = sessions.limits();
    if shutdown {
        Some("shutdown")
    } else if now >= end {
        Some("admission-timeout")
    } else if stats.wire_bytes >= limits.max_wire_bytes() {
        Some("wire-limit")
    } else if stats.attempts >= limits.max_attempts() {
        Some("attempt-limit")
    } else {
        None
    }
}
