//! Bounded actual-command observations over the existing public pool fixtures.
//! No helper below changes the frozen fixtures or adds a production test hook.

#[path = "../support_operator_v2/mod.rs"]
mod existing;

pub use existing::state::{Scratch, assert_complete_state, checkpoint, control_root, snapshot};
pub use existing::{OperatorProcess, command};

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File},
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::fs::MetadataExt,
    },
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::{Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::ChainV2;
use mirror_core::v2::SignedOperationV2;
use mirror_node::v2::NodeControlV2;
use mirror_storage::v2::ReplacementStoreV2;

static PROCESSES: Mutex<()> = Mutex::new(());
const OBSERVATION_LIMIT: Duration = Duration::from_secs(8);
const MAX_CHILD_FDS: usize = 128;

pub fn process_guard() -> MutexGuard<'static, ()> {
    // Every test in this executable participates. In particular, a directory
    // owner held by one test cannot be inherited by another test's child.
    PROCESSES.lock().unwrap_or_else(|error| error.into_inner())
}

pub struct Fixture {
    pub scratch: Scratch,
    pub chain: ChainV2,
    pub input: PathBuf,
}

impl Fixture {
    pub fn new(label: &str, chain: ChainV2) -> Self {
        let scratch = Scratch::new(label);
        drop(ReplacementStoreV2::initialize(scratch.root(), &chain).unwrap());
        drop(
            NodeControlV2::initialize(
                control_root(&scratch),
                scratch.root(),
                chain.manifest().identity(),
                checkpoint(&chain, 0),
            )
            .unwrap(),
        );
        let input = scratch.0.join("synthetic-private-input.hex");
        fs::write(&input, []).unwrap();
        Self {
            scratch,
            chain,
            input,
        }
    }

    pub fn write(&self, bytes: &[u8]) {
        fs::write(&self.input, bytes).unwrap();
    }

    pub fn arguments(&self) -> Vec<OsString> {
        arguments(&self.scratch, &self.chain, &self.input)
    }

    pub fn invoke(&self, arguments: &[OsString]) -> Output {
        let output = unchanged_command(&self.scratch, arguments);
        self.assert_reopened();
        output
    }

    pub fn assert_reopened(&self) {
        let before = snapshot(&self.scratch.0);
        let owner = NodeControlV2::open_writer(
            control_root(&self.scratch),
            self.scratch.root(),
            self.chain.manifest().identity(),
        )
        .unwrap();
        assert_eq!(owner.checkpoint(), checkpoint(&self.chain, 0));
        assert_complete_state(owner.chain(), &self.chain);
        drop(owner);
        // Strict A1 opening synchronizes existing records. Equality establishes
        // unchanged application contents/selection, not absence of fsync.
        assert_eq!(snapshot(&self.scratch.0), before);
    }
}

pub fn arguments(scratch: &Scratch, chain: &ChainV2, input: &Path) -> Vec<OsString> {
    let identity = chain.manifest().identity();
    vec![
        "pool-inspect".into(),
        "--control-dir".into(),
        control_root(scratch).into_os_string(),
        "--store-dir".into(),
        scratch.root().into_os_string(),
        "--config-hash".into(),
        identity.config_hash.to_string().into(),
        "--genesis-hash".into(),
        identity.genesis_hash.to_string().into(),
        "--input-file".into(),
        input.as_os_str().to_owned(),
        "--max-input-bytes".into(),
        "149248".into(),
        "--max-input-count".into(),
        "256".into(),
        "--max-entries".into(),
        "256".into(),
        "--max-encoded-bytes".into(),
        "74496".into(),
        "--max-batch-operations".into(),
        "64".into(),
        "--max-batch-bytes".into(),
        "19048".into(),
        "--max-work-units".into(),
        "67108864".into(),
        "--evaluate-count".into(),
        "0".into(),
    ]
}

pub fn set(arguments: &mut [OsString], flag: &str, value: impl Into<OsString>) {
    let index = arguments
        .iter()
        .position(|argument| argument == flag)
        .unwrap();
    arguments[index + 1] = value.into();
}

pub fn remove(arguments: &mut Vec<OsString>, flag: &str) {
    let index = arguments
        .iter()
        .position(|argument| argument == flag)
        .unwrap();
    arguments.drain(index..index + 2);
}

pub fn record(bytes: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = Vec::with_capacity(bytes.len() * 2 + 1);
    for &byte in bytes {
        result.push(HEX[usize::from(byte >> 4)]);
        result.push(HEX[usize::from(byte & 15)]);
    }
    result.push(b'\n');
    result
}

pub fn records(operations: &[SignedOperationV2]) -> Vec<u8> {
    operations
        .iter()
        .flat_map(|operation| record(&operation.encode()))
        .collect()
}

pub fn unchanged_command(scratch: &Scratch, arguments: &[OsString]) -> Output {
    let before = snapshot(&scratch.0);
    let output = command(arguments);
    bounded(&output);
    assert_eq!(snapshot(&scratch.0), before);
    output
}

pub fn bounded(output: &Output) {
    assert!(output.stdout.len() <= 4096, "unbounded pool stdout");
    assert!(output.stderr.len() <= 4096, "unbounded pool stderr");
    for bytes in [&output.stdout, &output.stderr] {
        assert!(bytes.is_empty() || bytes.last() == Some(&b'\n'));
        let text = std::str::from_utf8(bytes).expect("UTF8 pool output");
        assert!(!text.contains("synthetic-private"));
    }
}

pub fn failure(output: &Output, code: i32, error: &str) -> BTreeMap<String, String> {
    bounded(output);
    let values = existing::failure(output, code);
    assert_eq!(values["error"], error);
    assert_eq!(
        values["operation"],
        if code == 2 { "parse" } else { "pool-inspect" }
    );
    let mut keys = BTreeSet::from(["format", "operation", "error", "next"]);
    if code == 2 {
        keys.insert("reason");
        assert_eq!(values["next"], "help");
    }
    if error == "output_io" {
        keys.insert("operation_may_have_completed");
        assert_eq!(values["operation_may_have_completed"], "true");
        assert_eq!(values["next"], "inspect-status");
    }
    assert_eq!(
        values.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        keys
    );
    values
}

pub fn report(
    output: &Output,
    code: i32,
    fixture: &Fixture,
    arguments: &[OsString],
) -> BTreeMap<String, String> {
    bounded(output);
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let values = existing::fields(&output.stdout);
    let keys = [
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
    ];
    assert_eq!(values.len(), 31);
    assert_eq!(
        values.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from(keys)
    );
    assert_eq!(values["format"], "mirror-node-v2-1");
    assert_eq!(values["operation"], "pool-inspect");
    assert_eq!(values["status"], "inspected");
    assert_eq!(values["store_selection"], "confirmed");
    let identity = fixture.chain.manifest().identity();
    assert_eq!(values["config_hash"], identity.config_hash.to_string());
    assert_eq!(values["genesis_hash"], identity.genesis_hash.to_string());
    let selected = checkpoint(&fixture.chain, 0);
    assert_eq!(
        values["selected_generation"],
        selected.generation.to_string()
    );
    assert_eq!(values["selected_height"], selected.chain.height.to_string());
    assert_eq!(
        values["selected_tip_hash"],
        selected.chain.tip_hash.to_string()
    );
    assert_eq!(
        values["selected_ledger_root"],
        selected.chain.ledger_root.to_string()
    );
    assert_eq!(values["canonical_inclusion"], "not-checked");
    assert_eq!(values["pool_disposition"], "discarded");
    for (flag, field) in [
        ("--max-input-bytes", "max_input_bytes"),
        ("--max-input-count", "max_input_count"),
        ("--max-entries", "max_entries"),
        ("--max-encoded-bytes", "max_encoded_bytes"),
        ("--max-batch-operations", "max_batch_operations"),
        ("--max-batch-bytes", "max_batch_bytes"),
        ("--max-work-units", "max_work_units"),
        ("--evaluate-count", "evaluation_requested_count"),
    ] {
        let index = arguments
            .iter()
            .position(|argument| argument == flag)
            .unwrap();
        assert_eq!(values[field], arguments[index + 1].to_str().unwrap());
    }
    values
}

pub fn counts(values: &BTreeMap<String, String>, expected: [u64; 6]) {
    for (key, expected) in [
        "input_bytes",
        "input_count",
        "retained_count",
        "duplicate_count",
        "retained_encoded_bytes",
        "work_used",
    ]
    .into_iter()
    .zip(expected)
    {
        assert_eq!(values[key], expected.to_string(), "{key}");
    }
}

pub fn evaluation(
    values: &BTreeMap<String, String>,
    outcome: &str,
    reason: &str,
    index: Option<usize>,
    count: usize,
    bytes: usize,
) {
    assert_eq!(values["evaluation"], outcome);
    assert_eq!(values["evaluation_reason"], reason);
    assert_eq!(
        values["evaluation_operation_index"],
        index.map_or_else(|| "none".to_owned(), |index| index.to_string())
    );
    assert_eq!(values["evaluated_count"], count.to_string());
    assert_eq!(values["evaluated_block_bytes"], bytes.to_string());
}

/// Owns only an actual operator child and its deliberately full stdout pipe.
/// Linux /proc establishes a stable final-write phase, not intake timing.
pub struct BlockedOutputProcess {
    child: Option<Child>,
    reader: Option<File>,
}

impl BlockedOutputProcess {
    pub fn spawn(arguments: &[OsString]) -> Self {
        let (stdout, reader) = full_stdout_pipe();
        let child = Command::new(env!("CARGO_BIN_EXE_mirror-node-v2"))
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(Stdio::piped())
            .spawn()
            .expect("start actual pool command with full stdout pipe");
        assert_ne!(child.id(), std::process::id());
        Self {
            child: Some(child),
            reader: Some(reader),
        }
    }

    pub fn wait_stdout_blocked(&mut self) {
        let deadline = Instant::now() + OBSERVATION_LIMIT;
        loop {
            if self.child.as_mut().unwrap().try_wait().unwrap().is_some() {
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "pool command exited before observed stdout write: {} {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            let pid = self.child.as_ref().unwrap().id();
            let call = syscall(pid).expect("direct-parent Linux /proc syscall observation");
            if call.is_some_and(|(number, fd, length)| {
                (number == libc::SYS_write || number == libc::SYS_writev) && fd == 1 && length > 0
            }) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "pool final stdout write was not observed; last syscall={call:?}"
            );
            thread::sleep(Duration::from_millis(2));
        }
    }

    pub fn assert_paths_released(&self, paths: &[&Path]) {
        let identities: Vec<_> = paths
            .iter()
            .map(|path| {
                let metadata = fs::metadata(path).unwrap();
                (metadata.dev(), metadata.ino())
            })
            .collect();
        let pid = self.child.as_ref().unwrap().id();
        for (index, entry) in fs::read_dir(format!("/proc/{pid}/fd")).unwrap().enumerate() {
            assert!(
                index < MAX_CHILD_FDS,
                "bounded child descriptor observation"
            );
            let entry = entry.unwrap();
            let metadata = match fs::metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => panic!("cannot observe child descriptors: {error}"),
            };
            assert!(
                !identities.contains(&(metadata.dev(), metadata.ino())),
                "child retained input or an authority root during stdout write"
            );
        }
    }

    pub fn close_reader(&mut self) {
        assert!(self.reader.take().is_some());
    }

    pub fn finish(mut self) -> Output {
        let deadline = Instant::now() + existing::PROCESS_LIMIT;
        loop {
            if self.child.as_mut().unwrap().try_wait().unwrap().is_some() {
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                bounded(&output);
                return output;
            }
            if Instant::now() >= deadline {
                self.child.as_mut().unwrap().kill().unwrap();
                let output = self.child.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "pool process exceeded parent deadline: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub fn kill(mut self) -> Output {
        self.child.as_mut().unwrap().kill().unwrap();
        let output = self.child.take().unwrap().wait_with_output().unwrap();
        bounded(&output);
        output
    }
}

impl Drop for BlockedOutputProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn syscall(pid: u32) -> io::Result<Option<(libc::c_long, u64, u64)>> {
    let mut text = String::new();
    File::open(format!("/proc/{pid}/syscall"))?
        .take(4097)
        .read_to_string(&mut text)?;
    if text.len() > 4096 {
        return Err(io::Error::other("bounded syscall observation exceeded"));
    }
    let fields = text.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 9 {
        // "running" and the shorter userspace record establish no syscall phase.
        return Ok(None);
    }
    let number = fields[0]
        .parse()
        .map_err(|_| io::Error::other("invalid syscall number"))?;
    let argument = |index: usize| {
        u64::from_str_radix(fields[index].trim_start_matches("0x"), 16)
            .map_err(|_| io::Error::other("invalid syscall argument"))
    };
    Ok(Some((number, argument(1)?, argument(3)?)))
}

fn full_stdout_pipe() -> (Stdio, File) {
    let mut descriptors = [-1; 2];
    // SAFETY: two writable slots receive unique descriptors on success.
    assert_eq!(
        unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) },
        0
    );
    // SAFETY: pipe2 just returned each distinct descriptor, adopted exactly once.
    let reader = unsafe { OwnedFd::from_raw_fd(descriptors[0]) };
    // SAFETY: the distinct write descriptor is likewise uniquely owned.
    let writer = unsafe { OwnedFd::from_raw_fd(descriptors[1]) };
    // SAFETY: each fcntl uses our live descriptor and the matching integer argument.
    let capacity = unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETPIPE_SZ, 4096) };
    assert!(capacity > 0 && capacity <= 65_536);
    // SAFETY: F_GETFL reads flags on the still-owned descriptor.
    let flags = unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_GETFL) };
    assert!(flags >= 0);
    // SAFETY: F_SETFL receives the existing flags plus the documented nonblock flag.
    assert_eq!(
        unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) },
        0
    );
    let mut writer = File::from(writer);
    let mut written = 0;
    loop {
        match writer.write(&[b'x'; 4096]) {
            Ok(0) => panic!("stdout pipe accepted no filler"),
            Ok(count) => {
                written += count;
                assert!(written <= capacity as usize);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => panic!("cannot fill private stdout pipe: {error}"),
        }
    }
    assert_eq!(written, capacity as usize);
    // SAFETY: restore the original blocking flags before handing stdout to child.
    assert_eq!(
        unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETFL, flags) },
        0
    );
    (Stdio::from(writer), File::from(reader))
}
