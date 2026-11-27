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
NEXT_SESSION.md points to MB-04B4, bounded persistent-run lifecycle, beginning
with an explicit design. MB-04B3 adds bounded explicit-peer failover; B2
retains finite rejoin/serve, B1 finite provider/draining shutdown, A2 operator
commands and A1 durable control/explicit recovery. Broader scheduling, pool
and wallet remain later MB-04 work; B4 is not implemented.
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
wrapper. At the A1 checkpoint, finite operator commands were next. MB-04A2 now
implements separate explicit attach-existing initialization, read-only status,
one receive/adoption and explicit recovery through A1/F2. MB-04B1 adds a bounded
fixed-history provider, explicit admission limits and graceful draining shutdown.
MB-04B2 composes one receive, admitted keep/adopt and finite serving with the
same controller and cumulative budget. MB-04 remains IN_PROGRESS; broader
reconnect/multi-peer scheduling, pool and wallet are unfinished.
Gate A/B remain unmet. Current source/test/publication evidence belongs to the
checkpoint files, not a task-count percentage. A1's historical acceptance records
1,083 runtime tests, eleven doctests, twelve references and sixteen examples;
release reruns are not additional unique tests.

### Retained MB-04A2 acceptance

A2 exact-source acceptance passed: 1,137 workspace runtime
tests, including 54 new tests (30 binary tests: 23 parser, four output,
three diagnostic; 24 actual-command process tests), eleven
doctests, twelve references and sixteen examples. The node release suite records
127 runtime tests plus one doctest. Implementation source:
`eea42a403fef2a1442e553df2a6823b1b25da616`; tree: `cda30cd7e8b564a82d09a46b7067012e7112e2dd`; source CI: `45882757484`, job `140125894368`,
checkout `7dbc9921785f52c4c0136c27dc5890f9b0188e2c`; published source-file readback: 9 files.
All 43 source-CI steps passed; full logs and published readback were inspected.
Compatibility verification confirms 153 of 154 retained source, reference and
fixture files byte-identical; the sole retained change is
node Cargo.toml's explicit V1 default-run. Cargo.lock and lower-layer/V1 source,
dependencies, formats and frozen fixtures were unchanged at that checkpoint. See
master section 28 for the A2 checkpoint and separate later documentation-head evidence.

### Retained MB-04B1 source and acceptance

Checkpoint: MIRROR-MB04B1-001 — finite V2 provider and draining shutdown.
Implementation source: `ed631eae05ebeebe2b3a58a433ab8a0cc5753150`;
tree: `dc32d623bf6a8e5e3ab03023b699a729e0c630a1`;
parent checkpoint: `f507827aabcaecda85c02d66f0de6a10db8972ac`.
Source CI: [45904484131](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45904484131),
job `140197823058`.

**MB-04B1 is VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN.**
All **38 local command gates** and **43 source-CI steps** passed. Full source-CI
logs confirm **1,167 runtime tests and eleven compile-fail doctests**, zero failures
and three retained parent-exercised ignored helpers. The actual synthetic checkout
`cdb00671509b4f2876d51bd677ab39d861833396` has no file differences from the implementation
commit. All **17 published implementation/test/contract/dependency/workflow files**
were read back byte-for-byte. Later documentation-head CI is recorded separately
on PR #1; source acceptance does not assert that a future head has passed.

Workspace runtime count: 1,167, including 30 new tests.
Node runtime count: 148; network runtime count: 108.
Required evidence also includes eleven compile-fail doctests, all required release
suites, twelve independent references, sixteen synthetic examples and all 38 local
command gates. Check full exact-source CI logs and published file readback; record
the later documentation-head CI separately without self-referential source claims.

The source publication changes exactly 17 implementation/contract/workflow files.
Of 161 retained source/reference/fixture paths, 153 are byte-identical and eight
change intentionally. The controlled exceptions are the finite provider's node
integration, the additive F2 accepted-stream adapter and shared checked provider
routine, the node's Linux-target edge to already locked libc 0.2.189, and the CI
job timeout increase from 15 to 20 minutes. Dependency package versions, sources
and checksums, all verification commands, V1 behavior, A1/storage/consensus source,
wire/storage/hash formats, references and frozen fixtures remain unchanged.
Preserve these boundaries; B1's explicit exceptions do not authorize unrelated
dependency, protocol or policy changes.

### Retained MB-04B2 source and acceptance

Checkpoint: MIRROR-MB04B2-001 — finite V2 rejoin and serve coordination.
Implementation source: **bbac1622be18c97ab640f6834a158e5378f1d172**.
Tested source tree: **d724df618f6b45429abebaa3c24bb52ad742799b**.
Parent checkpoint: **966286bec62ce422db6bc091c472d74659935344**.
[Source PR CI 45927607784](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45927607784)
passed all **43 steps**. Full job `140272547972` logs were inspected and confirm
**1,201 runtime tests and eleven compile-fail doctests**, with zero failures.
The actual synthetic PR checkout `e00fbb793f5eb417cc0f0cda1ed52443c5cef34d` has no file differences from
the implementation commit. All **13 published implementation, test and
contract files** were read back byte-for-byte against locally tested content.
Source acceptance: **PASSED — all 38 local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; this
source acceptance does not assert that a future head is tested. PR #1 remains
OPEN and UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,201 workspace/all-target runtime tests**, zero failures: 1,167 retained
  plus **34 new** (six parser, one diagnostic and
  **27 actual rejoin-process tests**). Parameterized cases,
  subprocesses and release reruns are not extra unique tests.
- **Eleven compile-fail doctests**, unchanged; three retained subprocess helpers
  remain intentionally ignored in ordinary enumeration and exercised by parent tests.
- Required release suites: personhood 234 runtime + 3 doctests, core 121, state 49,
  consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network **108 + 2**,
  node **182 + 1**.
- All **twelve independent Python references** and **sixteen synthetic examples**;
  scoped formatting, whitespace and allocation/runtime-data checks also passed.

The 27 added actual-process tests cover complete seed `serve` -> `rejoin-serve`
-> third `sync-once` transfer with independent directories and an explicit second
restart/rejoin cycle. Fresh strict reopening compares full histories, account
bytes, membership records/bytes, retained approvals, sequences, population and
resource counters. The seed keeps local generation 0 while adopters advance to 1;
the restart keeps those local generations and complete state unchanged. The
preferred fork retains equal account bytes but different membership state.

Further cases cover effect-free parsing; local missing/pending/corrupt/mismatched/
busy refusal before contact or bind; identical/shorter/tied keep-current; shared
attempts and actual bytes; the exact terminal-EOF capacity boundary; rejected and
partial serving; malformed/trailing/incomplete/wrong-network sources; idle/active
service drain; original deadlines and admission timing; post-adoption bind/output
uncertainty; and selected/confirming/confirmed no-rollback fixtures. Constructed
pending layouts are explicitly separate from actual interrupted processes.

Bounded Linux observations identify the actual child's socket endpoints, inode,
descriptor and blocked syscall. SIGINT/SIGTERM during complete or partial receipt
permit original-deadline completion/failure; validated completion under a prior
stop discards the candidate before adoption. Directory descriptors and competing
busy commands are checked across controlled receive/service phases; source review
also verifies direct retained-controller flow. These observations do not claim
to sample every hypothetical microscopic ownership gap.

The adoption test extends a valid synthetic history by 128 mined empty blocks.
An inotify event only prompts SIGSTOP. While the actual child is verified stopped,
the test requires the complete canonical intent, independently validates both
histories and checkpoints, confirms both owners remain held and no listener exists,
then verifies caught, unblocked SIGTERM is pending before SIGCONT. Adoption finishes,
retires intent and reopens the candidate at generation 1, followed by normal shutdown
despite an occupied listen port. This observation passed in debug and release; it
establishes signal generation during live adoption, not a particular fsync boundary.

A full-pipe case observes the child blocked on final stdout while its listener is
gone and fresh status plus strict writer opening already succeed. SIGKILL cases
separately preserve the old branch during reception and the new branch during
adopted service. Existing A1 publication-failure/process tests remain the evidence
for their exact durable substeps. Zero-attempt stop before initial request is not
artificially forced by this new process suite; that branch retains source-level
lifecycle evidence. Process tests use Linux IPv4 loopback; numeric IPv6 parsing
has CLI coverage. No test adds a production hook, permission change, hidden retry,
power-loss guarantee or untested-filesystem claim.

**160 of 166 retained source/dependency/workflow/reference/fixture
paths remain byte-identical**; 6 existing private V2 command
modules change deliberately. New code adds the rejoin composition and its process
suite/helper. Earlier tests, lower-layer/V1/A1/F2/storage/consensus/monetary source,
public APIs, Cargo manifests and lockfile, all twelve references, frozen fixtures,
workflow commands and its 20-minute job budget remain unchanged. No wire/storage/hash
encoding, enrollment/device policy, dependency selection or runtime data changes.

### Current MB-04B3 source and acceptance

Checkpoint MIRROR-MB04B3-001.

Implementation source: **3311151ab627b3512abed47be20adea2f0e6d66a**.
Tested source tree: **fc238ae2df95b7b2b3df2457d06f739048812c7e**.
Parent checkpoint: **9ca7d99681877d014ae64aab31cb5eb22a957c35**.
[Source PR CI 45933868084](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45933868084)
passed all **43 steps**. Full job `140292436668` logs confirm **1,242 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`0cada2958afe985bb5fcd3b683c57a40b01042ce` has no file differences from the implementation source.
All **12 published implementation/test/contract files** were read back byte-for-byte.
The later checkpoint updates nine status/handoff records and clarifies one
contract compatibility wording: existing operational records remain unchanged; the help
catalogue gains the new command and options. It changes no runtime/test file,
and the other 198 files frozen during source acceptance remain byte-identical.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; this
source record does not assert that a future documentation head is tested.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed with **199 frozen noncheckpoint files unchanged**:

- **1,242 runtime tests**, zero failures: 1,201 retained plus **41 new** — six
  parser, eight typed-policy, three output, one error-context, one diagnostic-fallback
  and **22 actual-process tests**. Scenarios, subprocesses and release reruns are
  not additional unique test functions.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers are exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **223+1**. All **twelve references** and **sixteen examples** passed.
- Scoped formatting, whitespace and allocation/runtime-data checks passed.

Separate refused-first and malformed-first tests run a real seed executable,
the B3 executable and a third requester with independent roots and a second
explicit restart cycle. Strict reopening compares complete histories, accounts,
membership records/bytes, approvals, sequences, population and resource counters;
the preferred fork has equal accounts but different membership. First validated
identical/shorter/tied candidates stop acquisition and leave later peers untouched.

Actual cases cover all four peers failing, eligible malformed/wrong-network/
incomplete/trailing/missing-EOF transfers, and a constructed nested manifest
configuration error that remains fatal without later contact. Fixed invalid-header
exchanges establish 84 charged bytes (72 request + 12 consumed header). Exact
EOF-capacity and payload-larger-than-positive-remainder cases verify shared bytes;
last-allowed receive succeeds without binding an occupied address. Later successful
acquisition and service retain distinct earlier rejection evidence.

Bounded Linux observations identify the child socket endpoints/inode/FD and blocked
syscall before SIGINT/SIGTERM. They verify original deadlines, failed-receive stop,
later validated-candidate discard, both exact directory descriptors across two
receives and service, and busy competing commands with no outgoing contact.
Bind/closed-output failure preserves adopted state and possible-completion context.
Blocked final stdout releases the listener/owners before the write finishes;
SIGKILL during a partial second receive preserves old state. Unused listeners
remain reserved until exit when absence of later contact is asserted.

The retained B2 live-adoption signal and service-drain/no-rollback tests exercise
the same extracted post-receive path; they are separate retained evidence, not
22 additional B3 scenarios. No new production hook, permission change, induced
OOM, hidden retry, power-loss or untested-filesystem guarantee is introduced.
Pre-first-request zero-attempt and brief between-request shutdown branches retain
source-level evidence. Process tests use Linux IPv4; numeric IPv6 parsing is separate CLI coverage.

**164 of 169 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Five existing private command modules change: args, errors,
main, output and rejoin. New files add the coordinator, process suite/helper and
contract. All earlier test functions and process helpers, public/lower-layer/V1/
A1/F2/storage/consensus/monetary source, Cargo manifests/lockfile, references,
fixtures, workflow commands and 20-minute budget remain unchanged. No wire,
storage or hash encoding, enrollment/device or monetary policy changed.

Read docs/LEDGER_MEMBERSHIP_V1.md, docs/CONFIG_V2_ENCODING.md,
docs/OPERATIONS_V2_ENCODING.md, docs/CORE_V2_ENCODING.md,
docs/LEDGER_STATE_V2.md, docs/CHAIN_V2.md, docs/FORK_CHOICE_V2.md, docs/STORAGE_V2.md,
docs/STORAGE_APPEND_V2.md, docs/STORAGE_RECOVERY_V2.md, docs/STORAGE_REPLACEMENT_V2.md,
docs/STORAGE_REPLACEMENT_STORE_V2.md, docs/STORAGE_REPLACEMENT_RECOVERY_V2.md,
docs/PEER_HISTORY_V2.md, docs/PEER_SESSION_V2.md, docs/CANDIDATE_ADOPTION_V2.md,
docs/NODE_CONTROL_V2.md, docs/NODE_OPERATOR_V2.md, docs/NODE_PROVIDER_V2.md,
docs/NODE_REJOIN_V2.md, docs/NODE_FAILOVER_V2.md,
docs/MEMBERSHIP_STATE_V2.md,
docs/REPLAY_V2.md, docs/THREAT_MODEL.md and docs/MEMBERSHIP_TEST_MATRIX.md.
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

## MB-04A2 finite operator invariants

Read [docs/NODE_OPERATOR_V2.md](docs/NODE_OPERATOR_V2.md) before modifying the separate
mirror-node-v2 executable. Preserve V1's source and explicit package default-run.
The retained A2 commands are init, status, sync-once, recover and standalone help.
Require explicit control/store paths and independent C/genesis; no environment/config-file
fallback, DNS, implicit initialization, automatic recovery or retry is permitted.

Validate the entire command before filesystem/socket effects: at most 32 arguments
after the executable name, 4,096 bytes per token and 16,384 total bytes. Reject
non-UTF-8/control input, duplicate/unknown/mixed-command flags, positional extras,
missing values and malformed/overflowing numbers or hashes. Use canonical unsigned
decimal and exact 64-character ASCII hex; accept either hex case and output lowercase.
Init requires valueless --attach-existing plus independent height/tip/root/generation;
height must be 0 through 4,095 and generation no greater than height before calling
A1. It attaches control to an already initialized profile-3 store and never creates
fresh storage.

Status calls only A1 status: missing control exits 3, verified clean exits 0,
valid pending intent exits 4, invalid/operational errors exit 1. Pending output
labels confirmed/old/candidate pins and store_selection=uninspected; it never
asserts the unresolved store's selected endpoint. Usage errors exit 2 before effects.
Other commands use exit 1 for missing/pending operational failures.

Sync validates a numeric nonzero-port address, 1–60,000 ms timeout and positive
I/O allowance at most 157,129,488 bytes, then strictly opens the ready A1 owner
before network contact. Hold that same controller through exactly one F2 request and consuming
adoption; pending, malformed, mismatched or busy local state must contact no peer.
The timeout covers the session, not local opening/adoption or output. Recovery
takes an explicit complete/retain-old action and preserves every A1 no-rollback rule.

Emit bounded 4,096-byte key=value records only after API success; report actual
attempts/charged I/O and previous/selected pins for successful sync. Diagnostics
use static categories without raw arguments, paths or nested error text. Use
fallible writes/flushes; output loss after init/sync/recover can follow durable
success and must report operation_may_have_completed=true plus inspect-status.
Never retry a mutation because output failed or claim all errors leave intent.

## MB-04B1 provider and shutdown invariants

Read [docs/NODE_PROVIDER_V2.md](docs/NODE_PROVIDER_V2.md) and the accepted-stream
addition in [docs/PEER_SESSION_V2.md](docs/PEER_SESSION_V2.md) before modifying
`serve`. Require deliberate control/store paths, independent C/genesis and one
numeric nonzero-port listen address. Validate max-attempts 1..8, cumulative
max-io-bytes 1..157129488, timeout-ms 1..60000 and admission-ms 1..60000 before
effects. Preserve the existing argument limits and no environment/stdin/config
fallback. Signal setup is scoped to this command after parsing.

Strictly open clean A1 control before binding. Retain one controller, its fixed
complete accepted history and one F2 budget owner until shutdown; do not refresh
history, reacquire either root, create another owner or replenish allowance
between peers. Concurrent A2 status/sync remains busy under exclusive A1 ownership.
There is one provider listener, no listener cloning and at most one admitted
connection at a time. Kernel backlog and other processes are outside these bounds.

Check stop/time/remaining budgets before accept and stop/time again after accept.
Close a raced stream before request processing and without charging an admitted
session. `serve_connection` consumes an already accepted stream, retains the same
owner's counters, establishes one original attempt deadline and uses the shared
checked request/EOF/frame routine. It guards and closes even preflight refusals.
Existing `serve_once` retains its original listener-wait deadline and request
behavior remains unchanged. A valid independent MPQ2 request and request EOF are
required before any history output.

SIGINT/SIGTERM only set a sticky stop flag. Idle admission polls in bounded waits;
an admitted attempt drains under its original F2 timeout, without another accept
running concurrently. An expired admission window also stops new work and drains
at most the current attempt. The maximum configured admission-plus-drain allowance
is 120000 ms; it is not hard CPU preemption, a total process wall-clock guarantee,
or a bound on initial A1 opening/replay or final output. Preserve the normal stop
precedence: shutdown, admission timeout, wire limit, attempt limit.

Rejected peers retain actual charged bytes and attempts. Completed plus rejected
sessions equals attempts in a normal final report, and the latest static rejected
session category is retained even if a later peer succeeds. Local history,
allocation, listener/setup and internal-limit failures are fatal operational
errors; do not present them as harmless peer rejection. Normal bounded shutdown
exits 0, including runs with rejected peers; operational/output failure exits 1
and usage failure exits 2. Existing status alone retains exits 3 and 4.

Release sockets, listener and A1 ownership before the single final bounded output;
restore signal dispositions on success and failure. No readiness/progress/debug
output may block while provider ownership is held. Final output loss reports
`operation_may_have_completed=true` and never restarts serving. The reported
checkpoint identifies the fixed history served; it does not assert remote adoption
or prevent a later writer from changing storage after owner release.

## MB-04B2 rejoin and serve invariants

Read [docs/NODE_REJOIN_V2.md](docs/NODE_REJOIN_V2.md) before changing the command.
The separate `mirror-node-v2` executable adds `rejoin-serve`. It performs
at most one explicit complete receive, existing A1 keep/adopt, then finite serving
of the resulting fixed history. The contract was written before implementation:
[NODE_REJOIN_V2.md](docs/NODE_REJOIN_V2.md). Existing A2 commands, B1 `serve` and
the V1 default invocation remain available.

| Boundary | Delivered behavior |
|---|---|
| Startup | Complete argument validation, one signal guard, strict clean A1 opening before either outgoing contact or bind. Missing, pending, corrupt, wrong-network or busy state refuses. |
| Receive | One unchanged F2 request, actual complete history execution, matching End and mandatory terminal EOF. A failure is fatal even if shutdown was requested; there is no serve-old fallback. |
| Adoption | A stop observed after validated receipt but before adoption admission discards the candidate. Once admitted, synchronous A1 keep/adopt finishes or fails through its unchanged durable ordering. |
| Ownership | Retain `outcome.into_control()` directly. The same controller, signal state and F2 attempt/byte allowance span both phases; no reopen, reset or concurrent receive/serve. |
| Service | Check shutdown and exhausted budgets before binding. Serve the selected whole branch with B1 admission and original-deadline drain. A last-allowed receive can adopt and then stop without binding. |
| Reporting | Release sockets and both directory owners before a bounded 26-field result. Report previous/selected pins, actual choice and separate receive/service usage plus combined totals. |

All ten options are deliberate: separate control/store paths, independent C/genesis,
one numeric nonzero-port `--peer`, numeric nonzero-port `--listen`, `--max-attempts`
1..8, `--max-io-bytes` 1..157129488, `--timeout-ms` 1..60000 and `--admission-ms`
1..60000. Existing 32-argument, 4096-byte-token and 16384-total-byte parser bounds
remain, with no DNS, environment/stdin/config fallback or effects before validation.

Receipt charges one attempt and its actual I/O. Service uses only the remainder.
An allowance equal to the 72-byte request plus complete response bytes is still
insufficient: the terminal EOF probe needs one unspent byte of capacity even though
zero-byte EOF is not charged. The service admission clock starts after adoption,
immediately before bind when permitted. Receive timeout plus admission window plus
at most one final service timeout totals at most 180000 ms at configured ceilings.
This excludes opening, durable adoption and output and is not hard CPU preemption
or a complete wall-clock guarantee. Standalone B1 retains its 120000-ms allowance.

The post-receive stop check admits adoption. Signals after that check do not cancel
A1 or imply rollback. Every adoption error or later fatal bind/listener/service error preserves
its static error class and reports `operation_may_have_completed=true` with
`next=inspect-status`; no error asserts unresolved selected pins. Final output loss
uses the existing conservative rule, including normal pre-adoption shutdown.
No implicit initialization, recovery, retry or rollback is introduced.

Normal results obey receive_attempts + serve_attempts = attempts,
completed_sessions + rejected_sessions = serve_attempts, and receive_wire_bytes +
serve_wire_bytes = wire_bytes. A validated candidate discarded on shutdown reports
`receive_status=validated`, `choice=not-attempted` and the unchanged verified
selection. Normal limits/signals exit 0; operational/output failures exit 1 and
usage errors exit 2. Only status retains exits 3 and 4. No result claims peer
adoption, global latestness, human finality or protection against a later local writer.

## MB-04B3 failover invariants

Read [docs/NODE_FAILOVER_V2.md](docs/NODE_FAILOVER_V2.md) before changing this command.

The separate `mirror-node-v2 rejoin-failover-serve` command accepts one to four
ordered numeric peers in a single required `--peers ADDRESS[,ADDRESS...]` token.
The other nine deliberate options, 32-argument/4096-token-byte/16384-total-byte
limits and effect-free validation remain. Reject empty/malformed/scoped lists,
duplicate numeric endpoints including IPv4-mapped aliases, DNS and zero ports.
Original parsed connection addresses are retained; no distinct-physical-node
or peer-authentication claim follows from endpoint checks.

Strict clean A1 opening precedes all contact and binding. One controller, sticky
signal guard and cumulative F2 owner span a finite ordered acquisition prefix,
at most one admitted keep/adopt and fixed-history service. Each configured entry
is requested at most once. Classify the typed session/history error before
sanitizing it: only the explicit contract allowlist advances. Allocation,
authority, accounting/global-resource and all unlisted errors remain fatal,
including ambiguous nested configuration/operation/ledger failures. A failed
receive plus shutdown remains an operational error, with no next-peer contact.

The first complete validated candidate permanently ends acquisition, including
identical/shorter/tied keep-current choices. Matching End and terminal EOF remain
mandatory. The shared B2 post-receive routine handles adoption admission, existing
A1 durability, retained-controller service and checked phase deltas. Stop before
admission discards a validated candidate; admitted adoption finishes or fails.
All-peer failure never serves the old branch as a fallback.

Configured caps remain 1..8 total receive/service attempts and 1..157129488
actual I/O bytes;
each original attempt timeout and post-adoption service admission remain
1..60000 ms. Failed/partial traffic is charged without refund or replenishment.
The EOF probe needs one unused byte even though zero-byte EOF is not charged.
A final allowed successful receive can adopt then stop normally without binding.
For P peers, M attempts, timeout T and admission A, conservative configured phase
allowances are min(P,M)*T+A+T (at most 360000 ms) and M*T+A. All service attempts
remain within M; these are not hard wall-clock/CPU/RSS guarantees. Local opening,
adoption, nonpreemptible processing, scheduling and output retain their qualifications.

Normal output has 32..35 fields: B2's 26 fields plus bounded acquisition totals,
first validated ordinal, latest receive error, terminal acquisition reason and
one static result per configured entry. Service rejection evidence stays separate.
Errors attach sanitized acquisition evidence without selected-pin assertions.
Every adoption error and later fatal setup/service failure retains its category,
possible-completion warning and inspect-status guidance. Invalid optional trace
formatting cannot erase that core diagnostic. All sockets/owners/signal guards
are released before final output; output loss never repeats the operation.

## Exact next increment — MB-04B4

**Next: MB-04B4 — bounded persistent-run lifecycle. READY — DESIGN FIRST.**

Design a finite run mode that can repeat acquisition, keep/adopt and service
rounds while retaining one controller, one sticky signal guard and one cumulative
transport allowance. Choose the operator interface during that design; do not
assume another rejoin-* command or change the accepted B3 command.

Before implementation define explicit round and repeated-peer authorization,
bounded inter-round waits, whole-invocation attempt/byte/time limits, first-valid
ordering across rounds, all-peer-failure policy and shutdown at every phase.
The existing private provider consumes/releases the controller, so specify an
ownership-returning round boundary without dropping/reacquiring either root.
Reacquisition pauses serving; no concurrent listener/request loop is implied.
Repeatedly choosing a stale but valid first peer is possible and must not be
described as convergence or global latestness.

Once any round admits adoption, every later fatal error must retain truthful
possible-completion context, even during a later receive or wait. Distinguish
the initial checkpoint, each completed round and the final selected checkpoint;
bound all reports and preserve resource release before output. Verify actual
multiple rounds, whole-state reopening, cumulative exhaustion, ownership and
shutdown before declaring that lifecycle implemented.

B4 is planned only. Indefinite operation, discovery, parallel scheduling, pool,
wallet and policy choices are not silently included. Full MB-04 and Gate A/B
remain unfinished; all monetary, enrollment/device and accessibility decisions
and proposals retain their existing gates.

Run every existing gate above; keep exact-source and later documentation-head
CI separate. Refresh the handoff and preserve the accepted component boundaries.

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
