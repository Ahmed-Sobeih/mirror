# Next session — MB-02C

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-20**.
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md, docs/CORE_V2_ENCODING.md,
docs/LEDGER_STATE_V2.md, docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md,
docs/THREAT_MODEL.md, docs/MEMBERSHIP_TEST_MATRIX.md and the latest the-prompt.

## Recover live state

Verify default/active branches, PR #1, recent commits/comments and exact CI.
Active branch `work/personhood-register-v0.1`; PR #1 remains open/unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`. Use feature-branch records,
not stale main. Preserve concurrent/unpushed work and runtime data. Never reset
to a copied checkpoint or merge merely to publish handoff documents.

Latest implementation `420d7114ddf782469a6ace6ba0a2c4076fb93aaf`, tree
`6871ad0fbc178bd76262a26416a7d255d9fcbd66`. Exact-source push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619155972
CURRENT_STATUS.md records inspected evidence; check subsequent documentation-head
CI separately. Local Rust 1.98.0 gates passed 462 workspace tests, 3 doctests, core
release 121, personhood 234 plus the same 3 doctests, state 49, all 5 references/examples,
formatting and preservation audit. Release reruns do not add unique tests.
MB-01A, MB-01B/B1–B4, MB-02A and MB-02B are complete in their component scope.

## Reuse the completed components

Core provides immutable SignedTransferV2, SignedOperationV2, BlockHeaderV2 and
BlockV2 with count-bound operations_root_v2 and validate_operation_limits_v2.
Personhood provides immutable configuration/bootstrap and bounded indexed
membership staging. State provides LedgerV2 and StagedLedgerV2, atomic full-order
execution, checked account sequences/balances and canonical combined roots.
Dependencies remain acyclic: crypto <- personhood <- core, composed by state.

LedgerV2::bootstrap requires VerifiedBootstrap. stage_operations takes the actual
full ordered operation list and a stable trusted parent view. Its result has no
partial-state setters; failure leaves the immutable parent unchanged. Only after
all actual block checks succeed may the trusted outer layer call
StagedLedgerV2::bind_to_validated_block. That method validates no block itself.

Full indices include transfers. MPS2 runtime inclusion indices are bounded by
max_operations, not max_membership_operations. Preserve the documented range
extension, original bytes and untrusted-cache boundary. MPH2 remains membership-
only, keeps its existing budgets/vectors, and rejects transfers. Do not filter
MRB2 histories into it. Read LEDGER_STATE_V2.md before integrating execution.

## Exact next slice — MB-02C actual V2 block authority

Implement a bounded in-memory chain/acceptance layer for the explicit synthetic V2
profile. Preserve legacy APIs, runtime data and genesis; do not silently initialize
or migrate existing directories. No devices, money issuance or new user policy.

1. **Specify genesis and manifest authority first.** Freeze explicit configuration,
   seed-proof requirements, exact genesis header/body and expected genesis hash.
   Bootstrap roots must be independently reproduced. Decide and document precisely
   which metadata the trusted caller supplies and which bytes the node verifies.
   A different otherwise-valid genesis nonce must fail an exact manifest pin.
   Do not trust a peer's Hello or a supplied state snapshot as this manifest.
2. **Validate complete blocks.** Bound decode/body work, check network/version/C,
   height/parent, deterministic timestamp policy, configured target and PoW, ordered
   signed-operation commitment, and every execution rule. Stage the real operation
   body through LedgerV2. Compare its full ledger root with the header before
   binding its actual block hash or publishing any candidate state.
3. **Provide real trusted ancestry.** Implement ParentChainViewV2 from the validated
   immutable candidate branch, including its own canonical recent anchors. Avoid
   mutable views, peer-provided authority or a global tip that changes mid-execution.
   Keep exact parent binding, checked height arithmetic and empty-block processing.
4. **Replay actual accepted MRB2 histories.** Start from pinned verified bootstrap,
   validate every block with the same executor, and reproduce accounts, membership,
   counters, roots and tip. Replay full bodies, never an extracted membership log.
   Wrong/intermediate/final roots, altered operations, reordered history and foreign
   configuration/genesis must fail without publishing partially rebuilt state.
5. **Test authority failures and competing histories.** Include an otherwise valid
   block with a wrong final ledger root (the remaining T27 case), alternate valid
   genesis nonce, invalid work/target/height/parent, changed membership and balances,
   and anchors valid on one fork but absent on another. Independent replicas must
   reproduce the same actual block/ledger roots. Distinguish branch validity from
   final human consensus and durable canonical-tip selection.
6. **Publish a bounded verified slice.** Add independent exact-genesis/block/ledger
   vectors before blessing outputs, an explicit synthetic accepted-block example,
   relevant boundary tests and CI. Run required workspace/docs/release/reference/
   example/preservation gates, inspect exact-source CI, then refresh and read back
   all handoff documents. If splitting this slice, name the substep and leave the
   remaining MB-02C acceptance criteria clearly incomplete.

MB-03 follows with writer locking, durable consistent publication/recovery, storage
metadata, P2P and membership-aware reorganizations. In-memory accepted blocks do
not establish crash safety, latest-tip discovery or a deployed decentralized network.

## Constraints that remain unchanged

No founder bonus, privileged marketing reserve, grant/mint API, nonzero development
fees or reward allocation. Keep explicit zero account entries and spent account
sequence history; no eviction to regain capacity. Preserve separate account/member
sequences, B3 timing/eligibility and retained-history limits, and all frozen formats.
D-01/D-04 economic decisions remain open and do not block this zero-fee/no-mint slice.

A signature or opaque evidence commitment does not prove human uniqueness. Public
membership/review metadata remains linkable. Raw or encrypted biometric archives,
real keys, device secrets and personal records never belong on-chain or in GitHub.
Real capture and device purchasing are outside this task. Gate A/B remain unmet.

Before every Mirror final response update the-prompt and relevant checkpoints with
actual results and exact next work, preserve newer source, publish with head guards
and verify readback. Record source evidence separately from documentation commits.
