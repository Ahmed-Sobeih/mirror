# Next session — MB-02B

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated 2026-11-19.
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md, docs/CORE_V2_ENCODING.md,
docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md, docs/THREAT_MODEL.md,
docs/MEMBERSHIP_TEST_MATRIX.md and the latest the-prompt.

## Recover live state

Verify default/active branches, PR #1, recent commits/comments and exact CI.
At handoff branch `work/personhood-register-v0.1` is active; PR #1 is open/unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`. Read feature-branch records,
not old main. Preserve concurrent/unpushed work and runtime data. Never reset to a
copied checkpoint or merge merely to publish a handoff.

MB-02A implementation `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382`, tree
`930aa22bf9810525372fd31926812c1bde6dcf81`. Exact-source push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399
See CURRENT_STATUS.md for inspected job results; check later documentation-head CI
separately. The identical tree passed local Rust 1.98.0 checks: 403 workspace tests,
1 doctest, core release 121, personhood release 216 plus the same doctest,
four references, four examples, formatting and preservation/allocation audit.
MB-01A, MB-01B/B1–B4 and MB-02A are complete within component scope. Do not redo them.

## Existing APIs and validation boundaries

`mirror_core::v2` contains immutable SignedTransferV2, SignedOperationV2,
BlockHeaderV2 and BlockV2, with operations_root_v2. Core depends on personhood;
personhood depends only on crypto. Preserve this acyclic direction. Transfer
signatures use the frozen B2 prefix/domains; the sender uses the existing
MIRROR_ADDRESS_V1 namespace. Signed transfer =243 bytes, header=160, empty MRB2=168.
Membership bytes, MCF2, MPS2, MPH2 and legacy formats remain unchanged.

`BlockV2::verify_integrity` checks budgets, C, ordered-operation root and signatures.
It does NOT validate balances, sequences, transfer amount/fee policy, operation
windows, parent linkage, header height/time, PoW, ledger root or genesis. Transfer
`validate_for_parent` adds the static zero-fee/positive-amount profile and common
context/anchor/window checks, but still does not execute or grant acceptance.
Keep these named boundaries explicit when composing the next executor.

## Exact next slice — MB-02B atomic mixed-ledger execution

Deliver an immutable staged V2 ledger whose accounts and membership either both
advance or both remain unchanged. Use the reviewed ledger design, without minting,
grants or real-device handling. Inspect the current APIs before editing.

1. **Specify full-block positions first.** B3 stage_block and B4 MPH2 are currently
   membership-only. Add a deliberate indexed/full-block staging boundary retaining
   each membership operation's original index, including intervening transfers.
   Define total/member/enrollment counts, valid increasing indices and budgets.
   Review MPS2 index validation too: profiles with fewer membership slots than
   total slots must not reject valid full-block indices. Preserve frozen prior
   bytes and membership-only behavior; document any required profile/version
   change rather than silently weakening validation. Filtered MPH2 is not mixed history.
2. **Compose immutable state.** Add separate V2 state modules combining canonical
   accounts, membership and disabled-economics state. Bootstrap from exact
   ConfigV2/VerifiedBootstrap and explicit synthetic allocations. Check sums and
   max_accounts; expose no arbitrary mutation/untrusted-cache bypass. Actual
   chain/manifest authority remains MB-02C.
3. **Define execution order.** Preflight limits before input-sized cloning or
   signature batches. Specify error precedence. Reuse authorization checks and
   B3 expiry, retained-history, parent-eligibility and block-end activation rules.
   Same-block activation cannot grant review rights. Empty blocks still expire
   Pending requests and advance processed height.
4. **Execute checked transfers.** Require amount>0, fee=0, exact account sequence
   with checked increment, sufficient funds, checked recipient credit and account
   capacity. Specify self-transfers so no funds are created/destroyed. Account and
   membership sequences stay distinct. Add no unapproved member gate or hidden fee.
5. **Stage atomically.** Late transfer failure must discard earlier membership
   effects; late membership failure must discard earlier transfers. Success
   returns a complete staged ledger, leaving the parent unchanged. Enforce
   sum(balances)==initial synthetic supply. Membership counters grant no payouts.
6. **Freeze roots independently.** Specify sorted account fields/counts, disabled
   economics and combined C/height/accounts/membership/economics roots using
   ledger-spec section 7. Keep the current block hash outside state commitments.
   Independently compute genesis, empty and mixed-block vectors, including
   nonadjacent membership indices. Do not bless unexplained Rust results.
7. **Verify concrete risks.** Test overflow/exhaustion/capacity, wrong context/parent,
   replay/overspending/self-transfers, both directions of late atomic failure,
   original inclusion order, empty expiry, replay equality and supply conservation.
   A synthetic example must distinguish executed state from accepted canonical blocks.
8. **Publish a reviewable slice.** Run workspace/all-target and doc tests, relevant
   release suites, all references/examples, changed-file formatting and preservation
   audit. Inspect exact source CI, then update and read back master/status/matrix/
   next/the-prompt. Release reruns do not count as new tests.

If integration needs smaller substeps, name them explicitly. Indexed membership
support alone does not complete MB-02B. D-01/D-04 stay open; zero fees/no minting
are development semantics only.

## Following slices

- **MB-02C:** explicit manifest and pinned exact genesis; actual configuration,
  parent/height/time/target/PoW/operation/ledger-root validation; stable candidate
  ancestry and accepted-block replay. Only then bind staged state provenance to
  the validated hash. Test competing histories with different memberships.
- **MB-03:** writer locking, durable coherent publication/recovery, peer sync and
  reorganizations across accounts and membership. In-memory execution/replay is
  not crash safety, latest-tip discovery or distributed consensus.

## Existing trust and history boundaries

VerifiedBootstrap requires every seed proof; seeds have ordinary runtime rights.
Membership bind_to_validated_block attaches provenance but checks no actual block.
Parent views must be stable locally validated ancestry, never a peer oracle.
MPS2 decoding returns an untrusted wrapper without runtime conversion. MPH2 needs
trusted per-height hash/group digest/root and expected C/height/tip/root. Its digest
is not MRB2's root, and dropping transfers cannot preserve mixed execution history.

Preserve 4,096-group/1,048,576-state-item diagnostic replay budgets, including
configuration-copy charges, and approval/history capacities. Never evict tombstones
or retained approvals. Gate A/B are unmet; later monetary, privacy, human-round,
operator and device gates are unchanged. No real capture or procurement is needed.

Before every Mirror final response, refresh the-prompt and relevant checkpoints
with actual progress, exact source/CI and next action. Read before editing, preserve
concurrent changes, publish with SHA guards and verify readback. Separate source
evidence from documentation commits; do not chase self-referential SHAs.
