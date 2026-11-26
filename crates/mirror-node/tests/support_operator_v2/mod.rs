//! Bounded process/socket helpers for the finite V2 operator acceptance tests.
#![allow(dead_code, unused_imports)]

#[path = "../support_control_v2/mod.rs"]
pub mod state;

use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    os::{fd::OwnedFd, unix::net::UnixStream},
    process::{Child, Command, Output, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use mirror_chain::v2::ChainV2;
use mirror_network::v2::{
    HistoryFramesV2,
    session::{
        MAX_SESSION_WIRE_BYTES_V2, PeerSessionsV2, REQUEST_LEN_V2, SessionLimitsV2,
        validate_history_request_v2,
    },
};
use mirror_protocol::v2::{FRAME_HEADER_LEN_V2, FrameRefV2, encode_frame_v2};

use state::history::{fixture_bytes, independent_identity};

pub const PROCESS_LIMIT: Duration = Duration::from_secs(20);

pub struct OperatorProcess {
    child: Option<Child>,
}

impl OperatorProcess {
    pub fn spawn(arguments: &[OsString]) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_mirror-node-v2"))
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start actual V2 operator binary");
        assert_ne!(child.id(), std::process::id());
        Self { child: Some(child) }
    }

    pub fn spawn_closed_stdout(arguments: &[OsString]) -> Self {
        let (reader, writer) = UnixStream::pair().unwrap();
        // Close the receiving endpoint BEFORE launch: no scheduling assumption
        // determines whether the command observes its broken output channel.
        drop(reader);
        let child = Command::new(env!("CARGO_BIN_EXE_mirror-node-v2"))
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::from(OwnedFd::from(writer)))
            .stderr(Stdio::piped())
            .spawn()
            .expect("start actual V2 operator with a closed stdout peer");
        Self { child: Some(child) }
    }

    pub fn running(&mut self) -> bool {
        self.child.as_mut().unwrap().try_wait().unwrap().is_none()
    }

    pub fn finish(mut self) -> Output {
        let deadline = Instant::now() + PROCESS_LIMIT;
        loop {
            if self.child.as_mut().unwrap().try_wait().unwrap().is_some() {
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                assert!(output.stdout.len() <= 16_384, "unbounded operator stdout");
                assert!(output.stderr.len() <= 16_384, "unbounded operator stderr");
                return output;
            }
            if Instant::now() >= deadline {
                self.child.as_mut().unwrap().kill().unwrap();
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "operator exceeded its process deadline: {}",
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

impl Drop for OperatorProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn command(arguments: &[OsString]) -> Output {
    OperatorProcess::spawn(arguments).finish()
}

pub fn fields(bytes: &[u8]) -> BTreeMap<String, String> {
    let text = std::str::from_utf8(bytes).expect("UTF8 operator output");
    let mut result = BTreeMap::new();
    for line in text.lines() {
        let (key, value) = line.split_once('=').expect("one key=value per output line");
        assert!(!key.is_empty() && !value.is_empty());
        assert!(
            result.insert(key.to_owned(), value.to_owned()).is_none(),
            "duplicate output key"
        );
    }
    result
}

pub fn success(output: &Output, expected_code: i32, operation: &str) -> BTreeMap<String, String> {
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let values = fields(&output.stdout);
    assert_eq!(values["format"], "mirror-node-v2-1");
    assert_eq!(values["operation"], operation);
    assert_eq!(
        values["config_hash"],
        independent_identity().config_hash.to_string()
    );
    assert_eq!(
        values["genesis_hash"],
        independent_identity().genesis_hash.to_string()
    );
    values
}

pub fn failure(output: &Output, expected_code: i32) -> BTreeMap<String, String> {
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "failure emitted success data");
    let values = fields(&output.stderr);
    assert_eq!(values["format"], "mirror-node-v2-1");
    assert!(!values["operation"].is_empty());
    assert!(!values["error"].is_empty());
    assert!(matches!(
        values["next"].as_str(),
        "help" | "inspect-status" | "explicit-recover" | "verify-inputs"
    ));
    values
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerMode {
    Good,
    EarlyClose,
    MissingEnd,
    Trailing,
    InvalidBlock,
    HoldAfterEnd,
    HoldBeforeResponse,
    RequestOnly,
}

#[derive(Debug)]
pub struct PeerResult {
    pub request_bytes: usize,
    pub wire_bytes: u64,
}

pub struct Peer {
    address: SocketAddr,
    task: Option<JoinHandle<Result<PeerResult, String>>>,
    release: Sender<()>,
    ready: Receiver<()>,
}

impl Peer {
    pub fn spawn(chain: ChainV2, mode: PeerMode) -> Self {
        // Listening begins before the operator process is launched. No chosen-
        // then-released port, readiness sleep, discovery or address race is used.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (release, released) = mpsc::channel();
        let (ready_tx, ready) = mpsc::channel();
        let task = thread::spawn(move || {
            if mode == PeerMode::Good {
                let limits =
                    SessionLimitsV2::new(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_secs(5))
                        .map_err(|error| error.to_string())?;
                let mut owner = PeerSessionsV2::new(limits);
                owner
                    .serve_once(listener, independent_identity(), &chain)
                    .map_err(|error| error.to_string())?;
                return Ok(PeerResult {
                    request_bytes: REQUEST_LEN_V2,
                    wire_bytes: owner.stats().wire_bytes,
                });
            }
            raw_peer(listener, chain, mode, ready_tx, released).map_err(|error| error.to_string())
        });
        Self {
            address,
            task: Some(task),
            release,
            ready,
        }
    }

    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn wait_ready(&self) {
        self.ready
            .recv_timeout(Duration::from_secs(8))
            .expect("peer reached held-request/End phase");
    }

    pub fn finish(mut self) -> PeerResult {
        let _ = self.release.send(());
        self.task
            .take()
            .unwrap()
            .join()
            .expect("peer thread did not panic")
            .expect("bounded synthetic peer completed")
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.release.send(());
        if let Some(task) = self.task.take() {
            let _ = task.join();
        }
    }
}

fn accept(listener: TcpListener) -> io::Result<TcpStream> {
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "synthetic peer accept",
                    ));
                }
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => return Err(error),
        }
    }
}

fn raw_peer(
    listener: TcpListener,
    chain: ChainV2,
    mode: PeerMode,
    ready: Sender<()>,
    release: Receiver<()>,
) -> Result<PeerResult, Box<dyn std::error::Error>> {
    // Deliberately adversarial test provider; production uses PeerSessionsV2.
    let mut stream = accept(listener)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    if mode == PeerMode::RequestOnly {
        let mut request = Vec::new();
        stream
            .take((REQUEST_LEN_V2 + 1) as u64)
            .read_to_end(&mut request)?;
        return Ok(PeerResult {
            request_bytes: request.len(),
            wire_bytes: request.len() as u64,
        });
    }
    let mut request = [0; REQUEST_LEN_V2];
    stream.read_exact(&mut request)?;
    validate_history_request_v2(&request, independent_identity())?;
    let mut sentinel = [0; 1];
    assert_eq!(
        stream.read(&mut sentinel)?,
        0,
        "request did not end before response"
    );
    if mode == PeerMode::HoldBeforeResponse {
        ready.send(())?;
        release.recv_timeout(PROCESS_LIMIT)?;
        return Ok(PeerResult {
            request_bytes: REQUEST_LEN_V2,
            wire_bytes: REQUEST_LEN_V2 as u64,
        });
    }
    let frames = HistoryFramesV2::new(&chain)?.collect::<Result<Vec<_>, _>>()?;
    let mut response = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        if mode == PeerMode::MissingEnd && index == frames.len() - 1 {
            break;
        }
        if mode == PeerMode::EarlyClose && index == 2 {
            response.extend_from_slice(&frame[..FRAME_HEADER_LEN_V2 + 9]);
            break;
        }
        if mode == PeerMode::InvalidBlock && index == 3 {
            response.extend_from_slice(&encode_frame_v2(FrameRefV2::Block {
                height: 1,
                bytes: &fixture_bytes("wrong_root1_block"),
            })?);
        } else {
            response.extend_from_slice(frame);
        }
    }
    if mode == PeerMode::Trailing {
        response.push(0x7f);
    }
    if let Err(error) = stream.write_all(&response) {
        if !matches!(
            error.kind(),
            io::ErrorKind::BrokenPipe
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::ConnectionAborted
        ) {
            return Err(error.into());
        }
    }
    if mode == PeerMode::HoldAfterEnd {
        ready.send(())?;
        // The parent releases only AFTER the operator has returned its timeout.
        // No arbitrary sleep is used to induce the missing terminal EOF.
        release.recv_timeout(PROCESS_LIMIT)?;
    }
    match stream.shutdown(Shutdown::Write) {
        Ok(()) => {}
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotConnected | io::ErrorKind::BrokenPipe
            ) => {}
        Err(error) => return Err(error.into()),
    }
    Ok(PeerResult {
        request_bytes: REQUEST_LEN_V2,
        wire_bytes: (REQUEST_LEN_V2 + response.len()) as u64,
    })
}
