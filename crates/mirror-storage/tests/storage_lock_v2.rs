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

use mirror_storage::v2::{
    AppendCheckpointV2, AppendRecoveryActionV2, AppendRecoveryErrorV2, AppendRecoveryRequestV2,
    AppendStorageErrorV2, AppendStoreV2, StorageErrorV2, StoreAccessV2, StoreV2,
};
use support_v2::{fixture_identity, main_chain, main_expectation};

const CHILD_ROOT: &str = "MIRROR_STORAGE_V2_LOCK_PROBE_ROOT";
const CHILD_ACCESS: &str = "MIRROR_STORAGE_V2_LOCK_PROBE_ACCESS";
const CHILD_PROFILE: &str = "MIRROR_STORAGE_V2_LOCK_PROBE_PROFILE";
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

    fn initialize_append(&self) -> AppendStoreV2 {
        AppendStoreV2::initialize(&self.store, &main_chain(3)).unwrap()
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

fn append_checkpoint(height: u64, generation: u64) -> AppendCheckpointV2 {
    AppendCheckpointV2 {
        generation,
        chain: main_expectation(height),
    }
}

fn append_writer(root: &Path) -> Result<AppendStoreV2, AppendStorageErrorV2> {
    AppendStoreV2::open_writer(root, fixture_identity(), append_checkpoint(3, 0))
}

fn append_reader(root: &Path) -> Result<AppendStoreV2, AppendStorageErrorV2> {
    AppendStoreV2::inspect(root, fixture_identity(), append_checkpoint(3, 0))
}

fn assert_append_busy(result: Result<AppendStoreV2, AppendStorageErrorV2>) {
    assert!(
        matches!(
            result,
            Err(AppendStorageErrorV2::Storage(StorageErrorV2::Busy))
        ),
        "independent append-profile handle must report Busy"
    );
}

fn append_and_reopen_fourth_block(root: &Path, mut store: AppendStoreV2) {
    let expected = main_chain(4);
    assert_eq!(
        store.append(expected.block_at(4).unwrap().clone()).unwrap(),
        append_checkpoint(4, 1),
    );
    assert_eq!(store.chain().unwrap(), &expected);
    drop(store);
    let reopened =
        AppendStoreV2::open_writer(root, fixture_identity(), append_checkpoint(4, 1)).unwrap();
    assert_eq!(reopened.chain().unwrap(), &expected);
}

fn recovery_request() -> AppendRecoveryRequestV2 {
    AppendRecoveryRequestV2::new(
        append_checkpoint(2, 0),
        append_checkpoint(3, 1),
        main_chain(3).block_at(3).unwrap().clone(),
    )
    .unwrap()
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
        Self::spawn_profile(root, access, "immutable")
    }

    fn spawn_profile(root: &Path, access: StoreAccessV2, profile: &str) -> Self {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "lock_process_probe",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_ROOT, root)
            .env(CHILD_PROFILE, profile)
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
    enum ProbeStore {
        Immutable(StoreV2),
        Append(AppendStoreV2),
        Recovered(AppendStoreV2),
    }
    let access = std::env::var(CHILD_ACCESS).unwrap();
    let profile = std::env::var(CHILD_PROFILE).unwrap_or_else(|_| "immutable".into());
    let store = match (profile.as_str(), access.as_str()) {
        ("immutable", "writer") => ProbeStore::Immutable(writer(&root).unwrap()),
        ("immutable", "reader") => ProbeStore::Immutable(reader(&root).unwrap()),
        ("append", "writer") => ProbeStore::Append(append_writer(&root).unwrap()),
        ("append", "reader") => ProbeStore::Append(append_reader(&root).unwrap()),
        ("recovery", "writer") => ProbeStore::Recovered(
            AppendStoreV2::recover_interrupted(
                &root,
                fixture_identity(),
                &recovery_request(),
                AppendRecoveryActionV2::Complete,
            )
            .unwrap(),
        ),
        other => panic!("unknown child lock profile/mode: {other:?}"),
    };
    match &store {
        ProbeStore::Immutable(store) => {
            assert_eq!(store.chain().checkpoint(), main_expectation(3));
        }
        ProbeStore::Append(store) => {
            assert_eq!(store.chain().unwrap(), &main_chain(3));
            assert_eq!(store.checkpoint().unwrap(), append_checkpoint(3, 0));
        }
        ProbeStore::Recovered(store) => {
            assert_eq!(store.chain().unwrap(), &main_chain(3));
            assert_eq!(store.checkpoint().unwrap(), append_checkpoint(3, 1));
            assert_eq!(store.access(), StoreAccessV2::Writer);
            assert!(!store.is_fenced());
        }
    }
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

#[test]
fn append_writer_child_excludes_handles_until_drop_then_parent_can_commit() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize_append());
    let mut child = ChildHolder::spawn_profile(&directory.store, StoreAccessV2::Writer, "append");
    assert_append_busy(append_writer(&directory.store));
    assert_append_busy(append_reader(&directory.store));
    child.drop_store();
    let reopened = append_writer(&directory.store).unwrap();
    assert_eq!(reopened.chain().unwrap(), &main_chain(3));
    append_and_reopen_fourth_block(&directory.store, reopened);
    child.finish();
}

#[test]
fn killed_append_writer_releases_root_for_checked_reopen_and_append() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize_append());
    let inode = fs::metadata(&directory.store).unwrap();
    let selected_before = fs::read(directory.store.join("selection.v2")).unwrap();
    let child = ChildHolder::spawn_profile(&directory.store, StoreAccessV2::Writer, "append");
    assert_append_busy(append_writer(&directory.store));
    child.kill_and_wait();
    let reopened = append_writer(&directory.store).unwrap();
    assert_eq!(reopened.chain().unwrap(), &main_chain(3));
    assert_eq!(
        fs::read(directory.store.join("selection.v2")).unwrap(),
        selected_before
    );
    let reopened_inode = fs::metadata(&directory.store).unwrap();
    assert_eq!(
        (inode.dev(), inode.ino()),
        (reopened_inode.dev(), reopened_inode.ino())
    );
    append_and_reopen_fourth_block(&directory.store, reopened);
}

#[test]
fn append_shared_child_and_local_reader_exclude_writer_until_both_drop() {
    let _isolation = isolate_parent_test();
    let directory = TestDirectory::new();
    drop(directory.initialize_append());
    let mut child = ChildHolder::spawn_profile(&directory.store, StoreAccessV2::ReadOnly, "append");
    let local = append_reader(&directory.store).unwrap();
    assert_eq!(local.chain().unwrap(), &main_chain(3));
    assert_append_busy(append_writer(&directory.store));
    drop(local);
    assert_append_busy(append_writer(&directory.store));
    child.drop_store();
    let reopened = append_writer(&directory.store).unwrap();
    append_and_reopen_fourth_block(&directory.store, reopened);
    child.finish();
}

#[test]
fn recovered_child_process_keeps_exclusive_ownership_until_drop_or_termination() {
    let _isolation = isolate_parent_test();
    for terminate in [false, true] {
        let directory = TestDirectory::new();
        drop(AppendStoreV2::initialize(&directory.store, &main_chain(2)).unwrap());
        let inode = fs::metadata(&directory.store).unwrap();
        let expected = main_chain(3);
        let candidate = expected
            .block_at(3)
            .unwrap()
            .encode(expected.config())
            .unwrap();
        let tail = directory.store.join("blocks/00000000000000000003.mrb2");
        fs::write(&tail, &candidate[..100]).unwrap();

        // READY is emitted only after the child completes explicit recovery and
        // retains the returned writer. A pipe handshake proves this boundary;
        // no scheduling sleeps or weaker ownership assertions are used.
        let mut child =
            ChildHolder::spawn_profile(&directory.store, StoreAccessV2::Writer, "recovery");
        assert_eq!(fs::read(&tail).unwrap(), candidate);
        assert!(!directory.store.join("selection.pending").exists());
        assert!(matches!(
            AppendStoreV2::recover_interrupted(
                &directory.store,
                fixture_identity(),
                &recovery_request(),
                AppendRecoveryActionV2::Complete,
            ),
            Err(AppendRecoveryErrorV2::Storage(StorageErrorV2::Busy))
        ));
        assert_append_busy(AppendStoreV2::open_writer(
            &directory.store,
            fixture_identity(),
            append_checkpoint(3, 1),
        ));
        assert_append_busy(AppendStoreV2::inspect(
            &directory.store,
            fixture_identity(),
            append_checkpoint(3, 1),
        ));

        let living_child = if terminate {
            child.kill_and_wait();
            None
        } else {
            child.drop_store();
            Some(child)
        };
        let mut reopened = AppendStoreV2::open_writer(
            &directory.store,
            fixture_identity(),
            append_checkpoint(3, 1),
        )
        .unwrap();
        assert_eq!(reopened.chain().unwrap(), &expected);
        let current_inode = fs::metadata(&directory.store).unwrap();
        assert_eq!(
            (inode.dev(), inode.ino()),
            (current_inode.dev(), current_inode.ino())
        );
        let next = main_chain(4);
        assert_eq!(
            reopened.append(next.block_at(4).unwrap().clone()).unwrap(),
            append_checkpoint(4, 2)
        );
        assert_eq!(reopened.chain().unwrap(), &next);
        drop(reopened);
        let final_store = AppendStoreV2::open_writer(
            &directory.store,
            fixture_identity(),
            append_checkpoint(4, 2),
        )
        .unwrap();
        assert_eq!(final_store.chain().unwrap(), &next);
        if let Some(child) = living_child {
            child.finish();
        }
    }
}
