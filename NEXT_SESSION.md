# Next session — MB-01B/B2

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md).
Design: [LEDGER_MEMBERSHIP_V1.md](docs/LEDGER_MEMBERSHIP_V1.md).
Implemented configuration bytes: [CONFIG_V2_ENCODING.md](docs/CONFIG_V2_ENCODING.md).
Risks/tests: [THREAT_MODEL.md](docs/THREAT_MODEL.md) and
[MEMBERSHIP_TEST_MATRIX.md](docs/MEMBERSHIP_TEST_MATRIX.md).

## Recover the live state

Verify default branch, active branches, PR #1, recent commits/comments and exact CI.
At handoff branch work/personhood-register-v0.1 is active, PR #1 open/unmerged;
main is still the old development foundation. Read AGENTS.md, CURRENT_STATUS.md,
MASTER_PLAN.md, this file, docs/DECISIONS.md, the integration/configuration designs,
threat/test matrix, device contract and the latest the-prompt. Do not reset newer
work to a copied snapshot or merge merely to publish a checkpoint.

B1 is implemented and tested at `809661b5e6a9338d65c15c45e540289df24fcc99`, run
44201393943: 164 workspace tests, 57 personhood release reruns, Python vectors,
legacy synthetic demo, doc command and allocation audit passed. Do not redo B1
or MB-01A. Keep the exact frozen MCF2 bytes/reference fixture unchanged unless an
explicit encoding revision is documented and independently reverified.

## Exact next slice — MB-01B/B2

1. Inspect `crates/mirror-personhood/src/v2/` and crypto APIs. Add typed, bounded
   V2 membership operation/envelope modules alongside the configuration module.
   Preserve V1 APIs, MPH1/MRB1 and all existing vectors; no dependency cycle.
2. Implement the design's 131-byte unsigned prefix and typed Enroll/Approve/
   Withdraw bodies. Signed lengths: 291/227/227 bytes. Reserve Transfer=0 for the
   separately specified MB-02 ledger work; do not accidentally accept transfers
   through a membership-only decoder. Document shared-envelope ownership so
   MB-02 can extend it without changing already-signed membership bytes.
3. Bind the validated ConfigV2 context, ancestor height/hash, inclusive validity
   window, actor key/sequence, kind and body into strict signature verification.
   Derive complete-request enrollment IDs and complete-signed-operation IDs under
   the exact versioned domains in the design. Bind approvals to the entire target.
4. Add explicit parent-chain ancestry resolution/checking interfaces. Validate
   anchor_height < candidate height, exact canonical ancestor hash, maximum age,
   anchor_height < valid_from <= candidate height <= valid_until, inclusive span
   bound and checked arithmetic. Reject unknown/orphan/future anchors. Local clock
   claims and peer-supplied counts are never authoritative. Resolver trust must be
   explicit; B2 tests can use a bounded synthetic parent view, not a fake live chain.
5. Check exact codec lengths/versions/kinds before allocation; reject every
   truncation/trailing input. Test mutation of every signed field, wrong network/
   bootstrap config, invalid/weak keys, signature tampering, IDs, windows at limits,
   u64 overflow and anchor substitutions. Produce independent signed-preimage/hash
   vectors, not outputs copied from a failing Rust implementation.
6. Update matrix T04 membership portions, T05, target-binding portions of T06,
   and B2 portions of T09/T10 with exact tests/commit/CI. Record remaining stateful
   requirements honestly. Sequence fields are signed now; consuming sequences and
   checking active/pending actors requires B3, not just signature validation.
7. Run pinned workspace/all-target, documentation command, release personhood,
   independent references and demo/audit checks. Publish only observed results;
   do not mark all MB-01B or all 35 matrix rows complete from B2 alone.

## Then continue

B3: bounded staged V2 membership execution, sequence spaces, pending expiry,
parent-set eligibility, block-end activation, tombstones and consistent counters.
Test failed batches unchanged and prevent same-block chains of new approvers.
B4: full registry codec/roots/replay, adversarial regression suite and V2 demo.

MB-02 adds TransferV2, BlockV2 and one atomic accounts+membership ledger; MB-03
adds durable locking/replay/synchronization/reorgs with DIFFERENT membership histories.
Do not substitute a standalone membership log for consensus-integrated state.

No monetary choice blocks B2. D-01/D-04 still block real grants/minting/rewards.
Biometrics remain synthetic/off-chain; hardware support, privacy architecture and
real admission are later gates. Preserve local runtime data and test balances.

Before every Mirror-related final response, refresh the-prompt and relevant
master/status/agent/matrix records with actual work, tests, limitations and the
next exact slice. Read files before editing, use expected-head guards, and verify
published contents. Checkpoint documentation after a tested source is not itself
new runtime evidence; inspect its exact-head CI separately.
