//! Contract assertions for real bounded multi-round commands.
//! Existing child/FD observations and synthetic histories are reused unchanged.
#![allow(dead_code, unused_imports)]

#[path = "../support_rejoin_v2/mod.rs"]
mod existing;

pub use existing::{
    IntentWatch, Peer, PeerMode, ProviderProcess, RejoinProcess, Service, accept_request,
    adoption_history, assert_busy, assert_no_connection, assert_port_closed, command, common,
    connect, failure, no_response_yet, pending, pins, process_guard, provider, raw_request,
    read_response, ready, ready_with, receive_bytes, receive_complete, recover, remove, reserve,
    selected, set, state, success, sync, transcript, verify_stopped_intent,
};

use std::{collections::BTreeMap, ffi::OsString, net::SocketAddr, process::Output};

use mirror_chain::v2::ChainV2;
use mirror_network::v2::session::MAX_SESSION_WIRE_BYTES_V2;
use mirror_storage::v2::ReplacementCheckpointV2;

use state::{Scratch, checkpoint, history::main_chain};

pub const OPERATION: &str = "run";
pub const BAD_HEADER_BYTES: u64 = 72 + 12;

pub fn arguments(scratch: &Scratch, peers: &[SocketAddr], listen: SocketAddr) -> Vec<OsString> {
    let mut result = common(OPERATION, scratch);
    result.extend([
        "--peers".into(),
        peers
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
            .into(),
        "--listen".into(),
        listen.to_string().into(),
        "--max-attempts".into(),
        "8".into(),
        "--max-io-bytes".into(),
        MAX_SESSION_WIRE_BYTES_V2.to_string().into(),
        "--timeout-ms".into(),
        "5000".into(),
        "--admission-ms".into(),
        "300".into(),
        "--max-rounds".into(),
        "2".into(),
        "--round-wait-ms".into(),
        "25".into(),
    ]);
    result
}

#[derive(Clone, Copy)]
pub struct Acquisition<'a> {
    pub results: &'a [&'a str],
    pub bytes: u64,
    pub stop: &'a str,
}

impl Acquisition<'_> {
    pub fn attempts(&self) -> u64 {
        self.results
            .iter()
            .filter(|value| **value != "not-attempted")
            .count() as u64
    }

    pub fn validated(&self) -> u64 {
        self.results
            .iter()
            .position(|value| *value == "validated")
            .map_or(0, |i| i as u64 + 1)
    }

    pub fn failed(&self) -> u64 {
        self.results
            .iter()
            .filter(|value| !matches!(**value, "not-attempted" | "validated"))
            .count() as u64
    }

    pub fn last_error(&self) -> &str {
        self.results
            .iter()
            .rev()
            .copied()
            .find(|value| !matches!(*value, "not-attempted" | "validated"))
            .unwrap_or("none")
    }

    pub fn good(chain: &ChainV2) -> Acquisition<'static> {
        Acquisition {
            results: &["validated"],
            bytes: receive_bytes(chain),
            stop: "validated",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Round<'a> {
    pub selected: ReplacementCheckpointV2,
    pub choice: &'a str,
    pub acquisition: Acquisition<'a>,
    pub service_reason: &'a str,
    pub service: Service<'a>,
}

impl Round<'static> {
    pub fn adopted(chain: &ChainV2, generation: u64, service_reason: &'static str) -> Self {
        Self {
            selected: checkpoint(chain, generation),
            choice: "prefer-candidate",
            acquisition: Acquisition::good(chain),
            service_reason,
            service: Service::NONE,
        }
    }

    pub fn kept(chain: &ChainV2, generation: u64, service_reason: &'static str) -> Self {
        Self {
            choice: "keep-current",
            ..Self::adopted(chain, generation, service_reason)
        }
    }
}

pub struct Normal<'a> {
    pub initial: ReplacementCheckpointV2,
    pub limit: u8,
    pub peer_count: usize,
    pub rounds: &'a [Round<'a>],
    pub incomplete: Option<Acquisition<'a>>,
    pub reason: &'a str,
}

impl<'a> Normal<'a> {
    pub fn new(rounds: &'a [Round<'a>], reason: &'a str) -> Self {
        Self {
            initial: checkpoint(&main_chain(3), 0),
            limit: 2,
            peer_count: 1,
            rounds,
            incomplete: None,
            reason,
        }
    }
}

#[derive(Default)]
struct Totals<'a> {
    receive_attempts: u64,
    receive_bytes: u64,
    failed_receives: u64,
    serve_attempts: u64,
    serve_bytes: u64,
    completed: u64,
    rejected: u64,
    last_receive_error: Option<&'a str>,
    last_session_error: Option<&'a str>,
}

fn totals<'a>(rounds: &'a [Round<'a>], incomplete: Option<Acquisition<'a>>) -> Totals<'a> {
    let mut result = Totals::default();
    for round in rounds {
        let service = round.service;
        result.serve_attempts += u64::from(service.completed) + u64::from(service.rejected);
        result.serve_bytes += service.bytes;
        result.completed += u64::from(service.completed);
        result.rejected += u64::from(service.rejected);
        if service.last_error != "none" {
            result.last_session_error = Some(service.last_error);
        }
    }
    for acquisition in rounds
        .iter()
        .map(|round| round.acquisition)
        .chain(incomplete)
    {
        result.receive_attempts += acquisition.attempts();
        result.receive_bytes += acquisition.bytes;
        result.failed_receives += acquisition.failed();
        if let Some(error) = acquisition
            .results
            .iter()
            .rev()
            .copied()
            .find(|value| !matches!(*value, "not-attempted" | "validated"))
        {
            result.last_receive_error = Some(error);
        }
    }
    result
}

fn number(values: &BTreeMap<String, String>, key: &str, expected: u64) {
    assert_eq!(values[key], expected.to_string(), "{key}");
    assert_eq!(
        values[key].parse::<u64>().unwrap().to_string(),
        values[key],
        "{key}"
    );
}

pub fn stopped(output: &Output, expected: Normal<'_>) -> BTreeMap<String, String> {
    assert!(output.stdout.len() <= 4096);
    let values = success(output, 0, OPERATION);
    assert_eq!(
        values.len(),
        30 + 16 * expected.rounds.len() + 7 * usize::from(expected.incomplete.is_some())
    );
    assert_eq!(values["status"], "stopped");
    assert_eq!(values["store_selection"], "verified");
    assert_eq!(values["reason"], expected.reason);
    pins(&values, "initial", expected.initial);
    pins(
        &values,
        "selected",
        expected
            .rounds
            .last()
            .map_or(expected.initial, |round| round.selected),
    );
    number(&values, "round_limit", expected.limit.into());
    number(&values, "peer_count", expected.peer_count as u64);
    number(
        &values,
        "rounds_started",
        (expected.rounds.len() + usize::from(expected.incomplete.is_some())) as u64,
    );
    number(&values, "rounds_completed", expected.rounds.len() as u64);
    let total = totals(expected.rounds, expected.incomplete);
    for (key, value) in [
        ("attempts", total.receive_attempts + total.serve_attempts),
        ("receive_attempts", total.receive_attempts),
        ("serve_attempts", total.serve_attempts),
        ("completed_sessions", total.completed),
        ("rejected_sessions", total.rejected),
        ("wire_bytes", total.receive_bytes + total.serve_bytes),
        ("receive_wire_bytes", total.receive_bytes),
        ("serve_wire_bytes", total.serve_bytes),
        ("failed_receives", total.failed_receives),
    ] {
        number(&values, key, value);
    }
    assert_eq!(
        values["last_receive_error"],
        total.last_receive_error.unwrap_or("none")
    );
    assert_eq!(
        values["last_session_error"],
        total.last_session_error.unwrap_or("none")
    );
    for (index, round) in expected.rounds.iter().enumerate() {
        let prefix = format!("round_{}", index + 1);
        pins(&values, &prefix, round.selected);
        assert_eq!(values[&format!("{prefix}_choice")], round.choice);
        assert_eq!(
            values[&format!("{prefix}_service_reason")],
            round.service_reason
        );
        assert_eq!(
            values[&format!("{prefix}_last_receive_error")],
            round.acquisition.last_error()
        );
        assert_eq!(
            values[&format!("{prefix}_last_session_error")],
            round.service.last_error
        );
        assert_eq!(
            values[&format!("{prefix}_peer_results")],
            round.acquisition.results.join(",")
        );
        assert_eq!(round.acquisition.results.len(), expected.peer_count);
        for (key, value) in [
            ("validated_peer", round.acquisition.validated()),
            ("receive_attempts", round.acquisition.attempts()),
            ("receive_wire_bytes", round.acquisition.bytes),
            (
                "serve_attempts",
                u64::from(round.service.completed) + u64::from(round.service.rejected),
            ),
            ("serve_wire_bytes", round.service.bytes),
            ("completed_sessions", u64::from(round.service.completed)),
            ("rejected_sessions", u64::from(round.service.rejected)),
        ] {
            number(&values, &format!("{prefix}_{key}"), value);
        }
    }
    if let Some(trace) = expected.incomplete {
        number(
            &values,
            "incomplete_round",
            expected.rounds.len() as u64 + 1,
        );
        number(&values, "incomplete_receive_attempts", trace.attempts());
        number(&values, "incomplete_receive_wire_bytes", trace.bytes);
        number(&values, "incomplete_validated_peer", trace.validated());
        assert_eq!(values["incomplete_last_receive_error"], trace.last_error());
        assert_eq!(values["incomplete_acquisition_stop"], trace.stop);
        assert_eq!(values["incomplete_peer_results"], trace.results.join(","));
    }
    values
}

pub struct Error<'a> {
    pub limit: u8,
    pub peer_count: usize,
    pub rounds: &'a [Round<'a>],
    pub current: Option<Acquisition<'a>>,
    pub phase: &'a str,
    pub admitted: bool,
}

impl<'a> Error<'a> {
    pub fn acquisition(rounds: &'a [Round<'a>], current: Acquisition<'a>) -> Self {
        Self {
            limit: 2,
            peer_count: current.results.len(),
            rounds,
            current: Some(current),
            phase: "acquisition",
            admitted: !rounds.is_empty(),
        }
    }
}

pub fn failed(output: &Output, class: &str, expected: Error<'_>) -> BTreeMap<String, String> {
    assert!(output.stderr.len() <= 4096);
    let values = failure(output, 1);
    assert_eq!(
        values.len(),
        4 + usize::from(expected.admitted) + 15 + 8 * usize::from(expected.current.is_some())
    );
    assert_eq!(values["operation"], OPERATION);
    assert_eq!(values["error"], class);
    assert!(
        !values
            .keys()
            .any(|key| key.starts_with("selected_") || key.starts_with("initial_"))
    );
    assert!(!values.contains_key("store_selection"));
    assert!(!values.contains_key("completed_sessions"));
    assert!(!values.contains_key("rejected_sessions"));
    assert_eq!(values["phase"], expected.phase);
    assert_eq!(values["adoption_admitted"], expected.admitted.to_string());
    if expected.admitted {
        assert_eq!(values["operation_may_have_completed"], "true");
        assert_eq!(values["next"], "inspect-status");
    } else {
        assert!(!values.contains_key("operation_may_have_completed"));
    }
    number(&values, "round_limit", expected.limit.into());
    number(&values, "peer_count", expected.peer_count as u64);
    number(
        &values,
        "rounds_started",
        (expected.rounds.len() + usize::from(expected.current.is_some())) as u64,
    );
    number(&values, "rounds_completed", expected.rounds.len() as u64);
    let total = totals(expected.rounds, expected.current);
    for (key, value) in [
        ("attempts", total.receive_attempts + total.serve_attempts),
        ("wire_bytes", total.receive_bytes + total.serve_bytes),
        ("receive_attempts", total.receive_attempts),
        ("receive_wire_bytes", total.receive_bytes),
        ("serve_attempts", total.serve_attempts),
        ("serve_wire_bytes", total.serve_bytes),
        ("failed_receives", total.failed_receives),
    ] {
        number(&values, key, value);
    }
    assert_eq!(
        values["last_receive_error"],
        total.last_receive_error.unwrap_or("none")
    );
    assert_eq!(
        values["last_completed_service_error"],
        total.last_session_error.unwrap_or("none")
    );
    if let Some(trace) = expected.current {
        number(&values, "current_round", expected.rounds.len() as u64 + 1);
        number(&values, "current_validated_peer", trace.validated());
        number(&values, "current_failed_receives", trace.failed());
        number(&values, "current_receive_attempts", trace.attempts());
        number(&values, "current_receive_wire_bytes", trace.bytes);
        assert_eq!(values["current_acquisition_stop"], trace.stop);
        assert_eq!(values["current_last_receive_error"], trace.last_error());
        assert_eq!(values["current_peer_results"], trace.results.join(","));
    } else {
        assert!(!values.contains_key("current_round"));
    }
    values
}

pub fn output_lost(output: &Output) {
    let values = failure(output, 1);
    assert_eq!(values.len(), 5);
    assert_eq!(values["operation"], OPERATION);
    assert_eq!(values["error"], "output_io");
    assert_eq!(values["operation_may_have_completed"], "true");
    assert_eq!(values["next"], "inspect-status");
    assert!(!values.keys().any(|key| key.starts_with("selected_")));
    assert!(output.stderr.len() <= 4096);
}

pub fn sanitized(output: &Output, scratch: &Scratch, endpoints: &[SocketAddr]) {
    for bytes in [&output.stdout, &output.stderr] {
        let text = std::str::from_utf8(bytes).unwrap();
        assert!(!text.contains(scratch.0.to_str().unwrap()));
        for endpoint in endpoints {
            assert!(!text.contains(&endpoint.to_string()));
        }
    }
}
