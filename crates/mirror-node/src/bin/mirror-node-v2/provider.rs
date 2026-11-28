//! One fixed-history provider with one admission loop and one F2 budget owner.
//! No output is performed while sockets or directory ownership remain live.

use std::{
    io,
    net::{Shutdown, SocketAddr, TcpListener},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::ManifestIdentityV2;
use mirror_network::v2::session::{
    PeerSessionsV2, SessionErrorV2, SessionLimitsV2, SessionStatsV2,
};
use mirror_node::v2::NodeControlV2;
use mirror_storage::v2::ReplacementCheckpointV2;

use super::{args::Common, errors::Failure, signals::Signals};

pub(super) struct Report {
    pub checkpoint: ReplacementCheckpointV2,
    pub reason: &'static str,
    // Cumulative owner counters may include a preceding rejoin receive;
    // completed/rejected below count only this serving phase.
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
    let mut sessions = PeerSessionsV2::new(limits);
    serve_ready(
        ready,
        common.identity,
        listen,
        admission,
        &mut sessions,
        &signals,
    )
}

/// Serve the already selected history without reopening ownership, reinstalling
/// signals or replenishing the invocation's cumulative transport allowance.
pub(super) fn serve_ready(
    ready: NodeControlV2,
    identity: ManifestIdentityV2,
    listen: SocketAddr,
    admission: Duration,
    sessions: &mut PeerSessionsV2,
    signals: &Signals,
) -> Result<Report, Failure> {
    let (ready, report) = serve_retained(ready, identity, listen, admission, sessions, signals)?;
    drop(ready);
    Ok(report)
}

/// A normal round returns the same owner after closing its listener. The run
/// coordinator can continue without releasing/reopening either root; fatal
/// returns still consume the owner. Earlier commands use the dropping wrapper.
pub(super) fn serve_retained(
    ready: NodeControlV2,
    identity: ManifestIdentityV2,
    listen: SocketAddr,
    admission: Duration,
    sessions: &mut PeerSessionsV2,
    signals: &Signals,
) -> Result<(NodeControlV2, Report), Failure> {
    let checkpoint = ready.checkpoint();
    let mut completed = 0;
    let mut rejected = 0;
    let mut last_session_error = "none";

    // Opening/rejoin is separate from the finite service admission window.
    // Resolve shutdown and any already spent allowance before starting that
    // clock or binding: a last-attempt receive may still have adopted safely.
    let before_window = if signals.requested() {
        Some("shutdown")
    } else {
        budget_stop_reason(sessions)
    };
    if let Some(reason) = before_window {
        return Ok((
            ready,
            Report {
                checkpoint,
                reason,
                stats: sessions.stats(),
                completed,
                rejected,
                last_session_error,
            },
        ));
    }
    let end = Instant::now()
        .checked_add(admission)
        .ok_or_else(|| Failure::new("lifecycle_limit"))?;
    let listener = TcpListener::bind(listen).map_err(|_| Failure::new("listen_bind"))?;
    listener
        .set_nonblocking(true)
        .map_err(|_| Failure::new("listen_io"))?;

    let reason = loop {
        if let Some(reason) = stop_reason(signals.requested(), Instant::now(), end, sessions) {
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
        // A1 already established ready.chain().identity() == identity;
        // a WrongNetwork result here therefore concerns the untrusted request.
        match sessions.serve_connection(stream, identity, ready.chain()) {
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
    // The listener is gone before any next acquisition or inter-round wait.
    // Earlier command wrappers release this retained owner before output.
    drop(listener);
    Ok((ready, report))
}

fn stop_reason(
    shutdown: bool,
    now: Instant,
    end: Instant,
    sessions: &PeerSessionsV2,
) -> Option<&'static str> {
    if shutdown {
        Some("shutdown")
    } else if now >= end {
        Some("admission-timeout")
    } else {
        budget_stop_reason(sessions)
    }
}

fn budget_stop_reason(sessions: &PeerSessionsV2) -> Option<&'static str> {
    let stats = sessions.stats();
    let limits = sessions.limits();
    if stats.wire_bytes >= limits.max_wire_bytes() {
        Some("wire-limit")
    } else if stats.attempts >= limits.max_attempts() {
        Some("attempt-limit")
    } else {
        None
    }
}
