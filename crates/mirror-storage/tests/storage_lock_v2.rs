//! Actual store ownership across independent Linux processes and descriptors.
#![cfg(target_os = "linux")]

mod support_v2;

use std::{
    fs,
    io::{self, BufRead, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use mirror_storage::v2::{StorageErrorV2, StoreAccessV2, StoreV2};
use support_v2::{fixture_identity, main_chain, main_expectation};

const CHILD_ROOT: &str = "MIRROR_STORAGE_V2_LOCK_PROBE_ROOT";
const CHILD_ACCESS: &str = "MIRROR_STORAGE_V2_LOCK_PROBE_ACCESS";
const READY: &str = "MIRROR_STORAGE_V2_LOCK_READY";
const DROPPED: &str = "MIRROR_STORAGE_V2_LOCK_DROPPED";
const READY_TIMEOUT: Duration = Duration::from_secs(20);

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

// An unrelated parallel test's Command::spawn can briefly inherit every open
// directory descriptor between fork and exec, even when it is close-on-exec.
// Dropping the last parent handle then need not close the last kernel reference.
// Isolate only these parent cases so another case's spawn cannot prolong their
// lock lifetime. Each process test still exercises genuine concurrent processes
// and descriptors. There are no production retries, sleeps, or unlock changes.
static PARENT_TEST_LOCK: Mutex<()> = Mutex::new(());

fn isolate_parent_test() -> MutexGuard<'static, ()> {
    PARENT_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct TestDirectory {
    parent: PathBuf,
    store: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let parent = std::env::temp_dir().join(format!(
            "mirror-storage-v2-lock-{}-{nanos}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&parent).unwrap();
        let store = parent.join("store");
        Self { parent, store }
    }

    fn initialize(&self) -> StoreV2 {
        StoreV2::initialize(&self.store, &main_chain(3)).unwrap()
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        // Only this test's newly created temporary parent is removed. Store
        // handles and child guards are declared later, so they drop first.
        let _ = fs::remove_dir_all(&self.parent);
    }
}

fn writer(root: &Path) -> Result<StoreV2, StorageErrorV2> {
    StoreV2::open_writer(root, fixture_identity(), main_expectation(3))
}

fn reader(root: &Path) -> Result<StoreV2, StorageErrorV2> {
    StoreV2::inspect(root, fixture_identity(), main_expectation(3))
}

fn assert_busy(result: Result<StoreV2, StorageErrorV2>) {
    assert!(
        matches!(result, Err(StorageErrorV2::Busy)),
        "independent store handle must report Busy"
    );
}

/// A pipe handshake, not a scheduling delay, proves that a child has its lock.
/// The guard also kills/waits the helper if an assertion in the parent fails.
struct ChildHolder {
    child: Option<Child>,
    messages: Receiver<String>,
    output_reader: Option<JoinHandle<()>>,
}

impl ChildHolder {
    fn spawn(root: &Path, access: StoreAccessV2) -> Self {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "lock_process_probe",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_ROOT, root)
            .env(
                CHILD_ACCESS,
                match access {
                    StoreAccessV2::Writer => "writer",
                    StoreAccessV2::ReadOnly => "reader",
                },
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, messages) = mpsc::channel();
        let output_reader = thread::spawn(move || {
            for result in io::BufReader::new(stdout).lines() {
                let line = match result {
                    Ok(line) => line,
                    Err(error) => format!("child output I/O error: {error}"),
                };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let mut holder = Self {
            child: Some(child),
            messages,
            output_reader: Some(output_reader),
        };
        holder.await_message(READY);
        holder
    }

    fn await_message(&mut self, expected: &str) {
        let deadline = std::time::Instant::now() + READY_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let message = self
                .messages
                .recv_timeout(remaining)
                .unwrap_or_else(|error| {
                    panic!("child did not report {expected} before its deadline: {error}")
                });
            if message == expected {
                break;
            }
        }
    }

    fn send(&mut self, command: &str) {
        let input = self.child.as_mut().unwrap().stdin.as_mut().unwrap();
        writeln!(input, "{command}").unwrap();
        input.flush().unwrap();
    }

    fn drop_store(&mut self) {
        self.send("DROP");
        self.await_message(DROPPED);
        assert!(self.child.as_mut().unwrap().try_wait().unwrap().is_none());
    }

    fn finish(mut self) {
        self.send("EXIT");
        self.child.as_mut().unwrap().stdin.take();
        assert!(self.child.as_mut().unwrap().wait().unwrap().success());
        self.child.take();
        self.output_reader.take().unwrap().join().unwrap();
    }

    fn kill_and_wait(mut self) {
        let child = self.child.as_mut().unwrap();
        child.kill().unwrap();
        assert!(!child.wait().unwrap().success());
        self.child.take();
        self.output_reader.take().unwrap().join().unwrap();
    }
}

impl Drop for ChildHolder {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(reader) = self.output_reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
#[ignore = "subprocess harness entry; exercised by parent process tests"]
fn lock_process_probe() {
    let Some(root) = std::env::var_os(CHILD_ROOT) else {
        return;
    };
    let root = PathBuf::from(root);
    let store = match std::env::var(CHILD_ACCESS).unwrap().as_str() {
        "writer" => writer(&root).unwrap(),
        "reader" => reader(&root).unwrap(),
        other => panic!("unknown child lock mode: {other}"),
    };
    assert_eq!(store.chain().checkpoint(), main_expectation(3));
    println!("\n{READY}");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    assert_eq!(command.trim_end(), "DROP");
    drop(store);
    println!("{DROPPED}");
    io::stdout().flush().unwrap();

    command.clear();
    io::stdin().read_line(&mut command).unwrap();
    assert_eq!(command.trim_end(), "EXIT");
}

#[test]
fn writer_excludes_independent_handles_and_drop_releases() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    let store = directory.initialize();
    assert_eq!(store.access(), StoreAccessV2::Writer);
    assert_busy(writer(&directory.store));
    assert_busy(reader(&directory.store));
    drop(store);

    let reopened = writer(&directory.store).unwrap();
    assert_eq!(reopened.chain().checkpoint(), main_expectation(3));
    assert_eq!(reopened.access(), StoreAccessV2::Writer);
}

#[test]
fn shared_handles_exclude_writer_until_every_reader_drops() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize());
    let first = reader(&directory.store).unwrap();
    let second = reader(&directory.store).unwrap();
    assert_eq!(first.access(), StoreAccessV2::ReadOnly);
    assert_eq!(second.access(), StoreAccessV2::ReadOnly);
    assert_busy(writer(&directory.store));
    drop(first);
    assert_busy(writer(&directory.store));
    drop(second);
    assert_eq!(
        writer(&directory.store).unwrap().chain().checkpoint(),
        main_expectation(3)
    );
}

#[test]
fn writer_in_another_process_releases_on_store_drop_while_process_lives() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize());
    let mut child = ChildHolder::spawn(&directory.store, StoreAccessV2::Writer);
    assert_busy(writer(&directory.store));
    assert_busy(reader(&directory.store));
    child.drop_store();
    let reopened = writer(&directory.store).unwrap();
    assert_eq!(reopened.chain().checkpoint(), main_expectation(3));
    child.finish();
}

#[test]
fn killed_writer_releases_without_unlinking_or_repairing_anything() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize());
    let before = fs::symlink_metadata(&directory.store).unwrap();
    let mut names_before: Vec<_> = fs::read_dir(&directory.store)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names_before.sort();

    let child = ChildHolder::spawn(&directory.store, StoreAccessV2::Writer);
    assert_busy(writer(&directory.store));
    child.kill_and_wait();

    let reopened = writer(&directory.store).unwrap();
    assert_eq!(reopened.chain().checkpoint(), main_expectation(3));
    let after = fs::symlink_metadata(&directory.store).unwrap();
    assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    let mut names_after: Vec<_> = fs::read_dir(&directory.store)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names_after.sort();
    assert_eq!(names_after, names_before);
}

#[test]
fn shared_locks_in_separate_processes_coexist_and_exclude_a_writer() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize());
    let mut child = ChildHolder::spawn(&directory.store, StoreAccessV2::ReadOnly);
    let local = reader(&directory.store).unwrap();
    assert_busy(writer(&directory.store));
    drop(local);
    assert_busy(writer(&directory.store));
    child.drop_store();
    let reopened = writer(&directory.store).unwrap();
    assert_eq!(reopened.chain().checkpoint(), main_expectation(3));
    child.finish();
}
