# MB-04B4 — bounded persistent-run lifecycle

This contract specifies the separate `mirror-node-v2 run` command before its
implementation. It repeats finite acquisition, admitted keep/adopt and service
rounds while retaining one controller, one signal guard and one cumulative F2
allowance. It builds on [B3 failover](NODE_FAILOVER_V2.md),
[B2 rejoin](NODE_REJOIN_V2.md) and [B1 service](NODE_PROVIDER_V2.md).
Checkpoint records distinguish design, implementation and executed acceptance.

The invocation is finite. Explicit rounds authorize repeated use of the supplied
peer list; no peer discovery, parallel acquisition/service, automatic recovery,
indefinite daemon, transaction pool, wallet or policy selection is introduced.
First-valid ordering can repeatedly select a stale valid peer. It does not imply
convergence, best-peer selection, authentication or global latestness.

## 1. Explicit invocation and admission

```text
mirror-node-v2 run
  --control-dir PATH --store-dir PATH
  --config-hash HEX --genesis-hash HEX
  --peers ADDRESS[,ADDRESS...]
  --listen ADDRESS
  --max-attempts UINT --max-io-bytes UINT
  --timeout-ms UINT --admission-ms UINT
  --max-rounds UINT --round-wait-ms UINT
```

All twelve options are required. Preserve B3's one-to-four ordered numeric peer
list, fixed four-slot storage, duplicate/alias/scoped-address rejection, independent
network pins and existing scalar validation. The ordinary invocation has 25
argument tokens; retain the 32-argument, 4096-byte-per-token and 16384-total-byte
limits. Validate the entire invocation before signals, paths, owners or sockets.

`--max-rounds` is canonical unsigned decimal in **1..4**. `--round-wait-ms`
is canonical unsigned decimal in **0..60000**; zero deliberately authorizes an
immediate next round. The existing total-attempt, actual-byte, attempt-timeout
and per-round service-admission caps remain **1..8**, **1..157129488**,
**1..60000 ms** and **1..60000 ms** respectively. Configured caps do not imply
nonzero actual usage. Do not require the round limit or list length to fit the
attempt cap: exhausted allowance may end the invocation before all rounds.

The two new flags belong only to `run`. Existing commands reject them; `run`
rejects the single-source `--peer`. Existing operational flags, record semantics
and exit statuses remain unchanged. The help catalogue adds this command/options
within its existing 4096-byte output cap.

## 2. Retained ownership and private boundaries

Install one sticky signal guard and strictly open clean A1 authority once before
contact or binding. Missing, pending, malformed, wrong-network or busy local
authority refuses before contacting any configured peer. Do not initialize,
recover, choose a network from peer claims or reopen between rounds.

Create one `PeerSessionsV2`; keep it and the signal guard for the whole invocation.
Successful A1 adoption returns its retained controller through
`outcome.into_control()`. Private provider and post-receive helpers return that
owner on normal completion, including pre-bind shutdown/exhaustion. Drop each
round's listener before returning the owner. Fatal errors consume/drop the owner;
they never expose a usable controller or authorize an implicit retry.

Existing B1/B2/B3 entry points remain wrappers that explicitly drop returned
ownership before returning their existing records. No public A1/F2 API or
lower-layer semantics change. The same two directory descriptors remain owned
through acquisitions, adoptions, service and waits; reacquisition pauses service.
There is at most one active peer stream and no simultaneous listener/request loop.

## 3. Round lifecycle and advancement

Before entering a round, check sticky shutdown and cumulative wire/attempt
exhaustion. A normal stop here starts no additional round. Increment
`rounds_started` immediately before entering acquisition; a signal race can leave
a started round with zero requests.

Every admitted round begins with entry one of the unchanged ordered list. Request
each entry at most once in that round, preserving B3's exact closed typed failure
allowlist. Use complete F2 validation, matching End and terminal EOF. The first
validated candidate ends that round's acquisition, including identical, shorter
and tied keep-current candidates. The same entry may be used again only in a
later explicitly authorized round.

Each acquisition snapshots the existing cumulative counters and computes checked
round-local deltas. Validated ordinals and peer-prefix checks use those deltas,
not cumulative invocation attempts. Exhaustion always uses the real cumulative
owner. Reset only bounded per-round evidence; never reset/refund the owner.

All-peer failure, fatal typed failure or failed receipt plus shutdown remains an
operational error. This applies after earlier successful rounds too. Do not wait
for another round, retry the list, or serve the old branch after such failure.

After validated receipt, the existing stop check admits adoption. A prior stop
discards the candidate and retains the previous selection. Once admitted,
synchronous A1 keep/adopt finishes or fails through its existing durable ordering.
Then serve the resulting fixed history with the same remaining allowance and
the unchanged original-deadline drain behavior.

A round is completed only after validated acquisition, successful admitted
keep/adopt and a normal service return. Zero service attempts caused by shutdown
or exhaustion still count as a completed round. A candidate discarded before
adoption leaves one incomplete round. Thus:

```text
0 <= rounds_completed <= rounds_started <= round_limit <= 4
rounds_started - rounds_completed <= 1
```

After a completed round, overall stop priority is **shutdown, wire-limit,
attempt-limit, round-limit**, in that order. These checks occur before another
wait or round. A service phase may consume all remaining attempts; no allowance
is secretly reserved for later rounds. If no overall stop applies, service must
have ended with `admission-timeout`; any inconsistent internal result is fatal.
Only then enter the explicit wait and, if still permitted, the next round.

## 4. Waits and cumulative time bounds

Wait at most the configured W between completed rounds that can continue, at
most R-1 times. Retain controller, signal guard and cumulative sessions while
holding no listener or stream. Use a checked absolute wait deadline, bounded
short sleeps, sticky-signal checks during waiting and a fresh check before the
next acquisition. W=0 still performs stop checks. There is no wait after the final
round, exhaustion, shutdown or an operational error.

For configured total attempts M, attempt timeout T, round limit R, per-round
service admission A and wait W, a conservative network/admission/wait allowance is:

```text
M*T + R*A + (R-1)*W <= 900000 ms
```

Every receive and service attempt belongs to M; final drains do not add attempts.
There are at most R service windows and R-1 waits. Original in-flight timeouts are
never refreshed by a signal, round transition or wait. This derived finite bound
does not add a competing global network deadline or asynchronously cancel A1.
Local opening, synchronous adoption/replay/processing, scheduling and final output
retain their existing qualifications; it is not a hard total wall-clock, CPU or
RSS guarantee. Per-history/resource caps remain unchanged.

## 5. Shutdown and completion uncertainty across rounds

- Stop before the first round normally returns the initial verified checkpoint
  with zero rounds and attempts.
- Stop before any request in an entered acquisition may normally return an
  incomplete round with zero receive usage. Stop after a failed request preserves
  that failure, including during a later round.
- Stop during receive drains that request under its original deadline. Validated
  completion under a prior stop is discarded before admission; failed completion
  remains an error. No subsequent peer or round is contacted.
- An admitted adoption finishes or fails. Service honors existing B1 admission
  checks and drains an already admitted session under its original timeout.
- Stop during an inter-round wait returns normally at the last completed
  checkpoint, without another request or listener.

Once **any** round admits A1 adoption, every later fatal error retains
`operation_may_have_completed=true` and `next=inspect-status`, preserving its
original error category. This includes a keep-current admission and a later
receive, wait-deadline, accounting, bind or service error. A new round cannot reset
the invocation's completion context. Existing helper errors retain their own
admission annotation; successful returns establish sticky context before any
fallible report bookkeeping.

Errors make no unresolved selected-checkpoint assertion. Final output loss
retains the existing conservative rule and never repeats the operation. Drop all
sockets, owners and signal guard before formatting or writing a final normal or
error record. Invalid optional run evidence falls back to the original safe base
diagnostic without erasing error class, next action or completion uncertainty.

## 6. Fixed normal records

Use the existing format and 4096-byte buffer. Keep fixed storage for at most four
completed-round records and at most one incomplete acquisition; no growing history
of rounds or output during ownership is introduced. The normal record has
**30 + 16*C + 7*I fields**, where C is completed rounds and I is zero or one for an
incomplete acquisition, with C+I<=4.

The thirty base fields are the standard six state fields (`format`, `operation`,
`status`, `config_hash`, `genesis_hash`, `store_selection`), four `initial_*`
checkpoint fields, four `selected_*` checkpoint fields, and:

```text
round_limit rounds_started rounds_completed reason peer_count
attempts receive_attempts serve_attempts completed_sessions rejected_sessions
wire_bytes receive_wire_bytes serve_wire_bytes failed_receives
last_receive_error last_session_error
```

The operation is `run`, status `stopped`, selection `verified`; overall reasons
are `shutdown`, `wire-limit`, `attempt-limit` or `round-limit`. Initial and selected
checkpoints each contain generation, height, tip hash and ledger root.

For each completed round i, emit sixteen `round_i_*` fields:

```text
generation height tip_hash ledger_root choice validated_peer
receive_attempts receive_wire_bytes serve_attempts serve_wire_bytes
completed_sessions rejected_sessions service_reason
last_receive_error last_session_error peer_results
```

The checkpoint is that completed round's selected state. Its previous state is
the initial checkpoint or the prior completed round. `choice` uses existing
`keep-current`/`prefer-candidate` labels. `peer_results` is a comma-separated list
of fixed static B3 result labels for exactly the configured entries, never addresses.

When one entered acquisition remains incomplete on normal shutdown, append:

```text
incomplete_round incomplete_receive_attempts incomplete_receive_wire_bytes
incomplete_validated_peer incomplete_last_receive_error
incomplete_acquisition_stop incomplete_peer_results
```

An incomplete round never changes the selected checkpoint. Its trace may report
zero requests or a validated candidate discarded before admission. Preserve the
latest non-`none` acquisition/service error across rounds in the base record;
each round's fields describe that round only. Earlier errors are not erased by
later successful rounds.

Validate all counters, prefix/ordinal relationships, static labels and checkpoint
continuity before writing a normal record. Total receive plus service attempts
and bytes must equal owner totals; completed plus rejected sessions must equal
service attempts. Failed receives plus completed rounds plus an optional validated
incomplete receipt must equal receive attempts. Sum round-local usage and the
optional incomplete trace exactly. Use checked arithmetic throughout.

The planned conservative independent-value maximum is 3918 bytes for four
completed rounds; three completed rounds plus an incomplete acquisition is at
most 3529 bytes. Actual maximum-value formatting tests must verify the unchanged
4096-byte cap, including help. These are design bounds until executed acceptance.

## 7. Bounded error records

Retain the original safe base diagnostic. Optional run context reports configured
round/peer counts, started/completed rounds, a static phase, cumulative transport
and receive counters, checked service deltas and the current acquisition trace
when available. A combined `adoption-service` phase is used for a retained helper
whose fatal return cannot distinguish its inner subphase.

Emit fifteen fixed context keys:

```text
round_limit rounds_started rounds_completed phase adoption_admitted peer_count
attempts wire_bytes receive_attempts receive_wire_bytes
serve_attempts serve_wire_bytes failed_receives last_receive_error
last_completed_service_error
```

Phases are `startup`, `acquisition`, `adoption-service`, `wait` and `report`.
`adoption_admitted` is sticky and includes the current helper's admission
annotation on an error. Service usage is the checked difference between owner
totals and cumulative receive usage. `last_completed_service_error` explicitly
describes returned, completed service phases only.

When a current acquisition exists, append eight keys:

```text
current_round current_validated_peer current_failed_receives
current_receive_attempts current_receive_wire_bytes current_acquisition_stop
current_last_receive_error current_peer_results
```

The current round is `rounds_started`. Waits have no current acquisition. These
groups have fixed bounds and remain well within the same output cap.

Do not attach selected pins, raw peer/path values or nested error text. Do not
claim complete/rejected totals for a service phase that failed without returning
its report. Previously completed rounds and actual cumulative usage remain
distinguishable from the failing current phase. Run context supersedes B3's
standalone acquisition context rather than writing duplicate keys. Default-absent
run context preserves old diagnostics. Validate optional context before emitting
any of it; fallback preserves the original base if validation/formatting fails.

Normal configured stops exit 0, operational/output failures exit 1 and usage
failures exit 2. Only the existing status command uses exits 3/4.

## 8. Required implementation evidence and preserved scope

Before accepting implementation, run all existing gates and meaningful new cases:

- Actual effect-free option/list/cap rejection, existing-command flag separation,
  strict missing/pending/corrupt/mismatched/busy refusal, and maximum fixed output.
- One actual process running multiple rounds with real source/requester commands,
  complete reopened state and explicit invocation restart.
- Two preferred histories adopted in one invocation, retaining one whole selected
  branch at each checkpoint; first-valid keep-current repeatedly ends acquisition.
- A failed first peer followed by valid second peer in multiple rounds, proving
  round-local ordinals/counters despite earlier receive and service usage.
- Initial and later all-peer/fatal failure, sticky completion after earlier
  prefer-candidate and keep-current admissions, and no implicit round retry.
- Cumulative attempt/byte limits across rounds, failed-byte charges, terminal EOF
  capacity, normal last-allowed adoption without bind, and no wait when stopping.
- Actual retained root descriptor identities across service, a verified wait and
  later acquisition; competing commands remain busy without peer contact.
- SIGINT/SIGTERM during observed wait and later receive, validated-discard,
  original-deadline error, and later service drain without extra admission.
- Later bind/output failure, owner/listener release before blocked final output,
  and forced termination during a later receipt/adoption/service where claimed.

Reuse accepted observations and keep retained tests distinct from new B4 evidence.
A narrow additive test-only wait observer may extend existing support without
changing old helper/test bodies. No production hook, permission change, induced
OOM, exact fsync/power-loss or untested-filesystem guarantee is implied. Record
source-only timing branches honestly; process coverage is Linux IPv4, with IPv6
parser coverage separate.

Preserve earlier commands, public APIs, V1/lower-layer source, manifests/lockfile,
workflow commands and budget, references, fixtures and wire/storage/hash formats.
All monetary, enrollment/device, accessibility and privacy decisions/proposals
retain their existing gates. Full MB-04 and Gate A/B remain unfinished.
