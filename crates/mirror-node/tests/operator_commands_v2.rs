//! Actual finite-operator processes over synthetic state and loopback TCP.
//!
//! Pending recovery layouts below are constructed from canonical A1 records and
//! public storage APIs. They test command dispatch/authority, not termination
//! timing; A1's retained control_process_tests supply the actual SIGKILL evidence.
#![cfg(target_os = "linux")]

mod support_operator_v2;

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::ErrorKind,
    net::{SocketAddr, TcpListener},
    os::unix::ffi::OsStringExt,
    sync::Mutex,
};

use mirror_chain::v2::ChainV2;
use mirror_network::v2::session::MAX_SESSION_WIRE_BYTES_V2;
use mirror_node::v2::{NodeControlV2, records::IntentRecordV2};
use mirror_storage::v2::{ReplacementCheckpointV2, ReplacementStoreV2};

use support_operator_v2::{
    OperatorProcess, Peer, PeerMode, command, failure,
    state::{
        Scratch, assert_complete_state, checkpoint, control_root,
        history::{fork_chain, independent_identity, main_chain},
        preferred_fork, snapshot,
    },
    success,
};

// Every test in this executable participates: no concurrently held directory
// ownership descriptor can accidentally be inherited by an unrelated child.
static PROCESSES: Mutex<()> = Mutex::new(());

fn common(operation: &str, scratch: &Scratch) -> Vec<OsString> {
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

fn init(scratch: &Scratch) -> Vec<OsString> {
    let selected = checkpoint(&main_chain(3), 0);
    let mut arguments = common("init", scratch);
    arguments.extend([
        "--attach-existing".into(),
        "--height".into(),
        selected.chain.height.to_string().into(),
        "--tip-hash".into(),
        selected.chain.tip_hash.to_string().into(),
        "--ledger-root".into(),
        selected.chain.ledger_root.to_string().into(),
        "--generation".into(),
        selected.generation.to_string().into(),
    ]);
    arguments
}

fn sync(scratch: &Scratch, peer: SocketAddr) -> Vec<OsString> {
    let mut arguments = common("sync-once", scratch);
    arguments.extend([
        "--peer".into(),
        peer.to_string().into(),
        "--timeout-ms".into(),
        "2000".into(),
        "--max-io-bytes".into(),
        MAX_SESSION_WIRE_BYTES_V2.to_string().into(),
    ]);
    arguments
}

fn recover(scratch: &Scratch, action: &str) -> Vec<OsString> {
    let mut arguments = common("recover", scratch);
    arguments.extend(["--action".into(), action.into()]);
    arguments
}

fn set(arguments: &mut [OsString], flag: &str, value: impl Into<OsString>) {
    let index = arguments
        .iter()
        .position(|argument| argument == flag)
        .unwrap();
    arguments[index + 1] = value.into();
}

fn remove(arguments: &mut Vec<OsString>, flag: &str) {
    let index = arguments
        .iter()
        .position(|argument| argument == flag)
        .unwrap();
    arguments.drain(index..index + 2);
}

fn stored(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    drop(ReplacementStoreV2::initialize(scratch.root(), &main_chain(3)).unwrap());
    scratch
}

fn ready(label: &str) -> Scratch {
    let scratch = stored(label);
    drop(
        NodeControlV2::initialize(
            control_root(&scratch),
            scratch.root(),
            independent_identity(),
            checkpoint(&main_chain(3), 0),
        )
        .unwrap(),
    );
    scratch
}

fn pending(scratch: &Scratch) -> IntentRecordV2 {
    let intent = IntentRecordV2::new(
        independent_identity(),
        checkpoint(&main_chain(3), 0),
        main_chain(3),
        preferred_fork(),
    )
    .unwrap();
    fs::write(
        control_root(scratch).join("intent.v2"),
        intent.encode().unwrap(),
    )
    .unwrap();
    intent
}

fn pins(values: &BTreeMap<String, String>, prefix: &str, selected: ReplacementCheckpointV2) {
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

fn selected(scratch: &Scratch, expected: &ChainV2, generation: u64) {
    let owner = NodeControlV2::open_writer(
        control_root(scratch),
        scratch.root(),
        independent_identity(),
    )
    .unwrap();
    assert_eq!(owner.checkpoint(), checkpoint(expected, generation));
    assert_complete_state(owner.chain(), expected);
}

fn listener() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}

fn no_connection(listener: &TcpListener) {
    match listener.accept() {
        Err(error) if error.kind() == ErrorKind::WouldBlock => {}
        other => panic!("locally rejected command contacted its peer: {other:?}"),
    }
}

fn no_selected_fields(values: &BTreeMap<String, String>) {
    assert!(!values.keys().any(|key| key.starts_with("selected_")));
}

#[test]
fn help_is_finite_and_requires_no_paths_or_network() {
    let _serial = PROCESSES.lock().unwrap();
    for argument in ["help", "--help"] {
        let output = command(&[argument.into()]);
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        let text = std::str::from_utf8(&output.stdout).unwrap();
        for required in [
            "init",
            "status",
            "sync-once",
            "recover",
            "--attach-existing",
            "--config-hash",
            "--genesis-hash",
        ] {
            assert!(text.contains(required), "missing help token: {required}");
        }
    }
}

#[test]
fn malformed_missing_duplicate_and_cross_command_arguments_have_no_effects() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = Scratch::new("operator-usage-shape");
    let base = common("status", &scratch);
    let mut missing = base.clone();
    remove(&mut missing, "--genesis-hash");
    let mut duplicate = base.clone();
    duplicate.extend(["--control-dir".into(), "unused".into()]);
    let mut wrong_command = base.clone();
    wrong_command.extend(["--peer".into(), "127.0.0.1:1".into()]);
    let mut unknown = base.clone();
    unknown.extend(["--provider".into(), "synthetic-private-value".into()]);
    let mut without_attach = init(&scratch);
    without_attach.retain(|argument| argument != "--attach-existing");
    let mut missing_value = base.clone();
    missing_value.pop();
    let mut explicit_attach_value = init(&scratch);
    explicit_attach_value.push("true".into());
    let mut bad_action = recover(&scratch, "automatic");
    set(&mut bad_action, "--action", "synthetic-private-value");
    let before = snapshot(&scratch.0);
    for arguments in [
        Vec::new(),
        vec!["daemon".into()],
        vec!["help".into(), "status".into()],
        missing,
        duplicate,
        wrong_command,
        unknown,
        without_attach,
        missing_value,
        explicit_attach_value,
        bad_action,
    ] {
        let output = command(&arguments);
        let values = failure(&output, 2);
        assert_eq!(values["next"], "help");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("synthetic-private-value"));
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn invalid_hash_numeric_peer_and_limit_arguments_are_rejected_before_effects() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = Scratch::new("operator-usage-values");
    let unused = listener();
    let before = snapshot(&scratch.0);
    let mut cases = Vec::new();
    for (flag, value) in [
        ("--config-hash", "ab".to_owned()),
        ("--genesis-hash", "gg".repeat(32)),
        ("--control-dir", String::new()),
    ] {
        let mut arguments = common("status", &scratch);
        set(&mut arguments, flag, value);
        cases.push(arguments);
    }
    for (flag, value) in [
        ("--height", "4096"),
        ("--height", "03"),
        ("--height", "+3"),
        ("--height", "18446744073709551616"),
        ("--generation", "4"),
        ("--generation", "-1"),
        ("--tip-hash", "xyz"),
        ("--ledger-root", ""),
    ] {
        let mut arguments = init(&scratch);
        set(&mut arguments, flag, value);
        cases.push(arguments);
    }
    for (flag, value) in [
        ("--peer", "localhost:1000".to_owned()),
        ("--peer", "127.0.0.1:0".to_owned()),
        ("--peer", "127.0.0.1:65536".to_owned()),
        ("--timeout-ms", "0".to_owned()),
        ("--timeout-ms", "60001".to_owned()),
        ("--timeout-ms", "01".to_owned()),
        ("--max-io-bytes", "0".to_owned()),
        (
            "--max-io-bytes",
            (MAX_SESSION_WIRE_BYTES_V2 + 1).to_string(),
        ),
    ] {
        let mut arguments = sync(&scratch, unused.local_addr().unwrap());
        set(&mut arguments, flag, value);
        cases.push(arguments);
    }
    for arguments in cases {
        failure(&command(&arguments), 2);
        assert_eq!(snapshot(&scratch.0), before);
        no_connection(&unused);
    }
}

#[test]
fn non_utf8_control_characters_and_argument_budgets_are_bounded_without_effects() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = Scratch::new("operator-usage-budgets");
    let mut non_utf8 = common("status", &scratch);
    set(
        &mut non_utf8,
        "--store-dir",
        OsString::from_vec(vec![b'p', 0xff]),
    );
    let mut control = common("status", &scratch);
    set(&mut control, "--control-dir", "private\nsynthetic-token");
    let mut bidi = common("status", &scratch);
    set(&mut bidi, "--control-dir", "private\u{202e}synthetic-token");
    let mut long_token = common("status", &scratch);
    set(&mut long_token, "--control-dir", "x".repeat(4097));
    let mut long_total = vec![OsString::from("status")];
    long_total.extend((0..5).map(|_| OsString::from("x".repeat(4096))));
    let mut too_many = vec![OsString::from("status")];
    too_many.extend((0..32).map(|_| OsString::from("x")));
    let before = snapshot(&scratch.0);
    for arguments in [non_utf8, control, bidi, long_token, long_total, too_many] {
        let output = command(&arguments);
        failure(&output, 2);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("synthetic-token"));
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn missing_status_reports_uninspected_without_creating_or_reading_a_store() {
    let _serial = PROCESSES.lock().unwrap();
    for state in 0..3 {
        let scratch = Scratch::new("operator-status-missing");
        if state == 1 {
            drop(ReplacementStoreV2::initialize(scratch.root(), &main_chain(3)).unwrap());
        } else if state == 2 {
            fs::create_dir(scratch.root()).unwrap();
            fs::write(
                scratch.root().join("synthetic-unknown"),
                b"not a valid selected store",
            )
            .unwrap();
            fs::create_dir(control_root(&scratch)).unwrap();
        }
        let before = snapshot(&scratch.0);
        let values = success(&command(&common("status", &scratch)), 3, "status");
        assert_eq!(values["status"], "missing");
        assert_eq!(values["store_selection"], "uninspected");
        no_selected_fields(&values);
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn clean_status_verifies_full_pins_without_application_writes() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-status-clean");
    let before = snapshot(&scratch.0);
    let mut arguments = common("status", &scratch);
    set(
        &mut arguments,
        "--config-hash",
        independent_identity()
            .config_hash
            .to_string()
            .to_uppercase(),
    );
    set(
        &mut arguments,
        "--genesis-hash",
        independent_identity()
            .genesis_hash
            .to_string()
            .to_uppercase(),
    );
    let values = success(&command(&arguments), 0, "status");
    assert_eq!(values["status"], "clean");
    assert_eq!(values["store_selection"], "verified");
    pins(&values, "selected", checkpoint(&main_chain(3), 0));
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn invalid_clean_control_or_store_returns_failure_and_preserves_every_object() {
    let _serial = PROCESSES.lock().unwrap();
    for control_invalid in [true, false] {
        let scratch = ready("operator-status-invalid");
        let path = if control_invalid {
            control_root(&scratch).join("confirmed.v2")
        } else {
            scratch.root().join("selection.v2")
        };
        let mut bytes = fs::read(&path).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        fs::write(path, bytes).unwrap();
        let before = snapshot(&scratch.0);
        failure(&command(&common("status", &scratch)), 1);
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn pending_status_reports_authority_endpoints_without_inspecting_or_repairing_store() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-status-pending");
    let intent = pending(&scratch);
    // Status must not infer the current store selection from control endpoints.
    // A deliberately invalid store still yields the exact pending authority.
    fs::write(
        scratch.root().join("selection.v2"),
        b"synthetic-invalid-store",
    )
    .unwrap();
    let before = snapshot(&scratch.0);
    let values = success(&command(&common("status", &scratch)), 4, "status");
    assert_eq!(values["status"], "recovery-required");
    assert_eq!(values["store_selection"], "uninspected");
    pins(&values, "confirmed", intent.old());
    pins(&values, "old", intent.old());
    pins(&values, "candidate", intent.candidate());
    no_selected_fields(&values);
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn explicit_init_attaches_existing_store_and_a_fresh_process_verifies_it() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = stored("operator-init");
    let before = snapshot(&scratch.root());
    let values = success(&command(&init(&scratch)), 0, "init");
    assert_eq!(values["status"], "clean");
    assert_eq!(values["mode"], "attach-existing");
    assert_eq!(values["store_selection"], "verified");
    pins(&values, "selected", checkpoint(&main_chain(3), 0));
    assert_eq!(snapshot(&scratch.root()), before);
    let status = success(&command(&common("status", &scratch)), 0, "status");
    pins(&status, "selected", checkpoint(&main_chain(3), 0));
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn broken_stdout_reports_possible_completed_init_and_fresh_status_resolves_it() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = stored("operator-broken-stdout");
    let before = snapshot(&scratch.root());
    let output = OperatorProcess::spawn_closed_stdout(&init(&scratch)).finish();
    let values = failure(&output, 1);
    assert_eq!(values["error"], "output_io");
    assert_eq!(values["operation"], "init");
    assert_eq!(values["operation_may_have_completed"], "true");
    assert_eq!(values["next"], "inspect-status");
    assert_eq!(snapshot(&scratch.root()), before);
    let status = success(&command(&common("status", &scratch)), 0, "status");
    assert_eq!(status["status"], "clean");
    pins(&status, "selected", checkpoint(&main_chain(3), 0));
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn init_rejects_each_wrong_independent_identity_and_full_selection_pin() {
    let _serial = PROCESSES.lock().unwrap();
    for (flag, value) in [
        ("--config-hash", "09".repeat(32)),
        ("--genesis-hash", "08".repeat(32)),
        ("--height", "2".to_owned()),
        ("--tip-hash", "07".repeat(32)),
        ("--ledger-root", "06".repeat(32)),
        ("--generation", "1".to_owned()),
    ] {
        let scratch = stored("operator-init-wrong-pins");
        // A1 may create an empty control directory before an execution failure.
        // Precreate it here so the assertion addresses publication/store writes.
        fs::create_dir(control_root(&scratch)).unwrap();
        let before = snapshot(&scratch.0);
        let mut arguments = init(&scratch);
        set(&mut arguments, flag, value);
        failure(&command(&arguments), 1);
        assert_eq!(snapshot(&scratch.0), before);
        assert!(!control_root(&scratch).join("confirmed.v2").exists());
        let store = ReplacementStoreV2::open_writer(
            scratch.root(),
            independent_identity(),
            checkpoint(&main_chain(3), 0),
        )
        .unwrap();
        assert_complete_state(store.chain().unwrap(), &main_chain(3));
    }
}

#[test]
fn init_refuses_existing_control_and_never_creates_a_missing_selected_store() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-init-repeat");
    let before = snapshot(&scratch.0);
    failure(&command(&init(&scratch)), 1);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);

    let missing = Scratch::new("operator-init-no-store");
    failure(&command(&init(&missing)), 1);
    assert!(!missing.root().exists());
    assert!(!control_root(&missing).join("confirmed.v2").exists());
    assert!(
        fs::read_dir(control_root(&missing))
            .unwrap()
            .next()
            .is_none()
    );
}

#[test]
fn independent_config_and_genesis_mismatches_fail_before_connecting() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-independent-identity");
    let unused = listener();
    let before = snapshot(&scratch.0);
    for flag in ["--config-hash", "--genesis-hash"] {
        for mut arguments in [
            common("status", &scratch),
            recover(&scratch, "complete"),
            sync(&scratch, unused.local_addr().unwrap()),
        ] {
            set(&mut arguments, flag, "ab".repeat(32));
            let values = failure(&command(&arguments), 1);
            assert_eq!(values["error"], "wrong_network");
            assert_eq!(values["next"], "verify-inputs");
            no_connection(&unused);
            assert_eq!(snapshot(&scratch.0), before);
        }
    }
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn actual_tcp_identical_shorter_and_tied_histories_keep_complete_local_state() {
    let _serial = PROCESSES.lock().unwrap();
    let old = main_chain(3);
    for candidate in [old.clone(), main_chain(1), fork_chain()] {
        let scratch = ready("operator-sync-keep");
        let before = snapshot(&scratch.0);
        let peer = Peer::spawn(candidate, PeerMode::Good);
        let values = success(&command(&sync(&scratch, peer.address())), 0, "sync-once");
        let served = peer.finish();
        assert_eq!(values["status"], "clean");
        assert_eq!(values["choice"], "keep-current");
        assert_eq!(values["store_selection"], "verified");
        assert_eq!(values["attempts"], "1");
        assert_eq!(values["wire_bytes"], served.wire_bytes.to_string());
        pins(&values, "previous", checkpoint(&old, 0));
        pins(&values, "selected", checkpoint(&old, 0));
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &old, 0);
    }
}

#[test]
fn actual_tcp_preferred_membership_fork_survives_fresh_strict_reopen() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-sync-adopt");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let values = success(&command(&sync(&scratch, peer.address())), 0, "sync-once");
    let served = peer.finish();
    assert_eq!(values["choice"], "prefer-candidate");
    assert_eq!(values["attempts"], "1");
    assert_eq!(values["wire_bytes"], served.wire_bytes.to_string());
    pins(&values, "previous", checkpoint(&main_chain(3), 0));
    pins(&values, "selected", checkpoint(&candidate, 1));
    let status = success(&command(&common("status", &scratch)), 0, "status");
    pins(&status, "selected", checkpoint(&candidate, 1));
    assert_eq!(fs::read_dir(control_root(&scratch)).unwrap().count(), 1);
    assert!(!control_root(&scratch).join("intent.v2").exists());
    selected(&scratch, &candidate, 1);
}

#[test]
fn incomplete_trailing_and_invalid_block_transfers_preserve_selected_state() {
    let _serial = PROCESSES.lock().unwrap();
    for mode in [
        PeerMode::EarlyClose,
        PeerMode::MissingEnd,
        PeerMode::Trailing,
        PeerMode::InvalidBlock,
    ] {
        let scratch = ready("operator-sync-invalid-transfer");
        let before = snapshot(&scratch.0);
        let peer = Peer::spawn(preferred_fork(), mode);
        let values = failure(&command(&sync(&scratch, peer.address())), 1);
        assert_eq!(values["operation"], "sync-once");
        assert_eq!(
            values["error"],
            match mode {
                PeerMode::EarlyClose | PeerMode::MissingEnd => "peer_early_close",
                PeerMode::Trailing => "peer_trailing_data",
                PeerMode::InvalidBlock => "invalid_peer_history",
                _ => unreachable!(),
            }
        );
        peer.finish();
        assert_eq!(snapshot(&scratch.0), before, "mode={mode:?}");
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn valid_end_without_terminal_eof_reaches_the_supplied_deadline_without_adoption() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-sync-no-eof");
    let before = snapshot(&scratch.0);
    let peer = Peer::spawn(preferred_fork(), PeerMode::HoldAfterEnd);
    let mut arguments = sync(&scratch, peer.address());
    set(&mut arguments, "--timeout-ms", "200");
    let child = OperatorProcess::spawn(&arguments);
    peer.wait_ready();
    // The peer cannot close until finish is called, after the child returned.
    let values = failure(&child.finish(), 1);
    assert_eq!(values["error"], "peer_deadline");
    peer.finish();
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn positive_tiny_wire_budget_is_enforced_by_the_actual_session() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-sync-byte-budget");
    let before = snapshot(&scratch.0);
    let peer = Peer::spawn(preferred_fork(), PeerMode::RequestOnly);
    let mut arguments = sync(&scratch, peer.address());
    set(&mut arguments, "--max-io-bytes", "1");
    let values = failure(&command(&arguments), 1);
    assert_eq!(values["error"], "peer_io_budget");
    assert_eq!(peer.finish().request_bytes, 1);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn sync_rejects_missing_pending_or_malformed_local_state_before_any_tcp_connection() {
    let _serial = PROCESSES.lock().unwrap();
    for state in 0..4 {
        let scratch = if state == 0 {
            stored("operator-sync-local-missing")
        } else {
            ready("operator-sync-local-invalid")
        };
        match state {
            1 => {
                pending(&scratch);
            }
            2 => {
                fs::write(control_root(&scratch).join("confirmed.v2"), b"MNC2").unwrap();
            }
            3 => {
                fs::write(scratch.root().join("selection.v2"), b"MSR2").unwrap();
            }
            _ => {}
        }
        let unused = listener();
        let before = snapshot(&scratch.0);
        let values = failure(&command(&sync(&scratch, unused.local_addr().unwrap())), 1);
        if state == 1 {
            assert_eq!(values["error"], "recovery_required");
            assert_eq!(values["next"], "explicit-recover");
        }
        no_connection(&unused);
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn explicit_complete_command_publishes_the_canonical_pending_candidate() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-recover-complete");
    let intent = pending(&scratch);
    let values = success(&command(&recover(&scratch, "complete")), 0, "recover");
    assert_eq!(values["action"], "complete");
    assert_eq!(values["status"], "clean");
    assert_eq!(values["store_selection"], "verified");
    pins(&values, "selected", intent.candidate());
    assert!(!control_root(&scratch).join("intent.v2").exists());
    let status = success(&command(&common("status", &scratch)), 0, "status");
    pins(&status, "selected", intent.candidate());
    selected(&scratch, &preferred_fork(), 1);
}

#[test]
fn explicit_retain_old_command_retires_intent_without_changing_selected_store() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-recover-retain");
    let intent = pending(&scratch);
    let before_store = snapshot(&scratch.root());
    let values = success(&command(&recover(&scratch, "retain-old")), 0, "recover");
    assert_eq!(values["action"], "retain-old");
    pins(&values, "selected", intent.old());
    assert_eq!(snapshot(&scratch.root()), before_store);
    assert!(!control_root(&scratch).join("intent.v2").exists());
    selected(&scratch, &main_chain(3), 0);
    let before = snapshot(&scratch.0);
    failure(&command(&recover(&scratch, "complete")), 1);
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn candidate_selected_confirming_or_confirmed_can_only_complete_never_roll_back() {
    let _serial = PROCESSES.lock().unwrap();
    for confirmation in 0..3 {
        let scratch = ready("operator-recover-no-rollback");
        let intent = pending(&scratch);
        let mut store =
            ReplacementStoreV2::open_writer(scratch.root(), independent_identity(), intent.old())
                .unwrap();
        assert_eq!(
            store
                .replace(
                    independent_identity(),
                    intent.old(),
                    intent.candidate_chain()
                )
                .unwrap(),
            intent.candidate()
        );
        drop(store);
        if confirmation == 1 {
            fs::write(
                control_root(&scratch).join("confirmed.pending"),
                &intent.candidate_record().encode()[..17],
            )
            .unwrap();
        } else if confirmation == 2 {
            fs::write(
                control_root(&scratch).join("confirmed.v2"),
                intent.candidate_record().encode(),
            )
            .unwrap();
        }
        let before = snapshot(&scratch.0);
        failure(&command(&recover(&scratch, "retain-old")), 1);
        assert_eq!(snapshot(&scratch.0), before);
        let values = success(&command(&recover(&scratch, "complete")), 0, "recover");
        pins(&values, "selected", intent.candidate());
        selected(&scratch, &preferred_fork(), 1);
        assert_eq!(fs::read_dir(control_root(&scratch)).unwrap().count(), 1);
    }
}

#[test]
fn unknown_and_partial_control_artifacts_are_rejected_and_preserved() {
    let _serial = PROCESSES.lock().unwrap();
    for artifact in 0..4 {
        let scratch = if artifact == 0 {
            stored("operator-partial-init")
        } else {
            ready("operator-control-artifact")
        };
        let control = control_root(&scratch);
        match artifact {
            0 => {
                fs::create_dir(&control).unwrap();
                fs::write(control.join("confirmed.pending"), b"MNC2").unwrap();
            }
            1 => {
                fs::write(control.join("intent.pending"), b"MNI2").unwrap();
            }
            2 => {
                fs::write(control.join("foreign"), b"preserve synthetic unknown bytes").unwrap();
            }
            _ => {
                fs::create_dir(control.join("foreign")).unwrap();
                fs::write(control.join("foreign/keep"), b"untouched").unwrap();
            }
        }
        let before = snapshot(&scratch.0);
        for arguments in [
            common("status", &scratch),
            recover(&scratch, "complete"),
            recover(&scratch, "retain-old"),
            init(&scratch),
        ] {
            failure(&command(&arguments), 1);
            assert_eq!(snapshot(&scratch.0), before);
        }
    }
}

#[test]
fn sync_holds_both_roots_during_receive_and_kill_releases_ownership_without_adoption() {
    let _serial = PROCESSES.lock().unwrap();
    let scratch = ready("operator-sync-ownership");
    let before = snapshot(&scratch.0);
    let peer = Peer::spawn(preferred_fork(), PeerMode::HoldBeforeResponse);
    let mut arguments = sync(&scratch, peer.address());
    set(&mut arguments, "--timeout-ms", "10000");
    let mut first = OperatorProcess::spawn(&arguments);
    peer.wait_ready();
    assert!(first.running());
    let unused = listener();
    let values = failure(&command(&sync(&scratch, unused.local_addr().unwrap())), 1);
    assert_eq!(values["error"], "busy");
    no_connection(&unused);
    let values = failure(&command(&common("status", &scratch)), 1);
    assert_eq!(values["error"], "busy");
    assert!(
        ReplacementStoreV2::open_writer(
            scratch.root(),
            independent_identity(),
            checkpoint(&main_chain(3), 0)
        )
        .is_err()
    );
    assert!(
        first.running(),
        "first session unexpectedly ended before ownership checks"
    );
    let killed = first.kill();
    assert!(!killed.status.success());
    peer.finish();
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
    success(&command(&common("status", &scratch)), 0, "status");
}
