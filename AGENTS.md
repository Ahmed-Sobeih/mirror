# Mirror development instructions

Read CURRENT_STATUS.md, MASTER_PLAN.md, NEXT_SESSION.md and docs/DECISIONS.md first.
Verify the live default branch, active PRs and recent commits; old chat checkpoints
are not authority over newer verified work. Preserve local unpushed changes.

Use reviewable feature branches and exact base commits. Do not reset main, delete
runtime data, rewrite old genesis, or silently change consensus encodings. The
user prefers work directly in GitHub, not repeated fragile paste/replace scripts.

No founder premine/extra allocation. No privileged marketing/airdrop reserve.
First 1,000 qualifying humans receive the same 10-MRY bootstrap allocation,
including the founder. Ordinary allowance is 0.2 MRY per qualifying historical
population maximum. Combined bootstrap ceiling is unresolved: no assumed formula.

Raw OR encrypted biometric archives, personal identifiers and real keys must
never be committed. Tests use synthetic evidence/public development keys only.
Do not claim signatures/evidence commitments prove biological uniqueness or
anonymity. Device selection is outside the current blockchain task.

Existing fixed-PoW consensus remains a development foundation, not the final
human-weighted design. Library personhood tests are not deployed PoH. Record
unimplemented parts and trust assumptions explicitly. Never claim mainnet-ready,
crash-proof, audited, or unconditionally immutable from passing unit tests.

Run `cargo test --workspace --all-targets --locked`, documentation tests, release
core/personhood/state/consensus/chain tests, all six independent reference checks
and all six synthetic examples (three personhood examples, core block_v2_demo,
state ledger_v2_demo and chain chain_v2_demo).
Pin CI toolchain/dependencies; do not print secrets. Distinguish written code,
actual passing tests, published branch, PR and merge status. Release reruns are
not new tests. Verify exact source commits, not just an older green check.

## Roadmap execution — MIRROR-PLAN-001

MASTER_PLAN.md is the staged roadmap and progress tracker to device integration.
NEXT_SESSION.md points to the next unfinished task: currently MB-03A.
MB-01A design, MB-01B/B1–B4 and MB-02A/B/C are verified components. B4 supplies
MPS2/untrusted caches and trusted grouped MPH2 replay. MB-02A supplies transfers,
unified operations and bounded MRB2 codecs/roots. MB-02B supplies atomic full-order
accounts/membership execution and combined roots. MB-02C now supplies pinned MNM2
manifests, actual block/genesis/work/root validation, private accepted ancestry and
complete validated MRB2 replay. MB-02 is complete within its in-memory component
scope. Next is explicit V2 storage identity, locking and recovery contract, then
durable coherent publication/synchronization/reorganization within MB-03.
Read docs/LEDGER_MEMBERSHIP_V1.md, docs/CONFIG_V2_ENCODING.md,
docs/OPERATIONS_V2_ENCODING.md, docs/CORE_V2_ENCODING.md,
docs/LEDGER_STATE_V2.md, docs/CHAIN_V2.md, docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md,
docs/THREAT_MODEL.md and docs/MEMBERSHIP_TEST_MATRIX.md.
Coverage is scoped: B2 context/signature success does not establish reviewer rights,
target ownership/status or current sequences. Never use it alone to activate a
membership. The trusted parent view must come from locally validated stable ancestry.

B3 only stages bounded immutable membership state. bind_to_validated_block attaches
trusted provenance; it does not verify a real header/root/PoW/genesis. ChainV2 now
establishes that association through actual validation; lower-level binding remains
a trusted boundary. Keep tip hash outside committed state to avoid cycles.
Preserve B3 expiry/tombstone/sequence/parent-set/end-block timing and explicit approval
budgets (64 per record; aggregate retained <= configured max_records). Never evict
history to recover capacity. B4 replay retains block grouping and empty blocks;
a flat event list or structurally coherent snapshot is not trusted canonical history.
MPS2 decode returns an untrusted wrapper without runtime conversion. MPH2 requires
trusted stable per-height hashes/exact group digests/roots and expected C/height/tip/root;
do not derive this authority from the log or peer. Diagnostic group commitments are
not MRB2 operation roots. Preserve the 4,096-group/1,048,576-state-item replay budgets,
including configuration-copy charges, until a documented versioned replacement.

MB-02 mixed blocks must preserve original full-block operation positions, including
transfers. Do not filter transfers into membership-only stage_block or MPH2 and
silently change inclusion indices/activation order. Preserve the implemented consuming
indexed builder, documented full-block budgets and frozen prior bytes. Core now
uses personhood; personhood depends only on crypto. Preserve this acyclic direction.
MB-02A codecs/body integrity are not actual ledger/block acceptance. Body integrity
checks configuration agreement, the operation root and signatures; it omits transfer
policy/anchor/window authorization, balances/sequences, ledger roots and
parent/time/work/genesis checks. Compose the appropriate validators explicitly.
MPS2 now bounds runtime inclusion indices by max_operations, not the membership
count cap; this explicit range extension changes no layout/domain or snapshot
authority. The membership projection charges omitted transfers exactly as 4+243
bytes. Review this cross-crate contract before adding a new kind or variable size.
Keep the existing address namespace and separate account/member sequences.

Reuse LedgerV2::stage_operations on complete original bodies. It stages both
components atomically after full preflight and parent binding; late failure must
leave both unchanged. Preserve explicit zero/spent account records and initial
synthetic-supply conservation, positive zero-fee transfers and funded self-transfer
sequence semantics. There is no mint/grant/reward API or new membership spending
restriction. bind_to_validated_block is only trusted provenance attachment.
ChainV2 validates the actual header/genesis/parent/height/time/target/work/body and
resulting ledger root before binding or publishing state. Full MRB2 replay uses
its private accepted candidate-branch ancestry, never peer assertions or MPH2.
MNM2 decode requires externally selected C/genesis and every seed proof. Complete
replay requires external C/genesis/height/tip/ledger-root selection. These pins
identify a branch; they do not discover latestness or supply global fork choice.
Preserve strict child timestamp > parent and configured literal bits; no wall-clock
ceiling exists in this synthetic profile. Preserve 4,096 blocks, 16 MiB complete
MRB2 and 1,048,576 cumulative state/config items, including the exact charge formula
in CHAIN_V2.md. No pruning, cap bypass or silent profile change in storage/recovery.
MB-03 must define separate explicit initialization/open/read-only inspection,
single-writer ownership, bounded file reads and coherent accepted-tip publication.
Ambiguous I/O requires reconciliation before further writes. Reuse ChainV2 for
validity; do not install cached state as authority, silently initialize legacy
directories or mistake in-memory rollback for crash recovery.

Preserve V1 APIs/encodings and frozen MCF2/B2 vectors. New signed/root byte semantics
require explicit versioning/documentation and independent reference calculations.
Use docs/DEVICE_INPUT_CONTRACT.md for the private capture interface, not as a claim
of existing hardware support. Later tasks remain planned.

For every completed task record implementation commit, acceptance evidence/CI,
remaining defects and next subtask in the master plan and checkpoint files.
Do not mark integration complete because standalone tests pass. Gate A is tested
software readiness with synthetic inputs; Gate B additionally requires actual
device/capture/matching evidence. Neither is a mainnet security guarantee.

User policy decisions in docs/DECISIONS.md override proposed technical defaults.
D-01 through D-06 are scoped decision gates, not reasons to stall all work.
Continue dependency-ready work when a package is blocked. Do not choose a monetary
exception formula or collect real biometrics merely to pass a test.

## Living continuation prompt — explicit user requirement

The root `the-prompt` is the user's copy/paste handoff into a fresh conversation.
Keep it standalone and current during the conversation, not only at final handoff.
Before the final response to EACH Mirror-related message, refresh it on the verified
active branch. Include decisions, actual completed work, observed test results,
publication/PR state, unresolved questions and the exact next unfinished action.
Distinguish user decisions from proposals. On discussion-only turns preserve the
last verified checkpoint and next task rather than inventing progress.

Read current files before editing, reconcile newer work and use current content
SHAs/expected-head guards. Verify publication. Keep master/status/next/decisions
consistent. Include no secrets, biometrics or unrelated personal/project details.
A stale prompt never overrides newer live branches/PRs/CI. If unmerged, read feature
branch records, not main. After a merge/branch change update links; never reset newer
work or merge only to publish the handoff. Record tested source separately from
documentation updates; do not label an untested head tested or chase self-referential
SHAs. Work occurs in the current conversation, not promised background execution.
If publication fails, state that and provide the continuation text instead.
At session switch provide the direct live file link for copying its full contents.
