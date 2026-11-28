# Mirror current status — MIRROR-MB04C2-001

Updated **2026-11-28**. Read the active branch; an older main or chat checkpoint
does not override newer verified work.

**MB-04C2 is VERIFIED_COMPONENT — FINITE LOCAL POOL INTAKE AND INSPECTION.**
C2 adds the finite `pool-inspect` command around C1's accepted in-memory pool and
one clean A1 controller. C1 retention/evaluation/reconciliation, B4 finite run
rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown, A2 operator
commands and the existing storage/peer components retain their boundaries.
MB-04 remains IN_PROGRESS: wallet/key custody, pool persistence/gossip and retained
network-loop integration, wider APIs and broader scheduling remain unfinished.
Complete Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain
UNMET.

## Exact source and publication

Implementation source: **39948d40703ee832d528f662c04f6cd5df7d69e4**.
Tested source tree: **733e483ebaa0ecfba8737fd7fe2e1da12d4cd8f1**.
Parent checkpoint: **b488287ab47dba327d6b6d1efbde86213fb3fbfc**.
[Source PR CI 46097203497](https://github.com/Ahmed-Sobeih/mirror/actions/runs/46097203497)
**attempt 1** passed all **43 steps**. Full job `140831260936` logs confirm
**1,386 runtime tests and eleven compile-fail doctests**, zero failures and
three retained parent-exercised ignored helpers. Actual synthetic checkout
`d699c9be6e52114b2de1481e55d3b50f9a05fd4d` has no file differences from the implementation source.
All **12 published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **213 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

[POOL_INSPECT_V2.md](docs/POOL_INSPECT_V2.md) records the design-first contract.

The separate finite `mirror-node-v2 pool-inspect` command requires thirteen
deliberate option/value pairs: the existing roots and independent configuration/
genesis pins, one input path, input byte/count caps, all five C1 pool limits and
an explicit evaluation count. The existing 32-argument, 4096-byte-token and
16384-total-byte ceilings remain. Every argument and scalar relation is checked
before effects; old commands reject C2-only options. No default network or stdin
fallback is introduced.

The file is a bounded local regular file of LF-terminated lowercase hex records
around unchanged signed V2 operation bytes. Empty input is valid; nonempty
records have 2..582 even hexadecimal characters and one LF. Blank/uppercase/CRLF/
odd/nonhex/truncated framing fails before A1 opens. Input caps are independently
1..149248 bytes and 1..256 records; every duplicate counts. Retention, batch and
abstract work retain C1's separate limits. Signed input cannot choose its network.

The input path rejects lexical `..`, stdin-like `-`, symlink parents/leaves,
nonregular objects and multiply linked files. Read-only `O_NOFOLLOW|O_NONBLOCK`
opening, pre/open/post device/inode/type/link/length/mtime/ctime checks and a
fallibly allocated length-plus-one EOF sentinel reject observed changes. The
descriptor closes before A1 opens. Parent namespaces remain trusted: metadata
checks are not hostile-writer snapshot isolation, and nonblocking open prevents
a raced FIFO's writer wait without imposing a regular-filesystem deadline.

One sticky signal guard, one clean `NodeControlV2::open_writer` and one C1 pool
span the invocation. The real immutable accepted chain remains owned throughout
admission/evaluation. A1 still replays and synchronizes; unchanged application
bytes and selected checkpoint do not mean no synchronization. Missing, pending,
corrupt, wrong-network or busy roots retain their existing refusal/recovery rules.

Complete framing precedes sequential FIFO admission. Any file, operation,
capacity or admission-work refusal ends with a static diagnostic, exit 1 and no
normal partial-success result. The ephemeral pool is discarded. Exact duplicates
preserve their retained position and charged identification work, including at
full retention. No failure or retry replenishes the owner allowance.

`--evaluate-count 0` skips evaluation explicitly. A positive count evaluates
exactly that deduplicated FIFO prefix once; a count above retained length is
blocked instead of shortened. Success returns an evaluated count/framed size.
Expected operation/body/ledger/count/byte/work refusal returns `evaluation=blocked`
with exit 1, a closed reason and an available zero-based FIFO ledger index.
Allocation, arithmetic and internal-state failures remain operational diagnostics.
Typed nested membership errors are classified without formatting their text.

One fixed 31-field report names the complete observed checkpoint, input/retention/
duplicate/work counters, every supplied limit and evaluation result. It explicitly
reports `canonical_inclusion=not-checked` and `pool_disposition=discarded`.
The formatter checks scalar, limit, identity, count and evaluation consistency;
normal output and diagnostics remain within 4096 bytes. No raw paths, input
bytes, unbounded operation-ID list or nested error strings are emitted.

Sticky shutdown is checked around bounded reads and successful A1/C1 calls;
already admitted synchronous work finishes or fails before another check. A real
returned error takes precedence over relabeling it as shutdown. The pool,
controller, input bytes and signal guard drop before formatting/stdout. Lost
output retains conservative completion uncertainty and never triggers a retry.

C2 does not reconcile or scan accepted history, retain a pool after exit, sign,
mine, prepare/append/adopt a block, initialize/recover, use a peer or integrate
with B4's run loop. Contextual retention, exact batch executability and canonical
inclusion remain separate. C1/A1 public APIs, lower-layer code/encodings, crate
versions, Cargo.lock, workflow commands and timeout remain unchanged. The sole
dependency access adjustment promotes the existing local mirror-personhood path
dependency for typed nested error classification; no new crate is introduced.

## Acceptance evidence

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed, with all **222 tested source/checkpoint inputs
unchanged during execution**:

- **1,386 workspace/all-target runtime tests**, zero failures: 1,333 retained
  plus **53 new**. Parameterized cases, subprocesses and release reruns are
  not additional tests.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers remain exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **367+1**. All **twelve references** and **sixteen examples** passed.

New executable test inventory:

- `crates/mirror-node/src/bin/mirror-node-v2/args.rs`: **7** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/errors.rs`: **1** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/main.rs`: **1** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/output.rs`: **7** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/pool_input.rs`: **8** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/pool_inspect.rs`: **9** new tests.
- `crates/mirror-node/tests/pool_commands_v2.rs`: **20** new tests.

Focused binary verification first passed all 114 then-registered tests. Source
review then tightened impossible typed-body representation/context/root errors
to operational invariants and added one classifier test, which passed separately;
the final full gates exercise all 115 binary tests. These are 33 new private
tests plus the retained 82-test binary suite.

The first process-suite run passed 19 of 20 functions. Its sole failure was the
test harness's optional Unix-domain socket creation returning EPERM on this host,
before the operator command ran. That complete failed log is retained. Only the
optional socket fixture and unused import were removed; missing-file, directory,
writerless FIFO, symlink-parent/leaf and hardlink checks remain. The final focused
suite passed all 20 functions in debug and release, with no ignored cases. Socket
path rejection is not claimed as observed process evidence. No production
behavior or retained test was weakened to accommodate the host limitation.

The new process suite covers full framing admission before a busy controller;
empty input; malformed operation encodings; exact maximum 149248-byte/256-record
input and distinct count/byte refusals; all four operation kinds; duplicates at
full retention and exact FIFO order; independent retention caps; signature/context
refusal; overspending, gaps/conflicts, funding order and a late mixed-operation
failure; literal cumulative work and body quotas; missing/wrong/busy/corrupt/
unknown/pending A1 state; restart; bounded records; broken stdout; and actual
blocked stdout with input and both authority owners released. Observed SIGKILL
there occurs after computation/resource release, not during admission or staging.

The initial publication `e2c1c68d72380bb4c111d6839119377febed52f2` was not accepted.
Its first full local workspace gate passed all 53 new tests but failed a retained
B2 test before its chosen SIGINT: the helper found that the child had already
exited. That failed path did not retain the child's diagnostic, so the exact
exit class is unknown. The test constructed/transmitted/replayed its candidate
inside a 1000 ms session, then started another 650 ms sleep after observing EOF
receive, without establishing that 650 ms remained. The complete failed gate
and initial publication evidence are preserved separately.

The final source repairs only that test's observation timing. It precomputes
the transcript before launch, anchors the 650 ms offset to completed-request
observation before transmission/replay, sleeps only the remaining offset and
freshly observes the same one-byte EOF receive immediately before SIGINT. The
unchanged 1450 ms elapsed bound uses that earlier epoch; renewing 1000 ms after
the signal would require at least 1650 ms. All three timing constants, the full
post-signal assertions, the other 26 test bodies and all helpers are unchanged.
One exact targeted debug execution passed after the repair. Production B2 and
C2 behavior were not changed. The full final gates and exact-source CI reported
above apply to this repaired source; the earlier publication's CI alone cannot
accept the later repair.

The initial publication's PR CI run `46095907470`, job `140827064527`, ended
cancelled during the workspace gate. Its complete archived log contains all 53
new passing test names and no reported test-failure marker, but not a complete
workflow. That partial run is not source acceptance. The exact final-source run
identified above is verified independently from its full log and actual checkout.

New process tests use actual accepted synthetic chains, real signed operations,
separate roots and bounded child cleanup. Full canonical state, selected
checkpoint, control/store/input bytes and strict reopening are checked. The
observed full-pipe stdout write establishes release of both authority owners and
the input descriptor before output. This evidence is local and finite; it does
not establish production network operation or a wall-clock bound on filesystem I/O.

Private deterministic checks cover bounded framing/read behavior and the closed
error/report boundary. Static unsafe-file cases are actual process evidence.
Concurrent hostile namespace mutation, induced allocator exhaustion and signals
at every internal synchronous validation/synchronization phase are not claimed
as observed process evidence. A1 replay/synchronization retains its accepted
contract; equal file contents do not establish absence of synchronization.

Preservation audit: **71 of 72 existing test/helper files** and
**67 pre-existing private test functions** in changed binary
modules are byte-identical. The exception is the single retained B2 deadline
observer described in the evidence; all **26 other test bodies**, the rest of that
file and its complete post-signal assertions are unchanged. All lower-layer
production source, C1/A1 public APIs,
frozen vectors, reference tools, monetary policy and runtime-data boundaries
remain unchanged. The existing mirror-personhood path dependency is promoted to
normal dependency access for typed nested error classification; no new crate or
version is added, and **Cargo.lock is unchanged**. The existing workflow commands,
pinned toolchain and 20-minute job limit are unchanged. The new contract was
written before implementation, with the dependency-access refinement recorded
before coordinator implementation. No monetary, enrollment, privacy, device or
accessibility choice was silently changed.

## Exact next action

**Next: MB-04D1 — local wallet and signing boundary. READY — DESIGN FIRST.**

C2 now consumes already signed operations; the next useful MB-04 increment is
an explicit local wallet boundary that can eventually supply those inputs. Begin
with repository inspection and a written threat model/API/file-lifetime design,
then select the smallest reviewable implementation slice. D1 is not implemented
by this checkpoint, and choosing this next slice does not mark broader pool or
node integration complete.

Specify fresh operating-system randomness, key ownership and separate signing,
transport and membership roles; independently selected configuration/genesis;
the exact operation fields and user intent authorized by signing; and a truthful
distinction between constructing/signing bytes, pool admission, ledger execution
and canonical inclusion. Do not let an untrusted input choose its signer, network,
recipient, amount, sequence, anchor or authorization window silently.

Choose deliberate local secret input/output, authenticated protected storage,
file permissions/identity, bounded decoding/KDF work, backup/export/recovery,
cancellation and error semantics before implementation. Review established
cryptographic libraries and official documentation if new dependency support is
needed; pin exact reviewed versions and do not invent encryption or KDF schemes.
Document the limits of memory clearing, filesystem persistence and same-user
trust honestly. Keep real private keys, passwords, recovery seeds and biometric data
out of argv, logs, fixtures, repository files and diagnostics. Tests use fresh temporary
or explicitly synthetic public development material only; known demo keys are
not real wallets.

Keep signing local and explicitly authorized. Do not add automatic mining,
append/adoption, transaction gossip, pool persistence, a remote submission/signer
endpoint, default key selection or B4 loop integration as incidental work.
Existing canonical operation encodings, no-fee/no-mint development behavior,
monetary decisions, enrollment profiles and open privacy/accessibility/consensus
gates remain unchanged. Design separately any later persistence/gossip or retained
pool lifecycle across chain changes; C2 still discards its owner after each call.

Retain all existing gates and test bodies. Verify exact implementation source,
full local/CI evidence and published readbacks before the later nine-document
checkpoint; verify checkpoint-head CI separately and refresh `the-prompt`.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 34, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [pool inspection](docs/POOL_INSPECT_V2.md),
[pending pool](docs/PENDING_POOL_V2.md), [run lifecycle](docs/NODE_RUN_V2.md),
[failover](docs/NODE_FAILOVER_V2.md), [rejoin](docs/NODE_REJOIN_V2.md),
[provider](docs/NODE_PROVIDER_V2.md), [operator](docs/NODE_OPERATOR_V2.md),
[control](docs/NODE_CONTROL_V2.md), [sessions](docs/PEER_SESSION_V2.md),
[ledger](docs/LEDGER_STATE_V2.md), [chain](docs/CHAIN_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Recheck live refs and preserve newer/unpushed work
and runtime data.
