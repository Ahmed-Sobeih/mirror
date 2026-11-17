# Current status

Roadmap: MIRROR-PLAN-001, [MASTER_PLAN.md](MASTER_PLAN.md).
Latest task checkpoint: **MIRROR-MB01A-001 — MB-01A design deliverables complete**.
Runtime implementation checkpoint remains **MIRROR-PH-001**.
**Next: MB-01B/B1 — implement V2 configuration/bootstrap binding and bounded rules.**
Status: READY; V2 runtime implementation has not started.

## Publication and source reviewed

Active branch: `work/personhood-register-v0.1`.
PR #1: https://github.com/Ahmed-Sobeih/mirror/pull/1 — open, ready for review,
not merged. Default main observed at `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
Source inspected for MB-01A: `db2861930cc0f01f735ccfba80a4f530f4d1ddd6`.
Recheck live refs/comments/CI before continuing; never reset newer work.

## Completed in this turn

- Reviewed live personhood implementation/tests/codec, core transaction/block
  encodings, account execution, chain acceptance, storage and framed networking,
  plus PR #1 comments and the accepted decisions/roadmap.
- Published docs/LEDGER_MEMBERSHIP_V1.md: compatibility path, noncircular config/
  seed binding, proposed signed layouts, sequences/anchors/windows, state changes,
  parent eligibility, block-end activation, resource caps and combined ledger root.
- Published docs/THREAT_MODEL.md: F01–F11 source observations, risks and owners.
- Published docs/MEMBERSHIP_TEST_MATRIX.md: 35 proposed acceptance cases T01–T35,
  all explicitly NOT_IMPLEMENTED. Old tests are not substituted for these cases.
- Updated the master tracker, agent instructions, decision log and continuation
  files so the next session starts B1, not another MB-01A planning pass.

Engineering selection: an explicit synthetic membership-dev-v2 profile and new
MRB2/MPH2/versioned modules, not a silent change to MRB1/MPH1 or old genesis.
Proposed development limits/ordering/zero-fee integration profile are not final
production economic, reviewer, recovery or human-consensus decisions.

Design consistency arithmetic was checked independently: prefix 131 bytes;
header 160; empty block 168; signed operations 243/291/227/227 bytes, within the
proposed budgets. This does not execute or validate the proposed Rust protocol.
MB-01A VERIFIED means its design deliverables are complete, not implementation.
No Rust source, dependency, transaction encoding, balance or runtime data changed.

## Findings to implement, not already fixed

The existing rules context omits ordered bootstrap contents. Mutation APIs have
no overall pending/record/batch limits. Requests have no chain anchor/height expiry.
Activation is immediate, permitting new members to authorize further events in
one batch; V2 will use parent eligibility and block-end activation. Account state
and membership are still separate. Storage/transport limits and real private
deduplication need later milestones. F01–F11 remain open until tested fixes exist.

## Actual existing implementation and tests

Legacy devnet: signed transfers, account state, easy fixed PoW, persistence, TCP
sync and competing-history replacement. Standalone registry: signed Pending/
Active/Revoked membership, distinct active approvals, voluntary self-withdrawal,
exact request/key/evidence reuse rejection, atomic batches, counters, roots and
bounded log replay. Separate monetary helpers have no mint/claim API.

Source implementation: `3936d2a528e73444b667860f8987bbebcfe7ab78`.
Expanded CI: `b0e9b75aad28f3804fc3a35a0bde6bfa6807e2b3`, run 44181156418.
Later exact source/plan checkpoint `db2861930cc0f01f735ccfba80a4f530f4d1ddd6` passed
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44189324502.
Recorded suite: 133 workspace tests, including 26 personhood and 5 monetary-policy;
26 personhood release-mode reruns; synthetic demo; documentation-test command with
zero cases; allocation/runtime-data audit. MB-01A adds no new executed Rust tests.
Inspect exact publication-head CI/PR comments for checks after this documentation
commit. Do not describe an earlier CI run as testing a later implementation.

## Preserved boundaries

No real enrollment, biometric matching, anonymous credentials, live membership
blocks, human-round consensus, grants/rewards/fees or device adapters are enabled.
Both readiness gates remain UNMET. No cryptographic commitment alone establishes
biological uniqueness. Public member/reviewer relationships remain linkable.

No founder bonus or privileged marketing reserve. First 1,000 eligible humans
including founder have the same proposed 10 MRY bootstrap grant; ordinary allowance
is 0.2 MRY per qualified historical population maximum. D-01 combined-ceiling and
D-04 reward rules remain unresolved. No new policy was silently selected.

Preserve Alice's 100-MRY public dev fixture and local mirror-data, not a founder
entitlement. No raw/encrypted biometric archive, keys or user data in GitHub.
