# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03C-001 — explicit interrupted-append recovery**.
Updated **2026-11-20**. MB-01A, MB-01B/B1–B4, MB-02A/B/C and MB-03A/B remain
verified components. **MB-03C: VERIFIED_COMPONENT — EXPLICIT ONE-CHILD RECOVERY.**
**MB-03 remains IN PROGRESS. Next: MB-03D — canonical development fork choice.**

## Exact source and publication

Active branch: work/personhood-register-v0.1.
[PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1) is open and unmerged.
Main remains ad7ffd120fb451fe44cf59cb909176785b33bf8f.

Published source: **e8302052ab724f59ff4a212dc64aea9575a6d4b3**.
Tested tree: **dc9eb2c02436f0952ac54593ba8d02317e9d9314**, identical to the locally checked tree.
Source increment: **9 files, 3,138 insertions and 1 deletion**.
[Source push CI 44748718086](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748718086)
and [source PR CI 44748724632](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748724632).
Both runs **SUCCEEDED**. The full push-job log, job **136330677456**, was inspected:
its checkout is exactly the published source above. Workspace/doctest/release counts,
all eight references, all nine examples and the allocation/runtime-data audit passed.

The preceding documentation checkpoint 6c53f188ef7d5052f96577ed04f5f909f56966c1
passed push CI 44650139143 (full job 136002692765 inspected) and PR CI 44650145750.
Current documentation follows the new implementation. Its exact-head CI is checked
separately and recorded in PR #1; recheck that live head when continuing. Never
confuse an older successful run with a later head.

## What this increment implements

The [recovery contract](docs/STORAGE_RECOVERY_V2.md) adds two explicit operations
to AppendStoreV2: recover_interrupted acquires exclusive ownership of an existing
store; consuming recover retains the same writer lock, including after an append
has fenced that writer. Neither operation initializes or adopts a foreign store.

**Independent intent.** AppendRecoveryRequestV2 contains independently retained
parent and candidate checkpoints plus the exact candidate BlockV2. The checkpoints
must be adjacent in both height and local generation. Recovery replays every parent
block, accepts the child through normal ChainV2 rules, and derives exact old/new
MSU2 selectors before changing data. Complete candidate validation also applies
when retaining the parent.

**Two explicit actions.** Complete fills only a missing canonical suffix of the
authorized child or pending selector, synchronizes preparation, publishes the
candidate and fully replays/resynchronizes the result. RetainParent is allowed
only while the parent remains selected; it removes only matching authorized
pending/tail files, synchronizes removal in the specified order, then replays
and resynchronizes the parent. A selected candidate is never rolled back.

**Narrow supported states.** Parent selection permits no staging, an empty/partial/
complete child with no pending marker, or a complete child with an empty/partial/
complete intended pending selector. Candidate selection requires its exact complete
child and no pending marker, and allows idempotent Complete only. A pending marker
with a missing/partial child is rejected. Missing/corrupt selected history, unknown
files, mismatched bytes and unsupported layouts remain preserved and rejected.

**Caller authority is explicit.** Even an empty or common prefix cannot prove who
created a file. Prefix matching constrains bytes; the external request authorizes
only these named staging paths. The component does not store that request durably
or choose it from a peer, checksum, pending marker or highest generation.

**Ordering and ownership.** The validated parent history and marker are synchronized
before mutation. Completion checks exact bytes and safe current file identity
before synchronization and rechecks preparation before publication. Retention
removes pending first and synchronizes the root before removing the tail and
synchronizing blocks/. The root inode stays stable. Every error returns no usable
handle; uncertain publication is distinct from incomplete cleanup/preparation.
Final success requires strict clean loading, complete ledger/counter equality and
file/directory synchronization.

Normal open_writer, inspect and clean-state reconcile remain strict. Immutable
StoreV2 and all MNM2/MRB2/MSS2/MSU2 bytes, domains and existing fixtures are unchanged.
There is no new journal, storage profile, automatic repair or branch-selection rule.

## Observed local verification

| Check | Result |
|---|---|
| Workspace/all-target tests | **707 passed = 658 retained + 49 new** |
| New cases | **39 recovery integration + 9 private interruption + 1 process test** |
| Doctests | **3 retained compile-fail cases passed** |
| Subprocess helper | **1 intentionally ignored entry**, invoked by parent tests |
| Release core/personhood | **121 / 234 passed**, plus the same 3 doctests |
| Release state/consensus/chain | **49 / 23 / 57 passed** |
| Release storage | **194 passed**, with the same 1 helper ignored |
| Independent references | **All 8 passed**; no new encoding required a ninth |
| Synthetic examples | **All 9 passed**, including recovery_v2_demo |
| Focused formatting, diff, dependencies | **Passed** |
| Preservation | **99 retained source/dependency/reference/fixture files identical** |

The nine private tests include 28 completion boundaries across three initial shapes
(84 scenarios), 21 retention boundaries, 16 already-selected reconciliation
boundaries, and all 11 original append boundaries with both actions. Other cases
check mutations before writing/deleting/publishing, root replacement, new aliases
after opening and sentinel budget exhaustion. These are scenarios within nine
tests; release repeats and examples are not extra unique tests.

The new process parent test has two release scenarios: child handle drop while
the child stays alive, and child termination. The child performs recovery and
holds its returned writer; parent recovery/writer/inspection calls require Busy.
After release, the parent checks the selected state and stable root, then appends
and reopens the next actual block. The same helper and test isolation are retained.

The new example completes a partial child, retries Complete without another
generation, retains the parent from a fenced writer with matching pending staging,
and performs an ordinary next append and reopening. Final tip:
36b8ccbfdaa75ae33f3e522f7f636ddf02ff2c6b4c23df4ea694d207ada01c6b.
Final ledger root:
3f0c9f7ef212a7cae71717004d7a0206f027109490be8dcd07cba032ce32f86e.
Only synthetic fixtures and invocation-owned temporary directories are used.

## Limits and next task

Recognized recovery inputs are bounded to **19,559,412 encoded bytes**, including
the full proposed child and both selectors. A shared conservative allowance charges
every bounded payload read and sentinel, including repeated comparisons/final replay:
**156,508,096 bytes per call**. Neither number is a resident-memory limit.
The unchanged chain caps are 4,096 blocks, 16 MiB of MRB2 and 1,048,576 cumulative
state/configuration items. No history pruning or stored-cache authority is added.

**Next MB-03D:** specify and implement canonical development fork comparison over
already accepted V2 branches, with independent network identity, exact work/score
and tie rules, without conflating storage generation, population or hash luck with
branch validity. Durable branch replacement and membership-aware peer reconstruction
follow. Storage recovery alone does not complete MB-03 or the persistent V2 node.

No recovery is promised for interrupted initialization, missing/corrupt selection,
damage to selected blocks or lost independent request authority. Cooperative Linux
processes, trusted local owners/parents and correct locking/rename/synchronization
remain assumptions. WSL2/DrvFS, native Windows and network mounts are not established
by Ubuntu CI. The tests do not simulate physical power loss or certify disk firmware.
Two source/contract reviews found no blocking discrepancy within that scope.

Accepted monetary/admission/privacy policies remain unchanged: no founder bonus or
marketing reserve; equal first-1,000 10-MRY bootstrap policy; separate 0.2-MRY historical
population allowance; no combined formula, mint/grant/nonzero fee/reward API. D-01/D-04
remain open. Real admission, fingerprint/ECG intake, private matching, anonymity and
final human consensus are later work. **Gate A and Gate B remain UNMET.**
