# Next session — MB-04C1

Roadmap MIRROR-PLAN-001. Updated **2026-11-27**.

## Recover live state first

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 32, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [run lifecycle](docs/NODE_RUN_V2.md),
[failover](docs/NODE_FAILOVER_V2.md), [rejoin](docs/NODE_REJOIN_V2.md),
[provider](docs/NODE_PROVIDER_V2.md), [operator](docs/NODE_OPERATOR_V2.md),
[control](docs/NODE_CONTROL_V2.md), [sessions](docs/PEER_SESSION_V2.md),
[ledger](docs/LEDGER_STATE_V2.md), [chain](docs/CHAIN_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Preserve all prior contracts and newer/unpushed work.

Implementation source: **e768268fae2e4c5c9980cc74dc7a9973da56849c**.
Tested source tree: **daff130cc7652ea0946106f46a600c54671ecf18**.
Parent checkpoint: **e421494a5a2ba28992ad581af4067618c38c0dae**.
[Source PR CI 45941543950](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45941543950)
passed all **43 steps**. Full job `140316760168` logs confirm **1,281 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`e51ff7da8385981dd9d94e4329ed35861f2dc31f` has no file differences from the implementation source.
All **16 published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **203 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Current position

**MB-04B4 is VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE.**
B3's explicit-peer failover, B2's rejoin/serve, B1's provider/draining shutdown,
A2's operator commands, A1's durable control/recovery and MB-03's finite storage
and peer components remain verified within their accepted scope. MB-04 remains
IN_PROGRESS: a pending-operation pool, wallet, wider operator APIs and broader
network scheduling remain unfinished. B4 is finite and does not complete the
production persistent-node work. Complete Ant Kiss remains MB-05 and final human
consensus MB-06. Gate A/B remain UNMET.

## Exact next work

**Next: MB-04C1 — bounded local pending-operation pool. READY — DESIGN FIRST.**

Specify a local in-memory component for bounded signed `SignedOperationV2` inputs,
covering transfers and membership operations without changing their accepted bytes
or execution rules. Separate retained input, contextual validity, executable ordered
batches and canonical inclusion. Admission is neither spendability nor an inclusion
promise, and a pool must never acquire chain/store authority by holding a staged result.

Before implementation define independently named local entry, encoded-byte,
batch-size and cumulative validation/revalidation work caps. Check them before
input-sized copying or expensive validation. Canonical membership `max_pending`
is not a transaction-pool limit. Specify exact signed-operation-ID deduplication,
stable bounded ordering, full-pool behavior, expiry/anchor handling, conflicts and
removal. Transfer account and membership record sequences are separate domains;
do not collapse them into one actor/sequence namespace or invent fee replacement.

Bind evaluation to an actual immutable accepted `ChainV2` and its exact selected
parent checkpoint. Existing contextual/signature validation does not execute
balances or consume sequences; use the complete ordered mixed-operation ledger
stager to assess executability atomically. Define explicit reconciliation after
append or replacement, stale cached results, included/expired inputs, and handling
of gaps or future inputs. A narrow next-block-only first scope is possible, but
that choice and its failure behavior must be specified before code.

The pool must not bind staged ledger state, append or mine a block, sign, mutate
A1/store, persist, gossip, add submission CLI/API ingress or assume wallet custody.
Those remain separate increments. Verify duplicates, ordering, overspend/sequence
conflicts, mixed-operation late failure, caps and selected-parent changes without
mutating the accepted chain. Preserve zero-fee/no-mint development semantics;
production monetary, enrollment, device, privacy and accessibility decisions remain
at their existing unresolved gates. C1 is planned only. Broader scheduling, wallet,
operator APIs, full MB-04 and Gate A/B remain unfinished.

## Preserve the accepted B4 run boundary

The separate `mirror-node-v2 run` command requires all twelve deliberate
options. It uses B3's ordered one-to-four numeric peer list and existing common,
listen, attempt, byte, timeout and admission options, plus `--max-rounds` in 1..4
and `--round-wait-ms` in 0..60000. The old 32-argument, 4096-byte-token and
16384-total-byte limits remain. All validation precedes effects; existing commands
reject the new flags and `run` rejects the single-source `--peer` flag.

Strict clean A1 opening occurs once. One controller, sticky signal guard and
cumulative F2 owner span every acquisition, admitted keep/adopt, service phase and
wait. Successful private round helpers return the same owner after closing their
listener. Earlier commands remain dropping wrappers. Neither root is reopened or
released between rounds; at most one stream is active and acquisition pauses service.

Each entered round restarts the same ordered list at entry one, with one request
per entry and B3's unchanged closed typed advancement policy. Checked round-local
deltas determine peer ordinals; the actual cumulative owner enforces budgets.
First complete validated history, including terminal EOF, ends acquisition even
when stale or tied. All-peer or fatal failure ends the invocation even after
earlier successful rounds, without another-round retry or serving an old branch.

A round completes only after validated receipt, admitted successful keep/adopt and
normal service return. Service may consume the remaining allowance; no attempt is
reserved for a later round. Overall stop priority is shutdown, wire-limit,
attempt-limit, round-limit. Only a normal service admission expiry with allowance
and rounds remaining permits a bounded signal-aware wait and another acquisition.
Waits hold the roots but no listener or stream, and never occur after a final stop.

The unchanged invocation caps are 1..8 total attempts and 1..157129488 actual I/O
bytes, with each attempt timeout and per-round service admission in 1..60000 ms.
Failed/partial traffic is charged without refund. The terminal EOF probe still
needs one unused byte. For total attempts M, timeout T, round limit R, admission A
and wait W, `M*T + R*A + (R-1)*W <= 900000 ms` bounds configured network,
admission and wait allowances. Opening, synchronous replay/adoption/processing,
scheduling and output are separate; this is not a hard wall-clock, CPU or RSS bound.

Shutdown before adoption discards a validated candidate; admitted A1 completes or
fails. An active service request drains under its original deadline. Once any
round admits keep/adopt, every later fatal error preserves its original class,
possible completion and inspect-status guidance, including errors in later receipt
or bookkeeping. Keep-current admission also establishes that sticky context.

The fixed normal record has `30 + 16*C + 7*I` fields for completed rounds C and
optional incomplete acquisition I, with C+I<=4. It distinguishes initial, each
completed and final selected checkpoints, checked phase totals and retained prior
errors. Error context adds fifteen fixed keys and eight for a current acquisition;
it does not assert a selected checkpoint or unreturned service outcome counts.
Invalid optional evidence falls back to the safe original diagnostic. All sockets,
roots and signal guards are dropped before final output; output loss never repeats work.

## Preserve the accepted B3 failover boundary

The separate `mirror-node-v2 rejoin-failover-serve` command accepts one to four
ordered numeric peers in a single required `--peers ADDRESS[,ADDRESS...]` token.
The other nine deliberate options, 32-argument/4096-token-byte/16384-total-byte
limits and effect-free validation remain. Reject empty/malformed/scoped lists,
duplicate numeric endpoints including IPv4-mapped aliases, DNS and zero ports.
Original parsed connection addresses are retained; no distinct-physical-node
or peer-authentication claim follows from endpoint checks.

Strict clean A1 opening precedes all contact and binding. One controller, sticky
signal guard and cumulative F2 owner span a finite ordered acquisition prefix,
at most one admitted keep/adopt and fixed-history service. Each configured entry
is requested at most once. Classify the typed session/history error before
sanitizing it: only the explicit contract allowlist advances. Allocation,
authority, accounting/global-resource and all unlisted errors remain fatal,
including ambiguous nested configuration/operation/ledger failures. A failed
receive plus shutdown remains an operational error, with no next-peer contact.

The first complete validated candidate permanently ends acquisition, including
identical/shorter/tied keep-current choices. Matching End and terminal EOF remain
mandatory. The shared B2 post-receive routine handles adoption admission, existing
A1 durability, retained-controller service and checked phase deltas. Stop before
admission discards a validated candidate; admitted adoption finishes or fails.
All-peer failure never serves the old branch as a fallback.

Configured caps remain 1..8 total receive/service attempts and 1..157129488
actual I/O bytes;
each original attempt timeout and post-adoption service admission remain
1..60000 ms. Failed/partial traffic is charged without refund or replenishment.
The EOF probe needs one unused byte even though zero-byte EOF is not charged.
A final allowed successful receive can adopt then stop normally without binding.
For P peers, M attempts, timeout T and admission A, conservative configured phase
allowances are min(P,M)*T+A+T (at most 360000 ms) and M*T+A. All service attempts
remain within M; these are not hard wall-clock/CPU/RSS guarantees. Local opening,
adoption, nonpreemptible processing, scheduling and output retain their qualifications.

Normal output has 32..35 fields: B2's 26 fields plus bounded acquisition totals,
first validated ordinal, latest receive error, terminal acquisition reason and
one static result per configured entry. Service rejection evidence stays separate.
Errors attach sanitized acquisition evidence without selected-pin assertions.
Every adoption error and later fatal setup/service failure retains its category,
possible-completion warning and inspect-status guidance. Invalid optional trace
formatting cannot erase that core diagnostic. All sockets/owners/signal guards
are released before final output; output loss never repeats the operation.

## Preserve the accepted B2 rejoin boundary

The separate `mirror-node-v2` executable adds `rejoin-serve`. It performs
at most one explicit complete receive, existing A1 keep/adopt, then finite serving
of the resulting fixed history. The contract was written before implementation:
[NODE_REJOIN_V2.md](docs/NODE_REJOIN_V2.md). Existing A2 commands, B1 `serve` and
the V1 default invocation remain available.

| Boundary | Delivered behavior |
|---|---|
| Startup | Complete argument validation, one signal guard, strict clean A1 opening before either outgoing contact or bind. Missing, pending, corrupt, wrong-network or busy state refuses. |
| Receive | One unchanged F2 request, actual complete history execution, matching End and mandatory terminal EOF. A failure is fatal even if shutdown was requested; there is no serve-old fallback. |
| Adoption | A stop observed after validated receipt but before adoption admission discards the candidate. Once admitted, synchronous A1 keep/adopt finishes or fails through its unchanged durable ordering. |
| Ownership | Retain `outcome.into_control()` directly. The same controller, signal state and F2 attempt/byte allowance span both phases; no reopen, reset or concurrent receive/serve. |
| Service | Check shutdown and exhausted budgets before binding. Serve the selected whole branch with B1 admission and original-deadline drain. A last-allowed receive can adopt and then stop without binding. |
| Reporting | Release sockets and both directory owners before a bounded 26-field result. Report previous/selected pins, actual choice and separate receive/service usage plus combined totals. |

All ten options are deliberate: separate control/store paths, independent C/genesis,
one numeric nonzero-port `--peer`, numeric nonzero-port `--listen`, `--max-attempts`
1..8, `--max-io-bytes` 1..157129488, `--timeout-ms` 1..60000 and `--admission-ms`
1..60000. Existing 32-argument, 4096-byte-token and 16384-total-byte parser bounds
remain, with no DNS, environment/stdin/config fallback or effects before validation.

Receipt charges one attempt and its actual I/O. Service uses only the remainder.
An allowance equal to the 72-byte request plus complete response bytes is still
insufficient: the terminal EOF probe needs one unspent byte of capacity even though
zero-byte EOF is not charged. The service admission clock starts after adoption,
immediately before bind when permitted. Receive timeout plus admission window plus
at most one final service timeout totals at most 180000 ms at configured ceilings.
This excludes opening, durable adoption and output and is not hard CPU preemption
or a complete wall-clock guarantee. Standalone B1 retains its 120000-ms allowance.

The post-receive stop check admits adoption. Signals after that check do not cancel
A1 or imply rollback. Every adoption error or later fatal bind/listener/service error preserves
its static error class and reports `operation_may_have_completed=true` with
`next=inspect-status`; no error asserts unresolved selected pins. Final output loss
uses the existing conservative rule, including normal pre-adoption shutdown.
No implicit initialization, recovery, retry or rollback is introduced.

Normal results obey receive_attempts + serve_attempts = attempts,
completed_sessions + rejected_sessions = serve_attempts, and receive_wire_bytes +
serve_wire_bytes = wire_bytes. A validated candidate discarded on shutdown reports
`receive_status=validated`, `choice=not-attempted` and the unchanged verified
selection. Normal limits/signals exit 0; operational/output failures exit 1 and
usage errors exit 2. Only status retains exits 3 and 4. No result claims peer
adoption, global latestness, human finality or protection against a later local writer.

## Preserve the accepted B1 provider boundary

The existing `mirror-node-v2` executable now has an explicit finite `serve`
command. All A2 commands and the V1 default invocation remain available. The
complete contract is [NODE_PROVIDER_V2.md](docs/NODE_PROVIDER_V2.md).

| Boundary | Delivered behavior |
|---|---|
| Startup | Validate all arguments, install scoped Linux signal handling, then strictly open ready A1 control before binding. Missing, pending, corrupt, wrong-network or busy state refuses startup without listening. |
| Fixed selection | Retain one controller and its complete accepted history. Concurrent status/sync commands are busy; no history refresh or controller reacquisition occurs between clients. |
| Admission | One nonblocking listener, one retained F2 owner and at most one active admitted stream. Failed sessions keep their charged attempts and actual read/write bytes. |
| Shutdown | SIGINT/SIGTERM stop new admissions. An already admitted session completes or rejects under its original absolute F2 deadline. Admission expiry follows the same final-drain rule. |
| Final output | Release streams, listener and both directory owners before one bounded result record. Report full selected pins, reason, charged attempts/bytes and completed/rejected counts. |

Require explicit control/store paths, independent config/genesis hashes, numeric
nonzero-port `--listen`, `--max-attempts` 1..8, `--max-io-bytes` 1..157129488,
`--timeout-ms` 1..60000 and `--admission-ms` 1..60000. The existing parser caps remain
32 arguments after argv[0], 4,096 UTF-8 bytes per token and 16,384 total. No defaults,
DNS, environment/stdin/config fallback or side effects before complete validation.

The admission clock starts after A1 opening, immediately before bind. Check stop,
admission time, remaining bytes and attempts before accept, and check stop/time
again after accept. A raced stream closes before F2 and is not an admitted attempt.
A successful TCP handshake or kernel-backlog entry is not application admission.
One owner retains all attempt/byte charges across clients; creation of a fresh owner
cannot replenish this invocation's budget because the loop never creates one.

`PeerSessionsV2::serve_connection` adds an accepted-stream entry with an immediate
shutdown/drop guard, including preflight refusals. It shares the checked request,
EOF and history-output routine with `serve_once`; the latter retains its original
pre-accept deadline and consumed-listener ordering. The 72-byte MPQ2 request,
mandatory request EOF, complete F1 history and all original encodings remain.

The handler only sets a sticky atomic stop flag and prior handlers are restored on
exit. Final drain retains at most one original attempt timeout after the admission
window; configured ceilings total 120,000 ms. This is deadline-aware synchronous
behavior, not a hard process wall-clock guarantee: processing and scheduling are
not preempted, and initial open/replay plus final output lie outside that interval.
A1 opening may synchronize; it is distinct from read-only status. SIGKILL is forced
termination and does not establish graceful drain or physical power-loss safety.

Ordinary stop priority is shutdown, admission timeout, wire limit, then attempt
limit. Completed plus rejected equals charged attempts; the last static session
error persists across later successes. Normal limits/signals exit 0 even with
rejected peers, operational/output failures exit 1, and invalid commands exit 2.
Only status uses exits 3/4. Peer input errors can consume a bounded attempt; local
history/allocation/setup/listener errors terminate rather than masquerading as peer
rejections. Diagnostics echo no raw paths, addresses or peer bytes.

There is no readiness/progress output while resources are held. Final records stay
within 4,096 bytes. Output failure may follow completed serving and reports
`operation_may_have_completed=true` with explicit status inspection, without retry.
Provider completion reports local transfer completion, not remote validation,
durable adoption, global latestness or human finality.

## Preserve the accepted A2 command boundary

A separate `mirror-node-v2` executable composes the accepted A1 controller
and F2 peer receiver. The existing V1 executable source is unchanged; the package
sets `default-run = "mirror-node"` to preserve its prior default invocation.

| Command | Delivered behavior |
|---|---|
| `init --attach-existing` | Initialize only the separate control state for an **already initialized** profile-3 store, under independent C/genesis and all height/tip/ledger-root/generation pins. No store creation, mining, overwrite or failed-open fallback. |
| `status` | Call only A1 read-only status. Missing control exits 3, clean verified selection exits 0, validated pending intent exits 4; invalid/unsafe/busy state returns operational failure. |
| `sync-once` | Open and retain a clean ready controller before connecting, request exactly one candidate with one bounded F2 owner, then consume it through A1 adoption. No retry or automatic recovery. |
| `recover --action complete\|retain-old` | Forward the explicit action to A1's existing fully checked recovery. Selected/confirmed candidates cannot roll back. |

Every operational command requires explicit `--control-dir`, `--store-dir`,
`--config-hash` and `--genesis-hash`. The complete argument list is validated before
filesystem/socket operations: at most 32 arguments after argv[0], 4,096 UTF-8 bytes
per token and 16,384 total bytes. Reject duplicate/unknown/cross-command options,
non-UTF-8/control input, malformed exact-64 hexadecimal hashes, noncanonical
unsigned decimal, overflow and invalid init height/generation. Init height is
0..4095 and generation cannot exceed height. No env/stdin/file-config fallback.

Sync requires a numeric peer address with nonzero port, an explicit 1..60000 ms
timeout and positive I/O allowance up to 157,129,488 bytes. Its one F2 attempt
retains all prior header admission, replay, matching End and terminal EOF checks.
The timeout covers the session, not local open/replay, adoption or stdout; encoded
I/O bounds are not RSS, CPU preemption or global admission guarantees.

Output uses at most 4,096 bytes of deterministic `key=value` records. It reports
network and full checkpoints, successful sync choice/attempts/actual wire bytes,
or static non-echoing diagnostic categories. Pending fields use confirmed/old/
candidate labels with `store_selection=uninspected`; they do not assert current
store selection or infer which recovery actions are permitted. Usage exits 2
before effects; operational/output failures exit 1. Help exits 0 without paths.
Only status uses missing/pending exits 3/4.

Output failure after a mutating command can follow durable completion. The process
reports `operation_may_have_completed=true` and directs inspection instead of
retrying. Owners are released before stdout writing. Partial initialization and
unsupported/corrupt/unknown artifacts remain preserved under A1's rules.

## Preserve the accepted A1 boundary

A separate `mirror_node::v2` library now preserves trusted local restart authority
outside the strict profile-3 store. The operator supplies separate non-nested
control/store paths and independently chosen C/genesis. Both roots use cooperative
Linux directory-inode locks, always control first and store second. The ready
controller exposes no raw store writer; failures consume it and return no writer.

`NodeControlV2::initialize` explicitly attaches control to an already initialized
store under independent full checkpoint/generation pins. It never initializes or
repairs a store after failed opening. `open_writer` accepts only clean control and
strictly replays the selected store. `status` performs no application writes or
synchronization: absent/empty control is Missing, verified clean storage is Clean,
and fully replayed pending intent is RecoveryRequired. Invalid input rejects.
RecoveryRequired reports local intent and last-confirmed pins; it does not inspect
or assert the store's current selected endpoint.

MNC2 is an exact 464-byte confirmed record with full local checkpoint and unchanged
MSR2 canonical metadata. MNI2 has a 952-byte header, both complete unchanged MPN2
history streams and a checksum; its maximum is 39,283,208 bytes. Loading binds the
external network identity, executes both complete histories, checks both canonical
records and reuses existing strict replacement/generation rules. Checksums detect
corruption; they cannot defeat a hostile operator who replaces all trusted state.

Consuming adoption accepts a completed F1/F2 candidate. Ties, shorter and identical
histories keep the current controller without filesystem calls. Strict preference
retains an old-chain clone and moves the complete candidate into intent. The exact
intent is written, synchronized, read back, fully replayed, renamed and its control
directory synchronized BEFORE invoking existing storage replacement. The ready
writer rechecks confirmed bytes, intent and exact layout immediately before that
mutation. Accounts, memberships, approvals, sequences and population history always
come from one complete accepted branch.

After storage returns clean success, the exact new confirmed record is staged,
synchronized, renamed and the directory synchronized. Only then is the exact intent
removed and that absence synchronized. A retry reestablishes intent durability
before storage recovery and confirmed durability before retiring intent; visible
renames alone are insufficient. Clean writer reopening also resynchronizes the
confirmed record/control directory/parent before another adoption.

Startup never completes or retains pending intent automatically. Explicit recovery
selects Complete or RetainOld through existing storage rules. Once candidate
confirmation has begun, only Complete with a strictly clean candidate store is
permitted. Old-selected storage contradicting candidate confirmation rejects before
mutation. A selected candidate cannot roll back.

`intent.pending`, incomplete control initialization, unknown entries, corrupt data,
overlong files and unsafe links are preserved and rejected. The only recoverable
confirmation staging is an exact 0..464-byte candidate-record prefix beside valid
old confirmation and complete intent; explicit Complete extends it without truncation.
Candidate confirmation plus remaining intent is also explicitly completable. A
failure after intent retirement may leave an already confirmed clean state; errors
do not always leave intent and do not promise arbitrary corruption repair.

Control reads check type, length, inode and hardlink count, allocate only after
checked length-plus-one admission and charge one 314,269,392-byte per-call allowance.
Storage keeps its separate bounds. Encoded sizes are not RSS/time guarantees;
history copies, checksum buffers and replay consume additional memory. Parent paths
and local operators remain trusted. Process termination tests are not physical
power-loss certification or hostile-same-user race protection.


## Acceptance carried forward

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed with **203 frozen noncheckpoint files unchanged**:

- **1,281 runtime tests**, zero failures: 1,242 retained plus **39 new** — six
  parser, five output, one failure-context, one diagnostic-fallback, seven lifecycle
  unit tests and **19 actual-process tests**. Parameterized cases, subprocesses and
  release reruns are not additional unique test functions.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers are exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **262+1**. All **twelve references** and **sixteen examples** passed.
- The focused first execution passed **82 binary tests and all 19 new process
  tests**. Scoped formatting, whitespace and allocation/runtime-data checks passed.

Actual processes cover two preferred adoptions under unchanged root descriptors,
complete reopened history/accounts/membership/counters, three real executables and
an explicit restart, repeated stale first-valid selection, and actual four-round
execution with one eight-attempt allowance. A three-round sequence with earlier
failed-peer and rejected-service attempts, ending in clean receipt/service, proves
earlier errors remain visible and peer ordinals restart independently of prior usage.

The suite exercises first/later all-peer or fatal refusal, keep-current followed
by a later fatal error, cumulative receive/service bytes and terminal-EOF exhaustion,
observed inter-round waits under SIGINT/SIGTERM, later validated-candidate discard,
original-deadline failure, later active-service drain, second-adoption bind failure,
closed/blocked final stdout, and SIGKILL during a later partial receive. A live
second-adoption SIGTERM case observes and verifies the actual A1 intent before
queuing shutdown; the admitted operation finishes without binding afterward.

Linux observations identify owned directory descriptors and relevant socket/blocked
syscall or verified intent phases. Busy competing commands and unused listeners
check retained ownership and absence of later contact. Conservative independent-
value output envelopes format to exactly **3918** and **3529 bytes**; these are
synthetic width bounds, not simultaneously reachable runtime counter combinations.
Validated normal records and additive help stay within the unchanged 4096-byte cap.

Tests use Linux IPv4; numeric IPv6 parsing is separate CLI evidence. Pre-first-round
and narrow between-request signal races retain source-level coverage. New SIGKILL
evidence concerns a later partial receive, not forced death at every A1/fsync or
service boundary. Process death and replay do not prove physical power-loss safety.
No production test hook, induced OOM, hidden retry or untested-filesystem guarantee
is introduced. First-valid repeated rounds do not prove convergence or latestness.

**164 of 172 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Seven existing private command modules change intentionally:
args, errors, failover, main, output, provider and rejoin. The sole retained test
helper change appends an observed-wait method; all previous helper bytes remain an
exact prefix. All **62 earlier private unit-test bodies** and every other retained
test/helper file remain byte-identical. Four new files add the coordinator,
process suite/helper and contract; four existing command contracts gain additive
cross-references. Public/lower-layer/V1/A1/F2/storage/consensus/monetary source,
Cargo manifests/lockfile, reference tools, frozen fixtures, workflow commands and
the 20-minute CI budget remain unchanged. No wire, storage or hash encoding,
enrollment/device or monetary policy changed.

## User policy carried forward

No founder-specific or marketing reserve. The founder is one of the first 1,000
qualifying humans eligible for the same 10 MRY; ordinary allowance remains 0.2 MRY
per qualifying historical population maximum. The combined formula and fees/rewards
are open D-01/D-04 work. Easy PoW remains a development profile. The combined
ECG/designated-pinky/supervised Ant Kiss and accessibility proposals retain their
future evaluation gates. No personal records, biometric archive or real keys belong
in code, GitHub or on-chain. No user monetary/admission/device policy changed here.
