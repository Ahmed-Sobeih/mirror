# Bounded V2 provider lifecycle and shutdown

Implementation contract: **MB-04B1**, finite development profile,
**2026-11-27**. This design was reviewed before implementation. Actual source acceptance,
published commits and CI results belong in [CURRENT_STATUS.md](../CURRENT_STATUS.md)
and the [master plan](../MASTER_PLAN.md); this document alone is not verification.

The `serve` command extends the separate
[`mirror-node-v2` operator executable](NODE_OPERATOR_V2.md). It serves one fixed
complete accepted history through the unchanged MPQ2/MPN2 protocol, under one
retained [A1 controller](NODE_CONTROL_V2.md) and one cumulative
[F2 session owner](PEER_SESSION_V2.md). It adds explicit application admission,
a finite admission window and graceful shutdown. It is not a background daemon,
live synchronization scheduler, transaction pool or wallet.

## 1. Explicit invocation and limits

Every value below is supplied deliberately; the angle-bracket text describes
values and is not literal command input:

```text
mirror-node-v2 serve
  --control-dir <path> --store-dir <path>
  --config-hash <64 hex characters> --genesis-hash <64 hex characters>
  --listen <numeric IP:nonzero-port>
  --max-attempts <1..8> --max-io-bytes <1..157129488>
  --timeout-ms <1..60000> --admission-ms <1..60000>
```

All options use separate flag/value tokens. The existing whole-command limits
remain: at most 32 arguments after the executable name, 4,096 UTF-8 bytes per
token and 16,384 total bytes. Preserve rejection of duplicate, missing, unknown
and cross-command options, positional extras, non-UTF-8/control input, malformed
hashes, noncanonical unsigned decimal and overflow before filesystem or socket
effects. Hash input accepts either ASCII hex case and output is lowercase.

`--listen` is a numeric `SocketAddr`, including bracketed numeric IPv6 where
applicable, with a nonzero port. There is no DNS resolution, ephemeral-port mode,
environment/config-file default, stdin command channel or implicit peer selection.
The operator chooses the intended bind interface. A successful request identifies
a network; it does not authenticate the peer or provide encrypted transport.

| Resource | Bound for one invocation |
|---|---:|
| Independently selected histories served | Exactly one retained accepted history |
| Provider-owned listeners | At most one |
| Admitted connected streams | At most one at a time |
| Cumulative admitted attempts | Explicit 1 through 8 |
| Cumulative actual application read/write bytes | Explicit 1 through 157,129,488 |
| Original timeout of each admitted F2 attempt | Explicit 1 through 60,000 ms |
| Admission window | Explicit 1 through 60,000 ms |
| Idle polling sleep | At most 1 ms per wait, subject to scheduling |
| Final output record | At most 4,096 bytes |

The unchanged F1 frame, manifest, block, whole-history and state-work caps also
apply to every response. The aggregate byte allowance does not enlarge a single
history's bounds. It counts positive read/write syscall results, including failed
requests, partial output and trailing-data sentinels. It is not a total resident
memory, physical-bandwidth or kernel-buffer limit.

## 2. Startup and fixed selection

After complete argument validation, install the Linux graceful-shutdown handling
and strictly call `NodeControlV2::open_writer` with the explicit paths and
independent C/genesis. Acquire control ownership before store ownership exactly
as A1 requires. Missing, invalid, pending, mismatched or busy state rejects before
any listener is bound. Failed opening never becomes initialization or recovery.

Successful opening performs A1's existing full storage replay, exact selection
checks and synchronization. It is not the synchronization-free `status` operation.
The provider neither writes a new selection nor mutates history after opening.

Retain that one ready controller for the whole lifecycle. Borrow its chain
immutably for every transfer and retain its complete checkpoint for the final
report. Do not clone a substitute history, reacquire either root between sessions,
refresh the selected branch, adopt a candidate or infer authority from a peer.
Concurrent A2 `status`, `sync-once`, initialization and recovery cannot acquire
their conflicting control ownership; they report busy. This availability tradeoff
is part of the first provider slice.

The admission clock starts immediately before binding, after successful A1
opening. Bind one listener to the requested address and set it nonblocking.
Do not clone, export or replace the listener. A bind/setup failure returns an
operational error after releasing resources. If a stop request was observed
during startup, successful local opening can finish without binding or admitting
a connection. Opening/replay itself is not asynchronously preempted.

## 3. One provider-owned admission loop

Construct one `PeerSessionsV2` from the validated limits and retain it until exit.
The process does not replenish budgets by creating another session owner or
listener. Statistics are copied only for reporting; they are not a reset permit.

Before every accept attempt, inspect the sticky stop flag, admission deadline,
remaining byte allowance and attempt count. When multiple ordinary stop conditions
are observed together, use this order: shutdown, admission timeout, wire limit,
then attempt limit. Idle `WouldBlock` and interrupted accept do not consume an F2
attempt; they return through these checks. A wait sleeps for no more than one
millisecond or the remaining admission duration, whichever is smaller.

After a stream is accepted, check stop and admission time again before passing
it to F2. Close a stream caught by either check without calling F2, without reading
its request and without charging an admitted session. Such a raced connection is
not a successful application admission. This second check cannot turn a connection
already admitted before a later stop request into a new connection.

Pass one admitted stream to `serve_connection` and make no parallel accept call
until it returns. F2 charges that attempt before request processing and owns the
stream until success or failure. Every admitted failed session retains its charged
attempt and actual bytes. The provider then updates completed/rejected counts and
returns to the same owner and the same admission checks.

Kernel backlog entries, a client's successful TCP handshake and bytes not read
by the application are not admitted F2 sessions. Other processes and listeners
are outside this invocation's bounds. No per-human, per-address, authenticated or
system-wide admission guarantee is introduced.

## 4. Narrow accepted-stream integration with F2

Add the following entry point to the existing session owner:

```rust
sessions.serve_connection(
    stream: TcpStream,
    identity: ManifestIdentityV2,
    chain: &ChainV2,
) -> Result<HistorySummaryV2, SessionErrorV2>
```

It immediately puts the consumed accepted stream under a close/shutdown guard,
including budget and identity preflight errors. It charges one attempt against
the same owner, establishes that attempt's original absolute deadline, independently
checks the provider history's network, and reuses the existing checked provider
request/EOF/response routine. It must not introduce a second request decoder,
frame encoder or unaccounted read/write path.

Read and validate exactly one 72-byte MPQ2 request, require request EOF before
any response, stream the unchanged F1 Begin/Manifest/all Block/End sequence, then
shut down the response write half. Wrong-network, malformed, incomplete or trailing
requests receive no history. Every result consumes/closes the passed stream.

Existing `serve_once` still consumes its own listener, starts its deadline before
waiting, accepts at most one client and drops that listener before processing the
stream. Its original waiting time remains inside its attempt deadline. Existing
`request` behavior, original deadline checks, byte accounting and all frozen bytes
remain unchanged. Both provider entry points share the checked stream routine;
the new one does not reset or extend a deadline established by `serve_once`.

This targeted additive/refactoring change to F2 is an explicit exception to the
previous checkpoint's blanket lower-layer source preservation statement. It is
required to move admission into the provider without pretending that dropping an
outside listener clone cancels F2's private accept loop. Preserve all earlier F2
tests and semantics, A1/store/consensus behavior and protocol/storage/hash formats.

## 5. Explicit graceful shutdown and lifecycle duration

Linux `SIGINT` and `SIGTERM` request graceful shutdown. The handler only sets a
sticky atomic flag: it does not allocate, format output, perform filesystem or
socket I/O, lock a mutex, choose recovery or exit from inside the handler. The
flag is not cleared during the lifecycle. Signal setup is scoped to `serve`, after
valid parsing; standalone help and the existing commands keep their prior behavior.

When idle, the next bounded admission check observes shutdown and closes the
listener. A stream accepted during a shutdown/time race is discarded by the
post-accept check. When a connection was already admitted, no parallel accept is
running: the current F2 attempt completes or rejects under its original deadline.
Do not refresh that deadline, reconnect, repeat the request or start another
attempt merely because shutdown was requested.

The same drain rule applies when the admission window expires during an active
attempt. The window stops new admissions; it is not an asynchronous cancellation
deadline for the already admitted connection. The serving phase therefore has
an admission window plus, at most, one original attempt timeout of final drain.
With the configured ceilings these allowances total at most 120,000 ms.

This is deadline-aware synchronous behavior, not a hard process wall-clock or
instantaneous-shutdown guarantee. Encoding, allocation, local replay, kernel
operations and scheduling are not preempted. Existing F2 checks reject late
results after bounded processing. Initial A1 opening/replay and final output are
outside the admission-plus-drain interval. No socket work may obtain a fresh
deadline merely to finish after shutdown.

Release every stream, then the listener and ready controller, before final output.
A1 retains store-before-control release ordering. Scope signal handlers correctly
on normal completion and failures, including partial signal-setup failure.
An uncatchable process kill is separate forced termination; kernel descriptor
release after that event is not evidence of graceful drain or physical power-loss
safety.

The node adds a Linux-target direct dependency on the **already locked `libc`
0.2.189** for the signal boundary. The node manifest and its lockfile dependency edge may
change; no new package, package version, source or checksum is selected. This is
the explicit dependency exception for B1. Existing A1, store, consensus, V1 and
encoding behavior and all monetary/enrollment/device policies remain unchanged.
The CI job budget rises from 15 to 20 minutes: the prior accepted checkpoint ran
for about 14 minutes 30 seconds, and the new focused process suite adds about
24 seconds in local debug execution before its release rerun. Every existing
verification command and assertion remains required.

## 6. Session rejection and fatal failures

Malformed requests, wrong request identity, early close, trailing data, stream
I/O failure and original-deadline expiry are rejected sessions. They may be
followed by another independently admitted connection while the same budgets,
admission window and stop state permit it. They never cause reconnection to the
failed peer or restoration of consumed allowance.

A1 has already checked that the fixed accepted chain and independently supplied
identity agree. F2's `WrongNetwork` during this invocation therefore identifies
an untrusted request mismatch; the provider does not weaken local identity checks.

A `WireBudgetExhausted` result rejects that session and ends admission. Resolve
the stop reason using section 3's priority, including any concurrently observed
shutdown or admission expiry; otherwise report `reason=wire-limit`. A fatal listener/setup failure, local
controller failure, provider history error, allocation failure or invalid internal
limit terminates with a static operational diagnostic and exit 1. Do not downgrade
local accepted-history failure into a harmless malformed-peer result.

Bind failures use `listen_bind`, later listener I/O failures use `listen_io`, and
signal installation failures use `signal_setup`. These classes contain no raw
socket address, path or nested operating-system error text.

Completed plus rejected sessions equals charged attempts in every normal final
summary. A raced stream closed before F2 does not contribute to those counters.
The reported `last_session_error` retains the static category of the most recent
rejected session; subsequent successful sessions do not clear it. It is `none`
only when no session was rejected.

Reuse the existing sanitized diagnostic categories, including
`invalid_peer_request`, `peer_wrong_network`, `peer_early_close`,
`peer_trailing_data`, `peer_deadline`, `peer_io`, `peer_unreachable` and
`peer_io_budget` as applicable. Do not echo arguments, paths, peer payloads or
nested errors. Listener/local/internal failures are operational failures rather
than entries in a successful final summary.

## 7. Final output and exits

There are no readiness, per-peer, progress or debug writes to stdout/stderr while
the listener/controller is held. A blocked output consumer must not extend their
ownership. The required nonzero listen address avoids needing a startup message
to discover an ephemeral port. Test observation does not add a production hook.

After resource release, a normal stop emits one bounded record using the existing
output format. The sample values below are explanatory placeholders:

```text
format=mirror-node-v2-1
operation=serve
status=stopped
config_hash=<64 lowercase hex characters>
genesis_hash=<64 lowercase hex characters>
store_selection=verified
selected_generation=<canonical decimal>
selected_height=<canonical decimal>
selected_tip_hash=<64 lowercase hex characters>
selected_ledger_root=<64 lowercase hex characters>
reason=<shutdown|admission-timeout|attempt-limit|wire-limit>
attempts=<canonical decimal>
completed_sessions=<canonical decimal>
rejected_sessions=<canonical decimal>
wire_bytes=<canonical decimal>
last_session_error=<static category|none>
```

The selected checkpoint identifies the fixed history validated under the held
owner. It is not a claim that another process cannot change the store after
ownership release, nor a claim of latestness, human finality or remote adoption.
Provider completion only establishes the F2 local-write/shutdown result; it is
not an acknowledgment that a recipient validated or durably published the history.

| Exit | Meaning for `serve` |
|---:|---|
| 0 | Normal requested shutdown or admission/attempt/wire limit, including runs containing rejected peers |
| 1 | Startup, fatal local/listener/history/allocation or final-output failure |
| 2 | Invalid syntax, values or limits, rejected before filesystem/socket effects |

The complete record remains at most 4,096 bytes and uses fallible writes/flushes.
Final-output failure can follow completed serving. Report
`operation_may_have_completed=true` and direct explicit status inspection; never
restart serving or retry sessions because output was lost. Final output itself has
no hard deadline, but all provider sockets and directory owners are already gone.
Only the existing `status` command uses exits 3 and 4.

## 8. Acceptance before calling the component verified

Use real command processes with bounded parent waits, termination and reaping;
keep ownership-sensitive cases isolated from unrelated forks. Required evidence:

1. Reject invalid/missing/cross-command options and every bound before effects.
   Preserve all prior A2 command and standalone-help behavior.
2. Refuse missing, pending, malformed, wrong-network and busy state before bind.
   Preserve unsupported state. No failure initializes, repairs or recovers data.
3. Serve the same complete accepted history across multiple clients, including
   complete ledger, membership, approval and population state. Keep the same
   controller and selected checkpoint; strict fresh-process reopening agrees.
4. Hold both roots during idle and active serving. Concurrent A2 status/sync is
   busy, with no outbound peer contact from the refused sync command. Release
   ownership on normal limits, signals, fatal failures and forced termination.
5. Charge successful and failed sessions against one attempt allowance, and
   retain actual malformed/partial bytes across clients. No new application
   session follows attempt/wire exhaustion; a tiny remaining budget is not reset.
6. Reject malformed, wrong-network, incomplete, trailing and missing-EOF requests
   before producing history. Slow/drip input and stalled output cannot renew
   the original attempt deadline.
7. Exercise both SIGINT and SIGTERM, including idle shutdown with zero admitted
   attempts and in-flight drain/rejection. Admit no subsequent application
   session, retain the original deadline and prove listener/lock release.
8. Exercise admission expiry while idle and active. Verify the documented final
   drain, without claiming hard CPU preemption or confusing a queued TCP
   handshake with a newly admitted application session.
9. Prove output failure occurs after owner release, preserves the selected state,
   reports possible completed serving and causes no automatic restart/retry.
10. Preserve every retained runtime/doctest/reference/vector/example gate and run
    all required release suites. Verify source publication and exact-source CI
    separately from subsequent checkpoint-documentation CI.

Linux `/proc` child descriptor/socket/wait-state evidence may establish that a
real child has accepted a stream or is blocked in actual I/O. Keep those checks
bounded and tied to the specific child and connection. Do not infer accepted
application work merely from parent-side `connect` success or add unbounded
production readiness output to make a test easier.

## 9. Remaining scope and policy

No automatic initialization/recovery, history refresh, reconnect/rejoin,
multi-peer scheduling, transaction pool, wallet/signer or broader operator API
belongs to this slice. The cooperative Linux directory-inode locking and trusted
local operator/parent-path assumptions remain unchanged. F2 remains unauthenticated
and unencrypted; finite admission is not peer/person authentication or network
availability certification.

The combined ECG/designated-pinky/supervised Ant Kiss and accessible-exception
decisions/proposals remain recorded with their existing evaluation gates. No
monetary rule, founder privilege, issuance, biometric archive, real private key
or device purchase is introduced. Full MB-04, complete Ant Kiss, final human
consensus and Gate A/B acceptance remain separate unfinished work. Passing this
finite component is not a mainnet, external-audit or physical power-loss claim.
