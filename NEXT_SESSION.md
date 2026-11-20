# Next session — MB-03D

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-20**.
Next: **MB-03D — canonical development fork-choice specification and implementation.**

## Recover live state first

Read AGENTS.md, CURRENT_STATUS.md, the-prompt, MASTER_PLAN.md section 20 and
docs/DECISIONS.md. Verify the live default and active branches, PR #1, recent
commits and comments, and CI for the exact source and subsequent documentation
head. Preserve newer or unpushed changes and runtime data. Never reset to a copied
checkpoint.

Active branch: `work/personhood-register-v0.1`. [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
remains open and unmerged. Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Published source: **e8302052ab724f59ff4a212dc64aea9575a6d4b3**.
Tested tree: **dc9eb2c02436f0952ac54593ba8d02317e9d9314**, identical to the locally checked tree.
Source increment: **9 files, 3,138 insertions and 1 deletion**.
[Source push CI 44748718086](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748718086)
and [source PR CI 44748724632](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748724632).
Both source runs SUCCEEDED. The full push-job log, job 136330677456, was
inspected: its checkout is exactly e8302052ab724f59ff4a212dc64aea9575a6d4b3.
Workspace/doctest/release counts, all eight references, all nine examples and
the allocation/runtime-data audit passed.

The preceding documentation checkpoint `6c53f188ef7d5052f96577ed04f5f909f56966c1`
passed push CI 44650139143 (full job 136002692765 inspected) and PR CI 44650145750.
Current documentation follows the new implementation. Its exact-head CI is checked
separately and recorded in PR #1; recheck that live head when continuing. Never
confuse an older successful run with a later head.

MB-03C is **VERIFIED_COMPONENT — EXPLICIT ONE-CHILD RECOVERY**. MB-03 remains IN PROGRESS.
Gate A and Gate B remain UNMET. Earlier MB-03A/B evidence stays in master-plan
sections 18–19. Local verification passed 707 workspace tests, three compile-fail
doctests, 194 storage release tests, eight independent references and nine examples.
The exact-source CI evidence above is separate. The 49 new cases comprise
39 integration tests, nine private interruption tests and one process parent test.
One intentionally ignored subprocess helper is invoked by parent tests and is not
counted as another passing test.

Read the normative contracts:
[actual chain](docs/CHAIN_V2.md), [immutable storage](docs/STORAGE_V2.md),
[append storage](docs/STORAGE_APPEND_V2.md), [explicit recovery](docs/STORAGE_RECOVERY_V2.md),
[atomic ledger](docs/LEDGER_STATE_V2.md), [mixed blocks](docs/CORE_V2_ENCODING.md),
[configuration](docs/CONFIG_V2_ENCODING.md), [membership operations](docs/OPERATIONS_V2_ENCODING.md),
[membership state](docs/MEMBERSHIP_STATE_V2.md), [diagnostic replay](docs/REPLAY_V2.md),
[integration](docs/LEDGER_MEMBERSHIP_V1.md), [threats](docs/THREAT_MODEL.md)
and [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md).

## Preserve the completed recovery boundary

`AppendRecoveryRequestV2::new(parent, candidate, exact_block)` retains both
independent `AppendCheckpointV2` values and the exact intended child. Heights and
local generations must be adjacent. Recovery replays the complete on-disk parent
and accepts the actual child through `ChainV2`, even for `RetainParent`. It derives
the exact old and new selectors from accepted histories, including manifest and
byte-total metadata. Checksums alone cannot establish that those fields are correct.

`AppendStoreV2::recover_interrupted(root, identity, &request, action)` takes exclusive
ownership of an existing root. Consuming `store.recover(&request, action)` keeps
the same writer lock and requires its last confirmed memory checkpoint to match
one request endpoint. Every failure returns no usable handle.

`Complete` may resume an absent intended child or write only missing canonical
suffixes. An old selected marker with a pending selector requires the full exact
child. `RetainParent` removes the matching pending selector first, synchronizes
the root, then removes the matching unselected tail and synchronizes `blocks/`.
An already selected candidate cannot be rolled back. It supports only idempotent
`Complete`, with the complete child and no pending selector. Unknown or corrupt
artifacts, non-prefix data, missing selected history and unsupported layouts remain
preserved and rejected.

The external request is narrow authorization, not historical provenance proof.
Empty/common prefixes cannot identify who created them. No API derives authority
from the damaged files or stores that authority for the caller. Interrupted
initialization and arbitrary corruption repair remain unsupported.

Validated parent files are synchronized before mutation. The marker, layout and
artifact bytes are rechecked, and complete bytes and current file identity are
checked before synchronization. Final success requires strict clean loading,
equality with the complete expected chain, ledger and resource counters, and
resynchronization. Do not introduce deletion of unknown files, automatic cleanup,
selection by largest generation or a setter that makes cached state authoritative.

MSS2 profile 1, MSU2 profile 2, MNM2/MRB2, independent reference fixtures and normal
strict open/inspect/reconcile semantics are unchanged. Recovery adds no new format
or journal. Maximum distinct encoded recovery input is 19,559,412 bytes. The enforced
conservative allowance for repeated reads is 156,508,096 bytes, including sentinels.
Metadata counts are bounded independently. These are not resident-memory limits.
Chain caps and state-work charges remain unchanged. Keep the stable root inode and
current process-test isolation; inherited fork descriptors may prolong lock lifetime.

## Exact next work — MB-03D development branch comparison

The V2 accepted-chain and storage components currently require independently chosen
history. They do not define which competing valid branch should become canonical.
Implement that missing selection boundary before adding branch replacement or a
peer-driven automatic selection path.

1. **Specify selection inputs and authority.** Compare complete, already accepted
   `ChainV2` values under an independently chosen configuration and genesis. Reject
   different network identities. A peer's advertised height or work, a cache,
   a local storage generation or a structurally decoded block cannot substitute
   for acceptance. Keep the comparison result separate from durable publication.
2. **Freeze development work accounting and ties.** Inspect the configured literal
   difficulty, target decoding, accepted genesis and current finite caps. Define
   the work or score arithmetic, accumulation, overflow behavior and tie rule
   before coding. Document whether this fixed-target profile can compare height
   directly or should expose an explicit score. Do not import variable-difficulty
   assumptions or treat a lower individual block hash as extra work. This is a
   temporary development consensus choice, not equal-human finality.
3. **Define common ancestry and outcomes.** Identify a shared ancestor using actual
   accepted block hashes. Distinguish identical chain, extension, shorter/equal
   competing branch and genuinely divergent preferred branch. Specify exactly what
   a caller is authorized to do with each comparison result. No result by itself
   rewrites disk or proves global latestness.
4. **Preserve ledger semantics and ownership.** Selecting a candidate must include
   its complete accounts, membership, permanent records, approvals, population peaks
   and resource counters. Do not graft state from a sibling or preserve orphaned
   effects. Existing append/recovery operates on one selected lineage and must not
   become an implicit reorganization API.
5. **Exercise genuine conflicts.** Use the frozen membership-divergent forks, equal
   account bytes with different membership, shorter/longer/equal branches, exact
   equality, identity mismatch, boundary heights and any arithmetic edges. Every
   rejected comparison must leave both accepted inputs and all stores unchanged.
   If new externally encoded score/selection bytes are introduced, specify them
   and calculate independent vectors before freezing expected values.
6. **Verify and publish the component.** Run all required workspace, documentation,
   release, reference and example gates, then verify CI for the exact source and
   update the living handoff. Passing comparison tests does not complete durable
   reorganization, live synchronization or finality.

After this selection boundary is implemented, take a separately specified slice
for durable membership-aware branch replacement under the same ownership/recovery
constraints, then fresh-peer reconstruction. A future replacement format requires
explicit publication and interrupted-replacement rules; never reinterpret MSU2
generation as branch score or swap the locked root inode. MB-04 supplies the
persistent V2 daemon, pool, wallet and operator API; MB-06 supplies human finality.

## Required verification and preserved policies

Use pinned Rust 1.98.0 and Cargo.lock. Run:

- `cargo test --workspace --all-targets --locked`
- `cargo test --workspace --doc --locked`
- Release tests for core, personhood, state, consensus, chain and storage.
- All eight independent `reference_*_v2.py --check` scripts.
- All nine examples: `registry_demo`, `operation_v2_demo`, `state_v2_demo`,
  `block_v2_demo`, `ledger_v2_demo`, `chain_v2_demo`, `storage_v2_demo`,
  `append_v2_demo` and `recovery_v2_demo`.

Verify the actual published checkout and full job output. Release reruns and
parameterized scenarios are not additional unique tests.

Keep original mixed-operation indices, private accepted ancestry, strict child
time, literal configured bits, exact work/signature/root validation, zero/spent
accounts, separate account/member sequences, expiry/activation and permanent
history. Preserve 4,096 complete blocks, 16 MiB MRB2 and 1,048,576 cumulative
state/configuration items. No cap bypass, history eviction or authoritative cache.

No founder/marketing privilege, mint/grant API, combined bootstrap formula,
nonzero fee/reward, actual device capture, biometric archive, private deduplication
or anonymity claim. User policy gates remain explicit but do not block this
development branch-selection task. Gate A and Gate B remain unmet.
