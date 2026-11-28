//! Actual bounded provider processes, signal shutdown and ownership release.
//!
//! Linux /proc observations tie a child-owned accepted socket to its blocked
//! receive syscall. TCP handshake success or a delay alone is not admission.
#![cfg(target_os = "linux")]

mod support_provider_v2;

use std::{
    fs,
    io::Write,
    net::Shutdown,
    thread,
    time::{Duration, Instant},
};

use mirror_network::v2::session::{
    MAX_SESSION_WIRE_BYTES_V2, REQUEST_LEN_V2, encode_history_request_v2,
};
use mirror_node::v2::{NodeControlV2, records::IntentRecordV2};

use support_provider_v2::{
    ProviderProcess, arguments, assert_busy, assert_no_connection, assert_port_closed,
    assert_reopened, command, connect, failure, no_response_yet, process_guard, raw_request,
    read_response, ready, receive_complete, remove, reserve, set,
    state::{
        Scratch, assert_complete_state, checkpoint, control_root,
        history::{independent_identity, main_chain},
        preferred_fork, snapshot,
    },
    stopped, transcript,
};

#[test]
fn invalid_provider_options_and_all_scalar_bounds_fail_before_effects() {
    let _guard = process_guard();
    let scratch = Scratch::new("provider-invalid-options");
    let reservation = reserve();
    let original = arguments(&scratch, reservation.local_addr().unwrap());
    let mut cases = Vec::new();
    for flag in [
        "--listen",
        "--max-attempts",
        "--max-io-bytes",
        "--timeout-ms",
        "--admission-ms",
    ] {
        let mut changed = original.clone();
        remove(&mut changed, flag);
        cases.push(changed);
    }
    for (flag, value) in [
        ("--listen", "127.0.0.1:0".to_owned()),
        ("--listen", "localhost:49123".to_owned()),
        ("--max-attempts", "0".to_owned()),
        ("--max-attempts", "9".to_owned()),
        ("--max-attempts", "01".to_owned()),
        ("--max-io-bytes", "0".to_owned()),
        (
            "--max-io-bytes",
            (MAX_SESSION_WIRE_BYTES_V2 + 1).to_string(),
        ),
        ("--max-io-bytes", "18446744073709551616".to_owned()),
        ("--timeout-ms", "0".to_owned()),
        ("--timeout-ms", "60001".to_owned()),
        ("--admission-ms", "0".to_owned()),
        ("--admission-ms", "60001".to_owned()),
    ] {
        let mut changed = original.clone();
        set(&mut changed, flag, value);
        cases.push(changed);
    }
    for extra in [
        vec!["--max-attempts", "2"],
        vec!["--peer", "127.0.0.1:49123"],
        vec!["--action", "complete"],
        vec!["--watch", "yes"],
        vec!["extra"],
    ] {
        let mut changed = original.clone();
        changed.extend(extra.into_iter().map(std::ffi::OsString::from));
        cases.push(changed);
    }
    let before = snapshot(&scratch.0);
    for arguments in cases {
        let values = failure(&command(&arguments), 2);
        assert_eq!(values["error"], "usage");
        assert_eq!(snapshot(&scratch.0), before);
        assert_no_connection(&reservation);
    }
}

#[test]
fn missing_pending_corrupt_wrong_network_and_busy_state_refuse_before_bind() {
    let _guard = process_guard();
    for case in [
        "missing-control",
        "missing-store",
        "pending",
        "malformed",
        "wrong-config",
        "wrong-genesis",
        "busy",
    ] {
        let scratch = if case == "missing-control" {
            Scratch::new("provider-missing-control")
        } else {
            ready("provider-local-refusal")
        };
        // Keeping this listener bound makes a successful bind impossible. The
        // local-state diagnostic must take precedence over a listen_bind error.
        let occupied = reserve();
        let mut args = arguments(&scratch, occupied.local_addr().unwrap());
        let mut held = None;
        let expected = match case {
            "missing-control" => "missing_control",
            "missing-store" => {
                // Only the test's freshly created private store is removed.
                fs::remove_dir_all(scratch.root()).unwrap();
                "missing_store"
            }
            "pending" => {
                let intent = IntentRecordV2::new(
                    independent_identity(),
                    checkpoint(&main_chain(3), 0),
                    main_chain(3),
                    preferred_fork(),
                )
                .unwrap();
                fs::write(
                    control_root(&scratch).join("intent.v2"),
                    intent.encode().unwrap(),
                )
                .unwrap();
                "recovery_required"
            }
            "malformed" => {
                fs::write(control_root(&scratch).join("confirmed.v2"), b"unknown").unwrap();
                "invalid_control"
            }
            "wrong-config" => {
                set(&mut args, "--config-hash", "08".repeat(32));
                "wrong_network"
            }
            "wrong-genesis" => {
                set(&mut args, "--genesis-hash", "09".repeat(32));
                "wrong_network"
            }
            "busy" => {
                held = Some(
                    NodeControlV2::open_writer(
                        control_root(&scratch),
                        scratch.root(),
                        independent_identity(),
                    )
                    .unwrap(),
                );
                "busy"
            }
            _ => unreachable!(),
        };
        let before = snapshot(&scratch.0);
        let values = failure(&command(&args), 1);
        assert_eq!(values["error"], expected, "{case}");
        assert_eq!(snapshot(&scratch.0), before, "{case}");
        assert_no_connection(&occupied);
        drop(held);
        if case == "busy" {
            assert_reopened(&scratch);
        }
    }
}

#[test]
fn bind_failure_releases_both_owners_and_preserves_complete_selection() {
    let _guard = process_guard();
    let scratch = ready("provider-bind-failure");
    let before = snapshot(&scratch.0);
    let occupied = reserve();
    let values = failure(
        &command(&arguments(&scratch, occupied.local_addr().unwrap())),
        1,
    );
    assert_eq!(values["error"], "listen_bind");
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn repeated_actual_requests_serve_one_complete_history_and_exhaust_attempts() {
    let _guard = process_guard();
    let scratch = ready("provider-complete-history");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--max-attempts", "2");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    let expected = main_chain(3);
    let mut charged = 0;
    for _ in 0..2 {
        let (received, wire_bytes) = receive_complete(address);
        assert_complete_state(&received, &expected);
        charged += wire_bytes;
    }
    assert_eq!(
        charged,
        2 * (REQUEST_LEN_V2 + transcript(&expected).len()) as u64
    );
    stopped(
        &provider.finish(),
        "attempt-limit",
        2,
        2,
        0,
        charged,
        "none",
    );
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn malformed_failed_requests_keep_attempts_bytes_and_last_error_after_success() {
    let _guard = process_guard();
    let scratch = ready("provider-rejected-requests");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--max-attempts", "5");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    let valid = encode_history_request_v2(independent_identity());
    let mut malformed = valid.to_vec();
    malformed[0] ^= 1;
    let mut wrong_network = valid.to_vec();
    wrong_network[8] ^= 1;
    let short = valid[..REQUEST_LEN_V2 - 1].to_vec();
    let mut trailing = valid.to_vec();
    trailing.push(0x7f);
    let requests = [malformed, wrong_network, short, trailing];
    let mut charged = 0;
    for request in requests {
        charged += request.len() as u64;
        assert!(raw_request(address, &request).is_empty());
    }
    let (received, bytes) = receive_complete(address);
    assert_complete_state(&received, &main_chain(3));
    charged += bytes;
    stopped(
        &provider.finish(),
        "attempt-limit",
        5,
        1,
        4,
        charged,
        "peer_trailing_data",
    );
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn aggregate_wire_allowance_is_not_replenished_after_a_failed_client() {
    let _guard = process_guard();
    let scratch = ready("provider-wire-limit");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--max-attempts", "3");
    set(&mut args, "--max-io-bytes", "244");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    let valid = encode_history_request_v2(independent_identity());
    let mut malformed = valid;
    malformed[0] ^= 1;
    assert!(raw_request(address, &malformed).is_empty());
    let partial = raw_request(address, &valid);
    assert_eq!(partial.len(), 100);
    assert_eq!(partial, transcript(&main_chain(3))[..100]);
    stopped(
        &provider.finish(),
        "wire-limit",
        2,
        0,
        2,
        244,
        "peer_io_budget",
    );
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn queued_handshake_is_not_a_second_active_session_and_ownership_stays_held() {
    let _guard = process_guard();
    let scratch = ready("provider-one-active");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--max-attempts", "2");
    set(&mut args, "--timeout-ms", "10000");
    set(&mut args, "--admission-ms", "20000");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    assert_busy(&scratch);
    let address = provider.address();
    let request = encode_history_request_v2(independent_identity());
    let expected = transcript(&main_chain(3));
    let mut first = connect(address);
    first.write_all(&request).unwrap();
    // A one-byte receive on this accepted FD is F2's mandatory request-EOF
    // probe, after the complete request has been read and validated.
    provider.wait_receiving(&first, 1);
    assert_busy(&scratch);
    let mut queued = connect(address);
    queued.write_all(&request).unwrap();
    queued.shutdown(Shutdown::Write).unwrap();
    no_response_yet(&mut queued);
    first.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut first), expected);
    assert_eq!(read_response(&mut queued), expected);
    drop(first);
    drop(queued);
    stopped(
        &provider.finish(),
        "attempt-limit",
        2,
        2,
        0,
        2 * (REQUEST_LEN_V2 + expected.len()) as u64,
        "none",
    );
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn sigint_and_sigterm_stop_an_idle_provider_without_admitting_an_attempt() {
    let _guard = process_guard();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("provider-idle-signal");
        let before = snapshot(&scratch.0);
        let reservation = reserve();
        let args = arguments(&scratch, reservation.local_addr().unwrap());
        let mut provider = ProviderProcess::spawn(&args, reservation);
        provider.wait_listening();
        let address = provider.address();
        let stopped_at = Instant::now();
        provider.signal(signal);
        stopped(&provider.finish(), "shutdown", 0, 0, 0, 0, "none");
        assert!(stopped_at.elapsed() < Duration::from_secs(3));
        assert_port_closed(address);
        assert_eq!(snapshot(&scratch.0), before);
        assert_reopened(&scratch);
    }
}

#[test]
fn sigterm_drains_an_admitted_valid_request_without_admitting_a_queued_client() {
    let _guard = process_guard();
    let scratch = ready("provider-active-complete");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let args = arguments(&scratch, reservation.local_addr().unwrap());
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    let request = encode_history_request_v2(independent_identity());
    let expected = transcript(&main_chain(3));
    let mut first = connect(address);
    first.write_all(&request[..1]).unwrap();
    provider.wait_receiving(&first, REQUEST_LEN_V2 - 1);
    let mut queued = connect(address);
    queued.write_all(&request).unwrap();
    queued.shutdown(Shutdown::Write).unwrap();
    provider.signal(libc::SIGTERM);
    first.write_all(&request[1..]).unwrap();
    first.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut first), expected);
    stopped(
        &provider.finish(),
        "shutdown",
        1,
        1,
        0,
        (REQUEST_LEN_V2 + expected.len()) as u64,
        "none",
    );
    assert!(read_response(&mut queued).is_empty());
    drop(first);
    drop(queued);
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn sigint_preserves_the_original_deadline_of_an_admitted_missing_eof_request() {
    let _guard = process_guard();
    let scratch = ready("provider-active-timeout");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--timeout-ms", "1000");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    let mut client = connect(address);
    client
        .write_all(&encode_history_request_v2(independent_identity()))
        .unwrap();
    provider.wait_receiving(&client, 1);
    let admitted_at = Instant::now();
    // Admission has already been established by the blocked EOF probe. This
    // controlled delay places the stop request well into that existing timeout;
    // it is not used to infer admission.
    thread::sleep(Duration::from_millis(650));
    provider.signal(libc::SIGINT);
    stopped(
        &provider.finish(),
        "shutdown",
        1,
        0,
        1,
        REQUEST_LEN_V2 as u64,
        "peer_deadline",
    );
    assert!(
        admitted_at.elapsed() < Duration::from_millis(1450),
        "shutdown appears to have granted a fresh attempt timeout"
    );
    assert!(read_response(&mut client).is_empty());
    drop(client);
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn idle_admission_expiry_consumes_no_attempt_and_releases_the_listener() {
    let _guard = process_guard();
    let scratch = ready("provider-idle-admission");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--admission-ms", "250");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    stopped(&provider.finish(), "admission-timeout", 0, 0, 0, 0, "none");
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn admission_expiry_drains_only_the_previously_admitted_request() {
    let _guard = process_guard();
    let scratch = ready("provider-active-admission");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let mut args = arguments(&scratch, reservation.local_addr().unwrap());
    set(&mut args, "--admission-ms", "300");
    set(&mut args, "--timeout-ms", "3000");
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let observed_listener_at = Instant::now();
    let address = provider.address();
    let request = encode_history_request_v2(independent_identity());
    let expected = transcript(&main_chain(3));
    let mut first = connect(address);
    first.write_all(&request).unwrap();
    provider.wait_receiving(&first, 1);
    let mut queued = connect(address);
    queued.write_all(&request).unwrap();
    queued.shutdown(Shutdown::Write).unwrap();
    // The admission clock began before the observed listener existed. Waiting
    // beyond that window is controlled input timing after proven admission.
    while observed_listener_at.elapsed() < Duration::from_millis(400) {
        assert!(
            provider.running(),
            "active attempt was aborted at admission expiry"
        );
        thread::sleep(Duration::from_millis(5));
    }
    first.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut first), expected);
    stopped(
        &provider.finish(),
        "admission-timeout",
        1,
        1,
        0,
        (REQUEST_LEN_V2 + expected.len()) as u64,
        "none",
    );
    assert!(read_response(&mut queued).is_empty());
    drop(first);
    drop(queued);
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn killing_an_admitted_blocked_provider_releases_owners_without_state_mutation() {
    let _guard = process_guard();
    let scratch = ready("provider-kill");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let args = arguments(&scratch, reservation.local_addr().unwrap());
    let mut provider = ProviderProcess::spawn(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    let mut client = connect(address);
    client
        .write_all(&encode_history_request_v2(independent_identity())[..1])
        .unwrap();
    provider.wait_receiving(&client, REQUEST_LEN_V2 - 1);
    assert_busy(&scratch);
    let output = provider.kill();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(read_response(&mut client).is_empty());
    drop(client);
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn closed_stdout_reports_possible_completed_serving_and_never_restarts() {
    let _guard = process_guard();
    let scratch = ready("provider-closed-output");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let args = arguments(&scratch, reservation.local_addr().unwrap());
    let mut provider = ProviderProcess::spawn_closed_stdout(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    provider.signal(libc::SIGTERM);
    let values = failure(&provider.finish(), 1);
    assert_eq!(values["operation"], "serve");
    assert_eq!(values["error"], "output_io");
    assert_eq!(values["operation_may_have_completed"], "true");
    assert_eq!(values["next"], "inspect-status");
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    assert_reopened(&scratch);
}

#[test]
fn blocked_final_stdout_cannot_extend_listener_or_directory_ownership() {
    let _guard = process_guard();
    let scratch = ready("provider-blocked-output");
    let before = snapshot(&scratch.0);
    let reservation = reserve();
    let args = arguments(&scratch, reservation.local_addr().unwrap());
    let mut provider = ProviderProcess::spawn_blocked_stdout(&args, reservation);
    provider.wait_listening();
    let address = provider.address();
    provider.signal(libc::SIGTERM);
    provider.wait_stdout_blocked();
    assert!(provider.running());
    assert!(!provider.listener_is_owned());
    assert_port_closed(address);
    // The provider is still alive and blocked in its final write. A fresh
    // actual status process succeeding now proves release before output.
    assert_reopened(&scratch);
    assert_eq!(snapshot(&scratch.0), before);
    assert!(provider.running());
    provider.close_output_reader();
    let values = failure(&provider.finish(), 1);
    assert_eq!(values["operation"], "serve");
    assert_eq!(values["error"], "output_io");
    assert_eq!(values["operation_may_have_completed"], "true");
    assert_eq!(snapshot(&scratch.0), before);
}
