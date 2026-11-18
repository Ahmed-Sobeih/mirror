# Next session — MB-01B/B3

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md).
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md,
docs/THREAT_MODEL.md, docs/MEMBERSHIP_TEST_MATRIX.md and latest the-prompt first.
Device input contract and V1 scope remain relevant boundaries, not live support.

## Recover live state

Verify default/active branches, PR#1, recent commits/comments and exact CI.
At handoff branch work/personhood-register-v0.1 is active and PR#1 open/unmerged;
main is the old chain. Preserve concurrent/unpushed work and all local runtime data.
Do not repeat MB-01A, B1 or B2, reset to copied refs or merge to publish records.

B2 implementation af1dd66c4fb8811d8eab54a1c88a8bddae2ae80e passed Actions44205064789:
203 workspace tests, 96 personhood release reruns, both references and demos, docs
command (zero cases), allocation audit. Check later checkpoint-head CI separately.
Freeze all legacy/MCF2 and B2 membership bytes/vectors unless explicitly versioned.

## Exact next slice — B3 staged state execution

Implement separate V2 registry state and an atomic ordered candidate-block
transition using VerifiedBootstrap, ConfigV2 and SignedMembershipOperationV2.
Do not call the V1 immediate-activation executor and label it V2. No transfer,
block encoding, final human consensus or monetary API belongs in this slice.

1. Freeze state records and invariants before coding mutation: Pending/Active/
   Revoked/Expired, original signed request or seed descriptor, inclusion position,
   membership next_sequence, approvals, activation height/ordinal, used-key/evidence
   tombstones, active/pending counts, total activations and end-block high-water.
   Seed keys start sequence0 with ordinary member rights. Config context is mandatory.
2. Provide only bounded staged block application, not an unbounded direct apply
   bypass. Validate parent/config/height and operation/new-enrollment/byte budgets
   before expensive work or cloning input. Bind supplied parent view to this state;
   the resolver must represent locally validated candidate ancestry. Clone/stage
   state, never publish partial changes after an error.
3. Reuse B2 signature/context/anchor/window checks. Enroll requires unused identity
   references and sequence0, then establishes next1. Approve/Withdraw must exactly
   match current sequence, checked-increment once on success. Reject reused/skipped/
   exhausted sequences; invalid events consume nothing. Approve needs existing
   Pending target and matching request expiry, with no earlier valid_from. Withdraw
   must target the signer's own Pending/Active membership, never somebody else's.
4. Expire Pending requests at h>until (even empty blocks), preserve key/evidence/
   ID tombstones and free only pending quota. Parent-Active AND currently-Active
   reviewers may approve. Defer all activation until the whole block passes;
   a member newly admitted here cannot approve in this same block. Same-block
   withdrawals invalidate that reviewer's contribution to new activation. Do not
   cascade-revoke already-active members from old blocks.
5. Activate qualifying requests in original canonical inclusion-position order.
   Bound records/pending/approvals as required by the committed development profile,
   use checked arithmetic, derive and verify counters/indexes. High-water uses
   max(parent_high_water, end_active), never transient peaks. No currency is created.
6. Add tests for T03 runtime privileges, T06 historical reuse, T07, T11–T20/T22
   relevant subcases: unknown targets, unauthorized withdrawals, expired/duplicate
   requests, bad sequences, same-block new reviewers, withdrawal order, bounds,
   simultaneous activations and invalid last event preserving ALL prior state.
   Independent state instances must agree. Actual reorg/ledger rows remain MB-02/03.
7. Run pinned workspace/doc/release/reference/demo/audit CI and update exact evidence.
   B4 will complete canonical registry root/snapshot/log replay/reference vectors
   and adversarial suite; coordinate field layout now, do not silently change it.

B2 success is NOT state authorization. In particular its deliberate test allowing
a context-valid unknown target or u64::MAX approval sequence documents checks B3
must add, not acceptable execution. ParentChainViewV2 is a trust boundary, not proof.

## Following work and boundaries

B4 registry commitments/replay -> MB-02 atomic accounts+membership blocks -> MB-03
locking/durable sync/reorgs with DIFFERENT memberships. Keep the old PoW chain as
explicit development behavior until human-round protocol MB-06 is implemented.
Monetary D-01/D-04 do not block B3 but keep minting/grants/rewards disabled. Real
biometrics/private matching/admission and devices remain later gated tasks.

Before every Mirror final response, update the-prompt and relevant tracker/status/
agent/matrix files with actual work, tests, unresolved gaps and the precise next
unfinished slice. Use read-before-edit and expected-head guards; verify publication.
Do not claim a documentation-only follow-up commit was tested by an earlier run.
