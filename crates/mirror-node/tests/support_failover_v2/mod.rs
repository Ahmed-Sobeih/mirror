//! New-command assertions over unchanged B2/operator process observations.
//! Failed-byte expectations use bytes the child must consume, not offered data.
#![allow(dead_code, unused_imports)]

#[path = "../support_rejoin_v2/mod.rs"]
mod existing;

pub use existing::{
    Peer, PeerMode, ProviderProcess, RejoinProcess, Service, accept_request, assert_busy,
    assert_no_connection, assert_port_closed, command, common, connect, failure, pending, pins,
    process_guard, provider, raw_request, read_response, ready, ready_with, receive_bytes,
    receive_complete, recover, remove, reserve, selected, set, state, success, sync, transcript,
};

use std::{collections::BTreeMap, ffi::OsString, net::SocketAddr, process::Output};

use mirror_chain::v2::ChainV2;
use mirror_network::v2::session::MAX_SESSION_WIRE_BYTES_V2;
use mirror_storage::v2::ReplacementCheckpointV2;

use state::{Scratch, checkpoint, history::main_chain};

pub const OPERATION: &str = "rejoin-failover-serve";
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
        "10000".into(),
    ]);
    result
}

#[derive(Clone, Copy)]
pub struct Trace<'a> {
    pub results: &'a [&'a str],
    pub bytes: u64,
    pub stop: &'a str,
}

impl Trace<'_> {
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
            .map_or(0, |index| index as u64 + 1)
    }
}

pub fn evidence(values: &BTreeMap<String, String>, trace: Trace<'_>) {
    assert!((1..=4).contains(&trace.results.len()));
    assert_eq!(values["peer_count"], trace.results.len().to_string());
    assert_eq!(values["validated_peer"], trace.validated().to_string());
    assert_eq!(values["receive_attempts"], trace.attempts().to_string());
    assert_eq!(values["receive_wire_bytes"], trace.bytes.to_string());
    assert_eq!(values["acquisition_stop"], trace.stop);
    let errors = trace
        .results
        .iter()
        .copied()
        .filter(|value| !matches!(*value, "not-attempted" | "validated"))
        .collect::<Vec<_>>();
    assert_eq!(values["failed_receives"], errors.len().to_string());
    assert_eq!(
        values["last_receive_error"],
        errors.last().copied().unwrap_or("none")
    );
    for (index, result) in trace.results.iter().enumerate() {
        assert_eq!(values[&format!("peer_{}_result", index + 1)], *result);
    }
    for index in trace.results.len() + 1..=5 {
        assert!(!values.contains_key(&format!("peer_{index}_result")));
    }
    assert_eq!(
        trace.attempts(),
        errors.len() as u64 + u64::from(trace.validated() != 0)
    );
    if trace.validated() != 0 {
        assert_eq!(trace.validated(), trace.attempts());
    }
}

pub struct Normal<'a> {
    pub previous: ReplacementCheckpointV2,
    pub selected: ReplacementCheckpointV2,
    pub choice: &'a str,
    pub reason: &'a str,
    pub service: Service<'a>,
}

impl Normal<'static> {
    pub fn adopted(candidate: &ChainV2, reason: &'static str) -> Self {
        Self {
            previous: checkpoint(&main_chain(3), 0),
            selected: checkpoint(candidate, 1),
            choice: "prefer-candidate",
            reason,
            service: Service::NONE,
        }
    }

    pub fn retained(choice: &'static str, reason: &'static str) -> Self {
        let previous = checkpoint(&main_chain(3), 0);
        Self {
            previous,
            selected: previous,
            choice,
            reason,
            service: Service::NONE,
        }
    }
}

pub fn stopped(
    output: &Output,
    expected: Normal<'_>,
    trace: Trace<'_>,
) -> BTreeMap<String, String> {
    assert!(output.stdout.len() <= 4096);
    let values = success(output, 0, OPERATION);
    assert_eq!(values.len(), 31 + trace.results.len());
    evidence(&values, trace);
    assert_eq!(values["status"], "stopped");
    assert_eq!(values["store_selection"], "verified");
    pins(&values, "previous", expected.previous);
    pins(&values, "selected", expected.selected);
    assert_eq!(
        values["receive_status"],
        if trace.validated() == 0 {
            "not-started"
        } else {
            "validated"
        }
    );
    assert_eq!(values["choice"], expected.choice);
    assert_eq!(values["reason"], expected.reason);
    let service = expected.service;
    let serve_attempts = u64::from(service.completed) + u64::from(service.rejected);
    for (key, expected) in [
        ("serve_attempts", serve_attempts),
        ("attempts", trace.attempts() + serve_attempts),
        ("completed_sessions", u64::from(service.completed)),
        ("rejected_sessions", u64::from(service.rejected)),
        ("serve_wire_bytes", service.bytes),
        ("wire_bytes", trace.bytes + service.bytes),
    ] {
        assert_eq!(values[key], expected.to_string(), "{key}");
        assert_eq!(values[key].parse::<u64>().unwrap().to_string(), values[key]);
    }
    assert_eq!(values["last_session_error"], service.last_error);
    values
}

pub fn failed(
    output: &Output,
    class: &str,
    trace: Trace<'_>,
    after_adoption: bool,
) -> BTreeMap<String, String> {
    assert!(output.stderr.len() <= 4096);
    let values = failure(output, 1);
    assert_eq!(values["operation"], OPERATION);
    assert_eq!(values["error"], class);
    evidence(&values, trace);
    assert!(!values.keys().any(|key| key.starts_with("selected_")));
    assert!(!values.contains_key("store_selection"));
    if after_adoption {
        assert_eq!(values["operation_may_have_completed"], "true");
        assert_eq!(values["next"], "inspect-status");
    } else {
        assert!(!values.contains_key("operation_may_have_completed"));
    }
    values
}

pub fn output_lost(output: &Output) {
    let values = failure(output, 1);
    assert!(output.stderr.len() <= 4096);
    assert_eq!(values["operation"], OPERATION);
    assert_eq!(values["error"], "output_io");
    assert_eq!(values["operation_may_have_completed"], "true");
    assert_eq!(values["next"], "inspect-status");
    assert!(!values.keys().any(|key| key.starts_with("selected_")));
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
