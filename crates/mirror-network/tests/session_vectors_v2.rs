//! Independent request bytes and complete bidirectional transcript accounting.

mod support_history_v2;

use std::collections::BTreeMap;

use mirror_crypto::sha256d;
use mirror_network::v2::{
    HistoryFramesV2,
    session::{
        MAX_SESSION_ATTEMPTS_V2, MAX_SESSION_TIMEOUT_V2, MAX_SESSION_WIRE_BYTES_V2, REQUEST_LEN_V2,
        encode_history_request_v2, validate_history_request_v2,
    },
};
use support_history_v2::{fork_chain, independent_identity, main_chain, unhex};

fn vectors() -> BTreeMap<&'static str, &'static str> {
    include_str!("vectors/peer_session_v2.txt")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split_once('=').unwrap())
        .collect()
}

#[test]
fn request_and_four_complete_sessions_match_independent_python_vectors() {
    let reference = vectors();
    let request = encode_history_request_v2(independent_identity());
    assert_eq!(request.as_slice(), unhex(reference["request"]));
    assert_eq!(
        REQUEST_LEN_V2,
        reference["request_len"].parse::<usize>().unwrap()
    );
    validate_history_request_v2(&request, independent_identity()).unwrap();
    for (label, chain) in [
        ("genesis", main_chain(0)),
        ("main3", main_chain(3)),
        ("main4", main_chain(4)),
        ("fork3", fork_chain()),
    ] {
        let mut transcript = request.to_vec();
        for frame in HistoryFramesV2::new(&chain).unwrap() {
            transcript.extend_from_slice(&frame.unwrap());
        }
        assert_eq!(
            transcript.len(),
            reference[format!("{label}_session_wire_bytes").as_str()]
                .parse::<usize>()
                .unwrap()
        );
        assert_eq!(
            transcript.len() - request.len(),
            reference[format!("{label}_response_wire_bytes").as_str()]
                .parse::<usize>()
                .unwrap()
        );
        assert_eq!(
            sha256d(&transcript).to_hex(),
            reference[format!("{label}_transcript_sha256d").as_str()]
        );
    }
}

#[test]
fn session_limit_constants_match_independent_checked_arithmetic() {
    let reference = vectors();
    assert_eq!(
        MAX_SESSION_ATTEMPTS_V2,
        reference["max_attempts"].parse::<u8>().unwrap()
    );
    assert_eq!(
        MAX_SESSION_WIRE_BYTES_V2,
        reference["max_cumulative_wire_bytes"]
            .parse::<u64>()
            .unwrap()
    );
    assert_eq!(
        MAX_SESSION_TIMEOUT_V2.as_secs(),
        reference["max_session_seconds"].parse::<u64>().unwrap()
    );
    assert_eq!(
        reference["max_accepted_session_wire_bytes"]
            .parse::<usize>()
            .unwrap(),
        REQUEST_LEN_V2 + mirror_protocol::v2::MAX_HISTORY_WIRE_BYTES_V2
    );
}
