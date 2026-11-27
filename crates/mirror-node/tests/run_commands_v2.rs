//! Actual bounded multiple-round execution over privately owned synthetic state.
//! Signals follow exact child socket/FD/syscall or inter-round sleep observations.
#![cfg(target_os = "linux")]

mod support_run_v2;

use std::{
    ffi::OsString,
    fs,
    io::Write,
    net::{Shutdown, TcpListener},
    os::unix::{ffi::OsStringExt, process::ExitStatusExt},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::{ChainV2, ManifestErrorV2, NetworkManifestV2};
use mirror_consensus::v2::mine_header_v2;
use mirror_core::v2::BlockV2;
use mirror_network::v2::{
    HistoryErrorV2, HistoryFramesV2, HistoryReceiverV2,
    session::{REQUEST_LEN_V2, encode_history_request_v2},
};
use mirror_node::v2::{NodeControlV2, records::IntentRecordV2};
use mirror_protocol::v2::{FRAME_HEADER_LEN_V2, FrameRefV2, encode_frame_v2};

use support_run_v2::{
    Acquisition, BAD_HEADER_BYTES, Error, IntentWatch, Normal, ProviderProcess, RejoinProcess,
    Round, Service, accept_request, adoption_history, arguments, assert_busy, assert_no_connection,
    assert_port_closed, command, common, connect, failed, failure, no_response_yet, output_lost,
    pending, pins, process_guard, provider, read_response, ready, ready_with, receive_bytes,
    receive_complete, recover, remove, reserve, sanitized, selected, set,
    state::{
        Scratch, assert_complete_state, checkpoint, control_root,
        history::{independent_identity, main_chain},
        preferred_fork, snapshot,
    },
    stopped, success, sync, transcript, verify_stopped_intent,
};

fn send_history(listener: &TcpListener, chain: &ChainV2) {
    let mut stream = accept_request(listener);
    stream.write_all(&transcript(chain)).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
}

fn reject_header(listener: &TcpListener) {
    let mut stream = accept_request(listener);
    stream.write_all(&[0; FRAME_HEADER_LEN_V2]).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
}

fn extended(mut chain: ChainV2) -> ChainV2 {
    let proposal = chain
        .prepare_candidate(chain.blocks().last().unwrap().header().timestamp() + 1, &[])
        .unwrap();
    let mined = mine_header_v2(
        *proposal.header(),
        chain.config().parameters().pow_bits,
        10_000,
    )
    .unwrap();
    chain
        .append(BlockV2::new(chain.config(), mined.header, proposal.operations()).unwrap())
        .unwrap();
    chain
}

fn invalid_manifest_frames(chain: &ChainV2) -> Vec<u8> {
    let begin = encode_frame_v2(FrameRefV2::Begin(
        HistoryFramesV2::new(chain).unwrap().summary(),
    ))
    .unwrap();
    let mut manifest = chain.manifest().encode().unwrap();
    manifest[10] ^= 1;
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
    [begin, frame].concat()
}

#[test]
fn all_run_options_round_wait_bounds_and_old_command_separation_fail_before_effects() {
    let _guard = process_guard();
    let scratch = Scratch::new("run-invalid-options");
    let source = reserve();
    let occupied = reserve();
    let peers = [source.local_addr().unwrap()];
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
        "--max-rounds",
        "--round-wait-ms",
    ] {
        let mut args = original.clone();
        remove(&mut args, flag);
        cases.push(args);
    }
    for (flag, values) in [
        (
            "--max-rounds",
            vec!["0", "5", "01", "+1", "-1", "1.0", "18446744073709551616"],
        ),
        (
            "--round-wait-ms",
            vec![
                "60001",
                "00",
                "01",
                "+0",
                "-1",
                "0.0",
                "18446744073709551616",
            ],
        ),
        ("--max-attempts", vec!["0", "9"]),
        ("--max-io-bytes", vec!["0", "157129489"]),
        ("--timeout-ms", vec!["0", "60001"]),
        ("--admission-ms", vec!["0", "60001"]),
    ] {
        for value in values {
            let mut args = original.clone();
            set(&mut args, flag, value);
            cases.push(args);
        }
    }
    for value in [
        "".to_owned(),
        format!("{},{}", peers[0], peers[0]),
        "[::1]:1234,[0:0:0:0:0:0:0:1]:1234".to_owned(),
        "[fe80::1%0]:1234".to_owned(),
        "localhost:1234".to_owned(),
        "127.0.0.1:0".to_owned(),
        "127.0.0.1:1,127.0.0.1:2,127.0.0.1:3,127.0.0.1:4,127.0.0.1:5".to_owned(),
    ] {
        let mut args = original.clone();
        set(&mut args, "--peers", value);
        cases.push(args);
    }
    for flag in ["--max-rounds", "--round-wait-ms"] {
        let mut args = original.clone();
        args.extend([flag.into(), "1".into()]);
        cases.push(args);
    }
    let mut args = original.clone();
    args.extend(["--peer".into(), peers[0].to_string().into()]);
    cases.push(args);
    let mut args = original.clone();
    set(&mut args, "--control-dir", OsString::from_vec(vec![0xff]));
    cases.push(args);
    let mut args = original.clone();
    set(&mut args, "--control-dir", "x".repeat(4097));
    cases.push(args);
    let mut args = original.clone();
    args.extend((0..8).map(|_| OsString::from("extra")));
    cases.push(args);
    let before = snapshot(&scratch.0);
    for args in cases {
        let output = command(&args);
        failure(&output, 2);
        assert_eq!(snapshot(&scratch.0), before);
        assert_no_connection(&source);
        assert_no_connection(&occupied);
        sanitized(&output, &scratch, &peers);
    }
    for operation in [
        "status",
        "init",
        "recover",
        "sync-once",
        "serve",
        "rejoin-serve",
        "rejoin-failover-serve",
    ] {
        for flag in ["--max-rounds", "--round-wait-ms"] {
            // Supply every old command's ordinary required arguments, so the
            // added B4-only flag is itself the reason admission must fail.
            let mut args = match operation {
                "status" => common(operation, &scratch),
                "init" => {
                    let selected = checkpoint(&main_chain(3), 0);
                    let mut args = common(operation, &scratch);
                    args.extend([
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
                    args
                }
                "recover" => recover(&scratch, "retain-old"),
                "sync-once" => sync(&scratch, peers[0]),
                "serve" => provider::arguments(&scratch, occupied.local_addr().unwrap()),
                "rejoin-serve" | "rejoin-failover-serve" => {
                    let mut args = original.clone();
                    args[0] = operation.into();
                    remove(&mut args, "--max-rounds");
                    remove(&mut args, "--round-wait-ms");
                    if operation == "rejoin-serve" {
                        remove(&mut args, "--peers");
                        args.extend(["--peer".into(), peers[0].to_string().into()]);
                    }
                    args
                }
                _ => unreachable!(),
            };
            args.extend([flag.into(), "1".into()]);
            failure(&command(&args), 2);
            assert_no_connection(&source);
            assert_no_connection(&occupied);
        }
    }
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn local_startup_refusals_precede_every_peer_and_any_round() {
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
            Scratch::new("run-missing")
        } else {
            ready("run-refusal")
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
            Error {
                limit: 2,
                peer_count: 2,
                rounds: &[],
                current: None,
                phase: "startup",
                admitted: false,
            },
        );
        sanitized(&output, &scratch, &peers);
        assert_eq!(snapshot(&scratch.0), before);
        assert_no_connection(&first);
        assert_no_connection(&second);
        assert_no_connection(&occupied);
        drop(owner);
    }
}

#[test]
fn real_seed_and_requester_observe_two_rounds_per_invocation_and_explicit_restart() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    let seed = ready_with("run-real-seed", &candidate);
    let joining = ready("run-real-joining");
    let requester = ready("run-real-requester");
    assert_ne!(seed.0, joining.0);
    assert_ne!(joining.0, requester.0);
    assert_ne!(seed.0, requester.0);
    let seed_before = snapshot(&seed.0);
    let mut prior = None;
    for restart in [false, true] {
        let seed_port = reserve();
        let mut seed_args = provider::arguments(&seed, seed_port.local_addr().unwrap());
        set(&mut seed_args, "--max-attempts", "2");
        let mut seed_process = ProviderProcess::spawn(&seed_args, seed_port);
        seed_process.wait_listening();
        let service = reserve();
        let mut args = arguments(
            &joining,
            &[seed_process.address()],
            service.local_addr().unwrap(),
        );
        set(&mut args, "--max-attempts", "4");
        set(&mut args, "--admission-ms", "1000");
        set(&mut args, "--round-wait-ms", "200");
        let mut child = RejoinProcess::spawn(&args, service);
        child.wait_listening();
        let owners = child.directory_owners(&joining);
        assert_busy(&joining);
        for index in 0..2 {
            if index != 0 {
                child.wait_between_rounds();
                assert_eq!(child.directory_owners(&joining), owners);
                assert_busy(&joining);
                child.wait_listening();
            }
            assert_eq!(child.directory_owners(&joining), owners);
            let result = success(&command(&sync(&requester, child.address())), 0, "sync-once");
            assert_eq!(
                result["choice"],
                if !restart && index == 0 {
                    "prefer-candidate"
                } else {
                    "keep-current"
                }
            );
            pins(&result, "selected", checkpoint(&candidate, 1));
        }
        let mut rounds = [
            Round::adopted(&candidate, 1, "admission-timeout"),
            Round::kept(&candidate, 1, "attempt-limit"),
        ];
        if restart {
            rounds[0].choice = "keep-current";
        }
        for round in &mut rounds {
            round.service = Service::complete(receive_bytes(&candidate));
        }
        let output = child.finish();
        let mut expected = Normal::new(&rounds, "attempt-limit");
        if restart {
            expected.initial = checkpoint(&candidate, 1);
        }
        stopped(&output, expected);
        sanitized(&output, &joining, &[seed_process.address()]);
        let seed_result = success(&seed_process.finish(), 0, "serve");
        assert_eq!(seed_result["attempts"], "2");
        assert_eq!(seed_result["completed_sessions"], "2");
        assert_eq!(
            seed_result["wire_bytes"],
            (2 * receive_bytes(&candidate)).to_string()
        );
        selected(&seed, &candidate, 0);
        selected(&joining, &candidate, 1);
        selected(&requester, &candidate, 1);
        assert_eq!(snapshot(&seed.0), seed_before);
        let now = (snapshot(&joining.0), snapshot(&requester.0));
        if let Some(previous) = &prior {
            assert_eq!(&now, previous);
        } else {
            prior = Some(now);
        }
    }
}

#[test]
fn two_preferred_histories_preserve_complete_state_and_root_ownership_across_rounds() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    assert!(second.checkpoint().height > first.checkpoint().height);
    assert_ne!(
        first.ledger().membership().encode().unwrap(),
        second.ledger().membership().encode().unwrap()
    );
    let scratch = ready("run-two-adoptions");
    let source = reserve();
    let service = reserve();
    let mut args = arguments(
        &scratch,
        &[source.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    set(&mut args, "--max-attempts", "4");
    set(&mut args, "--admission-ms", "1000");
    set(&mut args, "--round-wait-ms", "200");
    let mut child = RejoinProcess::spawn(&args, service);
    let mut first_stream = accept_request(&source);
    child.wait_outbound_receiving(&first_stream, FRAME_HEADER_LEN_V2);
    let owners = child.directory_owners(&scratch);
    assert_busy(&scratch);
    first_stream.write_all(&transcript(&first)).unwrap();
    first_stream.shutdown(Shutdown::Write).unwrap();
    child.wait_listening();
    assert_eq!(child.directory_owners(&scratch), owners);
    let (received, wire) = receive_complete(child.address());
    assert_complete_state(&received, &first);
    assert_eq!(wire, receive_bytes(&first));
    child.wait_between_rounds();
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    let competing = reserve();
    let before = snapshot(&scratch.0);
    assert_eq!(
        failure(
            &command(&sync(&scratch, competing.local_addr().unwrap())),
            1
        )["error"],
        "busy"
    );
    assert_no_connection(&competing);
    assert_eq!(snapshot(&scratch.0), before);
    let mut second_stream = accept_request(&source);
    child.wait_outbound_receiving(&second_stream, FRAME_HEADER_LEN_V2);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    second_stream.write_all(&transcript(&second)).unwrap();
    second_stream.shutdown(Shutdown::Write).unwrap();
    child.wait_listening();
    assert_eq!(child.directory_owners(&scratch), owners);
    let (received, wire) = receive_complete(child.address());
    assert_complete_state(&received, &second);
    assert_eq!(wire, receive_bytes(&second));
    let mut rounds = [
        Round::adopted(&first, 1, "admission-timeout"),
        Round::adopted(&second, 2, "attempt-limit"),
    ];
    rounds[0].service = Service::complete(receive_bytes(&first));
    rounds[1].service = Service::complete(receive_bytes(&second));
    stopped(&child.finish(), Normal::new(&rounds, "attempt-limit"));
    assert_no_connection(&source);
    selected(&scratch, &second, 2);
}

#[test]
fn first_valid_stale_peer_repeats_without_contacting_a_later_peer() {
    let _guard = process_guard();
    let old = main_chain(3);
    let scratch = ready("run-first-valid-stale");
    let before = snapshot(&scratch.0);
    let first = reserve();
    let later = reserve();
    let service = reserve();
    let mut args = arguments(
        &scratch,
        &[first.local_addr().unwrap(), later.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    set(&mut args, "--max-rounds", "3");
    set(&mut args, "--round-wait-ms", "0");
    let child = RejoinProcess::spawn(&args, service);
    for _ in 0..3 {
        send_history(&first, &old);
    }
    let mut round = Round::kept(&old, 0, "admission-timeout");
    round.acquisition.results = &["validated", "not-attempted"];
    let rounds = [round; 3];
    let mut expected = Normal::new(&rounds, "round-limit");
    expected.limit = 3;
    expected.peer_count = 2;
    stopped(&child.finish(), expected);
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &old, 0);
}

#[test]
fn each_round_restarts_failover_ordinals_and_retains_prior_receive_and_service_errors() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    let scratch = ready("run-repeated-failover");
    let first = reserve();
    let second = reserve();
    let service = reserve();
    let peers = [first.local_addr().unwrap(), second.local_addr().unwrap()];
    let mut args = arguments(&scratch, &peers, service.local_addr().unwrap());
    set(&mut args, "--max-attempts", "7");
    set(&mut args, "--max-rounds", "3");
    set(&mut args, "--admission-ms", "1000");
    let mut child = RejoinProcess::spawn(&args, service);
    reject_header(&first);
    send_history(&second, &candidate);
    child.wait_listening();
    let mut rejected = connect(child.address());
    child.wait_receiving(&rejected, REQUEST_LEN_V2);
    // Deliver the request EOF while this owned child is stopped, avoiding a
    // server-active-close race when the same service address is later rebound.
    child.signal(libc::SIGSTOP);
    child.wait_stopped();
    let mut invalid = encode_history_request_v2(independent_identity());
    invalid[0] ^= 1;
    rejected.write_all(&invalid).unwrap();
    rejected.shutdown(Shutdown::Write).unwrap();
    child.assert_stopped();
    child.signal(libc::SIGCONT);
    assert!(read_response(&mut rejected).is_empty());
    reject_header(&first);
    send_history(&second, &candidate);
    child.wait_listening();
    send_history(&first, &candidate);
    child.wait_listening();
    let (received, bytes) = receive_complete(child.address());
    assert_complete_state(&received, &candidate);
    assert_eq!(bytes, receive_bytes(&candidate));
    let mut rounds = [
        Round::adopted(&candidate, 1, "admission-timeout"),
        Round::kept(&candidate, 1, "admission-timeout"),
        Round::kept(&candidate, 1, "attempt-limit"),
    ];
    for round in &mut rounds[..2] {
        round.acquisition = Acquisition {
            results: &["invalid_peer_history", "validated"],
            bytes: BAD_HEADER_BYTES + receive_bytes(&candidate),
            stop: "validated",
        };
    }
    rounds[0].service = Service {
        completed: 0,
        rejected: 1,
        bytes: REQUEST_LEN_V2 as u64,
        last_error: "invalid_peer_request",
    };
    rounds[2].acquisition.results = &["validated", "not-attempted"];
    rounds[2].service = Service::complete(bytes);
    let mut expected = Normal::new(&rounds, "attempt-limit");
    expected.limit = 3;
    expected.peer_count = 2;
    let output = child.finish();
    stopped(&output, expected);
    sanitized(&output, &scratch, &peers);
    assert_no_connection(&first);
    assert_no_connection(&second);
    selected(&scratch, &candidate, 1);
}

#[test]
fn initial_all_peer_and_fatal_failure_end_the_invocation_without_retry_or_service() {
    let _guard = process_guard();
    for fatal in [false, true] {
        let scratch = ready("run-initial-all-failed");
        let before = snapshot(&scratch.0);
        let first = reserve();
        let second = reserve();
        let occupied = reserve();
        let peers = [first.local_addr().unwrap(), second.local_addr().unwrap()];
        let mut args = arguments(&scratch, &peers, occupied.local_addr().unwrap());
        set(&mut args, "--max-rounds", "4");
        set(&mut args, "--round-wait-ms", "60000");
        let child = RejoinProcess::spawn_occupied(&args, occupied.local_addr().unwrap());
        let trace = if fatal {
            let frames = invalid_manifest_frames(&preferred_fork());
            let mut stream = accept_request(&first);
            stream.write_all(&frames).unwrap();
            stream.shutdown(Shutdown::Write).unwrap();
            Acquisition {
                results: &["invalid_peer_history", "not-attempted"],
                bytes: REQUEST_LEN_V2 as u64 + frames.len() as u64,
                stop: "fatal",
            }
        } else {
            reject_header(&first);
            reject_header(&second);
            Acquisition {
                results: &["invalid_peer_history", "invalid_peer_history"],
                bytes: 2 * BAD_HEADER_BYTES,
                stop: "peers-exhausted",
            }
        };
        let mut expected = Error::acquisition(&[], trace);
        expected.limit = 4;
        let output = child.finish();
        failed(&output, "invalid_peer_history", expected);
        sanitized(&output, &scratch, &peers);
        assert_no_connection(&first);
        assert_no_connection(&second);
        assert_no_connection(&occupied);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &main_chain(3), 0);
    }
}

#[test]
fn later_all_peer_failure_keeps_prior_adoption_context_and_never_starts_another_round() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    let scratch = ready("run-later-all-failed");
    let first = reserve();
    let second = reserve();
    let service = reserve();
    let peers = [first.local_addr().unwrap(), second.local_addr().unwrap()];
    let mut args = arguments(&scratch, &peers, service.local_addr().unwrap());
    set(&mut args, "--max-rounds", "4");
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&first, &candidate);
    child.wait_listening();
    assert_no_connection(&second);
    reject_header(&first);
    reject_header(&second);
    let mut first_round = Round::adopted(&candidate, 1, "admission-timeout");
    first_round.acquisition.results = &["validated", "not-attempted"];
    let rounds = [first_round];
    let trace = Acquisition {
        results: &["invalid_peer_history", "invalid_peer_history"],
        bytes: 2 * BAD_HEADER_BYTES,
        stop: "peers-exhausted",
    };
    let mut expected = Error::acquisition(&rounds, trace);
    expected.limit = 4;
    let output = child.finish();
    failed(&output, "invalid_peer_history", expected);
    sanitized(&output, &scratch, &peers);
    assert_no_connection(&first);
    assert_no_connection(&second);
    selected(&scratch, &candidate, 1);
}

#[test]
fn later_fatal_receive_after_keep_current_still_reports_possible_completion() {
    let _guard = process_guard();
    let old = main_chain(3);
    let scratch = ready("run-keep-then-fatal");
    let before = snapshot(&scratch.0);
    let first = reserve();
    let later = reserve();
    let service = reserve();
    let peers = [first.local_addr().unwrap(), later.local_addr().unwrap()];
    let args = arguments(&scratch, &peers, service.local_addr().unwrap());
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&first, &old);
    child.wait_listening();
    assert_no_connection(&later);
    let frames = invalid_manifest_frames(&preferred_fork());
    let mut stream = accept_request(&first);
    stream.write_all(&frames).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let mut first_round = Round::kept(&old, 0, "admission-timeout");
    first_round.acquisition.results = &["validated", "not-attempted"];
    let rounds = [first_round];
    let trace = Acquisition {
        results: &["invalid_peer_history", "not-attempted"],
        bytes: REQUEST_LEN_V2 as u64 + frames.len() as u64,
        stop: "fatal",
    };
    let output = child.finish();
    failed(
        &output,
        "invalid_peer_history",
        Error::acquisition(&rounds, trace),
    );
    sanitized(&output, &scratch, &peers);
    assert_no_connection(&first);
    assert_no_connection(&later);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &old, 0);
}

#[test]
fn one_cumulative_attempt_allowance_covers_service_and_all_four_authorized_rounds() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for maximum in [1u8, 2, 3, 8] {
        let scratch = ready("run-shared-attempts");
        let source = reserve();
        let reservation = reserve();
        let address = reservation.local_addr().unwrap();
        let mut args = arguments(&scratch, &[source.local_addr().unwrap()], address);
        set(&mut args, "--max-attempts", maximum.to_string());
        set(&mut args, "--max-rounds", "4");
        set(&mut args, "--admission-ms", "1000");
        // A maximum wait is irrelevant when a stop already applies. The two
        // cases which advance use zero, so every observed request belongs to
        // this process's explicit next round rather than a restarted command.
        set(
            &mut args,
            "--round-wait-ms",
            if maximum <= 2 { "60000" } else { "0" },
        );
        let (mut child, mut occupied) = if maximum == 1 {
            (
                RejoinProcess::spawn_occupied(&args, address),
                Some(reservation),
            )
        } else {
            (RejoinProcess::spawn(&args, reservation), None)
        };
        let count = if maximum == 8 {
            4
        } else if maximum == 3 {
            2
        } else {
            1
        };
        let mut rounds = Vec::new();
        for index in 0..count {
            let last = index + 1 == count;
            let serves = maximum == 8 || (index == 0 && maximum > 1);
            if maximum == 3 && last {
                let mut stream = accept_request(&source);
                child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2);
                assert!(!child.listener_is_owned());
                let listener = TcpListener::bind(address).unwrap();
                listener.set_nonblocking(true).unwrap();
                occupied = Some(listener);
                stream.write_all(&transcript(&candidate)).unwrap();
                stream.shutdown(Shutdown::Write).unwrap();
            } else {
                send_history(&source, &candidate);
            }
            let reason = if last {
                "attempt-limit"
            } else {
                "admission-timeout"
            };
            let mut round = if index == 0 {
                Round::adopted(&candidate, 1, reason)
            } else {
                Round::kept(&candidate, 1, reason)
            };
            if serves {
                child.wait_listening();
                let (received, bytes) = receive_complete(address);
                assert_complete_state(&received, &candidate);
                assert_eq!(bytes, receive_bytes(&candidate));
                round.service = Service::complete(bytes);
            }
            rounds.push(round);
        }
        let mut expected = Normal::new(&rounds, "attempt-limit");
        expected.limit = 4;
        stopped(&child.finish(), expected);
        assert_no_connection(&source);
        if let Some(listener) = occupied {
            assert_no_connection(&listener);
        }
        selected(&scratch, &candidate, 1);
    }
}

#[test]
fn failed_receives_service_and_later_terminal_eof_share_one_actual_byte_allowance() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    let transferred = receive_bytes(&candidate);
    for spare in [0u64, 1] {
        let scratch = ready("run-shared-wire-eof");
        let first = reserve();
        let second = reserve();
        let service = reserve();
        let peers = [first.local_addr().unwrap(), second.local_addr().unwrap()];
        let mut args = arguments(&scratch, &peers, service.local_addr().unwrap());
        set(&mut args, "--max-attempts", "5");
        set(&mut args, "--admission-ms", "1000");
        set(
            &mut args,
            "--max-io-bytes",
            (2 * BAD_HEADER_BYTES + 3 * transferred + spare).to_string(),
        );
        let mut child = RejoinProcess::spawn(&args, service);
        reject_header(&first);
        send_history(&second, &candidate);
        child.wait_listening();
        let (received, bytes) = receive_complete(child.address());
        assert_complete_state(&received, &candidate);
        assert_eq!(bytes, transferred);
        reject_header(&first);
        send_history(&second, &candidate);
        let mut first_round = Round::adopted(&candidate, 1, "admission-timeout");
        first_round.acquisition = Acquisition {
            results: &["invalid_peer_history", "validated"],
            bytes: BAD_HEADER_BYTES + transferred,
            stop: "validated",
        };
        first_round.service = Service::complete(transferred);
        let output = child.finish();
        if spare == 0 {
            let rounds = [first_round];
            let trace = Acquisition {
                results: &["invalid_peer_history", "peer_io_budget"],
                bytes: BAD_HEADER_BYTES + transferred,
                stop: "wire-limit",
            };
            failed(
                &output,
                "peer_io_budget",
                Error::acquisition(&rounds, trace),
            );
        } else {
            let mut second_round = Round::kept(&candidate, 1, "attempt-limit");
            second_round.acquisition = first_round.acquisition;
            let rounds = [first_round, second_round];
            let mut expected = Normal::new(&rounds, "attempt-limit");
            expected.peer_count = 2;
            stopped(&output, expected);
        }
        sanitized(&output, &scratch, &peers);
        assert_no_connection(&first);
        assert_no_connection(&second);
        selected(&scratch, &candidate, 1);
    }
}

#[test]
fn sigint_and_sigterm_during_observed_wait_keep_both_owners_and_skip_the_next_round() {
    let _guard = process_guard();
    let candidate = preferred_fork();
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("run-wait-shutdown");
        let source = reserve();
        let service = reserve();
        let mut args = arguments(
            &scratch,
            &[source.local_addr().unwrap()],
            service.local_addr().unwrap(),
        );
        set(&mut args, "--max-rounds", "3");
        set(&mut args, "--round-wait-ms", "5000");
        let mut child = RejoinProcess::spawn(&args, service);
        let mut stream = accept_request(&source);
        child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2);
        let owners = child.directory_owners(&scratch);
        stream.write_all(&transcript(&candidate)).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        child.wait_listening();
        assert_eq!(child.directory_owners(&scratch), owners);
        child.wait_between_rounds();
        assert_eq!(child.directory_owners(&scratch), owners);
        assert_busy(&scratch);
        assert!(!child.listener_is_owned());
        assert_no_connection(&source);
        let stopped_at = Instant::now();
        child.signal(signal);
        let rounds = [Round::adopted(&candidate, 1, "admission-timeout")];
        let mut expected = Normal::new(&rounds, "shutdown");
        expected.limit = 3;
        let address = child.address();
        stopped(&child.finish(), expected);
        assert!(
            stopped_at.elapsed() < Duration::from_millis(1500),
            "shutdown waited through the configured inter-round delay"
        );
        assert_port_closed(address);
        assert_no_connection(&source);
        selected(&scratch, &candidate, 1);
    }
}

#[test]
fn later_validated_receive_under_sigint_or_sigterm_is_discarded_before_admission() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    for signal in [libc::SIGINT, libc::SIGTERM] {
        let scratch = ready("run-later-validated-stop");
        let source = reserve();
        let service = reserve();
        let mut args = arguments(
            &scratch,
            &[source.local_addr().unwrap()],
            service.local_addr().unwrap(),
        );
        set(&mut args, "--max-rounds", "3");
        let mut child = RejoinProcess::spawn(&args, service);
        send_history(&source, &first);
        child.wait_listening();
        let owners = child.directory_owners(&scratch);
        let mut stream = accept_request(&source);
        stream.write_all(&transcript(&second)).unwrap();
        child.wait_outbound_receiving(&stream, 1);
        assert_eq!(child.directory_owners(&scratch), owners);
        assert_busy(&scratch);
        assert!(!child.listener_is_owned());
        let before = snapshot(&scratch.0);
        child.signal(signal);
        stream.shutdown(Shutdown::Write).unwrap();
        let rounds = [Round::adopted(&first, 1, "admission-timeout")];
        let mut expected = Normal::new(&rounds, "shutdown");
        expected.limit = 3;
        expected.incomplete = Some(Acquisition::good(&second));
        let address = child.address();
        stopped(&child.finish(), expected);
        assert_port_closed(address);
        assert_no_connection(&source);
        assert_eq!(snapshot(&scratch.0), before);
        selected(&scratch, &first, 1);
    }
}

#[test]
fn later_receive_failure_under_shutdown_keeps_its_original_deadline_and_sticky_context() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    let scratch = ready("run-later-receive-deadline");
    let source = reserve();
    let service = reserve();
    let mut args = arguments(
        &scratch,
        &[source.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    set(&mut args, "--timeout-ms", "1000");
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&source, &first);
    child.wait_listening();
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&second)).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    let observed_at = Instant::now();
    // The child is already blocked in this request's terminal-EOF read. This
    // delay places shutdown late in that existing allowance; it proves no phase.
    thread::sleep(Duration::from_millis(650));
    child.signal(libc::SIGINT);
    let rounds = [Round::adopted(&first, 1, "admission-timeout")];
    let trace = Acquisition {
        results: &["peer_deadline"],
        bytes: receive_bytes(&second),
        stop: "shutdown",
    };
    failed(
        &child.finish(),
        "peer_deadline",
        Error::acquisition(&rounds, trace),
    );
    assert!(
        observed_at.elapsed() < Duration::from_millis(1450),
        "shutdown renewed the later receive deadline"
    );
    drop(stream);
    assert_no_connection(&source);
    selected(&scratch, &first, 1);
}

#[test]
fn later_service_shutdown_drains_only_its_admitted_request_and_prevents_another_round() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    let scratch = ready("run-later-service-stop");
    let source = reserve();
    let service = reserve();
    let mut args = arguments(
        &scratch,
        &[source.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    set(&mut args, "--max-rounds", "3");
    set(&mut args, "--admission-ms", "1000");
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&source, &first);
    child.wait_listening();
    send_history(&source, &second);
    child.wait_listening();
    let address = child.address();
    let request = encode_history_request_v2(independent_identity());
    let mut admitted = connect(address);
    admitted.write_all(&request[..1]).unwrap();
    child.wait_receiving(&admitted, REQUEST_LEN_V2 - 1);
    assert_busy(&scratch);
    let mut queued = connect(address);
    queued.write_all(&request).unwrap();
    queued.shutdown(Shutdown::Write).unwrap();
    no_response_yet(&mut queued);
    child.signal(libc::SIGTERM);
    admitted.write_all(&request[1..]).unwrap();
    admitted.shutdown(Shutdown::Write).unwrap();
    assert_eq!(read_response(&mut admitted), transcript(&second));
    let mut rounds = [
        Round::adopted(&first, 1, "admission-timeout"),
        Round::adopted(&second, 2, "shutdown"),
    ];
    rounds[1].service = Service::complete(receive_bytes(&second));
    let mut expected = Normal::new(&rounds, "shutdown");
    expected.limit = 3;
    stopped(&child.finish(), expected);
    assert!(read_response(&mut queued).is_empty());
    assert_port_closed(address);
    assert_no_connection(&source);
    selected(&scratch, &second, 2);
}

#[test]
fn later_bind_failure_preserves_the_second_adoption_and_does_not_authorize_rollback() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    let scratch = ready("run-later-bind-failure");
    let source = reserve();
    let service = reserve();
    let args = arguments(
        &scratch,
        &[source.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&source, &first);
    child.wait_listening();
    let mut stream = accept_request(&source);
    child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2);
    assert!(!child.listener_is_owned());
    let occupied = TcpListener::bind(child.address()).unwrap();
    occupied.set_nonblocking(true).unwrap();
    stream.write_all(&transcript(&second)).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let rounds = [Round::adopted(&first, 1, "admission-timeout")];
    let mut expected = Error::acquisition(&rounds, Acquisition::good(&second));
    expected.phase = "adoption-service";
    failed(&child.finish(), "listen_bind", expected);
    assert_no_connection(&source);
    assert_no_connection(&occupied);
    selected(&scratch, &second, 2);
    let before = snapshot(&scratch.0);
    assert_eq!(
        failure(&command(&recover(&scratch, "retain-old")), 1)["error"],
        "no_pending_intent"
    );
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &second, 2);
}

#[test]
fn output_loss_after_two_adoptions_never_repeats_work_and_blocked_output_releases_owners() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    for blocked in [false, true] {
        let scratch = ready("run-later-output-loss");
        let source = reserve();
        let service = reserve();
        let mut args = arguments(
            &scratch,
            &[source.local_addr().unwrap()],
            service.local_addr().unwrap(),
        );
        set(&mut args, "--max-attempts", "2");
        let mut child = if blocked {
            RejoinProcess::spawn_blocked_stdout(&args, service)
        } else {
            RejoinProcess::spawn_closed_stdout(&args, service)
        };
        send_history(&source, &first);
        child.wait_listening();
        assert_busy(&scratch);
        send_history(&source, &second);
        let address = child.address();
        let before = if blocked {
            child.wait_stdout_blocked();
            assert!(child.running());
            assert!(!child.listener_is_owned());
            assert_port_closed(address);
            // Both actual status and strict writer/full replay succeed while
            // this invocation remains alive inside its final stdout write.
            selected(&scratch, &second, 2);
            let snapshot = snapshot(&scratch.0);
            assert!(child.running());
            child.close_output_reader();
            Some(snapshot)
        } else {
            None
        };
        output_lost(&child.finish());
        assert_port_closed(address);
        assert_no_connection(&source);
        if let Some(before) = before {
            assert_eq!(snapshot(&scratch.0), before);
        }
        selected(&scratch, &second, 2);
    }
}

#[test]
fn forced_death_during_a_later_partial_receive_retains_the_last_completed_checkpoint() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = extended(main_chain(4));
    let scratch = ready("run-killed-later-receive");
    let source = reserve();
    let service = reserve();
    let args = arguments(
        &scratch,
        &[source.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&source, &first);
    child.wait_listening();
    let owners = child.directory_owners(&scratch);
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&second)[..1]).unwrap();
    child.wait_outbound_receiving(&stream, FRAME_HEADER_LEN_V2 - 1);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_busy(&scratch);
    assert!(!child.listener_is_owned());
    let before = snapshot(&scratch.0);
    let address = child.address();
    let output = child.kill();
    assert_eq!(output.status.signal(), Some(libc::SIGKILL));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_no_connection(&source);
    assert_port_closed(address);
    assert_eq!(snapshot(&scratch.0), before);
    selected(&scratch, &first, 1);
}

#[test]
fn queued_sigterm_during_verified_second_adoption_finishes_it_without_binding() {
    let _guard = process_guard();
    let first = preferred_fork();
    let second = adoption_history();
    let expected_intent = IntentRecordV2::new(
        independent_identity(),
        checkpoint(&first, 1),
        first.clone(),
        second.clone(),
    )
    .unwrap();
    let scratch = ready("run-second-live-adoption");
    let source = reserve();
    let service = reserve();
    let args = arguments(
        &scratch,
        &[source.local_addr().unwrap()],
        service.local_addr().unwrap(),
    );
    let mut child = RejoinProcess::spawn(&args, service);
    send_history(&source, &first);
    child.wait_listening();
    let owners = child.directory_owners(&scratch);
    // Install after the first adoption has finished, so the watched intent can
    // only belong to the explicitly authorized second round.
    let mut watch = IntentWatch::new(&control_root(&scratch));
    let mut stream = accept_request(&source);
    stream.write_all(&transcript(&second)).unwrap();
    child.wait_outbound_receiving(&stream, 1);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert!(!child.listener_is_owned());
    let occupied = TcpListener::bind(child.address()).unwrap();
    occupied.set_nonblocking(true).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    watch.stop_on_intent(&mut child);
    // The event only prompted SIGSTOP. Verified stopped state, exact canonical
    // intent, both retained owners and absent listener establish live adoption.
    verify_stopped_intent(&mut child, &scratch, &expected_intent);
    assert_eq!(child.directory_owners(&scratch), owners);
    assert_no_connection(&occupied);
    child.signal(libc::SIGTERM);
    child.wait_pending_termination();
    child.assert_stopped();
    child.signal(libc::SIGCONT);
    let rounds = [
        Round::adopted(&first, 1, "admission-timeout"),
        Round::adopted(&second, 2, "shutdown"),
    ];
    stopped(&child.finish(), Normal::new(&rounds, "shutdown"));
    assert_no_connection(&source);
    assert_no_connection(&occupied);
    selected(&scratch, &second, 2);
    // This observes graceful signal generation during later synchronous
    // adoption, not a specific fsync boundary or power-loss durability.
}
