# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03D-001 — fixed-target development fork choice**.
Updated **2026-11-26**. Prior MB-01/02 and MB-03A/B/C components remain verified.
**MB-03D: VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON.**
**MB-03 remains IN PROGRESS. Next: MB-03E — durable membership-aware branch replacement.**

## Combined enrollment clarification — 2026-11-26

The user favors ECG + a designated pinky + human-supervised Ant Kiss as the initial
standard design, and ECG + observed capture limitations + reviewed Ant Kiss when
a fingerprint is unavailable. The user believes the combination can be sufficient
for launch and may consider more fingerprints later. The updated
[proposal](docs/ACCESSIBLE_ENROLLMENT_PROPOSAL.md) and
[device draft v0.3](docs/DEVICE_INPUT_CONTRACT.md) preserve that preference while
keeping measured performance, exact procedures and launch acceptance unresolved.
No additional collection or real launch is initiated. This documentation-only
clarification changes no source, test evidence, progress estimate or next task.

## Accessibility proposal — 2026-11-26

The user requested a future enrollment route for people unable to supply the
standard readings, with an accommodation description, a proposed committee of
three qualified people and controls on later changes/removal. The new
[accessible-enrollment proposal](docs/ACCESSIBLE_ENROLLMENT_PROPOSAL.md) preserves
that idea separately from recommendations for minimum private explanations,
reviewed corrections/appeals and continuity of the same membership and claim history.
A disability description is not proof of uniqueness. Reviewer qualifications,
selection, quorum and alternative evidence remain open; no public medical label or
runtime exception is implemented. The roadmap links this future work to MB-05,
MB-08, MB-09, MB-10 and MB-12. Source, verified test evidence, progress estimate and
the next coding task MB-03E are unchanged. This documentation update adds no test result.

## Device discussion — 2026-11-26

The user reaffirmed flexible equipment under human supervision and proposed one
anatomical finger plus ECG, possibly the left little finger, to minimize collection.
The exact finger/profile and exception policy remain undecided. The updated
[device input draft](docs/DEVICE_INPUT_CONTRACT.md) keeps observations off-chain,
permits compatible adapters/imports, and distinguishes witnessed capture from
tested duplicate matching. The chain receives authorized membership outcomes.
This documentation-only discussion adds no adapter, biological-uniqueness evidence,
test result or completed milestone. MB-03E remains next; the estimate below is unchanged.

## Progress estimate — 2026-11-26 discussion

For the user's question about finishing the software before fingerprint/ECG device
integration, the current rough estimate is **20–25% of Gate A**. This is an
engineering judgment about the agreed scope, not a measured completion ratio,
test-count fraction, time forecast or security score. It remains broadly consistent
with the earlier roughly 20% estimate; MB-03D alone does not imply a fixed increase.

The implemented foundations cover accepted blocks, combined balances/membership,
bounded storage/append/recovery and branch comparison. Durable V2 branch replacement,
peer reconstruction, persistent node/wallet interfaces, the complete Ant Kiss
workflow, human-based consensus, policy-dependent money rules, protected off-chain
intake/matching interfaces and integrated hardening remain substantial work.
Gate A means the complete agreed flow works with synthetic readings and is prepared
for compatible device adapters. Actual device compatibility, captures and recognition
evidence belong to Gate B. Both gates remain unmet.

This discussion changes no implementation, policy gate or next task. The last
verified documentation checkpoint remains `ff7bf31080a047bd612cb79adc4eba854c0a6845`,
with successful push CI 45704514304 and PR CI 45704523896. This later documentation-only
update adds no test result; check its own head's CI separately.

## Exact source and publication

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
CI 44750251798 and PR CI 44750262332. MB-03C source and exact-source acceptance
remain in master-plan section 20. This documentation follows the new source; its
own exact-head CI is checked separately and recorded on PR #1. Recheck live refs.

## Delivered behavior

The [fork-choice contract](docs/FORK_CHOICE_V2.md) and
`compare_branches_v2(expected_identity, current, candidate)` compare two complete,
already accepted `ChainV2` values. Both must match an independently selected
configuration hash and exact genesis hash. The function checks current C/genesis
before candidate C/genesis. It accepts no raw block, peer score, state cache or
storage-generation input.

**Score and choice.** Every accepted block in this profile must use the same literal
configured target. Accepted height is therefore an exact normalized ordering of
accumulated target-based work: each post-genesis block contributes one normalized
unit. No large-integer work total is computed. A strictly higher candidate score
wins; all ties keep the caller's current branch. An unusually low hash, large
timestamp, more people, more money or higher storage generation adds no score.

**Actual ancestry.** Matching accepted header hashes are compared from genesis
until the first mismatch. The result distinguishes identical histories, either
proper-prefix direction, and genuine divergence. It includes the complete checkpoint
of the last shared block, not a matching balance or membership root alone.

**Whole-state selection.** The private comparison result borrows both inputs and
returns an immutable reference to the actual preferred chain. Its blocks, manifest,
accounts, membership, approvals, permanent records, population history and resource
counters stay together. No component is grafted from a sibling; neither input is
mutated. Safe Rust prevents either input being changed while the comparison remains
in use.

The result is a preference for these two accepted inputs. It performs no disk or
network operation, publishes no canonical selection and supplies no durable commit
certificate. Copied choices/checkpoints can become stale; a later store operation
must revalidate current state and its candidate under exclusive ownership.

## Verification

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

The 28 integration cases cover all branch relations, both tie orientations, real
membership-divergent forks with identical account bytes, preserved candidate
approvals/population/counters, empty-block expiry, independent identity rejection
on both sides with exact error order, common ancestors at genesis and deeper
heights, terminal timestamps and complete 4,096-block histories. Rejected peer
blocks cannot increase accepted score.

Three new compile-fail doctests cover mutation of either borrowed input and passing
an unaccepted BlockV2 in place of a ChainV2. The three earlier doctests remain.
One subprocess helper remains intentionally ignored in ordinary enumeration and
is invoked by parent tests; it is not another passed test.

The new `fork_choice_v2_demo` constructs two accepted height-3 histories with equal
account bytes and different membership. It retains current on the tie, then extends
the candidate to height 4 and selects that complete branch. Observed selected tip:
`74ace822b0cbdc059fc8cfef336f5f3c88668a76ca437107443af0279860f666`.
Ledger root:
`2654699af4ccfeeffb0fca1b7b9ade6a76201134285de1737bb10cc964cbe0fd`.
Active / pending / high-water: **1 / 1 / 2**.
Encoded bytes / cumulative state items: **3,605 / 73**.
Only public synthetic fixtures are used; the example writes no store.

Focused formatting and preservation checks passed. **105 retained source/dependency/
reference/fixture files remain byte-identical.** Existing chain validation code is
unchanged; its module only gains exports/documentation. CI adds the tenth example.
No dependency, encoding, hash domain, reference vector, storage format or monetary
rule changed. Two source/contract reviews found no blocking discrepancy under the
documented accepted-chain invariants; this is not an external security audit.

## Limits and next work

Each input retains its own 4,096-block, 16-MiB MRB2 and 1,048,576 cumulative-item
bounds. Normalized score is in `0..=4095`. Comparison uses at most 4,096 pairs /
8,192 fixed-header hashes and constant additional state, with no heap allocation,
branch clone or ledger replay.

Equal-score retention is deterministic for the ordered inputs, not a global
unique winner. Different peers can retain different tied tips. At a terminal
timestamp or finite history cap, extension may be impossible; this component does
not promise eventual convergence or human finality. A future variable-target or
human-weighted profile needs an explicitly reviewed replacement rule.

**Next MB-03E:** specify and implement durable replacement of the selected branch,
using the complete preferred candidate under independently checked network/current
pins and the same exclusive stable-root ownership. Define publication, interrupted
replacement and retry behavior before implementation. The existing one-child
append/recovery API must not silently become a reorganization API. Fresh-peer
membership reconstruction follows.

MB-03 remains unfinished. The persistent V2 node, human consensus, policy-gated
issuance/fees, private biometric matching and actual fingerprint/ECG integration
remain later work. D-01/D-04 are open; no founder/marketing privilege or combined
bootstrap formula is added. **Gate A and Gate B remain UNMET.**
