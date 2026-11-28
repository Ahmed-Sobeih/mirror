# Mirror current status — MIRROR-MB04C1-001

Updated **2026-11-28**. Read the active branch; an older main or chat checkpoint
does not override newer verified work.

**MB-04C1 is VERIFIED_COMPONENT — BOUNDED LOCAL PENDING-OPERATION POOL.**
C1 adds an in-memory library component for context-valid signed retention,
one atomic ordered batch evaluation and explicit selected-parent reconciliation.
B4's finite run rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown,
A2 operator commands, A1 durable control/recovery and MB-03's storage/peer components
retain their accepted boundaries. MB-04 remains IN_PROGRESS: finite operator pool
intake, persistence/gossip integration, wallet, wider APIs and broader scheduling
remain unfinished. Complete Ant Kiss remains MB-05 and final human consensus
MB-06. Gate A/B remain UNMET.

## Exact source and publication

Implementation source: **94207e713dd89f4d33586263c510591be0e78201**.
Tested source tree: **9c40a4852c8ab935cd7318d33d81d0720cd0dd15**.
Parent checkpoint: **35bbd0817843c35a42ff8c09b779391df331d53c**.
[Source PR CI 46086769527](https://github.com/Ahmed-Sobeih/mirror/actions/runs/46086769527)
**attempt 2** passed all **43 steps**. Full job `140804733638` logs confirm **1,333 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`17ece65483bd57dbfb3c6e11c64895ddf0afbbe2` has no file differences from the implementation source.
All **seven published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **208 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

[PENDING_POOL_V2.md](docs/PENDING_POOL_V2.md) was written before implementation.

The new `mirror_node::v2::pool` component borrows an actual accepted `ChainV2`.
Its private binding contains configuration/genesis, height, tip and accepted ledger
root. Five deliberate local limits cap retained entries (at most 256), encoded
operation bytes (74,496), one batch's operations (64), complete framed block bytes
(19,048), and cumulative abstract work (67,108,864). Individual input is capped at
291 bytes before decoding/ID/signature work. Canonical membership `max_pending`
remains unrelated, and canonical operation/body limits apply independently.

Admission decodes the exact existing signed operation, deduplicates its complete
signed ID against retained entries and validates it for the selected next block.
Stable FIFO retention permits context-valid overspends, sequence gaps, competing
sequences and unknown membership targets. An exact duplicate preserves its position
even at full capacity; a distinct full-pool input never evicts another entry.
Admission does not scan history, so an already-included but still context-valid
input may be retained until evaluation/reconciliation. Future windows and invalid
expiry/anchor/signature/configuration reject. No fee replacement is invented.

Evaluation chooses exactly one requested nonempty FIFO prefix. Local count/framed-
byte and canonical complete-body preflight precede copying or execution. One full
`LedgerV2::stage_operations` call preserves original mixed order and atomic accounts/
membership semantics, including separate account and member sequence domains.
The staged ledger is discarded even on success; only immutable ordered input and
parent/revision observations are returned. Failures retain original indexed ledger
errors and leave all pool entries and accepted chain state unchanged.

Explicit reconciliation handles the same parent, append or accepted replacement
within the same configuration/genesis. It reserves complete work before history
ID hashing, contextual checks or temporary allocation, scans accepted history
once. Each input's removal reason prefers exact inclusion over contextual
invalidity; removal reports and survivors keep FIFO order. Context-valid stateful
blockers remain. Parent,
entries, exact bytes and revision publish together; budget/allocation/arithmetic
failure preserves the prior binding and semantic entries. Removed inputs are not
automatically resurrected after a replacement.

Work is cumulative for one non-Clone owner. Identification charges failed/duplicate
attempts, each evaluation charges the current parent-state size again, and every
reconciliation charges accepted history plus bounded retained-ID comparisons and
revalidation. No failure, clear, removal or parent change refunds work. The meter
is an explicit abstract budget, not CPU/RSS/wall-clock or a global rate guarantee.
Every successful reconciliation and actual entry mutation invalidates earlier
batch observations. Currentness checks the real chain, full parent, revision and
exact prefix; a different pool's different entries cannot match on revision alone.

The component never returns/binds staged state, appends/mines/prepares blocks,
signs, mutates A1/store, persists, gossips, exposes submission CLI/API ingress or
assumes wallet custody. Existing commands do not create a pool automatically.
Retained input, contextual validity, evaluated execution and canonical inclusion
remain separate. Actual block acceptance still checks complete blocks, timestamps,
work, roots and cumulative chain budgets. Zero-fee/no-mint development semantics
and all monetary, enrollment, privacy, device and accessibility decisions remain.

## Acceptance evidence

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed, with all **217 tested source/checkpoint inputs
unchanged during execution**:

- **1,333 workspace/all-target runtime tests**, zero failures: 1,281 retained plus
  **52 new** — 31 admission/evaluation and 21 reconciliation/work integration tests.
  Parameterized cases, subprocesses and release reruns are not additional tests.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers remain exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **314+1**. All **twelve references** and **sixteen examples** passed.
- Both new focused suites passed on their first execution: **31 and 21** tests,
  with no execution-driven production/test corrections. Scoped formatting,
  whitespace and allocation/runtime-data checks passed.

The first full local workspace attempt and one unchanged targeted retry failed in
the retained B4 test's inter-round observer: it waited for requester durability
before looking for a 200 ms phase. Both captured runner reports completed two
legal rounds successfully. The narrowly repaired test starts the requester
concurrently, observes the real listener-free wait, verifies an owned SIGSTOP,
retains the owner/busy checks, awaits requester completion and resumes before the
second request. Original timing values, two-request outcomes and restart/state
assertions remain. The repaired targeted case and all required gates passed on
the final source. Failed attempt evidence is retained; production lifecycle and
C1 implementation/test semantics were not changed to suppress the failure.

The final source's first hosted CI attempt passed every test/release gate, then
was cancelled during the examples just after the configured 20-minute job limit.
Its log records only a generic cancellation; timing supports a budget-boundary
explanation but does not establish a more specific cause. That incomplete attempt
is preserved and not accepted. The unchanged source/workflow was rerun; the
successful complete attempt is identified in the source record above. The
20-minute workflow limit and all verification commands remain unchanged.

The new suites construct actual mined, manifest-pinned accepted chains with public
synthetic keys. They cover all four operation kinds, exact signed bytes/IDs,
malformed/signature/context refusals, duplicates at full capacity, actual 256-entry
and 64-operation limits, independent local/canonical quotas, framed batch bytes,
FIFO order/removal, retained sequence gaps/conflicts, receive-before-spend,
aggregate overspending, independent account/member sequences and mixed late failure.

Reconciliation evidence covers exact included IDs from earlier history, unchanged-
parent inclusion, included-before-expiry reporting, survivor order, same-height and
shorter replacements, changed/old anchors, newly future windows, wrong configuration
and distinct accepted genesis, no resurrection and stale/cross-pool batches.
Literal boundary checks verify successful/failed state-work charges, historical
scan/revalidation charges, partial admission reservations, sticky exhaustion,
atomic failed reconciliation and no refund from removal/clear/parent changes.

Complete chain equality and exact retained-state observations establish this
in-memory boundary. New C1 tests do not add command/process, socket, persistence,
wallet or power-loss evidence; the existing required process/recovery suites are
retained and rerun. Fallible bounded pool allocation paths are reviewed but induced
OOM is not tested. Abstract work units are not CPU-time, RSS or global DoS limits.
No runtime behavior is inferred merely from the written contract.

**210 of 212 existing repository files are byte-identical to the B4
checkpoint before the later nine-document handoff.** The sole existing Rust change
in production is one `pub mod pool;` export in the V2 node module; removing that
line reproduces its entire earlier body byte-for-byte. **68 of 69 retained
test/helper files** and **173 of 175 retained source/dependency/workflow/reference/
fixture paths** remain byte-identical. The sole retained test exception is the
documented B4 observation-coordination repair; all other 18 test bodies and every
byte outside that function in its file are preserved. Five new files add the pool,
two suites, an isolated synthetic fixture and the contract. Existing command implementations, A1/F2/storage,
lower-layer/V1/consensus/monetary bodies, Cargo manifests/lockfile, workflow commands
and its 20-minute budget, reference tools and frozen vectors are unchanged.
No wire, storage, hash, monetary, enrollment, device or accessibility policy changed.

## Exact next action

**Next: MB-04C2 — finite local pool intake and inspection. READY — DESIGN FIRST.**

Design the smallest explicit operator path that can feed already signed V2 inputs
into C1 and inspect one local result against a clean accepted A1 chain. Specify the
command, deliberate input source/framing and finite invocation lifetime before
implementation. Preserve C1's exact-parent binding, contextual-only admission,
FIFO batch semantics, independent limits and cumulative work; never treat a pool
receipt as accepted chain state or permission to publish a block.
Require independently selected configuration/genesis identity; signed input must
never select its own network or accepted parent.

Define argument, input-file/stream, item-count and total-byte ceilings before any
input-sized read/copy, and a closed policy for malformed framing, duplicates,
individual rejections, budget exhaustion and a blocked requested batch. Choose
all-or-nothing intake versus bounded per-item reporting explicitly; do not let a
decoder or reporting loop silently reset C1's owner allowance. Keep retained,
contextually admitted, evaluated and canonically included observations distinct.
Choose bounded regular-file intake with explicit file-type/symlink/EOF rules, or
specify absolute deadline, cancellation and terminal-EOF behavior for streams.
Byte and item limits alone do not bound a stalled pipe, FIFO or stdin read.

Define clean controller ownership and selected checkpoint capture, shutdown/error
semantics, bounded output and release-before-output behavior. Any A1 open/replay/
synchronization must retain its existing contract. Report the exact observed
parent and finite local outcome; do not imply durable pool retention across exit,
network propagation, inclusion or currentness after owner release. Test actual
operator processes, malformed/overlong input, independent caps, duplicate/order
handling, contextual/execution errors, output failure and unchanged canonical
history/control selections before claiming operator integration.
Opening the A1 writer to borrow its accepted chain includes ownership, replay and
synchronization; unchanged canonical selection does not turn it into the existing
read-only, no-synchronization `status` path.

No wallet signing/key custody, automatic mining/append, new monetary rule,
transaction gossip, pool persistence, remote submission endpoint or integration
into B4's retained network loop is implicitly authorized by this narrow step.
Those require separately specified increments. Keep existing commands, A1/F2/store
authority, consensus encodings and all policy gates unchanged. C2 is planned only;
broader scheduling, wallet, wider APIs, full MB-04 and Gate A/B remain unfinished.

Run every existing AGENTS.md gate, verify published source and exact-source CI,
then check the later checkpoint head separately and refresh this handoff.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 33, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [pending pool](docs/PENDING_POOL_V2.md),
[run lifecycle](docs/NODE_RUN_V2.md), [failover](docs/NODE_FAILOVER_V2.md),
[rejoin](docs/NODE_REJOIN_V2.md), [provider](docs/NODE_PROVIDER_V2.md),
[operator](docs/NODE_OPERATOR_V2.md), [control](docs/NODE_CONTROL_V2.md),
[sessions](docs/PEER_SESSION_V2.md), [ledger](docs/LEDGER_STATE_V2.md),
[chain](docs/CHAIN_V2.md), [threat model](docs/THREAT_MODEL.md),
[acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md) and [the-prompt](the-prompt).
Recheck live refs and preserve newer/unpushed work and runtime data.
