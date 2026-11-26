# Bounded V2 TCP peer sessions

Contract: **MIRROR-MB03F2-001**, finite synthetic development profile, 2026-11-26.
Implementation: [`mirror_network::v2::session`](../crates/mirror-network/src/v2/session.rs).

This component transports the unchanged [F1 candidate history](PEER_HISTORY_V2.md)
over a one-request TCP connection. It adds independently pinned request selection,
bounded sequential attempts, cumulative actual I/O accounting, one absolute
deadline per attempt, and a mandatory terminal EOF. It returns a completely
accepted candidate. [Canonical adoption](CANDIDATE_ADOPTION_V2.md) remains a separate
explicit operation under existing [replacement storage](STORAGE_REPLACEMENT_STORE_V2.md)
ownership and [recovery](STORAGE_REPLACEMENT_RECOVERY_V2.md) rules.

## 1. Independent authority and public API

The caller chooses `ManifestIdentityV2 { config_hash, genesis_hash }` from its own
local configuration before making or accepting a request. The request and response
cannot supply this authority. `serve_once` checks that its immutable accepted
history belongs to that independently selected identity before accepting a client.
The additive `serve_connection` entry checks the same identity before reading an
already accepted connection; its caller owns admission as described in section 9.

The main APIs are:

```rust
SessionLimitsV2::new(max_attempts: u8, max_wire_bytes: u64, timeout: Duration)
    -> Result<SessionLimitsV2, SessionErrorV2>
PeerSessionsV2::new(limits: SessionLimitsV2) -> PeerSessionsV2
sessions.request(address: SocketAddr, identity: ManifestIdentityV2)
    -> Result<ReceivedCandidateV2, SessionErrorV2>
sessions.serve_once(listener: TcpListener, identity: ManifestIdentityV2, chain: &ChainV2)
    -> Result<HistorySummaryV2, SessionErrorV2>
sessions.serve_connection(stream: TcpStream, identity: ManifestIdentityV2, chain: &ChainV2)
    -> Result<HistorySummaryV2, SessionErrorV2>
sessions.stats() -> SessionStatsV2 { attempts: u8, wire_bytes: u64 }
```

`PeerSessionsV2` is not Clone. All operations require an exclusive mutable borrow
for the entire synchronous attempt. One owner therefore has at most one connected
or accepted stream at a time. A copied statistics value does not copy or replenish
the actual owner's budget. `limits()` exposes its immutable validated limits.

The provider borrows one accepted `ChainV2` immutably for the transfer. Its summary
is derived by the existing F1 sender from that exact history. A successful provider
call means that all response bytes were accepted by its local TCP write calls and
its write half was shut down; it is not an acknowledgment of remote validation,
storage adoption, finality, or physical delivery.

All addresses are numeric `SocketAddr` values. There is one connect operation per
request, without DNS, implicit reconnect, retry, negotiation, fallback, resume,
transaction exchange, or multiplexing. A caller may explicitly start another
attempt only while the same owner's cumulative budgets permit it.

## 2. Exact request encoding — MPQ2

The request is exactly **72 bytes**. Unsigned integers are little-endian and hashes
retain their exact stored byte order. There is no payload-length field, padding,
checksum, signed checkpoint, advertised height, local generation, or optional data.

| Byte interval | Length | Field | Required value |
|---|---:|---|---|
| `[0, 4)` | 4 | Magic | ASCII `MPQ2` |
| `[4, 6)` | 2 | Version | `u16`, exactly 2 |
| `[6, 8)` | 2 | Request kind | `u16`, exactly 1: full candidate history |
| `[8, 40)` | 32 | Configuration hash | Independent selected C |
| `[40, 72)` | 32 | Genesis hash | Independent selected exact genesis |

`encode_history_request_v2(identity)` returns a fixed `[u8; 72]`.
`validate_history_request_v2(bytes, identity)` checks exact length, magic, version,
kind and both independently supplied network pins. It exposes no unpinned identity
decoder. Wrong-network requests and malformed requests fail without a response.

MPQ2 is a separate request preamble. The **MPN2 F1 Begin/Manifest/Block/End frames
are unchanged**, as are the frozen V1 `MIRR` protocol, legacy transport APIs, and
legacy node commands. No new cryptographic hash domain is introduced. A correct
request identifies a network; it does not authenticate a person or a peer.

## 3. One-shot session boundary

The requester performs this exact sequence:

1. Charge one attempt and establish the absolute deadline before connecting.
2. Connect once and send the exact 72-byte request.
3. Shut down its write half. It will send no further data in this session.
4. Receive and validate one complete F1 transcript under independent C/genesis.
5. After accepting the matching End, require response EOF within the same deadline.
6. Only then consume the receiver with `finish()` and return the accepted candidate.

`serve_once` consumes one `TcpListener`, waits under the attempt deadline, accepts
at most one connection, and drops that listener before processing the connection.
It reads the fixed request, validates it, and requires request EOF before producing
any response. It streams the borrowed accepted history through `HistoryFramesV2`,
one frame at a time. After writing End, it shuts down its write half and closes.

Each EOF check reads a one-byte sentinel. A received byte is charged and rejects
as trailing data. Thus an extra request, a second response, any extra frame, or a
single arbitrary trailing byte is rejected. An EOF result transfers zero bytes.

**A valid End and EOF are both required for requester success.** EOF before End,
including after all advertised blocks, is incomplete and fails. End without EOF
eventually fails at the original absolute deadline. F1 End is still a framing
boundary; F2 explicitly adds this stricter one-shot socket boundary.

Every failure closes the owned socket and discards the consuming receiver or
pending-frame owner. No partial candidate or reusable connection is returned. The
implementation does not skip a bad frame or scan the input for another magic.

## 4. Header-first allocation and complete validation

The requester reads a fixed 12-byte F1 header onto the stack, recognizes its kind,
and calls consuming `HistoryReceiverV2::begin_frame`. Only an admitted pending owner
exposes its exact payload size. Before allocating that payload, the session also
requires the entire payload to fit its remaining cumulative wire allowance.

The payload buffer is reserved for that admitted length, read under the original
deadline, and passed to consuming `accept_payload`. I/O failure drops the pending
owner. Semantic failure consumes the receiver. Manifest proofs, configured limits,
actual genesis/parent/work validation, original mixed operations, complete ledger
execution, resource accounting and matching endpoint claims all remain F1 checks.

There is no all-history transport buffer in production. The accepted chain and
ledger still retain their bounded histories and execution structures. A provider
encodes at most one emitted frame at a time, using the existing F1 sender; its
manifest and temporary block encodings retain that sender's documented allocations.

## 5. Attempt and byte budgets

`SessionLimitsV2::new` accepts only:

| Limit | Accepted range |
|---|---:|
| Total attempts per owner | 1 through 8 |
| Cumulative actual read/write bytes | 1 through 157,129,488 |
| Absolute duration of each attempt | Greater than zero through 60 seconds |
| Simultaneously connected/accepted streams per owner | At most 1 |
| Requests and complete responses per attempt | At most 1 each |

The absolute cumulative byte ceiling is:

```text
8 * (MAX_HISTORY_WIRE_BYTES_V2 + REQUEST_LEN_V2 + 2)
= 8 * (19,641,112 + 72 + 2)
= 157,129,488
```

The two extra bytes allow conservative trailing-data rejection sentinels. Successful
EOF probes actually read zero bytes. The frozen F1 per-transfer frame, manifest,
block, wire and chain-state caps remain unchanged; this cumulative allowance does
not permit a single response to exceed them.

`wire_bytes` is the sum of positive byte counts actually returned by this owner's
read and write syscalls, in both directions. It includes request bytes, malformed
input, partially written output and trailing sentinel bytes on failed attempts.
The implementation caps each syscall's buffer to the remaining allowance, so it
does not read or write past that allowance and then reject retrospectively.
Successful data read immediately before a deadline rejection remains charged.

The EOF sentinel needs **one unspent byte of capacity** before probing. If the
allowance is exactly exhausted at a valid End, the requester fails without an
additional uncharged read, even if EOF would be available. With one spare byte,
zero-byte EOF succeeds and leaves that byte uncharged. This conservative rule is
covered by a boundary test.

One attempt is charged before network work, provider identity validation, or a
refusal due to an already exhausted wire allowance. Attempts beyond the configured
attempt cap are refused without opening a socket and do not increase the counter
beyond its cap. If a caller supplies an already accepted stream, that stream is
still shut down and dropped on refusal. No error restores consumed attempts or bytes.

For the frozen main history, the response is 5,406 bytes and each successful
endpoint accounts for **5,478 bytes**, including the 72-byte request. Accounting
does not include TCP/IP headers, retransmission traffic, kernel buffers or bytes
that a peer sent but this endpoint never read.

## 6. One absolute deadline, including fragmented I/O

Each attempt derives one `Instant` deadline. Connection establishment or listener
waiting, request transfer, request EOF, response framing, semantic processing,
terminal EOF and successful return all use this same deadline. A later explicit
attempt gets a new deadline but consumes another item from the owner's finite
attempt allowance. Caller-side waiting before `serve_connection` is outside that
entry's deadline; the provider must bound its own listener admission separately.

The listener uses nonblocking accept and bounded polling. The connected stream is
blocking, but the remaining deadline duration is recalculated and installed as the
socket timeout before **every individual read or write syscall**. The code uses
explicit partial-I/O loops rather than `read_exact`/`write_all` with a single fixed
timeout. Short reads, short writes, interrupted syscalls and early timeout wakeups
therefore cannot refresh the original deadline. Connect timeout is similarly
bounded by its remaining time.

The deadline is checked after successful I/O, after bounded allocation/encoding,
after semantic validation, and before returning success. A slow stream sending
occasional bytes cannot retain an incomplete session indefinitely by staying just
under a per-read idle timeout. A nonreading peer is also bounded during writes.

These are deadline-aware synchronous operations, **not hard CPU preemption or a
real-time scheduling guarantee**. A bounded block replay, allocator operation,
OS syscall or scheduler delay may overrun the instant; the following check rejects
a late result. The component does not promise a strict total process wall-clock,
RSS, physical bandwidth, or service-availability bound.

## 7. Adoption, recovery and remaining scope

Reception changes no filesystem or selected store. A complete accepted candidate
may still be shorter than current or lose an equal-height comparison. The separate
adoption API must independently validate local selection, compare the complete
accepted branches, and use owned storage publication only for a preferred branch.
Fresh initialization is explicit and separate from opening an existing selection.

Both exact old/candidate histories and their independently retained local pins
remain necessary for explicit interrupted replacement recovery. Transport completion
and peer claims supply no storage permit or recovery authorization. A provider's
success also says nothing about a receiver's eventual publication.

The limits belong to **one owner**. Other owners, processes, listener clones and
OS backlog entries are outside that owner's concurrency limit. This is not global
connection admission, peer discovery, an authenticated/encrypted transport, a
persistent daemon, a transaction pool or a wallet/operator service. Those require
separate design and acceptance work. No money, human-consensus, biometric capture,
privacy or device rule changes here; Gate A/B status belongs in the checkpoint files.

## 8. Verification sources

[`session_v2.rs`](../crates/mirror-network/tests/session_v2.rs) exercises real TCP
transfer, fragmented headers/payloads, premature closes, missing and trailing EOF,
wrong networks/claims/ledger execution, payload admission before allocation,
finite listener waits, request rejection before output, cumulative failures,
partial-output byte caps, and the spare sentinel capacity rule.

The session module's private actual-socket test fills the send buffer against a
nonreading peer and verifies the original write deadline with retained partial-byte
accounting. The drip-input test sends bytes within individual idle intervals while
exceeding the absolute deadline. These tests establish their observed software
behavior; they do not assert kernel-level CPU preemption or physical crash safety.

[`reference_peer_session_v2.py`](../tools/reference_peer_session_v2.py) independently
packs the MPQ2 bytes and complete request/response transcript accounting; its frozen
[vectors](../crates/mirror-network/tests/vectors/peer_session_v2.txt) are checked by
[`session_vectors_v2.rs`](../crates/mirror-network/tests/session_vectors_v2.rs).
Separate peer-process integration and adoption tests exercise the complete TCP,
comparison, durable publication and strict reopen path. Full gate counts, exact
source, CI and remaining work are recorded in [CURRENT_STATUS.md](../CURRENT_STATUS.md)
and [MASTER_PLAN.md](../MASTER_PLAN.md).

## 9. Accepted-connection adapter for MB-04B1

`PeerSessionsV2::serve_connection` consumes one `TcpStream` that the caller has
already accepted. It does not bind or accept a connection, clone a listener,
create another session owner or reset any allowance. The caller must independently
bound admission, retained listeners, lifecycle duration and concurrent work. The
[MB-04B1 provider contract](NODE_PROVIDER_V2.md) defines that separate boundary.

The stream has a shutdown/drop guard before preflight. The existing attempt rules
then charge this owner once, check remaining cumulative bytes and establish one
original absolute deadline. An already exhausted attempt cap cannot increase its
counter; an exhausted wire allowance still charges an otherwise permitted attempt.
The independently selected provider identity is checked before request I/O. Every
error, including either exhausted budget or wrong local identity, shuts down and
drops the supplied connection. No reusable stream is returned, and a descriptor
clone retained outside this API cannot keep that connection open after shutdown.

After socket configuration, both provider entry points use one shared checked
routine: read the exact MPQ2 request, validate its independent network, require
request EOF before any history output, emit unchanged `HistoryFramesV2` frames and
shut down the response write half. All positive request/response/sentinel bytes
remain charged against the same owner, including rejected requests and partial
output. Calls to `request`, `serve_once` and `serve_connection` cannot replenish
each other's attempt or byte allowance.

The new entry starts its deadline after caller-side acceptance. Existing
`serve_once` retains its earlier deadline, including its listener wait, and passes
that same deadline into the shared routine. This extraction changes no request,
frame, validation, EOF, byte-accounting or timeout rule. It adds no cancellation
or hard CPU-preemption promise; a caller can stop new admission and let an already
admitted connection finish or reject under its original F2 deadline.

[`provider_connection_v2.rs`](../crates/mirror-network/tests/provider_connection_v2.rs)
targets the additive entry with complete history reception, guarded preflight
closure, malformed/wrong-network/incomplete/trailing requests, missing EOF,
absolute-deadline drip input, cumulative failed-input/partial-output accounting
and mixed old/new provider calls on one session owner. All earlier F2 tests,
references, fixtures and examples remain part of the required acceptance gates.
