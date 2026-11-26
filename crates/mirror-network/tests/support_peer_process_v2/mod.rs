//! Bounded subprocess harness for public synthetic peer-session evidence.
//!
//! This module is used only by tests and the finite example. It is not a node,
//! discovery service, process supervisor, or source of production identity pins.
#![allow(dead_code)]

#[path = "../support_history_v2/mod.rs"]
pub mod history;

use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use mirror_chain::v2::ChainV2;
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::BlockV2;
use mirror_network::v2::{
    HistoryFramesV2,
    session::{MAX_SESSION_WIRE_BYTES_V2, PeerSessionsV2, REQUEST_LEN_V2, SessionLimitsV2},
};
use mirror_protocol::v2::{FRAME_HEADER_LEN_V2, FrameRefV2, encode_frame_v2};
use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementStoreV2};

use history::{fixture_bytes, fixture_hash, fork_chain, independent_identity, main_chain};

pub const CHILD_MODE: &str = "MIRROR_V2_SYNTHETIC_PEER_MODE";
pub const CHILD_READY: &str = "MIRROR_V2_SYNTHETIC_PEER_READY";
const PROCESS_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for _ in 0..32 {
            let path = std::env::temp_dir().join(format!(
                "mirror-v2-{label}-{}-{now}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("cannot create synthetic test directory: {error}"),
            }
        }
        panic!("synthetic temporary-directory namespace exhausted");
    }

    pub fn root(&self) -> PathBuf {
        self.0.join("selected-store")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // This parent was freshly created by the harness. Never clean a caller's
        // data path, nor a live root still owned by a store or child process.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A child is killed and reaped before its privately created files are removed,
/// including during an assertion unwind. Every readiness/exit wait is bounded.
pub struct PeerChild {
    child: Option<Child>,
    scratch: Scratch,
    address: SocketAddr,
}

impl PeerChild {
    pub fn spawn(arguments: &[&str], mode: &str) -> Self {
        let scratch = Scratch::new("peer-process");
        let ready = scratch.0.join("ready");
        let child = Command::new(std::env::current_exe().unwrap())
            .args(arguments)
            .env(CHILD_MODE, mode)
            .env(CHILD_READY, &ready)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        assert_ne!(child.id(), std::process::id());
        let mut owner = Self {
            child: Some(child),
            scratch,
            address: "127.0.0.1:0".parse().unwrap(),
        };
        let deadline = Instant::now() + PROCESS_TIMEOUT;
        loop {
            match fs::read_to_string(&ready) {
                Ok(text) => {
                    owner.address = text.trim().parse().expect("atomic child readiness address");
                    assert!(owner.address.ip().is_loopback());
                    assert_ne!(owner.address.port(), 0);
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => panic!("child readiness could not be read: {error}"),
            }
            if let Some(status) = owner.child.as_mut().unwrap().try_wait().unwrap() {
                panic!("synthetic provider exited before readiness: {status}");
            }
            assert!(
                Instant::now() < deadline,
                "synthetic provider readiness timed out"
            );
            thread::sleep(Duration::from_millis(5));
        }
        owner
    }

    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn finish(mut self) {
        let deadline = Instant::now() + PROCESS_TIMEOUT;
        loop {
            if let Some(status) = self.child.as_mut().unwrap().try_wait().unwrap() {
                self.child.take();
                assert!(status.success(), "synthetic provider failed: {status}");
                return;
            }
            assert!(
                Instant::now() < deadline,
                "synthetic provider exit timed out"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for PeerChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn sessions(timeout: Duration) -> PeerSessionsV2 {
    PeerSessionsV2::new(SessionLimitsV2::new(1, MAX_SESSION_WIRE_BYTES_V2, timeout).unwrap())
}

pub fn checkpoint(chain: &ChainV2, generation: u64) -> ReplacementCheckpointV2 {
    ReplacementCheckpointV2 {
        generation,
        chain: chain.checkpoint(),
    }
}

pub fn preferred_fork() -> ChainV2 {
    let mut chain = fork_chain();
    let proposal = chain
        .prepare_candidate(chain.blocks().last().unwrap().header().timestamp() + 1, &[])
        .unwrap();
    let mined = mine_header_v2(
        *proposal.header(),
        chain.config().parameters().pow_bits,
        10_000,
    )
    .unwrap();
    let block = BlockV2::new(chain.config(), mined.header, proposal.operations()).unwrap();
    chain.append(block).unwrap();
    chain
}

pub fn assert_complete_state(actual: &ChainV2, expected: &ChainV2) {
    assert_eq!(actual, expected);
    assert_eq!(actual.manifest(), expected.manifest());
    assert_eq!(actual.blocks(), expected.blocks());
    assert_eq!(
        actual.ledger().account_bytes().unwrap(),
        expected.ledger().account_bytes().unwrap()
    );
    assert_eq!(
        actual.ledger().membership().encode().unwrap(),
        expected.ledger().membership().encode().unwrap()
    );
    assert_eq!(actual.checkpoint(), expected.checkpoint());
    assert_eq!(actual.encoded_bytes(), expected.encoded_bytes());
    assert_eq!(actual.state_items_used(), expected.state_items_used());
    for id in [fixture_hash("c_member_id"), fixture_hash("seed0_id")] {
        assert_eq!(
            actual.ledger().membership().record(id),
            expected.ledger().membership().record(id)
        );
    }
    let actual_members = actual.ledger().membership();
    let expected_members = expected.ledger().membership();
    assert_eq!(
        actual_members.active_count(),
        expected_members.active_count()
    );
    assert_eq!(
        actual_members.pending_count(),
        expected_members.pending_count()
    );
    assert_eq!(
        actual_members.active_high_water(),
        expected_members.active_high_water()
    );
    assert_eq!(
        actual_members.total_activations(),
        expected_members.total_activations()
    );
    assert_eq!(
        actual_members.retained_approval_count(),
        expected_members.retained_approval_count()
    );
}

/// Include directory names as well as exact payloads in no-mutation assertions.
pub fn store_bytes(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(root: &Path, relative: &Path, result: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(root.join(relative)).unwrap() {
            let entry = entry.unwrap();
            let path = relative.join(entry.file_name());
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                result.insert(path.clone(), None);
                walk(root, &path, result);
            } else {
                assert!(
                    kind.is_file(),
                    "synthetic store contains unexpected path type"
                );
                result.insert(path, Some(fs::read(entry.path()).unwrap()));
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, Path::new(""), &mut result);
    result
}

pub fn run_child() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::var(CHILD_MODE)?;
    let ready = PathBuf::from(std::env::var_os(CHILD_READY).ok_or("missing child readiness path")?);
    let chain = match mode.as_str() {
        "fork" => fork_chain(),
        "preferred-fork" => preferred_fork(),
        _ => main_chain(4),
    };
    // The real provider owns a separate, explicitly initialized synthetic store.
    // No sender takes a mutable borrow while this complete history is transmitted.
    let source =
        ReplacementStoreV2::initialize(ready.parent().unwrap().join("source-store"), &chain)?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let pending = ready.with_extension("pending");
    fs::write(&pending, listener.local_addr()?.to_string())?;
    fs::rename(pending, &ready)?;
    match mode.as_str() {
        "main" | "fork" | "preferred-fork" => {
            let mut owner = sessions(Duration::from_secs(5));
            owner.serve_once(listener, independent_identity(), source.chain()?)?;
            assert_eq!(owner.stats().attempts, 1);
        }
        _ => malicious_or_fragmented_peer(listener, source.chain()?, &mode)?,
    }
    drop(source);
    Ok(())
}

fn malicious_or_fragmented_peer(
    listener: TcpListener,
    chain: &ChainV2,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Adversarial fixture code deliberately bypasses the production session API.
    // Parent process guards bound it even if the target implementation regresses.
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err("adversarial fixture accept timed out".into());
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(error.into()),
        }
    };
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut request = [0u8; REQUEST_LEN_V2];
    stream.read_exact(&mut request)?;
    assert_eq!(&request[..4], b"MPQ2");
    assert_eq!(&request[4..6], &2u16.to_le_bytes());
    assert_eq!(&request[6..8], &1u16.to_le_bytes());
    assert_eq!(
        &request[8..40],
        independent_identity().config_hash.as_bytes()
    );
    assert_eq!(
        &request[40..72],
        independent_identity().genesis_hash.as_bytes()
    );
    let mut sentinel = [0u8; 1];
    assert_eq!(
        stream.read(&mut sentinel)?,
        0,
        "requester did not half-close"
    );

    if mode == "stall" {
        stream.write_all(b"MPN")?;
        // Request EOF is already observed. A short, finite delay keeps the
        // response open past the client's deadline, without waiting on input.
        thread::sleep(Duration::from_millis(350));
        return Ok(());
    }
    if mode == "oversized" {
        let mut header = [0u8; FRAME_HEADER_LEN_V2];
        header[..4].copy_from_slice(b"MPN2");
        header[4..6].copy_from_slice(&2u16.to_le_bytes());
        header[6..8].copy_from_slice(&1u16.to_le_bytes());
        header[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        stream.write_all(&header)?;
        // No declared body is supplied. Admission must reject this header,
        // rather than waiting for the fixture to close or its deadline to expire.
        thread::sleep(Duration::from_millis(350));
        return Ok(());
    }

    for (index, frame) in HistoryFramesV2::new(chain)?.enumerate() {
        let mut frame = frame?;
        if mode == "early-close" && index == 3 {
            stream.write_all(&frame[..FRAME_HEADER_LEN_V2 + 9])?;
            return Ok(());
        }
        if mode == "changed-end" && index == chain.blocks().len() + 2 {
            frame[FRAME_HEADER_LEN_V2 + 72] ^= 1;
        }
        if mode == "invalid-block" && index == 3 {
            frame = encode_frame_v2(FrameRefV2::Block {
                height: 1,
                bytes: &fixture_bytes("wrong_root1_block"),
            })?;
        }
        let result = if mode == "fragmented" {
            write_fragmented(&mut stream, &frame)
        } else {
            stream.write_all(&frame)
        };
        match result {
            Ok(()) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
                ) =>
            {
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        }
    }
    match stream.shutdown(Shutdown::Write) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotConnected => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn write_fragmented(stream: &mut TcpStream, frame: &[u8]) -> io::Result<()> {
    // Headers cross separate writes; payloads cross a different chunk size.
    // TCP may coalesce writes, so the receiver must make no packet-boundary claim.
    for byte in &frame[..FRAME_HEADER_LEN_V2] {
        stream.write_all(&[*byte])?;
        thread::sleep(Duration::from_micros(100));
    }
    for chunk in frame[FRAME_HEADER_LEN_V2..].chunks(61) {
        stream.write_all(chunk)?;
        thread::sleep(Duration::from_micros(100));
    }
    Ok(())
}
