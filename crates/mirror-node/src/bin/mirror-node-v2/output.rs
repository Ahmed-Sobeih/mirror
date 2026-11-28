//! Bounded, deterministic output. Only static labels and validated numbers/hashes
//! enter these records; paths, raw arguments and lower-layer diagnostics do not.

use std::{fmt, fmt::Write as _, io};

use mirror_chain::v2::{ForkChoiceV2, MAX_CHAIN_BLOCKS_V2, ManifestIdentityV2};
use mirror_core::v2::MIN_ENCODED_BLOCK_LEN_V2;
use mirror_network::v2::session::{
    MAX_SESSION_ATTEMPTS_V2, MAX_SESSION_WIRE_BYTES_V2, SessionStatsV2,
};
use mirror_storage::v2::ReplacementCheckpointV2;

use super::{
    errors::Failure,
    pool_inspect::{BlockedReason, Evaluation, Report as PoolInspectionReport},
};
use mirror_node::v2::pool::MAX_POOL_OPERATION_BYTES_V2;

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

    fn prefixed_field(
        &mut self,
        prefix: &'static str,
        key: &'static str,
        value: impl fmt::Display,
    ) -> fmt::Result {
        writeln!(self, "{prefix}_{key}={value}")
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
fn validate_acquisition(
    acquisition: &super::failover::Acquisition,
    include_stats: bool,
) -> fmt::Result {
    let count = usize::from(acquisition.peer_count);
    let attempted = usize::from(acquisition.stats.attempts);
    let validated = usize::from(acquisition.validated_peer);
    if count == 0
        || count > super::args::MAX_FAILOVER_PEERS
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
    Ok(())
}

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
    validate_acquisition(acquisition, include_stats)?;
    let count = usize::from(acquisition.peer_count);
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

fn static_session_error(label: &str) -> bool {
    matches!(
        label,
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
    )
}

fn valid_stats(stats: SessionStatsV2) -> bool {
    stats.attempts <= MAX_SESSION_ATTEMPTS_V2
        && stats.wire_bytes <= MAX_SESSION_WIRE_BYTES_V2
        && (stats.attempts != 0 || stats.wire_bytes == 0)
}

fn checked_delta(
    total: SessionStatsV2,
    before: SessionStatsV2,
) -> Result<SessionStatsV2, fmt::Error> {
    let result = SessionStatsV2 {
        attempts: total
            .attempts
            .checked_sub(before.attempts)
            .ok_or(fmt::Error)?,
        wire_bytes: total
            .wire_bytes
            .checked_sub(before.wire_bytes)
            .ok_or(fmt::Error)?,
    };
    if !valid_stats(total) || !valid_stats(before) || !valid_stats(result) {
        return Err(fmt::Error);
    }
    Ok(result)
}

fn add_stats(total: &mut SessionStatsV2, value: SessionStatsV2) -> fmt::Result {
    *total = SessionStatsV2 {
        attempts: total
            .attempts
            .checked_add(value.attempts)
            .ok_or(fmt::Error)?,
        wire_bytes: total
            .wire_bytes
            .checked_add(value.wire_bytes)
            .ok_or(fmt::Error)?,
    };
    if !valid_stats(*total) || !valid_stats(value) {
        return Err(fmt::Error);
    }
    Ok(())
}

fn validate_run_trace(trace: &super::runner::RunTrace) -> Result<SessionStatsV2, fmt::Error> {
    let incomplete = trace
        .rounds_started
        .checked_sub(trace.rounds_completed)
        .ok_or(fmt::Error)?;
    if trace.round_limit == 0
        || usize::from(trace.round_limit) > super::args::MAX_RUN_ROUNDS
        || trace.peer_count == 0
        || usize::from(trace.peer_count) > super::args::MAX_FAILOVER_PEERS
        || trace.rounds_started > trace.round_limit
        || incomplete > 1
        || (trace.rounds_completed != 0 && !trace.adoption_admitted)
        || !matches!(
            trace.phase,
            "startup" | "acquisition" | "adoption-service" | "wait" | "report"
        )
        || !(trace.last_completed_service_error == "none"
            || static_session_error(trace.last_completed_service_error))
        || ((trace.failed_receives == 0) != (trace.last_receive_error == "none"))
        || (trace.failed_receives != 0 && !static_session_error(trace.last_receive_error))
    {
        return Err(fmt::Error);
    }
    let serve = checked_delta(trace.stats, trace.receive)?;
    let mut validated = trace.rounds_completed;
    match trace.acquisition {
        Some(acquisition) => {
            validate_acquisition(&acquisition, true)?;
            if incomplete != 1 || acquisition.peer_count != trace.peer_count {
                return Err(fmt::Error);
            }
            checked_delta(trace.receive, acquisition.stats)?;
            if trace.failed_receives < acquisition.failed_receives
                || (acquisition.failed_receives != 0
                    && trace.last_receive_error != acquisition.last_receive_error)
            {
                return Err(fmt::Error);
            }
            validated = validated
                .checked_add(u8::from(acquisition.validated_peer != 0))
                .ok_or(fmt::Error)?;
        }
        None if incomplete != 0 => return Err(fmt::Error),
        None => {}
    }
    if trace.failed_receives.checked_add(validated) != Some(trace.receive.attempts)
        || ((trace.rounds_completed == 0 || serve.attempts == 0)
            && trace.last_completed_service_error != "none")
        || (trace.phase == "startup"
            && (trace.rounds_started != 0
                || trace.stats != SessionStatsV2::default()
                || trace.adoption_admitted))
        || (trace.phase == "wait" && (trace.acquisition.is_some() || trace.rounds_completed == 0))
        || (matches!(trace.phase, "acquisition" | "adoption-service")
            && trace.acquisition.is_none())
    {
        return Err(fmt::Error);
    }
    Ok(serve)
}

struct PeerResults<'a>(&'a [&'static str]);

impl fmt::Display for PeerResults<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, result) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str(",")?;
            }
            formatter.write_str(result)?;
        }
        Ok(())
    }
}

/// Validate all optional invocation evidence before appending any fields. No
/// checkpoint or unreturned service-completion count enters a failure record.
pub(super) fn run_context(output: &mut Output, trace: &super::runner::RunTrace) -> fmt::Result {
    let serve = validate_run_trace(trace)?;
    output.field("round_limit", trace.round_limit)?;
    output.field("rounds_started", trace.rounds_started)?;
    output.field("rounds_completed", trace.rounds_completed)?;
    output.field("phase", trace.phase)?;
    output.field("adoption_admitted", trace.adoption_admitted)?;
    output.field("peer_count", trace.peer_count)?;
    output.field("attempts", trace.stats.attempts)?;
    output.field("wire_bytes", trace.stats.wire_bytes)?;
    output.field("receive_attempts", trace.receive.attempts)?;
    output.field("receive_wire_bytes", trace.receive.wire_bytes)?;
    output.field("serve_attempts", serve.attempts)?;
    output.field("serve_wire_bytes", serve.wire_bytes)?;
    output.field("failed_receives", trace.failed_receives)?;
    output.field("last_receive_error", trace.last_receive_error)?;
    output.field(
        "last_completed_service_error",
        trace.last_completed_service_error,
    )?;
    if let Some(acquisition) = trace.acquisition {
        output.field("current_round", trace.rounds_started)?;
        output.field("current_validated_peer", acquisition.validated_peer)?;
        output.field("current_failed_receives", acquisition.failed_receives)?;
        output.field("current_receive_attempts", acquisition.stats.attempts)?;
        output.field("current_receive_wire_bytes", acquisition.stats.wire_bytes)?;
        output.field("current_acquisition_stop", acquisition.stop)?;
        output.field("current_last_receive_error", acquisition.last_receive_error)?;
        output.field(
            "current_peer_results",
            PeerResults(&acquisition.results[..usize::from(acquisition.peer_count)]),
        )?;
    }
    Ok(())
}

fn valid_checkpoint(checkpoint: ReplacementCheckpointV2, identity: ManifestIdentityV2) -> bool {
    checkpoint.chain.config_hash == identity.config_hash
        && checkpoint.chain.genesis_hash == identity.genesis_hash
        && checkpoint.chain.height < MAX_CHAIN_BLOCKS_V2 as u64
        && checkpoint.generation <= checkpoint.chain.height
}

fn validate_run_report(
    identity: ManifestIdentityV2,
    report: &super::runner::Report,
) -> Result<SessionStatsV2, fmt::Error> {
    let trace = &report.trace;
    let serve = validate_run_trace(trace)?;
    if trace.phase != "report"
        || trace.adoption_admitted != (trace.rounds_completed != 0)
        || !valid_checkpoint(report.initial, identity)
        || !valid_checkpoint(report.selected, identity)
        || !matches!(
            report.reason,
            "shutdown" | "wire-limit" | "attempt-limit" | "round-limit"
        )
        || (report.reason == "round-limit" && trace.rounds_completed != trace.round_limit)
        || (report.reason == "attempt-limit" && trace.stats.attempts == 0)
        || (report.reason == "wire-limit" && trace.stats.wire_bytes == 0)
        || (trace.acquisition.is_some() && report.reason != "shutdown")
    {
        return Err(fmt::Error);
    }
    let mut previous = report.initial;
    let mut received = SessionStatsV2::default();
    let mut served = SessionStatsV2::default();
    let mut completed = 0u8;
    let mut rejected = 0u8;
    let mut failed = 0u8;
    let mut last_receive = "none";
    let mut last_service = "none";
    for (index, entry) in report.rounds.iter().enumerate() {
        if index >= usize::from(trace.rounds_completed) {
            if entry.is_some() {
                return Err(fmt::Error);
            }
            continue;
        }
        let round = entry.as_ref().ok_or(fmt::Error)?;
        validate_acquisition(&round.acquisition, false)?;
        if round.previous != previous
            || !valid_checkpoint(round.selected, identity)
            || round.acquisition.peer_count != trace.peer_count
            || round.acquisition.validated_peer == 0
            || round.acquisition.stop != "validated"
            || round.completed.checked_add(round.rejected) != Some(round.serve_stats.attempts)
            || !(round.last_session_error == "none"
                || static_session_error(round.last_session_error))
            || ((round.rejected == 0) != (round.last_session_error == "none"))
            || !matches!(
                round.service_reason,
                "shutdown" | "wire-limit" | "attempt-limit" | "admission-timeout"
            )
            || (index + 1 < usize::from(trace.rounds_started)
                && round.service_reason != "admission-timeout")
        {
            return Err(fmt::Error);
        }
        match round.choice {
            ForkChoiceV2::KeepCurrent if round.selected != previous => return Err(fmt::Error),
            ForkChoiceV2::PreferCandidate
                if previous.generation.checked_add(1) != Some(round.selected.generation)
                    || round.selected.chain.height <= previous.chain.height =>
            {
                return Err(fmt::Error);
            }
            _ => {}
        }
        add_stats(&mut received, round.acquisition.stats)?;
        add_stats(&mut served, round.serve_stats)?;
        completed = completed.checked_add(round.completed).ok_or(fmt::Error)?;
        rejected = rejected.checked_add(round.rejected).ok_or(fmt::Error)?;
        failed = failed
            .checked_add(round.acquisition.failed_receives)
            .ok_or(fmt::Error)?;
        if round.acquisition.last_receive_error != "none" {
            last_receive = round.acquisition.last_receive_error;
        }
        if round.last_session_error != "none" {
            last_service = round.last_session_error;
        }
        previous = round.selected;
    }
    if let Some(acquisition) = trace.acquisition {
        if !((acquisition.stop == "shutdown" && acquisition.stats.attempts == 0)
            || (acquisition.stop == "validated" && acquisition.validated_peer != 0))
        {
            return Err(fmt::Error);
        }
        add_stats(&mut received, acquisition.stats)?;
        failed = failed
            .checked_add(acquisition.failed_receives)
            .ok_or(fmt::Error)?;
        if acquisition.last_receive_error != "none" {
            last_receive = acquisition.last_receive_error;
        }
    }
    if received != trace.receive
        || served != serve
        || previous != report.selected
        || failed != trace.failed_receives
        || last_receive != trace.last_receive_error
        || last_service != trace.last_completed_service_error
        || completed != report.completed
        || rejected != report.rejected
        || completed.checked_add(rejected) != Some(serve.attempts)
    {
        return Err(fmt::Error);
    }
    Ok(serve)
}

/// The owner has already dropped when this validates the fixed history of
/// completed rounds and formats the only normal invocation record.
pub(super) fn run_report(
    identity: ManifestIdentityV2,
    report: &super::runner::Report,
) -> Result<Output, fmt::Error> {
    const PREFIXES: [&str; super::args::MAX_RUN_ROUNDS] =
        ["round_1", "round_2", "round_3", "round_4"];
    let serve = validate_run_report(identity, report)?;
    let trace = &report.trace;
    let mut output = Output::state("run", "stopped", identity, "verified")?;
    output.checkpoint("initial", report.initial)?;
    output.checkpoint("selected", report.selected)?;
    output.field("round_limit", trace.round_limit)?;
    output.field("rounds_started", trace.rounds_started)?;
    output.field("rounds_completed", trace.rounds_completed)?;
    output.field("reason", report.reason)?;
    output.field("peer_count", trace.peer_count)?;
    output.field("attempts", trace.stats.attempts)?;
    output.field("receive_attempts", trace.receive.attempts)?;
    output.field("serve_attempts", serve.attempts)?;
    output.field("completed_sessions", report.completed)?;
    output.field("rejected_sessions", report.rejected)?;
    output.field("wire_bytes", trace.stats.wire_bytes)?;
    output.field("receive_wire_bytes", trace.receive.wire_bytes)?;
    output.field("serve_wire_bytes", serve.wire_bytes)?;
    output.field("failed_receives", trace.failed_receives)?;
    output.field("last_receive_error", trace.last_receive_error)?;
    output.field("last_session_error", trace.last_completed_service_error)?;
    for (prefix, entry) in PREFIXES
        .iter()
        .zip(&report.rounds)
        .take(usize::from(trace.rounds_completed))
    {
        let round = entry.as_ref().ok_or(fmt::Error)?;
        output.checkpoint(prefix, round.selected)?;
        output.prefixed_field(
            prefix,
            "choice",
            match round.choice {
                ForkChoiceV2::KeepCurrent => "keep-current",
                ForkChoiceV2::PreferCandidate => "prefer-candidate",
            },
        )?;
        output.prefixed_field(prefix, "validated_peer", round.acquisition.validated_peer)?;
        output.prefixed_field(prefix, "receive_attempts", round.acquisition.stats.attempts)?;
        output.prefixed_field(
            prefix,
            "receive_wire_bytes",
            round.acquisition.stats.wire_bytes,
        )?;
        output.prefixed_field(prefix, "serve_attempts", round.serve_stats.attempts)?;
        output.prefixed_field(prefix, "serve_wire_bytes", round.serve_stats.wire_bytes)?;
        output.prefixed_field(prefix, "completed_sessions", round.completed)?;
        output.prefixed_field(prefix, "rejected_sessions", round.rejected)?;
        output.prefixed_field(prefix, "service_reason", round.service_reason)?;
        output.prefixed_field(
            prefix,
            "last_receive_error",
            round.acquisition.last_receive_error,
        )?;
        output.prefixed_field(prefix, "last_session_error", round.last_session_error)?;
        output.prefixed_field(
            prefix,
            "peer_results",
            PeerResults(&round.acquisition.results[..usize::from(trace.peer_count)]),
        )?;
    }
    if let Some(acquisition) = trace.acquisition {
        output.field("incomplete_round", trace.rounds_started)?;
        output.field("incomplete_receive_attempts", acquisition.stats.attempts)?;
        output.field(
            "incomplete_receive_wire_bytes",
            acquisition.stats.wire_bytes,
        )?;
        output.field("incomplete_validated_peer", acquisition.validated_peer)?;
        output.field(
            "incomplete_last_receive_error",
            acquisition.last_receive_error,
        )?;
        output.field("incomplete_acquisition_stop", acquisition.stop)?;
        output.field(
            "incomplete_peer_results",
            PeerResults(&acquisition.results[..usize::from(trace.peer_count)]),
        )?;
    }
    Ok(output)
}

/// Validate only fixed scalar/checkpoint observations after the C2 resource
/// owners have dropped. This record includes expected blocked evaluation;
/// neither retention nor execution is a claim of canonical inclusion.
fn validate_pool_inspection(
    identity: ManifestIdentityV2,
    report: &PoolInspectionReport,
) -> Result<(), Failure> {
    let invalid = || Failure::new("pool_invariant");
    let options = report.options;
    let limits = options.limits;
    options.validate().map_err(|_| invalid())?;
    if report.checkpoint.chain.config_hash != identity.config_hash
        || report.checkpoint.chain.genesis_hash != identity.genesis_hash
        || report.checkpoint.chain.height >= MAX_CHAIN_BLOCKS_V2 as u64
        || report.checkpoint.generation > report.checkpoint.chain.height
        || report.input_bytes > options.max_input_bytes
        || report.input_count > options.max_input_count
        || report.retained_count > limits.max_entries
        || report.retained_count.checked_add(report.duplicate_count) != Some(report.input_count)
        || report.retained_encoded_bytes > limits.max_encoded_bytes
        || report.work_used > limits.max_work_units
        || ((report.input_count == 0) != (report.input_bytes == 0))
        || ((report.input_count == 0) != (report.retained_count == 0))
        || ((report.retained_count == 0) != (report.retained_encoded_bytes == 0))
        || ((report.input_count == 0) != (report.work_used == 0))
    {
        return Err(invalid());
    }
    // Each record has one LF and twice its decoded byte length. Duplicates
    // remain in the input totals but contribute no new retained bytes.
    let hex_bytes = report
        .input_bytes
        .checked_sub(report.input_count)
        .filter(|bytes| bytes % 2 == 0)
        .ok_or_else(invalid)?;
    let decoded_input_bytes = hex_bytes / 2;
    let minimum_decoded_input = report
        .retained_encoded_bytes
        .checked_add(report.duplicate_count)
        .ok_or_else(invalid)?;
    let maximum_decoded_input = report
        .duplicate_count
        .checked_mul(MAX_POOL_OPERATION_BYTES_V2)
        .and_then(|bytes| bytes.checked_add(report.retained_encoded_bytes))
        .ok_or_else(invalid)?;
    let maximum_retained = report
        .retained_count
        .checked_mul(MAX_POOL_OPERATION_BYTES_V2)
        .ok_or_else(invalid)?;
    if decoded_input_bytes < minimum_decoded_input
        || decoded_input_bytes > maximum_decoded_input
        || report.retained_encoded_bytes < report.retained_count
        || report.retained_encoded_bytes > maximum_retained
    {
        return Err(invalid());
    }
    match report.evaluation {
        Evaluation::NotRequested => {
            if options.evaluate_count != 0 {
                return Err(invalid());
            }
        }
        Evaluation::Evaluated { count, block_bytes } => {
            if count == 0 || count != options.evaluate_count || count > report.retained_count {
                return Err(invalid());
            }
            let framing = count
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(MIN_ENCODED_BLOCK_LEN_V2))
                .ok_or_else(invalid)?;
            let encoded_bytes = block_bytes.checked_sub(framing).ok_or_else(invalid)?;
            let maximum_bytes = count
                .checked_mul(MAX_POOL_OPERATION_BYTES_V2)
                .ok_or_else(invalid)?;
            if block_bytes > limits.max_batch_bytes
                || encoded_bytes < count
                || encoded_bytes > maximum_bytes
                || encoded_bytes > report.retained_encoded_bytes
                || (count == report.retained_count
                    && encoded_bytes != report.retained_encoded_bytes)
            {
                return Err(invalid());
            }
        }
        Evaluation::Blocked { reason, index } => {
            if options.evaluate_count == 0
                || (matches!(reason, BlockedReason::InvalidCount)
                    != (options.evaluate_count > report.retained_count))
                || index.is_some_and(|index| {
                    index >= options.evaluate_count
                        || index >= report.retained_count
                        || !matches!(
                            reason,
                            BlockedReason::Operation | BlockedReason::Body | BlockedReason::Ledger
                        )
                })
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

pub(super) fn pool_inspection(
    identity: ManifestIdentityV2,
    report: &PoolInspectionReport,
) -> Result<Output, Failure> {
    validate_pool_inspection(identity, report)?;
    let options = report.options;
    let limits = options.limits;
    let mut output = Output::state("pool-inspect", "inspected", identity, "confirmed")?;
    output.checkpoint("selected", report.checkpoint)?;
    output.field("input_bytes", report.input_bytes)?;
    output.field("input_count", report.input_count)?;
    output.field("retained_count", report.retained_count)?;
    output.field("duplicate_count", report.duplicate_count)?;
    output.field("retained_encoded_bytes", report.retained_encoded_bytes)?;
    output.field("work_used", report.work_used)?;
    output.field("max_input_bytes", options.max_input_bytes)?;
    output.field("max_input_count", options.max_input_count)?;
    output.field("max_entries", limits.max_entries)?;
    output.field("max_encoded_bytes", limits.max_encoded_bytes)?;
    output.field("max_batch_operations", limits.max_batch_operations)?;
    output.field("max_batch_bytes", limits.max_batch_bytes)?;
    output.field("max_work_units", limits.max_work_units)?;
    output.field("evaluation_requested_count", options.evaluate_count)?;
    let (evaluation, reason, index, count, block_bytes) = match report.evaluation {
        Evaluation::NotRequested => ("not-requested", "none", None, 0, 0),
        Evaluation::Evaluated { count, block_bytes } => {
            ("evaluated", "none", None, count, block_bytes)
        }
        Evaluation::Blocked { reason, index } => ("blocked", reason.as_str(), index, 0, 0),
    };
    output.field("evaluation", evaluation)?;
    output.field("evaluation_reason", reason)?;
    match index {
        Some(index) => output.field("evaluation_operation_index", index)?,
        None => output.field("evaluation_operation_index", "none")?,
    }
    output.field("evaluated_count", count)?;
    output.field("evaluated_block_bytes", block_bytes)?;
    output.field("canonical_inclusion", "not-checked")?;
    output.field("pool_disposition", "discarded")?;
    Ok(output)
}

pub fn help() -> Result<Output, fmt::Error> {
    let mut output = Output::base("help")?;
    for (key, value) in [
        (
            "commands",
            "init,status,sync-once,recover,serve,rejoin-serve,pool-inspect,rejoin-failover-serve,run,help",
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
        (
            "run_options",
            "--peers ADDRESS[,ADDRESS...] plus every serve option; --max-rounds UINT --round-wait-ms UINT",
        ),
        (
            "run_scope",
            "1..4 explicit rounds; wait 0..60000 ms; reuse ordered peers after successful rounds; one cumulative allowance",
        ),
        (
            "run_timing",
            "M*timeout + R*admission + (R-1)*wait; at most 900000 ms; local work/output excluded",
        ),
        (
            "pool_options",
            "--input-file FILE --max-input-bytes UINT --max-input-count UINT --max-entries UINT --max-encoded-bytes UINT --max-batch-operations UINT --max-batch-bytes UINT --max-work-units UINT --evaluate-count UINT",
        ),
        (
            "pool_scope",
            "finite local admission and optional exact FIFO evaluation; no inclusion or publication claim",
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
mod pool_tests {
    use super::*;
    use crate::args::PoolInspectOptions;
    use mirror_chain::v2::ChainReplayExpectationV2;
    use mirror_crypto::Hash256;
    use mirror_node::v2::pool::{
        MAX_POOL_BATCH_BYTES_V2, MAX_POOL_BATCH_OPERATIONS_V2, MAX_POOL_ENCODED_BYTES_V2,
        MAX_POOL_ENTRIES_V2, MAX_POOL_WORK_UNITS_V2, PendingPoolLimitsV2,
    };

    fn pool_fixture() -> (ManifestIdentityV2, PoolInspectionReport) {
        let identity = ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([0xab; 32]),
            genesis_hash: Hash256::from_bytes([0xcd; 32]),
        };
        (
            identity,
            PoolInspectionReport {
                checkpoint: ReplacementCheckpointV2 {
                    generation: 1,
                    chain: ChainReplayExpectationV2 {
                        config_hash: identity.config_hash,
                        genesis_hash: identity.genesis_hash,
                        height: 2,
                        tip_hash: Hash256::from_bytes([0xef; 32]),
                        ledger_root: Hash256::from_bytes([0x12; 32]),
                    },
                },
                options: PoolInspectOptions {
                    max_input_bytes: crate::pool_input::MAX_INPUT_BYTES,
                    max_input_count: crate::pool_input::MAX_INPUT_COUNT,
                    limits: PendingPoolLimitsV2 {
                        max_entries: MAX_POOL_ENTRIES_V2,
                        max_encoded_bytes: MAX_POOL_ENCODED_BYTES_V2,
                        max_batch_operations: MAX_POOL_BATCH_OPERATIONS_V2,
                        max_batch_bytes: MAX_POOL_BATCH_BYTES_V2,
                        max_work_units: MAX_POOL_WORK_UNITS_V2,
                    },
                    evaluate_count: 2,
                },
                input_bytes: 1461,
                input_count: 3,
                retained_count: 2,
                duplicate_count: 1,
                retained_encoded_bytes: 486,
                work_used: 2000,
                evaluation: Evaluation::Evaluated {
                    count: 2,
                    block_bytes: 662,
                },
            },
        )
    }

    fn pool_text(identity: ManifestIdentityV2, report: &PoolInspectionReport) -> String {
        let output = pool_inspection(identity, report)
            .unwrap_or_else(|failure| panic!("unexpected report error: {}", failure.class));
        let mut bytes = Vec::new();
        output.write_to(&mut bytes).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    fn pool_invariant(identity: ManifestIdentityV2, report: &PoolInspectionReport) {
        let Err(failure) = pool_inspection(identity, report) else {
            panic!("inconsistent report was formatted");
        };
        assert_eq!(failure.class, "pool_invariant");
    }

    #[test]
    fn pool_report_has_exact_unique_fields_and_separate_scope_claims() {
        let (identity, report) = pool_fixture();
        let text = pool_text(identity, &report);
        let keys: Vec<_> = text
            .lines()
            .map(|line| line.split_once('=').unwrap().0)
            .collect();
        assert_eq!(
            keys,
            [
                "format",
                "operation",
                "status",
                "config_hash",
                "genesis_hash",
                "store_selection",
                "selected_generation",
                "selected_height",
                "selected_tip_hash",
                "selected_ledger_root",
                "input_bytes",
                "input_count",
                "retained_count",
                "duplicate_count",
                "retained_encoded_bytes",
                "work_used",
                "max_input_bytes",
                "max_input_count",
                "max_entries",
                "max_encoded_bytes",
                "max_batch_operations",
                "max_batch_bytes",
                "max_work_units",
                "evaluation_requested_count",
                "evaluation",
                "evaluation_reason",
                "evaluation_operation_index",
                "evaluated_count",
                "evaluated_block_bytes",
                "canonical_inclusion",
                "pool_disposition",
            ]
        );
        assert!(
            text.starts_with("format=mirror-node-v2-1\noperation=pool-inspect\nstatus=inspected\n")
        );
        assert!(text.contains("store_selection=confirmed\n"));
        assert!(text.contains("input_count=3\nretained_count=2\nduplicate_count=1\n"));
        assert!(text.contains("evaluation=evaluated\nevaluation_reason=none\nevaluation_operation_index=none\nevaluated_count=2\nevaluated_block_bytes=662\n"));
        assert!(text.ends_with("canonical_inclusion=not-checked\npool_disposition=discarded\n"));
        assert_eq!(report.exit_code(), 0);
        assert_eq!(text, pool_text(identity, &report));
    }

    #[test]
    fn pool_empty_input_and_explicit_no_evaluation_are_complete_observations() {
        let (identity, mut report) = pool_fixture();
        report.options.evaluate_count = 0;
        report.evaluation = Evaluation::NotRequested;
        let retained = pool_text(identity, &report);
        assert!(retained.contains("evaluation=not-requested\n"));
        assert!(retained.contains("evaluated_count=0\nevaluated_block_bytes=0\n"));
        report.input_bytes = 0;
        report.input_count = 0;
        report.retained_count = 0;
        report.duplicate_count = 0;
        report.retained_encoded_bytes = 0;
        report.work_used = 0;
        let empty = pool_text(identity, &report);
        assert_eq!(empty.lines().count(), 31);
        assert!(empty.contains("input_bytes=0\ninput_count=0\nretained_count=0\n"));
        assert!(empty.contains("work_used=0\n"));
        assert_eq!(report.exit_code(), 0);
    }

    #[test]
    fn pool_each_closed_blocked_reason_keeps_zero_success_and_typed_index() {
        for (reason, index, label) in [
            (BlockedReason::InvalidCount, None, "invalid-count"),
            (BlockedReason::BatchByteLimit, None, "batch-byte-limit"),
            (BlockedReason::WorkLimit, None, "work-limit"),
            (BlockedReason::Operation, Some(1), "operation"),
            (BlockedReason::Body, Some(1), "body"),
            (BlockedReason::Ledger, Some(1), "ledger"),
        ] {
            let (identity, mut report) = pool_fixture();
            if reason == BlockedReason::InvalidCount {
                report.options.evaluate_count = 3;
            }
            report.evaluation = Evaluation::Blocked { reason, index };
            let text = pool_text(identity, &report);
            assert_eq!(text.lines().count(), 31);
            assert!(text.contains(&format!("evaluation=blocked\nevaluation_reason={label}\n")));
            let index = index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".into());
            assert!(text.contains(&format!("evaluation_operation_index={index}\n")));
            assert!(text.contains("evaluated_count=0\nevaluated_block_bytes=0\n"));
            assert_eq!(report.exit_code(), 1);
        }
    }

    #[test]
    fn pool_success_can_describe_only_the_requested_fifo_prefix() {
        let (identity, mut report) = pool_fixture();
        report.options.evaluate_count = 1;
        report.evaluation = Evaluation::Evaluated {
            count: 1,
            block_bytes: 415,
        };
        let text = pool_text(identity, &report);
        assert!(text.contains("retained_count=2\n"));
        assert!(text.contains("evaluation_requested_count=1\n"));
        assert!(text.contains("evaluated_count=1\nevaluated_block_bytes=415\n"));
    }

    #[test]
    fn pool_malformed_parent_counts_bytes_work_and_limits_never_format() {
        let mutations: &[fn(&mut PoolInspectionReport)] = &[
            |report| report.checkpoint.chain.config_hash = Hash256::from_bytes([1; 32]),
            |report| report.checkpoint.chain.genesis_hash = Hash256::from_bytes([1; 32]),
            |report| report.checkpoint.chain.height = MAX_CHAIN_BLOCKS_V2 as u64,
            |report| report.checkpoint.generation = report.checkpoint.chain.height + 1,
            |report| report.options.max_input_bytes = 0,
            |report| {
                report.options.limits.max_batch_operations = report.options.limits.max_entries + 1
            },
            |report| report.input_bytes = report.options.max_input_bytes + 1,
            |report| report.input_bytes = 0,
            |report| report.input_bytes = 1,
            |report| report.input_bytes = 1462,
            |report| report.input_bytes = 975,
            |report| report.input_count = report.options.max_input_count + 1,
            |report| report.input_count = 0,
            |report| report.retained_count = report.options.limits.max_entries + 1,
            |report| report.duplicate_count = usize::MAX,
            |report| report.duplicate_count = 0,
            |report| report.retained_encoded_bytes = report.options.limits.max_encoded_bytes + 1,
            |report| report.retained_encoded_bytes = 0,
            |report| report.retained_encoded_bytes = 1,
            |report| report.work_used = report.options.limits.max_work_units + 1,
            |report| report.work_used = 0,
        ];
        for mutation in mutations {
            let (identity, mut report) = pool_fixture();
            mutation(&mut report);
            pool_invariant(identity, &report);
        }
    }

    #[test]
    fn pool_malformed_evaluation_or_impossible_index_never_format() {
        for (requested, evaluation) in [
            (2, Evaluation::NotRequested),
            (
                0,
                Evaluation::Evaluated {
                    count: 2,
                    block_bytes: 662,
                },
            ),
            (
                2,
                Evaluation::Evaluated {
                    count: 0,
                    block_bytes: 168,
                },
            ),
            (
                2,
                Evaluation::Evaluated {
                    count: 3,
                    block_bytes: 909,
                },
            ),
            (
                2,
                Evaluation::Evaluated {
                    count: 2,
                    block_bytes: 0,
                },
            ),
            (
                2,
                Evaluation::Evaluated {
                    count: 2,
                    block_bytes: 661,
                },
            ),
            (
                2,
                Evaluation::Evaluated {
                    count: 2,
                    block_bytes: MAX_POOL_BATCH_BYTES_V2 + 1,
                },
            ),
            (
                0,
                Evaluation::Blocked {
                    reason: BlockedReason::Ledger,
                    index: None,
                },
            ),
            (
                2,
                Evaluation::Blocked {
                    reason: BlockedReason::InvalidCount,
                    index: None,
                },
            ),
            (
                3,
                Evaluation::Blocked {
                    reason: BlockedReason::Ledger,
                    index: None,
                },
            ),
            (
                2,
                Evaluation::Blocked {
                    reason: BlockedReason::Ledger,
                    index: Some(2),
                },
            ),
            (
                2,
                Evaluation::Blocked {
                    reason: BlockedReason::Body,
                    index: Some(usize::MAX),
                },
            ),
            (
                2,
                Evaluation::Blocked {
                    reason: BlockedReason::WorkLimit,
                    index: Some(0),
                },
            ),
        ] {
            let (identity, mut report) = pool_fixture();
            report.options.evaluate_count = requested;
            report.evaluation = evaluation;
            pool_invariant(identity, &report);
        }
    }

    #[test]
    fn pool_report_and_help_keep_the_existing_output_ceiling() {
        let (identity, mut report) = pool_fixture();
        report.input_bytes = crate::pool_input::MAX_INPUT_BYTES;
        report.input_count = crate::pool_input::MAX_INPUT_COUNT;
        report.retained_count = MAX_POOL_ENTRIES_V2;
        report.duplicate_count = 0;
        report.retained_encoded_bytes = MAX_POOL_ENCODED_BYTES_V2;
        report.work_used = MAX_POOL_WORK_UNITS_V2;
        report.options.evaluate_count = MAX_POOL_BATCH_OPERATIONS_V2;
        report.evaluation = Evaluation::Blocked {
            reason: BlockedReason::Body,
            index: Some(63),
        };
        let text = pool_text(identity, &report);
        let mut envelope = String::new();
        for line in text.lines() {
            let (key, value) = line.split_once('=').unwrap();
            let longest = if key == "evaluation" {
                "not-requested".to_owned()
            } else if key == "evaluation_reason" {
                "batch-byte-limit".to_owned()
            } else if value.len() != 64 && value.bytes().all(|byte| byte.is_ascii_digit()) {
                u64::MAX.to_string()
            } else {
                value.to_owned()
            };
            writeln!(envelope, "{key}={longest}").unwrap();
        }
        assert_eq!(text.lines().count(), 31);
        assert!(text.len() <= envelope.len());
        assert!(envelope.len() < MAX_OUTPUT_BYTES);
        let mut bytes = Vec::new();
        help().unwrap().write_to(&mut bytes).unwrap();
        let help = String::from_utf8(bytes).unwrap();
        assert_eq!(help.len(), 4045);
        assert!(help.len() < MAX_OUTPUT_BYTES);
        assert!(help.contains("pool-inspect,"));
        assert!(help.contains("rejoin-failover-serve,run,help\n"));
        let options = help
            .lines()
            .find_map(|line| line.strip_prefix("pool_options="))
            .unwrap();
        assert_eq!(options.split_whitespace().count(), 18);
        for flag in [
            "--input-file",
            "--max-input-bytes",
            "--max-input-count",
            "--max-entries",
            "--max-encoded-bytes",
            "--max-batch-operations",
            "--max-batch-bytes",
            "--max-work-units",
            "--evaluate-count",
        ] {
            assert!(options.split_whitespace().any(|value| value == flag));
        }
    }
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

    fn run_fixture() -> (ManifestIdentityV2, crate::runner::Report) {
        let identity = ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([0xab; 32]),
            genesis_hash: Hash256::from_bytes([0xcd; 32]),
        };
        let checkpoint = |offset: u8| ReplacementCheckpointV2 {
            generation: 4091 + u64::from(offset),
            chain: ChainReplayExpectationV2 {
                config_hash: identity.config_hash,
                genesis_hash: identity.genesis_hash,
                height: 4091 + u64::from(offset),
                tip_hash: Hash256::from_bytes([offset; 32]),
                ledger_root: Hash256::from_bytes([offset + 10; 32]),
            },
        };
        let rounds = std::array::from_fn(|index| {
            Some(crate::runner::CompletedRound {
                previous: checkpoint(index as u8),
                selected: checkpoint(index as u8 + 1),
                choice: ForkChoiceV2::PreferCandidate,
                acquisition: crate::failover::Acquisition {
                    peer_count: 4,
                    validated_peer: 1,
                    failed_receives: 0,
                    last_receive_error: "none",
                    stop: "validated",
                    results: [
                        "validated",
                        "not-attempted",
                        "not-attempted",
                        "not-attempted",
                    ],
                    stats: SessionStatsV2 {
                        attempts: 1,
                        wire_bytes: 1000,
                    },
                },
                serve_stats: SessionStatsV2 {
                    attempts: 1,
                    wire_bytes: if index == 3 {
                        MAX_SESSION_WIRE_BYTES_V2 - 7000
                    } else {
                        1000
                    },
                },
                completed: u8::from(index % 2 == 1),
                rejected: u8::from(index % 2 == 0),
                service_reason: if index == 3 {
                    "attempt-limit"
                } else {
                    "admission-timeout"
                },
                last_session_error: match index {
                    0 => "peer_io",
                    2 => "peer_wrong_network",
                    _ => "none",
                },
            })
        });
        (
            identity,
            crate::runner::Report {
                initial: checkpoint(0),
                selected: checkpoint(4),
                reason: "attempt-limit",
                trace: crate::runner::RunTrace {
                    round_limit: 4,
                    rounds_started: 4,
                    rounds_completed: 4,
                    peer_count: 4,
                    phase: "report",
                    stats: SessionStatsV2 {
                        attempts: 8,
                        wire_bytes: MAX_SESSION_WIRE_BYTES_V2,
                    },
                    receive: SessionStatsV2 {
                        attempts: 4,
                        wire_bytes: 4000,
                    },
                    failed_receives: 0,
                    last_receive_error: "none",
                    last_completed_service_error: "peer_wrong_network",
                    acquisition: None,
                    adoption_admitted: true,
                },
                completed: 2,
                rejected: 2,
                rounds,
            },
        )
    }

    fn output_text(output: Output) -> String {
        let mut bytes = Vec::new();
        output.write_to(&mut bytes).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    fn assert_run_independent_field_bound(text: &str, expected: usize) {
        // This deliberately unreachable envelope overestimates each field
        // independently: u64 generations, four longest failure labels even in
        // a validated CSV, and maximum usage in every phase. Real validated
        // records are smaller because of their checked cumulative constraints.
        let mut maximum = Output::new();
        for line in text.lines() {
            let (key, value) = line.split_once('=').unwrap();
            let replacement = if key.ends_with("_generation") {
                u64::MAX.to_string()
            } else if key.ends_with("_height") {
                (MAX_CHAIN_BLOCKS_V2 - 1).to_string()
            } else if key.ends_with("wire_bytes") {
                MAX_SESSION_WIRE_BYTES_V2.to_string()
            } else if key.ends_with("_peer_results") {
                ["invalid_session_limits"; 4].join(",")
            } else if key.ends_with("_error") {
                "invalid_session_limits".into()
            } else if key.ends_with("_choice") {
                "prefer-candidate".into()
            } else if key.ends_with("_service_reason") {
                "admission-timeout".into()
            } else if key.ends_with("_acquisition_stop") {
                "peers-exhausted".into()
            } else if key == "reason" {
                "attempt-limit".into()
            } else {
                value.into()
            };
            writeln!(&mut maximum, "{key}={replacement}").unwrap();
        }
        assert_eq!(maximum.length, expected);
        assert!(maximum.length < MAX_OUTPUT_BYTES);
    }

    #[test]
    fn run_four_completed_rounds_fit_fixed_output_and_preserve_each_checkpoint() {
        let (identity, report) = run_fixture();
        let text = output_text(run_report(identity, &report).unwrap());
        assert_eq!(text.lines().count(), 94);
        assert!(text.len() <= 3918 && text.len() < MAX_OUTPUT_BYTES);
        assert_run_independent_field_bound(&text, 3918);
        assert!(text.contains("operation=run\n"));
        assert!(text.contains("initial_height=4091\n"));
        assert!(text.contains("round_1_height=4092\n"));
        assert!(text.contains("round_4_height=4095\n"));
        assert!(text.contains("selected_height=4095\n"));
        assert!(text.contains("wire_bytes=157129488\n"));
        assert!(text.contains("\nlast_session_error=peer_wrong_network\n"));
        assert!(text.contains("round_4_last_session_error=none\n"));
        assert!(text.contains(
            "round_4_peer_results=validated,not-attempted,not-attempted,not-attempted\n"
        ));
        assert!(!text.contains("incomplete_") && !text.contains("127.0.0.1"));
        let keys: std::collections::BTreeSet<_> = text
            .lines()
            .map(|line| line.split_once('=').unwrap().0)
            .collect();
        assert_eq!(keys.len(), text.lines().count());
        let help = output_text(help().unwrap());
        assert!(help.contains("rejoin-failover-serve,run,help\n"));
        assert!(help.contains("run_options=--peers ADDRESS[,ADDRESS...] plus every serve option; --max-rounds UINT --round-wait-ms UINT\n"));
        assert!(help.len() < MAX_OUTPUT_BYTES);
    }

    #[test]
    fn run_incomplete_validated_and_zero_attempt_shutdown_keep_selected_checkpoint() {
        let (identity, mut report) = run_fixture();
        report.rounds[3] = None;
        report.selected = report.rounds[2].unwrap().selected;
        report.reason = "shutdown";
        report.completed = 1;
        report.rejected = 2;
        report.trace.rounds_completed = 3;
        report.trace.receive = SessionStatsV2 {
            attempts: 5,
            wire_bytes: MAX_SESSION_WIRE_BYTES_V2 - 3000,
        };
        report.trace.failed_receives = 1;
        report.trace.last_receive_error = "peer_unreachable";
        report.trace.acquisition = Some(crate::failover::Acquisition {
            peer_count: 4,
            validated_peer: 2,
            failed_receives: 1,
            last_receive_error: "peer_unreachable",
            stop: "validated",
            results: [
                "peer_unreachable",
                "validated",
                "not-attempted",
                "not-attempted",
            ],
            stats: SessionStatsV2 {
                attempts: 2,
                wire_bytes: MAX_SESSION_WIRE_BYTES_V2 - 6000,
            },
        });
        let text = output_text(run_report(identity, &report).unwrap());
        assert_eq!(text.lines().count(), 85);
        assert!(text.len() <= 3529 && text.len() < MAX_OUTPUT_BYTES);
        assert_run_independent_field_bound(&text, 3529);
        assert!(text.contains("rounds_started=4\nrounds_completed=3\n"));
        assert!(text.contains("selected_height=4094\n"));
        assert!(!text.contains("round_4_"));
        assert!(text.contains("incomplete_round=4\n"));
        assert!(text.contains("incomplete_validated_peer=2\n"));
        assert!(text.contains(
            "incomplete_peer_results=peer_unreachable,validated,not-attempted,not-attempted\n"
        ));
        let acquisition = report.trace.acquisition.as_mut().unwrap();
        acquisition.validated_peer = 0;
        acquisition.failed_receives = 0;
        acquisition.last_receive_error = "none";
        acquisition.stop = "shutdown";
        acquisition.results = ["not-attempted"; 4];
        acquisition.stats = SessionStatsV2::default();
        report.trace.stats = SessionStatsV2 {
            attempts: 6,
            wire_bytes: 6000,
        };
        report.trace.receive = SessionStatsV2 {
            attempts: 3,
            wire_bytes: 3000,
        };
        report.trace.failed_receives = 0;
        report.trace.last_receive_error = "none";
        let text = output_text(run_report(identity, &report).unwrap());
        assert_eq!(text.lines().count(), 85);
        assert!(text.contains("incomplete_receive_attempts=0\n"));
        assert!(text.contains("incomplete_acquisition_stop=shutdown\n"));
        report.selected = report.initial;
        report.rounds = [None; 4];
        report.completed = 0;
        report.rejected = 0;
        report.trace.rounds_started = 0;
        report.trace.rounds_completed = 0;
        report.trace.stats = SessionStatsV2::default();
        report.trace.receive = SessionStatsV2::default();
        report.trace.last_completed_service_error = "none";
        report.trace.acquisition = None;
        report.trace.adoption_admitted = false;
        let text = output_text(run_report(identity, &report).unwrap());
        assert_eq!(text.lines().count(), 30);
        assert!(text.contains("selected_height=4091\n"));
        assert!(!text.contains("round_1_") && !text.contains("incomplete_"));
    }

    #[test]
    fn run_normal_records_reject_broken_prefix_totals_labels_and_checkpoint_continuity() {
        let (identity, _) = run_fixture();
        let mut invalid = Vec::new();
        for count in [0, 5] {
            let mut report = run_fixture().1;
            report.trace.round_limit = count;
            invalid.push(report);
            let mut report = run_fixture().1;
            report.trace.peer_count = count;
            invalid.push(report);
        }
        let mut report = run_fixture().1;
        report.rounds[1] = None;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.rounds_completed = 3;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.selected = report.initial;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.rounds[2].as_mut().unwrap().previous = report.initial;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.rounds[1].as_mut().unwrap().choice = ForkChoiceV2::KeepCurrent;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.rounds[1].as_mut().unwrap().selected.generation = 0;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.stats.attempts = 9;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.receive.wire_bytes = MAX_SESSION_WIRE_BYTES_V2;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.last_completed_service_error = "none";
        invalid.push(report);
        let mut report = run_fixture().1;
        report.rounds[0].as_mut().unwrap().rejected = 0;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.rounds[0].as_mut().unwrap().acquisition.results[0] = "secret\nstatus=clean";
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.phase = "wait";
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.adoption_admitted = false;
        invalid.push(report);
        let mut report = run_fixture().1;
        report.initial.chain.config_hash = Hash256::from_bytes([0; 32]);
        invalid.push(report);
        let mut report = run_fixture().1;
        report.reason = "unknown";
        invalid.push(report);
        let mut report = run_fixture().1;
        report.rounds[0].as_mut().unwrap().service_reason = "shutdown";
        invalid.push(report);
        let mut report = run_fixture().1;
        report.trace.acquisition = Some(acquisition());
        invalid.push(report);
        for report in invalid {
            assert!(run_report(identity, &report).is_err());
        }
    }

    fn run_failure_trace() -> crate::runner::RunTrace {
        crate::runner::RunTrace {
            round_limit: 4,
            rounds_started: 2,
            rounds_completed: 1,
            peer_count: 4,
            phase: "acquisition",
            stats: SessionStatsV2 {
                attempts: 4,
                wire_bytes: 2084,
            },
            receive: SessionStatsV2 {
                attempts: 3,
                wire_bytes: 1084,
            },
            failed_receives: 2,
            last_receive_error: "peer_deadline",
            last_completed_service_error: "peer_io",
            acquisition: Some(crate::failover::Acquisition {
                peer_count: 4,
                validated_peer: 0,
                failed_receives: 2,
                last_receive_error: "peer_deadline",
                stop: "attempt-limit",
                results: ["peer_io", "peer_deadline", "not-attempted", "not-attempted"],
                stats: SessionStatsV2 {
                    attempts: 2,
                    wire_bytes: 84,
                },
            }),
            adoption_admitted: true,
        }
    }

    #[test]
    fn run_failure_context_distinguishes_completed_service_from_current_failure() {
        let mut trace = run_failure_trace();
        let mut output =
            diagnostic("run", "peer_deadline", "inspect-status", None, Some(true)).unwrap();
        run_context(&mut output, &trace).unwrap();
        let text = output_text(output);
        assert_eq!(text.lines().count(), 28);
        assert!(text.len() < MAX_OUTPUT_BYTES);
        assert!(text.contains("rounds_started=2\nrounds_completed=1\n"));
        assert!(text.contains("serve_attempts=1\n"));
        assert!(text.contains("last_completed_service_error=peer_io\n"));
        assert!(text.contains("current_round=2\n"));
        assert!(
            text.contains(
                "current_peer_results=peer_io,peer_deadline,not-attempted,not-attempted\n"
            )
        );
        assert!(!text.contains("selected_") && !text.contains("initial_"));
        assert!(!text.contains("completed_sessions=") && !text.contains("rejected_sessions="));
        trace.phase = "adoption-service";
        trace.stats = SessionStatsV2 {
            attempts: 5,
            wire_bytes: 4096,
        };
        trace.failed_receives = 1;
        trace.last_receive_error = "peer_io";
        let acquisition = trace.acquisition.as_mut().unwrap();
        acquisition.validated_peer = 2;
        acquisition.failed_receives = 1;
        acquisition.last_receive_error = "peer_io";
        acquisition.stop = "validated";
        acquisition.results[1] = "validated";
        let mut output = diagnostic(
            "run",
            "allocation_failed",
            "inspect-status",
            None,
            Some(true),
        )
        .unwrap();
        run_context(&mut output, &trace).unwrap();
        let text = output_text(output);
        assert!(text.contains("serve_attempts=2\n"));
        assert!(text.contains("last_completed_service_error=peer_io\n"));
        assert!(!text.contains("completed_sessions=") && !text.contains("rejected_sessions="));
        trace.phase = "wait";
        trace.rounds_started = 1;
        trace.stats = SessionStatsV2 {
            attempts: 2,
            wire_bytes: 2000,
        };
        trace.receive = SessionStatsV2 {
            attempts: 1,
            wire_bytes: 1000,
        };
        trace.failed_receives = 0;
        trace.last_receive_error = "none";
        trace.acquisition = None;
        let mut output =
            diagnostic("run", "lifecycle_limit", "inspect-status", None, Some(true)).unwrap();
        run_context(&mut output, &trace).unwrap();
        let text = output_text(output);
        assert_eq!(text.lines().count(), 20);
        assert!(text.contains("phase=wait\n"));
        assert!(!text.contains("current_"));
        trace.phase = "startup";
        trace.rounds_started = 0;
        trace.rounds_completed = 0;
        trace.stats = SessionStatsV2::default();
        trace.receive = SessionStatsV2::default();
        trace.last_completed_service_error = "none";
        trace.adoption_admitted = false;
        let mut output = diagnostic("run", "busy", "inspect-status", None, None).unwrap();
        run_context(&mut output, &trace).unwrap();
        let text = output_text(output);
        assert_eq!(text.lines().count(), 19);
        assert!(text.contains("phase=startup\n"));
        assert!(!text.contains("current_") && !text.contains("operation_may_have_completed="));
    }

    #[test]
    fn run_invalid_error_context_rejects_before_any_optional_field_is_appended() {
        let valid = run_failure_trace();
        let mut invalid = Vec::new();
        for count in [0, 5] {
            let mut trace = valid;
            trace.round_limit = count;
            invalid.push(trace);
            let mut trace = valid;
            trace.peer_count = count;
            invalid.push(trace);
        }
        let mut trace = valid;
        trace.rounds_completed = 3;
        invalid.push(trace);
        let mut trace = valid;
        trace.adoption_admitted = false;
        invalid.push(trace);
        let mut trace = valid;
        trace.receive.attempts = 5;
        invalid.push(trace);
        let mut trace = valid;
        trace.receive.wire_bytes = 2085;
        invalid.push(trace);
        let mut trace = valid;
        trace.failed_receives = 0;
        invalid.push(trace);
        let mut trace = valid;
        trace.last_completed_service_error = "secret\nselected_height=5";
        invalid.push(trace);
        let mut trace = valid;
        trace.stats = trace.receive;
        invalid.push(trace);
        let mut trace = valid;
        trace.phase = "wait";
        invalid.push(trace);
        let mut trace = valid;
        trace.phase = "unknown";
        invalid.push(trace);
        let mut trace = valid;
        trace.acquisition = None;
        invalid.push(trace);
        let mut trace = valid;
        trace.acquisition.as_mut().unwrap().results[0] = "secret";
        invalid.push(trace);
        let mut trace = valid;
        trace.acquisition.as_mut().unwrap().peer_count = 3;
        invalid.push(trace);
        for trace in invalid {
            let mut output = Output::base("run").unwrap();
            let before = output.length;
            assert!(run_context(&mut output, &trace).is_err());
            assert_eq!(output.length, before);
        }
    }
}
