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

Use Rust 1.98.0 and locked dependencies. Run
`cargo test --workspace --all-targets --locked`, documentation tests, release
core/personhood/state/consensus/chain/storage/protocol/network/node tests, all twelve
independent reference checks (including `tools/reference_storage_v2.py --check`,
`tools/reference_append_storage_v2.py --check`,
`tools/reference_replacement_storage_v2.py --check`,
`tools/reference_peer_history_v2.py --check`,
`tools/reference_peer_session_v2.py --check` and
`tools/reference_node_control_v2.py --check`) and all sixteen synthetic examples
(three personhood examples, core block_v2_demo, state ledger_v2_demo,
chain chain_v2_demo/fork_choice_v2_demo and storage
storage_v2_demo/append_v2_demo/recovery_v2_demo/replacement_plan_v2_demo/
replacement_store_v2_demo/replacement_recovery_v2_demo, network
history_receiver_v2_demo/peer_session_v2_demo, and node node_control_v2_demo).
Include `cargo test -p mirror-storage --release --locked`,
`cargo test -p mirror-protocol -p mirror-network --release --locked`,
`cargo test -p mirror-node --release --locked`, all six storage examples,
`cargo run -p mirror-network --example history_receiver_v2_demo --locked`,
`cargo run -p mirror-network --example peer_session_v2_demo --locked` and
`cargo run -p mirror-node --example node_control_v2_demo --locked` in the gates.
Pin CI toolchain/dependencies; do not print secrets. Distinguish written code,
actual passing tests, published branch, PR and merge status. Release reruns are
not new tests. Verify exact source commits, not just an older green check.

## Roadmap execution — MIRROR-PLAN-001

MASTER_PLAN.md is the staged roadmap and progress tracker to device integration.
NEXT_SESSION.md points to the next unfinished task: currently MB-04A2, finite
V2 init/status/sync-once/explicit-recover operator commands. MB-04A1 is a verified
durable control/restart component; persistent serving, scheduling, pool and wallet
remain later MB-04 work.
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
separate MSR2 profile-3 metadata and checked pure replacement preparation. E2a adds
the actual bounded replacement store, checked cleanup, fencing and clean reconciliation.
E2b supplies explicit interrupted replacement recovery; MB-03E/E2 are complete
within local store component scope. MB-03F1 supplies separate MPN2 framing,
checked header admission and a consuming complete-history receiver under independent
network selection. MB-03F2 adds finite MPQ2-request/TCP sessions, absolute deadlines,
cumulative owner budgets, mandatory terminal EOF and explicit whole-candidate
adoption through existing storage/recovery ownership. MB-03F2 is
VERIFIED_COMPONENT — FINITE TCP / DURABLE ADOPTION; MB-03F is VERIFIED_COMPONENT —
BOUNDED FRESH-PEER RECONSTRUCTION; MB-03 is VERIFIED_COMPONENT — V2 STORAGE,
EXPLICIT RECOVERY AND FINITE PEER SYNCHRONIZATION. Local acceptance, exact source
publication and exact-source CI evidence remain separate in CURRENT_STATUS.md.
E1 remains pure; E2a never treats its saved plan as a durable write permit.
MB-04A1 adds NodeControlV2's separate durable confirmed record and exact
old/candidate intent, clean restart, read-only status and explicit recovery.
It is VERIFIED_COMPONENT — DURABLE V2 CONTROL / EXPLICIT RESTART RECOVERY.
F2's own recovery inputs remain memory-only; A1 supplies a separate persistent
wrapper. Finite operator commands are next in MB-04A2; daemon/reconnect/multi-peer
scheduling, pool and wallet remain later MB-04. Gate A/B remain unmet. Current
source/test/publication evidence belongs to the checkpoint files, not a task-count
percentage. A1 acceptance records 1,083 runtime tests, eleven doctests, twelve
references and sixteen examples; release reruns are not additional unique tests.
Read docs/LEDGER_MEMBERSHIP_V1.md, docs/CONFIG_V2_ENCODING.md,
docs/OPERATIONS_V2_ENCODING.md, docs/CORE_V2_ENCODING.md,
docs/LEDGER_STATE_V2.md, docs/CHAIN_V2.md, docs/FORK_CHOICE_V2.md, docs/STORAGE_V2.md,
docs/STORAGE_APPEND_V2.md, docs/STORAGE_RECOVERY_V2.md, docs/STORAGE_REPLACEMENT_V2.md,
docs/STORAGE_REPLACEMENT_STORE_V2.md, docs/STORAGE_REPLACEMENT_RECOVERY_V2.md,
docs/PEER_HISTORY_V2.md, docs/PEER_SESSION_V2.md, docs/CANDIDATE_ADOPTION_V2.md,
docs/NODE_CONTROL_V2.md, docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md,
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

All three V2 store profiles use Linux nonblocking kernel locks on the root directory inode for the
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
MB-03E2a revalidates identity, current selected endpoint and exact candidate under
the held store lock. E2b preserves those checks during explicit interrupted recovery.
Never reinterpret MSU2 generation as branch score or replace the locked root inode.

MSR2 is exactly 280 bytes: version 2/profile 3, outer local generation, unchanged
232-byte MSS2, and the MIRROR_STORAGE_REPLACEMENT_SELECTION_V2 checksum domain.
It is separate from MSS2/MSU2. E2a implements its generation-directory store;
E2b supplies separately invoked interrupted staging/cleanup recovery.
Decode only against independently retained full endpoint and generation pins.
Generation starts at zero at any accepted height and advances once per strictly
preferred whole-branch publication, not per skipped block; generation <= height.
An equal-height/human-weighted replacement rule needs an explicitly reviewed profile.
Generation does not establish publication history or freshness. Preserve all twelve
independent reference checks and all previous fixture bytes.

prepare_replacement_v2 first checks both network identities via the comparator,
then independent current endpoint, selector endpoint/generation, exact canonical
current metadata, strict preference and checked generation/candidate metadata.
Endpoint equality and valid checksums cannot substitute for the full current
manifest checksum/length/block-byte comparison. The private plan immutably borrows
both accepted histories and keeps the exact candidate manifest and complete ledger.
It encodes two bounded manifests but does not clone/reexecute chains or touch disk.
A copied selector or plan is not a durable permit; E2 must revalidate under its
held stable root lock. Current bounds make GenerationOverflow defensive/unreachable
for valid inputs, not an exercised valid-chain transition. E1 is pure. E2a now
supplies bounded generation layout, cleanup and fencing; E2b explicit interrupted
replacement recovery is implemented separately.

ReplacementStoreV2 uses exact generation-{generation:020} directories with the
candidate's own manifest and complete history. Clean root is one selected generation
plus selection.v2; mutation permits only current/next generations and one pending
selector. Never infer authority from scanning a higher generation. Reopen/inspect/
consuming reconcile require independent full pins and a clean fully replayed layout.
Do not weaken these operations to delete or promote residue. Replace revalidates
current history under the held lock, stages/syncs/replays the exact candidate,
publishes/syncs the marker, verifies candidate and all old bytes before any old
unlink, removes only exact named unselected files, syncs cleanup, and replays/syncs
clean selection before memory advance. No recursive deletion or locked-root swap.
Disk errors fence APIs; last-confirmed state is memory only. Publication attempt
onward may already select candidate even if cleanup fails. Never return a writer
from unresolved failure. Preserve the 39,118,264-byte distinct payload cap and one
313,011,680-byte per-call read allowance including sentinels. Five full replays on
successful replace retain per-history state/work caps; this is not a total RSS cap.

E2b requires BOTH exact accepted old and candidate histories, independently
retained identity/checkpoints/generations, and explicit completion/retention. Old
checkpoint plus candidate cannot authorize partially deleted old contents. Validate
all input/residue before any mutation. Directory entries can survive as arbitrary
subsets before directory sync; candidate residual contents must be authorized byte
prefixes, old residual files must be exact complete bytes. Pending selector requires
complete candidate; selected candidate must be complete and never rolled back to
old. Keep the same lock throughout consuming recovery, explicit bounded retries,
final replay/sync and return. Unknown/corrupt selectors or artifacts remain rejected
and preserved. Interrupted initialization remains outside the current recovery scope.

Recover with Complete or RetainOld only under the exact borrowed request. Old-selected
candidate staging can be arbitrary canonical prefixes; candidate-selected old cleanup
permits only full exact surviving files. A blocks-present old generation requires
its manifest. Resynchronize selected history/selector/root before any cleanup.
Persist pending absence before candidate retention cleanup, and persist old blocks-
directory absence before old manifest removal EVEN if blocks is already absent.
Never roll back selected candidate. Ordinary opening stays clean-only. Preserve
same-action/action-switch failure retries and the single read allowance, including
length-plus-one charges before existing-prefix payload opens. Full recovery read
bounds: old Complete 293,447,607 bytes / 4 replays; new Complete 176,068,227 bytes / 2;
RetainOld 215,195,249 bytes / 2.
These are encoded-read bounds, not memory/latency/power-loss guarantees. MB-03F1
treats peer endpoints as candidate claims and fully reconstructs actual history under
independent network selection. Preserve all caps and old V1 bytes. MB-03F2 publishes
only appropriately selected accepted candidates through existing storage/recovery
ownership. Do not import a peer's local generation as fork score, freshness or
independent selection authority.

MPN2 is a separate version-2 protocol, with an exact 12-byte header and Begin,
Manifest, Block and End phases. V1 MIRR bytes/APIs remain unchanged with no fallback.
Select C/genesis independently before HistoryReceiverV2::new; never derive network
authority or a trusted ChainReplayExpectationV2 from peer summary/manifest fields.
The 152-byte Begin/End summary is an untrusted finite-transfer claim. Actual
NetworkManifestV2::decode_for and ChainV2::from_genesis/append must validate every
original mixed-operation block and complete candidate ancestry/state.

Use consuming begin_frame before allocating/reading a socket payload. Its checked
PendingHistoryFrameV2::payload_len includes phase, exact/configured lengths and
remaining feasible byte/frame accounting. accept_payload requires that exact length.
All errors consume continuation; do not expose partial state, skip bad frames,
rescan for magic or resume a failed owner. accept_frame is only convenience for an
already supplied complete frame and proves nothing about the caller's allocation.
Every actual block and matching explicit End must pass before finish releases a
ReceivedCandidateV2. End defines this transfer; EOF alone never proves completion.
Further input through the completed receiver rejects. The separate F2 adapter
requires response EOF after a valid End before calling finish and returning its
candidate; all unfinished or failed attempts close and discard their owners.

Preserve 4096 blocks, 16 MiB complete MRB2, 2781636 manifest bytes, 4099 frames and
19641112 total framed-input bytes, plus configured block limits and the unchanged
1048576 cumulative state/configuration-item budget. Wire accounting is
340 + manifest_bytes + block_bytes + 20*block_count; feasible future-byte arithmetic
uses checked u64, including intermediate products too large for 32-bit usize.
The exact 168-byte genesis tightens the configured total. These are encoded-input
and abstract-work caps, not RSS, bandwidth or wall-clock guarantees. The sender
borrows one immutable accepted history and is fused after error/completion.

F1 remains the pure framing/receiver component. F2 adds a separate exact 72-byte
MPQ2/version-2/kind-1 request containing independently selected C/genesis. The
provider independently checks its accepted history and request before responding.
The requester shuts down its write half after the request; the provider requires
request EOF before any response. One unchanged F1 transfer follows, with matching
End and response EOF both required. One trailing byte rejects in either direction.
No multiplexing, fallback, receiver resumption, DNS or implicit reconnect is added.

PeerSessionsV2 is a non-Clone synchronous owner: at most one live connected/accepted
stream, 1..=8 charged attempts, a positive configured cumulative I/O allowance up to
157129488 bytes, and one absolute duration >0 through 60 seconds per attempt. The
owner charges actual positive read/write results, including malformed/partial data
and trailing sentinels; failures never refund bytes or attempts. Before every syscall
it limits buffers to remaining capacity and sets timeout to the original deadline's
remaining duration. Header admission and the remaining payload budget precede
allocation. A one-byte EOF probe requires one unspent byte; zero-byte EOF is not
charged. Short reads/writes and Interrupted errors cannot refresh the deadline.
Check the deadline after semantic processing as well; synchronous validation is not
preempted and this is not a hard CPU/return-latency guarantee. The listener is consumed
and accepts at most one connection. These limits belong to one explicitly created
owner, not all process sockets, the kernel backlog or a future persistent scheduler.

Adoption consumes a completed candidate and owned writer, independently checks both
histories against the selected network and the current full checkpoint/generation,
and keeps current on ties or shorter histories without filesystem writes. Only a
strict preference creates
AdoptionRecoveryInputsV2: clone the exact old accepted history once, move the complete
candidate, and retain the independent identity and both local checkpoints BEFORE
publication. Storage repeats actual checks under the same lock. Publication failure
retains that lock and both histories without exposing a normal writer. Explicit
consuming recovery keeps the same lock; a recovery failure returns no handle but
keeps root/inputs for an explicit retry that reacquires ownership. No implicit retry
or cleanup occurs. RetainOld never rolls back a selected candidate. Fresh initialization
is a separate explicit operation, never a fallback after open fails. The extra old
clone is not covered by encoded-input ceilings. Adoption has no socket deadline.

F2's exact recovery inputs are memory-only, not a persisted journal or automatic
archive. Real-process transfer/reopen and failure tests verify the finite component;
they do not establish daemon lifecycle, global latestness, human finality, anonymity,
biological uniqueness or Gate A/B completion.

## MB-04A1 control and restart invariants

Read docs/NODE_CONTROL_V2.md before modifying this layer. Control remains separate
from the strict profile-3 store root, under independent C/genesis and trusted local
operator/parent paths. Acquire nonblocking cooperative Linux control ownership
before store ownership; release the store first. The ready owner's store is private.
No returned error exposes an ordinary writer or automatically chooses a retry.

MNC2 is exactly 464 bytes: version 2/kind 1, full checkpoint/generation and unchanged
canonical MSR2 plus a separate checksum. MNI2 has a 952-byte fixed header, both
MNC2 records, two complete unchanged MPN2 histories and a checksum; maximum encoded
intent is 39,283,208 bytes. Use external identity, full replay of BOTH histories,
exact canonical metadata and existing strict preference/generation validation.
Neither checksum nor directory scanning supplies authorization. Preserve the
314,269,392-byte per-call control read allowance, including length-plus-one
sentinels and all comparisons, separate from storage budgets. These are not RSS,
wall-clock or physical power-loss guarantees.

Initialize control explicitly for an already initialized profile-3 store under
independent full pins. Failed open never becomes store initialization or repair.
Status is read-only and reports Missing/Clean/RecoveryRequired or validation errors;
RecoveryRequired describes validated control intent, not actual selected-store
contents. Clean open/inspect must match the complete canonical MSR2 record as well
as the full endpoint/generation after storage replay.

Persist, validate and synchronize complete intent before storage replacement.
Persist/synchronize confirmed selection before removing intent, then synchronize
its removal. Resynchronize exact intent, control root and parent before explicit
restart recovery mutates storage. Clean writer open resynchronizes confirmed
control/root/parent, including after a visible intent unlink whose sync failed.
Errors after intent retirement may leave an already confirmed clean state: do not
claim every error necessarily leaves pending intent. Reinspect through the defined
read-only status and explicit open/recovery paths without guessing success.

Unknown/corrupt/overlong/unsafe control and every intent.pending reject and remain
preserved. Partial control initialization is unsupported. A confirmed.pending
prefix of 0..=464 exact candidate bytes is recognized only with confirmed-old plus
complete intent. That state requires explicit Complete and a strict clean candidate
store; do not use it to repair an old-selected or residual store. Append only its
exact missing suffix without truncation. Confirmed-candidate plus intent also
requires strict candidate opening and Complete. Old-confirmed plus intent without
confirmation staging delegates explicit Complete/RetainOld to existing recovery.
A selected or confirmed candidate cannot roll back. Never weaken storage's accepted
layouts, delete unknown data or add control artifacts inside the store root.

## Exact next increment — MB-04A2

Implement finite, separate V2 init/status/sync-once/explicit-recover operator
commands through the accepted A1 control and F2 transport APIs. Keep independent
network/path selection explicit. Initialization is separately requested and never
an automatic fallback after failed opening. Status must not initialize, synchronize,
mine, contact peers or repair. Sync-once uses F2's bounded owner/deadlines and A1's
persisted-intent ordering; unresolved intent requires an explicit recovery command.
Verify useful operator output, failure exits, restart behavior and full whole-branch
state through independent process tests. Do not bundle a daemon, reconnect/multi-peer
scheduler, transaction pool or wallet into this increment. No unresolved monetary,
enrollment or device policy blocks these dependency-ready operator commands.

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
