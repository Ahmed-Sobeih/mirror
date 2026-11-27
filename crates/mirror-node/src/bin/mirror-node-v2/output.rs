//! Bounded, deterministic output. Only static labels and validated numbers/hashes
//! enter these records; paths, raw arguments and lower-layer diagnostics do not.

use std::{fmt, fmt::Write as _, io};

use mirror_chain::v2::ManifestIdentityV2;
use mirror_network::v2::session::MAX_SESSION_WIRE_BYTES_V2;
use mirror_storage::v2::ReplacementCheckpointV2;

pub const MAX_OUTPUT_BYTES: usize = 4096;

pub struct Output {
    bytes: [u8; MAX_OUTPUT_BYTES],
    length: usize,
}

impl Output {
    pub const fn new() -> Self {
        Self {
            bytes: [0; MAX_OUTPUT_BYTES],
            length: 0,
        }
    }

    pub fn base(operation: &'static str) -> Result<Self, fmt::Error> {
        let mut output = Self::new();
        output.field("format", "mirror-node-v2-1")?;
        output.field("operation", operation)?;
        Ok(output)
    }

    pub fn state(
        operation: &'static str,
        status: &'static str,
        identity: ManifestIdentityV2,
        store_selection: &'static str,
    ) -> Result<Self, fmt::Error> {
        let mut output = Self::base(operation)?;
        output.field("status", status)?;
        output.field("config_hash", identity.config_hash.to_hex())?;
        output.field("genesis_hash", identity.genesis_hash.to_hex())?;
        output.field("store_selection", store_selection)?;
        Ok(output)
    }

    pub fn field(&mut self, key: &'static str, value: impl fmt::Display) -> fmt::Result {
        writeln!(self, "{key}={value}")
    }

    pub fn checkpoint(
        &mut self,
        prefix: &'static str,
        checkpoint: ReplacementCheckpointV2,
    ) -> fmt::Result {
        writeln!(self, "{prefix}_generation={}", checkpoint.generation)?;
        writeln!(self, "{prefix}_height={}", checkpoint.chain.height)?;
        writeln!(
            self,
            "{prefix}_tip_hash={}",
            checkpoint.chain.tip_hash.to_hex()
        )?;
        writeln!(
            self,
            "{prefix}_ledger_root={}",
            checkpoint.chain.ledger_root.to_hex()
        )
    }

    /// Complete short writes and report write/flush errors without panicking or
    /// repeating the operation. I/O itself has no deadline.
    pub fn write_to(&self, writer: &mut impl io::Write) -> io::Result<()> {
        writer.write_all(&self.bytes[..self.length])?;
        writer.flush()
    }
}

impl fmt::Write for Output {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.length.checked_add(text.len()).ok_or(fmt::Error)?;
        if end > MAX_OUTPUT_BYTES {
            return Err(fmt::Error);
        }
        self.bytes[self.length..end].copy_from_slice(text.as_bytes());
        self.length = end;
        Ok(())
    }
}

pub fn diagnostic(
    operation: &'static str,
    class: &'static str,
    next: &'static str,
    reason: Option<&'static str>,
    may_have_completed: Option<bool>,
) -> Result<Output, fmt::Error> {
    let mut output = Output::base(operation)?;
    output.field("error", class)?;
    if let Some(reason) = reason {
        output.field("reason", reason)?;
    }
    if let Some(value) = may_have_completed {
        output.field("operation_may_have_completed", value)?;
    }
    output.field("next", next)?;
    Ok(output)
}

/// Fixed, sanitized B3 evidence. Validate the complete trace before writing any
/// of it; malformed internal accounting cannot become a normal success record.
pub(super) fn acquisition_context(
    output: &mut Output,
    acquisition: &super::failover::Acquisition,
    include_stats: bool,
) -> fmt::Result {
    const KEYS: [&str; super::args::MAX_FAILOVER_PEERS] = [
        "peer_1_result",
        "peer_2_result",
        "peer_3_result",
        "peer_4_result",
    ];
    let count = usize::from(acquisition.peer_count);
    let attempted = usize::from(acquisition.stats.attempts);
    let validated = usize::from(acquisition.validated_peer);
    if count == 0
        || count > KEYS.len()
        || attempted > count
        || validated > count
        || (validated != 0 && validated != attempted)
        || acquisition
            .failed_receives
            .checked_add(u8::from(validated != 0))
            != Some(acquisition.stats.attempts)
        || acquisition.stats.wire_bytes > MAX_SESSION_WIRE_BYTES_V2
        || (attempted == 0 && acquisition.stats.wire_bytes != 0)
        || !matches!(
            acquisition.stop,
            "startup"
                | "fatal"
                | "wire-limit"
                | "attempt-limit"
                | "peers-exhausted"
                | "shutdown"
                | "validated"
        )
        || (acquisition.stop == "startup" && attempted != 0)
        || (acquisition.stop == "peers-exhausted" && attempted != count)
        || ((acquisition.stop == "validated") != (validated != 0))
        || (!include_stats
            && acquisition.stop != "validated"
            && !(acquisition.stop == "shutdown" && attempted == 0))
    {
        return Err(fmt::Error);
    }
    let mut last_error = "none";
    for (index, result) in acquisition.results.iter().enumerate() {
        if index >= attempted {
            if *result != "not-attempted" {
                return Err(fmt::Error);
            }
        } else if index + 1 == validated {
            if *result != "validated" {
                return Err(fmt::Error);
            }
        } else {
            if !matches!(
                *result,
                "invalid_session_limits"
                    | "invalid_peer_request"
                    | "peer_wrong_network"
                    | "peer_attempt_budget"
                    | "peer_io_budget"
                    | "peer_deadline"
                    | "peer_early_close"
                    | "peer_trailing_data"
                    | "allocation_failed"
                    | "peer_unreachable"
                    | "peer_io"
                    | "invalid_peer_history"
            ) {
                return Err(fmt::Error);
            }
            last_error = result;
        }
    }
    if acquisition.last_receive_error != last_error {
        return Err(fmt::Error);
    }
    output.field("peer_count", acquisition.peer_count)?;
    output.field("validated_peer", acquisition.validated_peer)?;
    output.field("failed_receives", acquisition.failed_receives)?;
    output.field("last_receive_error", acquisition.last_receive_error)?;
    output.field("acquisition_stop", acquisition.stop)?;
    if include_stats {
        output.field("receive_attempts", acquisition.stats.attempts)?;
        output.field("receive_wire_bytes", acquisition.stats.wire_bytes)?;
    }
    for (key, result) in KEYS.iter().zip(acquisition.results.iter()).take(count) {
        output.field(key, result)?;
    }
    Ok(())
}

pub fn help() -> Result<Output, fmt::Error> {
    let mut output = Output::base("help")?;
    for (key, value) in [
        (
            "commands",
            "init,status,sync-once,recover,serve,rejoin-serve,rejoin-failover-serve,help",
        ),
        (
            "common_options",
            "--control-dir DIR --store-dir DIR --config-hash HEX --genesis-hash HEX",
        ),
        (
            "init_options",
            "--attach-existing --height UINT --tip-hash HEX --ledger-root HEX --generation UINT",
        ),
        (
            "init_scope",
            "attach control to an already initialized profile-3 store using independent full pins",
        ),
        (
            "status_scope",
            "read-only control observation; missing means missing control, not missing storage",
        ),
        (
            "sync_once_options",
            "--peer ADDRESS --timeout-ms UINT --max-io-bytes UINT",
        ),
        (
            "sync_once_scope",
            "one numeric-address TCP request through F2 followed by consuming A1 adoption",
        ),
        ("recover_options", "--action complete|retain-old"),
        (
            "recover_scope",
            "explicit A1 recovery; no automatic retry or action switch",
        ),
        (
            "serve_options",
            "--listen ADDRESS --max-attempts UINT --max-io-bytes UINT --timeout-ms UINT --admission-ms UINT",
        ),
        (
            "serve_scope",
            "one fixed accepted history; at most one active stream; cumulative 1..8 attempts",
        ),
        (
            "serve_shutdown",
            "Linux SIGINT/SIGTERM stops admission; active session drains its original timeout",
        ),
        (
            "rejoin_serve_options",
            "--peer ADDRESS plus every serve option; one shared attempt and byte allowance",
        ),
        (
            "rejoin_serve_scope",
            "one complete receive, admitted durable keep/adopt, then finite service of the selected history",
        ),
        (
            "rejoin_serve_shutdown",
            "stop before adoption discards receipt; admitted adoption completes or fails; no automatic retry",
        ),
        (
            "rejoin_serve_timing",
            "receive timeout plus service admission window plus at most one final session timeout; local work/output excluded",
        ),
        (
            "rejoin_failover_serve_options",
            "--peers ADDRESS[,ADDRESS...] plus every serve option; one shared attempt and byte allowance",
        ),
        (
            "rejoin_failover_serve_scope",
            "1..4 ordered unique numeric peers; one request each; first validated candidate ends acquisition",
        ),
        (
            "rejoin_failover_serve_errors",
            "only the closed typed peer-failure allowlist may advance; no serve-old fallback or repeated entry",
        ),
        (
            "rejoin_failover_peer_rules",
            "no empty entries, whitespace or scopes; IPv6 and mapped-IPv4 aliases count as duplicate endpoints",
        ),
        (
            "rejoin_failover_timing",
            "at most min(peer count,attempt limit)*timeout plus service admission and one final timeout; local work/output excluded",
        ),
        (
            "admission_ms_range",
            "1..60000; admission window plus at most one original session timeout; opening/output excluded",
        ),
        ("help_scope", "help or --help alone"),
        (
            "paths",
            "explicit separate trusted control/store directories; no environment or configuration fallback",
        ),
        (
            "hash_encoding",
            "exactly 64 ASCII hex characters; either case accepted; lowercase output",
        ),
        (
            "uint_encoding",
            "canonical unsigned decimal; no leading zeros except 0",
        ),
        (
            "peer_encoding",
            "numeric IPv4:port or [IPv6]:port with nonzero port; no DNS",
        ),
        ("height_range", "0..4095"),
        ("generation_range", "0..height"),
        (
            "timeout_ms_range",
            "1..60000; session only; does not bound opening, adoption or output",
        ),
        (
            "max_io_bytes_range",
            "1..157129488; cumulative F2 application I/O",
        ),
        ("argument_count_max", "32; excluding executable name"),
        ("argument_bytes_max", "4096 per token; 16384 total"),
        (
            "argument_rules",
            "separate flag/value tokens; no duplicates, mixed-command flags, extras or control characters",
        ),
        ("output_bytes_max", "4096"),
        ("exit_0", "help, clean status or completed operation"),
        (
            "exit_1",
            "operational or output failure; inspect status before deciding an explicit next action",
        ),
        (
            "exit_2",
            "usage error; no filesystem or network operation started",
        ),
        ("exit_3", "status only: missing control"),
        (
            "exit_4",
            "status only: recovery required; store selection remains uninspected",
        ),
    ] {
        output.field(key, value)?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirror_chain::v2::ChainReplayExpectationV2;
    use mirror_crypto::Hash256;
    use mirror_network::v2::session::SessionStatsV2;

    fn acquisition() -> crate::failover::Acquisition {
        crate::failover::Acquisition {
            peer_count: 4,
            validated_peer: 4,
            failed_receives: 3,
            last_receive_error: "invalid_peer_history",
            stop: "validated",
            results: [
                "peer_unreachable",
                "peer_early_close",
                "invalid_peer_history",
                "validated",
            ],
            stats: SessionStatsV2 {
                attempts: 4,
                wire_bytes: MAX_SESSION_WIRE_BYTES_V2 - 1000,
            },
        }
    }

    #[test]
    fn pending_record_labels_intent_without_claiming_current_store_selection() {
        let identity = ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([0xab; 32]),
            genesis_hash: Hash256::from_bytes([0xcd; 32]),
        };
        let mut output =
            Output::state("status", "recovery-required", identity, "uninspected").unwrap();
        for (prefix, generation, height) in
            [("confirmed", 1, 5), ("old", 1, 5), ("candidate", 2, 6)]
        {
            output
                .checkpoint(
                    prefix,
                    ReplacementCheckpointV2 {
                        generation,
                        chain: ChainReplayExpectationV2 {
                            config_hash: identity.config_hash,
                            genesis_hash: identity.genesis_hash,
                            height,
                            tip_hash: Hash256::from_bytes([0xef; 32]),
                            ledger_root: Hash256::from_bytes([0x12; 32]),
                        },
                    },
                )
                .unwrap();
        }
        let mut bytes = Vec::new();
        output.write_to(&mut bytes).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("store_selection=uninspected\n"));
        assert!(text.contains("confirmed_height=5\n"));
        assert!(text.contains("candidate_generation=2\n"));
        assert!(!text.contains("selected_"));
        assert!(text.len() < MAX_OUTPUT_BYTES);
        assert!(text.lines().all(|line| line.contains('=')));
    }

    #[test]
    fn output_cap_is_checked_before_buffer_copy() {
        let mut output = Output::new();
        output.write_str(&"x".repeat(MAX_OUTPUT_BYTES)).unwrap();
        assert!(output.write_str("x").is_err());
        let mut bytes = Vec::new();
        output.write_to(&mut bytes).unwrap();
        assert_eq!(bytes.len(), MAX_OUTPUT_BYTES);
    }

    #[test]
    fn broken_writes_and_failed_flushes_return_errors_without_panics() {
        struct Failing {
            flush_only: bool,
        }
        impl io::Write for Failing {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.flush_only {
                    Ok(bytes.len())
                } else {
                    Err(io::ErrorKind::BrokenPipe.into())
                }
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
        }
        let output = help().unwrap();
        for flush_only in [false, true] {
            assert_eq!(
                output
                    .write_to(&mut Failing { flush_only })
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::BrokenPipe
            );
        }
    }

    #[test]
    fn help_and_output_failure_diagnostics_fit_the_bound_and_state_uncertainty() {
        let mut bytes = Vec::new();
        help().unwrap().write_to(&mut bytes).unwrap();
        assert!(bytes.len() < MAX_OUTPUT_BYTES);
        let mut bytes = Vec::new();
        diagnostic("sync-once", "output_io", "inspect-status", None, Some(true))
            .unwrap()
            .write_to(&mut bytes)
            .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("operation_may_have_completed=true\n"));
        assert!(text.ends_with("next=inspect-status\n"));
        assert!(!text.contains("status=clean"));
    }

    #[test]
    fn failover_maximum_normal_record_keeps_separate_phase_evidence_within_bound() {
        let identity = ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([0xab; 32]),
            genesis_hash: Hash256::from_bytes([0xcd; 32]),
        };
        let mut output =
            Output::state("rejoin-failover-serve", "stopped", identity, "verified").unwrap();
        for prefix in ["previous", "selected"] {
            output
                .checkpoint(
                    prefix,
                    ReplacementCheckpointV2 {
                        generation: 4095,
                        chain: ChainReplayExpectationV2 {
                            config_hash: identity.config_hash,
                            genesis_hash: identity.genesis_hash,
                            height: 4095,
                            tip_hash: Hash256::from_bytes([0xef; 32]),
                            ledger_root: Hash256::from_bytes([0x12; 32]),
                        },
                    },
                )
                .unwrap();
        }
        for (key, value) in [
            ("receive_status", "validated"),
            ("choice", "prefer-candidate"),
            ("reason", "attempt-limit"),
        ] {
            output.field(key, value).unwrap();
        }
        for (key, value) in [
            ("attempts", 8),
            ("receive_attempts", 4),
            ("serve_attempts", 4),
            ("completed_sessions", 3),
            ("rejected_sessions", 1),
            ("wire_bytes", MAX_SESSION_WIRE_BYTES_V2),
            ("receive_wire_bytes", MAX_SESSION_WIRE_BYTES_V2 - 1000),
            ("serve_wire_bytes", 1000),
        ] {
            output.field(key, value).unwrap();
        }
        output
            .field("last_session_error", "invalid_peer_request")
            .unwrap();
        acquisition_context(&mut output, &acquisition(), false).unwrap();
        let mut bytes = Vec::new();
        output.write_to(&mut bytes).unwrap();
        assert!(bytes.len() < MAX_OUTPUT_BYTES);
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(text.lines().count(), 35);
        assert!(text.contains("last_session_error=invalid_peer_request\n"));
        assert!(text.contains("last_receive_error=invalid_peer_history\n"));
        assert!(text.contains("peer_4_result=validated\n"));
        assert!(!text.contains("peer_5"));
        assert_eq!(text.matches("receive_attempts=").count(), 1);
        assert_eq!(text.matches("receive_wire_bytes=").count(), 1);
    }

    #[test]
    fn failover_failure_and_startup_context_never_claim_selected_state() {
        let mut failed = acquisition();
        failed.validated_peer = 0;
        failed.failed_receives = 4;
        failed.results[3] = "peer_deadline";
        failed.last_receive_error = "peer_deadline";
        failed.stop = "peers-exhausted";
        let startup = crate::failover::Acquisition {
            peer_count: 1,
            validated_peer: 0,
            failed_receives: 0,
            last_receive_error: "none",
            stop: "startup",
            results: ["not-attempted"; 4],
            stats: SessionStatsV2::default(),
        };
        for trace in [failed, startup, acquisition()] {
            let mut output = diagnostic(
                "rejoin-failover-serve",
                "control_io",
                "inspect-status",
                None,
                Some(true),
            )
            .unwrap();
            acquisition_context(&mut output, &trace, true).unwrap();
            let mut bytes = Vec::new();
            output.write_to(&mut bytes).unwrap();
            assert!(bytes.len() < MAX_OUTPUT_BYTES);
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains("receive_attempts="));
            assert!(text.contains("receive_wire_bytes="));
            assert!(!text.contains("selected_"));
            assert!(!text.contains("store_selection="));
            assert!(!text.contains("status=clean"));
            assert!(!text.contains("127.0.0.1"));
            assert_eq!(
                text.lines()
                    .filter(|line| line.starts_with("peer_") && line.contains("_result="))
                    .count(),
                usize::from(trace.peer_count)
            );
        }
        assert!(acquisition_context(&mut Output::new(), &failed, false).is_err());
        let mut stopped = startup;
        stopped.stop = "shutdown";
        acquisition_context(&mut Output::new(), &stopped, false).unwrap();
    }

    #[test]
    fn failover_inconsistent_internal_evidence_rejects_before_fields_are_written() {
        let valid = acquisition();
        let mut invalid = Vec::new();
        for count in [0, 5] {
            let mut trace = valid;
            trace.peer_count = count;
            invalid.push(trace);
        }
        for ordinal in [1, 5] {
            let mut trace = valid;
            trace.validated_peer = ordinal;
            invalid.push(trace);
        }
        let mut trace = valid;
        trace.stats.attempts = 5;
        invalid.push(trace);
        let mut trace = valid;
        trace.failed_receives = 4;
        invalid.push(trace);
        let mut trace = valid;
        trace.last_receive_error = "peer_deadline";
        invalid.push(trace);
        let mut trace = valid;
        trace.results[0] = "not-attempted";
        invalid.push(trace);
        let mut trace = valid;
        trace.results[0] = "validated";
        invalid.push(trace);
        let mut trace = valid;
        trace.results[0] = "secret\nstatus=clean";
        invalid.push(trace);
        let mut trace = valid;
        trace.stats.wire_bytes = MAX_SESSION_WIRE_BYTES_V2 + 1;
        invalid.push(trace);
        for stop in ["startup", "fatal", "peers-exhausted", "unknown"] {
            let mut trace = valid;
            trace.stop = stop;
            invalid.push(trace);
        }
        for trace in invalid {
            let mut output = Output::base("rejoin-failover-serve").unwrap();
            let before = output.length;
            assert!(acquisition_context(&mut output, &trace, true).is_err());
            assert_eq!(output.length, before);
        }
    }
}
