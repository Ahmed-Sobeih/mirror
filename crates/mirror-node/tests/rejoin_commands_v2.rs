//! Actual finite receive/adopt/serve commands over independent synthetic stores.
//!
//! Linux parent observations establish real receive/adoption/service phases.
//! Canonical pending layouts are labelled separately from process interruption;
//! retained A1 tests remain the evidence for exact publication fault boundaries.
#![cfg(target_os = "linux")]

mod support_rejoin_v2;

use std::{
    ffi::OsString,
    fs,
    io::Write,
    net::Shutdown,
    os::unix::{ffi::OsStringExt, process::ExitStatusExt},
    thread,
    time::{Duration, Instant},
};

use mirror_crypto::Hash256;
use mirror_network::v2::{
    HistoryFramesV2,
    session::{MAX_SESSION_WIRE_BYTES_V2, REQUEST_LEN_V2, encode_history_request_v2},
};
use mirror_node::v2::{NodeControlV2, records::IntentRecordV2};
use mirror_protocol::v2::{FRAME_HEADER_LEN_V2, FrameRefV2, encode_frame_v2};
use mirror_storage::v2::ReplacementStoreV2;

use support_rejoin_v2::{
    IntentWatch, Normal, Peer, PeerMode, ProviderProcess, RejoinProcess, Service, accept_request,
    adoption_history, arguments, assert_busy, assert_no_connection, assert_port_closed, command,
    connect, failure, no_response_yet, pending, pins, process_guard, provider, raw_request,
    read_response, ready, ready_with, receive_bytes, receive_complete, recover, remove, reserve,
    selected, set,
    state::{
        Scratch, assert_complete_state, checkpoint, control_root,
        history::{fork_chain, independent_identity, main_chain},
        preferred_fork, snapshot,
    },
    stopped, success, sync, transcript, uncertain, verify_stopped_intent,
};

#[test]
fn all_required_options_scalar_bounds_and_malformed_arguments_fail_before_effects() {
    let _guard = process_guard();
    let scratch = Scratch::new("rejoin-invalid-options");
    let source = reserve();
    let occupied = reserve();
    let original = arguments(
        &scratch,
        source.local_addr().unwrap(),
        occupied.local_addr().unwrap(),
    );
    let mut cases = Vec::new();
    for flag in [
        "--control-dir",
        "--store-dir",
        "--config-hash",
        "--genesis-hash",
        "--peer",
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
        ("--peer", "127.0.0.1:0".to_owned()),
        ("--peer", "localhost:49123".to_owned()),
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
        ("--config-hash", "ab".repeat(31)),
        ("--genesis-hash", "zz".repeat(32)),
        ("--store-dir", "x".repeat(4097)),
        ("--control-dir", "bad\npath".to_owned()),
        ("--control-dir", "bad\u{202e}path".to_owned()),
    ] {
        let mut changed = original.clone();
        set(&mut changed, flag, value);
        cases.push(changed);
    }
    for extra in [
        vec!["--peer", "127.0.0.1:49123"],
        vec!["--listen", "127.0.0.1:49123"],
        vec!["--action", "complete"],
        vec!["--attach-existing"],
        vec!["--watch", "yes"],
        vec!["extra"],
    ] {
        let mut changed = original.clone();
        changed.extend(extra.into_iter().map(OsString::from));
        cases.push(changed);
    }
    let mut non_utf8 = original.clone();
    set(
        &mut non_utf8,
        "--control-dir",
        OsString::from_vec(vec![0xff]),
    );
    cases.push(non_utf8);
    let mut count = original.clone();
    count.extend((0..13).map(|_| OsString::from("extra")));
    cases.push(count);
    let before = snapshot(&scratch.0);
    for arguments in cases {
        let values = failure(&command(&arguments), 2);
        assert_eq!(values["error"], "usage");
        assert_eq!(snapshot(&scratch.0), before);
        assert_no_connection(&source);
        assert_no_connection(&occupied);
    }
}

#[test]
fn missing_pending_corrupt_wrong_network_and_busy_state_refuse_before_peer_or_bind() {
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
            Scratch::new("rejoin-missing-control")
        } else {
            ready("rejoin-local-refusal")
        };
        let source = reserve();
        let occupied = reserve();
        let mut args = arguments(
            &scratch,
            source.local_addr().unwrap(),
            occupied.local_addr().unwrap(),
        );
        let mut held = None;
        let expected = match case {
            "missing-control" => "missing_control",
            "missing-store" => {
                fs::remove_dir_all(scratch.root()).unwrap();
                "missing_store"
            }
            "pending" => {
                pending(&scratch, &preferred_fork());
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
        assert_eq!(values["operation"], "rejoin-serve");
        assert_eq!(values["error"], expected, "{case}");
        assert!(!values.contains_key("operation_may_have_completed"));
        assert_eq!(snapshot(&scratch.0), before, "{case}");
        assert_no_connection(&source);
        assert_no_connection(&occupied);
        drop(held);
    }
}

#[test]
fn three_executable_nodes_adopt_serve_and_explicitly_restart_with_complete_state() {
    let _guard = process_guard();
    let old = main_chain(3);
    let candidate = preferred_fork();
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        old.ledger().membership().encode().unwrap(),
        candidate.ledger().membership().encode().unwrap()
    );
    let seed = ready_with("rejoin-three-seed", &candidate);
    let joining = ready("rejoin-three-joining");
    let third = ready("rejoin-three-requester");
    assert_ne!(seed.0, joining.0);
    assert_ne!(joining.0, third.0);
    assert_ne!(third.0, seed.0);
    let seed_before = snapshot(&seed.0);
    let mut after_first = None;
    for restart in [false, true] {
        let seed_port = reserve();
        let mut seed_args = provider::arguments(&seed, seed_port.local_addr().unwrap());
        // Keep the real seed alive until the real third process has completed.
        set(&mut seed_args, "--max-attempts", "2");
        let mut seed_process = ProviderProcess::spawn(&seed_args, seed_port);
        seed_process.wait_listening();
        let rejoin_port = reserve();
        let mut args = arguments(
            &joining,
            seed_process.address(),
            rejoin_port.local_addr().unwrap(),
        );
        set(&mut args, "--max-attempts", "2");
        let mut rejoining = RejoinProcess::spawn(&args, rejoin_port);
        rejoining.wait_listening();
        assert_busy(&joining);
        let third_result = success(&command(&sync(&third, rejoining.address())), 0, "sync-once");
        assert_eq!(
            third_result["choice"],
            if restart {
                "keep-current"
            } else {
                "prefer-candidate"
            }
        );
        pins(&third_result, "selected", checkpoint(&candidate, 1));
        let rejoin_address = rejoining.address();
        let mut expected = Normal::adopted(&candidate, "attempt-limit");
        if restart {
            expected.previous = checkpoint(&candidate, 1);
            expected.choice = "keep-current";
        }
        expected.service = Service::complete(receive_bytes(&candidate));
        stopped(&rejoining.finish(), expected);
        assert_port_closed(rejoin_address);
        assert!(seed_process.running());
        seed_process.signal(libc::SIGTERM);
        let seed_result = success(&seed_process.finish(), 0, "serve");
        assert_eq!(seed_result["reason"], "shutdown");
        assert_eq!(seed_result["attempts"], "1");
        assert_eq!(seed_result["completed_sessions"], "1");
        assert_eq!(seed_result["rejected_sessions"], "0");
        assert_eq!(
            seed_result["wire_bytes"],
            receive_bytes(&candidate).to_string()
        );
        pins(&seed_result, "selected", checkpoint(&candidate, 0));
        selected(&seed, &candidate, 0);
        selected(&joining, &candidate, 1);
        selected(&third, &candidate, 1);
        assert_eq!(snapshot(&seed.0), seed_before);
        let snapshots = (snapshot(&joining.0), snapshot(&third.0));
        if let Some(first) = &after_first {
            assert_eq!(&snapshots, first);
        } else {
            after_first = Some(snapshots);
        }
    }
}

#[test]
fn identical_shorter_and_tied_receives_serve_the_retained_complete_branch() {
    let _guard = process_guard();
    let old = main_chain(3);
    for candidate in [old.clone(), main_chain(1), fork_chain()] {
        let scratch = ready("rejoin-keep-current");
        let before = snapshot(&scratch.0);
        let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
        let reservation = reserve();
        let mut args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
        set(&mut args, "--max-attempts", "2");
        let mut child = RejoinProcess::spawn(&args, reservation);
        child.wait_listening();
        let (received, wire) = receive_complete(child.address());
        assert_complete_state(&received, &old);
        assert_eq!(wire, receive_bytes(&old));
        let mut expected = Normal::retained(&candidate, "keep-current", "attempt-limit");
        expected.service = Service::complete(wire);
        stopped(&child.finish(), expected);
        assert_eq!(peer.finish().wire_bytes, receive_bytes(&candidate));
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &old, 0);
    }
}

#[test]
fn sole_attempt_adopts_then_stops_normally_without_binding_an_occupied_port() {
    let _guard = process_guard();
    let scratch = ready("rejoin-one-attempt");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let occupied = reserve();
    let mut args = arguments(&scratch, peer.address(), occupied.local_addr().unwrap());
    set(&mut args, "--max-attempts", "1");
    stopped(
        &command(&args),
        Normal::adopted(&candidate, "attempt-limit"),
    );
    assert_eq!(peer.finish().wire_bytes, receive_bytes(&candidate));
    assert_no_connection(&occupied);
    selected(&scratch, &candidate, 1);
}

#[test]
fn exact_response_allowance_requires_one_spare_eof_byte_without_charging_eof() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    let charged = receive_bytes(&candidate);
    for spare in [0, 1] {
        let scratch = ready("rejoin-eof-budget");
        let before = snapshot(&scratch.0);
        let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
        let occupied = reserve();
        let mut args = arguments(&scratch, peer.address(), occupied.local_addr().unwrap());
        set(&mut args, "--max-attempts", "1");
        set(&mut args, "--max-io-bytes", (charged + spare).to_string());
        let output = command(&args);
        if spare == 0 {
            let values = failure(&output, 1);
            assert_eq!(values["error"], "peer_io_budget");
            assert!(!values.contains_key("operation_may_have_completed"));
            assert_eq!(snapshot(&scratch.0), before);
            selected(&scratch, &main_chain(3), 0);
        } else {
            stopped(&output, Normal::adopted(&candidate, "attempt-limit"));
            selected(&scratch, &candidate, 1);
        }
        assert_eq!(peer.finish().wire_bytes, charged);
        assert_no_connection(&occupied);
    }
}

#[test]
fn remaining_wire_allowance_is_shared_across_receive_rejection_and_partial_service() {
    let _guard = process_guard();
    let scratch = ready("rejoin-shared-wire");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let mut args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
    set(&mut args, "--max-attempts", "4");
    set(
        &mut args,
        "--max-io-bytes",
        (receive_bytes(&candidate) + 244).to_string(),
    );
    let mut child = RejoinProcess::spawn(&args, reservation);
    child.wait_listening();
    let valid = encode_history_request_v2(independent_identity());
    let mut invalid = valid;
    invalid[0] ^= 1;
    assert!(raw_request(child.address(), &invalid).is_empty());
    let partial = raw_request(child.address(), &valid);
    assert_eq!(partial, transcript(&candidate)[..100]);
    let mut expected = Normal::adopted(&candidate, "wire-limit");
    expected.service = Service {
        completed: 0,
        rejected: 2,
        bytes: 244,
        last_error: "peer_io_budget",
    };
    stopped(&child.finish(), expected);
    peer.finish();
    selected(&scratch, &candidate, 1);
}

#[test]
fn rejected_service_keeps_attempts_bytes_and_last_error_after_a_later_success() {
    let _guard = process_guard();
    let scratch = ready("rejoin-shared-attempts");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let mut args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
    set(&mut args, "--max-attempts", "3");
    let mut child = RejoinProcess::spawn(&args, reservation);
    child.wait_listening();
    let mut invalid = encode_history_request_v2(independent_identity());
    invalid[0] ^= 1;
    assert!(raw_request(child.address(), &invalid).is_empty());
    let (received, bytes) = receive_complete(child.address());
    assert_complete_state(&received, &candidate);
    let mut expected = Normal::adopted(&candidate, "attempt-limit");
    expected.service = Service {
        completed: 1,
        rejected: 1,
        bytes: REQUEST_LEN_V2 as u64 + bytes,
        last_error: "invalid_peer_request",
    };
    stopped(&child.finish(), expected);
    peer.finish();
    selected(&scratch, &candidate, 1);
}

#[test]
fn incomplete_trailing_and_invalid_history_transfers_never_adopt_or_serve() {
    let _guard = process_guard();
    for mode in [
        PeerMode::EarlyClose,
        PeerMode::MissingEnd,
        PeerMode::Trailing,
        PeerMode::InvalidBlock,
    ] {
        let scratch = ready("rejoin-invalid-history");
        let before = snapshot(&scratch.0);
        let peer = Peer::spawn(preferred_fork(), mode);
        let occupied = reserve();
        let args = arguments(&scratch, peer.address(), occupied.local_addr().unwrap());
        let values = failure(&command(&args), 1);
        assert_eq!(
            values["error"],
            match mode {
                PeerMode::EarlyClose | PeerMode::MissingEnd => "peer_early_close",
                PeerMode::Trailing => "peer_trailing_data",
                PeerMode::InvalidBlock => "invalid_peer_history",
                _ => unreachable!(),
            }
        );
        assert!(!values.contains_key("operation_may_have_completed"));
        peer.finish();
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn malformed_header_and_independently_wrong_network_begin_refuse_before_adoption() {
    let _guard = process_guard();
    for wrong_network in [false, true] {
        let scratch = ready("rejoin-invalid-framing");
        let before = snapshot(&scratch.0);
        let source = reserve();
        let occupied = reserve();
        let args = arguments(
            &scratch,
            source.local_addr().unwrap(),
            occupied.local_addr().unwrap(),
        );
        let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
        let mut stream = accept_request(&source);
        let response = if wrong_network {
            let mut summary = HistoryFramesV2::new(&preferred_fork()).unwrap().summary();
            summary.config_hash = Hash256::from_bytes([0x82; 32]);
            encode_frame_v2(FrameRefV2::Begin(summary)).unwrap()
        } else {
            vec![0; FRAME_HEADER_LEN_V2]
        };
        stream.write_all(&response).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let values = failure(&child.finish(), 1);
        assert_eq!(
            values["error"],
            if wrong_network {
                "peer_wrong_network"
            } else {
                "invalid_peer_history"
            }
        );
        assert!(!values.contains_key("operation_may_have_completed"));
        assert_no_connection(&source);
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn valid_end_without_terminal_eof_remains_a_receive_failure_without_adoption() {
    let _guard = process_guard();
    let scratch = ready("rejoin-no-eof");
    let before = snapshot(&scratch.0);
    let source = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        source.local_addr().unwrap(),
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--timeout-ms", "1000");
    let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&preferred_fork())).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    let values = failure(&child.finish(), 1);
    assert_eq!(values["error"], "peer_deadline");
    assert!(!values.contains_key("operation_may_have_completed"));
    drop(stream);
    assert_no_connection(&source);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn sigint_and_sigterm_in_real_terminal_receive_discard_the_validated_candidate() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("rejoin-receive-stop");
        let before = snapshot(&scratch.0);
        let source = reserve();
        let occupied = reserve();
        let args = arguments(
            &scratch,
            source.local_addr().unwrap(),
            occupied.local_addr().unwrap(),
        );
        let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
        let mut stream = accept_request(&source);
        stream.write_all(&transcript(&candidate)).unwrap();
        child.wait_outbound_receiving(&stream, 1);
        assert_busy(&scratch);
        assert_eq!(child.directory_owners(&scratch).len(), 2);
        assert!(!child.listener_is_owned());
        child.signal(signal);
        stream.shutdown(Shutdown::Write).unwrap();
        stopped(
            &child.finish(),
            Normal::retained(&candidate, "not-attempted", "shutdown"),
        );
        assert_no_connection(&source);
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn sigterm_during_a_partial_frame_drains_receipt_then_discards_without_adoption() {
    let _guard = process_guard();
    let scratch = ready("rejoin-partial-receive-stop");
    let before = snapshot(&scratch.0);
    let candidate = preferred_fork();
    let source = reserve();
    let occupied = reserve();
    let args = arguments(
        &scratch,
        source.local_addr().unwrap(),
        occupied.local_addr().unwrap(),
    );
    let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&source);
    let response = transcript(&candidate);
    stream.write_all(&response[..1]).unwrap();
    child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2 - 1);
    assert_busy(&scratch);
    child.signal(libc::SIGTERM);
    stream.write_all(&response[1..]).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    stopped(
        &child.finish(),
        Normal::retained(&candidate, "not-attempted", "shutdown"),
    );
    assert_no_connection(&source);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn sigint_during_receive_preserves_the_original_deadline_and_operational_error() {
    let _guard = process_guard();
    let scratch = ready("rejoin-receive-original-deadline");
    let before = snapshot(&scratch.0);
    let source = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        source.local_addr().unwrap(),
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--timeout-ms", "1000");
    let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&preferred_fork())).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    let observed_at = Instant::now();
    // The exact child socket/EOF syscall established reception before this
    // delay. The delay only places the signal inside the existing allowance.
    thread::sleep(Duration::from_millis(650));
    child.signal(libc::SIGINT);
    let values = failure(&child.finish(), 1);
    assert_eq!(values["error"], "peer_deadline");
    assert!(!values.contains_key("operation_may_have_completed"));
    assert!(
        observed_at.elapsed() < Duration::from_millis(1450),
        "shutdown renewed the receive deadline"
    );
    drop(stream);
    assert_no_connection(&source);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn receive_and_service_retain_both_directory_descriptors_and_refuse_competing_sync() {
    let _guard = process_guard();
    let scratch = ready("rejoin-owner-continuity");
    let candidate = preferred_fork();
    let source = reserve();
    let reservation = reserve();
    let mut args = arguments(
        &scratch,
        source.local_addr().unwrap(),
        reservation.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "2");
    let mut child = RejoinProcess::spawn(&args, reservation);
    let mut stream = accept_request(&source);
    child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2);
    let owners = child.directory_owners(&scratch);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    stream.write_all(&transcript(&candidate)).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    child.wait_listening();
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_busy(&scratch);
    let address = child.address();
    let mut requester = connect(address);
    requester
        .write_all(&encode_history_request_v2(independent_identity()))
        .unwrap();
    child.wait_receiving(&requester, 1);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_busy(&scratch);
    requester.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut requester), transcript(&candidate));
    let mut expected = Normal::adopted(&candidate, "attempt-limit");
    expected.service = Service::complete(receive_bytes(&candidate));
    stopped(&child.finish(), expected);
    assert_no_connection(&source);
    assert_port_closed(address);
    selected(&scratch, &candidate, 1);
}

#[test]
fn sigint_and_sigterm_stop_idle_service_with_the_already_adopted_selection() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("rejoin-idle-stop");
        let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
        let reservation = reserve();
        let args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
        let mut child = RejoinProcess::spawn(&args, reservation);
        child.wait_listening();
        let address = child.address();
        assert_busy(&scratch);
        let signaled_at = Instant::now();
        child.signal(signal);
        stopped(&child.finish(), Normal::adopted(&candidate, "shutdown"));
        assert!(signaled_at.elapsed() < Duration::from_secs(3));
        peer.finish();
        assert_port_closed(address);
        selected(&scratch, &candidate, 1);
    }
}

#[test]
fn active_service_shutdown_drains_only_the_admitted_request_and_rejects_queued_work() {
    let _guard = process_guard();
    let scratch = ready("rejoin-active-stop");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
    let mut child = RejoinProcess::spawn(&args, reservation);
    child.wait_listening();
    let address = child.address();
    let request = encode_history_request_v2(independent_identity());
    let mut first = connect(address);
    first.write_all(&request[..1]).unwrap();
    child.wait_receiving(&first, REQUEST_LEN_V2 - 1);
    assert_busy(&scratch);
    let mut queued = connect(address);
    queued.write_all(&request).unwrap();
    queued.shutdown(Shutdown::Write).unwrap();
    no_response_yet(&mut queued);
    child.signal(libc::SIGTERM);
    first.write_all(&request[1..]).unwrap();
    first.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut first), transcript(&candidate));
    let mut expected = Normal::adopted(&candidate, "shutdown");
    expected.service = Service::complete(receive_bytes(&candidate));
    stopped(&child.finish(), expected);
    assert!(read_response(&mut queued).is_empty());
    peer.finish();
    assert_port_closed(address);
    selected(&scratch, &candidate, 1);
}

#[test]
fn service_missing_eof_under_shutdown_keeps_its_original_deadline_and_phase_counters() {
    let _guard = process_guard();
    let scratch = ready("rejoin-service-original-deadline");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let mut args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
    set(&mut args, "--timeout-ms", "1000");
    let mut child = RejoinProcess::spawn(&args, reservation);
    child.wait_listening();
    let mut first = connect(child.address());
    first
        .write_all(&encode_history_request_v2(independent_identity()))
        .unwrap();
    child.wait_receiving(&first, 1);
    let observed_at = Instant::now();
    thread::sleep(Duration::from_millis(650));
    child.signal(libc::SIGINT);
    let mut expected = Normal::adopted(&candidate, "shutdown");
    expected.service = Service {
        completed: 0,
        rejected: 1,
        bytes: REQUEST_LEN_V2 as u64,
        last_error: "peer_deadline",
    };
    stopped(&child.finish(), expected);
    assert!(
        observed_at.elapsed() < Duration::from_millis(1450),
        "shutdown renewed the service deadline"
    );
    assert!(read_response(&mut first).is_empty());
    peer.finish();
    selected(&scratch, &candidate, 1);
}

#[test]
fn admission_window_excludes_receive_and_only_its_admitted_service_can_drain_after_expiry() {
    let _guard = process_guard();
    let scratch = ready("rejoin-admission-window");
    let candidate = preferred_fork();
    let source = reserve();
    let reservation = reserve();
    let mut args = arguments(
        &scratch,
        source.local_addr().unwrap(),
        reservation.local_addr().unwrap(),
    );
    set(&mut args, "--admission-ms", "500");
    let mut child = RejoinProcess::spawn(&args, reservation);
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&candidate)).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    // Deliberately exceed the service window while a verified receive is still
    // active. The service admission clock must not have started yet.
    thread::sleep(Duration::from_millis(650));
    stream.shutdown(Shutdown::Write).unwrap();
    child.wait_listening();
    let request = encode_history_request_v2(independent_identity());
    let mut first = connect(child.address());
    first.write_all(&request[..1]).unwrap();
    child.wait_receiving(&first, REQUEST_LEN_V2 - 1);
    let mut queued = connect(child.address());
    queued.write_all(&request).unwrap();
    queued.shutdown(Shutdown::Write).unwrap();
    no_response_yet(&mut queued);
    // Admission is already established by exact FD/syscall observation. This
    // finite delay only ensures the service admission window has expired.
    thread::sleep(Duration::from_millis(550));
    first.write_all(&request[1..]).unwrap();
    first.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut first), transcript(&candidate));
    let mut expected = Normal::adopted(&candidate, "admission-timeout");
    expected.service = Service::complete(receive_bytes(&candidate));
    stopped(&child.finish(), expected);
    assert!(read_response(&mut queued).is_empty());
    assert_no_connection(&source);
    selected(&scratch, &candidate, 1);
}

#[test]
fn post_adoption_bind_failure_reports_uncertainty_and_preserves_the_selected_candidate() {
    let _guard = process_guard();
    let scratch = ready("rejoin-post-adoption-bind");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let occupied = reserve();
    let mut args = arguments(&scratch, peer.address(), occupied.local_addr().unwrap());
    set(&mut args, "--max-attempts", "2");
    uncertain(&command(&args), "listen_bind");
    assert_eq!(peer.finish().wire_bytes, receive_bytes(&candidate));
    assert_no_connection(&occupied);
    selected(&scratch, &candidate, 1);
    let before = snapshot(&scratch.0);
    assert_eq!(
        failure(&command(&recover(&scratch, "retain-old")), 1)["error"],
        "no_pending_intent"
    );
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &candidate, 1);
}

#[test]
fn closed_stdout_reports_possible_completed_adoption_without_repeating_it() {
    let _guard = process_guard();
    let scratch = ready("rejoin-closed-stdout");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let address = reservation.local_addr().unwrap();
    let mut args = arguments(&scratch, peer.address(), address);
    set(&mut args, "--max-attempts", "1");
    let child = RejoinProcess::spawn_closed_stdout(&args, reservation);
    uncertain(&child.finish(), "output_io");
    peer.finish();
    assert_port_closed(address);
    selected(&scratch, &candidate, 1);
}

#[test]
fn blocked_final_stdout_releases_listener_and_both_owners_before_the_write_finishes() {
    let _guard = process_guard();
    let scratch = ready("rejoin-blocked-stdout");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
    let mut child = RejoinProcess::spawn_blocked_stdout(&args, reservation);
    child.wait_listening();
    assert_busy(&scratch);
    let address = child.address();
    child.signal(libc::SIGTERM);
    child.wait_stdout_blocked();
    assert!(child.running());
    assert!(!child.listener_is_owned());
    assert_port_closed(address);
    // Actual status and strict writer/full replay succeed while the original
    // child remains alive and blocked in its final output write.
    selected(&scratch, &candidate, 1);
    let after = snapshot(&scratch.0);
    assert!(child.running());
    child.close_output_reader();
    uncertain(&child.finish(), "output_io");
    peer.finish();
    assert_eq!(snapshot(&scratch.0), after);
}

#[test]
fn forced_death_during_real_partial_receive_releases_owners_without_publishing_candidate() {
    let _guard = process_guard();
    let scratch = ready("rejoin-killed-receive");
    let before = snapshot(&scratch.0);
    let source = reserve();
    let reservation = reserve();
    let address = reservation.local_addr().unwrap();
    let args = arguments(&scratch, source.local_addr().unwrap(), address);
    let mut child = RejoinProcess::spawn(&args, reservation);
    let mut stream = accept_request(&source);
    stream
        .write_all(&transcript(&preferred_fork())[..1])
        .unwrap();
    child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2 - 1);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    let output = child.kill();
    assert_eq!(output.status.signal(), Some(libc::SIGKILL));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_port_closed(address);
    assert_no_connection(&source);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn forced_death_during_adopted_service_preserves_the_complete_new_selection() {
    let _guard = process_guard();
    let scratch = ready("rejoin-killed-service");
    let candidate = preferred_fork();
    let peer = Peer::spawn(candidate.clone(), PeerMode::Good);
    let reservation = reserve();
    let args = arguments(&scratch, peer.address(), reservation.local_addr().unwrap());
    let mut child = RejoinProcess::spawn(&args, reservation);
    child.wait_listening();
    let address = child.address();
    let mut first = connect(address);
    first
        .write_all(&encode_history_request_v2(independent_identity()))
        .unwrap();
    child.wait_receiving(&first, 1);
    assert_busy(&scratch);
    let adopted = snapshot(&scratch.0);
    let output = child.kill();
    assert_eq!(output.status.signal(), Some(libc::SIGKILL));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(read_response(&mut first).is_empty());
    peer.finish();
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), adopted);
    selected(&scratch, &candidate, 1);
}

#[test]
fn canonical_selected_pending_layout_refuses_rejoin_and_cannot_roll_back() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for confirmation in 0..3 {
        let scratch = ready("rejoin-pending-no-rollback");
        // This is a deliberately constructed canonical A1 layout. It tests B2
        // refusal/authority, not crash timing or an observed durable substep.
        let intent = pending(&scratch, &candidate);
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
        let source = reserve();
        let occupied = reserve();
        let args = arguments(
            &scratch,
            source.local_addr().unwrap(),
            occupied.local_addr().unwrap(),
        );
        let before = snapshot(&scratch.0);
        let values = failure(&command(&args), 1);
        assert_eq!(values["error"], "recovery_required");
        assert_eq!(values["next"], "explicit-recover");
        assert!(!values.contains_key("operation_may_have_completed"));
        assert_no_connection(&source);
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        let refused = failure(&command(&recover(&scratch, "retain-old")), 1);
        assert_eq!(
            refused["error"],
            if confirmation == 0 {
                "cannot_retain_selected_candidate"
            } else {
                "cannot_retain_confirmed_candidate"
            }
        );
        assert_eq!(snapshot(&scratch.0), before);
        let recovered = success(&command(&recover(&scratch, "complete")), 0, "recover");
        pins(&recovered, "selected", intent.candidate());
        selected(&scratch, &candidate, 1);
    }
}

#[test]
fn queued_sigterm_during_verified_live_adoption_finishes_adoption_without_binding() {
    let _guard = process_guard();
    let scratch = ready("rejoin-live-adoption-stop");
    let candidate = adoption_history();
    let old = main_chain(3);
    let expected_intent = IntentRecordV2::new(
        independent_identity(),
        checkpoint(&old, 0),
        old,
        candidate.clone(),
    )
    .unwrap();
    let source = reserve();
    // Keep occupied throughout: after successful adoption, any attempted bind
    // would be a failure instead of the required normal shutdown record.
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        source.local_addr().unwrap(),
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "2");
    let mut watch = IntentWatch::new(&control_root(&scratch));
    let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&candidate)).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    let owners = child.directory_owners(&scratch);
    stream.shutdown(Shutdown::Write).unwrap();
    watch.stop_on_intent(&mut child);
    // A publication event only prompted SIGSTOP. All phase evidence below is
    // verified after State:T; a missed phase fails, without retry or relabeling.
    verify_stopped_intent(&mut child, &scratch, &expected_intent);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_no_connection(&occupied);
    child.signal(libc::SIGTERM);
    child.wait_pending_termination();
    child.assert_stopped();
    child.signal(libc::SIGCONT);
    stopped(&child.finish(), Normal::adopted(&candidate, "shutdown"));
    assert_no_connection(&source);
    assert_no_connection(&occupied);
    selected(&scratch, &candidate, 1);
    // This proves graceful signal generation during live adoption. It does not
    // identify a particular fsync boundary or claim power-loss durability.
}

#[test]
fn source_connection_failure_never_serves_the_old_branch_or_changes_selection() {
    let _guard = process_guard();
    let scratch = ready("rejoin-source-refused");
    let before = snapshot(&scratch.0);
    let refused = reserve();
    let source = refused.local_addr().unwrap();
    let occupied = reserve();
    // Reserve the distinct listen address before releasing the source port, so
    // this test cannot accidentally recycle its own deliberately refused port.
    drop(refused);
    let args = arguments(&scratch, source, occupied.local_addr().unwrap());
    let values = failure(&command(&args), 1);
    assert_eq!(values["error"], "peer_unreachable");
    assert!(!values.contains_key("operation_may_have_completed"));
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}
