# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB04B1-001 — finite V2 provider and draining shutdown**.
Updated **2026-11-27**.

**MB-04B1 is VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN.**
MB-04A2 remains verified for finite operator commands, A1 for durable local control
and explicit restart recovery, and MB-03 for its finite storage/recovery/peer scope.
MB-04 remains IN_PROGRESS: rejoin-and-serve coordination, broader reconnect and
multi-peer scheduling, transaction pool, wallet and wider operator APIs remain.
Complete Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain UNMET.

## Exact source and publication

Implementation source: **ed631eae05ebeebe2b3a58a433ab8a0cc5753150**.
Tested source tree: **dc32d623bf6a8e5e3ab03023b699a729e0c630a1**.
Parent checkpoint: **f507827aabcaecda85c02d66f0de6a10db8972ac**.
[Source PR CI 45904484131](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45904484131)
passed all **43 steps**. Full job `140197823058` logs were inspected and confirm
**1,167 runtime tests and eleven compile-fail doctests**, with zero failures.
Its actual synthetic PR checkout `cdb00671509b4f2876d51bd677ab39d861833396` has no file
differences from the implementation commit. All **17 published implementation,
test, contract, dependency and workflow files** were read back byte-for-byte against
locally tested content. Source acceptance: **PASSED — local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; source
acceptance does not assert that a future head is tested. PR #1 remains OPEN and
UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

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

## Acceptance evidence

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,167 workspace/all-target runtime tests**, zero failures: 1,137 retained
  plus **30 new** (six parser, nine accepted-stream adapter and fifteen
  actual provider-process tests). Parameterized cases, subprocesses and release
  reruns are not extra unique tests.
- **Eleven compile-fail doctests**, unchanged from A2. Three retained subprocess
  helpers remain intentionally ignored in ordinary enumeration and are exercised
  by their parent tests.
- Required release suites: personhood 234 runtime + 3 doctests, core 121, state 49,
  consensus 23, chain 85 + 3, storage 350 + 2, protocol 38,
  network **108 + 2**, node **148 + 1**.
- All **twelve independent Python references** and **sixteen synthetic examples**;
  scoped formatting, whitespace and allocation/runtime-data checks also passed.

The new process tests establish refusal before bind; complete fixed-history
responses across clients; exact successful/rejected counts and cumulative bytes;
malformed/wrong-network/short/trailing/missing-EOF handling; idle and active
SIGINT/SIGTERM; unchanged in-flight deadlines; admission expiry and final drain;
competing commands busy with zero outbound sync contact; forced termination;
strict fresh reopening; and closed or blocked final output.

The blocked-output case fills a CLOEXEC pipe before launch, observes the child
actually blocked writing fd 1, then proves its listener is gone and a new status
process succeeds **while the provider child is still blocked**. Only afterward does
the parent close the reader and observe the output error. This verifies release
before output, rather than only release after process exit.

Admission/readiness evidence ties bounded Linux `/proc` socket-inode/descriptor
and blocked-syscall observations to the specific child and connection, including
the EOF probe. It uses no production readiness hook, ptrace or permission change.
Parents use bounded waits, byte/descriptor caps and kill/reap cleanup. The receive
helper has one original five-second deadline; direct store ownership requires the
exact wrapped Busy error. Unsupported observation fails visibly; these process
results apply to the tested Linux environment, not every OS/filesystem.

**153 of 161 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Eight deliberate changes are the workflow, lockfile, F2 session
module, node manifest and four existing V2 command modules. All earlier tests,
references, frozen fixtures, V1/A1/storage/consensus and monetary code remain.
The node adds a direct Linux-target edge to already locked `libc` 0.2.189; no package
version, source or checksum changes. CI timeout rises from 15 to 20 minutes because
the prior run took about 14 minutes 30 seconds and this slice adds process tests
in debug and release. Every prior verification command/assertion remains required.
No wire/hash/storage encoding, enrollment/device policy or runtime data changed.

## Exact continuation

**Next: MB-04B2 — finite rejoin and serve coordination. READY — DESIGN FIRST.**

Specify one deliberate lifecycle before implementation:

1. Require explicit control/store paths, independent C/genesis and one chosen
   numeric incoming peer. Strictly open A1, receive at most one complete F2
   candidate, then apply the existing consuming keep/adopt rules.
2. Retain the returned ready controller and serve its resulting fixed complete
   history. Do not release/reacquire ownership between adoption and serving or
   accept another branch while serving. Define listener startup and failure order.
3. Use one cumulative F2 attempt/actual-byte budget across reception and serving.
   Neither phase nor a new owner may replenish it. Specify how the incoming attempt
   affects remaining service capacity; keep all failed charges and original
   deadlines. There is no concurrent receive/serve or automatic reconnect.
4. Specify shutdown before reception, after validated receipt but before adoption,
   during durable adoption and during serving. Durable publication is not
   asynchronously cancellable. A later bind or output failure must truthfully
   report that adoption may already have completed and direct explicit inspection.
5. Verify three independent processes/directories: seed provider -> rejoining node
   -> third requester. Include explicit clean restart/rejoin and complete accounts,
   membership, approvals and population agreement, plus pending-state refusal,
   ownership, cumulative limits and selected-candidate no-rollback.

B2 is planned work, not supplied by B1. Automatic initialization/recovery, peer
discovery, multi-peer scheduling, pool, wallet and unresolved monetary/enrollment/
device choices remain outside this increment. Full MB-04 and Gate A/B remain open.
No policy decision blocks this dependency-ready design.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies. Preserve twelve
references, sixteen examples and all required release suites. Record exact source
acceptance separately from later documentation-head CI and refresh the handoff.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 29, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [provider lifecycle](docs/NODE_PROVIDER_V2.md),
[operator commands](docs/NODE_OPERATOR_V2.md), [node control](docs/NODE_CONTROL_V2.md),
[peer sessions](docs/PEER_SESSION_V2.md), [adoption](docs/CANDIDATE_ADOPTION_V2.md),
[history reconstruction](docs/PEER_HISTORY_V2.md),
[replacement storage](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[explicit storage recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Recheck live refs and preserve newer/unpushed work.
