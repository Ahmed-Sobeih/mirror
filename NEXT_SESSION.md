# Next session — MB-03B

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-20**.
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/STORAGE_V2.md,
docs/CHAIN_V2.md, docs/LEDGER_STATE_V2.md, docs/CORE_V2_ENCODING.md,
docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md,
docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/THREAT_MODEL.md, docs/MEMBERSHIP_TEST_MATRIX.md and the latest the-prompt.

## Recover live state

Verify default/active branches, PR #1, recent commits/comments and exact CI.
Active branch `work/personhood-register-v0.1`; PR #1 is open/unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`. Read feature-branch records,
not stale main. Preserve concurrent/unpushed work and all runtime data. Never reset
to a copied checkpoint or merge merely to publish handoff documents.

Latest published implementation `528c8912c5a50cfd2e86856d345b29d6bb79d2c5`, tree
`65004e73f4c19deedafdb8cf2874939df0ab367a`, identical to the locally verified tree.
[Corrected-source push CI 44639322362](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44639322362),
job `135965942902`, and [PR CI 44639329630](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44639329630)
both SUCCEEDED. Full push-job logs confirmed the exact corrected-source checkout
and all verification gates. The original source's push run 44638337326 exposed a reproduced
cross-test fork-inheritance race. The corrected test isolation preserves all real
process assertions and passed 200 repeated release runs. No production retry or
lock behavior was changed. Check later documentation-head CI separately.

Local Rust 1.98.0 gates passed: 588 workspace tests, one explicitly ignored subprocess
helper, 3 retained doctests; release core 121 / personhood 234 plus the same 3 doctests /
state 49 / consensus 23 / chain 57 / storage 75; all 7 references and 7 examples;
formatting, diff, preservation and dependency checks. The 65 new tests are 24 codec,
34 filesystem, 5 ownership/process and 2 fault tests covering 9 initialization
boundaries. Release reruns and those boundary scenarios are not additional tests.
No external dependency changed. Source changes are published; CI verification is
not inferred from the local result.

**MB-03A VERIFIED_COMPONENT — IMMUTABLE INIT/REOPEN. MB-03 IN PROGRESS.**
**Next MB-03B READY. Gate A/B UNMET.** Previous milestone evidence remains in the
master plan, particularly sections 17–18.

## Reuse the completed storage and chain

`StoreV2::initialize(path, &ChainV2)` initializes an absent final path under an
existing parent or an explicitly empty directory. `open_writer` and `inspect`
require independent `ManifestIdentityV2` and `ChainReplayExpectationV2` arguments.
Every reopen reconstructs complete actual blocks through `ChainV2::replay_encoded`.
No cached accounts/membership or on-disk assertion supplies authority. External
pins choose a finite branch; they do not discover global freshness.

The frozen MSS2 profile 1 record is exactly 232 bytes at generation zero. The root
contains exact `manifest.v2`, contiguous `blocks/*.mrb2` and `selection.v2` published
last. `selection.pending` cannot authorize a branch. Initialization synchronizes
prepared files/directories before marker rename and synchronizes the root afterward.
Prepublication failure retains partial data; publication uncertainty returns no
handle and requires checked reconciliation. Nothing automatically repairs, deletes,
initializes from a peer or rewrites legacy data. There is no append or reorg API.

Linux root-directory inode locks last as long as the handle. An exclusive writer
blocks writers and inspectors; shared inspectors coexist and block writers. Root
identity must remain stable: do not rename, remove or replace a locked store root.
There is no lock file or PID recovery. Closing a handle closes its descriptor,
but ownership lasts until all inherited descriptors for the same open file
description close; a concurrent fork can therefore prolong `Busy` briefly.
Local owners and parent directories are
trusted; static alias checks are not a hostile-owner sandbox. The operating-system
gate does not validate every filesystem. See STORAGE_V2.md for actual tested and
untested filesystem assumptions and the exact error/publication contract.

Keep ChainV2's private accepted ancestry and atomic full-body executor. Validate
version/configuration, height/parent/time, exact configured target/work, ordered
operation root and complete executed ledger root before binding accepted state.
Do not introduce external mutable ancestry or a cache/state setter. Lower-level
`bind_to_validated_block` only attaches trusted provenance. Empty blocks, original
mixed-operation indices, separate account/member sequences, explicit zero/spent
accounts, permanent membership records and initial test-supply conservation persist.
MPH2 is membership-only; MPS2 remains an untrusted cache with no runtime conversion.

Chain caps include genesis: 4,096 blocks, 16 MiB complete MRB2 and 1,048,576 cumulative
state/configuration items. Preserve the exact charge formula in CHAIN_V2.md, strict
child timestamp greater than parent, and literal configured difficulty bits. There
is no wall-clock/future ceiling or retargeting. Recovery must not evade capacity,
erase history or silently choose a new finite profile.

## Exact next slice — MB-03B durable accepted-tip updates

Freeze the update contract before implementing a new file format or changing
publication. Work remains separate from the legacy node and from monetary/device
policy. The first deliverable is a small reviewable continuation of verified storage.

1. **Define a versioned update profile and migration boundary.** Preserve MSS2
   profile 1/generation zero and every prior encoding. Specify whether a new profile
   uses new metadata and how an existing immutable store can be explicitly adopted,
   if supported. Never silently reinterpret an old store or accept nonzero generations
   under profile 1. Distinguish external network/branch selection from local durable
   generation authority; metadata must not manufacture its own trust or freshness.
2. **Define ownership, staging and publication.** Keep the same stable locked root
   for the owner lifetime. Start with an already validated full ChainV2 and a bounded
   one-block append transition, reusing actual acceptance for the complete body.
   Specify which files can be staged, how they are named/bounded, and the exact file
   and directory synchronization order before a new selected-tip record is published.
   Account, membership, population, approvals and resource counters must all describe
   the same accepted history; do not create a second authoritative cache.
3. **Specify failure states before exposing writes.** A candidate must not become
   selected before all required durable data is prepared. If I/O may have published
   a new selection, fence further writes until explicit reconciliation. Define how
   retained old/new metadata and staged data are interpreted on restart, who supplies
   expected authority, and when the operation returns success, uncertainty or failure.
   Do not delete unexplained files or treat a directory/checksum as sufficient proof.
4. **Implement and test the bounded append slice.** Cover accepted empty and mixed
   transfer/membership blocks, rejection without changes, capacity and height/time
   boundaries, a second writer, and reopened exact ledger/counter equality. Inject
   deterministic failures at writes, syncs, selector publication and reconciliation.
   Restart must identify a complete old or new selected branch, or explicitly reject;
   no mixed ledger/selection state or partial success may escape. Document the limits
   of simulated errors versus actual power-loss/filesystem testing.
5. **Publish precise evidence and continuation.** Run required workspace/doc/release
   gates, all 7 existing references/examples plus any added ones, preservation checks
   and exact-source CI. Record implemented substeps, unimplemented failure cases and
   the next action. Never mark all MB-03 complete from an append-only storage slice.

After durable accepted-tip updates, specify canonical development fork choice before
branch replacement, then implement durable reorganization with genuinely different
admissions/withdrawals and peer reconstruction. Orphan approval/population/resource
state must not survive selecting another branch. A peer-advertised height/root
cannot choose local trust. MB-04 provides the persistent V2 node, wallet/pool and
operator APIs; human finality belongs to MB-06.

## Constraints unchanged

No founder bonus, privileged marketing reserve, mint/grant API, nonzero development
fees or reward allocation. First 1,000 qualifying humans receive the same 10 MRY,
including the founder. Ordinary allowance is 0.2 MRY per qualifying historical
population maximum; 1 MRY is 10,000,000 Nusa. D-01 combined bootstrap treatment and
D-04 fees/rewards remain open and do not block this no-mint storage work. The legacy
public Alice fixture is not a founder entitlement and must not be rewritten.

Key/evidence equality is not biological uniqueness; public review metadata is
linkable. Fingerprint/ECG devices are interchangeable off-chain Ant Kiss instruments.
Raw or encrypted biometric archives, real keys, device secrets and personal records
never belong on-chain or in GitHub. No real capture or device purchase is needed.
Current fixed easy PoW is not the final human-participation consensus.

Before every Mirror final response refresh the-prompt and relevant checkpoints
with actual results and next work, preserve newer source, publish with head guards
and verify readback. Record implementation evidence separately from documentation
heads; never chase a self-referential SHA or merge merely to publish a checkpoint.
