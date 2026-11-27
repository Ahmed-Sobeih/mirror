//! Bounded actual-command observations for receive/adopt/serve composition.
//!
//! Existing B1/operator fixtures are imported without changes. The additional
//! Linux observations establish actual outbound receive and stopped adoption;
//! neither TCP handshake completion nor elapsed time is treated as admission.
#![allow(dead_code, unused_imports)]

#[path = "../support_operator_v2/mod.rs"]
mod operator;
#[path = "../support_provider_v2/mod.rs"]
pub mod provider;

pub use operator::{Peer, PeerMode};
pub use provider::{
    ProviderProcess, assert_busy, assert_no_connection, assert_port_closed, command, common,
    connect, failure, no_response_yet, process_guard, raw_request, read_response, ready,
    receive_complete, remove, reserve, set, state, success, transcript,
};

use std::{
    collections::BTreeMap,
    ffi::{CString, OsString},
    fs::{self, File},
    io::{self, Read, Write},
    net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::{fs::MetadataExt, net::UnixStream},
    },
    path::Path,
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::ChainV2;
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::BlockV2;
use mirror_network::v2::session::{
    MAX_SESSION_WIRE_BYTES_V2, REQUEST_LEN_V2, validate_history_request_v2,
};
use mirror_node::v2::{
    NodeControlV2,
    records::{CONTROL_RECORD_LEN_V2, ControlRecordV2, IntentRecordV2},
};
use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementStoreV2};

use state::{
    Scratch, assert_complete_state, checkpoint, control_root,
    history::{independent_identity, main_chain},
    preferred_fork,
};

const PROCESS_LIMIT: Duration = Duration::from_secs(20);
const OBSERVATION_LIMIT: Duration = Duration::from_secs(8);
const SOCKET_LIMIT: Duration = Duration::from_secs(5);
const MAX_PROC_BYTES: usize = 1_048_576;
const MAX_CHILD_FDS: usize = 128;

pub fn arguments(scratch: &Scratch, peer: SocketAddr, listen: SocketAddr) -> Vec<OsString> {
    let mut result = common("rejoin-serve", scratch);
    result.extend([
        "--peer".into(),
        peer.to_string().into(),
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

pub fn sync(scratch: &Scratch, peer: SocketAddr) -> Vec<OsString> {
    let mut result = common("sync-once", scratch);
    result.extend([
        "--peer".into(),
        peer.to_string().into(),
        "--max-io-bytes".into(),
        MAX_SESSION_WIRE_BYTES_V2.to_string().into(),
        "--timeout-ms".into(),
        "5000".into(),
    ]);
    result
}

pub fn recover(scratch: &Scratch, action: &str) -> Vec<OsString> {
    let mut result = common("recover", scratch);
    result.extend(["--action".into(), action.into()]);
    result
}

pub fn ready_with(label: &str, chain: &ChainV2) -> Scratch {
    let scratch = Scratch::new(label);
    drop(ReplacementStoreV2::initialize(scratch.root(), chain).unwrap());
    drop(
        NodeControlV2::initialize(
            control_root(&scratch),
            scratch.root(),
            independent_identity(),
            checkpoint(chain, 0),
        )
        .unwrap(),
    );
    scratch
}

pub fn pending(scratch: &Scratch, candidate: &ChainV2) -> IntentRecordV2 {
    let old = main_chain(3);
    let intent = IntentRecordV2::new(
        independent_identity(),
        checkpoint(&old, 0),
        old,
        candidate.clone(),
    )
    .unwrap();
    fs::write(
        control_root(scratch).join("intent.v2"),
        intent.encode().unwrap(),
    )
    .unwrap();
    intent
}

pub fn pins(values: &BTreeMap<String, String>, prefix: &str, selected: ReplacementCheckpointV2) {
    assert_eq!(
        values[&format!("{prefix}_generation")],
        selected.generation.to_string()
    );
    assert_eq!(
        values[&format!("{prefix}_height")],
        selected.chain.height.to_string()
    );
    assert_eq!(
        values[&format!("{prefix}_tip_hash")],
        selected.chain.tip_hash.to_string()
    );
    assert_eq!(
        values[&format!("{prefix}_ledger_root")],
        selected.chain.ledger_root.to_string()
    );
}

pub fn selected(scratch: &Scratch, expected: &ChainV2, generation: u64) {
    let status = success(&command(&common("status", scratch)), 0, "status");
    assert_eq!(status["status"], "clean");
    assert_eq!(status["store_selection"], "verified");
    pins(&status, "selected", checkpoint(expected, generation));
    let owner = NodeControlV2::open_writer(
        control_root(scratch),
        scratch.root(),
        independent_identity(),
    )
    .unwrap();
    assert_eq!(owner.checkpoint(), checkpoint(expected, generation));
    assert_complete_state(owner.chain(), expected);
    assert_eq!(fs::read_dir(control_root(scratch)).unwrap().count(), 1);
    assert!(control_root(scratch).join("confirmed.v2").is_file());
}

pub fn receive_bytes(chain: &ChainV2) -> u64 {
    (REQUEST_LEN_V2 + transcript(chain).len()) as u64
}

#[derive(Clone, Copy)]
pub struct Service<'a> {
    pub completed: u8,
    pub rejected: u8,
    pub bytes: u64,
    pub last_error: &'a str,
}

impl Service<'static> {
    pub const NONE: Self = Self {
        completed: 0,
        rejected: 0,
        bytes: 0,
        last_error: "none",
    };

    pub const fn complete(bytes: u64) -> Self {
        Self {
            completed: 1,
            rejected: 0,
            bytes,
            last_error: "none",
        }
    }
}

pub struct Normal<'a> {
    pub previous: ReplacementCheckpointV2,
    pub selected: ReplacementCheckpointV2,
    pub choice: &'a str,
    pub reason: &'a str,
    pub receive_bytes: u64,
    pub service: Service<'a>,
}

impl Normal<'static> {
    pub fn adopted(candidate: &ChainV2, reason: &'static str) -> Self {
        Self {
            previous: checkpoint(&main_chain(3), 0),
            selected: checkpoint(candidate, 1),
            choice: "prefer-candidate",
            reason,
            receive_bytes: receive_bytes(candidate),
            service: Service::NONE,
        }
    }

    pub fn retained(candidate: &ChainV2, choice: &'static str, reason: &'static str) -> Self {
        let previous = checkpoint(&main_chain(3), 0);
        Self {
            previous,
            selected: previous,
            choice,
            reason,
            receive_bytes: receive_bytes(candidate),
            service: Service::NONE,
        }
    }
}

pub fn stopped(output: &Output, expected: Normal<'_>) -> BTreeMap<String, String> {
    assert!(output.stdout.len() <= 4096);
    let values = success(output, 0, "rejoin-serve");
    assert_eq!(
        values.len(),
        26,
        "unexpected fields in B2's final normal record"
    );
    assert_eq!(values["status"], "stopped");
    assert_eq!(values["store_selection"], "verified");
    pins(&values, "previous", expected.previous);
    pins(&values, "selected", expected.selected);
    assert_eq!(values["receive_status"], "validated");
    assert_eq!(values["choice"], expected.choice);
    assert_eq!(values["reason"], expected.reason);
    let service = expected.service;
    let serve_attempts = service.completed + service.rejected;
    for (key, expected) in [
        ("receive_attempts", 1),
        ("serve_attempts", u64::from(serve_attempts)),
        ("attempts", u64::from(serve_attempts) + 1),
        ("completed_sessions", u64::from(service.completed)),
        ("rejected_sessions", u64::from(service.rejected)),
        ("receive_wire_bytes", expected.receive_bytes),
        ("serve_wire_bytes", service.bytes),
        ("wire_bytes", expected.receive_bytes + service.bytes),
    ] {
        assert_eq!(values[key], expected.to_string(), "{key}");
        assert_eq!(values[key].parse::<u64>().unwrap().to_string(), values[key]);
    }
    assert_eq!(values["last_session_error"], service.last_error);
    values
}

pub fn uncertain(output: &Output, error: &str) -> BTreeMap<String, String> {
    assert!(output.stderr.len() <= 4096);
    let values = failure(output, 1);
    assert_eq!(values["operation"], "rejoin-serve");
    assert_eq!(values["error"], error);
    assert_eq!(values["operation_may_have_completed"], "true");
    assert_eq!(values["next"], "inspect-status");
    assert!(!values.keys().any(|key| key.starts_with("selected_")));
    values
}

/// Accept exactly this command's outgoing request and retain the response half
/// in the parent. Each helper read shares one fixed original deadline.
pub fn accept_request(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + SOCKET_LIMIT;
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "rejoin did not contact its selected peer"
                );
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("cannot accept synthetic source request: {error}"),
        }
    };
    stream.set_nodelay(true).unwrap();
    stream.set_write_timeout(Some(SOCKET_LIMIT)).unwrap();
    let mut request = [0; REQUEST_LEN_V2];
    let mut offset = 0;
    while offset < request.len() {
        stream
            .set_read_timeout(Some(
                deadline.checked_duration_since(Instant::now()).unwrap(),
            ))
            .unwrap();
        match stream.read(&mut request[offset..]) {
            Ok(0) => panic!("rejoin closed an incomplete request"),
            Ok(count) => offset += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => panic!("source request read failed: {error}"),
        }
        assert!(Instant::now() < deadline);
    }
    validate_history_request_v2(&request, independent_identity()).unwrap();
    stream
        .set_read_timeout(Some(
            deadline.checked_duration_since(Instant::now()).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        stream.read(&mut [0; 1]).unwrap(),
        0,
        "request must end before response"
    );
    assert!(Instant::now() < deadline);
    stream
}

/// More genuine publication work makes the externally observed live adoption
/// interval practical. This is a valid bounded history, never a production hook.
pub fn adoption_history() -> ChainV2 {
    let mut chain = preferred_fork();
    for _ in 0..128 {
        let proposal = chain
            .prepare_candidate(chain.blocks().last().unwrap().header().timestamp() + 1, &[])
            .unwrap();
        let mined = mine_header_v2(
            *proposal.header(),
            chain.config().parameters().pow_bits,
            10_000,
        )
        .unwrap();
        chain
            .append(BlockV2::new(chain.config(), mined.header, proposal.operations()).unwrap())
            .unwrap();
    }
    assert!(chain.blocks().len() < 256);
    assert!(transcript(&chain).len() < 65_536);
    chain
}

/// An event only prompts stopping the child. The test must subsequently verify
/// stopped state, canonical complete intent and both live owners; the event
/// itself does not establish any particular completed fsync boundary.
pub struct IntentWatch(File);

impl IntentWatch {
    pub fn new(control: &Path) -> Self {
        // SAFETY: no pointers; success returns one new uniquely owned descriptor.
        let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        assert!(fd >= 0, "{}", io::Error::last_os_error());
        // SAFETY: fd is the successful, newly owned inotify descriptor.
        let file = unsafe { File::from_raw_fd(fd) };
        let path = CString::new(control.as_os_str().as_encoded_bytes()).unwrap();
        // SAFETY: the file remains owned and path is a valid terminated C string.
        assert!(
            unsafe { libc::inotify_add_watch(file.as_raw_fd(), path.as_ptr(), libc::IN_MOVED_TO) }
                >= 0
        );
        Self(file)
    }

    pub fn stop_on_intent(&mut self, child: &mut RejoinProcess) {
        let deadline = Instant::now() + OBSERVATION_LIMIT;
        let mut buffer = [0u8; 8192];
        loop {
            assert!(
                child.running(),
                "child ended before a live adoption could be observed"
            );
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("intent publication observation timed out");
            let mut poll = libc::pollfd {
                fd: self.0.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: poll points to one valid stack descriptor, for this bounded call.
            let result = unsafe { libc::poll(&mut poll, 1, remaining.as_millis().min(100) as i32) };
            assert!(
                result >= 0,
                "inotify poll failed: {}",
                io::Error::last_os_error()
            );
            if result == 0 {
                continue;
            }
            assert_eq!(
                poll.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL),
                0
            );
            let count = self.0.read(&mut buffer).unwrap();
            let header_len = std::mem::size_of::<libc::inotify_event>();
            let mut offset = 0;
            while offset < count {
                assert!(count - offset >= header_len, "truncated inotify event");
                // SAFETY: bounds above cover the header; unaligned reads are
                // required because the byte array has no C-structure alignment.
                let event = unsafe {
                    std::ptr::read_unaligned(
                        buffer.as_ptr().add(offset).cast::<libc::inotify_event>(),
                    )
                };
                let end = offset.checked_add(header_len + event.len as usize).unwrap();
                assert!(end <= count, "truncated inotify name");
                assert_eq!(
                    event.mask & libc::IN_Q_OVERFLOW,
                    0,
                    "inotify queue overflow"
                );
                let name = &buffer[offset + header_len..end];
                let name = name.split(|byte| *byte == 0).next().unwrap();
                if event.mask & libc::IN_MOVED_TO != 0 && name == b"intent.v2" {
                    child.signal(libc::SIGSTOP);
                    child.wait_stopped();
                    return;
                }
                offset = end;
            }
        }
    }
}

pub fn verify_stopped_intent(
    child: &mut RejoinProcess,
    scratch: &Scratch,
    expected: &IntentRecordV2,
) {
    child.assert_stopped();
    let encoded = expected.encode().unwrap();
    let mut actual = Vec::new();
    File::open(control_root(scratch).join("intent.v2"))
        .expect("missed live adoption: published intent absent after verified stop")
        .take(encoded.len() as u64 + 1)
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(
        actual, encoded,
        "stopped intent must be the complete canonical expected record"
    );
    let decoded = IntentRecordV2::decode_for(&actual, independent_identity()).unwrap();
    assert_eq!(decoded.identity(), independent_identity());
    assert_eq!(decoded.old(), expected.old());
    assert_eq!(decoded.candidate(), expected.candidate());
    assert_complete_state(decoded.old_chain(), expected.old_chain());
    assert_complete_state(decoded.candidate_chain(), expected.candidate_chain());
    let mut confirmed = Vec::new();
    File::open(control_root(scratch).join("confirmed.v2"))
        .unwrap()
        .take(CONTROL_RECORD_LEN_V2 as u64 + 1)
        .read_to_end(&mut confirmed)
        .unwrap();
    decoded
        .validate_confirmed(
            &ControlRecordV2::decode_for(&confirmed, independent_identity()).unwrap(),
        )
        .unwrap();
    assert_busy(scratch);
    assert!(!child.listener_is_owned());
    assert_eq!(child.directory_owners(scratch).len(), 2);
    child.assert_stopped();
}

enum OutputMode {
    Captured,
    Closed,
    Blocked,
}

pub struct RejoinProcess {
    child: Option<Child>,
    address: SocketAddr,
    output_reader: Option<File>,
}

impl RejoinProcess {
    pub fn spawn(arguments: &[OsString], reservation: TcpListener) -> Self {
        let address = reservation.local_addr().unwrap();
        Self::spawn_with_output(arguments, address, Some(reservation), OutputMode::Captured)
    }

    pub fn spawn_closed_stdout(arguments: &[OsString], reservation: TcpListener) -> Self {
        let address = reservation.local_addr().unwrap();
        Self::spawn_with_output(arguments, address, Some(reservation), OutputMode::Closed)
    }

    pub fn spawn_blocked_stdout(arguments: &[OsString], reservation: TcpListener) -> Self {
        let address = reservation.local_addr().unwrap();
        Self::spawn_with_output(arguments, address, Some(reservation), OutputMode::Blocked)
    }

    fn spawn_with_output(
        arguments: &[OsString],
        address: SocketAddr,
        reservation: Option<TcpListener>,
        mode: OutputMode,
    ) -> Self {
        assert_ne!(address.port(), 0);
        let (stdout, output_reader) = match mode {
            OutputMode::Captured => (Stdio::piped(), None),
            OutputMode::Closed => {
                let (reader, writer) = UnixStream::pair().unwrap();
                drop(reader);
                (Stdio::from(OwnedFd::from(writer)), None)
            }
            OutputMode::Blocked => full_stdout_pipe(),
        };
        let mut command = Command::new(env!("CARGO_BIN_EXE_mirror-node-v2"));
        command
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(Stdio::piped());
        // Production requires a chosen nonzero port. This deliberately exposed
        // reservation/launch gap has no retry: a genuine collision fails the test.
        drop(reservation);
        let child = command.spawn().expect("start actual V2 rejoin command");
        assert_ne!(child.id(), std::process::id());
        Self {
            child: Some(child),
            address,
            output_reader,
        }
    }

    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn running(&mut self) -> bool {
        self.child.as_mut().unwrap().try_wait().unwrap().is_none()
    }

    pub fn wait_listening(&mut self) {
        let address = self.address;
        self.observe("owned listening socket", move |pid| {
            Ok(!socket_fds(pid, address, None, 0x0a)?.is_empty())
        });
    }

    pub fn wait_receiving(&mut self, client: &TcpStream, length: usize) {
        assert_eq!(client.peer_addr().unwrap(), self.address);
        let address = self.address;
        let peer = client.local_addr().unwrap();
        self.observe(
            "accepted socket blocked in the expected receive",
            move |pid| {
                let fds = socket_fds(pid, address, Some(peer), 0x01)?;
                let Some(call) = syscall(pid)? else {
                    return Ok(false);
                };
                Ok(
                    (call.number == libc::SYS_read || call.number == libc::SYS_recvfrom)
                        && fds.iter().any(|fd| u64::try_from(*fd).unwrap() == call.fd)
                        && call.length == length as u64,
                )
            },
        );
    }

    pub fn wait_stdout_blocked(&mut self) {
        assert!(self.output_reader.is_some());
        self.observe("final stdout write blocked on the full pipe", |pid| {
            Ok(syscall(pid)?.is_some_and(|call| {
                (call.number == libc::SYS_write || call.number == libc::SYS_writev)
                    && call.fd == 1
                    && call.length > 0
            }))
        });
    }

    pub fn listener_is_owned(&self) -> bool {
        !socket_fds(self.child.as_ref().unwrap().id(), self.address, None, 0x0a)
            .unwrap()
            .is_empty()
    }

    pub fn signal(&mut self, signal: i32) {
        assert!(
            self.running(),
            "rejoin command ended before the chosen signal"
        );
        let pid = i32::try_from(self.child.as_ref().unwrap().id()).unwrap();
        // SAFETY: this is the positive PID of our live, still-owned Child; only
        // explicitly selected test signals target this still-owned child.
        assert_eq!(
            unsafe { libc::kill(pid, signal) },
            0,
            "{}",
            io::Error::last_os_error()
        );
    }

    pub fn close_output_reader(&mut self) {
        assert!(self.output_reader.take().is_some());
    }

    fn observe(&mut self, label: &str, mut predicate: impl FnMut(u32) -> io::Result<bool>) {
        let deadline = Instant::now() + OBSERVATION_LIMIT;
        loop {
            if !self.running() {
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "rejoin command exited before {label}: status={} stdout={} stderr={}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            let pid = self.child.as_ref().unwrap().id();
            match predicate(pid) {
                Ok(true) => return,
                Ok(false) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    panic!("direct-parent Linux /proc observation for {label} unavailable: {error}")
                }
            }
            assert!(
                Instant::now() < deadline,
                "timed out observing {label}; last syscall={:?}",
                read_proc(&format!("/proc/{pid}/syscall"), 4096)
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    pub fn finish(mut self) -> Output {
        let deadline = Instant::now() + PROCESS_LIMIT;
        loop {
            if !self.running() {
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                assert!(
                    output.stdout.len() <= 4096,
                    "unbounded rejoin command stdout"
                );
                assert!(
                    output.stderr.len() <= 4096,
                    "unbounded rejoin command stderr"
                );
                return output;
            }
            if Instant::now() >= deadline {
                self.child.as_mut().unwrap().kill().unwrap();
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "rejoin command exceeded parent process deadline: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub fn kill(mut self) -> Output {
        self.child.as_mut().unwrap().kill().unwrap();
        self.child.take().unwrap().wait_with_output().unwrap()
    }
}

impl Drop for RejoinProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl RejoinProcess {
    /// Keep a parent-owned listening socket occupied for no-bind/order proofs.
    pub fn spawn_occupied(arguments: &[OsString], address: SocketAddr) -> Self {
        Self::spawn_with_output(arguments, address, None, OutputMode::Captured)
    }

    pub fn wait_outbound_receiving(&mut self, source: &TcpStream, length: usize) {
        let address = source.peer_addr().unwrap();
        let peer = source.local_addr().unwrap();
        self.observe(
            "outbound socket blocked in the expected receive",
            move |pid| {
                // The requester has already shutdown its write half. Reading its
                // full request/EOF in the parent establishes this FIN_WAIT2 side,
                // unlike B1's still-open accepted request socket (ESTABLISHED).
                let fds = socket_fds(pid, address, Some(peer), 0x05)?;
                let Some(call) = syscall(pid)? else {
                    return Ok(false);
                };
                Ok(
                    (call.number == libc::SYS_read || call.number == libc::SYS_recvfrom)
                        && fds.iter().any(|fd| u64::try_from(*fd).unwrap() == call.fd)
                        && call.length == length as u64,
                )
            },
        );
    }

    pub fn wait_stopped(&mut self) {
        self.observe("owned child in an actual SIGSTOP state", |pid| {
            Ok(
                status_value(&read_proc(&format!("/proc/{pid}/status"), 65_536)?, "State")?
                    .split_whitespace()
                    .next()
                    == Some("T"),
            )
        });
    }

    pub fn assert_stopped(&mut self) {
        assert!(self.running());
        let pid = self.child.as_ref().unwrap().id();
        let status = read_proc(&format!("/proc/{pid}/status"), 65_536).unwrap();
        assert_eq!(
            status_value(&status, "State")
                .unwrap()
                .split_whitespace()
                .next(),
            Some("T")
        );
    }

    pub fn wait_pending_termination(&mut self) {
        self.observe("SIGTERM pending while the child remains stopped", |pid| {
            let status = read_proc(&format!("/proc/{pid}/status"), 65_536)?;
            if status_value(&status, "State")?.split_whitespace().next() != Some("T") {
                return Err(io::Error::other(
                    "child resumed before pending signal verification",
                ));
            }
            let mask = |key| {
                u64::from_str_radix(status_value(&status, key)?.trim(), 16)
                    .map_err(|_| io::Error::other("invalid Linux signal mask"))
            };
            let term = 1u64 << (libc::SIGTERM - 1);
            assert_ne!(
                mask("SigCgt")? & term,
                0,
                "child must retain the graceful handler"
            );
            assert_eq!(mask("SigBlk")? & term, 0, "SIGTERM must not be blocked");
            Ok((mask("SigPnd")? | mask("ShdPnd")?) & term != 0)
        });
    }

    /// Observe the same two directory inode descriptors at controlled phases.
    /// Finite observations complement the one-controller implementation; they
    /// are not described as detecting every hypothetical microscopic gap.
    pub fn directory_owners(&self, scratch: &Scratch) -> Vec<(u64, u64, i32)> {
        let wanted = [control_root(scratch), scratch.root()].map(|root| {
            let metadata = fs::metadata(root).unwrap();
            (metadata.dev(), metadata.ino())
        });
        let pid = self.child.as_ref().unwrap().id();
        let mut result = Vec::new();
        for (index, entry) in fs::read_dir(format!("/proc/{pid}/fd")).unwrap().enumerate() {
            assert!(index < MAX_CHILD_FDS);
            let entry = entry.unwrap();
            let metadata = match fs::metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => panic!("cannot observe owned directory descriptor: {error}"),
            };
            if metadata.is_dir() && wanted.contains(&(metadata.dev(), metadata.ino())) {
                let fd = entry.file_name().to_string_lossy().parse().unwrap();
                result.push((metadata.dev(), metadata.ino(), fd));
            }
        }
        result.sort();
        assert_eq!(
            result.len(),
            2,
            "child must hold both independent root descriptors"
        );
        result
    }
}

fn status_value<'a>(status: &'a str, key: &str) -> io::Result<&'a str> {
    status
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name == key).then_some(value.trim())
        })
        .ok_or_else(|| io::Error::other("missing Linux process-status field"))
}

struct Syscall {
    number: libc::c_long,
    fd: u64,
    length: u64,
}

fn syscall(pid: u32) -> io::Result<Option<Syscall>> {
    let text = read_proc(&format!("/proc/{pid}/syscall"), 4096)?;
    let fields = text.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 9 {
        // The kernel reports "running" or a shorter -1/SP/PC userspace record
        // between syscalls; neither establishes a blocked receive or write.
        return Ok(None);
    }
    let number = fields[0]
        .parse::<libc::c_long>()
        .map_err(|_| io::Error::other("invalid Linux syscall number"))?;
    let argument = |index: usize| {
        u64::from_str_radix(fields[index].trim_start_matches("0x"), 16)
            .map_err(|_| io::Error::other("invalid Linux syscall argument"))
    };
    Ok(Some(Syscall {
        number,
        fd: argument(1)?,
        length: argument(3)?,
    }))
}

fn socket_fds(
    pid: u32,
    address: SocketAddr,
    peer: Option<SocketAddr>,
    state: u8,
) -> io::Result<Vec<i32>> {
    let mut owned = Vec::new();
    for (index, entry) in fs::read_dir(format!("/proc/{pid}/fd"))?.enumerate() {
        if index >= MAX_CHILD_FDS {
            return Err(io::Error::other(
                "child descriptor observation bound exceeded",
            ));
        }
        let entry = entry?;
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let Some(inode) = target
            .to_str()
            .and_then(|value| value.strip_prefix("socket:["))
            .and_then(|value| value.strip_suffix(']'))
        else {
            continue;
        };
        let fd = entry
            .file_name()
            .to_string_lossy()
            .parse::<i32>()
            .map_err(|_| io::Error::other("invalid child descriptor name"))?;
        owned.push((fd, inode.to_owned()));
    }
    let table = read_proc(&format!("/proc/{pid}/net/tcp"), MAX_PROC_BYTES)?;
    let mut matching = Vec::new();
    for line in table.lines().skip(1) {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 10 {
            return Err(io::Error::other("incomplete Linux TCP-table record"));
        }
        let current_state = u8::from_str_radix(fields[3], 16)
            .map_err(|_| io::Error::other("invalid Linux TCP state"))?;
        if current_state != state || tcp_address(fields[1])? != address {
            continue;
        }
        if let Some(peer) = peer {
            if tcp_address(fields[2])? != peer {
                continue;
            }
        }
        matching.extend(
            owned
                .iter()
                .filter(|(_, inode)| inode == fields[9])
                .map(|(fd, _)| *fd),
        );
    }
    Ok(matching)
}

fn tcp_address(text: &str) -> io::Result<SocketAddr> {
    let (address, port) = text
        .split_once(':')
        .ok_or_else(|| io::Error::other("invalid Linux TCP endpoint"))?;
    let address = u32::from_str_radix(address, 16)
        .map_err(|_| io::Error::other("invalid Linux TCP address"))?;
    let port =
        u16::from_str_radix(port, 16).map_err(|_| io::Error::other("invalid Linux TCP port"))?;
    Ok(SocketAddrV4::new(Ipv4Addr::from(address.to_ne_bytes()), port).into())
}

fn read_proc(path: &str, maximum: usize) -> io::Result<String> {
    let mut result = String::new();
    File::open(path)?
        .take(maximum as u64 + 1)
        .read_to_string(&mut result)?;
    if result.len() > maximum {
        return Err(io::Error::other("Linux /proc observation bound exceeded"));
    }
    Ok(result)
}

fn full_stdout_pipe() -> (Stdio, Option<File>) {
    let mut descriptors = [-1; 2];
    // SAFETY: the array provides space for both returned descriptors. On success
    // they are immediately adopted by separate owning values and remain private.
    assert_eq!(
        unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) },
        0,
        "{}",
        io::Error::last_os_error()
    );
    // SAFETY: pipe2 just returned these unique, valid descriptors.
    let reader = unsafe { OwnedFd::from_raw_fd(descriptors[0]) };
    // SAFETY: the write descriptor is distinct and also uniquely owned.
    let writer = unsafe { OwnedFd::from_raw_fd(descriptors[1]) };
    // SAFETY: fcntl operates on our still-owned pipe descriptor. Shrinking to
    // one page requires no privilege and keeps the test's fill work bounded.
    let capacity = unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETPIPE_SZ, 4096) };
    assert!(capacity > 0 && capacity <= 65_536);
    // SAFETY: F_GETFL and F_SETFL use our valid descriptor and integer flags.
    let flags = unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_GETFL) };
    assert!(flags >= 0);
    assert_eq!(
        unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) },
        0
    );
    let mut writer = File::from(writer);
    let buffer = [b'x'; 4096];
    let mut written = 0;
    loop {
        match writer.write(&buffer) {
            Ok(0) => panic!("pipe accepted no filler bytes"),
            Ok(count) => {
                written += count;
                assert!(written <= capacity as usize);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => panic!("could not prepare a full stdout pipe: {error}"),
        }
    }
    assert_eq!(written, capacity as usize);
    // SAFETY: restore the same owned descriptor's original blocking flags before
    // handing it to the child. The parent keeps the read endpoint open and idle.
    assert_eq!(
        unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETFL, flags) },
        0
    );
    (Stdio::from(writer), Some(File::from(reader)))
}
