# Current status

Roadmap [MIRROR-PLAN-001](MASTER_PLAN.md). Updated **2026-11-27**.
Checkpoint **MIRROR-MB04B3-001 — bounded explicit-peer failover**.

**MB-04B3 is VERIFIED_COMPONENT — BOUNDED EXPLICIT-PEER FAILOVER.**
B2 remains verified for finite rejoin/serve coordination, B1 for provider/draining
shutdown, A2 for operator commands, A1 for durable control/explicit recovery and
MB-03 for its finite storage and peer scope. MB-04 remains IN_PROGRESS: repeated
run scheduling, transaction pool, wallet and wider operator APIs remain. Complete
Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain UNMET.

## Exact source and publication

Implementation source: **3311151ab627b3512abed47be20adea2f0e6d66a**.
Tested source tree: **fc238ae2df95b7b2b3df2457d06f739048812c7e**.
Parent checkpoint: **9ca7d99681877d014ae64aab31cb5eb22a957c35**.
[Source PR CI 45933868084](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45933868084)
passed all **43 steps**. Full job `140292436668` logs confirm **1,242 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`0cada2958afe985bb5fcd3b683c57a40b01042ce` has no file differences from the implementation source.
All **12 published implementation/test/contract files** were read back byte-for-byte.
The later checkpoint updates nine status/handoff records and clarifies one
contract compatibility wording: existing operational records remain unchanged; the help
catalogue gains the new command and options. It changes no runtime/test file,
and the other 198 files frozen during source acceptance remain byte-identical.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; this
source record does not assert that a future documentation head is tested.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

[NODE_FAILOVER_V2.md](docs/NODE_FAILOVER_V2.md) was written before implementation.

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

## Acceptance evidence

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed with **199 frozen noncheckpoint files unchanged**:

- **1,242 runtime tests**, zero failures: 1,201 retained plus **41 new** — six
  parser, eight typed-policy, three output, one error-context, one diagnostic-fallback
  and **22 actual-process tests**. Scenarios, subprocesses and release reruns are
  not additional unique test functions.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers are exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **223+1**. All **twelve references** and **sixteen examples** passed.
- Scoped formatting, whitespace and allocation/runtime-data checks passed.

Separate refused-first and malformed-first tests run a real seed executable,
the B3 executable and a third requester with independent roots and a second
explicit restart cycle. Strict reopening compares complete histories, accounts,
membership records/bytes, approvals, sequences, population and resource counters;
the preferred fork has equal accounts but different membership. First validated
identical/shorter/tied candidates stop acquisition and leave later peers untouched.

Actual cases cover all four peers failing, eligible malformed/wrong-network/
incomplete/trailing/missing-EOF transfers, and a constructed nested manifest
configuration error that remains fatal without later contact. Fixed invalid-header
exchanges establish 84 charged bytes (72 request + 12 consumed header). Exact
EOF-capacity and payload-larger-than-positive-remainder cases verify shared bytes;
last-allowed receive succeeds without binding an occupied address. Later successful
acquisition and service retain distinct earlier rejection evidence.

Bounded Linux observations identify the child socket endpoints/inode/FD and blocked
syscall before SIGINT/SIGTERM. They verify original deadlines, failed-receive stop,
later validated-candidate discard, both exact directory descriptors across two
receives and service, and busy competing commands with no outgoing contact.
Bind/closed-output failure preserves adopted state and possible-completion context.
Blocked final stdout releases the listener/owners before the write finishes;
SIGKILL during a partial second receive preserves old state. Unused listeners
remain reserved until exit when absence of later contact is asserted.

The retained B2 live-adoption signal and service-drain/no-rollback tests exercise
the same extracted post-receive path; they are separate retained evidence, not
22 additional B3 scenarios. No new production hook, permission change, induced
OOM, hidden retry, power-loss or untested-filesystem guarantee is introduced.
Pre-first-request zero-attempt and brief between-request shutdown branches retain
source-level evidence. Process tests use Linux IPv4; numeric IPv6 parsing is separate CLI coverage.

**164 of 169 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Five existing private command modules change: args, errors,
main, output and rejoin. New files add the coordinator, process suite/helper and
contract. All earlier test functions and process helpers, public/lower-layer/V1/
A1/F2/storage/consensus/monetary source, Cargo manifests/lockfile, references,
fixtures, workflow commands and 20-minute budget remain unchanged. No wire,
storage or hash encoding, enrollment/device or monetary policy changed.

## Exact next action

**Next: MB-04B4 — bounded persistent-run lifecycle. READY — DESIGN FIRST.**

Design a finite run mode that can repeat acquisition, keep/adopt and service
rounds while retaining one controller, one sticky signal guard and one cumulative
transport allowance. Choose the operator interface during that design; do not
assume another rejoin-* command or change the accepted B3 command.

Before implementation define explicit round and repeated-peer authorization,
bounded inter-round waits, whole-invocation attempt/byte/time limits, first-valid
ordering across rounds, all-peer-failure policy and shutdown at every phase.
The existing private provider consumes/releases the controller, so specify an
ownership-returning round boundary without dropping/reacquiring either root.
Reacquisition pauses serving; no concurrent listener/request loop is implied.
Repeatedly choosing a stale but valid first peer is possible and must not be
described as convergence or global latestness.

Once any round admits adoption, every later fatal error must retain truthful
possible-completion context, even during a later receive or wait. Distinguish
the initial checkpoint, each completed round and the final selected checkpoint;
bound all reports and preserve resource release before output. Verify actual
multiple rounds, whole-state reopening, cumulative exhaustion, ownership and
shutdown before declaring that lifecycle implemented.

B4 is planned only. Indefinite operation, discovery, parallel scheduling, pool,
wallet and policy choices are not silently included. Full MB-04 and Gate A/B
remain unfinished; all monetary, enrollment/device and accessibility decisions
and proposals retain their existing gates.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 31, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [failover](docs/NODE_FAILOVER_V2.md),
[rejoin](docs/NODE_REJOIN_V2.md), [provider](docs/NODE_PROVIDER_V2.md),
[operator](docs/NODE_OPERATOR_V2.md), [control](docs/NODE_CONTROL_V2.md),
[sessions](docs/PEER_SESSION_V2.md), [adoption](docs/CANDIDATE_ADOPTION_V2.md),
[history](docs/PEER_HISTORY_V2.md), [threat model](docs/THREAT_MODEL.md),
[acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md) and [the-prompt](the-prompt).
Preserve all prior storage/recovery/chain/ledger contracts and newer/unpushed work.
