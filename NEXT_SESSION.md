# Next session — MB-03C

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-20**.

Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/STORAGE_APPEND_V2.md,
docs/STORAGE_V2.md, docs/CHAIN_V2.md, docs/LEDGER_STATE_V2.md,
docs/CORE_V2_ENCODING.md, docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md,
docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/THREAT_MODEL.md, docs/MEMBERSHIP_TEST_MATRIX.md and the latest the-prompt.

## Recover live state

Verify the default and active branches, PR #1, recent commits and comments, and
CI results for the exact published commits. The active branch is
`work/personhood-register-v0.1`; PR #1 remains open and unmerged. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`. Read the feature-branch records, preserve
concurrent or unpushed changes and all runtime data, and never reset to a copied
checkpoint or merge simply to publish the handoff.

MB-03B source: `ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c`.
Tree: `06d7c3f4fa24d96c844e2301c76c01f804151192`, identical to the locally checked source tree.
Push CI: **44649111203**; job: **135999222947**.
PR CI: **44649119266**.
Exact-source verification status: **SUCCESS**. Both the
[push run](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203)
and [PR run](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
passed. Full push-job logs were inspected and confirmed the exact source
checkout, 658 workspace tests, three doctests, 145 storage release tests,
all eight references, eight examples and the allocation/runtime-data audit.

Check subsequent documentation-head CI separately. Historical MB-03A acceptance,
including its test fork-inheritance correction, remains in master-plan section 18.
Section 19 records MB-03B. No production retry or weakening of locks was introduced.

Local locked Rust 1.98.0 gates passed: **658 workspace tests** (588 retained and
70 new), one intentionally ignored subprocess helper, and three retained compile-fail
doctests. Release checks passed for core (121), personhood (234 plus the same three
doctests), state (49), consensus (23), chain (57), and storage (145). All eight
independent references and eight examples passed. The 70 additions comprise
28 selector tests, 31 filesystem integration tests, eight private failure tests,
and three real process tests. Parameterized scenarios and release reruns are not
additional test cases. Dependencies, prior wire encodings and prior fixtures are
unchanged; source-preservation and focused-formatting checks passed.

**MB-03B VERIFIED_COMPONENT: bounded append and clean-state reconciliation.**
**MB-03 remains IN PROGRESS. Next: MB-03C explicit residual-staging recovery — READY.**
**Gate A and Gate B remain UNMET.**

## Reuse the completed components

`StoreV2` is the immutable profile 1 API, using the unchanged 232-byte MSS2 record
at generation zero. `AppendStoreV2` is a separate, explicit profile 2 API; no
implicit migration exists. Initialize it only in a new or empty root from a fully
accepted `ChainV2`. MNM2 manifest and MRB2 block bytes remain unchanged.

The profile 2 `selection.v2` contains an MSU2 record of exactly 280 bytes: magic,
version 2, profile 2, generation, an unchanged 232-byte MSS2 snapshot, and a SHA256d
checksum with the `MIRROR_STORAGE_APPEND_SELECTION_V2` domain. The outer generation
starts at zero even when initialization uses a chain at a nonzero height. Each
selected child advances generation and height by one, and generation never exceeds
height. This is local publication history, not global freshness, reward weight or
chain finality.

Every open, inspection and reconciliation requires independently retained network
and complete chain-plus-generation selection. Never derive those pins from the
untrusted selector or peer being checked. Full MRB2 replay reconstructs accounts,
membership, permanent records, population, approvals and resource counters together.

Append first validates a cloned candidate through `ChainV2`. It then fully replays
the old disk selection under the retained lock and compares that complete state
with memory. Candidate-validation errors leave files and memory unchanged and allow
a later valid append. A disk mismatch fences the handle. After creating and
synchronizing the new block and the blocks directory, append creates and synchronizes
`selection.pending`, compares the old marker again, renames the pending marker to
`selection.v2`, and synchronizes the root. Memory advances only after success.

An uncertain write fences the live handle. The `chain`, `selection` and `checkpoint`
getters return `Fenced`; `last_confirmed_checkpoint` describes only the previously
successful in-memory state. `UpdateIncomplete` and `PublicationUncertain` carry the
locally validated candidate checkpoint for the caller to retain. Consuming
`reconcile` keeps the same lock and returns an unfenced writer only after full
selected replay and file/directory resynchronization. Writer opening does the
same. Inspection holds a shared lock and performs no write, synchronization or
repair. Consuming reconciliation returns no handle on failure.

A clean selected layout contains only `manifest.v2`, `blocks/` and `selection.v2`.
Current opening, inspection and reconciliation reject and preserve pending markers,
extra tail blocks, partial files and unknown entries. This is the precise gap for
MB-03C. Do not describe it as completed automatic recovery or discard those files
to make tests pass.

## Exact next slice — MB-03C interrupted-append staging recovery

Start by specifying and reviewing the existing append failure states, then implement
a small, bounded recovery operation. This is engineering work within the approved
route, not an economic or real-biometric policy gate.

1. **Freeze recovery authority and format boundaries.** Preserve MSS2 profile 1,
   MSU2 profile 2 and their strict normal decoders. Determine which existing residual
   states can be inspected safely using independently retained old and candidate
   checkpoints, and whether future writes need a separately versioned intent or
   journal. Do not retrofit authority into arbitrary older pending files. Any new
   format requires exact framing, bounds, a hash domain and independent reference
   vectors.
2. **Define explicit recoverable choices.** Specify when a complete candidate may
   be published and when the old selection may be retained. A pending checksum
   cannot choose a branch. Establish the provenance of every managed artifact before
   modifying it. Preserve and reject unknown, foreign, ambiguous or corrupt artifacts.
   State which cases require operator action and which may complete mechanically.
3. **Keep stable ownership and bounded work.** Retain the root directory inode and
   exclusive kernel lock throughout inspection and recovery. Never swap or delete
   the root, infer ownership from PIDs, or permit a second writer. Bound residual
   filenames, counts, lengths and the combined recovery read budget before allocation.
   Full `ChainV2` replay remains the validity authority; stored caches or counters
   cannot replace it or enlarge its resource limits.
4. **Freeze publication and retry semantics before coding.** Document file/directory
   synchronization order, comparison before publication, success versus uncertainty,
   live-handle fencing, restart behavior and idempotence after another interruption.
   A visible, replayable file is not evidence that a failed synchronization completed.
   Normal opening must not acquire undisclosed repair behavior.
5. **Test actual failure shapes.** Cover partial and complete new blocks; absent,
   partial and complete pending selectors; old and new selected markers; incorrect
   external checkpoints; stale generations; unexpected files; damaged, hardlinked
   or symlinked data; missing blocks; mutation during inspection; and repeated
   interruption during recovery. Require a complete old or new ledger, or explicit
   rejection, never mixed account and membership state. Preserve earlier tests and
   isolation of process-fork tests.
6. **Publish exact evidence.** Run all required gates and CI for the exact source.
   Update this file, the master plan, current status, decisions and the-prompt.
   Distinguish implemented recovery cases from those still rejected. Recovery alone
   does not complete MB-03.

Afterward, specify canonical development fork choice before durable branch replacement.
Test genuine membership-divergent reorganization and fresh-peer reconstruction.
Orphaned approvals, population peaks and counters must not survive as selected state.
MB-04 remains the persistent V2 node, wallet, pool and operator API; final human
consensus belongs to MB-06. Current storage work requires no device purchase,
biometric collection or monetary policy decision.

## Invariants and gates

Preserve original complete mixed-operation indices, private accepted ancestry,
strict child timestamps greater than the parent, configured literal difficulty,
signature and ledger-root checks, zero/spent account retention, membership expiry
and activation, and permanent identity history. Do not introduce automatic pruning,
root setters or conversion of untrusted caches into runtime state.

Chain caps remain 4,096 blocks, 16 MiB of complete MRB2 bytes, and 1,048,576 cumulative
abstract state/configuration items. Profile 2's selected encoded-input maximum is
19,559,132 bytes; this is not a total resident-memory limit. Residual-recovery budgets
must be specified separately. Linux cooperating processes and trusted local owners
and parent directories remain explicit assumptions. Kernel ownership ends when all
descriptors for the same open file description close, including inherited fork
descriptors. Linux CI does not establish WSL, DrvFS or network-filesystem behavior.

Required checks:

- `cargo test --workspace --all-targets --locked`
- `cargo test --workspace --doc --locked`
- Release tests for core, personhood, state, consensus, chain and storage.
- All eight `tools/reference_*_v2.py --check` scripts.
- `registry_demo`, `operation_v2_demo`, `state_v2_demo`, `block_v2_demo`,
  `ledger_v2_demo`, `chain_v2_demo`, `storage_v2_demo` and `append_v2_demo`.
- Focused formatting and diff checks, dependency/fixture/runtime preservation,
  and the allocation audit.

Record exact commits, trees and results. An older green run does not verify a newer
source commit.

No founder bonus, privileged marketing reserve, mint/grant API, combined bootstrap
formula, nonzero fee/reward or actual biometric input is introduced. D-01 and D-04
remain open. One MRY equals 10,000,000 Nusa. The first 1,000 qualifying humans,
including the founder, share the same 10-MRY bootstrap policy; the ordinary 0.2-MRY
allowance uses the qualifying historical population maximum. Founder-led startup
creates no permanent administrative right. Fingerprints and ECG are off-chain
instruments. Raw or encrypted biometric archives never enter GitHub or on-chain
data. Signatures and exact evidence keys do not prove biological uniqueness or
anonymity.
