# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03E2A-001 — durable whole-branch replacement**.
Updated **2026-11-26**. Prior MB-01/02 and MB-03A/B/C/D/E1 components remain verified.
**MB-03E2a: VERIFIED_COMPONENT — REPLACEMENT / CLEAN RECONCILIATION.**
**MB-03 remains IN PROGRESS. Next: MB-03E2b — explicit interrupted replacement recovery.**

## Exact source and publication

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

Prior implementation and exact-source CI remain in master sections 18–22. The E1
source was `349edf5a6f453f365fa55480f6117c240876b0e1`; this increment preserves its
MSR2 bytes and pure preparation contract.

## Delivered behavior

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

See [the replacement store contract](docs/STORAGE_REPLACEMENT_STORE_V2.md) for the
exact lifecycle, ordering, residual-state table and limits.

## Verification

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

The new `replacement_store_v2_demo` performs an actual synthetic disk switch from
fork height **3 / generation 0** to main height **4 / generation 1**, removes the
unselected old generation, then reopens and reconstructs the exact candidate.
Tip: `33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`.
Ledger root: `785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.
Active/pending/high-water: **1/0/3**. MRB2 bytes/state items: **4330/75**.
A separate integration case switches branches with identical account bytes but
different membership history, checking the whole chain and all counters after reopen.

## Next work and practical limits

The encoded selected-payload ceiling is **19,559,132 bytes**; two histories and
two selectors are bounded by **39,118,264 bytes**. Every payload read and growth
sentinel is charged to one **313,011,680-byte** allowance per call. A replacement
performs five bounded full replays plus exact byte comparisons, with a conservative
successful-read bound of **176,068,508 bytes**. These are not RSS or latency bounds.
The unchanged per-chain caps are 4096 blocks, 16 MiB MRB2 and 1,048,576 cumulative
state/configuration items. Cooperative Linux locks and trusted local owners/parents
remain assumptions; WSL/DrvFS/native Windows/network filesystems are not newly verified.

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

This is durable local storage, not a persistent V2 peer service. Gate A and Gate B
remain UNMET. No issuance, fees/rewards, biometric capture or human consensus is added.

## Retained user decisions and design input

No founder privilege or privileged marketing reserve. The equal first 1,000 grants
of 10 MRY include the founder. The ordinary allowance is 0.2 MRY per qualifying
historical population maximum; the combined bootstrap formula and fee/reward rules
remain unresolved. This step introduces no issuance, grant, fee or reward API.

The user's preferred initial standard enrollment combines ECG, one designated pinky
and human-supervised Ant Kiss. A proposed exception combines ECG, observed capture
limitations and reviewed human supervision, with a proposed committee of three
qualified people. The exact finger, committee/quorum, procedures and measured
acceptance remain open. The user considers the combined routes potentially enough
for launch and may consider more fingerprints later; this is a design preference
and sufficiency hypothesis, not measured uniqueness or launch acceptance.

Keep minimum explanations and biometric records protected off-chain. Normal and
exception routes share duplicate-checking scope and the same continuing membership
and entitlements. No public medical label, mandatory all-finger collection or
runtime accommodation mechanism is implemented. See [the proposal](docs/ACCESSIBLE_ENROLLMENT_PROPOSAL.md)
and [device draft v0.3](docs/DEVICE_INPUT_CONTRACT.md).

The last rough estimate remains **20–25% of Gate A** as discussed on 2026-11-26.
It is an engineering scope estimate, not a ratio of tests, a time forecast or a
security score; this bounded increment does not justify an automatic percentage rise.
Gate A uses synthetic readings and complete software integration. Actual device
compatibility, capture and recognition evidence belong to Gate B.
