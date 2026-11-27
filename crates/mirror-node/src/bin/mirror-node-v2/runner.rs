//! Explicit finite rounds under one retained controller and transport allowance.
//! No output, implicit recovery or replacement session owner occurs in this path.

use std::{
    net::SocketAddr,
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::ForkChoiceV2;
use mirror_network::v2::session::{PeerSessionsV2, SessionLimitsV2, SessionStatsV2};
use mirror_node::v2::NodeControlV2;
use mirror_storage::v2::ReplacementCheckpointV2;

use super::{
    args::{Common, MAX_FAILOVER_PEERS, MAX_RUN_ROUNDS},
    errors::Failure,
    failover::{self, Acquisition},
    rejoin,
    signals::Signals,
};

#[derive(Clone, Copy, Debug)]
pub(super) struct RunTrace {
    pub round_limit: u8,
    pub rounds_started: u8,
    pub rounds_completed: u8,
    pub peer_count: u8,
    pub phase: &'static str,
    pub stats: SessionStatsV2,
    pub receive: SessionStatsV2,
    pub failed_receives: u8,
    pub last_receive_error: &'static str,
    pub last_completed_service_error: &'static str,
    pub acquisition: Option<Acquisition>,
    pub adoption_admitted: bool,
}

impl RunTrace {
    fn new(round_limit: u8, peer_count: u8) -> Self {
        Self {
            round_limit,
            rounds_started: 0,
            rounds_completed: 0,
            peer_count,
            phase: "startup",
            stats: SessionStatsV2::default(),
            receive: SessionStatsV2::default(),
            failed_receives: 0,
            last_receive_error: "none",
            last_completed_service_error: "none",
            acquisition: None,
            adoption_admitted: false,
        }
    }

    fn record_acquisition(
        &mut self,
        before: SessionStatsV2,
        acquisition: Acquisition,
    ) -> Result<(), Failure> {
        if acquisition.peer_count != self.peer_count
            || !acquisition.consistent()
            || usage_since(self.stats, before)? != acquisition.stats
            || self.rounds_completed.checked_add(1) != Some(self.rounds_started)
        {
            return Err(Failure::new("lifecycle_limit"));
        }
        let receive = add_usage(self.receive, acquisition.stats)?;
        let failed_receives = self
            .failed_receives
            .checked_add(acquisition.failed_receives)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?;
        // Prior service usage cannot be counted as a new round's receipt.
        usage_since(self.stats, receive)?;
        self.receive = receive;
        self.failed_receives = failed_receives;
        if acquisition.last_receive_error != "none" {
            self.last_receive_error = acquisition.last_receive_error;
        }
        Ok(())
    }

    fn complete_round(
        &mut self,
        round: &CompletedRound,
        completed: u8,
        rejected: u8,
    ) -> Result<(u8, u8), Failure> {
        let rounds_completed = self
            .rounds_completed
            .checked_add(1)
            .filter(|count| *count == self.rounds_started && *count <= self.round_limit)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?;
        let completed = completed
            .checked_add(round.completed)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?;
        let rejected = rejected
            .checked_add(round.rejected)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?;
        let service = usage_since(self.stats, self.receive)?;
        if !self.adoption_admitted || completed.checked_add(rejected) != Some(service.attempts) {
            return Err(Failure::new("lifecycle_limit"));
        }
        self.rounds_completed = rounds_completed;
        if round.last_session_error != "none" {
            self.last_completed_service_error = round.last_session_error;
        }
        self.acquisition = None;
        Ok((completed, rejected))
    }

    fn failure(&mut self, failure: Failure) -> Failure {
        // A failed retained helper may have admitted its first adoption without
        // returning a normal round. Absorb that annotation before attaching trace.
        self.adoption_admitted |= failure.may_have_completed == Some(true);
        let failure = if self.adoption_admitted {
            failure.after_adoption()
        } else {
            failure
        };
        failure.with_run(*self)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CompletedRound {
    pub previous: ReplacementCheckpointV2,
    pub selected: ReplacementCheckpointV2,
    pub choice: ForkChoiceV2,
    pub acquisition: Acquisition,
    pub serve_stats: SessionStatsV2,
    pub completed: u8,
    pub rejected: u8,
    pub service_reason: &'static str,
    pub last_session_error: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Report {
    pub initial: ReplacementCheckpointV2,
    pub selected: ReplacementCheckpointV2,
    pub reason: &'static str,
    pub trace: RunTrace,
    pub completed: u8,
    pub rejected: u8,
    pub rounds: [Option<CompletedRound>; MAX_RUN_ROUNDS],
}

pub(super) fn run(
    common: &Common,
    peers: &[SocketAddr],
    listen: SocketAddr,
    limits: SessionLimitsV2,
    admission: Duration,
    max_rounds: u8,
    round_wait: Duration,
) -> Result<Report, Failure> {
    let peer_count = u8::try_from(peers.len()).unwrap_or(0);
    let mut trace = RunTrace::new(max_rounds, peer_count);
    let result = if max_rounds == 0
        || usize::from(max_rounds) > MAX_RUN_ROUNDS
        || peer_count == 0
        || usize::from(peer_count) > MAX_FAILOVER_PEERS
        || round_wait > Duration::from_millis(60_000)
        || admission.is_zero()
        || admission > Duration::from_millis(60_000)
    {
        Err(Failure::new("lifecycle_limit"))
    } else {
        run_owned(
            common, peers, listen, limits, admission, round_wait, &mut trace,
        )
    };
    // All sockets, roots and the signal guard have left the owned function before
    // either normal reporting or optional error context reaches the formatter.
    result.map_err(|failure| trace.failure(failure))
}

fn run_owned(
    common: &Common,
    peers: &[SocketAddr],
    listen: SocketAddr,
    limits: SessionLimitsV2,
    admission: Duration,
    round_wait: Duration,
    trace: &mut RunTrace,
) -> Result<Report, Failure> {
    let signals = Signals::install()?;
    let mut ready = NodeControlV2::open_writer(&common.control, &common.store, common.identity)?;
    let initial = ready.checkpoint();
    let mut sessions = PeerSessionsV2::new(limits);
    let mut rounds = [None; MAX_RUN_ROUNDS];
    let mut completed = 0;
    let mut rejected = 0;

    loop {
        trace.stats = sessions.stats();
        if let Some(reason) = stop_reason(signals.requested(), trace, limits) {
            return finish(ready, initial, reason, trace, completed, rejected, rounds);
        }
        let previous = ready.checkpoint();
        trace.phase = "acquisition";
        trace.rounds_started = trace
            .rounds_started
            .checked_add(1)
            .filter(|count| *count <= trace.round_limit)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?;
        let mut acquisition = Acquisition::new(trace.peer_count);
        trace.acquisition = Some(acquisition);
        let before = sessions.stats();
        let received = failover::acquire(
            peers,
            common.identity,
            &mut sessions,
            &signals,
            &mut acquisition,
        );
        trace.stats = sessions.stats();
        trace.acquisition = Some(acquisition);
        trace.record_acquisition(before, acquisition)?;
        let Some(received) = received? else {
            if acquisition.stats != SessionStatsV2::default()
                || acquisition.validated_peer != 0
                || acquisition.stop != "shutdown"
            {
                return Err(Failure::new("lifecycle_limit"));
            }
            return finish(
                ready, initial, "shutdown", trace, completed, rejected, rounds,
            );
        };

        trace.phase = "adoption-service";
        let before_service = sessions.stats();
        let outcome = rejoin::finish_received_retained(
            ready,
            received,
            common.identity,
            listen,
            admission,
            &mut sessions,
            &signals,
        );
        trace.stats = sessions.stats();
        if let Err(failure) = &outcome {
            trace.adoption_admitted |= failure.may_have_completed == Some(true);
        }
        let (retained, lifecycle) = outcome?;
        ready = retained;
        // Preserve completion before any fallible accounting/report checks.
        trace.adoption_admitted |= lifecycle.choice.is_some();

        if lifecycle.choice.is_none() {
            if lifecycle.previous != previous
                || ready.checkpoint() != previous
                || lifecycle.receive != before_service
                || lifecycle.service.checkpoint != previous
                || lifecycle.service.stats != trace.stats
                || trace.stats != before_service
                || lifecycle.serve_stats != SessionStatsV2::default()
                || lifecycle.service.completed != 0
                || lifecycle.service.rejected != 0
                || lifecycle.service.reason != "shutdown"
                || lifecycle.service.last_session_error != "none"
            {
                return Err(Failure::new("lifecycle_limit"));
            }
            return finish(
                ready, initial, "shutdown", trace, completed, rejected, rounds,
            );
        }

        let round = round_record(
            previous,
            ready.checkpoint(),
            before_service,
            trace.stats,
            acquisition,
            &lifecycle,
        )?;
        let index = usize::from(trace.rounds_completed);
        if index >= rounds.len() || rounds[index].is_some() {
            return Err(Failure::new("lifecycle_limit"));
        }
        (completed, rejected) = trace.complete_round(&round, completed, rejected)?;
        rounds[index] = Some(round);

        if let Some(reason) = stop_reason(signals.requested(), trace, limits) {
            return finish(ready, initial, reason, trace, completed, rejected, rounds);
        }
        if round.service_reason != "admission-timeout" {
            return Err(Failure::new("lifecycle_limit"));
        }
        trace.phase = "wait";
        wait_between_rounds(round_wait, &signals)?;
    }
}

fn round_record(
    previous: ReplacementCheckpointV2,
    selected: ReplacementCheckpointV2,
    before_service: SessionStatsV2,
    after_service: SessionStatsV2,
    acquisition: Acquisition,
    lifecycle: &rejoin::Report,
) -> Result<CompletedRound, Failure> {
    let choice = lifecycle
        .choice
        .ok_or_else(|| Failure::new("lifecycle_limit"))?;
    if lifecycle.previous != previous
        || lifecycle.receive != before_service
        || lifecycle.service.checkpoint != selected
        || lifecycle.service.stats != after_service
        || usage_since(after_service, before_service)? != lifecycle.serve_stats
        || lifecycle
            .service
            .completed
            .checked_add(lifecycle.service.rejected)
            != Some(lifecycle.serve_stats.attempts)
        || acquisition.validated_peer == 0
        || acquisition.stop != "validated"
        || !acquisition.consistent()
        || !valid_choice(previous, selected, choice)
    {
        return Err(Failure::new("lifecycle_limit"));
    }
    Ok(CompletedRound {
        previous,
        selected,
        choice,
        acquisition,
        serve_stats: lifecycle.serve_stats,
        completed: lifecycle.service.completed,
        rejected: lifecycle.service.rejected,
        service_reason: lifecycle.service.reason,
        last_session_error: lifecycle.service.last_session_error,
    })
}

fn valid_choice(
    previous: ReplacementCheckpointV2,
    selected: ReplacementCheckpointV2,
    choice: ForkChoiceV2,
) -> bool {
    match choice {
        ForkChoiceV2::KeepCurrent => previous == selected,
        ForkChoiceV2::PreferCandidate => {
            previous.generation.checked_add(1) == Some(selected.generation)
                && selected.chain.height > previous.chain.height
                && selected.chain.config_hash == previous.chain.config_hash
                && selected.chain.genesis_hash == previous.chain.genesis_hash
        }
    }
}

fn finish(
    ready: NodeControlV2,
    initial: ReplacementCheckpointV2,
    reason: &'static str,
    trace: &mut RunTrace,
    completed: u8,
    rejected: u8,
    rounds: [Option<CompletedRound>; MAX_RUN_ROUNDS],
) -> Result<Report, Failure> {
    trace.phase = "report";
    let selected = ready.checkpoint();
    let mut previous = initial;
    let mut receive = SessionStatsV2::default();
    let mut service = SessionStatsV2::default();
    let mut counted_completed = 0u8;
    let mut counted_rejected = 0u8;
    let mut counted_failures = 0u8;
    for (index, entry) in rounds.iter().enumerate() {
        if index < usize::from(trace.rounds_completed) {
            let round = entry
                .as_ref()
                .ok_or_else(|| Failure::new("lifecycle_limit"))?;
            if round.previous != previous || !valid_choice(previous, round.selected, round.choice) {
                return Err(Failure::new("lifecycle_limit"));
            }
            previous = round.selected;
            receive = add_usage(receive, round.acquisition.stats)?;
            service = add_usage(service, round.serve_stats)?;
            counted_completed = counted_completed
                .checked_add(round.completed)
                .ok_or_else(|| Failure::new("lifecycle_limit"))?;
            counted_rejected = counted_rejected
                .checked_add(round.rejected)
                .ok_or_else(|| Failure::new("lifecycle_limit"))?;
            counted_failures = counted_failures
                .checked_add(round.acquisition.failed_receives)
                .ok_or_else(|| Failure::new("lifecycle_limit"))?;
        } else if entry.is_some() {
            return Err(Failure::new("lifecycle_limit"));
        }
    }
    let mut incomplete_validated = 0;
    if let Some(acquisition) = trace.acquisition {
        if trace.rounds_completed.checked_add(1) != Some(trace.rounds_started)
            || !acquisition.consistent()
            || acquisition.peer_count != trace.peer_count
            || !(acquisition.stop == "validated"
                || (acquisition.stop == "shutdown" && acquisition.stats.attempts == 0))
        {
            return Err(Failure::new("lifecycle_limit"));
        }
        incomplete_validated = u8::from(acquisition.validated_peer != 0);
        receive = add_usage(receive, acquisition.stats)?;
        counted_failures = counted_failures
            .checked_add(acquisition.failed_receives)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?;
    } else if trace.rounds_started != trace.rounds_completed {
        return Err(Failure::new("lifecycle_limit"));
    }
    if previous != selected
        || receive != trace.receive
        || add_usage(receive, service)? != trace.stats
        || counted_completed != completed
        || counted_rejected != rejected
        || completed.checked_add(rejected) != Some(service.attempts)
        || counted_failures != trace.failed_receives
        || counted_failures
            .checked_add(trace.rounds_completed)
            .and_then(|count| count.checked_add(incomplete_validated))
            != Some(receive.attempts)
        || trace.adoption_admitted != (trace.rounds_completed != 0)
    {
        return Err(Failure::new("lifecycle_limit"));
    }
    let report = Report {
        initial,
        selected,
        reason,
        trace: *trace,
        completed,
        rejected,
        rounds,
    };
    drop(ready);
    Ok(report)
}

fn stop_reason(shutdown: bool, trace: &RunTrace, limits: SessionLimitsV2) -> Option<&'static str> {
    if shutdown {
        Some("shutdown")
    } else if let Some(reason) = failover::exhausted(trace.stats, limits) {
        Some(reason)
    } else if trace.rounds_completed >= trace.round_limit {
        Some("round-limit")
    } else {
        None
    }
}

fn wait_between_rounds(duration: Duration, signals: &Signals) -> Result<(), Failure> {
    let end = Instant::now()
        .checked_add(duration)
        .ok_or_else(|| Failure::new("lifecycle_limit"))?;
    while !signals.requested() {
        let remaining = end.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        thread::sleep(remaining.min(Duration::from_millis(1)));
    }
    Ok(())
}

fn usage_since(after: SessionStatsV2, before: SessionStatsV2) -> Result<SessionStatsV2, Failure> {
    Ok(SessionStatsV2 {
        attempts: after
            .attempts
            .checked_sub(before.attempts)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?,
        wire_bytes: after
            .wire_bytes
            .checked_sub(before.wire_bytes)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?,
    })
}

fn add_usage(a: SessionStatsV2, b: SessionStatsV2) -> Result<SessionStatsV2, Failure> {
    Ok(SessionStatsV2 {
        attempts: a
            .attempts
            .checked_add(b.attempts)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?,
        wire_bytes: a
            .wire_bytes
            .checked_add(b.wire_bytes)
            .ok_or_else(|| Failure::new("lifecycle_limit"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirror_chain::v2::ChainReplayExpectationV2;
    use mirror_crypto::Hash256;

    fn stats(attempts: u8, wire_bytes: u64) -> SessionStatsV2 {
        SessionStatsV2 {
            attempts,
            wire_bytes,
        }
    }

    fn acquired(failed: bool) -> Acquisition {
        let mut acquisition = Acquisition::new(2);
        acquisition.stop = "validated";
        if failed {
            acquisition.stats = stats(2, 184);
            acquisition.failed_receives = 1;
            acquisition.last_receive_error = "peer_unreachable";
            acquisition.validated_peer = 2;
            acquisition.results = [
                "peer_unreachable",
                "validated",
                "not-attempted",
                "not-attempted",
            ];
        } else {
            acquisition.stats = stats(1, 100);
            acquisition.validated_peer = 1;
            acquisition.results = [
                "validated",
                "not-attempted",
                "not-attempted",
                "not-attempted",
            ];
        }
        acquisition
    }

    fn checkpoint(generation: u64, height: u64, tag: u8) -> ReplacementCheckpointV2 {
        ReplacementCheckpointV2 {
            generation,
            chain: ChainReplayExpectationV2 {
                config_hash: Hash256::from_bytes([1; 32]),
                genesis_hash: Hash256::from_bytes([2; 32]),
                height,
                tip_hash: Hash256::from_bytes([tag; 32]),
                ledger_root: Hash256::from_bytes([tag; 32]),
            },
        }
    }

    #[test]
    fn run_global_stops_keep_shutdown_wire_attempt_and_round_priority() {
        let limits = SessionLimitsV2::new(4, 1000, Duration::from_millis(50)).unwrap();
        let mut trace = RunTrace::new(2, 2);
        trace.rounds_completed = 2;
        trace.stats = stats(4, 1000);
        assert_eq!(stop_reason(true, &trace, limits), Some("shutdown"));
        assert_eq!(stop_reason(false, &trace, limits), Some("wire-limit"));
        trace.stats.wire_bytes = 999;
        assert_eq!(stop_reason(false, &trace, limits), Some("attempt-limit"));
        trace.stats.attempts = 3;
        assert_eq!(stop_reason(false, &trace, limits), Some("round-limit"));
        trace.rounds_completed = 1;
        assert_eq!(stop_reason(false, &trace, limits), None);
    }

    #[test]
    fn run_round_receive_deltas_exclude_prior_service_and_keep_earlier_errors() {
        let mut trace = RunTrace::new(4, 2);
        trace.rounds_started = 1;
        let first = acquired(true);
        trace.stats = first.stats;
        assert!(trace.record_acquisition(stats(0, 0), first).is_ok());
        trace.rounds_completed = 1;
        trace.rounds_started = 2;
        let before = stats(3, 384); // First acquisition plus a serving attempt.
        let second = acquired(false);
        trace.stats = stats(4, 484);
        assert!(trace.record_acquisition(before, second).is_ok());
        assert_eq!(trace.receive, stats(3, 284));
        assert_eq!(trace.failed_receives, 1);
        assert_eq!(trace.last_receive_error, "peer_unreachable");
        assert_eq!(
            usage_since(trace.stats, trace.receive).ok(),
            Some(stats(1, 200))
        );
    }

    #[test]
    fn run_inconsistent_round_usage_rejects_without_committing_receive_totals() {
        let mut trace = RunTrace::new(4, 2);
        trace.rounds_started = 1;
        trace.stats = stats(3, 284);
        let before = trace;
        assert!(
            trace
                .record_acquisition(stats(0, 0), acquired(true))
                .is_err()
        );
        assert_eq!(trace.receive, before.receive);
        assert_eq!(trace.failed_receives, before.failed_receives);
        assert_eq!(trace.last_receive_error, before.last_receive_error);
    }

    #[test]
    fn run_usage_arithmetic_rejects_underflow_and_overflow() {
        assert!(usage_since(stats(0, 10), stats(1, 10)).is_err());
        assert!(usage_since(stats(1, 9), stats(1, 10)).is_err());
        assert!(add_usage(stats(u8::MAX, 0), stats(1, 0)).is_err());
        assert!(add_usage(stats(0, u64::MAX), stats(0, 1)).is_err());
    }

    #[test]
    fn run_possible_completion_is_sticky_and_absorbs_failed_helper_admission() {
        let mut startup = RunTrace::new(2, 1);
        let early = startup.failure(Failure::new("peer_unreachable"));
        assert_eq!(early.may_have_completed, None);
        assert!(!startup.adoption_admitted);
        let admitted = startup.failure(Failure::new("listen_bind").after_adoption());
        assert!(startup.adoption_admitted);
        assert_eq!(admitted.class, "listen_bind");
        assert_eq!(admitted.may_have_completed, Some(true));
        let later = startup.failure(Failure::new("invalid_peer_request"));
        assert_eq!(later.class, "invalid_peer_request");
        assert_eq!(later.next, "inspect-status");
        assert_eq!(later.may_have_completed, Some(true));
        assert!(later.run.as_ref().unwrap().adoption_admitted);
    }

    #[test]
    fn run_completed_service_accounting_rejects_missing_outcomes_and_keeps_prior_error() {
        let previous = checkpoint(0, 1, 3);
        let acquisition = acquired(false);
        let round = CompletedRound {
            previous,
            selected: previous,
            choice: ForkChoiceV2::KeepCurrent,
            acquisition,
            serve_stats: stats(1, 10),
            completed: 1,
            rejected: 0,
            service_reason: "admission-timeout",
            last_session_error: "none",
        };
        let mut trace = RunTrace::new(2, 2);
        // A prior completed round received once and rejected one service call.
        // The current round has received once and completed one service call.
        trace.rounds_started = 2;
        trace.rounds_completed = 1;
        trace.receive = stats(2, 200);
        trace.stats = stats(4, 220);
        trace.acquisition = Some(acquisition);
        trace.adoption_admitted = true;
        trace.last_completed_service_error = "peer_wrong_network";
        let mut inconsistent = round;
        inconsistent.completed = 0;
        assert!(trace.complete_round(&inconsistent, 0, 1).is_err());
        assert_eq!(trace.rounds_completed, 1);
        assert!(trace.acquisition.is_some());
        assert_eq!(trace.complete_round(&round, 0, 1).ok(), Some((1, 1)));
        assert_eq!(trace.rounds_completed, 2);
        assert!(trace.acquisition.is_none());
        assert_eq!(trace.last_completed_service_error, "peer_wrong_network");
    }

    #[test]
    fn run_checkpoint_transitions_preserve_identity_generation_and_whole_selection() {
        let previous = checkpoint(2, 5, 3);
        let candidate = checkpoint(3, 6, 4);
        assert!(valid_choice(previous, previous, ForkChoiceV2::KeepCurrent));
        assert!(valid_choice(
            previous,
            candidate,
            ForkChoiceV2::PreferCandidate
        ));
        assert!(!valid_choice(
            previous,
            candidate,
            ForkChoiceV2::KeepCurrent
        ));
        let mut wrong = candidate;
        wrong.generation = 4;
        assert!(!valid_choice(
            previous,
            wrong,
            ForkChoiceV2::PreferCandidate
        ));
        wrong = candidate;
        wrong.chain.height = previous.chain.height;
        assert!(!valid_choice(
            previous,
            wrong,
            ForkChoiceV2::PreferCandidate
        ));
        wrong = candidate;
        wrong.chain.genesis_hash = Hash256::from_bytes([9; 32]);
        assert!(!valid_choice(
            previous,
            wrong,
            ForkChoiceV2::PreferCandidate
        ));
    }
}
