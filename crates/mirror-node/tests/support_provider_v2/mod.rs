//! Linux-only, bounded observation of actual provider command processes.
//!
//! The production executable exposes no readiness or test hook. These helpers
//! correlate the specific child's socket inode, descriptor and blocked syscall;
//! a parent-side TCP handshake alone is never used as application admission.
#![allow(dead_code, unused_imports)]

#[path = "../support_operator_v2/mod.rs"]
mod existing;

pub use existing::{command, failure, fields, state, success};

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    io::{self, Read, Write},
    net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::net::UnixStream,
    },
    path::Path,
    process::{Child, Command, Output, Stdio},
    sync::{Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::ChainV2;
use mirror_network::v2::{
    HistoryFramesV2,
    session::{MAX_SESSION_WIRE_BYTES_V2, PeerSessionsV2, SessionLimitsV2},
};
use mirror_node::v2::NodeControlV2;
use mirror_storage::v2::{ReplacementStorageErrorV2, ReplacementStoreV2, StorageErrorV2};

use state::{
    Scratch, checkpoint, control_root,
    history::{independent_identity, main_chain},
};

pub const PROCESS_LIMIT: Duration = Duration::from_secs(20);
const OBSERVATION_LIMIT: Duration = Duration::from_secs(8);
const SOCKET_LIMIT: Duration = Duration::from_secs(5);
const MAX_RESPONSE_BYTES: usize = 65_536;
const MAX_PROC_BYTES: usize = 1_048_576;
const MAX_CHILD_FDS: usize = 128;

pub fn process_guard() -> MutexGuard<'static, ()> {
    static PROCESSES: Mutex<()> = Mutex::new(());
    PROCESSES.lock().unwrap_or_else(|error| error.into_inner())
}

pub fn ready(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    let chain = main_chain(3);
    drop(ReplacementStoreV2::initialize(scratch.root(), &chain).unwrap());
    drop(
        NodeControlV2::initialize(
            control_root(&scratch),
            scratch.root(),
            independent_identity(),
            checkpoint(&chain, 0),
        )
        .unwrap(),
    );
    scratch
}

pub fn common(operation: &str, scratch: &Scratch) -> Vec<OsString> {
    vec![
        operation.into(),
        "--control-dir".into(),
        control_root(scratch).into_os_string(),
        "--store-dir".into(),
        scratch.root().into_os_string(),
        "--config-hash".into(),
        independent_identity().config_hash.to_string().into(),
        "--genesis-hash".into(),
        independent_identity().genesis_hash.to_string().into(),
    ]
}

pub fn arguments(scratch: &Scratch, address: SocketAddr) -> Vec<OsString> {
    let mut result = common("serve", scratch);
    result.extend([
        "--listen".into(),
        address.to_string().into(),
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

pub fn set(arguments: &mut [OsString], flag: &str, value: impl Into<OsString>) {
    let index = arguments.iter().position(|value| value == flag).unwrap();
    arguments[index + 1] = value.into();
}

pub fn remove(arguments: &mut Vec<OsString>, flag: &str) {
    let index = arguments.iter().position(|value| value == flag).unwrap();
    arguments.drain(index..index + 2);
}

pub fn reserve() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}

pub fn assert_no_connection(listener: &TcpListener) {
    match listener.accept() {
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
        other => panic!("a locally rejected command contacted its peer: {other:?}"),
    }
}

pub fn assert_port_closed(address: SocketAddr) {
    match TcpStream::connect_timeout(&address, Duration::from_millis(300)) {
        Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {}
        other => panic!("provider listener survived termination: {other:?}"),
    }
}

pub fn assert_pins(values: &BTreeMap<String, String>) {
    let expected = checkpoint(&main_chain(3), 0);
    assert_eq!(
        values["selected_generation"],
        expected.generation.to_string()
    );
    assert_eq!(values["selected_height"], expected.chain.height.to_string());
    assert_eq!(
        values["selected_tip_hash"],
        expected.chain.tip_hash.to_string()
    );
    assert_eq!(
        values["selected_ledger_root"],
        expected.chain.ledger_root.to_string()
    );
}

pub fn assert_reopened(scratch: &Scratch) {
    let values = success(&command(&common("status", scratch)), 0, "status");
    assert_eq!(values["status"], "clean");
    assert_eq!(values["store_selection"], "verified");
    assert_pins(&values);
}

pub fn assert_busy(scratch: &Scratch) {
    assert_eq!(
        failure(&command(&common("status", scratch)), 1)["error"],
        "busy"
    );
    let unused = reserve();
    let mut sync = common("sync-once", scratch);
    sync.extend([
        "--peer".into(),
        unused.local_addr().unwrap().to_string().into(),
        "--timeout-ms".into(),
        "1000".into(),
        "--max-io-bytes".into(),
        MAX_SESSION_WIRE_BYTES_V2.to_string().into(),
    ]);
    assert_eq!(failure(&command(&sync), 1)["error"], "busy");
    assert_no_connection(&unused);
    match ReplacementStoreV2::open_writer(
        scratch.root(),
        independent_identity(),
        checkpoint(&main_chain(3), 0),
    ) {
        Err(ReplacementStorageErrorV2::Storage(StorageErrorV2::Busy)) => {}
        Err(error) => panic!("store refusal did not prove retained ownership: {error:?}"),
        Ok(_) => panic!("provider did not retain storage ownership"),
    }
}

pub fn stopped(
    output: &Output,
    reason: &str,
    attempts: u8,
    completed: u8,
    rejected: u8,
    wire_bytes: u64,
    last_error: &str,
) -> BTreeMap<String, String> {
    assert!(output.stdout.len() <= 4096);
    let values = success(output, 0, "serve");
    assert_eq!(
        values.len(),
        16,
        "unexpected fields in final provider record"
    );
    assert_eq!(values["status"], "stopped");
    assert_eq!(values["store_selection"], "verified");
    assert_pins(&values);
    assert_eq!(values["reason"], reason);
    assert_eq!(values["attempts"], attempts.to_string());
    assert_eq!(values["completed_sessions"], completed.to_string());
    assert_eq!(values["rejected_sessions"], rejected.to_string());
    assert_eq!(values["wire_bytes"], wire_bytes.to_string());
    assert_eq!(values["last_session_error"], last_error);
    assert_eq!(completed + rejected, attempts);
    values
}

pub fn connect(address: SocketAddr) -> TcpStream {
    let stream = TcpStream::connect_timeout(&address, SOCKET_LIMIT).unwrap();
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(SOCKET_LIMIT)).unwrap();
    stream.set_write_timeout(Some(SOCKET_LIMIT)).unwrap();
    stream
}

pub fn transcript(chain: &ChainV2) -> Vec<u8> {
    let mut bytes = Vec::new();
    for frame in HistoryFramesV2::new(chain).unwrap() {
        bytes.extend_from_slice(&frame.unwrap());
    }
    assert!(bytes.len() < MAX_RESPONSE_BYTES);
    bytes
}

pub fn read_response(stream: &mut TcpStream) -> Vec<u8> {
    let mut result = Vec::new();
    let mut buffer = [0u8; 4096];
    let deadline = Instant::now() + SOCKET_LIMIT;
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .expect("provider response exceeded its original receive deadline");
        stream.set_read_timeout(Some(remaining)).unwrap();
        let read = stream.read(&mut buffer);
        assert!(
            Instant::now() < deadline,
            "provider response exceeded its original receive deadline"
        );
        match read {
            Ok(0) => return result,
            Ok(count) => {
                assert!(
                    result.len() + count <= MAX_RESPONSE_BYTES,
                    "synthetic provider response exceeded its fixture bound"
                );
                result.extend_from_slice(&buffer[..count]);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
                ) =>
            {
                return result;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                ) =>
            {
                // An early kernel timeout, interruption or short read cannot
                // renew this helper's single absolute receive allowance.
            }
            Err(error) => panic!("provider response did not finish within socket bound: {error}"),
        }
    }
}

pub fn raw_request(address: SocketAddr, request: &[u8]) -> Vec<u8> {
    let mut stream = connect(address);
    stream.write_all(request).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    read_response(&mut stream)
}

pub fn no_response_yet(stream: &mut TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_millis(120)))
        .unwrap();
    let mut byte = [0u8; 1];
    match stream.read(&mut byte) {
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
            ) => {}
        other => panic!("a queued stream received application data or closed early: {other:?}"),
    }
    stream.set_read_timeout(Some(SOCKET_LIMIT)).unwrap();
}

pub fn receive_complete(address: SocketAddr) -> (ChainV2, u64) {
    let mut requester = PeerSessionsV2::new(
        SessionLimitsV2::new(1, MAX_SESSION_WIRE_BYTES_V2, SOCKET_LIMIT).unwrap(),
    );
    let candidate = requester.request(address, independent_identity()).unwrap();
    (candidate.chain().clone(), requester.stats().wire_bytes)
}

enum OutputMode {
    Captured,
    Closed,
    Blocked,
}

pub struct ProviderProcess {
    child: Option<Child>,
    address: SocketAddr,
    output_reader: Option<File>,
}

impl ProviderProcess {
    pub fn spawn(arguments: &[OsString], reservation: TcpListener) -> Self {
        Self::spawn_with_output(arguments, reservation, OutputMode::Captured)
    }

    pub fn spawn_closed_stdout(arguments: &[OsString], reservation: TcpListener) -> Self {
        Self::spawn_with_output(arguments, reservation, OutputMode::Closed)
    }

    pub fn spawn_blocked_stdout(arguments: &[OsString], reservation: TcpListener) -> Self {
        Self::spawn_with_output(arguments, reservation, OutputMode::Blocked)
    }

    fn spawn_with_output(
        arguments: &[OsString],
        reservation: TcpListener,
        mode: OutputMode,
    ) -> Self {
        let address = reservation.local_addr().unwrap();
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
        let child = command.spawn().expect("start actual V2 provider command");
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
        assert!(self.running(), "provider ended before the chosen signal");
        let pid = i32::try_from(self.child.as_ref().unwrap().id()).unwrap();
        // SAFETY: this is the positive PID of our live, still-owned Child; only
        // the explicitly selected graceful signal is sent by these tests.
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
                    "provider exited before {label}: status={} stdout={} stderr={}",
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
                assert!(output.stdout.len() <= 4096, "unbounded provider stdout");
                assert!(output.stderr.len() <= 4096, "unbounded provider stderr");
                return output;
            }
            if Instant::now() >= deadline {
                self.child.as_mut().unwrap().kill().unwrap();
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "provider exceeded parent process deadline: {}",
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

impl Drop for ProviderProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
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
