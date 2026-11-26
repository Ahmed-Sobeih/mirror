//! Bounded, deterministic output. Only static labels and validated numbers/hashes
//! enter these records; paths, raw arguments and lower-layer diagnostics do not.

use std::{fmt, fmt::Write as _, io};

use mirror_chain::v2::ManifestIdentityV2;
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

pub fn help() -> Result<Output, fmt::Error> {
    let mut output = Output::base("help")?;
    for (key, value) in [
        ("commands", "init,status,sync-once,recover,serve,help"),
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
}
