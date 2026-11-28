//! Already accepted provider connections retain F2's protocol and lifetime budgets.

mod support_history_v2;

use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use mirror_crypto::Hash256;
use mirror_network::v2::session::{
    MAX_SESSION_WIRE_BYTES_V2, PeerSessionsV2, REQUEST_LEN_V2, SessionErrorV2 as Error,
    SessionLimitsV2, SessionStatsV2, encode_history_request_v2,
};
use mirror_protocol::v2::HistorySummaryV2;
use support_history_v2::{independent_identity, main_chain};

const HARNESS_TIMEOUT: Duration = Duration::from_secs(3);

fn owner(attempts: u8, bytes: u64, timeout: Duration) -> PeerSessionsV2 {
    PeerSessionsV2::new(SessionLimitsV2::new(attempts, bytes, timeout).unwrap())
}

fn standard_owner() -> PeerSessionsV2 {
    owner(1, MAX_SESSION_WIRE_BYTES_V2, HARNESS_TIMEOUT)
}

fn listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").unwrap()
}

fn accept_bounded(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let end = Instant::now() + HARNESS_TIMEOUT;
    loop {
        assert!(Instant::now() < end, "test connection was not accepted");
        match listener.accept() {
            Ok((stream, _)) => return stream,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("test accept failed: {error}"),
        }
    }
}

fn client(listener: &TcpListener) -> TcpStream {
    let stream =
        TcpStream::connect_timeout(&listener.local_addr().unwrap(), HARNESS_TIMEOUT).unwrap();
    stream.set_nodelay(true).unwrap();
    stream.set_read_timeout(Some(HARNESS_TIMEOUT)).unwrap();
    stream.set_write_timeout(Some(HARNESS_TIMEOUT)).unwrap();
    stream
}

fn connected_pair() -> (TcpStream, TcpStream) {
    let listener = listener();
    let client = client(&listener);
    let server = accept_bounded(&listener);
    (server, client)
}

fn read_response(stream: &mut TcpStream) -> Vec<u8> {
    // The frozen test history is 5,406 bytes. Keep even an erroneous response
    // bounded, and require close rather than accepting a read timeout as EOF.
    let mut response = Vec::new();
    let result = stream.take(16_384).read_to_end(&mut response);
    assert!(response.len() < 16_384, "unexpectedly large test response");
    match result {
        Ok(_) => {}
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
            ) => {}
        Err(error) => panic!("provider did not close its connection: {error}"),
    }
    response
}

enum Entry {
    Connection,
    Listener,
}

enum Endpoint {
    Connection(TcpStream),
    Listener(TcpListener),
}

struct Exchange {
    owner: PeerSessionsV2,
    result: Result<HistorySummaryV2, Error>,
    response: Vec<u8>,
    elapsed: Duration,
}

fn exchange(mut owner: PeerSessionsV2, request: &[u8], half_close: bool, entry: Entry) -> Exchange {
    let chain = main_chain(4);
    let listener = listener();
    let mut client = client(&listener);
    let endpoint = match entry {
        Entry::Connection => {
            let stream = accept_bounded(&listener);
            drop(listener);
            Endpoint::Connection(stream)
        }
        Entry::Listener => Endpoint::Listener(listener),
    };
    let worker = thread::spawn(move || {
        let start = Instant::now();
        let result = match endpoint {
            Endpoint::Connection(stream) => {
                owner.serve_connection(stream, independent_identity(), &chain)
            }
            Endpoint::Listener(listener) => {
                owner.serve_once(listener, independent_identity(), &chain)
            }
        };
        (owner, result, start.elapsed())
    });
    client.write_all(request).unwrap();
    if half_close {
        client.shutdown(Shutdown::Write).unwrap();
    }
    let response = read_response(&mut client);
    let (owner, result, elapsed) = worker.join().unwrap();
    Exchange {
        owner,
        result,
        response,
        elapsed,
    }
}

fn assert_closed_with_retained_clone(client: &mut TcpStream, retained: TcpStream) {
    // Dropping just the supplied descriptor would leave this connection alive.
    // The consuming adapter must shut it down even on a preflight refusal.
    client
        .set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    assert_eq!(client.read(&mut [0u8; 1]).unwrap(), 0);
    drop(retained);
}

#[test]
fn accepted_connection_serves_complete_history_with_exact_accounting() {
    let expected = main_chain(4);
    let source = expected.clone();
    let listener = listener();
    let address = listener.local_addr().unwrap();
    let worker = thread::spawn(move || {
        let stream = accept_bounded(&listener);
        drop(listener);
        let mut provider = standard_owner();
        let summary = provider
            .serve_connection(stream, independent_identity(), &source)
            .unwrap();
        (summary, provider.stats())
    });
    let mut requester = standard_owner();
    let received = requester.request(address, independent_identity()).unwrap();
    assert_eq!(received.chain(), &expected);
    assert_eq!(received.wire_bytes(), 5406);
    let (summary, stats) = worker.join().unwrap();
    assert_eq!(summary, received.summary());
    assert_eq!(stats, requester.stats());
    assert_eq!(
        stats,
        SessionStatsV2 {
            attempts: 1,
            wire_bytes: 5478,
        }
    );
}

#[test]
fn local_identity_refusal_consumes_and_shuts_down_the_supplied_connection() {
    let source = main_chain(0);
    let mut wrong = independent_identity();
    wrong.config_hash = Hash256::from_bytes([0x91; 32]);
    let (stream, mut client) = connected_pair();
    let retained = stream.try_clone().unwrap();
    let mut provider = standard_owner();
    assert!(matches!(
        provider.serve_connection(stream, wrong, &source),
        Err(Error::WrongNetwork)
    ));
    assert_eq!(provider.stats().attempts, 1);
    assert_eq!(provider.stats().wire_bytes, 0);
    assert_closed_with_retained_clone(&mut client, retained);
}

#[test]
fn exhausted_attempts_close_supplied_connections_without_increasing_the_cap() {
    let source = main_chain(0);
    let mut wrong = independent_identity();
    wrong.genesis_hash = Hash256::from_bytes([0x92; 32]);
    let mut provider = standard_owner();
    let (first, _client) = connected_pair();
    assert!(matches!(
        provider.serve_connection(first, wrong, &source),
        Err(Error::WrongNetwork)
    ));
    let (stream, mut client) = connected_pair();
    let retained = stream.try_clone().unwrap();
    assert!(matches!(
        provider.serve_connection(stream, independent_identity(), &source),
        Err(Error::AttemptsExhausted)
    ));
    assert_eq!(provider.stats().attempts, 1);
    assert_eq!(provider.stats().wire_bytes, 0);
    assert_closed_with_retained_clone(&mut client, retained);
}

#[test]
fn exhausted_wire_allowance_closes_supplied_connections_and_retains_attempt_charging() {
    let first = exchange(owner(3, 1, HARNESS_TIMEOUT), b"M", true, Entry::Connection);
    assert!(matches!(first.result, Err(Error::WireBudgetExhausted)));
    assert!(first.response.is_empty());
    assert_eq!(first.owner.stats().wire_bytes, 1);
    let mut provider = first.owner;
    let (stream, mut client) = connected_pair();
    let retained = stream.try_clone().unwrap();
    assert!(matches!(
        provider.serve_connection(stream, independent_identity(), &main_chain(0)),
        Err(Error::WireBudgetExhausted)
    ));
    assert_eq!(provider.stats().attempts, 2);
    assert_eq!(provider.stats().wire_bytes, 1);
    assert_closed_with_retained_clone(&mut client, retained);
}

#[test]
fn invalid_supplied_requests_receive_no_history_and_keep_actual_byte_charges() {
    #[derive(Clone, Copy)]
    enum Expected {
        Malformed,
        Network,
        Trailing,
        Short,
    }
    let valid = encode_history_request_v2(independent_identity());
    let mut malformed = valid.to_vec();
    malformed[0] ^= 1;
    let mut wrong = valid.to_vec();
    wrong[8] ^= 1;
    let mut trailing = valid.to_vec();
    trailing.push(0x81);
    for (request, expected, bytes) in [
        (malformed, Expected::Malformed, 72),
        (wrong, Expected::Network, 72),
        (trailing, Expected::Trailing, 73),
        (valid[..17].to_vec(), Expected::Short, 17),
    ] {
        let exchange = exchange(standard_owner(), &request, true, Entry::Connection);
        let error = exchange.result.unwrap_err();
        assert!(
            matches!(
                (expected, &error),
                (Expected::Malformed, Error::InvalidRequest)
                    | (Expected::Network, Error::WrongNetwork)
                    | (Expected::Trailing, Error::TrailingData)
                    | (Expected::Short, Error::EarlyClose)
            ),
            "unexpected rejection: {error:?}"
        );
        assert!(exchange.response.is_empty());
        assert_eq!(exchange.owner.stats().attempts, 1);
        assert_eq!(exchange.owner.stats().wire_bytes, bytes);
    }
}

#[test]
fn supplied_connection_requires_request_eof_within_its_original_deadline() {
    let exchange = exchange(
        owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(150)),
        &encode_history_request_v2(independent_identity()),
        false,
        Entry::Connection,
    );
    assert!(matches!(exchange.result, Err(Error::DeadlineExceeded)));
    assert!(exchange.response.is_empty());
    assert_eq!(exchange.owner.stats().attempts, 1);
    assert_eq!(exchange.owner.stats().wire_bytes, REQUEST_LEN_V2 as u64);
    assert!(exchange.elapsed < Duration::from_secs(2));
}

#[test]
fn drip_fed_supplied_request_cannot_refresh_the_original_deadline() {
    let source = main_chain(0);
    let (stream, mut client) = connected_pair();
    let (started, ready) = mpsc::channel();
    let worker = thread::spawn(move || {
        let mut provider = owner(1, MAX_SESSION_WIRE_BYTES_V2, Duration::from_millis(200));
        let start = Instant::now();
        started.send(()).unwrap();
        let result = provider.serve_connection(stream, independent_identity(), &source);
        (result, provider.stats(), start.elapsed())
    });
    ready.recv_timeout(HARNESS_TIMEOUT).unwrap();
    for byte in encode_history_request_v2(independent_identity()) {
        if client.write_all(&[byte]).is_err() {
            break;
        }
        thread::sleep(Duration::from_millis(40));
    }
    let response = read_response(&mut client);
    let (result, stats, elapsed) = worker.join().unwrap();
    assert!(matches!(result, Err(Error::DeadlineExceeded)));
    assert!(response.is_empty());
    assert_eq!(stats.attempts, 1);
    assert!(stats.wire_bytes > 0);
    assert!(stats.wire_bytes < REQUEST_LEN_V2 as u64);
    assert!(elapsed < Duration::from_secs(2));
}

#[test]
fn successive_supplied_connections_share_failed_input_and_partial_output_allowance() {
    let mut wrong = encode_history_request_v2(independent_identity());
    wrong[8] ^= 1;
    let first = exchange(
        owner(3, 72 + 72 + 100, HARNESS_TIMEOUT),
        &wrong,
        true,
        Entry::Connection,
    );
    assert!(matches!(first.result, Err(Error::WrongNetwork)));
    assert!(first.response.is_empty());
    assert_eq!(first.owner.stats().wire_bytes, 72);
    let second = exchange(
        first.owner,
        &encode_history_request_v2(independent_identity()),
        true,
        Entry::Connection,
    );
    assert!(matches!(second.result, Err(Error::WireBudgetExhausted)));
    assert_eq!(second.response.len(), 100);
    assert_eq!(second.owner.stats().attempts, 2);
    assert_eq!(second.owner.stats().wire_bytes, 244);
}

#[test]
fn listener_and_supplied_connection_entries_share_one_attempt_and_wire_budget() {
    let mut malformed = encode_history_request_v2(independent_identity());
    malformed[6] ^= 1;
    let first = exchange(
        owner(2, MAX_SESSION_WIRE_BYTES_V2, HARNESS_TIMEOUT),
        &malformed,
        true,
        Entry::Listener,
    );
    assert!(matches!(first.result, Err(Error::InvalidRequest)));
    assert_eq!(first.owner.stats().attempts, 1);
    assert_eq!(first.owner.stats().wire_bytes, 72);
    let second = exchange(
        first.owner,
        &encode_history_request_v2(independent_identity()),
        true,
        Entry::Connection,
    );
    assert_eq!(second.result.unwrap().height, 4);
    assert_eq!(second.response.len(), 5406);
    assert_eq!(second.owner.stats().attempts, 2);
    assert_eq!(second.owner.stats().wire_bytes, 72 + 5478);
    let mut provider = second.owner;
    let (stream, mut client) = connected_pair();
    let retained = stream.try_clone().unwrap();
    assert!(matches!(
        provider.serve_connection(stream, independent_identity(), &main_chain(0)),
        Err(Error::AttemptsExhausted)
    ));
    assert_eq!(provider.stats().attempts, 2);
    assert_eq!(provider.stats().wire_bytes, 72 + 5478);
    assert_closed_with_retained_clone(&mut client, retained);
}
