//! Real process termination and fresh-process control recovery evidence.
//!
//! The private callback only places an actual process at a named completed I/O
//! boundary. The parent kills and reaps it before a new process opens the same
//! paths. These cases do not simulate physical power loss or hostile local users.
#![cfg(target_os = "linux")]

#[path = "../../tests/support_control_v2/mod.rs"]
mod support;

use std::{
    fs, io,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::ForkChoiceV2;
use mirror_storage::v2::{ReplacementRecoveryActionV2, ReplacementStoreV2};

use super::*;
use support::{
    Scratch, assert_complete_state, checkpoint, control_root,
    history::{fork_chain, independent_identity, main_chain},
    preferred_fork, received, snapshot,
};

const HELPER: &str = "v2::control_process_tests::control_process_child";
const MODE: &str = "MIRROR_NODE_CONTROL_TEST_MODE";
const PARENT: &str = "MIRROR_NODE_CONTROL_TEST_PARENT";
const READY: &str = "MIRROR_NODE_CONTROL_TEST_READY";
const BOUNDARY: &str = "MIRROR_NODE_CONTROL_TEST_BOUNDARY";
const PROCESS_LIMIT: Duration = Duration::from_secs(15);

// Unrelated forks must not inherit another parent test's live directory locks.
// Child processes acquire their own descriptors after exec; the parent never
// holds a node/store writer while it launches a subprocess.
pub(super) static PROCESSES: Mutex<()> = Mutex::new(());

struct ControlChild {
    child: Option<Child>,
    ready: PathBuf,
    _scratch: Scratch,
}

impl ControlChild {
    fn spawn(parent: &Path, mode: &str, boundary: &str) -> Self {
        let scratch = Scratch::new("node-child-readiness");
        let ready = scratch.0.join("ready");
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", HELPER, "--ignored", "--nocapture"])
            .env(MODE, mode)
            .env(PARENT, parent)
            .env(READY, &ready)
            .env(BOUNDARY, boundary)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        assert_ne!(child.id(), std::process::id());
        Self {
            child: Some(child),
            ready,
            _scratch: scratch,
        }
    }

    fn wait_ready(&mut self, boundary: &str) {
        let deadline = Instant::now() + PROCESS_LIMIT;
        loop {
            match fs::read_to_string(&self.ready) {
                Ok(message) => {
                    assert_eq!(message.trim(), boundary);
                    return;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => panic!("readiness failed: {error}"),
            }
            if let Some(status) = self.child.as_mut().unwrap().try_wait().unwrap() {
                panic!("node child exited before {boundary}: {status}");
            }
            assert!(Instant::now() < deadline, "node child readiness timed out");
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn finish(mut self) {
        let deadline = Instant::now() + PROCESS_LIMIT;
        loop {
            if let Some(status) = self.child.as_mut().unwrap().try_wait().unwrap() {
                self.child.take();
                assert!(status.success(), "node child failed: {status}");
                return;
            }
            assert!(Instant::now() < deadline, "node child exit timed out");
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn kill_at(mut self, boundary: &str) {
        self.wait_ready(boundary);
        self.child.as_mut().unwrap().kill().unwrap();
        let status = self.child.as_mut().unwrap().wait().unwrap();
        self.child.take();
        assert_eq!(status.signal(), Some(9), "expected actual SIGKILL");
    }
}

impl Drop for ControlChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn pause_at_boundary(boundary: ControlBoundaryV2) -> Result<(), NodeControlErrorV2> {
    let requested = std::env::var(BOUNDARY).unwrap();
    let observed = format!("{boundary:?}");
    if observed == requested {
        signal_and_park(&observed);
    }
    Ok(())
}

fn signal_and_park(message: &str) -> ! {
    let ready = PathBuf::from(std::env::var_os(READY).unwrap());
    let pending = ready.with_extension("pending");
    fs::write(&pending, message).unwrap();
    fs::rename(pending, ready).unwrap();
    let deadline = Instant::now() + PROCESS_LIMIT;
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    panic!("parent did not terminate the parked node child");
}

fn run(parent: &Path, mode: &str) {
    ControlChild::spawn(parent, mode, "").finish();
}

fn kill(parent: &Path, mode: &str, boundary: &str) {
    ControlChild::spawn(parent, mode, boundary).kill_at(boundary);
}

fn selected_store(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    drop(ReplacementStoreV2::initialize(scratch.root(), &main_chain(3)).unwrap());
    scratch
}

fn initialized(label: &str) -> Scratch {
    let scratch = selected_store(label);
    // A distinct process creates the confirmed record from independent pins.
    run(&scratch.0, "initialize");
    scratch
}

fn verify_recovery_status(control: &Path, store: &Path) {
    let old = checkpoint(&main_chain(3), 0);
    let candidate = checkpoint(&preferred_fork(), 1);
    let before = (snapshot(control), snapshot(store));
    match NodeControlV2::status(control, store, independent_identity()).unwrap() {
        NodeStatusV2::RecoveryRequired {
            confirmed,
            old: actual_old,
            candidate: actual_candidate,
        } => {
            assert!(confirmed == old || confirmed == candidate);
            assert_eq!(actual_old, old);
            assert_eq!(actual_candidate, candidate);
        }
        other => panic!("unresolved intent was not reported: {other:?}"),
    }
    assert!(NodeControlV2::open_writer(control, store, independent_identity()).is_err());
    assert_eq!((snapshot(control), snapshot(store)), before);
}

fn verify_clean(control: &Path, store: &Path, candidate: bool) {
    let expected = if candidate {
        preferred_fork()
    } else {
        main_chain(3)
    };
    let pinned = checkpoint(&expected, u64::from(candidate));
    let before = (snapshot(control), snapshot(store));
    assert_eq!(
        NodeControlV2::status(control, store, independent_identity()).unwrap(),
        NodeStatusV2::Clean { checkpoint: pinned },
    );
    assert_eq!((snapshot(control), snapshot(store)), before);
    let node = NodeControlV2::open_writer(control, store, independent_identity()).unwrap();
    assert_eq!(node.checkpoint(), pinned);
    assert_complete_state(node.chain(), &expected);
}

#[test]
#[ignore = "invoked only by bounded parent subprocess tests"]
fn control_process_child() {
    let Ok(mode) = std::env::var(MODE) else {
        return;
    };
    let parent = PathBuf::from(std::env::var_os(PARENT).unwrap());
    let control = parent.join("control");
    let store = parent.join("selected-store");
    let identity = independent_identity();
    match mode.as_str() {
        "initialize" => {
            let node = NodeControlV2::initialize(
                &control,
                &store,
                identity,
                checkpoint(&main_chain(3), 0),
            )
            .unwrap();
            assert_complete_state(node.chain(), &main_chain(3));
        }
        "crash-initialize" => {
            initialize_inner(
                &control,
                &store,
                identity,
                checkpoint(&main_chain(3), 0),
                pause_at_boundary,
            )
            .unwrap();
            panic!("requested initialization boundary was not reached");
        }
        "crash-adopt" => {
            let node = NodeControlV2::open_writer(&control, &store, identity).unwrap();
            node.adopt_inner(received(&preferred_fork()), pause_at_boundary)
                .unwrap();
            panic!("requested adoption boundary was not reached");
        }
        "crash-recover" => {
            recover_inner(
                &control,
                &store,
                identity,
                ReplacementRecoveryActionV2::Complete,
                pause_at_boundary,
            )
            .unwrap();
            panic!("requested recovery boundary was not reached");
        }
        "recovery-status" => verify_recovery_status(&control, &store),
        "clean-old" => verify_clean(&control, &store, false),
        "clean-candidate" => verify_clean(&control, &store, true),
        "reject-partial" => {
            let before = (snapshot(&control), snapshot(&store));
            assert!(NodeControlV2::status(&control, &store, identity).is_err());
            assert!(NodeControlV2::open_writer(&control, &store, identity).is_err());
            for action in [
                ReplacementRecoveryActionV2::Complete,
                ReplacementRecoveryActionV2::RetainOld,
            ] {
                assert!(NodeControlV2::recover(&control, &store, identity, action).is_err());
                assert_eq!((snapshot(&control), snapshot(&store)), before);
            }
        }
        "reject-retain" => {
            let before = (snapshot(&control), snapshot(&store));
            assert!(
                NodeControlV2::recover(
                    &control,
                    &store,
                    identity,
                    ReplacementRecoveryActionV2::RetainOld
                )
                .is_err()
            );
            assert_eq!((snapshot(&control), snapshot(&store)), before);
        }
        "complete" | "retain" => {
            let candidate = mode == "complete";
            let action = if candidate {
                ReplacementRecoveryActionV2::Complete
            } else {
                ReplacementRecoveryActionV2::RetainOld
            };
            let node = NodeControlV2::recover(&control, &store, identity, action).unwrap();
            let expected = if candidate {
                preferred_fork()
            } else {
                main_chain(3)
            };
            assert_eq!(
                node.checkpoint(),
                checkpoint(&expected, u64::from(candidate))
            );
            assert_complete_state(node.chain(), &expected);
        }
        "adopt" | "keep-tie" => {
            let node = NodeControlV2::open_writer(&control, &store, identity).unwrap();
            let candidate = if mode == "adopt" {
                preferred_fork()
            } else {
                fork_chain()
            };
            let outcome = node.adopt(received(&candidate)).unwrap();
            assert_eq!(
                outcome.report().choice,
                if mode == "adopt" {
                    ForkChoiceV2::PreferCandidate
                } else {
                    ForkChoiceV2::KeepCurrent
                }
            );
            let node = outcome.into_control();
            let expected = if mode == "adopt" {
                candidate
            } else {
                main_chain(3)
            };
            assert_complete_state(node.chain(), &expected);
        }
        "hold-owner" => {
            let _node = NodeControlV2::open_writer(&control, &store, identity).unwrap();
            signal_and_park("owner");
        }
        "reject-competing" => {
            let before = (snapshot(&control), snapshot(&store));
            assert!(NodeControlV2::status(&control, &store, identity).is_err());
            assert!(NodeControlV2::open_writer(&control, &store, identity).is_err());
            assert!(
                NodeControlV2::recover(
                    &control,
                    &store,
                    identity,
                    ReplacementRecoveryActionV2::Complete
                )
                .is_err()
            );
            assert_eq!((snapshot(&control), snapshot(&store)), before);
        }
        _ => panic!("unexpected synthetic child mode: {mode}"),
    }
}

#[test]
fn separate_process_clean_restarts_preserve_tie_and_preferred_membership_history() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = initialized("node-clean-restart");
    run(&scratch.0, "clean-old");
    let before = (snapshot(&control_root(&scratch)), snapshot(&scratch.root()));
    run(&scratch.0, "keep-tie");
    assert_eq!(
        (snapshot(&control_root(&scratch)), snapshot(&scratch.root())),
        before
    );
    run(&scratch.0, "adopt");
    run(&scratch.0, "clean-candidate");
}

#[test]
fn killed_partial_intent_writes_reject_and_preserve_store_and_control() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in [
        "IntentCreated",
        "IntentPrefixWritten",
        "IntentWritten",
        "IntentFileSynced",
        "IntentValidated",
    ] {
        let scratch = initialized("node-partial-intent");
        let before = snapshot(&scratch.root());
        kill(&scratch.0, "crash-adopt", boundary);
        assert_eq!(
            snapshot(&scratch.root()),
            before,
            "store changed at {boundary}"
        );
        run(&scratch.0, "reject-partial");
        assert_eq!(snapshot(&scratch.root()), before);
    }
}

#[test]
fn killed_published_intent_restarts_require_explicit_completion_or_retention() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in ["IntentPublished", "IntentDirectorySynced"] {
        for action in ["complete", "retain"] {
            let scratch = initialized("node-published-intent");
            let before = snapshot(&scratch.root());
            kill(&scratch.0, "crash-adopt", boundary);
            assert_eq!(snapshot(&scratch.root()), before);
            run(&scratch.0, "recovery-status");
            run(&scratch.0, action);
            run(
                &scratch.0,
                if action == "complete" {
                    "clean-candidate"
                } else {
                    "clean-old"
                },
            );
            if action == "retain" {
                assert_eq!(snapshot(&scratch.root()), before);
            }
        }
    }
}

#[test]
fn killed_after_store_publication_uses_persisted_histories_and_never_rolls_back() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = initialized("node-store-confirmation-gap");
    kill(&scratch.0, "crash-adopt", "StorePublicationCompleted");
    run(&scratch.0, "recovery-status");
    run(&scratch.0, "reject-retain");
    run(&scratch.0, "complete");
    run(&scratch.0, "clean-candidate");
}

#[test]
fn killed_confirmed_record_writes_allow_only_candidate_completion() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in [
        "ConfirmedCreated",
        "ConfirmedPrefixWritten",
        "ConfirmedWritten",
        "ConfirmedFileSynced",
        "ConfirmedPublished",
        "ConfirmedDirectorySynced",
    ] {
        let scratch = initialized("node-confirmed-publication");
        kill(&scratch.0, "crash-adopt", boundary);
        run(&scratch.0, "recovery-status");
        run(&scratch.0, "reject-retain");
        run(&scratch.0, "complete");
        run(&scratch.0, "clean-candidate");
    }
}

#[test]
fn killed_intent_retirement_has_durable_candidate_confirmation_for_clean_restart() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in ["IntentRemoved", "IntentRetirementSynced"] {
        let scratch = initialized("node-intent-retirement");
        kill(&scratch.0, "crash-adopt", boundary);
        run(&scratch.0, "clean-candidate");
    }
}

#[test]
fn recovery_itself_survives_another_process_termination_without_losing_authority() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in [
        "StorePublicationCompleted",
        "ConfirmedPrefixWritten",
        "ConfirmedPublished",
        "IntentRemoved",
    ] {
        let scratch = initialized("node-recovery-restart");
        kill(&scratch.0, "crash-adopt", "IntentDirectorySynced");
        kill(&scratch.0, "crash-recover", boundary);
        if boundary == "IntentRemoved" {
            run(&scratch.0, "clean-candidate");
        } else {
            run(&scratch.0, "recovery-status");
            run(&scratch.0, "reject-retain");
            run(&scratch.0, "complete");
            run(&scratch.0, "clean-candidate");
        }
    }
}

#[test]
fn killed_recovery_resynchronization_preserves_both_explicit_action_choices() {
    let _serial = PROCESSES.lock().unwrap();
    for action in ["complete", "retain"] {
        let scratch = initialized("node-recovery-resync");
        let before = snapshot(&scratch.root());
        kill(&scratch.0, "crash-adopt", "IntentDirectorySynced");
        kill(&scratch.0, "crash-recover", "IntentResynchronized");
        assert_eq!(snapshot(&scratch.root()), before);
        run(&scratch.0, "recovery-status");
        run(&scratch.0, action);
        run(
            &scratch.0,
            if action == "complete" {
                "clean-candidate"
            } else {
                "clean-old"
            },
        );
    }
}

#[test]
fn twice_killed_confirmation_prefix_extension_remains_exact_and_completable() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = initialized("node-confirmation-prefix-retry");
    kill(&scratch.0, "crash-adopt", "ConfirmedPrefixWritten");
    let path = control_root(&scratch).join("confirmed.pending");
    assert_eq!(
        fs::metadata(&path).unwrap().len() as usize,
        records::CONTROL_RECORD_LEN_V2 / 2,
    );
    kill(&scratch.0, "crash-recover", "ConfirmedPrefixWritten");
    assert_eq!(
        fs::metadata(&path).unwrap().len() as usize,
        3 * records::CONTROL_RECORD_LEN_V2 / 4,
    );
    run(&scratch.0, "recovery-status");
    run(&scratch.0, "reject-retain");
    run(&scratch.0, "complete");
    run(&scratch.0, "clean-candidate");
}

#[test]
fn killed_partial_control_initialization_rejects_without_touching_the_selected_store() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in [
        "InitializedCreated",
        "InitializedPrefixWritten",
        "InitializedWritten",
        "InitializedFileSynced",
    ] {
        let scratch = selected_store("node-initialization-partial");
        let before = snapshot(&scratch.root());
        kill(&scratch.0, "crash-initialize", boundary);
        assert_eq!(snapshot(&scratch.root()), before);
        run(&scratch.0, "reject-partial");
        assert_eq!(snapshot(&scratch.root()), before);
    }
}

#[test]
fn killed_published_control_initialization_reopens_under_its_independent_pins() {
    let _serial = PROCESSES.lock().unwrap();
    for boundary in ["InitializedPublished", "InitializedDirectorySynced"] {
        let scratch = selected_store("node-initialization-published");
        let before = snapshot(&scratch.root());
        kill(&scratch.0, "crash-initialize", boundary);
        run(&scratch.0, "clean-old");
        assert_eq!(snapshot(&scratch.root()), before);
    }
}

#[test]
fn competing_operator_process_is_excluded_until_actual_owner_exit() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = initialized("node-competing-owner");
    let mut owner = ControlChild::spawn(&scratch.0, "hold-owner", "");
    owner.wait_ready("owner");
    run(&scratch.0, "reject-competing");
    owner.kill_at("owner");
    run(&scratch.0, "clean-old");
}
