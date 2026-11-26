//! One-shot, bounded V2 TCP sessions under independently selected network pins.
//!
//! One mutable owner performs at most eight sequential attempts. Each attempt
//! owns one socket, uses one absolute deadline, and transfers one history. Failed
//! attempts and actual I/O bytes remain charged. No storage mutation occurs here.

use std::{
    fmt,
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::{ChainV2, ManifestIdentityV2};
use mirror_protocol::v2::{
    FRAME_HEADER_LEN_V2, FrameHeaderV2, FrameKindV2, HistorySummaryV2, MAX_HISTORY_FRAMES_V2,
    MAX_HISTORY_WIRE_BYTES_V2,
};

use super::{HistoryErrorV2, HistoryFramesV2, HistoryReceiverV2, ReceivedCandidateV2};

pub const REQUEST_LEN_V2: usize = 72;
pub const REQUEST_MAGIC_V2: [u8; 4] = *b"MPQ2";
pub const REQUEST_VERSION_V2: u16 = 2;
pub const REQUEST_HISTORY_KIND_V2: u16 = 1;
pub const MAX_SESSION_ATTEMPTS_V2: u8 = 8;
/// Eight complete histories, eight fixed requests, and two one-byte rejection
/// sentinels per attempt. EOF itself transfers no bytes.
pub const MAX_SESSION_WIRE_BYTES_V2: u64 =
    MAX_SESSION_ATTEMPTS_V2 as u64 * (MAX_HISTORY_WIRE_BYTES_V2 as u64 + 74);
pub const MAX_SESSION_TIMEOUT_V2: Duration = Duration::from_secs(60);

/// Encode exactly one request using independently chosen configuration/genesis.
/// This is a network selection, not a peer authentication credential.
pub fn encode_history_request_v2(identity: ManifestIdentityV2) -> [u8; REQUEST_LEN_V2] {
    let mut bytes = [0u8; REQUEST_LEN_V2];
    bytes[..4].copy_from_slice(&REQUEST_MAGIC_V2);
    bytes[4..6].copy_from_slice(&REQUEST_VERSION_V2.to_le_bytes());
    bytes[6..8].copy_from_slice(&REQUEST_HISTORY_KIND_V2.to_le_bytes());
    bytes[8..40].copy_from_slice(identity.config_hash.as_bytes());
    bytes[40..72].copy_from_slice(identity.genesis_hash.as_bytes());
    bytes
}

/// Validate the fixed request against independently retained local selection.
/// No unpinned request decoder can supply network authority to the caller.
pub fn validate_history_request_v2(
    bytes: &[u8],
    identity: ManifestIdentityV2,
) -> Result<(), SessionErrorV2> {
    if bytes.len() != REQUEST_LEN_V2
        || bytes[..4] != REQUEST_MAGIC_V2
        || bytes[4..6] != REQUEST_VERSION_V2.to_le_bytes()
        || bytes[6..8] != REQUEST_HISTORY_KIND_V2.to_le_bytes()
    {
        return Err(SessionErrorV2::InvalidRequest);
    }
    if bytes[8..40] != identity.config_hash.as_bytes()[..]
        || bytes[40..72] != identity.genesis_hash.as_bytes()[..]
    {
        return Err(SessionErrorV2::WrongNetwork);
    }
    Ok(())
}

/// Validated lifetime budget for one sequential session owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionLimitsV2 {
    max_attempts: u8,
    max_wire_bytes: u64,
    timeout: Duration,
}

impl SessionLimitsV2 {
    pub fn new(
        max_attempts: u8,
        max_wire_bytes: u64,
        timeout: Duration,
    ) -> Result<Self, SessionErrorV2> {
        if max_attempts == 0 || max_attempts > MAX_SESSION_ATTEMPTS_V2 {
            return Err(SessionErrorV2::InvalidLimits("attempt count must be 1..=8"));
        }
        if max_wire_bytes == 0 || max_wire_bytes > MAX_SESSION_WIRE_BYTES_V2 {
            return Err(SessionErrorV2::InvalidLimits(
                "wire allowance is out of range",
            ));
        }
        if timeout.is_zero() || timeout > MAX_SESSION_TIMEOUT_V2 {
            return Err(SessionErrorV2::InvalidLimits(
                "deadline must be positive and at most 60s",
            ));
        }
        Ok(Self {
            max_attempts,
            max_wire_bytes,
            timeout,
        })
    }

    pub const fn max_attempts(&self) -> u8 {
        self.max_attempts
    }

    pub const fn max_wire_bytes(&self) -> u64 {
        self.max_wire_bytes
    }

    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Cumulative accounting, including failed attempts and partial successful I/O.
/// Kernel buffering, TCP/IP overhead, RSS and work in other owners are excluded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SessionStatsV2 {
    pub attempts: u8,
    pub wire_bytes: u64,
}

/// A non-Clone sequential owner: an exclusive mutable borrow covers each whole
/// attempt, so this owner has at most one connected/accepted stream at a time.
/// Creating another owner is an explicit separate budget, not a global limiter.
#[derive(Debug)]
pub struct PeerSessionsV2 {
    limits: SessionLimitsV2,
    stats: SessionStatsV2,
}

impl PeerSessionsV2 {
    pub const fn new(limits: SessionLimitsV2) -> Self {
        Self {
            limits,
            stats: SessionStatsV2 {
                attempts: 0,
                wire_bytes: 0,
            },
        }
    }

    pub const fn stats(&self) -> SessionStatsV2 {
        self.stats
    }

    pub const fn limits(&self) -> SessionLimitsV2 {
        self.limits
    }

    /// Connect once to a numeric address, request one history, and consume the
    /// connection. Only accepted End followed by EOF can release a candidate.
    /// An error never returns a partial candidate or resets this owner's budget.
    pub fn request(
        &mut self,
        address: SocketAddr,
        identity: ManifestIdentityV2,
    ) -> Result<ReceivedCandidateV2, SessionErrorV2> {
        let deadline = self.begin_attempt()?;
        let stream = TcpStream::connect_timeout(&address, deadline.remaining()?)?;
        deadline.check()?;
        let mut socket = SessionSocket::new(stream)?;
        deadline.check()?;
        self.write_bytes(
            &mut socket.stream,
            &encode_history_request_v2(identity),
            deadline,
        )?;
        socket.stream.shutdown(Shutdown::Write)?;
        deadline.check()?;

        let mut receiver = HistoryReceiverV2::new(identity);
        for _ in 0..MAX_HISTORY_FRAMES_V2 {
            let mut header = [0u8; FRAME_HEADER_LEN_V2];
            self.read_bytes(&mut socket.stream, &mut header, deadline)?;
            // Decode only the fixed header to recognize terminal End. Admission
            // still consumes the receiver and precedes ANY payload allocation.
            let kind = FrameHeaderV2::decode(&header)
                .map_err(HistoryErrorV2::from)?
                .kind();
            let pending = receiver.begin_frame(&header)?;
            deadline.check()?;
            if pending.payload_len() as u64 > self.remaining_wire_bytes()? {
                return Err(SessionErrorV2::WireBudgetExhausted);
            }
            let mut payload = Vec::new();
            payload
                .try_reserve_exact(pending.payload_len())
                .map_err(|_| SessionErrorV2::AllocationFailed)?;
            payload.resize(pending.payload_len(), 0);
            deadline.check()?;
            self.read_bytes(&mut socket.stream, &mut payload, deadline)?;
            receiver = pending.accept_payload(&payload)?;
            // Replay is bounded but not preemptible. A result that finishes
            // after the absolute deadline is rejected instead of published.
            deadline.check()?;
            if kind == FrameKindV2::End {
                self.expect_eof(&mut socket.stream, deadline)?;
                let candidate = receiver.finish()?;
                deadline.check()?;
                return Ok(candidate);
            }
        }
        Err(SessionErrorV2::History(HistoryErrorV2::FrameBudget))
    }

    /// Consume one listener, accept at most one connection, and serve a single
    /// immutable accepted history. Request EOF is mandatory before any response.
    /// There is no retry/accept loop after failure and no persistent daemon.
    pub fn serve_once(
        &mut self,
        listener: TcpListener,
        identity: ManifestIdentityV2,
        chain: &ChainV2,
    ) -> Result<HistorySummaryV2, SessionErrorV2> {
        let deadline = self.begin_attempt()?;
        if chain.manifest().identity() != identity {
            return Err(SessionErrorV2::WrongNetwork);
        }
        deadline.check()?;
        listener.set_nonblocking(true)?;
        let stream = loop {
            deadline.check()?;
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(deadline.remaining()?.min(Duration::from_millis(1)));
                }
                Err(error) => return Err(error.into()),
            }
        };
        // The consumed listener cannot accept a second client while this one
        // is active. OS backlog sockets and independently cloned descriptors
        // are not a process-wide availability guarantee.
        drop(listener);
        deadline.check()?;
        let mut socket = SessionSocket::new(stream)?;
        let mut request = [0u8; REQUEST_LEN_V2];
        self.read_bytes(&mut socket.stream, &mut request, deadline)?;
        validate_history_request_v2(&request, identity)?;
        deadline.check()?;
        self.expect_eof(&mut socket.stream, deadline)?;

        let mut frames = HistoryFramesV2::new(chain)?;
        deadline.check()?;
        let summary = frames.summary();
        loop {
            deadline.check()?;
            let Some(frame) = frames.next() else {
                break;
            };
            let frame = frame?;
            deadline.check()?;
            self.write_bytes(&mut socket.stream, &frame, deadline)?;
        }
        socket.stream.shutdown(Shutdown::Write)?;
        deadline.check()?;
        Ok(summary)
    }

    fn begin_attempt(&mut self) -> Result<Deadline, SessionErrorV2> {
        if self.stats.attempts >= self.limits.max_attempts {
            return Err(SessionErrorV2::AttemptsExhausted);
        }
        self.stats.attempts += 1;
        // Even a refused attempt cannot be retried without being charged.
        self.remaining_wire_bytes()?;
        let end = Instant::now()
            .checked_add(self.limits.timeout)
            .ok_or(SessionErrorV2::InvalidLimits("deadline overflow"))?;
        Ok(Deadline(end))
    }

    fn remaining_wire_bytes(&self) -> Result<u64, SessionErrorV2> {
        let remaining = self.limits.max_wire_bytes - self.stats.wire_bytes;
        if remaining == 0 {
            Err(SessionErrorV2::WireBudgetExhausted)
        } else {
            Ok(remaining)
        }
    }

    fn charge(&mut self, count: usize) -> Result<(), SessionErrorV2> {
        self.stats.wire_bytes = self
            .stats
            .wire_bytes
            .checked_add(count as u64)
            .filter(|total| *total <= self.limits.max_wire_bytes)
            .ok_or(SessionErrorV2::WireBudgetExhausted)?;
        Ok(())
    }

    fn read_some(
        &mut self,
        stream: &mut TcpStream,
        bytes: &mut [u8],
        deadline: Deadline,
    ) -> Result<usize, SessionErrorV2> {
        loop {
            let available = self.remaining_wire_bytes()?;
            let len = available.min(bytes.len() as u64) as usize;
            stream.set_read_timeout(Some(deadline.remaining()?))?;
            match stream.read(&mut bytes[..len]) {
                Ok(count) => {
                    self.charge(count)?;
                    deadline.check()?;
                    return Ok(count);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    // OS timeout rounding can wake slightly early. It may not
                    // refresh the original absolute deadline.
                    deadline.check()?;
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn read_bytes(
        &mut self,
        stream: &mut TcpStream,
        mut bytes: &mut [u8],
        deadline: Deadline,
    ) -> Result<(), SessionErrorV2> {
        while !bytes.is_empty() {
            let count = self.read_some(stream, bytes, deadline)?;
            if count == 0 {
                return Err(SessionErrorV2::EarlyClose);
            }
            bytes = &mut bytes[count..];
        }
        deadline.check()
    }

    fn write_bytes(
        &mut self,
        stream: &mut TcpStream,
        mut bytes: &[u8],
        deadline: Deadline,
    ) -> Result<(), SessionErrorV2> {
        while !bytes.is_empty() {
            let available = self.remaining_wire_bytes()?;
            let len = available.min(bytes.len() as u64) as usize;
            stream.set_write_timeout(Some(deadline.remaining()?))?;
            match stream.write(&bytes[..len]) {
                Ok(count) => {
                    self.charge(count)?;
                    deadline.check()?;
                    if count == 0 {
                        return Err(SessionErrorV2::Io(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "peer accepted no output bytes",
                        )));
                    }
                    bytes = &bytes[count..];
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    deadline.check()?;
                }
                Err(error) => return Err(error.into()),
            }
        }
        deadline.check()
    }

    fn expect_eof(
        &mut self,
        stream: &mut TcpStream,
        deadline: Deadline,
    ) -> Result<(), SessionErrorV2> {
        // One unused byte of budget is required to perform this sentinel read;
        // a zero-byte EOF is not charged, while trailing data is charged.
        let mut sentinel = [0u8; 1];
        if self.read_some(stream, &mut sentinel, deadline)? != 0 {
            return Err(SessionErrorV2::TrailingData);
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Deadline(Instant);

impl Deadline {
    fn remaining(self) -> Result<Duration, SessionErrorV2> {
        self.0
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or(SessionErrorV2::DeadlineExceeded)
    }

    fn check(self) -> Result<(), SessionErrorV2> {
        self.remaining().map(|_| ())
    }
}

struct SessionSocket {
    stream: TcpStream,
}

impl SessionSocket {
    fn new(stream: TcpStream) -> Result<Self, SessionErrorV2> {
        let socket = Self { stream };
        socket.stream.set_nonblocking(false)?;
        socket.stream.set_nodelay(true)?;
        Ok(socket)
    }
}

impl Drop for SessionSocket {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

#[derive(Debug)]
pub enum SessionErrorV2 {
    InvalidLimits(&'static str),
    InvalidRequest,
    WrongNetwork,
    AttemptsExhausted,
    WireBudgetExhausted,
    DeadlineExceeded,
    EarlyClose,
    TrailingData,
    AllocationFailed,
    Io(io::Error),
    History(HistoryErrorV2),
}

impl fmt::Display for SessionErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 peer session rejected: {self:?}")
    }
}

impl std::error::Error for SessionErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::History(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for SessionErrorV2 {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<HistoryErrorV2> for SessionErrorV2 {
    fn from(error: HistoryErrorV2) -> Self {
        Self::History(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn nonreading_peer_cannot_extend_partial_writes_past_absolute_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (done, wait) = mpsc::channel();
        let peer = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            let _ = wait.recv_timeout(Duration::from_secs(3));
        });
        let mut stream = TcpStream::connect(address).unwrap();
        let mut sessions = PeerSessionsV2::new(
            SessionLimitsV2::new(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(150)).unwrap(),
        );
        // This directly exercises the private syscall loop with more data than
        // the loopback send/receive buffers. It is not a forged accepted history.
        let bytes = vec![0x5a; 8 * 1024 * 1024];
        let deadline = sessions.begin_attempt().unwrap();
        let start = Instant::now();
        assert!(matches!(
            sessions.write_bytes(&mut stream, &bytes, deadline),
            Err(SessionErrorV2::DeadlineExceeded)
        ));
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(sessions.stats().wire_bytes > 0);
        assert!(sessions.stats().wire_bytes < bytes.len() as u64);
        assert_eq!(sessions.stats().attempts, 1);
        done.send(()).unwrap();
        peer.join().unwrap();
    }
}
