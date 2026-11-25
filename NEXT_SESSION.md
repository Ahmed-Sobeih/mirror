# Next session — MB-03E2

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-26**.
Next: **MB-03E2 — durable replacement layout, publication and explicit recovery.**

## Recover current state first

Read AGENTS.md, CURRENT_STATUS.md, the-prompt, MASTER_PLAN.md's tracker and section 22,
and docs/DECISIONS.md. Verify live branches, PR #1, recent commits/comments and CI.
Preserve newer/unpushed changes and runtime data. Never reset to this snapshot.

Active branch: `work/personhood-register-v0.1`.
[PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1) is open and unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Implementation source: **349edf5a6f453f365fa55480f6117c240876b0e1**.
Tested tree: **b88c9cafc8bf417d5609743bacb1ab278070bd0e**.
[Source push CI 45721188614](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45721188614)
and [source PR CI 45721198833](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45721198833)
both passed. Full push-job `139579639094` logs confirmed the exact source checkout,
787 workspace tests, eight compile-fail doctests, every required release suite,
nine independent references, eleven examples and the allocation/runtime-data audit.
Later documentation-head CI is a separate check; verify the live branch/PR rather
than treating this record as evidence for an unknown future head.

**MB-03E1: VERIFIED_COMPONENT — METADATA / IN MEMORY.** MB-03 remains IN PROGRESS.
Gate A and Gate B remain UNMET. Prior source evidence remains in master sections 18–21.

Read [replacement metadata/preparation](docs/STORAGE_REPLACEMENT_V2.md),
[fork choice](docs/FORK_CHOICE_V2.md), [accepted chain](docs/CHAIN_V2.md),
[immutable storage](docs/STORAGE_V2.md), [append storage](docs/STORAGE_APPEND_V2.md),
[explicit append recovery](docs/STORAGE_RECOVERY_V2.md), [ledger](docs/LEDGER_STATE_V2.md),
[threats](docs/THREAT_MODEL.md) and [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md).

## Preserve the implemented boundaries

MSR2 is exactly 280 bytes: magic MSR2, encoding version 2/profile 3, u64 little-endian
local generation, unchanged 232-byte MSS2, then SHA256d of the replacement domain
and first 248 bytes. Independent full endpoint/generation pins are mandatory.
Generation zero initializes any accepted height; each strictly preferred replacement
would increment once. Generation <= height, currently <=4095. It is not score,
publication-history proof or global freshness. Never reinterpret old MSS2/MSU2.

The pure `prepare_replacement_v2` checks network identity for both accepted chains,
then current chain endpoint, selector endpoint/generation, entire canonical current
metadata, strict preference and checked candidate generation/metadata. Recomputed
checksums plus matching endpoints cannot bypass the exact manifest/byte-total check.
The private plan retains both immutable complete chains and the candidate's exact
manifest/ledger/resources. It encodes bounded manifests but performs no filesystem
operation. The two borrow doctests prevent either input changing while the plan is used.
A saved plan/selector/choice is not a disk mutation permit.

The underlying comparator accepts only complete ChainV2 histories and independent
configuration/genesis identity. Every block uses the same literal target; normalized
height is the score. Strictly higher wins, all ties keep current. Actual hashes
establish ancestry; balances alone do not. Sticky local ties are not global finality.

Local verification used Rust 1.98.0 with locked dependencies:

- **787 workspace/all-target runtime tests passed:** 735 retained + 31 new codec
  tests + 21 new preparation tests. Zero failures.
- **Eight compile-fail doctests passed:** six retained + two replacement-plan
  borrow checks. Both accepted chains remain immutable while their plan is used.
- Required release suites passed: personhood **234** runtime + **3** doctests;
  core **121**; state **49**; consensus **23**; chain **85** + **3** doctests;
  storage **246** + **2** doctests. Reruns are not additional unique tests.
- **Nine independent Python reference checks** and **eleven synthetic examples**
  passed, including the new MSR2 reference and replacement preparation demo.
- One subprocess-only storage helper remains intentionally ignored in ordinary
  enumeration and is explicitly invoked by parent tests; it is not another pass.
- Scoped formatting, whitespace and allocation/runtime-data audits passed.
  **108 retained source/dependency/reference/fixture files are byte-identical**
  against parent `3da75720319094f1b91b4a337b45cfdeb6869e7b`. The only existing Rust
  file changed is the storage module's additive documentation/exports. Dependencies,
  old format bytes/vectors and Cargo.lock are unchanged. No runtime data is tracked.

## Exact next work — MB-03E2

1. **Freeze the disk and cleanup state machine before writes.** Read replacement
   contract section 5. Choose a bounded separate layout that can hold two different
   blocks at the same height. Decide explicitly between reusable slots and bounded
   retained-generation directories, including interrupted cleanup. E1 assigns no
   path to a generation and implements no persisted intent. Keep root inode stable.
2. **Specify ownership and authority.** Retain cooperative exclusive ownership for
   current revalidation, staging, publication, recovery and in-memory advancement.
   Independently pin network, full current checkpoint/generation and exact candidate.
   Replay actual MRB2; use the candidate's own manifest. Recheck disk metadata and
   complete history under the held lock. A checksum, peer height, scan result or
   previously saved preference never selects a branch on its own.
3. **Implement whole-state publication.** Synchronize complete candidate files and
   directories before marker publication; synchronize the stable root before memory
   advances. Keep accounts, memberships, permanent reservations, approvals, sequences,
   population history and resource counters together. Orphaned effects must not
   remain canonical. Preserve old selected data until its replacement is safely
   selected; define exactly when retained data can be cleaned up.
4. **Define failure and explicit recovery.** Freeze every recognized selected/staged
   state, fencing/uncertainty error, independently retained retry intent and allowed
   completion/retention action. A selected new generation cannot be rolled back by
   retaining old. Unknown or corrupt artifacts remain untouched and rejected.
   Existing one-child Complete/RetainParent is not general fork recovery. Keep the
   lock throughout consuming recovery, and return no writer after unresolved error.
5. **Bound all resources.** Retain actual-chain caps: 4096 blocks, 16 MiB complete
   MRB2, 1,048,576 cumulative state/config items. One complete MSR2-selected payload
   is at most 19,559,132 bytes; two manifests/histories/selectors total 39,118,264
   bytes. This is planning arithmetic, not an implemented E2 staging/read/RSS budget.
   Account for extra markers, repeated comparison/replay reads, sentinels and cleanup.
6. **Verify real forks and every boundary.** Include equal-account/different-membership
   histories; reopen exact complete candidate state; check stale/wrong pins and
   unpreferred candidates, every publication/recovery interruption, repeated actions,
   corrupt/foreign staging, caps and real second-process ownership. Preserve all old
   formats, fixtures and APIs. New persisted bytes need an explicit contract and
   independent vectors. Run full gates and inspect exact-source CI before checkpointing.

Keep implementation increments manageable and label any remaining publication,
cleanup or recovery work explicitly. No money-policy choice, device purchase or
real biometric collection is needed for this storage work. After E2, implement
fresh-peer reconstruction; MB-04 supplies persistent node/pool/wallet/operator APIs,
MB-05 the full Ant Kiss lifecycle and MB-06 human consensus.

## Required checks and retained policy

Use Rust 1.98.0 and locked dependencies. AGENTS.md requires workspace all-target and
doc tests, release core/personhood/state/consensus/chain/storage tests, nine Python
references, eleven examples and allocation/runtime-data checks. Scoped formatting
must not rewrite unrelated old source. Verify the exact published source and its
full CI logs; verify the later documentation head separately.

No extra founder funds or marketing reserve. Do not choose D-01's combined bootstrap
formula or D-04's fees/rewards to get tests green. Historical Alice funds remain
public development fixtures. No raw/encrypted biometric archive, personal medical
record or real secret belongs on-chain or in GitHub. The user's combined initial
pinky/ECG/witness and accessible-route preferences remain in the-prompt and the
protected-device/accessibility planning documents; no uniqueness guarantee is implied.

Before every Mirror final response update and publish the-prompt plus relevant
checkpoint files under an expected-head guard. Keep implementation evidence separate
from subsequent documentation commits. Do not merge merely to publish the handoff.
