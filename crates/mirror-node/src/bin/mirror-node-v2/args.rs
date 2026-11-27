//! Finite, effect-free command parsing. No environment/configuration fallback.

use std::{ffi::OsString, net::SocketAddr, path::PathBuf, time::Duration};

use mirror_chain::v2::{MAX_CHAIN_BLOCKS_V2, ManifestIdentityV2};
use mirror_crypto::Hash256;
use mirror_network::v2::session::{MAX_SESSION_WIRE_BYTES_V2, SessionLimitsV2};
use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementRecoveryActionV2};

pub const MAX_ARGUMENTS: usize = 32;
pub const MAX_TOKEN_BYTES: usize = 4096;
pub const MAX_ARGUMENT_BYTES: usize = 16_384;
pub(super) const MAX_FAILOVER_PEERS: usize = 4;
pub(super) const MAX_RUN_ROUNDS: usize = 4;

/// Fully parsed ordered endpoints. Unused fixed slots never enter acquisition.
#[derive(Debug, PartialEq, Eq)]
pub struct PeerList {
    addresses: [SocketAddr; MAX_FAILOVER_PEERS],
    length: usize,
}

impl PeerList {
    pub(super) fn as_slice(&self) -> &[SocketAddr] {
        &self.addresses[..self.length]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Common {
    pub control: PathBuf,
    pub store: PathBuf,
    pub identity: ManifestIdentityV2,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Init {
        common: Common,
        expected: ReplacementCheckpointV2,
    },
    Status(Common),
    Sync {
        common: Common,
        peer: SocketAddr,
        limits: SessionLimitsV2,
    },
    Serve {
        common: Common,
        listen: SocketAddr,
        limits: SessionLimitsV2,
        admission: Duration,
    },
    RejoinServe {
        common: Common,
        peer: SocketAddr,
        listen: SocketAddr,
        limits: SessionLimitsV2,
        admission: Duration,
    },
    RejoinFailoverServe {
        common: Common,
        peers: PeerList,
        listen: SocketAddr,
        limits: SessionLimitsV2,
        admission: Duration,
    },
    Run {
        common: Common,
        peers: PeerList,
        listen: SocketAddr,
        limits: SessionLimitsV2,
        admission: Duration,
        max_rounds: u8,
        round_wait: Duration,
    },
    Recover {
        common: Common,
        action: ReplacementRecoveryActionV2,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsageError {
    TooManyArguments,
    TokenTooLong,
    ArgumentsTooLong,
    NonUtf8,
    ControlCharacter,
    MissingCommand,
    UnknownCommand,
    HelpWithArguments,
    UnknownOption,
    UnexpectedArgument,
    WrongCommandOption,
    DuplicateOption,
    MissingOption,
    MissingValue,
    EmptyPath,
    InvalidHash,
    InvalidNumber,
    OutOfRange,
    InvalidPeer,
    InvalidPeerList,
    TooManyPeers,
    DuplicatePeer,
    InvalidListen,
    InvalidAction,
}

impl UsageError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::TooManyArguments => "too_many_arguments",
            Self::TokenTooLong => "token_too_long",
            Self::ArgumentsTooLong => "arguments_too_long",
            Self::NonUtf8 => "non_utf8",
            Self::ControlCharacter => "control_character",
            Self::MissingCommand => "missing_command",
            Self::UnknownCommand => "unknown_command",
            Self::HelpWithArguments => "help_with_arguments",
            Self::UnknownOption => "unknown_option",
            Self::UnexpectedArgument => "unexpected_argument",
            Self::WrongCommandOption => "wrong_command_option",
            Self::DuplicateOption => "duplicate_option",
            Self::MissingOption => "missing_option",
            Self::MissingValue => "missing_value",
            Self::EmptyPath => "empty_path",
            Self::InvalidHash => "invalid_hash",
            Self::InvalidNumber => "invalid_number",
            Self::OutOfRange => "out_of_range",
            Self::InvalidPeer => "invalid_peer",
            Self::InvalidPeerList => "invalid_peer_list",
            Self::TooManyPeers => "too_many_peers",
            Self::DuplicatePeer => "duplicate_peer",
            Self::InvalidListen => "invalid_listen",
            Self::InvalidAction => "invalid_action",
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Init,
    Status,
    Sync,
    Serve,
    RejoinServe,
    RejoinFailoverServe,
    Run,
    Recover,
}

const CONTROL: usize = 0;
const STORE: usize = 1;
const CONFIG: usize = 2;
const GENESIS: usize = 3;
const ATTACH: usize = 4;
const HEIGHT: usize = 5;
const TIP: usize = 6;
const ROOT: usize = 7;
const GENERATION: usize = 8;
const PEER: usize = 9;
const TIMEOUT: usize = 10;
const IO_BYTES: usize = 11;
const ACTION: usize = 12;
const LISTEN: usize = 13;
const ATTEMPTS: usize = 14;
const ADMISSION: usize = 15;
const PEERS: usize = 16;
const ROUNDS: usize = 17;
const ROUND_WAIT: usize = 18;
const COMMON_MASK: u32 = (1 << CONTROL) | (1 << STORE) | (1 << CONFIG) | (1 << GENESIS);

fn required(kind: Kind) -> u32 {
    COMMON_MASK
        | match kind {
            Kind::Init => {
                (1 << ATTACH) | (1 << HEIGHT) | (1 << TIP) | (1 << ROOT) | (1 << GENERATION)
            }
            Kind::Status => 0,
            Kind::Sync => (1 << PEER) | (1 << TIMEOUT) | (1 << IO_BYTES),
            Kind::Serve => {
                (1 << LISTEN)
                    | (1 << ATTEMPTS)
                    | (1 << IO_BYTES)
                    | (1 << TIMEOUT)
                    | (1 << ADMISSION)
            }
            Kind::RejoinServe => required(Kind::Serve) | (1 << PEER),
            Kind::RejoinFailoverServe => required(Kind::Serve) | (1 << PEERS),
            Kind::Run => required(Kind::RejoinFailoverServe) | (1 << ROUNDS) | (1 << ROUND_WAIT),
            Kind::Recover => 1 << ACTION,
        }
}

fn flag(value: &str) -> Result<usize, UsageError> {
    Ok(match value {
        "--control-dir" => CONTROL,
        "--store-dir" => STORE,
        "--config-hash" => CONFIG,
        "--genesis-hash" => GENESIS,
        "--attach-existing" => ATTACH,
        "--height" => HEIGHT,
        "--tip-hash" => TIP,
        "--ledger-root" => ROOT,
        "--generation" => GENERATION,
        "--peer" => PEER,
        "--peers" => PEERS,
        "--max-rounds" => ROUNDS,
        "--round-wait-ms" => ROUND_WAIT,
        "--timeout-ms" => TIMEOUT,
        "--max-io-bytes" => IO_BYTES,
        "--action" => ACTION,
        "--listen" => LISTEN,
        "--max-attempts" => ATTEMPTS,
        "--admission-ms" => ADMISSION,
        value if value.starts_with('-') => return Err(UsageError::UnknownOption),
        _ => return Err(UsageError::UnexpectedArgument),
    })
}

/// The count excludes argv[0]. Admission consumes at most one excess token.
/// Only bounded UTF-8 tokens are retained; no path lookup or socket operation occurs.
pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let mut tokens = Vec::new();
    let mut bytes = 0usize;
    for argument in arguments {
        if tokens.len() == MAX_ARGUMENTS {
            return Err(UsageError::TooManyArguments);
        }
        let length = argument.as_os_str().as_encoded_bytes().len();
        if length > MAX_TOKEN_BYTES {
            return Err(UsageError::TokenTooLong);
        }
        bytes = bytes
            .checked_add(length)
            .ok_or(UsageError::ArgumentsTooLong)?;
        if bytes > MAX_ARGUMENT_BYTES {
            return Err(UsageError::ArgumentsTooLong);
        }
        let argument = argument.into_string().map_err(|_| UsageError::NonUtf8)?;
        if argument.chars().any(forbidden_character) {
            return Err(UsageError::ControlCharacter);
        }
        tokens.push(argument);
    }
    let first = tokens.first().ok_or(UsageError::MissingCommand)?;
    if first == "help" || first == "--help" {
        return if tokens.len() == 1 {
            Ok(Command::Help)
        } else {
            Err(UsageError::HelpWithArguments)
        };
    }
    let kind = match first.as_str() {
        "init" => Kind::Init,
        "status" => Kind::Status,
        "sync-once" => Kind::Sync,
        "serve" => Kind::Serve,
        "rejoin-serve" => Kind::RejoinServe,
        "rejoin-failover-serve" => Kind::RejoinFailoverServe,
        "run" => Kind::Run,
        "recover" => Kind::Recover,
        _ => return Err(UsageError::UnknownCommand),
    };
    let required = required(kind);
    let mut seen = 0u32;
    let mut values: [Option<&str>; 19] = [None; 19];
    let mut position = 1usize;
    while position < tokens.len() {
        let index = flag(&tokens[position])?;
        let bit = 1u32 << index;
        if required & bit == 0 {
            return Err(UsageError::WrongCommandOption);
        }
        if seen & bit != 0 {
            return Err(UsageError::DuplicateOption);
        }
        seen |= bit;
        if index == ATTACH {
            position += 1;
            continue;
        }
        let value = tokens.get(position + 1).ok_or(UsageError::MissingValue)?;
        if value.starts_with("--") {
            return Err(UsageError::MissingValue);
        }
        values[index] = Some(value);
        position += 2;
    }
    if seen != required {
        return Err(UsageError::MissingOption);
    }
    let value = |index: usize| values[index].ok_or(UsageError::MissingValue);
    let path = |index: usize| -> Result<PathBuf, UsageError> {
        let text = value(index)?;
        if text.is_empty() {
            return Err(UsageError::EmptyPath);
        }
        Ok(PathBuf::from(text))
    };
    let identity = ManifestIdentityV2 {
        config_hash: hash(value(CONFIG)?)?,
        genesis_hash: hash(value(GENESIS)?)?,
    };
    let common = Common {
        control: path(CONTROL)?,
        store: path(STORE)?,
        identity,
    };
    Ok(match kind {
        Kind::Status => Command::Status(common),
        Kind::Init => {
            let height = unsigned(value(HEIGHT)?)?;
            let generation = unsigned(value(GENERATION)?)?;
            if height >= MAX_CHAIN_BLOCKS_V2 as u64 || generation > height {
                return Err(UsageError::OutOfRange);
            }
            Command::Init {
                common,
                expected: ReplacementCheckpointV2 {
                    generation,
                    chain: mirror_chain::v2::ChainReplayExpectationV2 {
                        config_hash: identity.config_hash,
                        genesis_hash: identity.genesis_hash,
                        height,
                        tip_hash: hash(value(TIP)?)?,
                        ledger_root: hash(value(ROOT)?)?,
                    },
                },
            }
        }
        Kind::Sync => {
            let peer: SocketAddr = value(PEER)?.parse().map_err(|_| UsageError::InvalidPeer)?;
            if peer.port() == 0 {
                return Err(UsageError::InvalidPeer);
            }
            let timeout = unsigned(value(TIMEOUT)?)?;
            let allowance = unsigned(value(IO_BYTES)?)?;
            if timeout == 0
                || timeout > 60_000
                || allowance == 0
                || allowance > MAX_SESSION_WIRE_BYTES_V2
            {
                return Err(UsageError::OutOfRange);
            }
            let limits = SessionLimitsV2::new(1, allowance, Duration::from_millis(timeout))
                .map_err(|_| UsageError::OutOfRange)?;
            Command::Sync {
                common,
                peer,
                limits,
            }
        }
        Kind::Serve | Kind::RejoinServe | Kind::RejoinFailoverServe | Kind::Run => {
            let listen: SocketAddr = value(LISTEN)?
                .parse()
                .map_err(|_| UsageError::InvalidListen)?;
            if listen.port() == 0 {
                return Err(UsageError::InvalidListen);
            }
            let attempts =
                u8::try_from(unsigned(value(ATTEMPTS)?)?).map_err(|_| UsageError::OutOfRange)?;
            let allowance = unsigned(value(IO_BYTES)?)?;
            let timeout = unsigned(value(TIMEOUT)?)?;
            let admission = unsigned(value(ADMISSION)?)?;
            if timeout == 0 || timeout > 60_000 || admission == 0 || admission > 60_000 {
                return Err(UsageError::OutOfRange);
            }
            let limits = SessionLimitsV2::new(attempts, allowance, Duration::from_millis(timeout))
                .map_err(|_| UsageError::OutOfRange)?;
            let admission = Duration::from_millis(admission);
            if matches!(kind, Kind::Run) {
                let max_rounds =
                    u8::try_from(unsigned(value(ROUNDS)?)?).map_err(|_| UsageError::OutOfRange)?;
                let round_wait = unsigned(value(ROUND_WAIT)?)?;
                if max_rounds == 0
                    || usize::from(max_rounds) > MAX_RUN_ROUNDS
                    || round_wait > 60_000
                {
                    return Err(UsageError::OutOfRange);
                }
                Command::Run {
                    common,
                    peers: peer_list(value(PEERS)?)?,
                    listen,
                    limits,
                    admission,
                    max_rounds,
                    round_wait: Duration::from_millis(round_wait),
                }
            } else if matches!(kind, Kind::RejoinFailoverServe) {
                Command::RejoinFailoverServe {
                    common,
                    peers: peer_list(value(PEERS)?)?,
                    listen,
                    limits,
                    admission,
                }
            } else if matches!(kind, Kind::RejoinServe) {
                let peer: SocketAddr = value(PEER)?.parse().map_err(|_| UsageError::InvalidPeer)?;
                if peer.port() == 0 {
                    return Err(UsageError::InvalidPeer);
                }
                Command::RejoinServe {
                    common,
                    peer,
                    listen,
                    limits,
                    admission,
                }
            } else {
                Command::Serve {
                    common,
                    listen,
                    limits,
                    admission,
                }
            }
        }
        Kind::Recover => {
            let action = match value(ACTION)? {
                "complete" => ReplacementRecoveryActionV2::Complete,
                "retain-old" => ReplacementRecoveryActionV2::RetainOld,
                _ => return Err(UsageError::InvalidAction),
            };
            Command::Recover { common, action }
        }
    })
}

fn peer_list(text: &str) -> Result<PeerList, UsageError> {
    if text.is_empty() || text.chars().any(char::is_whitespace) || text.contains('%') {
        return Err(UsageError::InvalidPeerList);
    }
    let mut peers = PeerList {
        addresses: [SocketAddr::from(([0, 0, 0, 0], 0)); MAX_FAILOVER_PEERS],
        length: 0,
    };
    for value in text.split(',') {
        if value.is_empty() {
            return Err(UsageError::InvalidPeerList);
        }
        if peers.length == MAX_FAILOVER_PEERS {
            return Err(UsageError::TooManyPeers);
        }
        let address: SocketAddr = value.parse().map_err(|_| UsageError::InvalidPeer)?;
        if address.port() == 0 || matches!(address, SocketAddr::V6(value) if value.scope_id() != 0)
        {
            return Err(UsageError::InvalidPeer);
        }
        // Equality is numeric, not spelling equality. Mapped-v4 normalization
        // is used only for duplicate detection, never to change the request.
        if peers
            .as_slice()
            .iter()
            .any(|previous| duplicate_key(*previous) == duplicate_key(address))
        {
            return Err(UsageError::DuplicatePeer);
        }
        peers.addresses[peers.length] = address;
        peers.length += 1;
    }
    Ok(peers)
}

fn duplicate_key(address: SocketAddr) -> SocketAddr {
    if let SocketAddr::V6(value) = address {
        if let Some(ipv4) = value.ip().to_ipv4_mapped() {
            return SocketAddr::from((ipv4, value.port()));
        }
    }
    address
}

fn forbidden_character(character: char) -> bool {
    character.is_control()
        || matches!(character,
        '\u{2028}' | '\u{2029}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn unsigned(text: &str) -> Result<u64, UsageError> {
    if text.is_empty()
        || !text.bytes().all(|byte| byte.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return Err(UsageError::InvalidNumber);
    }
    text.parse().map_err(|_| UsageError::InvalidNumber)
}

/// Both ASCII hex cases are accepted; Hash256 output is canonical lowercase.
fn hash(text: &str) -> Result<Hash256, UsageError> {
    if text.len() != 64 {
        return Err(UsageError::InvalidHash);
    }
    fn nibble(byte: u8) -> Result<u8, UsageError> {
        match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            b'A'..=b'F' => Ok(byte - b'A' + 10),
            _ => Err(UsageError::InvalidHash),
        }
    }
    let mut bytes = [0u8; 32];
    for (index, pair) in text.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = nibble(pair[0])? * 16 + nibble(pair[1])?;
    }
    Ok(Hash256::from_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn common(command: &str) -> Vec<String> {
        vec![
            command.into(),
            "--control-dir".into(),
            "control".into(),
            "--store-dir".into(),
            "store".into(),
            "--config-hash".into(),
            "ab".repeat(32),
            "--genesis-hash".into(),
            "cd".repeat(32),
        ]
    }
    fn init() -> Vec<String> {
        let mut values = common("init");
        values.extend([
            "--attach-existing".into(),
            "--height".into(),
            "3".into(),
            "--tip-hash".into(),
            "ef".repeat(32),
            "--ledger-root".into(),
            "12".repeat(32),
            "--generation".into(),
            "0".into(),
        ]);
        values
    }
    fn sync() -> Vec<String> {
        let mut values = common("sync-once");
        values.extend([
            "--peer".into(),
            "127.0.0.1:9000".into(),
            "--timeout-ms".into(),
            "1000".into(),
            "--max-io-bytes".into(),
            "20000000".into(),
        ]);
        values
    }
    fn recover() -> Vec<String> {
        let mut values = common("recover");
        values.extend(["--action".into(), "complete".into()]);
        values
    }
    fn serve() -> Vec<String> {
        let mut values = common("serve");
        values.extend([
            "--listen".into(),
            "127.0.0.1:9000".into(),
            "--max-attempts".into(),
            "2".into(),
            "--max-io-bytes".into(),
            "20000000".into(),
            "--timeout-ms".into(),
            "1000".into(),
            "--admission-ms".into(),
            "3000".into(),
        ]);
        values
    }
    fn rejoin() -> Vec<String> {
        let mut values = serve();
        values[0] = "rejoin-serve".into();
        values.extend(["--peer".into(), "127.0.0.1:9001".into()]);
        values
    }
    fn failover() -> Vec<String> {
        let mut values = serve();
        values[0] = "rejoin-failover-serve".into();
        values.extend(["--peers".into(), "127.0.0.1:9001,[::1]:9002".into()]);
        values
    }
    fn parsed(values: Vec<String>) -> Result<Command, UsageError> {
        parse(values.into_iter().map(OsString::from))
    }
    fn change(values: &mut [String], flag: &str, value: &str) {
        let index = values.iter().position(|entry| entry == flag).unwrap();
        values[index + 1] = value.into();
    }

    #[test]
    fn help_is_deliberate_and_standalone() {
        for value in ["help", "--help"] {
            assert_eq!(parsed(vec![value.into()]), Ok(Command::Help));
        }
        assert_eq!(parsed(vec![]), Err(UsageError::MissingCommand));
        assert_eq!(
            parsed(vec!["help".into(), "status".into()]),
            Err(UsageError::HelpWithArguments)
        );
        assert_eq!(
            parsed(vec!["--help".into(), "--control-dir".into(), "x".into()]),
            Err(UsageError::HelpWithArguments)
        );
    }

    #[test]
    fn command_names_have_no_fallback_or_abbreviation() {
        for value in ["", "INIT", "sync", "serve-once", "version", "-h", "status "] {
            assert_eq!(parsed(vec![value.into()]), Err(UsageError::UnknownCommand));
        }
    }

    #[test]
    fn serve_requires_every_explicit_input_and_rejects_duplicate_options() {
        let full = serve();
        for start in (1..full.len()).step_by(2) {
            let mut missing = full.clone();
            missing.drain(start..start + 2);
            assert_eq!(parsed(missing), Err(UsageError::MissingOption));
            let mut duplicate = full.clone();
            duplicate.extend([full[start].clone(), full[start + 1].clone()]);
            assert_eq!(parsed(duplicate), Err(UsageError::DuplicateOption));
        }
    }

    #[test]
    fn serve_preserves_independent_identity_and_all_budget_boundaries() {
        for (address, attempts, allowance, timeout, admission) in [
            ("127.0.0.1:1", 1, 1, 1, 1),
            ("[::1]:65535", 8, MAX_SESSION_WIRE_BYTES_V2, 60_000, 60_000),
        ] {
            let mut values = serve();
            for (flag, value) in [
                ("--listen", address.to_string()),
                ("--max-attempts", attempts.to_string()),
                ("--max-io-bytes", allowance.to_string()),
                ("--timeout-ms", timeout.to_string()),
                ("--admission-ms", admission.to_string()),
            ] {
                change(&mut values, flag, &value);
            }
            let Command::Serve {
                common,
                listen,
                limits,
                admission: window,
            } = parsed(values).unwrap()
            else {
                panic!("wrong command");
            };
            assert_eq!(common.identity.config_hash.to_hex(), "ab".repeat(32));
            assert_eq!(common.identity.genesis_hash.to_hex(), "cd".repeat(32));
            assert_eq!(listen, address.parse::<SocketAddr>().unwrap());
            assert_eq!(limits.max_attempts(), attempts);
            assert_eq!(limits.max_wire_bytes(), allowance);
            assert_eq!(limits.timeout(), Duration::from_millis(timeout));
            assert_eq!(window, Duration::from_millis(admission));
        }
    }

    #[test]
    fn serve_rejects_zero_and_excessive_resource_limits_before_effects() {
        for (flag, value) in [
            ("--max-attempts", "0"),
            ("--max-attempts", "9"),
            ("--max-attempts", "256"),
            ("--max-attempts", "18446744073709551615"),
            ("--max-io-bytes", "0"),
            ("--max-io-bytes", "157129489"),
            ("--timeout-ms", "0"),
            ("--timeout-ms", "60001"),
            ("--admission-ms", "0"),
            ("--admission-ms", "60001"),
        ] {
            let mut values = serve();
            change(&mut values, flag, value);
            assert_eq!(parsed(values), Err(UsageError::OutOfRange));
        }
    }

    #[test]
    fn serve_requires_canonical_unsigned_decimal_for_every_limit() {
        for flag in [
            "--max-attempts",
            "--max-io-bytes",
            "--timeout-ms",
            "--admission-ms",
        ] {
            for value in ["", "01", "+1", "-1", "1.0", " 1", "18446744073709551616"] {
                let mut values = serve();
                change(&mut values, flag, value);
                assert_eq!(parsed(values), Err(UsageError::InvalidNumber));
            }
        }
    }

    #[test]
    fn serve_requires_numeric_nonzero_listen_address() {
        for address in [
            "localhost:9000",
            "127.0.0.1:0",
            "[::1]:0",
            "127.0.0.1:65536",
            "127.0.0.1",
            "",
        ] {
            let mut values = serve();
            change(&mut values, "--listen", address);
            assert_eq!(parsed(values), Err(UsageError::InvalidListen));
        }
    }

    #[test]
    fn serve_options_do_not_leak_into_other_commands_or_accept_receiver_flags() {
        for mut values in [common("status"), init(), sync(), recover()] {
            values.extend(["--listen".into(), "127.0.0.1:9000".into()]);
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
        for (flag, value) in [("--peer", "127.0.0.1:9000"), ("--action", "complete")] {
            let mut values = serve();
            values.extend([flag.into(), value.into()]);
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
    }

    #[test]
    fn rejoin_requires_every_explicit_input_and_rejects_duplicates() {
        let full = rejoin();
        assert_eq!(full.len(), 21);
        for start in (1..full.len()).step_by(2) {
            let mut missing = full.clone();
            missing.drain(start..start + 2);
            assert_eq!(parsed(missing), Err(UsageError::MissingOption));
            let mut duplicate = full.clone();
            duplicate.extend([full[start].clone(), full[start + 1].clone()]);
            assert_eq!(parsed(duplicate), Err(UsageError::DuplicateOption));
        }
    }

    #[test]
    fn rejoin_preserves_identity_and_all_limits_including_one_attempt() {
        for (address, attempts, allowance, timeout, admission) in [
            ("127.0.0.1:1", 1, 1, 1, 1),
            ("[::1]:65535", 8, MAX_SESSION_WIRE_BYTES_V2, 60_000, 60_000),
        ] {
            let mut values = rejoin();
            for (flag, value) in [
                ("--peer", address.to_string()),
                ("--listen", address.to_string()),
                ("--max-attempts", attempts.to_string()),
                ("--max-io-bytes", allowance.to_string()),
                ("--timeout-ms", timeout.to_string()),
                ("--admission-ms", admission.to_string()),
            ] {
                change(&mut values, flag, &value);
            }
            let Command::RejoinServe {
                common,
                peer,
                listen,
                limits,
                admission: window,
            } = parsed(values).unwrap()
            else {
                panic!("wrong command");
            };
            assert_eq!(common.identity.config_hash.to_hex(), "ab".repeat(32));
            assert_eq!(common.identity.genesis_hash.to_hex(), "cd".repeat(32));
            assert_eq!(peer, address.parse::<SocketAddr>().unwrap());
            assert_eq!(listen, peer);
            assert_eq!(limits.max_attempts(), attempts);
            assert_eq!(limits.max_wire_bytes(), allowance);
            assert_eq!(limits.timeout(), Duration::from_millis(timeout));
            assert_eq!(window, Duration::from_millis(admission));
        }
    }

    #[test]
    fn rejoin_rejects_zero_and_excessive_limits_before_effects() {
        for (flag, value) in [
            ("--max-attempts", "0"),
            ("--max-attempts", "9"),
            ("--max-attempts", "256"),
            ("--max-io-bytes", "0"),
            ("--max-io-bytes", "157129489"),
            ("--timeout-ms", "0"),
            ("--timeout-ms", "60001"),
            ("--admission-ms", "0"),
            ("--admission-ms", "60001"),
        ] {
            let mut values = rejoin();
            change(&mut values, flag, value);
            assert_eq!(parsed(values), Err(UsageError::OutOfRange));
        }
    }

    #[test]
    fn rejoin_rejects_noncanonical_limits() {
        for flag in [
            "--max-attempts",
            "--max-io-bytes",
            "--timeout-ms",
            "--admission-ms",
        ] {
            for value in ["", "01", "+1", "-1", "1.0", " 1", "18446744073709551616"] {
                let mut values = rejoin();
                change(&mut values, flag, value);
                assert_eq!(parsed(values), Err(UsageError::InvalidNumber));
            }
        }
    }

    #[test]
    fn rejoin_requires_both_numeric_nonzero_addresses() {
        for (flag, error) in [
            ("--peer", UsageError::InvalidPeer),
            ("--listen", UsageError::InvalidListen),
        ] {
            for address in [
                "localhost:9000",
                "127.0.0.1:0",
                "[::1]:0",
                "127.0.0.1:65536",
                "127.0.0.1",
                "",
            ] {
                let mut values = rejoin();
                change(&mut values, flag, address);
                assert_eq!(parsed(values), Err(error));
            }
        }
    }

    #[test]
    fn rejoin_does_not_accept_initialization_or_recovery_options() {
        for (flag, value) in [
            ("--action", "complete"),
            ("--generation", "0"),
            ("--height", "0"),
        ] {
            let mut values = rejoin();
            values.extend([flag.into(), value.into()]);
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
        let mut values = rejoin();
        values.push("--attach-existing".into());
        assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
    }

    #[test]
    fn failover_requires_all_ten_options_and_rejects_duplicate_flags() {
        let full = failover();
        assert_eq!(full.len(), 21);
        for start in (1..full.len()).step_by(2) {
            let mut missing = full.clone();
            missing.drain(start..start + 2);
            assert_eq!(parsed(missing), Err(UsageError::MissingOption));
            let mut duplicate = full.clone();
            duplicate.extend([full[start].clone(), full[start + 1].clone()]);
            assert_eq!(parsed(duplicate), Err(UsageError::DuplicateOption));
        }
    }

    #[test]
    fn failover_preserves_order_parsed_addresses_identity_and_shared_bounds() {
        for (list, attempts, allowance, timeout, admission) in [
            ("127.0.0.1:1", 1, 1, 1, 1),
            (
                "[::ffff:192.0.2.1]:65535,[2001:db8::1]:1,127.0.0.1:7,[::1]:9",
                8,
                MAX_SESSION_WIRE_BYTES_V2,
                60_000,
                60_000,
            ),
        ] {
            let mut values = failover();
            for (flag, value) in [
                ("--peers", list.to_string()),
                ("--max-attempts", attempts.to_string()),
                ("--max-io-bytes", allowance.to_string()),
                ("--timeout-ms", timeout.to_string()),
                ("--admission-ms", admission.to_string()),
            ] {
                change(&mut values, flag, &value);
            }
            let Command::RejoinFailoverServe {
                common,
                peers,
                limits,
                admission: window,
                ..
            } = parsed(values).unwrap()
            else {
                panic!("wrong command");
            };
            assert_eq!(common.identity.config_hash.to_hex(), "ab".repeat(32));
            assert_eq!(common.identity.genesis_hash.to_hex(), "cd".repeat(32));
            let expected: Vec<SocketAddr> = list.split(',').map(|x| x.parse().unwrap()).collect();
            assert_eq!(peers.as_slice(), expected.as_slice());
            assert_eq!(limits.max_attempts(), attempts);
            assert_eq!(limits.max_wire_bytes(), allowance);
            assert_eq!(limits.timeout(), Duration::from_millis(timeout));
            assert_eq!(window, Duration::from_millis(admission));
        }
        // The list need not fit the total attempt allowance; later entries may
        // remain unattempted. Reordering flags must not reorder peers.
        let mut first = failover();
        change(&mut first, "--max-attempts", "1");
        let mut second = vec![first[0].clone()];
        for pair in first[1..].chunks_exact(2).rev() {
            second.extend_from_slice(pair);
        }
        assert_eq!(parsed(first), parsed(second));
    }

    #[test]
    fn failover_rejects_empty_oversized_scoped_and_malformed_peer_lists() {
        for list in [
            "",
            ",127.0.0.1:1",
            "127.0.0.1:1,",
            "127.0.0.1:1,,127.0.0.1:2",
            "127.0.0.1:1, 127.0.0.1:2",
            "127.0.0.1:1,\u{a0}127.0.0.1:2",
            "[fe80::1%3]:1",
            "[fe80::1%0]:1",
            "[fe80::1%eth0]:1",
        ] {
            let mut values = failover();
            change(&mut values, "--peers", list);
            assert_eq!(parsed(values), Err(UsageError::InvalidPeerList), "{list}");
        }
        for list in [
            "localhost:1",
            "127.0.0.1:0",
            "[::1]:0",
            "127.0.0.1:65536",
            "127.0.0.1",
            "::1:1",
            "127.0.0.1:1;127.0.0.1:2",
        ] {
            let mut values = failover();
            change(&mut values, "--peers", list);
            assert_eq!(parsed(values), Err(UsageError::InvalidPeer), "{list}");
        }
        let mut values = failover();
        change(
            &mut values,
            "--peers",
            "127.0.0.1:1,127.0.0.1:2,127.0.0.1:3,127.0.0.1:4,127.0.0.1:5",
        );
        assert_eq!(parsed(values), Err(UsageError::TooManyPeers));
    }

    #[test]
    fn failover_rejects_numeric_aliases_but_preserves_distinct_endpoints() {
        for list in [
            "127.0.0.1:1,127.0.0.1:1",
            "[::1]:9,[0:0:0:0:0:0:0:1]:9",
            "[2001:DB8::1]:9,[2001:0db8:0:0:0:0:0:1]:9",
            "192.0.2.1:9,[::ffff:192.0.2.1]:9",
            "[::ffff:c000:201]:9,192.0.2.1:9",
            "[::ffff:c000:201]:9,[::ffff:192.0.2.1]:9",
        ] {
            let mut values = failover();
            change(&mut values, "--peers", list);
            assert_eq!(parsed(values), Err(UsageError::DuplicatePeer), "{list}");
        }
        for list in [
            "127.0.0.1:1,127.0.0.1:2",
            "192.0.2.1:1,[::ffff:192.0.2.1]:2",
            "192.0.2.1:1,[::192.0.2.1]:1",
            "[2001:db8::1]:1,[2001:db8::2]:1",
        ] {
            let mut values = failover();
            change(&mut values, "--peers", list);
            assert!(parsed(values).is_ok(), "{list}");
        }
    }

    #[test]
    fn failover_list_flag_never_changes_existing_command_option_rules() {
        for mut values in [
            common("status"),
            init(),
            sync(),
            serve(),
            rejoin(),
            recover(),
        ] {
            values.extend(["--peers".into(), "127.0.0.1:1".into()]);
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
        for (flag, value) in [
            ("--peer", "127.0.0.1:1"),
            ("--action", "complete"),
            ("--generation", "0"),
            ("--height", "0"),
        ] {
            let mut values = failover();
            values.extend([flag.into(), value.into()]);
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
        let mut values = failover();
        values.push("--attach-existing".into());
        assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
    }

    #[test]
    fn failover_preserves_all_numeric_and_argument_admission_limits() {
        for flag in [
            "--max-attempts",
            "--max-io-bytes",
            "--timeout-ms",
            "--admission-ms",
        ] {
            for value in ["", "01", "+1", "-1", "1.0", " 1", "18446744073709551616"] {
                let mut values = failover();
                change(&mut values, flag, value);
                assert_eq!(parsed(values), Err(UsageError::InvalidNumber));
            }
        }
        for (flag, value) in [
            ("--max-attempts", "0"),
            ("--max-attempts", "9"),
            ("--max-attempts", "256"),
            ("--max-io-bytes", "0"),
            ("--max-io-bytes", "157129489"),
            ("--timeout-ms", "0"),
            ("--timeout-ms", "60001"),
            ("--admission-ms", "0"),
            ("--admission-ms", "60001"),
        ] {
            let mut values = failover();
            change(&mut values, flag, value);
            assert_eq!(parsed(values), Err(UsageError::OutOfRange));
        }
        let mut values = failover();
        change(&mut values, "--peers", &"x".repeat(MAX_TOKEN_BYTES + 1));
        assert_eq!(parsed(values), Err(UsageError::TokenTooLong));
        let mut values = failover();
        values.resize(MAX_ARGUMENTS + 1, "extra".into());
        assert_eq!(parsed(values), Err(UsageError::TooManyArguments));
        let mut values = failover();
        change(&mut values, "--peers", "127.0.0.1:1,\n127.0.0.1:2");
        assert_eq!(parsed(values), Err(UsageError::ControlCharacter));
    }

    #[test]
    fn status_requires_every_common_input() {
        let full = common("status");
        assert!(matches!(parsed(full.clone()), Ok(Command::Status(_))));
        for start in [1, 3, 5, 7] {
            let mut values = full.clone();
            values.drain(start..start + 2);
            assert_eq!(parsed(values), Err(UsageError::MissingOption));
        }
    }

    #[test]
    fn init_requires_explicit_attach_and_all_full_checkpoint_flags() {
        let full = init();
        assert!(matches!(parsed(full.clone()), Ok(Command::Init { .. })));
        for option in [
            "--attach-existing",
            "--height",
            "--tip-hash",
            "--ledger-root",
            "--generation",
        ] {
            let mut values = full.clone();
            let index = values.iter().position(|value| value == option).unwrap();
            values.drain(index..index + if option == "--attach-existing" { 1 } else { 2 });
            assert_eq!(parsed(values), Err(UsageError::MissingOption));
        }
    }

    #[test]
    fn init_checkpoint_bounds_are_checked_before_any_action() {
        for (height, generation, accepted) in [
            ("0", "0", true),
            ("4095", "4095", true),
            ("4096", "0", false),
            ("3", "4", false),
        ] {
            let mut values = init();
            change(&mut values, "--height", height);
            change(&mut values, "--generation", generation);
            if accepted {
                assert!(parsed(values).is_ok());
            } else {
                assert_eq!(parsed(values), Err(UsageError::OutOfRange));
            }
        }
    }

    #[test]
    fn init_network_and_checkpoint_values_are_preserved() {
        let Command::Init { common, expected } = parsed(init()).unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(expected.chain.config_hash, common.identity.config_hash);
        assert_eq!(expected.chain.genesis_hash, common.identity.genesis_hash);
        assert_eq!(expected.chain.height, 3);
        assert_eq!(expected.generation, 0);
        assert_eq!(expected.chain.tip_hash.to_hex(), "ef".repeat(32));
        assert_eq!(expected.chain.ledger_root.to_hex(), "12".repeat(32));
    }

    #[test]
    fn sync_requires_all_three_deliberate_session_options() {
        let full = sync();
        for option in ["--peer", "--timeout-ms", "--max-io-bytes"] {
            let mut values = full.clone();
            let index = values.iter().position(|value| value == option).unwrap();
            values.drain(index..index + 2);
            assert_eq!(parsed(values), Err(UsageError::MissingOption));
        }
    }

    #[test]
    fn sync_admits_one_attempt_and_only_f2_limit_ranges() {
        for (timeout, allowance) in [(1, 1), (60_000, MAX_SESSION_WIRE_BYTES_V2)] {
            let mut values = sync();
            change(&mut values, "--timeout-ms", &timeout.to_string());
            change(&mut values, "--max-io-bytes", &allowance.to_string());
            let Command::Sync { limits, .. } = parsed(values).unwrap() else {
                panic!("wrong command")
            };
            assert_eq!(limits.max_attempts(), 1);
            assert_eq!(limits.timeout(), Duration::from_millis(timeout));
            assert_eq!(limits.max_wire_bytes(), allowance);
        }
        for (option, value) in [
            ("--timeout-ms", "0"),
            ("--timeout-ms", "60001"),
            ("--max-io-bytes", "0"),
            ("--max-io-bytes", "157129489"),
        ] {
            let mut values = sync();
            change(&mut values, option, value);
            assert_eq!(parsed(values), Err(UsageError::OutOfRange));
        }
    }

    #[test]
    fn sync_peer_is_numeric_ipv4_or_bracketed_ipv6_with_nonzero_port() {
        for address in ["127.0.0.1:1", "192.0.2.1:65535", "[::1]:9000"] {
            let mut values = sync();
            change(&mut values, "--peer", address);
            assert!(parsed(values).is_ok());
        }
        for address in [
            "localhost:9000",
            "example.com:9000",
            "127.0.0.1:0",
            "[::1]:0",
            "127.0.0.1",
            "::1:9000",
            "127.0.0.1:65536",
            "",
        ] {
            let mut values = sync();
            change(&mut values, "--peer", address);
            assert_eq!(parsed(values), Err(UsageError::InvalidPeer));
        }
    }

    #[test]
    fn recovery_action_is_required_and_exact() {
        assert!(matches!(
            parsed(recover()),
            Ok(Command::Recover {
                action: ReplacementRecoveryActionV2::Complete,
                ..
            })
        ));
        let mut values = recover();
        change(&mut values, "--action", "retain-old");
        assert!(matches!(
            parsed(values),
            Ok(Command::Recover {
                action: ReplacementRecoveryActionV2::RetainOld,
                ..
            })
        ));
        for action in ["", "retain", "retain_old", "Complete", "auto"] {
            let mut values = recover();
            change(&mut values, "--action", action);
            assert_eq!(parsed(values), Err(UsageError::InvalidAction));
        }
        assert_eq!(parsed(common("recover")), Err(UsageError::MissingOption));
    }

    #[test]
    fn hashes_accept_ascii_case_and_normalize_output() {
        let mut values = common("status");
        change(&mut values, "--config-hash", &"aB".repeat(32));
        let Command::Status(common) = parsed(values).unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(common.identity.config_hash.to_hex(), "ab".repeat(32));
    }

    #[test]
    fn every_hash_field_rejects_length_prefix_nonhex_and_unicode() {
        for option in [
            "--config-hash",
            "--genesis-hash",
            "--tip-hash",
            "--ledger-root",
        ] {
            for text in [
                "".to_string(),
                "a".repeat(63),
                "a".repeat(65),
                "0x".to_string() + &"ab".repeat(31),
                "g".repeat(64),
                "é".repeat(32),
            ] {
                let mut values = init();
                change(&mut values, option, &text);
                assert_eq!(parsed(values), Err(UsageError::InvalidHash));
            }
        }
    }

    #[test]
    fn numeric_fields_reject_signs_leading_zeros_spaces_unicode_and_overflow() {
        for (base, option) in [
            (init(), "--height"),
            (init(), "--generation"),
            (sync(), "--timeout-ms"),
            (sync(), "--max-io-bytes"),
        ] {
            for text in [
                "",
                "+1",
                "-1",
                "00",
                "01",
                " 1",
                "1 ",
                "1.0",
                "1e2",
                "１",
                "18446744073709551616",
            ] {
                let mut values = base.clone();
                change(&mut values, option, text);
                assert_eq!(parsed(values), Err(UsageError::InvalidNumber));
            }
        }
    }

    #[test]
    fn duplicate_options_reject_even_with_identical_values() {
        for mut values in [common("status"), init(), sync(), recover()] {
            values.extend(["--control-dir".into(), "control".into()]);
            assert_eq!(parsed(values), Err(UsageError::DuplicateOption));
        }
        let mut values = init();
        values.push("--attach-existing".into());
        assert_eq!(parsed(values), Err(UsageError::DuplicateOption));
    }

    #[test]
    fn options_from_other_commands_are_rejected() {
        for (mut values, extra) in [
            (common("status"), vec!["--attach-existing"]),
            (init(), vec!["--action", "complete"]),
            (sync(), vec!["--height", "3"]),
            (recover(), vec!["--peer", "127.0.0.1:9000"]),
        ] {
            values.extend(extra.into_iter().map(str::to_owned));
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
    }

    #[test]
    fn unknown_flags_inline_values_and_positionals_reject() {
        for token in ["--unknown", "--control-dir=elsewhere", "-x", "--"] {
            let mut values = common("status");
            values.push(token.into());
            assert_eq!(parsed(values), Err(UsageError::UnknownOption));
        }
        for token in ["extra", "status", "true"] {
            let mut values = common("status");
            values.push(token.into());
            assert_eq!(parsed(values), Err(UsageError::UnexpectedArgument));
        }
    }

    #[test]
    fn missing_values_do_not_consume_another_flag() {
        let mut values = common("status");
        values.pop();
        assert_eq!(parsed(values), Err(UsageError::MissingValue));
        let mut values = common("status");
        values.remove(2);
        assert_eq!(parsed(values), Err(UsageError::MissingValue));
        let mut values = init();
        let at = values
            .iter()
            .position(|value| value == "--attach-existing")
            .unwrap();
        values.insert(at + 1, "true".into());
        assert_eq!(parsed(values), Err(UsageError::UnexpectedArgument));
    }

    #[test]
    fn flags_can_be_reordered_without_changing_selection() {
        let first = common("status");
        let mut second = vec!["status".into()];
        for pair in first[1..].chunks_exact(2).rev() {
            second.extend_from_slice(pair);
        }
        assert_eq!(parsed(first), parsed(second));
    }

    #[test]
    fn paths_allow_unicode_and_spaces_but_not_empty_values() {
        let mut values = common("status");
        change(&mut values, "--control-dir", "узел/control data");
        let Command::Status(parsed_common) = parsed(values).unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(parsed_common.control, PathBuf::from("узел/control data"));
        for option in ["--control-dir", "--store-dir"] {
            let mut values = common("status");
            change(&mut values, option, "");
            assert_eq!(parsed(values), Err(UsageError::EmptyPath));
        }
    }

    #[test]
    fn argument_count_is_bounded_before_command_parsing() {
        assert_eq!(
            parsed(vec!["x".into(); 32]),
            Err(UsageError::UnknownCommand)
        );
        assert_eq!(
            parsed(vec!["x".into(); 33]),
            Err(UsageError::TooManyArguments)
        );
    }

    #[test]
    fn token_and_total_byte_limits_are_inclusive_and_admitted_first() {
        let mut values = common("status");
        change(&mut values, "--control-dir", &"x".repeat(4096));
        assert!(parsed(values).is_ok());
        assert_eq!(
            parsed(vec!["x".repeat(4097)]),
            Err(UsageError::TokenTooLong)
        );
        assert_eq!(
            parsed(vec!["x".repeat(4096); 4]),
            Err(UsageError::UnknownCommand)
        );
        let mut values = vec!["x".repeat(4096); 4];
        values.push("x".into());
        assert_eq!(parsed(values), Err(UsageError::ArgumentsTooLong));
    }

    #[test]
    fn control_line_separator_and_bidi_tokens_reject_without_echo() {
        for text in [
            "a\nb", "a\rb", "a\tb", "\u{0}", "\u{7f}", "\u{85}", "\u{2028}", "\u{202e}", "\u{2066}",
        ] {
            let mut values = common("status");
            change(&mut values, "--control-dir", text);
            assert_eq!(parsed(values), Err(UsageError::ControlCharacter));
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_os_arguments_reject_before_any_command() {
        use std::os::unix::ffi::OsStringExt;
        assert_eq!(
            parse([OsString::from_vec(vec![0xff])]),
            Err(UsageError::NonUtf8)
        );
    }

    fn run_command() -> Vec<String> {
        let mut values = failover();
        values[0] = "run".into();
        values.extend([
            "--max-rounds".into(),
            "4".into(),
            "--round-wait-ms".into(),
            "0".into(),
        ]);
        values
    }

    #[test]
    fn run_requires_all_twelve_options_and_rejects_duplicate_flags() {
        let full = run_command();
        assert_eq!(full.len(), 25);
        for start in (1..full.len()).step_by(2) {
            let mut missing = full.clone();
            missing.drain(start..start + 2);
            assert_eq!(parsed(missing), Err(UsageError::MissingOption));
            let mut duplicate = full.clone();
            duplicate.extend([full[start].clone(), full[start + 1].clone()]);
            assert_eq!(parsed(duplicate), Err(UsageError::DuplicateOption));
        }
    }

    #[test]
    fn run_preserves_explicit_order_identity_and_independent_round_attempt_caps() {
        for (rounds, wait, attempts, bytes, timeout, admission) in [
            (1, 0, 1, 1, 1, 1),
            (4, 60_000, 8, MAX_SESSION_WIRE_BYTES_V2, 60_000, 60_000),
            (4, 0, 1, 1000, 10, 20),
        ] {
            let mut values = run_command();
            for (flag, value) in [
                ("--max-rounds", rounds.to_string()),
                ("--round-wait-ms", wait.to_string()),
                ("--max-attempts", attempts.to_string()),
                ("--max-io-bytes", bytes.to_string()),
                ("--timeout-ms", timeout.to_string()),
                ("--admission-ms", admission.to_string()),
            ] {
                change(&mut values, flag, &value);
            }
            change(
                &mut values,
                "--peers",
                "[::ffff:127.0.0.1]:9010,[::1]:9020,127.0.0.2:9030,127.0.0.3:9040",
            );
            let Command::Run {
                common,
                peers,
                listen,
                limits,
                admission: window,
                max_rounds,
                round_wait,
            } = parsed(values).unwrap()
            else {
                panic!("wrong command");
            };
            assert_eq!(common.identity.config_hash.to_hex(), "ab".repeat(32));
            assert_eq!(common.identity.genesis_hash.to_hex(), "cd".repeat(32));
            assert_eq!(common.control, PathBuf::from("control"));
            assert_eq!(common.store, PathBuf::from("store"));
            assert_eq!(listen, "127.0.0.1:9000".parse::<SocketAddr>().unwrap());
            assert_eq!(
                peers.as_slice(),
                &[
                    "[::ffff:127.0.0.1]:9010".parse::<SocketAddr>().unwrap(),
                    "[::1]:9020".parse().unwrap(),
                    "127.0.0.2:9030".parse().unwrap(),
                    "127.0.0.3:9040".parse().unwrap(),
                ]
            );
            assert!(peers.as_slice()[0].is_ipv6());
            assert_eq!(limits.max_attempts(), attempts);
            assert_eq!(limits.max_wire_bytes(), bytes);
            assert_eq!(limits.timeout(), Duration::from_millis(timeout));
            assert_eq!(window, Duration::from_millis(admission));
            assert_eq!(max_rounds, rounds);
            assert_eq!(round_wait, Duration::from_millis(wait));
        }
    }

    #[test]
    fn run_rejects_noncanonical_and_out_of_range_round_wait_and_transport_limits() {
        for flag in [
            "--max-rounds",
            "--round-wait-ms",
            "--max-attempts",
            "--max-io-bytes",
            "--timeout-ms",
            "--admission-ms",
        ] {
            for value in [
                "",
                "01",
                "+1",
                "-1",
                "1.0",
                " 1",
                "١",
                "18446744073709551616",
            ] {
                let mut values = run_command();
                change(&mut values, flag, value);
                assert_eq!(parsed(values), Err(UsageError::InvalidNumber));
            }
        }
        for (flag, value) in [
            ("--max-rounds", "0"),
            ("--max-rounds", "5"),
            ("--max-rounds", "256"),
            ("--round-wait-ms", "60001"),
            ("--max-attempts", "0"),
            ("--max-attempts", "9"),
            ("--max-io-bytes", "0"),
            ("--max-io-bytes", "157129489"),
            ("--timeout-ms", "0"),
            ("--timeout-ms", "60001"),
            ("--admission-ms", "0"),
            ("--admission-ms", "60001"),
        ] {
            let mut values = run_command();
            change(&mut values, flag, value);
            assert_eq!(parsed(values), Err(UsageError::OutOfRange));
        }
    }

    #[test]
    fn run_retains_bounded_numeric_list_and_alias_rejection() {
        for (list, error) in [
            ("", UsageError::InvalidPeerList),
            (",127.0.0.1:1", UsageError::InvalidPeerList),
            ("127.0.0.1:1,", UsageError::InvalidPeerList),
            ("127.0.0.1:1,,127.0.0.1:2", UsageError::InvalidPeerList),
            ("127.0.0.1:1, 127.0.0.1:2", UsageError::InvalidPeerList),
            ("[fe80::1%1]:1", UsageError::InvalidPeerList),
            ("localhost:1", UsageError::InvalidPeer),
            ("127.0.0.1:0", UsageError::InvalidPeer),
            ("[::1]:0", UsageError::InvalidPeer),
            (
                "127.0.0.1:1,127.0.0.1:2,127.0.0.1:3,127.0.0.1:4,127.0.0.1:5",
                UsageError::TooManyPeers,
            ),
            (
                "127.0.0.1:1,[::ffff:127.0.0.1]:1",
                UsageError::DuplicatePeer,
            ),
            ("[::1]:1,[0:0:0:0:0:0:0:1]:1", UsageError::DuplicatePeer),
        ] {
            let mut values = run_command();
            change(&mut values, "--peers", list);
            assert_eq!(parsed(values), Err(error));
        }
        for listen in ["localhost:1", "127.0.0.1:0", "[::1]:0"] {
            let mut values = run_command();
            change(&mut values, "--listen", listen);
            assert_eq!(parsed(values), Err(UsageError::InvalidListen));
        }
    }

    #[test]
    fn run_flags_are_separate_from_every_earlier_command_and_single_peer_mode() {
        for full in [
            common("status"),
            init(),
            sync(),
            recover(),
            serve(),
            rejoin(),
            failover(),
        ] {
            for (flag, value) in [("--max-rounds", "2"), ("--round-wait-ms", "0")] {
                let mut values = full.clone();
                values.extend([flag.into(), value.into()]);
                assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
            }
        }
        for (flag, value) in [
            ("--peer", "127.0.0.1:1"),
            ("--action", "complete"),
            ("--height", "0"),
        ] {
            let mut values = run_command();
            values.extend([flag.into(), value.into()]);
            assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
        }
        let mut values = run_command();
        values.push("--attach-existing".into());
        assert_eq!(parsed(values), Err(UsageError::WrongCommandOption));
    }

    #[test]
    fn run_keeps_original_argument_byte_count_and_control_character_admission() {
        let mut values = run_command();
        change(&mut values, "--control-dir", &"x".repeat(MAX_TOKEN_BYTES));
        assert!(parsed(values).is_ok());
        let mut values = run_command();
        change(
            &mut values,
            "--control-dir",
            &"x".repeat(MAX_TOKEN_BYTES + 1),
        );
        assert_eq!(parsed(values), Err(UsageError::TokenTooLong));
        let mut values = run_command();
        values.resize(MAX_ARGUMENTS + 1, "extra".into());
        assert_eq!(parsed(values), Err(UsageError::TooManyArguments));
        let mut values = run_command();
        change(&mut values, "--peers", "127.0.0.1:1,\n127.0.0.1:2");
        assert_eq!(parsed(values), Err(UsageError::ControlCharacter));
        let mut values = run_command();
        for flag in ["--control-dir", "--store-dir", "--peers", "--listen"] {
            change(&mut values, flag, &"x".repeat(MAX_TOKEN_BYTES));
        }
        assert_eq!(parsed(values), Err(UsageError::ArgumentsTooLong));
    }
}
