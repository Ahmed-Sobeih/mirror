# Current status

Roadmap [MIRROR-PLAN-001](MASTER_PLAN.md). Updated **2026-11-27**.
Checkpoint **MIRROR-MB04B4-001 — bounded persistent-run lifecycle**.

**MB-04B4 is VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE.**
B3's explicit-peer failover, B2's rejoin/serve, B1's provider/draining shutdown,
A2's operator commands, A1's durable control/recovery and MB-03's finite storage
and peer components remain verified within their accepted scope. MB-04 remains
IN_PROGRESS: a pending-operation pool, wallet, wider operator APIs and broader
network scheduling remain unfinished. B4 is finite and does not complete the
production persistent-node work. Complete Ant Kiss remains MB-05 and final human
consensus MB-06. Gate A/B remain UNMET.

## Exact source and publication

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

## Delivered behavior

[NODE_RUN_V2.md](docs/NODE_RUN_V2.md) was written before implementation.

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

## Acceptance evidence

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

## Exact next action

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

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 32, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [run lifecycle](docs/NODE_RUN_V2.md),
[failover](docs/NODE_FAILOVER_V2.md), [rejoin](docs/NODE_REJOIN_V2.md),
[provider](docs/NODE_PROVIDER_V2.md), [operator](docs/NODE_OPERATOR_V2.md),
[control](docs/NODE_CONTROL_V2.md), [sessions](docs/PEER_SESSION_V2.md),
[ledger](docs/LEDGER_STATE_V2.md), [chain](docs/CHAIN_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Preserve all prior contracts and newer/unpushed work.
