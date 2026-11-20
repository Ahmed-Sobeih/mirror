# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03B-001 — bounded accepted-block append and clean-state reconciliation**.
Updated: **2026-11-20**. MB-01A, MB-01B/B1–B4, MB-02A/B/C and MB-03A remain
verified components. MB-03B is **VERIFIED_COMPONENT — bounded append and clean-state reconciliation**.
Local gates and exact-source CI passed. **MB-03 remains IN PROGRESS.**
**Next: MB-03C — explicit interrupted-append staging recovery. READY.**

## Exact source and publication

Active branch: `work/personhood-register-v0.1`; [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
is open and unmerged. Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
Published MB-03B implementation:
[`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c`](https://github.com/Ahmed-Sobeih/mirror/commit/ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c).
Source tree: `06d7c3f4fa24d96c844e2301c76c01f804151192`, identical to the locally checked source tree.
The implementation increment changes **14 files, 3,568 insertions and 11 deletions**.
It adds the separate append profile, contract, reference, fixtures, tests and example;
all dependencies and prior format fixtures remain unchanged.

[Source push CI 44649111203](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203),
job `135999222947`, and [source PR CI 44649119266](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
both **SUCCEEDED**. Full push-job logs were inspected and confirmed checkout
`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c` and every workspace, documentation,
release, reference, example and audit gate. These are exact published-source results,
separate from the successful local checks. Verification date: 2026-11-20.

The previous MB-03A source `528c8912c5a50cfd2e86856d345b29d6bb79d2c5` and its
corrected source CI remain recorded in master-plan section 18. The previously
published handoff `61a71018ced5263c47fb85186ebcca73200e4807` passed push CI
44639960912 (full job 135968176444 inspected) and PR CI 44639971028. New checkpoint
documents follow the current implementation and require separate exact-head CI.
Do not reset newer work or merge merely to publish handoffs.

## What MB-03B implements

**A distinct appendable store.** `mirror_storage::v2::AppendStoreV2` initializes an
absent or empty directory from a fully accepted ChainV2, appends one complete child,
reopens with exclusive ownership, or inspects with shared ownership. The existing
immutable StoreV2/MSS2 profile remains separate and unchanged. There is no implicit
migration or adoption of an existing directory.

**An explicit local generation.** MSU2 is a new 280-byte profile-2 envelope containing
one exact unchanged 232-byte MSS2 snapshot plus a local generation and domain-separated
checksum. Initialization begins at generation zero at any accepted height. Each
published child advances height and generation together. An uncertain call may
already have published that generation even though it returned an error. Generation
never exceeds selected height and is not global freshness or consensus weight.
Every open requires independent network pins and full chain-plus-generation selection.

**Validation before writes.** Append clones the private accepted chain and validates
the complete proposed block through normal ChainV2 acceptance. Invalid candidates
leave disk and memory unchanged and the handle usable. Before writing, it also fully
replays the current selected disk history against retained expectations and compares
the complete state with memory. Disk disagreement fences the handle.

**Publication before memory advancement.** Under the same exclusive directory lock,
append creates and synchronizes the new block, synchronizes its directory, writes
and synchronizes a pending selector, compares the old selector again, renames the
pending selector and synchronizes the root. Only then does it publish the candidate
chain and generation in memory and return the checkpoint. The root inode remains
stable; no populated directory is swapped.

**Failure fencing and checked reconciliation.** A mutation error preserves artifacts
and prevents further append. Once selector publication may have been attempted,
the error explicitly reports uncertainty. Chain/selection/checkpoint getters reject
while fenced; last_confirmed_checkpoint is labeled as old memory, not disk authority.
Consuming reconcile keeps the same lock, validates an independently selected complete
on-disk chain, synchronizes its files/directories, and returns a usable writer only
on success. Writer reopening does the same resynchronization. Read-only inspection
never synchronizes or changes stored bytes.

**A precise recovery limit.** This slice reconciles only a complete clean selected
layout. An extra next-height block, pending selector, partial file or unknown artifact
is preserved and rejected. It never automatically promotes staging, deletes files,
chooses the largest generation or substitutes stored counters for full replay.
Resolving those interrupted preparation states is MB-03C.

The normative [append contract](docs/STORAGE_APPEND_V2.md) specifies exact bytes,
error boundaries, write order and filesystem assumptions. The frozen
[immutable contract](docs/STORAGE_V2.md) and [actual-chain contract](docs/CHAIN_V2.md)
remain authoritative within their respective scopes.

## Observed local and exact-source CI verification

| Check | Result |
|---|---|
| Workspace/all-target tests | **658 passed = 588 retained + 70 new** |
| New cases | **28 selector + 31 append integration + 8 private failure/recovery + 3 process tests** |
| Documentation tests | **3 retained compile-fail cases passed** |
| Subprocess helper | **1 intentionally ignored entry**, invoked by parent tests and not counted as another pass |
| Release core/personhood | **121 / 234 passed**, plus the same 3 personhood doctests |
| Release state/consensus/chain | **49 / 23 / 57 passed** |
| Release storage | **145 passed**, with the same 1 helper ignored in normal enumeration |
| Independent references | **All 8 passed**, including reference_append_storage_v2.py |
| Synthetic examples | **All 8 passed**, including append_v2_demo |
| Focused format, diff and preservation checks | **Passed**; 90 retained source/dependency/reference/fixture files compared byte-for-byte |

The final workspace run also passed after the last documentation-comment/example
wording edits. Release reruns are not new unique tests. The eight private tests cover
11 append boundaries, eight synchronization boundaries through both reopen and
consuming reconciliation, nine initialization boundaries, partial block/selector
files, ownership continuity, stale pins, compare-before-publication and root replacement.
Scenarios inside a test are not additional tests.

The independent Python reference derives MSU2 vectors from unchanged frozen chain
and MSS2 data before Rust comparison. New fixture SHA-256:
`d6e597fdac8c5be081a54019149ebbe79651817848139d51bc691681b96df668`.
The example appends/reopens each main-history block, verifies every complete state,
rejects a wrong-height candidate and a second writer, and performs consuming clean
reconciliation. Its final tip is `33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`.
All example inputs and temporary directories are synthetic and invocation-owned.

Three new real subprocess tests retain the existing parent-test isolation. They
cover an exclusive child dropping its handle, process termination and shared child/
local inspection, followed by checked reopening and a successful next append.
No sleeps, retries or weaker assertions were added. Two source/contract reviews found
no blocking mismatch; these tests/reviews are not an external security audit or
power-loss/disk-firmware certification.

## Exact boundary and next work

**Next MB-03C:** specify independently authorized recovery of interrupted append
staging, including exact ownership/provenance of residual files, bounded inspection,
explicit complete-or-abandon decisions, synchronization and idempotent retries.
Preserve both existing formats and reject unknown data. Freeze any new intent/metadata
format before implementing it; checksum validity cannot authorize a branch or deletion.
Then specify canonical development fork choice and durable membership-aware branch
replacement, followed by peer reconstruction. MB-04 supplies the persistent V2 node,
pool, wallet and operator API; MB-06 supplies final human consensus.

Chain caps remain **4,096 blocks**, **16 MiB of complete MRB2**, and **1,048,576 cumulative
state/configuration items**. Maximum selected encoded payload for profile 2 is
**19,559,132 bytes**, separately from decoded-memory/staging overhead. No cache authority,
history pruning, cap increase or network consensus rule was introduced. Linux
cooperative directory locks and trusted local owners/parent directories remain explicit
assumptions. WSL2/DrvFS and network filesystems are not verified by the local Linux/Ubuntu
checks. Inherited descriptors can prolong a lock until their last reference closes.

Accepted monetary policy is unchanged: no extra founder allocation or privileged
marketing reserve; equal 10-MRY bootstrap policy for the first 1,000 qualifying humans,
including the founder; separate ordinary 0.2-MRY allowance per historical qualifying
population maximum; 10,000,000 Nusa per MRY. No payout/mint/nonzero fee/reward API is
introduced. D-01/D-04 remain open. Fingerprint/ECG intake, private matching, real
admission and anonymity remain later work. No raw or encrypted biometric archive
belongs on-chain or in GitHub. **Gate A and Gate B remain UNMET.**
