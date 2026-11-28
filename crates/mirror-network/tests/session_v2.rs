//! Actual TCP behavior for finite V2 sessions. All histories are public fixtures.

mod support_history_v2;

use std::{
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use mirror_chain::v2::ChainErrorV2;
use mirror_crypto::Hash256;
use mirror_network::v2::{
    HistoryErrorV2, HistoryFramesV2,
    session::{
        MAX_SESSION_ATTEMPTS_V2, MAX_SESSION_TIMEOUT_V2, MAX_SESSION_WIRE_BYTES_V2, PeerSessionsV2,
        REQUEST_LEN_V2, SessionErrorV2 as Error, SessionLimitsV2, encode_history_request_v2,
    },
};
use mirror_protocol::v2::{FrameRefV2, encode_frame_v2};
use support_history_v2::{fixture_bytes, independent_identity, main_chain};

fn owner(attempts: u8, bytes: u64, timeout: Duration) -> PeerSessionsV2 {
    PeerSessionsV2::new(SessionLimitsV2::new(attempts, bytes, timeout).unwrap())
}

fn standard_owner() -> PeerSessionsV2 {
    owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_secs(3))
}

fn listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").unwrap()
}

fn scripted_peer(handler: impl FnOnce(TcpStream) + Send + 'static) -> (SocketAddr, JoinHandle<()>) {
    let listener = listener();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream.set_nodelay(true).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        handler(stream);
    });
    (address, handle)
}

fn read_request(stream: &mut TcpStream) {
    let mut request = [0; REQUEST_LEN_V2];
    stream.read_exact(&mut request).unwrap();
    assert_eq!(request, encode_history_request_v2(independent_identity()));
    let mut sentinel = [0];
    assert_eq!(stream.read(&mut sentinel).unwrap(), 0);
}

fn history_frames(height: u64) -> Vec<Vec<u8>> {
    HistoryFramesV2::new(&main_chain(height))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn transcript(height: u64) -> Vec<u8> {
    history_frames(height).concat()
}

fn peer_bytes(bytes: Vec<u8>) -> (SocketAddr, JoinHandle<()>) {
    scripted_peer(move |mut stream| {
        read_request(&mut stream);
        // A deliberately invalid response may cause the client to close early.
        let _ = stream.write_all(&bytes);
        let _ = stream.shutdown(Shutdown::Write);
    })
}

fn withhold_after(bytes: Vec<u8>) -> (SocketAddr, JoinHandle<()>, mpsc::Sender<()>) {
    let (done, wait) = mpsc::channel();
    let (address, handle) = scripted_peer(move |mut stream| {
        read_request(&mut stream);
        stream.write_all(&bytes).unwrap();
        let _ = wait.recv_timeout(Duration::from_secs(3));
    });
    (address, handle, done)
}

#[test]
fn complete_tcp_session_reconstructs_and_accounts_both_owners() {
    let expected = main_chain(4);
    let source = expected.clone();
    let listener = listener();
    let address = listener.local_addr().unwrap();
    let provider = thread::spawn(move || {
        let mut sessions = standard_owner();
        let summary = sessions
            .serve_once(listener, independent_identity(), &source)
            .unwrap();
        (summary, sessions.stats())
    });
    let mut requester = standard_owner();
    let received = requester.request(address, independent_identity()).unwrap();
    assert_eq!(received.chain(), &expected);
    assert_eq!(received.wire_bytes(), 5406);
    assert_eq!(received.frame_count(), 8);
    let (summary, provider_stats) = provider.join().unwrap();
    assert_eq!(summary, received.summary());
    assert_eq!(provider_stats, requester.stats());
    assert_eq!(requester.stats().wire_bytes, 72 + 5406);
    assert_eq!(requester.stats().attempts, 1);
}

#[test]
fn fragmented_headers_and_payloads_preserve_the_complete_candidate() {
    let bytes = transcript(4);
    let (address, provider) = scripted_peer(move |mut stream| {
        read_request(&mut stream);
        for part in bytes.chunks(3) {
            stream.write_all(part).unwrap();
        }
        stream.shutdown(Shutdown::Write).unwrap();
    });
    let received = standard_owner()
        .request(address, independent_identity())
        .unwrap();
    assert_eq!(received.into_chain(), main_chain(4));
    provider.join().unwrap();
}

#[test]
fn every_early_close_phase_fails_without_a_candidate() {
    let bytes = transcript(1);
    for end in [0, 5, 12, 165, bytes.len() - 164, bytes.len() - 1] {
        let (address, provider) = peer_bytes(bytes[..end].to_vec());
        let mut requester = standard_owner();
        assert!(matches!(
            requester.request(address, independent_identity()),
            Err(Error::EarlyClose)
        ));
        assert_eq!(requester.stats().wire_bytes, 72 + end as u64);
        provider.join().unwrap();
    }
}

#[test]
fn oversized_header_rejects_without_waiting_for_or_allocating_payload() {
    let mut header = transcript(0)[..12].to_vec();
    header[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    let (address, provider, done) = withhold_after(header);
    let mut requester = standard_owner();
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::History(HistoryErrorV2::Protocol(_)))
    ));
    assert_eq!(requester.stats().wire_bytes, 84);
    done.send(()).unwrap();
    provider.join().unwrap();
}

#[test]
fn phase_invalid_header_rejects_before_reading_an_otherwise_bounded_payload() {
    let header = history_frames(0)[1][..12].to_vec();
    let (address, provider, done) = withhold_after(header);
    let mut requester = standard_owner();
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::History(HistoryErrorV2::UnexpectedFrame { .. }))
    ));
    assert_eq!(requester.stats().wire_bytes, 84);
    done.send(()).unwrap();
    provider.join().unwrap();
}

#[test]
fn one_trailing_byte_or_a_second_transcript_rejects_after_end() {
    let bytes = transcript(1);
    for suffix in [vec![0x81], bytes.clone()] {
        let mut wire = bytes.clone();
        wire.extend(suffix);
        let (address, provider) = peer_bytes(wire);
        let mut requester = standard_owner();
        assert!(matches!(
            requester.request(address, independent_identity()),
            Err(Error::TrailingData)
        ));
        assert_eq!(requester.stats().wire_bytes, 72 + bytes.len() as u64 + 1);
        provider.join().unwrap();
    }
}

#[test]
fn accepted_end_without_eof_expires_instead_of_returning_a_candidate() {
    let bytes = transcript(1);
    let wire_len = bytes.len() as u64;
    let (address, provider, done) = withhold_after(bytes);
    let mut requester = owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(150));
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::DeadlineExceeded)
    ));
    assert_eq!(requester.stats().wire_bytes, 72 + wire_len);
    done.send(()).unwrap();
    provider.join().unwrap();
}

#[test]
fn partial_payload_stall_obeys_the_original_deadline() {
    let bytes = transcript(0)[..13].to_vec();
    let (address, provider, done) = withhold_after(bytes);
    let mut requester = owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(150));
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::DeadlineExceeded)
    ));
    assert_eq!(requester.stats().wire_bytes, 85);
    done.send(()).unwrap();
    provider.join().unwrap();
}

#[test]
fn drip_fed_header_cannot_refresh_the_absolute_deadline() {
    let bytes = transcript(0);
    let (address, provider) = scripted_peer(move |mut stream| {
        read_request(&mut stream);
        for byte in bytes.into_iter().take(12) {
            if stream.write_all(&[byte]).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(30));
        }
    });
    let mut requester = owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(140));
    let start = Instant::now();
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::DeadlineExceeded)
    ));
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(requester.stats().wire_bytes < 84);
    provider.join().unwrap();
}

#[test]
fn false_network_claim_rejects_under_independent_requester_pins() {
    let mut bytes = transcript(0);
    bytes[12] ^= 1;
    let (address, provider) = peer_bytes(bytes);
    let mut requester = standard_owner();
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::History(HistoryErrorV2::WrongNetwork))
    ));
    assert_eq!(requester.stats().wire_bytes, 72 + 164);
    provider.join().unwrap();
}

#[test]
fn changed_terminal_claim_rejects_over_real_tcp() {
    let mut input = history_frames(1);
    let last = input.last_mut().unwrap();
    last[12 + 72] ^= 1;
    let (address, provider) = peer_bytes(input.concat());
    assert!(matches!(
        standard_owner().request(address, independent_identity()),
        Err(Error::History(HistoryErrorV2::ChangedSummary))
    ));
    provider.join().unwrap();
}

#[test]
fn actual_wrong_ledger_block_rejects_over_real_tcp() {
    let mut input = history_frames(1);
    input[3] = encode_frame_v2(FrameRefV2::Block {
        height: 1,
        bytes: &fixture_bytes("wrong_root1_block"),
    })
    .unwrap();
    let (address, provider) = peer_bytes(input.concat());
    assert!(matches!(
        standard_owner().request(address, independent_identity()),
        Err(Error::History(HistoryErrorV2::Chain(
            ChainErrorV2::WrongLedgerRoot
        )))
    ));
    provider.join().unwrap();
}

#[test]
fn provider_checks_its_independent_network_before_accepting() {
    let mut wrong = independent_identity();
    wrong.genesis_hash = Hash256::from_bytes([0x91; 32]);
    let mut provider = standard_owner();
    assert!(matches!(
        provider.serve_once(listener(), wrong, &main_chain(0)),
        Err(Error::WrongNetwork)
    ));
    assert_eq!(provider.stats().attempts, 1);
    assert_eq!(provider.stats().wire_bytes, 0);
}

fn send_to_provider(
    request: Vec<u8>,
    half_close: bool,
    timeout: Duration,
    byte_budget: u64,
) -> (
    Result<mirror_protocol::v2::HistorySummaryV2, Error>,
    u64,
    Vec<u8>,
) {
    let chain = main_chain(0);
    let listener = listener();
    let address = listener.local_addr().unwrap();
    let provider = thread::spawn(move || {
        let mut provider = owner(1, byte_budget, timeout);
        let result = provider.serve_once(listener, independent_identity(), &chain);
        (result, provider.stats().wire_bytes)
    });
    let mut client = TcpStream::connect(address).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    client.write_all(&request).unwrap();
    if half_close {
        client.shutdown(Shutdown::Write).unwrap();
    }
    let mut response = Vec::new();
    // Rejecting extra queued input may reset the socket, which is an allowed
    // failed-session close. Every observed response byte is still retained.
    let _ = client.read_to_end(&mut response);
    let (result, bytes) = provider.join().unwrap();
    (result, bytes, response)
}

#[test]
fn provider_rejects_wrong_network_request_without_responding() {
    let mut request = encode_history_request_v2(independent_identity()).to_vec();
    request[8] ^= 1;
    let (result, bytes, response) = send_to_provider(
        request,
        true,
        Duration::from_secs(2),
        MAX_SESSION_WIRE_BYTES_V2,
    );
    assert!(matches!(result, Err(Error::WrongNetwork)));
    assert_eq!(bytes, 72);
    assert!(response.is_empty());
}

#[test]
fn provider_rejects_repeated_requests_before_any_history_output() {
    let mut request = encode_history_request_v2(independent_identity()).to_vec();
    request.extend_from_slice(&encode_history_request_v2(independent_identity()));
    let (result, bytes, response) = send_to_provider(
        request,
        true,
        Duration::from_secs(2),
        MAX_SESSION_WIRE_BYTES_V2,
    );
    assert!(matches!(result, Err(Error::TrailingData)));
    assert_eq!(bytes, 73);
    assert!(response.is_empty());
}

#[test]
fn provider_requires_request_eof_within_the_same_deadline() {
    let (result, bytes, response) = send_to_provider(
        encode_history_request_v2(independent_identity()).to_vec(),
        false,
        Duration::from_millis(150),
        MAX_SESSION_WIRE_BYTES_V2,
    );
    assert!(matches!(result, Err(Error::DeadlineExceeded)));
    assert_eq!(bytes, 72);
    assert!(response.is_empty());
}

#[test]
fn provider_malformed_and_short_requests_never_produce_output() {
    let valid = encode_history_request_v2(independent_identity());
    let mut requests = vec![valid[..71].to_vec()];
    for index in [0, 4, 6] {
        let mut request = valid.to_vec();
        request[index] ^= 0x40;
        requests.push(request);
    }
    for request in requests {
        let length = request.len();
        let (result, bytes, response) = send_to_provider(
            request,
            true,
            Duration::from_secs(2),
            MAX_SESSION_WIRE_BYTES_V2,
        );
        if length < 72 {
            assert!(matches!(result, Err(Error::EarlyClose)));
        } else {
            assert!(matches!(result, Err(Error::InvalidRequest)));
        }
        assert_eq!(bytes, length as u64);
        assert!(response.is_empty());
    }
}

#[test]
fn an_unconnected_listener_has_a_finite_accept_deadline() {
    let mut provider = owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(80));
    let source = main_chain(0);
    let start = Instant::now();
    assert!(matches!(
        provider.serve_once(listener(), independent_identity(), &source),
        Err(Error::DeadlineExceeded)
    ));
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(provider.stats().attempts, 1);
    assert_eq!(provider.stats().wire_bytes, 0);
}

#[test]
fn failed_attempts_remain_charged_and_the_owner_cannot_reset_them() {
    let mut requester = owner(2, MAX_SESSION_WIRE_BYTES_V2, Duration::from_secs(2));
    for attempt in 1..=2 {
        let (address, provider) = peer_bytes(vec![1, 2, 3]);
        assert!(matches!(
            requester.request(address, independent_identity()),
            Err(Error::EarlyClose)
        ));
        assert_eq!(requester.stats().attempts, attempt);
        assert_eq!(requester.stats().wire_bytes, u64::from(attempt) * 75);
        provider.join().unwrap();
    }
    assert!(matches!(
        requester.request("127.0.0.1:1".parse().unwrap(), independent_identity()),
        Err(Error::AttemptsExhausted)
    ));
    assert_eq!(requester.stats().attempts, 2);
    assert_eq!(requester.stats().wire_bytes, 150);
}

#[test]
fn failed_and_partial_io_spend_one_cumulative_wire_allowance() {
    let mut wrong = transcript(0);
    wrong[12] ^= 1;
    let mut requester = owner(3, 236 + 72 + 5, Duration::from_secs(2));
    let (address, provider) = peer_bytes(wrong);
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::History(HistoryErrorV2::WrongNetwork))
    ));
    provider.join().unwrap();
    assert_eq!(requester.stats().wire_bytes, 236);
    let (address, provider) = peer_bytes(transcript(0));
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::WireBudgetExhausted)
    ));
    provider.join().unwrap();
    assert_eq!(requester.stats().wire_bytes, 313);
    assert!(matches!(
        requester.request("127.0.0.1:1".parse().unwrap(), independent_identity()),
        Err(Error::WireBudgetExhausted)
    ));
    assert_eq!(requester.stats().attempts, 3);
    assert_eq!(requester.stats().wire_bytes, 313);
}

#[test]
fn admitted_payload_must_fit_remaining_owner_budget_before_allocation() {
    let (address, provider, done) = withhold_after(transcript(0)[..12].to_vec());
    let mut requester = owner(1, 72 + 12 + 151, Duration::from_secs(2));
    assert!(matches!(
        requester.request(address, independent_identity()),
        Err(Error::WireBudgetExhausted)
    ));
    assert_eq!(requester.stats().wire_bytes, 84);
    done.send(()).unwrap();
    provider.join().unwrap();
}

#[test]
fn terminal_eof_probe_requires_spare_capacity_but_charges_zero_for_eof() {
    let bytes = transcript(0);
    let exact = 72 + bytes.len() as u64;
    for spare in [0, 1] {
        let (address, provider) = peer_bytes(bytes.clone());
        let mut requester = owner(1, exact + spare, Duration::from_secs(2));
        let result = requester.request(address, independent_identity());
        if spare == 0 {
            assert!(matches!(result, Err(Error::WireBudgetExhausted)));
        } else {
            assert_eq!(result.unwrap().into_chain(), main_chain(0));
        }
        assert_eq!(requester.stats().wire_bytes, exact);
        provider.join().unwrap();
    }
}

#[test]
fn provider_partial_output_never_exceeds_its_remaining_allowance() {
    let (result, bytes, response) = send_to_provider(
        encode_history_request_v2(independent_identity()).to_vec(),
        true,
        Duration::from_secs(2),
        72 + 100,
    );
    assert!(matches!(result, Err(Error::WireBudgetExhausted)));
    assert_eq!(bytes, 172);
    assert_eq!(response.len(), 100);
}

#[test]
fn configuration_enforces_finite_attempt_byte_and_time_ceilings() {
    let valid_time = Duration::from_secs(1);
    for (attempts, bytes, timeout) in [
        (0, 1, valid_time),
        (MAX_SESSION_ATTEMPTS_V2 + 1, 1, valid_time),
        (1, 0, valid_time),
        (1, MAX_SESSION_WIRE_BYTES_V2 + 1, valid_time),
        (1, 1, Duration::ZERO),
        (1, 1, MAX_SESSION_TIMEOUT_V2 + Duration::from_nanos(1)),
    ] {
        assert!(matches!(
            SessionLimitsV2::new(attempts, bytes, timeout),
            Err(Error::InvalidLimits(_))
        ));
    }
    let maximum = SessionLimitsV2::new(
        MAX_SESSION_ATTEMPTS_V2,
        MAX_SESSION_WIRE_BYTES_V2,
        MAX_SESSION_TIMEOUT_V2,
    )
    .unwrap();
    assert_eq!(maximum.max_attempts(), 8);
    assert_eq!(maximum.timeout(), Duration::from_secs(60));
    assert_eq!(maximum.max_wire_bytes(), MAX_SESSION_WIRE_BYTES_V2);
}
