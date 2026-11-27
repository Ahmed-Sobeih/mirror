//! Actual bounded failover over independent synthetic stores and real processes.
//! Owned-child socket/FD observations establish signal phases without hooks.
#![cfg(target_os = "linux")]

mod support_failover_v2;

use std::{
    ffi::OsString,
    fs,
    io::Write,
    net::{Shutdown, TcpListener},
    os::unix::{ffi::OsStringExt, process::ExitStatusExt},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::{ManifestErrorV2, NetworkManifestV2};
use mirror_crypto::Hash256;
use mirror_network::v2::{
    HistoryErrorV2, HistoryFramesV2, HistoryReceiverV2,
    session::{MAX_SESSION_WIRE_BYTES_V2, REQUEST_LEN_V2, encode_history_request_v2},
};
use mirror_node::v2::NodeControlV2;
use mirror_protocol::v2::{FRAME_HEADER_LEN_V2, FrameRefV2, encode_frame_v2};

use support_failover_v2::{
    BAD_HEADER_BYTES, Normal, Peer, PeerMode, ProviderProcess, RejoinProcess, Service, Trace,
    accept_request, arguments, assert_busy, assert_no_connection, assert_port_closed, command,
    connect, failed, failure, output_lost, pending, pins, process_guard, provider, raw_request,
    read_response, ready, ready_with, receive_bytes, receive_complete, recover, remove, reserve,
    sanitized, selected, set,
    state::{
        Scratch, assert_complete_state, checkpoint, control_root,
        history::{fixture_bytes, fork_chain, independent_identity, main_chain},
        preferred_fork, snapshot,
    },
    stopped, success, sync, transcript,
};

fn reject_header(listener: &TcpListener) {
    let mut stream = accept_request(listener);
    stream.write_all(&[0; FRAME_HEADER_LEN_V2]).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
}

#[test]
fn all_required_options_and_bounded_peer_lists_fail_before_effects() {
    let _guard = process_guard();
    let scratch = Scratch::new("failover-invalid-options");
    let first = reserve();
    let second = reserve();
    let occupied = reserve();
    let peers = [first.local_addr().unwrap(), second.local_addr().unwrap()];
    let original = arguments(&scratch, &peers, occupied.local_addr().unwrap());
    let mut cases = Vec::new();
    for flag in [
        "--control-dir",
        "--store-dir",
        "--config-hash",
        "--genesis-hash",
        "--peers",
        "--listen",
        "--max-attempts",
        "--max-io-bytes",
        "--timeout-ms",
        "--admission-ms",
    ] {
        let mut args = original.clone();
        remove(&mut args, flag);
        cases.push(args);
    }
    for list in [
        String::new(),
        format!(",{}", peers[0]),
        format!("{},", peers[0]),
        format!("{},,{}", peers[0], peers[1]),
        format!("{}, {}", peers[0], peers[1]),
        format!("{},localhost:1234", peers[0]),
        format!("{},127.0.0.1:0", peers[0]),
        format!("{},bad,{}", peers[0], peers[1]),
        format!("{},{}", peers[0], peers[0]),
        format!("{},[::ffff:127.0.0.1]:{}", peers[0], peers[0].port()),
        "[::1]:1234,[0:0:0:0:0:0:0:1]:1234".to_owned(),
        "[fe80::1%1]:1234".to_owned(),
        "[fe80::1%0]:1234".to_owned(),
        "127.0.0.1:1,127.0.0.1:2,127.0.0.1:3,127.0.0.1:4,127.0.0.1:5".to_owned(),
    ] {
        let mut args = original.clone();
        set(&mut args, "--peers", list);
        cases.push(args);
    }
    for (flag, value) in [
        ("--listen", "localhost:1234".to_owned()),
        ("--listen", "127.0.0.1:0".to_owned()),
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
        let mut args = original.clone();
        set(&mut args, flag, value);
        cases.push(args);
    }
    for extra in [
        vec!["--peer", "127.0.0.1:1234"],
        vec!["--peers", "127.0.0.1:1234"],
        vec!["--action", "complete"],
        vec!["--attach-existing"],
        vec!["--watch", "yes"],
        vec!["extra"],
    ] {
        let mut args = original.clone();
        args.extend(extra.into_iter().map(OsString::from));
        cases.push(args);
    }
    let mut non_utf8 = original.clone();
    set(&mut non_utf8, "--peers", OsString::from_vec(vec![0xff]));
    cases.push(non_utf8);
    let mut count = original.clone();
    count.extend((0..13).map(|_| OsString::from("extra")));
    cases.push(count);
    let before = snapshot(&scratch.0);
    for args in cases {
        let output = command(&args);
        assert_eq!(failure(&output, 2)["error"], "usage");
        assert!(output.stderr.len() <= 4096);
        assert_eq!(snapshot(&scratch.0), before);
        assert_no_connection(&first);
        assert_no_connection(&second);
        assert_no_connection(&occupied);
        sanitized(&output, &scratch, &peers);
    }
}

#[test]
fn missing_pending_corrupt_mismatched_and_busy_state_refuse_every_peer() {
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
            Scratch::new("failover-missing")
        } else {
            ready("failover-refusal")
        };
        let first = reserve();
        let second = reserve();
        let occupied = reserve();
        let peers = [first.local_addr().unwrap(), second.local_addr().unwrap()];
        let mut args = arguments(&scratch, &peers, occupied.local_addr().unwrap());
        let mut owner = None;
        let class = match case {
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
                owner = Some(
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
        let output = command(&args);
        failed(
            &output,
            class,
            Trace {
                results: &["not-attempted", "not-attempted"],
                bytes: 0,
                stop: "startup",
            },
            false,
        );
        sanitized(&output, &scratch, &peers);
        assert_eq!(snapshot(&scratch.0), before);
        assert_no_connection(&first);
        assert_no_connection(&second);
        assert_no_connection(&occupied);
        drop(owner);
    }
}

fn three_nodes(malformed_first: bool) {
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
    let seed = ready_with("failover-three-seed", &candidate);
    let joining = ready("failover-three-joining");
    let requester = ready("failover-three-requester");
    assert_ne!(seed.0, joining.0);
    assert_ne!(joining.0, requester.0);
    assert_ne!(seed.0, requester.0);
    let seed_before = snapshot(&seed.0);
    let mut snapshots = None;
    for restart in [false, true] {
        let seed_port = reserve();
        let mut seed_args = provider::arguments(&seed, seed_port.local_addr().unwrap());
        set(&mut seed_args, "--max-attempts", "2");
        let mut seed_process = ProviderProcess::spawn(&seed_args, seed_port);
        seed_process.wait_listening();
        let first = reserve();
        let first_address = first.local_addr().unwrap();
        let unused = reserve();
        let service = reserve();
        let peers = [
            first_address,
            seed_process.address(),
            unused.local_addr().unwrap(),
        ];
        let mut args = arguments(&joining, &peers, service.local_addr().unwrap());
        set(&mut args, "--max-attempts", "3");
        // Reserve every other endpoint before exposing the deliberately refused port.
        let first = if malformed_first {
            Some(first)
        } else {
            drop(first);
            None
        };
        let mut child = RejoinProcess::spawn(&args, service);
        if let Some(first) = &first {
            reject_header(first);
        }
        child.wait_listening();
        assert_busy(&joining);
        let requester_result =
            success(&command(&sync(&requester, child.address())), 0, "sync-once");
        assert_eq!(
            requester_result["choice"],
            if restart {
                "keep-current"
            } else {
                "prefer-candidate"
            }
        );
        pins(&requester_result, "selected", checkpoint(&candidate, 1));
        let mut expected = Normal::adopted(&candidate, "attempt-limit");
        if restart {
            expected.previous = checkpoint(&candidate, 1);
            expected.choice = "keep-current";
        }
        expected.service = Service::complete(receive_bytes(&candidate));
        let class = if malformed_first {
            "invalid_peer_history"
        } else {
            "peer_unreachable"
        };
        let output = child.finish();
        stopped(
            &output,
            expected,
            Trace {
                results: &[class, "validated", "not-attempted"],
                bytes: receive_bytes(&candidate)
                    + if malformed_first { BAD_HEADER_BYTES } else { 0 },
                stop: "validated",
            },
        );
        sanitized(&output, &joining, &peers);
        assert_no_connection(&unused);
        if let Some(first) = &first {
            assert_no_connection(first);
        }
        assert!(seed_process.running());
        seed_process.signal(libc::SIGTERM);
        let seed_result = success(&seed_process.finish(), 0, "serve");
        assert_eq!(seed_result["attempts"], "1");
        assert_eq!(seed_result["completed_sessions"], "1");
        assert_eq!(
            seed_result["wire_bytes"],
            receive_bytes(&candidate).to_string()
        );
        selected(&seed, &candidate, 0);
        selected(&joining, &candidate, 1);
        selected(&requester, &candidate, 1);
        assert_eq!(snapshot(&seed.0), seed_before);
        let current = (snapshot(&joining.0), snapshot(&requester.0));
        if let Some(previous) = &snapshots {
            assert_eq!(&current, previous);
        } else {
            snapshots = Some(current);
        }
    }
}

#[test]
fn refused_first_then_real_seed_then_third_node_preserves_complete_restart_state() {
    let _guard = process_guard();
    three_nodes(false);
}

#[test]
fn malformed_first_then_real_seed_then_third_node_preserves_complete_restart_state() {
    let _guard = process_guard();
    three_nodes(true);
}

#[test]
fn first_valid_identical_shorter_and_tied_history_ends_acquisition() {
    let _guard = process_guard();
    let old = main_chain(3);
    for candidate in [old.clone(), main_chain(1), fork_chain()] {
        let scratch = ready("failover-first-valid");
        let before = snapshot(&scratch.0);
        let first = Peer::spawn(candidate.clone(), PeerMode::Good);
        let later = reserve();
        let service = reserve();
        let mut args = arguments(
            &scratch,
            &[first.address(), later.local_addr().unwrap()],
            service.local_addr().unwrap(),
        );
        set(&mut args, "--max-attempts", "2");
        let mut child = RejoinProcess::spawn(&args, service);
        child.wait_listening();
        let (received, bytes) = receive_complete(child.address());
        assert_complete_state(&received, &old);
        let mut expected = Normal::retained("keep-current", "attempt-limit");
        expected.service = Service::complete(bytes);
        stopped(
            &child.finish(),
            expected,
            Trace {
                results: &["validated", "not-attempted"],
                bytes: receive_bytes(&candidate),
                stop: "validated",
            },
        );
        assert_eq!(first.finish().wire_bytes, receive_bytes(&candidate));
        assert_no_connection(&later);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &old, 0);
    }
}

#[test]
fn eligible_incomplete_trailing_invalid_block_and_missing_eof_sources_advance() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    let frames = HistoryFramesV2::new(&candidate)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for mode in [
        PeerMode::EarlyClose,
        PeerMode::MissingEnd,
        PeerMode::Trailing,
        PeerMode::InvalidBlock,
        PeerMode::HoldAfterEnd,
    ] {
        let scratch = ready("failover-eligible-errors");
        let first = Peer::spawn(candidate.clone(), mode);
        let second = Peer::spawn(candidate.clone(), PeerMode::Good);
        let later = reserve();
        let occupied = reserve();
        let mut args = arguments(
            &scratch,
            &[
                first.address(),
                second.address(),
                later.local_addr().unwrap(),
            ],
            occupied.local_addr().unwrap(),
        );
        set(&mut args, "--max-attempts", "2");
        if mode == PeerMode::HoldAfterEnd {
            set(&mut args, "--timeout-ms", "1000");
        }
        let (class, response_bytes) = match mode {
            PeerMode::EarlyClose => (
                "peer_early_close",
                frames[..2].iter().map(Vec::len).sum::<usize>() + FRAME_HEADER_LEN_V2 + 9,
            ),
            PeerMode::MissingEnd => (
                "peer_early_close",
                frames[..frames.len() - 1].iter().map(Vec::len).sum(),
            ),
            PeerMode::Trailing => ("peer_trailing_data", transcript(&candidate).len() + 1),
            PeerMode::InvalidBlock => {
                let bad = encode_frame_v2(FrameRefV2::Block {
                    height: 1,
                    bytes: &fixture_bytes("wrong_root1_block"),
                })
                .unwrap();
                (
                    "invalid_peer_history",
                    frames[..3].iter().map(Vec::len).sum::<usize>() + bad.len(),
                )
            }
            PeerMode::HoldAfterEnd => ("peer_deadline", transcript(&candidate).len()),
            _ => unreachable!(),
        };
        stopped(
            &command(&args),
            Normal::adopted(&candidate, "attempt-limit"),
            Trace {
                results: &[class, "validated", "not-attempted"],
                bytes: (REQUEST_LEN_V2 + response_bytes) as u64 + receive_bytes(&candidate),
                stop: "validated",
            },
        );
        // Raw malformed peer totals are offered bytes, not the child's consumed bytes.
        first.finish();
        assert_eq!(second.finish().wire_bytes, receive_bytes(&candidate));
        assert_no_connection(&later);
        assert_no_connection(&occupied);
        selected(&scratch, &candidate, 1);
    }
}

#[test]
fn received_wrong_network_advances_without_changing_independent_authority() {
    let _guard = process_guard();
    let scratch = ready("failover-peer-network");
    let candidate = preferred_fork();
    let first = reserve();
    let second = Peer::spawn(candidate.clone(), PeerMode::Good);
    let later = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.address(),
            later.local_addr().unwrap(),
        ],
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "2");
    let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&first);
    let mut summary = HistoryFramesV2::new(&candidate).unwrap().summary();
    summary.config_hash = Hash256::from_bytes([0x82; 32]);
    let response = encode_frame_v2(FrameRefV2::Begin(summary)).unwrap();
    stream.write_all(&response).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    stopped(
        &child.finish(),
        Normal::adopted(&candidate, "attempt-limit"),
        Trace {
            results: &["peer_wrong_network", "validated", "not-attempted"],
            bytes: (REQUEST_LEN_V2 + response.len()) as u64 + receive_bytes(&candidate),
            stop: "validated",
        },
    );
    second.finish();
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    selected(&scratch, &candidate, 1);
}

#[test]
fn excluded_nested_manifest_configuration_error_is_fatal_without_next_contact() {
    let _guard = process_guard();
    let scratch = ready("failover-fatal-nested-config");
    let before = snapshot(&scratch.0);
    let candidate = preferred_fork();
    let first = reserve();
    let later = reserve();
    let occupied = reserve();
    let args = arguments(
        &scratch,
        &[first.local_addr().unwrap(), later.local_addr().unwrap()],
        occupied.local_addr().unwrap(),
    );
    let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&first);
    let begin = encode_frame_v2(FrameRefV2::Begin(
        HistoryFramesV2::new(&candidate).unwrap().summary(),
    ))
    .unwrap();
    let mut manifest = candidate.manifest().encode().unwrap();
    manifest[10] ^= 1; // First MCF2 byte inside an otherwise exact MNM2 envelope.
    assert!(matches!(
        NetworkManifestV2::decode_for(&manifest, independent_identity()),
        Err(ManifestErrorV2::Config(_))
    ));
    let frame = encode_frame_v2(FrameRefV2::Manifest(&manifest)).unwrap();
    let receiver = HistoryReceiverV2::new(independent_identity())
        .accept_frame(&begin)
        .unwrap();
    assert!(matches!(
        receiver.accept_frame(&frame),
        Err(HistoryErrorV2::Manifest(ManifestErrorV2::Config(_)))
    ));
    stream.write_all(&begin).unwrap();
    stream.write_all(&frame).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    failed(
        &child.finish(),
        "invalid_peer_history",
        Trace {
            results: &["invalid_peer_history", "not-attempted"],
            bytes: (REQUEST_LEN_V2 + begin.len() + frame.len()) as u64,
            stop: "fatal",
        },
        false,
    );
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn all_configured_peers_fail_without_serving_or_mutating_the_old_branch() {
    let _guard = process_guard();
    let scratch = ready("failover-all-failed");
    let before = snapshot(&scratch.0);
    let first = reserve();
    let second = reserve();
    let third = reserve();
    let fourth = reserve();
    let occupied = reserve();
    let peers = [
        first.local_addr().unwrap(),
        second.local_addr().unwrap(),
        third.local_addr().unwrap(),
        fourth.local_addr().unwrap(),
    ];
    let args = arguments(&scratch, &peers, occupied.local_addr().unwrap());
    let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    reject_header(&first);
    let stream = accept_request(&second);
    stream.shutdown(Shutdown::Write).unwrap();
    reject_header(&third);
    let fourth_stream = accept_request(&fourth);
    fourth_stream.shutdown(Shutdown::Write).unwrap();
    let output = child.finish();
    failed(
        &output,
        "peer_early_close",
        Trace {
            results: &[
                "invalid_peer_history",
                "peer_early_close",
                "invalid_peer_history",
                "peer_early_close",
            ],
            bytes: 2 * (BAD_HEADER_BYTES + REQUEST_LEN_V2 as u64),
            stop: "peers-exhausted",
        },
        false,
    );
    sanitized(&output, &scratch, &peers);
    assert_no_connection(&first);
    assert_no_connection(&second);
    assert_no_connection(&third);
    assert_no_connection(&fourth);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn one_attempt_preserves_the_received_error_and_leaves_later_entries_unattempted() {
    let _guard = process_guard();
    let scratch = ready("failover-attempt-cap");
    let before = snapshot(&scratch.0);
    let refused = reserve();
    let address = refused.local_addr().unwrap();
    let later = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        &[address, later.local_addr().unwrap()],
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "1");
    drop(refused);
    failed(
        &command(&args),
        "peer_unreachable",
        Trace {
            results: &["peer_unreachable", "not-attempted"],
            bytes: 0,
            stop: "attempt-limit",
        },
        false,
    );
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn successful_last_allowed_receive_adopts_without_binding_an_occupied_port() {
    let _guard = process_guard();
    let scratch = ready("failover-last-attempt-success");
    let candidate = preferred_fork();
    let first = reserve();
    let second = Peer::spawn(candidate.clone(), PeerMode::Good);
    let later = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.address(),
            later.local_addr().unwrap(),
        ],
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "2");
    let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    reject_header(&first);
    stopped(
        &child.finish(),
        Normal::adopted(&candidate, "attempt-limit"),
        Trace {
            results: &["invalid_peer_history", "validated", "not-attempted"],
            bytes: BAD_HEADER_BYTES + receive_bytes(&candidate),
            stop: "validated",
        },
    );
    second.finish();
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    selected(&scratch, &candidate, 1);
}

#[test]
fn failed_exchange_bytes_and_terminal_eof_capacity_are_shared_with_the_next_peer() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for spare in [0, 1] {
        let scratch = ready("failover-eof-capacity");
        let before = snapshot(&scratch.0);
        let first = reserve();
        let second = Peer::spawn(candidate.clone(), PeerMode::Good);
        let later = reserve();
        let occupied = reserve();
        let mut args = arguments(
            &scratch,
            &[
                first.local_addr().unwrap(),
                second.address(),
                later.local_addr().unwrap(),
            ],
            occupied.local_addr().unwrap(),
        );
        set(&mut args, "--max-attempts", "2");
        let charged = BAD_HEADER_BYTES + receive_bytes(&candidate);
        set(&mut args, "--max-io-bytes", (charged + spare).to_string());
        let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
        reject_header(&first);
        let output = child.finish();
        if spare == 0 {
            failed(
                &output,
                "peer_io_budget",
                Trace {
                    results: &["invalid_peer_history", "peer_io_budget", "not-attempted"],
                    bytes: charged,
                    stop: "wire-limit",
                },
                false,
            );
            assert_eq!(snapshot(&scratch.0), before);
            selected(&scratch, &main_chain(3), 0);
        } else {
            stopped(
                &output,
                Normal::adopted(&candidate, "attempt-limit"),
                Trace {
                    results: &["invalid_peer_history", "validated", "not-attempted"],
                    bytes: charged,
                    stop: "validated",
                },
            );
            selected(&scratch, &candidate, 1);
        }
        assert_eq!(second.finish().wire_bytes, receive_bytes(&candidate));
        assert_no_connection(&first);
        assert_no_connection(&later);
        assert_no_connection(&occupied);
    }
}

#[test]
fn payload_that_exceeds_positive_remaining_wire_capacity_is_fatal_before_next_peer() {
    let _guard = process_guard();
    let scratch = ready("failover-positive-wire-remainder");
    let before = snapshot(&scratch.0);
    let first = reserve();
    let later = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        &[first.local_addr().unwrap(), later.local_addr().unwrap()],
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--max-io-bytes", "85");
    let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&first);
    let begin = encode_frame_v2(FrameRefV2::Begin(
        HistoryFramesV2::new(&preferred_fork()).unwrap().summary(),
    ))
    .unwrap();
    stream.write_all(&begin[..FRAME_HEADER_LEN_V2]).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    failed(
        &child.finish(),
        "peer_io_budget",
        Trace {
            results: &["peer_io_budget", "not-attempted"],
            bytes: BAD_HEADER_BYTES,
            stop: "wire-limit",
        },
        false,
    );
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn acquisition_and_service_errors_remain_separate_after_subsequent_success() {
    let _guard = process_guard();
    let scratch = ready("failover-all-phase-counters");
    let candidate = preferred_fork();
    let first = reserve();
    let second = Peer::spawn(candidate.clone(), PeerMode::Good);
    let later = reserve();
    let service = reserve();
    let mut args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.address(),
            later.local_addr().unwrap(),
        ],
        service.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "4");
    let mut child = RejoinProcess::spawn(&args, service);
    reject_header(&first);
    child.wait_listening();
    let mut invalid = encode_history_request_v2(independent_identity());
    invalid[0] ^= 1;
    assert!(raw_request(child.address(), &invalid).is_empty());
    let (received, wire) = receive_complete(child.address());
    assert_complete_state(&received, &candidate);
    let mut expected = Normal::adopted(&candidate, "attempt-limit");
    expected.service = Service {
        completed: 1,
        rejected: 1,
        bytes: REQUEST_LEN_V2 as u64 + wire,
        last_error: "invalid_peer_request",
    };
    stopped(
        &child.finish(),
        expected,
        Trace {
            results: &["invalid_peer_history", "validated", "not-attempted"],
            bytes: BAD_HEADER_BYTES + receive_bytes(&candidate),
            stop: "validated",
        },
    );
    second.finish();
    assert_no_connection(&first);
    assert_no_connection(&later);
    selected(&scratch, &candidate, 1);
}

#[test]
fn failed_receive_under_sigint_or_sigterm_preserves_error_and_prevents_advancement() {
    let _guard = process_guard();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("failover-failed-receive-stop");
        let before = snapshot(&scratch.0);
        let first = reserve();
        let later = reserve();
        let occupied = reserve();
        let args = arguments(
            &scratch,
            &[first.local_addr().unwrap(), later.local_addr().unwrap()],
            occupied.local_addr().unwrap(),
        );
        let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
        let mut stream = accept_request(&first);
        child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2);
        assert_busy(&scratch);
        assert!(!child.listener_is_owned());
        child.signal(signal);
        stream.write_all(&[0; FRAME_HEADER_LEN_V2]).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        failed(
            &child.finish(),
            "invalid_peer_history",
            Trace {
                results: &["invalid_peer_history", "not-attempted"],
                bytes: BAD_HEADER_BYTES,
                stop: "shutdown",
            },
            false,
        );
        assert_no_connection(&first);
        assert_no_connection(&later);
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn a_later_validated_receive_under_shutdown_discards_the_candidate_and_keeps_evidence() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("failover-second-receive-stop");
        let before = snapshot(&scratch.0);
        let first = reserve();
        let second = reserve();
        let later = reserve();
        let occupied = reserve();
        let args = arguments(
            &scratch,
            &[
                first.local_addr().unwrap(),
                second.local_addr().unwrap(),
                later.local_addr().unwrap(),
            ],
            occupied.local_addr().unwrap(),
        );
        let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
        reject_header(&first);
        let mut stream = accept_request(&second);
        stream.write_all(&transcript(&candidate)).unwrap();
        child.wait_outbound_receiving(&stream, 1);
        assert_busy(&scratch);
        child.signal(signal);
        stream.shutdown(Shutdown::Write).unwrap();
        stopped(
            &child.finish(),
            Normal::retained("not-attempted", "shutdown"),
            Trace {
                results: &["invalid_peer_history", "validated", "not-attempted"],
                bytes: BAD_HEADER_BYTES + receive_bytes(&candidate),
                stop: "validated",
            },
        );
        assert_no_connection(&first);
        assert_no_connection(&second);
        assert_no_connection(&later);
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn shutdown_during_receive_keeps_the_original_deadline_and_last_peer_untouched() {
    let _guard = process_guard();
    let scratch = ready("failover-original-deadline");
    let before = snapshot(&scratch.0);
    let candidate = preferred_fork();
    let first = reserve();
    let later = reserve();
    let occupied = reserve();
    let mut args = arguments(
        &scratch,
        &[first.local_addr().unwrap(), later.local_addr().unwrap()],
        occupied.local_addr().unwrap(),
    );
    set(&mut args, "--timeout-ms", "1000");
    let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    let mut stream = accept_request(&first);
    stream.write_all(&transcript(&candidate)).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    let observed = Instant::now();
    // Socket/FD/syscall evidence precedes this timing placement; sleep is not admission.
    thread::sleep(Duration::from_millis(650));
    child.signal(libc::SIGINT);
    failed(
        &child.finish(),
        "peer_deadline",
        Trace {
            results: &["peer_deadline", "not-attempted"],
            bytes: receive_bytes(&candidate),
            stop: "shutdown",
        },
        false,
    );
    assert!(
        observed.elapsed() < Duration::from_millis(1450),
        "shutdown refreshed the original receive deadline"
    );
    drop(stream);
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}

#[test]
fn both_root_descriptors_remain_held_across_failed_receive_valid_receive_and_service() {
    let _guard = process_guard();
    let scratch = ready("failover-owner-continuity");
    let candidate = preferred_fork();
    let first = reserve();
    let second = reserve();
    let later = reserve();
    let service = reserve();
    let mut args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.local_addr().unwrap(),
            later.local_addr().unwrap(),
        ],
        service.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "3");
    let mut child = RejoinProcess::spawn(&args, service);
    let mut first_stream = accept_request(&first);
    child.wait_outbound_receiving(&first_stream, FRAME_HEADER_LEN_V2);
    let owners = child.directory_owners(&scratch);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    first_stream.write_all(&[0; FRAME_HEADER_LEN_V2]).unwrap();
    first_stream.shutdown(Shutdown::Write).unwrap();
    let mut second_stream = accept_request(&second);
    child.wait_outbound_receiving(&second_stream, FRAME_HEADER_LEN_V2);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    second_stream.write_all(&transcript(&candidate)).unwrap();
    second_stream.shutdown(Shutdown::Write).unwrap();
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
    stopped(
        &child.finish(),
        expected,
        Trace {
            results: &["invalid_peer_history", "validated", "not-attempted"],
            bytes: BAD_HEADER_BYTES + receive_bytes(&candidate),
            stop: "validated",
        },
    );
    assert_no_connection(&first);
    assert_no_connection(&second);
    assert_no_connection(&later);
    assert_port_closed(address);
    selected(&scratch, &candidate, 1);
}

#[test]
fn post_adoption_bind_failure_preserves_the_candidate_and_all_acquisition_evidence() {
    let _guard = process_guard();
    let scratch = ready("failover-post-adoption-bind");
    let candidate = preferred_fork();
    let first = reserve();
    let second = Peer::spawn(candidate.clone(), PeerMode::Good);
    let later = reserve();
    let occupied = reserve();
    let args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.address(),
            later.local_addr().unwrap(),
        ],
        occupied.local_addr().unwrap(),
    );
    let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    reject_header(&first);
    failed(
        &child.finish(),
        "listen_bind",
        Trace {
            results: &["invalid_peer_history", "validated", "not-attempted"],
            bytes: BAD_HEADER_BYTES + receive_bytes(&candidate),
            stop: "validated",
        },
        true,
    );
    second.finish();
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    selected(&scratch, &candidate, 1);
    let before = snapshot(&scratch.0);
    assert_eq!(
        failure(&command(&recover(&scratch, "retain-old")), 1)["error"],
        "no_pending_intent"
    );
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn closed_output_after_failover_reports_possible_completion_without_repeating_work() {
    let _guard = process_guard();
    let scratch = ready("failover-closed-output");
    let candidate = preferred_fork();
    let first = reserve();
    let second = Peer::spawn(candidate.clone(), PeerMode::Good);
    let later = reserve();
    let service = reserve();
    let address = service.local_addr().unwrap();
    let mut args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.address(),
            later.local_addr().unwrap(),
        ],
        address,
    );
    set(&mut args, "--max-attempts", "2");
    let child = RejoinProcess::spawn_closed_stdout(&args, service);
    reject_header(&first);
    output_lost(&child.finish());
    second.finish();
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_port_closed(address);
    selected(&scratch, &candidate, 1);
}

#[test]
fn blocked_final_output_releases_listener_and_both_owners_after_failover() {
    let _guard = process_guard();
    let scratch = ready("failover-blocked-output");
    let candidate = preferred_fork();
    let first = reserve();
    let second = Peer::spawn(candidate.clone(), PeerMode::Good);
    let later = reserve();
    let service = reserve();
    let args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.address(),
            later.local_addr().unwrap(),
        ],
        service.local_addr().unwrap(),
    );
    let mut child = RejoinProcess::spawn_blocked_stdout(&args, service);
    reject_header(&first);
    child.wait_listening();
    assert_busy(&scratch);
    let address = child.address();
    child.signal(libc::SIGTERM);
    child.wait_stdout_blocked();
    assert!(child.running());
    assert!(!child.listener_is_owned());
    assert_port_closed(address);
    selected(&scratch, &candidate, 1);
    let after = snapshot(&scratch.0);
    assert!(child.running());
    child.close_output_reader();
    output_lost(&child.finish());
    second.finish();
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_eq!(snapshot(&scratch.0), after);
}

#[test]
fn forced_death_during_second_partial_receive_cannot_publish_candidate_or_contact_third() {
    let _guard = process_guard();
    let scratch = ready("failover-killed-second-receive");
    let before = snapshot(&scratch.0);
    let first = reserve();
    let second = reserve();
    let later = reserve();
    let occupied = reserve();
    let args = arguments(
        &scratch,
        &[
            first.local_addr().unwrap(),
            second.local_addr().unwrap(),
            later.local_addr().unwrap(),
        ],
        occupied.local_addr().unwrap(),
    );
    let mut child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
    reject_header(&first);
    let mut stream = accept_request(&second);
    stream
        .write_all(&transcript(&preferred_fork())[..1])
        .unwrap();
    child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2 - 1);
    assert_busy(&scratch);
    assert_eq!(child.directory_owners(&scratch).len(), 2);
    assert!(!child.listener_is_owned());
    let output = child.kill();
    assert_eq!(output.status.signal(), Some(libc::SIGKILL));
    assert!(output.stdout.is_empty());
    drop(stream);
    assert_no_connection(&first);
    assert_no_connection(&second);
    assert_no_connection(&later);
    assert_no_connection(&occupied);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &main_chain(3), 0);
}
