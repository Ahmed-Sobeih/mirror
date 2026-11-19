# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB02B-001 — atomic V2 accounts/membership VERIFIED_COMPONENT**.
Updated: **2026-11-20**. MB-01A, MB-01B/B1–B4, MB-02A and MB-02B are complete
within their recorded component scope. MB-02 remains **IN_PROGRESS**.
**Next: MB-02C — pinned V2 genesis, actual block acceptance and replay. READY.**

## Exact source and publication

Active branch: `work/personhood-register-v0.1`; [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
is open and unmerged. Main preserved at `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
MB-02B implementation: [`420d7114ddf782469a6ace6ba0a2c4076fb93aaf`](https://github.com/Ahmed-Sobeih/mirror/commit/420d7114ddf782469a6ace6ba0a2c4076fb93aaf).
Source tree: `6871ad0fbc178bd76262a26416a7d255d9fcbd66`, identical to the locally
verified tree. [Exact-source push CI 44619155972](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619155972).
**SUCCESS**. Job `135895834032` full logs were inspected: exact source checkout,
all test/reference/example gates and the allocation/runtime audit passed.
[PR CI 44619172378](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619172378).
also **SUCCESS**.

Checkpoint documentation follows the source implementation. Inspect later
exact-head CI separately; do not label a newer documentation head tested merely
because the implementation passed. Never reset newer work to this checkpoint.
Previous MB-02A source `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382` and its evidence
remain in master-plan section 15; section 16 records the MB-02B increment.

## What this milestone implements

**One atomic ledger transition.** `mirror_state::v2::LedgerV2` composes accounts
with membership; `StagedLedgerV2` holds the complete successful candidate. Full
body limits and parent association precede state cloning and ordered execution.
Every transfer, enrollment, approval and withdrawal executes at its original
position. A late failure discards both components, including candidate expiry,
account sequences, earlier transfers and membership changes. The parent never
mutates. Empty blocks still process membership expiry and height.

**Checked transfers and preserved policy.** Positive zero-fee transfers consume
an exact independent account sequence, check funds, recipient arithmetic and
account capacity, and preserve the initial synthetic supply. Self-transfers
consume one sequence without changing money. Explicit zero allocations and
zero-balance accounts remain present so capacity and sequence history cannot
vanish. Membership status adds no unapproved spending restriction. There is no
mint, reward, fee redistribution, bootstrap payout or founder privilege.

**Indexed membership staging.** A consuming, non-cloneable builder preserves full
mixed-block indices while retaining the old contiguous membership-only API. It
charges omitted transfers at the frozen 243-byte size plus four-byte prefix.
Only a completely executed plan can finish; failed/skipped operations cannot leave
an accessible partial candidate. MPS2 now bounds runtime inclusion positions by
max_operations, rather than the membership-count cap. This is an explicit accepted
index-range extension; serialized bytes/domains and all old fixtures are unchanged.
Decoded snapshots remain untrusted. MPH2 remains membership-only.

**Canonical commitments.** Sorted account records include address, balance and
account sequence. Separate account, existing MPS2 membership and disabled-economics
roots feed a combined root committing configuration and processed height. Tip hash
remains outside state to avoid a circular header commitment. See the exact
[ledger contract](docs/LEDGER_STATE_V2.md).

**Independent calculations.** A fifth Python reference reproduces five states and
an alternate mixed ordering from explicit synthetic history. Sixteen signatures
were independently generated and verified before Rust execution. Eight Rust vector
tests matched on their first run; no failed Rust output became an expectation.
The new example shows successful mixed execution and unchanged parent state after
late failures in both directions. It accepts no actual block or real enrollment.

## Observed local and exact-source CI verification

| Check | Result |
|---|---|
| Workspace/all-target tests | **462 passed = 403 retained + 59 new** |
| New cases | **18 indexed staging + 28 ledger integration + 5 private arithmetic/invariant + 8 independent vectors** |
| Documentation tests | **3 compile-fail cases passed**: snapshot authority and two consuming-builder boundaries |
| Core release rerun | **121 tests passed** |
| Personhood release rerun | **234 tests + the same 3 doctests passed** |
| State release rerun | **49 tests passed** |
| Independent references | **All 5 passed**; the new reference also regenerated/verified all 16 signatures locally |
| Synthetic examples | **All 5 passed**, including atomic ledger execution |
| Changed Rust formatting and diff checks | **Passed** |
| Preservation and dependency audit | **Passed**; only one internal state-to-personhood dependency was added |

Release reruns are not new unique tests. Independent cross-reviews found no
blocking discrepancy in roots, budgets, original execution order or atomicity.
These checks are not an external security audit. No external package changed.
All legacy source behavior, frozen previous vectors, historical genesis/runtime
data and monetary-policy constants are preserved. No runtime data is tracked.

## Exact boundary and next work

**A staged V2 ledger is now implemented; actual V2 block acceptance is next.**
`bind_to_validated_block` attaches trusted caller provenance but does not verify
that a block, work proof or claimed root is correct. A stable trusted parent view
is still supplied externally. MB-02C must establish that authority from pinned
configuration/genesis and actual headers/bodies, reject a wrong final ledger root
without publication, and replay complete validated MRB2 histories. T27's wrong-root
case and canonical-chain recovery remain unfinished despite mixed-operation rollback.

MB-03 then adds durable coherent publication, locking, synchronization and recovery
across accounts and membership. Existing MPH2 diagnostic replay cannot replace
mixed-block history by omitting transfers. MPS2 decoding cannot restore authoritative
runtime state. Current easy PoW does not supply the intended equal-human consensus.

No extra founder allocation or privileged marketing reserve exists. The first
1,000 qualifying humans, including the founder, have the same agreed 10-MRY
bootstrap policy, not yet paid. Ordinary allowance is 0.2 MRY per qualifying
historical population maximum; 1 MRY = 10,000,000 Nusa. D-01 combined bootstrap
formula and D-04 fees/rewards remain unresolved. The historical public Alice
100-MRY fixture remains development data, not a founder entitlement.

Fingerprint/ECG capture, private duplicate matching, real admission, anonymous
membership proofs and human-round consensus remain later work. Exact key/evidence
equality is not biological uniqueness, and public review metadata remains linkable.
**Gate A and Gate B remain UNMET.**
