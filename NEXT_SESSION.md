# Next session — MB-03E2b

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-26**.

## Recover current state first

Read AGENTS.md, CURRENT_STATUS.md, the-prompt, MASTER_PLAN.md's tracker and section 23,
and docs/DECISIONS.md. Verify live branches, PR #1, recent commits/comments and CI.
Preserve newer/unpushed work and runtime data. Never reset to this snapshot.

Implementation source: **9ad479d7c89a5b68e28e752c28b174f9885f16c5**.
Tested tree: **bc91bc43e0446be3fbffec1c3da0601cf12619c8**.
Parent: **b0582a41fa9286ebe7e8f29d8c5ab91b8ee366f4**.
[Source push CI 45726543378](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45726543378)
and [source PR CI 45726549735](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45726549735)
both passed. Full push-job `139597729216` logs confirmed the exact source checkout and
all required workspace, documentation, release, reference, example and audit gates.
Later documentation-head CI is checked separately; do not treat these source runs
as evidence for an unknown future head. PR #1 remains open and unmerged on
`work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

**MB-03E2a is VERIFIED_COMPONENT — REPLACEMENT / CLEAN RECONCILIATION.**
MB-03 and E2 remain IN PROGRESS. Gate A and Gate B remain UNMET.

Read [store lifecycle](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[MSR2/preparation](docs/STORAGE_REPLACEMENT_V2.md), [fork choice](docs/FORK_CHOICE_V2.md),
[actual chain](docs/CHAIN_V2.md), [ledger](docs/LEDGER_STATE_V2.md),
[append recovery](docs/STORAGE_RECOVERY_V2.md), [threats](docs/THREAT_MODEL.md)
and [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md).

## Preserve the implemented boundaries

`ReplacementStoreV2` now initializes, inspects, reopens and replaces complete
accepted V2 branches on disk. The selected generation lives in
`generation-{generation:020}` with its exact MNM2 manifest and full MRB2 history;
unchanged MSR2 selects it. A clean store contains exactly that directory and
`selection.v2`. At most the current and next generation coexist during publication,
with one optional `selection.pending`. No directory scan chooses authority.

A replacement rechecks independent network/current checkpoint/generation pins,
strict preference, the entire selected disk history and canonical metadata under
continuous exclusive ownership of the stable root inode. It stages, synchronizes
and fully validates the complete candidate, publishes the marker and synchronizes
the root, then revalidates the selected candidate and **all old bytes before any
old deletion**. Cleanup removes only exact named unselected files and empty
directories. It never replaces the locked root or recursively deletes a tree.

Final clean replay, synchronization and exact comparisons precede the single memory
update. Accounts, memberships, permanent reservations, retained approvals, sequences,
population high-water and resource counters all come from the selected branch.
No orphaned state is grafted into it. The candidate input is borrowed and remains
available if the operation fails.

Disk-phase failures fence ordinary state access and subsequent replacements.
The last-confirmed checkpoint describes prior memory only. Any error after a
publication attempt, including cleanup failure, may leave the new branch selected.
Consuming `reconcile` retains the same lock and returns a writer only after full
validation/synchronization of a **clean** independently selected layout.
Interrupted staging/cleanup is rejected and preserved by normal open/inspect/reconcile.

The encoded selected-payload ceiling is **19,559,132 bytes**; two histories and
two selectors are bounded by **39,118,264 bytes**. Every payload read and growth
sentinel is charged to one **313,011,680-byte** allowance per call. A replacement
performs five bounded full replays plus exact byte comparisons, with a conservative
successful-read bound of **176,068,508 bytes**. These are not RSS or latency bounds.
The unchanged per-chain caps are 4096 blocks, 16 MiB MRB2 and 1,048,576 cumulative
state/configuration items. Cooperative Linux locks and trusted local owners/parents
remain assumptions; WSL/DrvFS/native Windows/network filesystems are not newly verified.

MSR2 remains exactly 280 bytes, version 2/profile 3, local u64 generation,
unchanged 232-byte MSS2 and the replacement checksum domain. Initialization uses
generation zero at any accepted height; each strictly preferred complete replacement
advances generation once, with generation <= height <=4095. Generation is not score,
proof of past publication or global freshness. Prior profiles/fixtures stay frozen.

## Exact next work

**Next: MB-03E2b — explicit recovery of interrupted replacement staging and cleanup.**

Use [the store contract](docs/STORAGE_REPLACEMENT_STORE_V2.md), especially section 6.
Retain independent identity, old/new full checkpoints and generations, **both exact
accepted old and candidate histories**, and an explicit completion/retention action.
Old checkpoint plus candidate alone cannot authenticate partly deleted old files.
Reconstruct any missing reference history independently against its retained pins;
never infer it from corrupt or missing residue.

Before mutation, fully validate the request, strict preference/generation transition,
the actually selected complete history and every allowed residual name/byte. Support
only the documented bounded shapes. Before directory synchronization, surviving
staged files or old cleanup files may be arbitrary subsets rather than ordered
height prefixes. Each candidate residual must match its authorized canonical byte
prefix; every surviving old file must match its full exact bytes. A pending selector
requires a complete candidate. A selected candidate must be complete and cannot be
rolled back by retaining old.

Keep the same exclusive root ownership through consuming recovery, final replay,
synchronization and returned writer. Errors return no writer. Define idempotent
completion and retention, retry every failure boundary, preserve unknown/corrupt
artifacts and missing/corrupt selected markers, and retain finite input/read/work
budgets. Existing one-child recovery is not generic branch recovery. Interrupted
initialization stays outside this recovery scope unless separately reviewed.

Then proceed to fresh-peer reconstruction; persistent node/pool/wallet/operator APIs
remain MB-04, complete Ant Kiss MB-05 and human consensus MB-06. No monetary-policy
choice, device purchase or real biometric collection is needed for E2b.

Future recovery must account for cleanup ordering: block unlinks and block-directory
sync, empty blocks-directory removal and generation sync, exact manifest unlink
and generation sync, then empty old-generation removal and root sync. Partial old
files are not an authorized deletion subset. Unknown entries and invalid selected
history must reject before the first cleanup mutation. A candidate publication is
not undone by restoring an earlier marker after cleanup has started.

Add concrete completion/retention/retry tests for every recognized state and failure
boundary, with arbitrary surviving subsets, wrong pins/history, foreign prefixes,
partial selectors, read budgets, actual independent-process ownership and genuine
membership forks. Preserve E2a normal-open strictness. A persisted intent is not
required by the chosen layout; adding one would need a separate reviewed contract.

## Verified checkpoint and required gates

Local verification used Rust **1.98.0** and locked dependencies:

- **836 workspace/all-target runtime tests passed**, zero failures: 787 retained
  plus **49 new** (32 public integration, 14 private failure/read-budget and three
  cross-process ownership tests). Multiple injected boundaries within one function
  are scenarios, not additional tests.
- **Eight retained compile-fail doctests passed**. No new doctest count is claimed.
- Required release gates passed: personhood **234 runtime + 3 doctests**; core
  **121**; state **49**; consensus **23**; chain **85 runtime + 3 doctests**;
  storage **295 runtime + 2 doctests**. Release reruns are not new unique tests.
- All **nine independent Python references** and **twelve synthetic examples**
  passed. Existing fixtures, formats, dependencies and Cargo.lock are unchanged.
- One subprocess-only helper remains intentionally ignored during ordinary
  enumeration and explicitly invoked by parent tests; it is not an extra pass.
- Scoped formatting/whitespace and monetary/runtime-data audits passed.
  **114 of 116 retained source/dependency/reference/fixture files are byte-identical**
  to the parent. The two changed retained Rust files add module exports/docs and
  replacement-profile cases to the existing process harness. No runtime data is tracked.

Two read-only implementation/contract reviews found no blocker. These checks are
not an external security audit or physical power-loss certification.

Run all AGENTS.md gates, including the twelfth synthetic example
`cargo run -p mirror-storage --example replacement_store_v2_demo --locked`.
Verify exact published source/full CI logs and later documentation-head CI separately.
Before every Mirror final response refresh/publish the-prompt and relevant checkpoint
files under an expected-head guard. Keep PR #1 unmerged; do not change main.

No extra founder funds or marketing reserve. Do not choose D-01's combined bootstrap
formula or D-04's fees/rewards. Historical Alice funds remain public development
fixtures. No raw/encrypted biometric archive, personal medical record or real secret
belongs on-chain or in GitHub. Pinky/ECG/human and accessible-route preferences remain
planning inputs, not measured uniqueness or device support.
