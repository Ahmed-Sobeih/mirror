# Next session — MB-03E

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-26**.
Next: **MB-03E — durable membership-aware branch replacement.**

## Recover current state first

Read AGENTS.md, CURRENT_STATUS.md, the-prompt, MASTER_PLAN.md section 21 and
docs/DECISIONS.md. Verify the live default/active branches, PR #1, recent commits,
comments and exact-head CI. Preserve newer or unpushed changes and runtime data.
Never reset to this snapshot.

Active branch: `work/personhood-register-v0.1`.
[PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1) is open and unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Implementation source: **b9a2e870996fcafe1128dcb5cfdfb6b01b158ded**.
Tested tree: **dd0a3fc29f429dcd4999261801c2bc7897d6aace**.
Published-source [push CI 45703714223](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703714223)
and [PR CI 45703726306](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703726306)
both passed. Full push-job `139520118282` logs confirmed the exact implementation
checkout, **735 workspace tests, six compile-fail doctests, all required release
suites, eight independent references, ten examples and the allocation/runtime-data
audit**. The single intentionally ignored subprocess helper is invoked by parent
tests and is not another pass. Release reruns are not additional unique tests.
MB-03D is **VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON**. Subsequent
documentation-head CI is checked separately and recorded on PR #1.

The preceding handoff `c3de6f4be32fd2c22523f06674a96295da8ec48e` passed push
CI 44750251798 and PR CI 44750262332. Prior storage/recovery evidence is preserved
in master-plan sections 18–20. Later documentation-head CI is checked separately
and recorded on PR #1; recheck the live head before continuing.

**MB-03D: VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON.** MB-03 remains IN PROGRESS.
Gate A and Gate B remain UNMET.

Read [fork choice](docs/FORK_CHOICE_V2.md), [actual chain](docs/CHAIN_V2.md),
[immutable storage](docs/STORAGE_V2.md), [append storage](docs/STORAGE_APPEND_V2.md),
[explicit recovery](docs/STORAGE_RECOVERY_V2.md), [atomic ledger](docs/LEDGER_STATE_V2.md),
[blocks](docs/CORE_V2_ENCODING.md), [configuration](docs/CONFIG_V2_ENCODING.md),
[membership operations](docs/OPERATIONS_V2_ENCODING.md),
[membership state](docs/MEMBERSHIP_STATE_V2.md), [replay](docs/REPLAY_V2.md),
[integration](docs/LEDGER_MEMBERSHIP_V1.md), [threats](docs/THREAT_MODEL.md)
and [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md).

## Preserve the comparison boundary

`compare_branches_v2(expected: ManifestIdentityV2, current: &ChainV2,
candidate: &ChainV2)` requires two complete accepted histories. It checks current
configuration/genesis and then candidate configuration/genesis against independent
expected identity. It takes no raw blocks, untrusted score or storage generation.

All accepted blocks use the exact configured literal bits. Score is accepted
post-genesis block count (= height, 0..=4095). A strictly longer valid candidate
wins; equal score retains current. This normalized ordering avoids literal work
multiplication/overflow, not actual hardware differences. A lower individual hash,
population, balances, bytes, timestamps and local generation add no weight.

`BranchRelationV2` distinguishes Identical, CandidateExtendsCurrent,
CurrentExtendsCandidate and Diverged. The common ancestor is the last matching
accepted block hash and its complete checkpoint. Comparisons use at most 4,096
pairs / 8,192 fixed-header hashes, no heap allocation or execution.

`BranchComparisonV2` privately borrows both inputs. Its selected_chain() returns
the whole preferred ChainV2, preserving all ledger and resource state. Safe Rust
prevents changes to either input while that result is still used. Copied enums
and checkpoints are diagnostic and can become stale. They are not a durable
authorization, certificate, checkpoint codec or global freshness proof.

Sticky ties do not define a global ordering of equal-score branches. Different
current tips can remain different until a strictly stronger valid branch appears;
finite caps/terminal timestamps can prevent that. This is development fork choice,
not human finality. No encoding, target, admission or monetary rule was changed.

Local verification used Rust 1.98.0 and locked dependencies. All required gates passed:

- **735 workspace/all-target tests = 707 retained + 28 new integration tests.**
- **Six compile-fail doctests = three retained + three new ownership/type checks.**
- Release suites: core **121**, personhood **234** plus its three retained doctests,
  state **49**, consensus **23**, chain **85** plus its three new doctests, and
  storage **194**. These reruns are not additional unique tests.
- **All eight independent reference checks and all ten synthetic examples passed.**
- The single subprocess-only helper remains intentionally ignored in ordinary
  enumeration and is explicitly invoked by parent tests; it is not another pass.
- Focused formatting and preservation checks passed: **105 retained source,
  dependency, reference and fixture files are byte-identical**. Existing ChainV2
  validation code is unchanged; its module gains documentation and exports only.

## Exact next work — MB-03E durable replacement

The comparator now identifies a preferred accepted branch; no implemented V2
storage API can durably replace a selected history with a divergent one. Implement
that boundary as a separately specified increment.

1. **Freeze the ownership and authority contract first.** Review both frozen storage
   profiles and MB-03C recovery. Retain independently chosen network identity and
   the exact expected current checkpoint/local generation. Validate/replay the
   complete candidate through ChainV2 and compare under the same stable exclusive
   root lock. Never derive freshness or authority from a peer, checksum, highest
   generation, cached counters or a saved ForkChoiceV2 enum.
2. **Specify storage layout and publication.** Existing height-named files cannot
   hold two differing blocks at the same path. Decide explicitly how candidate
   history is staged, retained and selected without replacing the locked root
   inode, overwriting selected history early or silently changing MSS2/MSU2 meaning.
   Prefer a separate reviewed profile/layout if needed. New persisted bytes need
   versioned contracts and independent reference vectors before expected values
   are frozen. An in-memory preference is not a crash-consistent transaction.
3. **Define complete-state replacement.** Preserve the common prefix and execute
   each branch under its own ancestry. Install the candidate's complete blocks,
   manifest, accounts, membership, permanent reservations, approvals, sequences,
   population peaks and cumulative resource counters together. Orphaned effects
   must not survive. Do not invent a monetary rollback policy or enable issuance.
4. **Specify failures and recovery before writing.** Define each durable boundary,
   fencing/uncertainty result, independent retry intent, selected old/new layouts,
   and what happens to known staging after interruption. Unknown or ambiguous data
   stays preserved/rejected. Existing one-child Complete/RetainParent is not a
   generic fork replacement procedure. Separate initialization/append/replacement
   states and maintain bounded reads/scans/staging.
5. **Test genuine forks and ownership.** Use unchanged membership-divergent
   histories with equal account bytes; assert exact candidate state and removal
   of orphaned effects after reopening. Cover shorter/equal/rejected candidates,
   wrong network/current pins, every publication/recovery boundary, repeated
   recovery, malformed staging, finite bounds and actual second-process exclusion.
   Compare pre/post bytes and complete states, not only tip hashes or balances.
6. **Verify and publish the scoped component.** Run required full gates, inspect
   exact-source CI, then update master/current/next/decisions/the-prompt. A durable
   replacement API still does not implement live peer synchronization or finality.

Follow this with fresh-peer reconstruction of complete V2 membership-bearing
histories. MB-04 supplies persistent node/pool/wallet/operator APIs and MB-06 human
consensus. If the replacement format design is large, keep specification and its
implementation/acceptance as explicit consecutive substeps without claiming the
unimplemented part complete.

## Required checks and invariants

Use Rust 1.98.0 and locked dependencies. Required gates:

- `cargo test --workspace --all-targets --locked`
- `cargo test --workspace --doc --locked`
- Release tests for core, personhood, state, consensus, chain and storage.
- All eight independent reference scripts with `--check`.
- All ten examples: registry_demo, operation_v2_demo, state_v2_demo, block_v2_demo,
  ledger_v2_demo, chain_v2_demo, fork_choice_v2_demo, storage_v2_demo,
  append_v2_demo and recovery_v2_demo.

Preserve actual-block validation, original mixed-operation indices, strict
timestamps/literal bits, full ledger roots, zero/spent accounts, separate sequences,
expiry/end-block activation, permanent identity history and existing finite caps.
No pruning, cap bypass, authoritative cache or root-inode swap.

Current append recovery retains independently specified parent/candidate endpoints
and exact child bytes, full replay before mutation, explicit Complete/RetainParent,
no rollback of a selected child, strict ordinary opening, and its 19,559,412-byte
encoded-input / 156,508,096-byte conservative repeated-read bounds. Unknown files
are never deleted merely to make opening succeed. Preserve existing lock tests.

No founder/marketing privilege, combined bootstrap formula, mint/grant/nonzero
fee/reward API, actual biometric capture or anonymity claim. User policy gates stay
explicit; they do not block dependency-ready storage work. Gate A/B remain unmet.
