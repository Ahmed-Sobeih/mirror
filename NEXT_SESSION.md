# Next session — MB-01B/B4

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md).
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md,
docs/MEMBERSHIP_STATE_V2.md, docs/THREAT_MODEL.md, docs/MEMBERSHIP_TEST_MATRIX.md,
and the latest the-prompt. Device/V1 documents remain scope boundaries.

## Recover live state

Verify default/active branches, PR #1, recent commits/comments and exact CI.
At handoff branch work/personhood-register-v0.1 is active and PR #1 open/unmerged;
main still contains the old chain. Preserve concurrent/unpushed work and runtime data.
Do not repeat completed MB-01A/B1/B2/B3, reset to copied refs or merge for handoff.

B3 implementation b6f1233d038901aa0a13c8ddac9afbc34418e846 passed direct push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317174000
Job 134846238261 logs inspected: 254 workspace tests, 147 personhood release reruns,
both references, all three demos, doc command (zero cases), allocation audit.
The identical source tree also passed locally with Rust 1.98.0. Check later
checkpoint-head CI separately. Freeze legacy/MCF2/B2 bytes and B3 transition rules.

## Exact next slice — B4 registry commitments, codecs and replay

B3 already executes bounded immutable membership-only candidate blocks. It does
not yet serialize/hash/replay V2 state. Implement that missing boundary using the
frozen layout in docs/MEMBERSHIP_STATE_V2.md, without adding live money or blocks.

1. Add canonical MPS2 state encoding and MIRROR_MEMBERSHIP_STATE_V2 commitment.
   Follow exact field order, status/origin/optional tags, sorted record IDs and
   reviewer keys. Commit configuration, height, all counters, original requests,
   inclusion positions, sequences, activation data and complete approval evidence.
   Exclude derived indexes and external tip-hash provenance; do not create a
   block-hash/state-root cycle. Preserve public immutable-state guarantees.
2. Publish independently calculated preimages/hashes and fixtures before treating
   roots as verified. Cover seeds, Pending/Active/Revoked/Expired outcomes, approvals,
   sequence changes, absence versus height/ordinal zero, and deterministic ordering.
   Do not copy an unexplained failed Rust result into an expected test vector.
3. Specify a versioned bounded MPH2 diagnostic replay format that retains BLOCK
   GROUPING, including empty blocks, because expiry and activation happen at block
   boundaries. A flat event list or immediate V1 executor is not equivalent.
   Keep exact configuration/bootstrap and trusted parent ancestry explicit. Bind
   replay results to caller-trusted expected context/height/tip/root; a valid old
   log or self-consistent snapshot is not authoritative or fresh by itself.
4. Decode untrusted data with count/length/profile budgets before allocation and
   reject unknown tags/versions, truncation, trailing bytes and inconsistency.
   The state-size bound differs from max_log_bytes: use the documented bounds for
   each format. Structural decoding/invariant checks alone must not grant runtime
   rights. Reconstruct through the same B3 executor and verified bootstrap, or
   compare cache data against a trusted validated state/root before using it.
5. Replay must reproduce records, all tombstones/indexes, sequences, approval
   archives, activation order and final high-water. Wrong signatures, anchors,
   altered grouping, skipped/reordered blocks, wrong config, truncated histories
   and mismatched trusted expectations fail without publishing partial state.
   Real MRB2 headers/ledger authority and durable latest-tip recovery stay MB-02/03.
6. Extend the synthetic state_v2_demo with commitments/replay and add necessary
   adversarial/independent-reference tests. Use only synthetic keys/evidence. Keep
   B3's 64-per-record and max_records-total retained-approval budgets; never evict
   historical identity protections to make replay fit. No money or devices.
7. Run pinned workspace/doc/release checks, existing and new independent references,
   all three examples, formatting of changed files and preservation audits. Publish
   source on the verified branch, inspect exact CI, then update the master/status/
   matrix/next/the-prompt with real source and evidence. Complete MB-01B only when
   its scoped root/replay acceptance is proven, not by moving a progress label.

## B3 boundaries to preserve

MembershipRegistryV2::bootstrap accepts only VerifiedBootstrap and returns a staged
state. Registry.stage_block takes the correct locally trusted immutable parent view
and bounded ordered operations; original state remains unchanged even on success.
StagedMembershipStateV2::bind_to_validated_block only attaches trusted provenance.
It does not verify block/root/PoW/genesis validity. MB-02 must enforce that boundary
when composing accounts, membership and actual blocks. Do not make it a peer API.

Seeds have no continuing override. Enroll uses sequence0 then next1; Approve/Withdraw
consume exact next sequence with checked increment. Terminal keys/evidence/IDs and
all accepted approvals stay retained. Parent-Active and post-operation-Active
reviewers alone contribute; activations wait until the end and follow original
inclusion position. The aggregate approval cap is a documented synthetic resource
choice, not a guarantee that every pending applicant can eventually be admitted.

## Following work

B4 -> MB-02 atomic accounts + membership / TransferV2 / BlockV2 / exact genesis ->
MB-03 locking, durable replay/sync and reorgs with DIFFERENT membership histories.
Legacy easy PoW remains development behavior until human-round MB-06. D-01/D-04
remain open and prevent minting/grants/rewards; they do not block B4. Real privacy,
biometrics, admission policy, hardware acceptance and Gate A/B remain later tasks.

Before every Mirror final response refresh the-prompt and relevant repository
checkpoints with actual progress and exact next action. Read current files, preserve
concurrent work, publish with SHA guards, and verify saved content. Distinguish code
validation from later documentation commits; do not chase self-referential SHAs.
