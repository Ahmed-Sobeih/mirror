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
core/personhood/state/consensus/chain/storage tests, all nine independent reference
checks (including `tools/reference_storage_v2.py --check` and
`tools/reference_append_storage_v2.py --check` and
`tools/reference_replacement_storage_v2.py --check`) and all eleven synthetic examples
(three personhood examples, core block_v2_demo, state ledger_v2_demo,
chain chain_v2_demo/fork_choice_v2_demo and storage
storage_v2_demo/append_v2_demo/recovery_v2_demo/replacement_plan_v2_demo). Include
`cargo test -p mirror-storage --release --locked` and
all four storage examples in the gates.
Pin CI toolchain/dependencies; do not print secrets. Distinguish written code,
actual passing tests, published branch, PR and merge status. Release reruns are
not new tests. Verify exact source commits, not just an older green check.

## Roadmap execution — MIRROR-PLAN-001

MASTER_PLAN.md is the staged roadmap and progress tracker to device integration.
NEXT_SESSION.md points to the next unfinished task: currently MB-03E2.
MB-01A design, MB-01B/B1–B4 and MB-02A/B/C are verified components. B4 supplies
MPS2/untrusted caches and trusted grouped MPH2 replay. MB-02A supplies transfers,
unified operations and bounded MRB2 codecs/roots. MB-02B supplies atomic full-order
accounts/membership execution and combined roots. MB-02C now supplies pinned MNM2
manifests, actual block/genesis/work/root validation, private accepted ancestry and
complete validated MRB2 replay. MB-02 is complete within its in-memory component
scope. MB-03A implements immutable selected-chain storage, explicit initialization,
exclusive reopening, shared inspection and complete checked reconstruction.
MB-03B adds a separate append profile, bounded one-child publication, write fencing
and full replay/resynchronization of clean selected states. MB-03C adds explicit
bounded recovery of recognized one-child staging under independent parent/candidate
pins, exact child bytes and an explicit completion/retention action. MB-03D adds
pure comparison of complete accepted branches under independently selected network
identity, fixed-target normalized height scores and keep-current ties. MB-03E1 adds
separate MSR2 profile-3 metadata and checked pure replacement preparation. Next is
MB-03E2: freeze and implement durable replacement layout, publication and recovery,
followed by fresh-peer reconstruction. E1 supplies no store or filesystem mutation.
MB-03 as a whole and the persistent V2 node/API remain unfinished; current
source/test/publication evidence belongs to the checkpoint files.
Read docs/LEDGER_MEMBERSHIP_V1.md, docs/CONFIG_V2_ENCODING.md,
docs/OPERATIONS_V2_ENCODING.md, docs/CORE_V2_ENCODING.md,
docs/LEDGER_STATE_V2.md, docs/CHAIN_V2.md, docs/FORK_CHOICE_V2.md, docs/STORAGE_V2.md,
docs/STORAGE_APPEND_V2.md, docs/STORAGE_RECOVERY_V2.md, docs/STORAGE_REPLACEMENT_V2.md,
docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md,
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
identify a branch; they do not discover latestness. MB-03D's separate comparator
ranks two accepted branches but does not supply global latestness or finality.
Preserve strict child timestamp > parent and configured literal bits; no wall-clock
ceiling exists in this synthetic profile. Preserve 4,096 blocks, 16 MiB complete
MRB2 and 1,048,576 cumulative state/config items, including the exact charge formula
in CHAIN_V2.md. No pruning, cap bypass or silent profile change in storage/recovery.
Preserve MB-03A's separate explicit initialization, exclusive open and shared
read-only inspection APIs. MSS2 is exactly 232 bytes: encoding version 2, storage
profile 1, generation zero only. This immutable profile has no append, replacement
or repair API. MB-03B's separate AppendStoreV2 uses exact 280-byte MSU2 version 2,
profile 2, with an outer generation and the unchanged embedded MSS2 snapshot.
Generation starts at zero at any accepted initialization height and advances once
per accepted append; it never exceeds selected height. Preserve exact MNM2/MRB2
files and all prior wire bytes. Checksum validity is not selection authority:
independently retained network, full checkpoint and, for profile 2, generation
pins remain mandatory on every reopen. A local generation is not global freshness.

Both V2 profiles use Linux nonblocking kernel locks on the root directory inode for the
handle lifetime. All processes must cooperate; parent directories and the local
owner are trusted. Never rename/remove/replace a live root inode, infer ownership
from a PID/lock-file payload, or claim hostile same-user race protection. WSL,
DrvFS and other untested filesystems must not be described as verified. Preserve
bounded metadata/enumeration and length-plus-one sentinel reads, the profile-1
19,559,084-byte successful encoded-payload ceiling, and all separate replay/resource
caps. Profile 2's larger selector makes its corresponding successful selected-read
ceiling 19,559,132 bytes. Neither ceiling is a total resident-memory limit or the
combined I/O of preflight, publication and reconciliation.

Closing a handle releases its own descriptor; inherited fork descriptors may
briefly prolong the kernel lock until they also close. Keep subprocess ownership
tests isolated from unrelated test forks while retaining actual concurrent child
checks. Do not hide failed ownership assertions with sleeps or production retries.

Initial selection is published only after prepared files and directories are
synchronized. AppendStoreV2 stages through actual ChainV2 acceptance, rechecks its
complete selected disk history, synchronizes the new block and directory, stages
the new selector, compares the old selection, renames the marker, and synchronizes
the stable root before advancing memory. Disk discrepancy or I/O failure fences
normal chain/selection access and further writes. last_confirmed_checkpoint then
reports only the last successful memory state, not a current-disk assertion.
Writer reopen and consuming reconcile require full replay and resynchronization;
they accept only a clean complete layout and never release/reacquire ownership
during consuming reconciliation. Normal open/inspect/reconcile still reject and
preserve all staging. MB-03C's separate explicit recovery requires independently
retained parent and candidate checkpoints, the exact child, and Complete or
RetainParent. Fully replay the parent and accept the child for either action before
mutation. Only exact authorized prefixes may be extended without truncation or
removed while retaining the still-selected parent. A pending selector requires a
complete candidate block; a selected candidate requires a complete clean layout.
Never roll it back via RetainParent. Missing/corrupt selected markers, interrupted
initialization, unknown bytes and other layouts reject and remain untouched.
Prefix equality does not prove origin; the trusted caller request authorizes these
specific paths. Keep the same root and exclusive lock throughout consuming recovery,
revalidate/synchronize before returning a writer, and return no handle on error.
Preserve the 19,559,412-byte distinct encoded-input ceiling and enforced conservative
156,508,096-byte per-call payload-read allowance, including comparisons/sentinels.
These are not RSS limits. No format or journal was introduced. Do not silently
promote pending metadata, overwrite foreign/legacy data, or discard accepted history.
Reuse ChainV2's complete MRB2 replay for validity, including all transfers and
membership effects. No cached state or stored counter can replace reexecution
and its resource accounting; in-memory rollback does not establish crash recovery.

Preserve MB-03D's accepted-type comparison boundary: independently check both
branches against the selected C/genesis before scoring or ancestry lookup. Every
accepted block has the same configured literal target, so normalized score is
post-genesis height in 0..=4095. Strictly greater candidate height wins; equal
scores keep current. This is a development rule, not equal-human consensus or a
global tie breaker. At finite caps or terminal timestamps ties may not resolve.
Find the common ancestor from accepted hashes, not account/member roots or peer
assertions. Keep the private borrowed comparison and selected whole-chain reference;
never graft accounts, membership, permanent records, approvals or counters across
branches. The comparator allocates no collection, replays no state and touches no
disk. Copied choices/checkpoints are diagnostics, not durable publication tokens.
MB-03E2 must revalidate identity, current selected endpoint and exact candidate under
the held store lock, with separately specified publication/interruption/retry rules.
Never reinterpret MSU2 generation as branch score or replace the locked root inode.

MSR2 is exactly 280 bytes: version 2/profile 3, outer local generation, unchanged
232-byte MSS2, and the MIRROR_STORAGE_REPLACEMENT_SELECTION_V2 checksum domain.
It is separate from MSS2/MSU2 and has no implemented disk layout/store/recovery API.
Decode only against independently retained full endpoint and generation pins.
Generation starts at zero at any accepted height and advances once per strictly
preferred whole-branch publication, not per skipped block; generation <= height.
An equal-height/human-weighted replacement rule needs an explicitly reviewed profile.
Generation does not establish publication history or freshness. Preserve nine
independent reference cases and all previous fixture bytes.

prepare_replacement_v2 first checks both network identities via the comparator,
then independent current endpoint, selector endpoint/generation, exact canonical
current metadata, strict preference and checked generation/candidate metadata.
Endpoint equality and valid checksums cannot substitute for the full current
manifest checksum/length/block-byte comparison. The private plan immutably borrows
both accepted histories and keeps the exact candidate manifest and complete ledger.
It encodes two bounded manifests but does not clone/reexecute chains or touch disk.
A copied selector or plan is not a durable permit; E2 must revalidate under its
held stable root lock. Current bounds make GenerationOverflow defensive/unreachable
for valid inputs, not an exercised valid-chain transition. E1 leaves retained-slot
layout, cleanup, write fencing and explicit replacement recovery to E2.

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
