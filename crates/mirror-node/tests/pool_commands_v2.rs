//! Actual finite local pool commands over public synthetic keys and owned files.
//!
//! File/layout refusals and unchanged full replayed state are deterministic.
//! Only the final full-pipe write is used as a process-phase observation; these
//! tests do not claim raced-file snapshots, preempted admission, or power loss.
#![cfg(target_os = "linux")]

mod support_pool_commands_v2;
mod support_pool_v2;

use std::{
    ffi::{CString, OsString},
    fs,
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::symlink,
        process::ExitStatusExt,
    },
};

use mirror_node::v2::{NodeControlV2, NodeStatusV2, records::IntentRecordV2};
use mirror_personhood::v2::LimitsV2;
use mirror_storage::v2::ReplacementStoreV2;

use support_pool_commands_v2::*;
use support_pool_v2::*;

#[test]
fn finite_help_and_all_thirteen_required_options_preserve_effect_free_usage_refusals() {
    let _serial = process_guard();
    let scratch = Scratch::new("pool-command-shape");
    let chain = fresh();
    let base = arguments(
        &scratch,
        &chain,
        &scratch.0.join("synthetic-private-missing.hex"),
    );
    assert_eq!(base.len(), 27);
    let help = command(&["help".into()]);
    bounded(&help);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let text = std::str::from_utf8(&help.stdout).unwrap();
    assert!(text.contains("pool-inspect"));
    for flag in base.iter().skip(1).step_by(2) {
        assert!(text.contains(flag.to_str().unwrap()));
        let mut missing = base.clone();
        remove(&mut missing, flag.to_str().unwrap());
        failure(&unchanged_command(&scratch, &missing), 2, "usage");
        let mut duplicate = base.clone();
        duplicate.extend([flag.clone(), "synthetic-private-value".into()]);
        failure(&unchanged_command(&scratch, &duplicate), 2, "usage");
    }
    let mut unknown = base.clone();
    unknown.extend(["--unknown".into(), "synthetic-private-value".into()]);
    let mut cross = base.clone();
    cross.extend(["--peer".into(), "127.0.0.1:1".into()]);
    let mut old_command = base.clone();
    old_command[0] = "status".into();
    let mut missing_value = base.clone();
    missing_value.pop();
    for arguments in [unknown, cross, old_command, missing_value] {
        failure(&unchanged_command(&scratch, &arguments), 2, "usage");
    }
    assert!(!control_root(&scratch).exists());
    assert!(!scratch.root().exists());
}

#[test]
fn scalar_relations_utf8_controls_and_argument_budgets_precede_file_or_authority_access() {
    let _serial = process_guard();
    let scratch = Scratch::new("pool-command-values");
    let chain = fresh();
    let base = arguments(
        &scratch,
        &chain,
        &scratch.0.join("synthetic-private-missing.hex"),
    );
    for (flag, value) in [
        ("--max-input-bytes", "0"),
        ("--max-input-bytes", "149249"),
        ("--max-input-count", "0"),
        ("--max-input-count", "257"),
        ("--max-entries", "0"),
        ("--max-entries", "257"),
        ("--max-encoded-bytes", "0"),
        ("--max-encoded-bytes", "74497"),
        ("--max-batch-operations", "0"),
        ("--max-batch-operations", "65"),
        ("--max-batch-bytes", "167"),
        ("--max-batch-bytes", "19049"),
        ("--max-work-units", "0"),
        ("--max-work-units", "67108865"),
        ("--evaluate-count", "65"),
        ("--evaluate-count", "-1"),
        ("--evaluate-count", "00"),
        ("--evaluate-count", "+1"),
        ("--max-input-count", "01"),
        ("--max-input-count", "1.0"),
        ("--max-work-units", "18446744073709551616"),
        ("--config-hash", "ab"),
        ("--genesis-hash", "not-a-hash"),
        ("--input-file", ""),
        ("--input-file", "synthetic-private\ninput"),
    ] {
        let mut arguments = base.clone();
        set(&mut arguments, flag, value);
        failure(&unchanged_command(&scratch, &arguments), 2, "usage");
    }
    let mut batch_above_entries = base.clone();
    set(&mut batch_above_entries, "--max-entries", "1");
    let mut evaluation_above_batch = base.clone();
    set(&mut evaluation_above_batch, "--max-batch-operations", "1");
    set(&mut evaluation_above_batch, "--evaluate-count", "2");
    let mut non_utf8 = base.clone();
    set(
        &mut non_utf8,
        "--input-file",
        OsString::from_vec(vec![0xff]),
    );
    let mut long_token = base.clone();
    set(&mut long_token, "--input-file", "x".repeat(4097));
    let mut many = base.clone();
    many.extend((0..6).map(|_| OsString::from("x")));
    let mut total = base.clone();
    for flag in [
        "--control-dir",
        "--store-dir",
        "--input-file",
        "--config-hash",
        "--genesis-hash",
    ] {
        set(&mut total, flag, "x".repeat(4000));
    }
    for arguments in [
        batch_above_entries,
        evaluation_above_batch,
        non_utf8,
        long_token,
        many,
        total,
    ] {
        failure(&unchanged_command(&scratch, &arguments), 2, "usage");
    }
    assert!(!control_root(&scratch).exists());
    assert!(!scratch.root().exists());
}

#[test]
fn empty_regular_file_has_an_explicit_skip_and_exact_minimum_limits() {
    let _serial = process_guard();
    let mut chain = fresh();
    append(&mut chain, &[]);
    let fixture = Fixture::new("pool-command-empty", chain);
    let mut arguments = fixture.arguments();
    for (flag, value) in [
        ("--max-input-bytes", "1"),
        ("--max-input-count", "1"),
        ("--max-entries", "1"),
        ("--max-encoded-bytes", "1"),
        ("--max-batch-operations", "1"),
        ("--max-batch-bytes", "168"),
        ("--max-work-units", "1"),
    ] {
        set(&mut arguments, flag, value);
    }
    set(
        &mut arguments,
        "--input-file",
        fixture
            .scratch
            .0
            .join(".")
            .join(fixture.input.file_name().unwrap())
            .into_os_string(),
    );
    // Reversing option pairs establishes that no option ordering is required.
    let mut reordered = vec![arguments[0].clone()];
    for pair in arguments[1..].chunks_exact(2).rev() {
        reordered.extend_from_slice(pair);
    }
    let output = fixture.invoke(&reordered);
    let values = report(&output, 0, &fixture, &reordered);
    counts(&values, [0, 0, 0, 0, 0, 0]);
    evaluation(&values, "not-requested", "none", None, 0, 0);
    set(&mut reordered, "--evaluate-count", "1");
    let output = fixture.invoke(&reordered);
    let values = report(&output, 1, &fixture, &reordered);
    counts(&values, [0, 0, 0, 0, 0, 0]);
    evaluation(&values, "blocked", "invalid-count", None, 0, 0);
}

#[test]
fn malformed_hex_line_framing_is_rejected_in_full_before_opening_a_busy_controller() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-framing", fresh());
    let valid = records(&[transfer(&fixture.chain, 1, 2, 0, 1)]);
    let mut missing_lf = valid.clone();
    missing_lf.pop();
    let mut crlf = missing_lf.clone();
    crlf.extend_from_slice(b"\r\n");
    let mut trailing_blank = valid.clone();
    trailing_blank.push(b'\n');
    let mut trailing_junk = valid.clone();
    trailing_junk.extend_from_slice(b"synthetic-private-junk\n");
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&valid);
    let owner = NodeControlV2::open_writer(
        control_root(&fixture.scratch),
        fixture.scratch.root(),
        fixture.chain.manifest().identity(),
    )
    .unwrap();
    for bytes in [
        vec![b'\n'],
        b"0\n".to_vec(),
        b"000\n".to_vec(),
        b"AA\n".to_vec(),
        b"gg\n".to_vec(),
        b"00 \n".to_vec(),
        b"\t00\n".to_vec(),
        b"#comment\n".to_vec(),
        vec![0xff, b'\n'],
        [vec![b'0'; 584], vec![b'\n']].concat(),
        missing_lf,
        crlf,
        trailing_blank,
        trailing_junk,
        bom,
    ] {
        fixture.write(&bytes);
        failure(
            &unchanged_command(&fixture.scratch, &fixture.arguments()),
            1,
            "pool_input_framing",
        );
    }
    assert_complete_state(owner.chain(), &fixture.chain);
    drop(owner);
    fixture.assert_reopened();
}

#[test]
fn valid_hex_with_malformed_operation_encodings_aborts_without_partial_stdout() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-decoder", fresh());
    let first = transfer(&fixture.chain, 1, 2, 0, 1);
    let mut trailing = first.encode();
    trailing.push(0);
    for malformed in [
        vec![0],
        vec![1, 0, 0],
        vec![2, 0, 4],
        vec![2, 0, 0],
        vec![2, 0, 1],
        trailing,
    ] {
        let mut input = records(&[first]);
        input.extend_from_slice(&record(&malformed));
        fixture.write(&input);
        failure(&fixture.invoke(&fixture.arguments()), 1, "pool_operation");
    }
}

#[test]
fn exact_input_byte_and_record_caps_include_duplicates_and_the_hard_maximum() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-input-caps", fresh());
    let enrollment = enroll(&fixture.chain, 3, 8);
    assert_eq!(enrollment.encoded_len(), 291);
    let line = records(&[enrollment]);
    assert_eq!(line.len(), 583);
    let mut arguments = fixture.arguments();
    for (flag, value) in [
        ("--max-entries", "1"),
        ("--max-batch-operations", "1"),
        ("--max-encoded-bytes", "291"),
    ] {
        set(&mut arguments, flag, value);
    }
    for count in [255u64, 256] {
        let input = line.repeat(count as usize);
        fixture.write(&input);
        set(&mut arguments, "--max-input-bytes", input.len().to_string());
        set(&mut arguments, "--max-input-count", count.to_string());
        let output = fixture.invoke(&arguments);
        let values = report(&output, 0, &fixture, &arguments);
        // First admission: 2*(1+291); every duplicate: 1+291+one retained ID.
        counts(
            &values,
            [
                583 * count,
                count,
                1,
                count - 1,
                291,
                584 + 293 * (count - 1),
            ],
        );
        evaluation(&values, "not-requested", "none", None, 0, 0);
        set(
            &mut arguments,
            "--max-input-bytes",
            (input.len() - 1).to_string(),
        );
        failure(&fixture.invoke(&arguments), 1, "pool_input_bytes");
        set(&mut arguments, "--max-input-bytes", input.len().to_string());
        set(&mut arguments, "--max-input-count", (count - 1).to_string());
        failure(&fixture.invoke(&arguments), 1, "pool_input_count");
    }
    assert_eq!(line.len() * 256, 149_248);
    fixture.write(&line.repeat(257));
    set(&mut arguments, "--max-input-bytes", "149248");
    set(&mut arguments, "--max-input-count", "256");
    failure(&fixture.invoke(&arguments), 1, "pool_input_bytes");
    // Shorter records separate the hard count ceiling from the hard byte ceiling.
    let transfer_line = records(&[transfer(&fixture.chain, 1, 2, 0, 1)]);
    fixture.write(&transfer_line.repeat(257));
    failure(&fixture.invoke(&arguments), 1, "pool_input_count");
}

#[test]
fn missing_unsafe_and_nonregular_inputs_fail_without_following_links_or_waiting_for_fifo() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-paths", fresh());
    fixture.write(&records(&[transfer(&fixture.chain, 1, 2, 0, 1)]));
    let missing = fixture.scratch.0.join("synthetic-private-absent.hex");
    let directory = fixture.scratch.0.join("directory");
    fs::create_dir(&directory).unwrap();
    let linked = fixture.scratch.0.join("symlink.hex");
    symlink(&fixture.input, &linked).unwrap();
    let dangling = fixture.scratch.0.join("dangling.hex");
    symlink(&missing, &dangling).unwrap();
    let parent_link = fixture.scratch.0.join("parent-link");
    symlink(&directory, &parent_link).unwrap();
    fs::write(directory.join("child.hex"), []).unwrap();
    let hardlink = fixture.scratch.0.join("hardlink.hex");
    fs::hard_link(&fixture.input, &hardlink).unwrap();
    let fifo = fixture.scratch.0.join("writerless-fifo");
    let c_fifo = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    // SAFETY: the NUL-terminated path names a fresh private test entry; mode is valid.
    assert_eq!(unsafe { libc::mkfifo(c_fifo.as_ptr(), 0o600) }, 0);
    for (path, error) in [
        (missing.as_os_str().to_owned(), "pool_input_missing"),
        (directory.into_os_string(), "pool_input_unsafe"),
        (linked.into_os_string(), "pool_input_unsafe"),
        (dangling.into_os_string(), "pool_input_unsafe"),
        (
            parent_link.join("child.hex").into_os_string(),
            "pool_input_unsafe",
        ),
        (hardlink.into_os_string(), "pool_input_unsafe"),
        (fixture.input.as_os_str().to_owned(), "pool_input_unsafe"),
        (fifo.into_os_string(), "pool_input_unsafe"),
        ("-".into(), "pool_input_path"),
        (
            fixture
                .scratch
                .0
                .join("directory/../synthetic-private-input.hex")
                .into_os_string(),
            "pool_input_path",
        ),
    ] {
        let mut arguments = fixture.arguments();
        set(&mut arguments, "--input-file", path);
        failure(&fixture.invoke(&arguments), 1, error);
    }
}

#[test]
fn all_four_kinds_evaluate_once_against_the_pinned_parent_and_each_invocation_starts_empty() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-four-kinds", fresh());
    let enrollment = enroll(&fixture.chain, 3, 8);
    let Operation::Membership(member) = enrollment else {
        unreachable!()
    };
    let operations = [
        transfer(&fixture.chain, 1, 2, 0, 10),
        enrollment,
        approve(&fixture.chain, 1, 0, member.enrollment_id().unwrap(), 8),
        withdraw(&fixture.chain, 2, 0),
    ];
    fixture.write(&records(&operations));
    let mut arguments = fixture.arguments();
    set(&mut arguments, "--evaluate-count", "4");
    let first = fixture.invoke(&arguments);
    let values = report(&first, 0, &fixture, &arguments);
    counts(&values, [1980, 4, 4, 0, 988, 3009]);
    evaluation(&values, "evaluated", "none", None, 4, 1172);
    let again = fixture.invoke(&arguments);
    report(&again, 0, &fixture, &arguments);
    assert_eq!(again.stdout, first.stdout);
}

#[test]
fn exact_duplicates_at_full_retention_preserve_fifo_and_requested_count_is_not_shortened() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-duplicates", fresh());
    let first = transfer(&fixture.chain, 1, 2, 0, 10);
    let second = transfer(&fixture.chain, 1, 2, 1, 10);
    // Repeating the first entry after the second must not move it to the tail:
    // doing so would make the sequence-one transfer execute before sequence zero.
    fixture.write(&records(&[first, second, first]));
    let mut arguments = fixture.arguments();
    for (flag, value) in [
        ("--max-entries", "2"),
        ("--max-encoded-bytes", "486"),
        ("--max-batch-operations", "2"),
        ("--evaluate-count", "2"),
    ] {
        set(&mut arguments, flag, value);
    }
    let output = fixture.invoke(&arguments);
    let values = report(&output, 0, &fixture, &arguments);
    counts(&values, [1461, 3, 2, 1, 486, 1732]);
    evaluation(&values, "evaluated", "none", None, 2, 662);
    for (flag, value) in [
        ("--max-entries", "3"),
        ("--max-batch-operations", "3"),
        ("--evaluate-count", "3"),
    ] {
        set(&mut arguments, flag, value);
    }
    let output = fixture.invoke(&arguments);
    let values = report(&output, 1, &fixture, &arguments);
    counts(&values, [1461, 3, 2, 1, 486, 1223]);
    evaluation(&values, "blocked", "invalid-count", None, 0, 0);
}

#[test]
fn entry_and_retained_encoding_caps_are_independent_and_intake_refusal_is_all_or_nothing() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-retention", fresh());
    let first = transfer(&fixture.chain, 1, 2, 0, 1);
    let second = withdraw(&fixture.chain, 2, 0);
    fixture.write(&records(&[first, second]));
    let mut entries = fixture.arguments();
    set(&mut entries, "--max-entries", "1");
    set(&mut entries, "--max-batch-operations", "1");
    failure(&fixture.invoke(&entries), 1, "pool_entry_limit");
    let mut bytes = fixture.arguments();
    for cap in ["1", "469"] {
        set(&mut bytes, "--max-encoded-bytes", cap);
        failure(&fixture.invoke(&bytes), 1, "pool_encoded_byte_limit");
    }
    fixture.write(&records(&[first, second, first]));
    set(&mut bytes, "--max-encoded-bytes", "470");
    let output = fixture.invoke(&bytes);
    let values = report(&output, 0, &fixture, &bytes);
    counts(&values, [1429, 3, 2, 1, 470, 1191]);
    evaluation(&values, "not-requested", "none", None, 0, 0);
}

#[test]
fn contextual_and_signature_refusals_abort_after_earlier_valid_records_without_echoing_payloads() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-context", fresh());
    let first = transfer(&fixture.chain, 1, 2, 0, 1);
    let base = header(&fixture.chain, 1, 1, 8);
    let mut wrong_anchor = base;
    wrong_anchor.anchor_hash = tag(99);
    let mut wrong_context = base;
    wrong_context.config_hash = tag(99);
    let mut future = base;
    future.valid_from = 2;
    let mut inverted = base;
    inverted.valid_from = 3;
    inverted.valid_until = 2;
    let mut bad_transfer = transfer(&fixture.chain, 1, 2, 1, 1).encode();
    *bad_transfer.last_mut().unwrap() ^= 1;
    let mut bad_enrollment = enroll(&fixture.chain, 3, 8).encode();
    *bad_enrollment.last_mut().unwrap() ^= 1;
    for bytes in [
        transfer_header(1, wrong_anchor, 2, 1, 0).encode(),
        transfer_header(1, wrong_context, 2, 1, 0).encode(),
        transfer_header(1, future, 2, 1, 0).encode(),
        transfer_header(1, inverted, 2, 1, 0).encode(),
        transfer_header(1, base, 2, 1, 1).encode(),
        bad_transfer,
        bad_enrollment,
    ] {
        let mut input = records(&[first]);
        input.extend_from_slice(&record(&bytes));
        fixture.write(&input);
        failure(&fixture.invoke(&fixture.arguments()), 1, "pool_operation");
    }
}

#[test]
fn overspending_sequence_gaps_and_conflicts_are_retained_but_exact_evaluation_blocks() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-stateful", fresh());
    let cases = [
        (
            vec![
                transfer(&fixture.chain, 1, 2, 0, 70),
                transfer(&fixture.chain, 1, 2, 1, 70),
            ],
            1,
        ),
        (vec![transfer(&fixture.chain, 1, 2, 1, 1)], 0),
        (
            vec![
                transfer(&fixture.chain, 1, 2, 0, 10),
                transfer(&fixture.chain, 1, 2, 0, 11),
            ],
            1,
        ),
    ];
    for (operations, index) in cases {
        fixture.write(&records(&operations));
        let count = operations.len() as u64;
        let admitted_work = if count == 1 { 488 } else { 977 };
        let mut arguments = fixture.arguments();
        let output = fixture.invoke(&arguments);
        let values = report(&output, 0, &fixture, &arguments);
        counts(
            &values,
            [487 * count, count, count, 0, 243 * count, admitted_work],
        );
        evaluation(&values, "not-requested", "none", None, 0, 0);
        set(&mut arguments, "--evaluate-count", count.to_string());
        let output = fixture.invoke(&arguments);
        let values = report(&output, 1, &fixture, &arguments);
        counts(
            &values,
            [
                487 * count,
                count,
                count,
                0,
                243 * count,
                admitted_work + 15 + 247 * count,
            ],
        );
        evaluation(&values, "blocked", "ledger", Some(index), 0, 0);
        if count == 2 {
            set(&mut arguments, "--evaluate-count", "1");
            let output = fixture.invoke(&arguments);
            let values = report(&output, 0, &fixture, &arguments);
            evaluation(&values, "evaluated", "none", None, 1, 415);
        }
    }
}

#[test]
fn receive_before_spend_depends_on_fifo_and_blocked_index_uses_deduplicated_position() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-order", fresh());
    let receive = transfer(&fixture.chain, 1, 2, 0, 10);
    let spend = transfer(&fixture.chain, 2, 1, 0, 10);
    let mut arguments = fixture.arguments();
    set(&mut arguments, "--evaluate-count", "2");
    fixture.write(&records(&[receive, spend]));
    let output = fixture.invoke(&arguments);
    let values = report(&output, 0, &fixture, &arguments);
    counts(&values, [974, 2, 2, 0, 486, 1486]);
    evaluation(&values, "evaluated", "none", None, 2, 662);
    fixture.write(&records(&[spend, receive]));
    let output = fixture.invoke(&arguments);
    let values = report(&output, 1, &fixture, &arguments);
    counts(&values, [974, 2, 2, 0, 486, 1486]);
    evaluation(&values, "blocked", "ledger", Some(0), 0, 0);
    fixture.write(&records(&[
        receive,
        receive,
        transfer(&fixture.chain, 2, 1, 0, 11),
    ]));
    let output = fixture.invoke(&arguments);
    let values = report(&output, 1, &fixture, &arguments);
    counts(&values, [1461, 3, 2, 1, 486, 1731]);
    evaluation(&values, "blocked", "ledger", Some(1), 0, 0);
}

#[test]
fn late_mixed_operation_failure_discards_staged_transfer_and_membership_effects() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-late-mixed", fresh());
    let first = transfer(&fixture.chain, 1, 2, 0, 10);
    let enrollment = enroll(&fixture.chain, 3, 8);
    for (last, encoded, admitted, evaluated) in [
        (
            approve(&fixture.chain, 1, 0, tag(99), 8),
            761u64,
            1531,
            2319,
        ),
        (transfer(&fixture.chain, 1, 2, 1, 100), 777, 1563, 2367),
    ] {
        fixture.write(&records(&[first, enrollment, last]));
        let mut arguments = fixture.arguments();
        set(&mut arguments, "--evaluate-count", "3");
        let output = fixture.invoke(&arguments);
        let values = report(&output, 1, &fixture, &arguments);
        counts(&values, [2 * encoded + 3, 3, 3, 0, encoded, evaluated]);
        evaluation(&values, "blocked", "ledger", Some(2), 0, 0);
        set(&mut arguments, "--evaluate-count", "2");
        let output = fixture.invoke(&arguments);
        let values = report(&output, 0, &fixture, &arguments);
        counts(&values, [2 * encoded + 3, 3, 3, 0, encoded, admitted + 557]);
        evaluation(&values, "evaluated", "none", None, 2, 710);
    }
}

#[test]
fn complete_block_bytes_and_canonical_body_quotas_block_without_spending_evaluation_work() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-batch-bytes", fresh());
    fixture.write(&records(&[transfer(&fixture.chain, 1, 2, 0, 1)]));
    let mut arguments = fixture.arguments();
    set(&mut arguments, "--evaluate-count", "1");
    for cap in ["168", "414"] {
        set(&mut arguments, "--max-batch-bytes", cap);
        let output = fixture.invoke(&arguments);
        let values = report(&output, 1, &fixture, &arguments);
        counts(&values, [487, 1, 1, 0, 243, 488]);
        evaluation(&values, "blocked", "batch-byte-limit", None, 0, 0);
    }
    set(&mut arguments, "--max-batch-bytes", "415");
    let output = fixture.invoke(&arguments);
    let values = report(&output, 0, &fixture, &arguments);
    counts(&values, [487, 1, 1, 0, 243, 750]);
    evaluation(&values, "evaluated", "none", None, 1, 415);

    let chain = chain_with(
        LimitsV2 {
            max_operations: 1,
            max_membership_operations: 1,
            max_new_enrollments: 1,
            ..LimitsV2::default()
        },
        &[(1, 100), (2, 0)],
        1,
        30,
    );
    let fixture = Fixture::new("pool-command-body", chain);
    fixture.write(&records(&[
        transfer(&fixture.chain, 1, 2, 0, 1),
        transfer(&fixture.chain, 1, 2, 1, 1),
    ]));
    let mut arguments = fixture.arguments();
    set(&mut arguments, "--evaluate-count", "2");
    let output = fixture.invoke(&arguments);
    let values = report(&output, 1, &fixture, &arguments);
    counts(&values, [974, 2, 2, 0, 486, 977]);
    evaluation(&values, "blocked", "body", None, 0, 0);
}

#[test]
fn cumulative_work_is_shared_by_distinct_inputs_duplicates_and_the_one_evaluation() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-work", fresh());
    let first = transfer(&fixture.chain, 1, 2, 0, 1);
    let second = transfer(&fixture.chain, 1, 2, 1, 1);
    // Literal C1 charges: fresh transfer 488, second distinct 489, duplicate
    // with one retained ID 245; evaluation costs 262 for one or 509 for two.
    // No second PendingPool instance is used as this process test's oracle.
    for (operations, admitted, evaluation_count, evaluation_charge) in [
        ([first, second], 977u64, 2usize, 509u64),
        ([first, first], 733, 1, 262),
    ] {
        fixture.write(&records(&operations));
        let mut arguments = fixture.arguments();
        set(
            &mut arguments,
            "--max-work-units",
            (admitted - 1).to_string(),
        );
        failure(&fixture.invoke(&arguments), 1, "pool_work_limit");
        set(&mut arguments, "--max-work-units", admitted.to_string());
        let output = fixture.invoke(&arguments);
        let values = report(&output, 0, &fixture, &arguments);
        let retained = evaluation_count as u64;
        counts(
            &values,
            [974, 2, retained, 2 - retained, 243 * retained, admitted],
        );
        evaluation(&values, "not-requested", "none", None, 0, 0);
        set(
            &mut arguments,
            "--evaluate-count",
            evaluation_count.to_string(),
        );
        set(
            &mut arguments,
            "--max-work-units",
            (admitted + evaluation_charge - 1).to_string(),
        );
        let output = fixture.invoke(&arguments);
        let values = report(&output, 1, &fixture, &arguments);
        counts(
            &values,
            [974, 2, retained, 2 - retained, 243 * retained, admitted],
        );
        evaluation(&values, "blocked", "work-limit", None, 0, 0);
        set(
            &mut arguments,
            "--max-work-units",
            (admitted + evaluation_charge).to_string(),
        );
        let output = fixture.invoke(&arguments);
        let values = report(&output, 0, &fixture, &arguments);
        counts(
            &values,
            [
                974,
                2,
                retained,
                2 - retained,
                243 * retained,
                admitted + evaluation_charge,
            ],
        );
        evaluation(
            &values,
            "evaluated",
            "none",
            None,
            evaluation_count,
            168 + 247 * evaluation_count,
        );
    }
}

#[test]
fn clean_only_authority_preserves_missing_wrong_busy_corrupt_and_unknown_layout_refusals() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-authority", fresh());
    fixture.write(&records(&[transfer(&fixture.chain, 1, 2, 0, 1)]));
    for flag in ["--config-hash", "--genesis-hash"] {
        let mut arguments = fixture.arguments();
        set(&mut arguments, flag, tag(99).to_string());
        failure(&fixture.invoke(&arguments), 1, "wrong_network");
    }
    for (root, error) in [
        (control_root(&fixture.scratch), "missing_control"),
        (fixture.scratch.root(), "missing_store"),
    ] {
        let saved = fixture.scratch.0.join("saved-root");
        fs::rename(&root, &saved).unwrap();
        failure(
            &unchanged_command(&fixture.scratch, &fixture.arguments()),
            1,
            error,
        );
        fs::rename(&saved, &root).unwrap();
        fixture.assert_reopened();
    }
    let owner = NodeControlV2::open_writer(
        control_root(&fixture.scratch),
        fixture.scratch.root(),
        fixture.chain.manifest().identity(),
    )
    .unwrap();
    failure(
        &unchanged_command(&fixture.scratch, &fixture.arguments()),
        1,
        "busy",
    );
    assert_complete_state(owner.chain(), &fixture.chain);
    drop(owner);
    fixture.assert_reopened();
    let store = ReplacementStoreV2::open_writer(
        fixture.scratch.root(),
        fixture.chain.manifest().identity(),
        checkpoint(&fixture.chain, 0),
    )
    .unwrap();
    failure(
        &unchanged_command(&fixture.scratch, &fixture.arguments()),
        1,
        "busy",
    );
    drop(store);
    fixture.assert_reopened();
    let confirmed = control_root(&fixture.scratch).join("confirmed.v2");
    let original = fs::read(&confirmed).unwrap();
    fs::write(&confirmed, b"synthetic-private-corrupt-record").unwrap();
    failure(
        &unchanged_command(&fixture.scratch, &fixture.arguments()),
        1,
        "invalid_control",
    );
    fs::write(&confirmed, original).unwrap();
    fixture.assert_reopened();
    let unexpected = control_root(&fixture.scratch).join("unexpected");
    fs::write(&unexpected, b"synthetic-private-unknown").unwrap();
    failure(
        &unchanged_command(&fixture.scratch, &fixture.arguments()),
        1,
        "invalid_control",
    );
    fs::remove_file(unexpected).unwrap();
    fixture.assert_reopened();
}

#[test]
fn pending_controller_intent_requires_separate_explicit_recovery_without_automatic_changes() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-pending", fresh());
    fixture.write(&records(&[transfer(&fixture.chain, 1, 2, 0, 1)]));
    let mut candidate = fixture.chain.clone();
    append(&mut candidate, &[]);
    // This is an explicit valid pending layout, not evidence of a kill boundary.
    let intent = IntentRecordV2::new(
        fixture.chain.manifest().identity(),
        checkpoint(&fixture.chain, 0),
        fixture.chain.clone(),
        candidate,
    )
    .unwrap();
    fs::write(
        control_root(&fixture.scratch).join("intent.v2"),
        intent.encode().unwrap(),
    )
    .unwrap();
    let output = unchanged_command(&fixture.scratch, &fixture.arguments());
    let values = failure(&output, 1, "recovery_required");
    assert_eq!(values["next"], "explicit-recover");
    assert!(matches!(
        NodeControlV2::status(
            control_root(&fixture.scratch),
            fixture.scratch.root(),
            fixture.chain.manifest().identity()
        )
        .unwrap(),
        NodeStatusV2::RecoveryRequired { .. }
    ));
    let mut recovery = fixture.arguments()[..9].to_vec();
    recovery[0] = "recover".into();
    recovery.extend(["--action".into(), "retain-old".into()]);
    let recovered = command(&recovery);
    bounded(&recovered);
    assert!(
        recovered.status.success(),
        "{}",
        String::from_utf8_lossy(&recovered.stderr)
    );
    assert!(recovered.stderr.is_empty());
    fixture.assert_reopened();
    assert_eq!(
        fs::read(&fixture.input).unwrap(),
        records(&[transfer(&fixture.chain, 1, 2, 0, 1)])
    );
}

#[test]
fn broken_stdout_retains_conservative_completion_context_after_local_inspection() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-broken-output", fresh());
    fixture.write(&records(&[transfer(&fixture.chain, 1, 2, 0, 1)]));
    let mut arguments = fixture.arguments();
    set(&mut arguments, "--evaluate-count", "1");
    let before = snapshot(&fixture.scratch.0);
    let output = OperatorProcess::spawn_closed_stdout(&arguments).finish();
    failure(&output, 1, "output_io");
    assert_eq!(snapshot(&fixture.scratch.0), before);
    fixture.assert_reopened();
}

#[test]
fn observed_blocked_stdout_has_no_input_or_authority_owners_before_output_loss_or_kill() {
    let _serial = process_guard();
    let fixture = Fixture::new("pool-command-blocked-output", fresh());
    fixture.write(&records(&[transfer(&fixture.chain, 1, 2, 0, 1)]));
    let mut arguments = fixture.arguments();
    set(&mut arguments, "--evaluate-count", "1");
    for forced_death in [false, true] {
        let before = snapshot(&fixture.scratch.0);
        let mut child = BlockedOutputProcess::spawn(&arguments);
        child.wait_stdout_blocked();
        child.assert_paths_released(&[
            &fixture.input,
            &control_root(&fixture.scratch),
            &fixture.scratch.root(),
        ]);
        // A fresh strict writer can own both roots while the actual child is
        // still observed blocked in its final stdout write.
        fixture.assert_reopened();
        if forced_death {
            // SIGKILL here is after computation/resource release. It establishes
            // no claim about termination during input, admission or evaluation.
            let output = child.kill();
            assert_eq!(output.status.signal(), Some(libc::SIGKILL));
            assert!(output.stdout.is_empty() && output.stderr.is_empty());
        } else {
            child.close_reader();
            failure(&child.finish(), 1, "output_io");
        }
        assert_eq!(snapshot(&fixture.scratch.0), before);
        fixture.assert_reopened();
    }
}
