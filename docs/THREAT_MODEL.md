# Mirror membership integration threat model

Historical design checkpoint: **MIRROR-MB01A-001**. Scope: MB-01A source review
and V2 design. Current implementation: **MB-04C2 finite local pool intake and inspection**,
with source, focused results and exact acceptance recorded in the C2 update below.
B3's failover, B2's rejoin, B1's provider, A1's durable control/restart
authority and A2's finite commands remain recorded separately. Exact acceptance evidence is
tracked in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
[MASTER_PLAN.md](../MASTER_PLAN.md).
Source inspected at `db2861930cc0f01f735ccfba80a4f530f4d1ddd6`; PR #1 comments
were reviewed and contained checkpoint/roadmap notes, not an independent security
review. This is an engineering review, not an audit certification.

Read with [the proposed integration specification](LEDGER_MEMBERSHIP_V1.md) and
[the test matrix](MEMBERSHIP_TEST_MATRIX.md). No issue below is represented as fixed
merely because a mitigation is documented. At the original design checkpoint,
no V2 acceptance test had run.

The preceding sentence describes the original design checkpoint. Current B1–B4,
MB-02A/B/C, MB-03A/B/C/D/E1/E2a/E2b/F1/F2 and MB-04A1/A2/B1/B2/B3/B4/C1/C2 acceptance evidence
are in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
[MEMBERSHIP_TEST_MATRIX.md](MEMBERSHIP_TEST_MATRIX.md). The updates below record
component-level mitigations without rewriting historical findings. Actual V2
branch acceptance, complete-block replay, immutable selected-store reopening and
one-block append publication now exist. MB-03C adds explicit recovery of one
independently authorized append's recognized residual shapes. MB-03D compares fully
accepted branches under independent network identity and returns the complete
preferred branch as an immutable reference. MB-03E1 adds independently pinned
selection metadata and preparation that retains both complete accepted histories.
MB-03E2a adds synchronized whole-branch publication, exact old-copy cleanup and
fencing under stable cooperative ownership. MB-03E2b adds explicit completion or
retention of the narrowly recognized replacement residue under independent pins
and both complete accepted histories. MB-03F1 adds a separate bounded frame codec
and consuming receiver that executes complete candidate histories under independent
network selection. MB-03F2 composes it with finite TCP sessions, actual partial-I/O
and deadline bounds, independent branch comparison, owned whole-history publication
and retained inputs for explicit recovery. **MB-03F2, MB-03F and MB-03 are
VERIFIED_COMPONENT within this finite storage/peer-session scope.** MB-04A1 now
persists trusted full local checkpoints and both exact accepted histories in a
separate operator-owned control directory before store mutation. Its finite library
and process-restart harness are a VERIFIED_COMPONENT. MB-04A2 exposes that unchanged
authority through finite `init`, `status`, `sync-once` and explicit `recover`
commands in a separate executable. MB-04B1 adds finite `serve` with one fixed
accepted history, held controller, listener and cumulative F2 owner, explicit
admission limits and SIGINT/SIGTERM draining shutdown. Its scope and evidence are
recorded in the B1 update below. B2 composes one receive, keep/adopt and fixed-history
serving. B3 adds a capped ordered acquisition prefix and closed typed advancement
policy. B4 repeats that finite path under explicit round/wait limits while retaining
one controller and cumulative transport allowance. Its source and acceptance
evidence are recorded below; earlier continuation paragraphs are historical.
C1 adds local signed-input retention, one full ordered ledger evaluation and
explicit exact-parent reconciliation, with independent cumulative work caps.
C2 adds finite local regular-file intake and optional exact FIFO inspection,
with explicit input/retention/work caps and release before bounded output.
The exact current continuation appears in the C2 update: MB-04D1 wallet and
signing boundary, design first. Arbitrary corruption
repair, indefinite reconnect, broader scheduling, retained pool integration/persistence/gossip,
wallet and human finality remain unfinished. MB-04 is IN_PROGRESS and Gate A/B remain unmet; exact source
and later documentation-head CI are tracked separately.

## Historical B3 implementation update

Source `b6f1233d038901aa0a13c8ddac9afbc34418e846`, exact-source push CI44317174000
and PR CI44317184524 passed. B3 addresses F02/F04/F05 for the separate V2 in-memory
executor: bounded direct inputs and retained approval history, parent-snapshot
eligibility, deferred activation, same-block withdrawal effects, final population
high-water and complete candidate discard on failure. B2 authorization windows are
now enforced during state execution, with pending expiry and permanent tombstones.
Legacy V1 is preserved, not retrospectively relabeled hardened.

The supplied parent view and bind_to_validated_block remain trusted integration
boundaries, not proofs of actual chain validity. Finite approval/history budgets
can halt admissions and do not solve availability. A valid old log/snapshot is not
authoritative latest state. At that checkpoint, B4 roots/replay, MB-02 ledger
authority, MB-03 durable reorgs, human consensus and real private biological
matching remained unfinished. B4 component progress is recorded next.
No finding is declared globally closed or externally audited from these tests.

## Historical B4 implementation update

Source `21e2272b5ac92c00e840d7aaa26a99570a5ec24f` passed
[push CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870)
and [PR CI 44404351436](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404351436).
The exact push checkout and full logs were inspected. There are 69 new B4 tests
(26 codec, 10 independent vector, 31 replay and two private arithmetic cases),
323 workspace tests total, 216 personhood release reruns and one compile-fail
documentation test. Three references, three examples and preservation checks pass.

Canonical MPS2 commits the complete membership records and counters while excluding
external tip-hash provenance. The decoder bounds input, reconstructs indexes,
checks metadata/accounting and verifies retained signatures. Its untrusted wrapper
cannot expose a runtime state or registry. Internally coherent tampered withdrawal
status can still pass structural decoding; only comparison against independently
trusted state or full verified replay establishes the intended history.

MPH2 retains empty blocks and exact group boundaries. Replay re-executes B3 and
requires independently trusted block hashes, exact ordered-group digests and every
intermediate membership root, plus an external final C/height/tip/root expectation.
Tests include commuting approvals and alternate valid withdrawal bytes that yield
the same state root: they still reject against the trusted group digest. A valid
old prefix does not satisfy a newer expected checkpoint. The provider of these
trusted values remains an explicit caller boundary, not a deployed chain validator.

The default log cap is 4 MiB, distinct from the default 41,484,378-byte snapshot
bound. Diagnostic replay additionally caps groups at 4,096 and cumulative state
items at 1,048,576, including configuration clones and empty-block scans. Bounds
are checked before input-sized copies or repeated state execution; failures return
no final registry. This constrains work but does not guarantee honest admission,
continued availability, fixed execution time or unbounded whole-chain replay.

F06/F07 actual block/ledger authority, F08 durable publication, F09 network session
limits and F10 private biological matching remain unresolved in their stated
scope. The exact diagnostic group digest is not the future MRB2 operation root.
Future mixed blocks must retain transfers and original inclusion positions; they
cannot be replayed by filtering operations into the current membership-only log.
No biometric uniqueness, anonymity, currency issuance or human consensus follows
from these passing component tests. MB-01B is complete only in that bounded scope.

## Historical MB-02A implementation update

Published source `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382` adds 80 tests
(37 operation, 33 block and 10 independent-vector cases). Local verification passed
403 workspace tests and one documentation test, 121 core release tests, and 216
personhood release tests plus its documentation case. Four reference checks and
four synthetic examples passed.
[Exact-source push CI 44428924399](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399)
and [PR CI 44428936866](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428936866)
passed. The push checkout and full job log were inspected; CURRENT_STATUS.md
records the exact acceptance evidence.

The new `mirror_core::v2` component adds Transfer and a unified four-kind operation
envelope, a 160-byte header and bounded MRB2 bodies. Strict signatures and separate
domains bind Transfer fields. The block operation root binds count, order, record
lengths and complete signed bytes, including the odd-leaf duplicate-last case.
Construction, encoding, decoding and root calculation enforce complete byte,
operation, membership and enrollment limits. Independent Python vectors guard the
actual Transfer/header/block preimages and preserve existing membership bytes.
See [CORE_V2_ENCODING.md](CORE_V2_ENCODING.md) and test-matrix section 7.

This supplies F06's missing mixed-body representation and integrity component;
**it does not close F06 or F07**. `verify_integrity` checks bounds, configuration,
the operation root and signatures. It does not execute balances/membership,
consume account sequences, validate the supplied ledger root, check actual
parent/height/timestamp/PoW or pin an authoritative genesis. Even positive amount,
zero-fee and authorization-window checks belong to the separate operation-context
method and must be invoked by the future executor. Its parent view remains a
trusted caller boundary. Body integrity cannot authorize a canonical state update.

F08/F09 durable publication and network-session limits remain unresolved; an
in-memory decoder's bounds are not disk-read, session or global availability
guarantees. F10 private biological matching and F11 the final fee/reward economy
also remain open. Original mixed-operation positions are now represented and
committed, but B3/MPH2 must not be fed a transfer-filtered list as though it were
equivalent execution. At that checkpoint, **MB-02B combined-ledger execution was
next**. Its component implementation is recorded below. No launch gate, human
uniqueness, anonymity, currency issuance or device readiness was established by
MB-02A.

## Historical MB-02B atomic execution implementation update

Published source `420d7114ddf782469a6ace6ba0a2c4076fb93aaf`, tree
`6871ad0fbc178bd76262a26416a7d255d9fcbd66`, implements atomic accounts/membership
staging under [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md). Local acceptance passed
462 workspace tests (403 retained + 18 indexed membership + 33 ledger execution
and private fault cases + eight independent vectors), three compile-fail
documentation cases, release reruns for 121 core, 234 personhood and 49 state tests,
and five reference checks and five synthetic examples. Personhood's three release
documentation cases passed too. These counts are component evidence,
not an external security audit or additional completed roadmap rows.
[Exact-source push CI 44619155972](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619155972)
passed; exact checkout `420d711` and full job `135895834032` logs were inspected.
The same test, reference, example and audit gates passed.
[PR CI 44619172378](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619172378)
also passed. [CURRENT_STATUS.md](../CURRENT_STATUS.md) records final acceptance.

The `mirror_state::v2::LedgerV2` executor addresses another part of F06: it stages
the account map and membership state together and commits both through a combined
root. Complete body/count/byte preflight occurs before input-sized candidate
cloning or signature work. The executor then follows original mixed operation
order, so an earlier invalid transfer cannot be hidden by a later membership
error, or vice versa. Both candidates are discarded on any failure, including a
late account invariant failure. Tests compare the complete parent and its roots,
not only the returned error. This verifies T27's staged failure directions;
**rejecting a wrong claimed final block root remained an MB-02C requirement at
that checkpoint**.

Accounts retain explicit zero allocations and exhausted balances with their
sequences. Transfers require positive amounts, zero fees, exact account sequences
and sufficient balance; self-transfers advance only the account sequence. Checked
arithmetic, account capacity and the exact initial synthetic-supply invariant
bound execution. Membership sequences remain independent. Admission, activation
or withdrawal neither issues funds nor silently changes spending rights. No
founder allocation, marketing privilege, mint, reward or grant API is added.
F11's final monetary-policy problem is still open; D-01/D-04 remain unresolved.

The personhood projection preserves full-block inclusion indices, including
intervening transfers. Its immutable plan checks strictly increasing indices and
all quotas before state cloning. An opaque, non-cloneable builder is consumed by
each operation and by finish; compile-fail cases protect against reusing or cloning
a failed partial candidate. Finish requires every planned membership step and then
applies the original parent-Active, reviewer-survival and activation-order rules.
This extends F02/F04/F05 mitigations into mixed staging without establishing
resource availability, honest admission or human-weighted consensus.

The projected byte budget treats each omitted V2 Transfer=0 as exactly 243 signed
bytes plus its four-byte prefix. The complete core preflight independently checks
the actual operations. A cross-crate test accepts the exact 710-byte mixed body
and rejects a 709-byte cap. This contract needs explicit review when adding an
operation kind or changing transfer size; a projection cannot validate the omitted
transfers on its own.

MPS2 permits runtime enrollment positions below `max_operations` even when above
`max_membership_operations`, because the latter limits a count rather than a full
body position. This is an explicit structural-acceptance extension, not a wire or
hash-domain change; older decoders may reject those mixed states. A coherently
altered position can still decode but fails comparison with independently executed
trusted state. The snapshot retains no executable-state conversion. MPH2 still
rejects transfers and retains its membership-only replay semantics.

Independent reference vectors reproduce account bytes, membership bytes,
disabled-economics roots and combined ledger roots across mixed synthetic histories.
They include equal final accounts with different membership positions and therefore
different combined roots. The current block hash remains external provenance to
avoid a commitment cycle. This supplies T25's staged component evidence; it does
not supply an actual accepted header's root comparison. Reexecution of a signed
fixture under a synthetic trusted parent is not T26's canonical chain recovery.

**F06 is partially mitigated at the staging boundary; F07 is not closed.**
`bind_to_validated_block` attaches caller-supplied provenance and does not verify
genesis, actual header height/parent/configuration/timestamp/target/work, body
commitment, claimed ledger root or finality. **At the MB-02B checkpoint,
next was MB-02C**, to validate those real block inputs and replay accepted MRB2
history before binding state. Its implementation is recorded below.
F08 durable recovery and reorg publication remain MB-03; F09 network sessions,
F10 biological uniqueness/privacy and the final human consensus/economy remain
their later gates. No launch, mainnet, device readiness or sensitive-data processing
is authorized or established by these component tests.

## Historical MB-02C actual block acceptance implementation update

Published source `db2cb58ba5a9a828ec949debd1b52b5915a66f3b`, tree
`3fa1d3b44abb885898cc256f9525355025649a0a`, matches the locally tested source.
The contract is [CHAIN_V2.md](CHAIN_V2.md). Local acceptance passed **523 workspace
tests = 462 retained + 61 new**: 15 work, 14 manifest, 20 public chain, three
private budget and nine independent-vector cases. The same three compile-fail
documentation tests passed. Release reruns passed core 121, personhood 234 plus
the same three documentation cases, state 49, consensus 23 and chain 57. All six
independent reference checks, six synthetic examples, changed-file formatting
and preservation/allocation audits passed. Reruns are not additional unique tests.

[Exact-source push CI 44623282448](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623282448)
**passed**. Job `135910328823` full logs and the exact
`db2cb58ba5a9a828ec949debd1b52b5915a66f3b` checkout were inspected; all listed test,
reference, example and audit gates passed in CI too.
[PR CI 44623296178](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623296178)
also **passed**. Evidence was verified on 2026-11-20. Later documentation-head
checks remain separate from this source evidence and are tracked in
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and test-matrix section 9.

The new `MNM2` manifest requires an independently selected configuration hash and
exact genesis hash, and verifies every ordered seed proof. Its decoder preflights
complete framing/count/size limits before configuration/proof allocations and
signature work. A manifest chooses a network; it does not itself validate genesis
or establish the biological uniqueness of its seeds. A peer cannot replace the
local choice by supplying a different self-consistent manifest.

`ChainV2` now covers F06's actual V2 in-memory block/state association. It checks
genesis against the manifest and independently executed bootstrap, then validates
each child against the accepted configuration, next height, exact parent hash,
strictly increasing parent timestamp, literal configured target and actual work.
It recomputes the ordered full signed-operation commitment, executes the complete
mixed body and compares the resulting accounts/membership/economics root before
binding state to that actual block. Valid work cannot excuse a wrong ledger root.
Original per-operation order and full inclusion positions remain intact; a later
signature failure cannot hide an earlier execution failure.

Preparation returns an unaccepted proposal without changing the chain. Every
returned acceptance error preserves the complete accepted branch, ledger, manifest
and resource counters. Tests reach late mixed execution and a re-mined wrong final
root, rather than stopping only at an invalid signature or hash. This completes
T27's in-memory wrong-root subcase; process failure and disk publication are not
covered by that atomicity claim.

Only the chain's private accepted block prefix supplies ancestry to its own
executor. It does not import a caller-bound `LedgerV2`, untrusted MPS2 cache or
externally supplied parent oracle as accepted state. The lower-level public
`bind_to_validated_block` helper remains a trusted staging boundary for other
callers; its existence is not a separate chain-validation path. A different
branch's signed anchor rejects even when the child parent, work and signatures
otherwise pass.

Typed and encoded replay rebuild complete MRB2 histories through the same actual
acceptance path. An external expectation specifies C, exact genesis, height, tip
and ledger root. An authentic old prefix or unintended branch does not satisfy
that expectation. The replay method neither discovers the latest global tip nor
protects an application that supplies its own obsolete expectation. MPH2 stays
membership-only; MPS2 decoding still does not produce authoritative ledger state.
F07's exact genesis/configuration and selected replay checks are now covered in
memory, including another valid genesis nonce. Its durable reopen/latest-tip
portion and the original V1 behavior remain unresolved.

The local synthetic branch is capped at 4,096 blocks, 16 MiB of complete encoded
history and 1,048,576 cumulative state/configuration work items. Aggregate count
and byte checks precede replay decoding; every transition checks its state-work
charge before staging, including empty blocks. Charges commit only on acceptance;
overflow and exhaustion fail. These are finite profile limits, not guarantees of
admission, availability, disk-read bounds or network-session protection. The local
mining helper's one-million-attempt ceiling is separate from consensus validation.
Strict parent timestamps use no local clock or future-time ceiling; accepting
`u64::MAX` leaves no successor. This is an explicit synthetic rule, not a finished
production time-manipulation or liveness policy.

The independent Python fixture was frozen before its Rust comparison. Twenty
signatures use public development seeds and actual mined ancestor hashes. The
fixture reproduces five main-branch states and a genuine membership fork where
reviewer withdrawal leaves a candidate pending despite identical final accounts.
It also checks empty-block expiry, an alternate valid genesis, re-mined invalid
root and foreign-branch anchors. All nine Rust vector cases matched on their
first run; no failed Rust output became a new expectation.

**At the MB-02C checkpoint, MB-02A/B/C completed MB-02 as a VERIFIED_COMPONENT
for bounded in-memory acceptance, and MB-03A was next.** That checkpoint did not
establish durable recovery, a live V2 daemon, global canonical choice or finality.
The following MB-03A update records its later storage increment. F08–F11 retain
their remaining scope; neither Gate A nor Gate B is met.

## Historical MB-03A immutable selected-store implementation update

Contract [STORAGE_V2.md](STORAGE_V2.md) freezes storage profile 1: an explicitly
selected finite chain, generation zero, with separate initialization, exclusive
reopen and shared read-only inspection. It preserves the legacy `BlockStore`,
node recovery paths and all previous encodings. MB-03A has no append, generation
update, replacement, automatic repair, global fork-choice or live node API.

Local Rust 1.98.0 acceptance passed **588 workspace tests = 523 retained + 65 new**,
with **one ignored subprocess helper** and the same **three documentation tests**.
The additions are 24 selection-codec/vector cases, 34 filesystem cases, five
substantive lock cases and two initialization-fault cases. The helper is invoked
only by the process tests; its harness invocation is not an extra unique test.
Storage release reran 75 cases. All seven independent reference checks passed.
These are local results: implementation commit, publication and exact-source CI
must be read from [CURRENT_STATUS.md](../CURRENT_STATUS.md). No MB-03A CI success
is inferred from historical green runs. Neither these checks nor this review is
an independent security audit.

F07's V2 immutable-store reopen boundary now requires independently retained
configuration/genesis identity **and** final height/tip/combined-ledger-root
expectations. The arguments must agree before I/O. A checksum-correct record
cannot select its own trust; every public seed proof and every actual MRB2 block
are checked again through `ChainV2`. No MPS2 snapshot, cached account database or
stored counter becomes accepted state. Another valid genesis, an old prefix,
a wrong intermediate root and an unintended membership branch reject. Supplying
an obsolete external checkpoint still deliberately selects an old valid branch;
this is not global freshness discovery or rollback protection for a caller that
forgets its trusted checkpoint.

F08 gains a separate bounded V2 path. Root enumeration has a four-entry bound,
so oversized layouts can return `InvalidLayout` before legacy/foreign classification;
all reject without cleanup. Selected metadata is exactly 232 bytes; manifest bytes
are capped at 2,781,636; complete history is capped at 4,096 blocks and 16 MiB.
Per-file and aggregate lengths are checked before block-payload allocation. Reads
are limited to their checked length plus one sentinel byte; shortening/growth
rejects. The maximum accepted encoded payload is 19,559,084 bytes, not a total
resident-memory guarantee. Replay recomputes the existing 1,048,576 cumulative
state/configuration-item budget and retains permanent membership/account history.

Local ownership uses a private un-cloned Linux directory descriptor: exclusive
writers exclude readers/writers, shared inspectors coexist, and closing the
handle or terminating its process closes its descriptor. The kernel lock releases
after all descriptors for that open file description close, including any briefly
inherited by a concurrent fork before exec. Parent ownership tests are isolated
from unrelated test forks while preserving their real concurrent child checks.
There is no PID lease or removable lock file. Named tests include
`writer_in_another_process_releases_on_store_drop_while_process_lives`,
`killed_writer_releases_without_unlinking_or_repairing_anything` and
`shared_locks_in_separate_processes_coexist_and_exclude_a_writer`. Pipe
acknowledgments establish process ordering without sleeps. Read-only paths never
create, write, mine, repair or delete; OS access-time effects are outside that
application-level guarantee. Writer access means exclusive ownership, not proof
that permissions or the mount will permit future writes.

The stable root must never be renamed, replaced or removed while a handle lives.
All processes must cooperate; the parent tree and local owner are trusted. Static
symlinks, nonregular selected files and hardlinked managed files reject; opened
identity checks do not defeat hostile same-user rename races. Linux tmpfs and
overlayfs were exercised locally. WSL2's native Linux filesystem is intended but
not directly verified; native Windows rejects, and DrvFS/NFS/SMB are unvalidated.
The code checks the operating system, not a filesystem-type whitelist.

Initialization synchronizes data and directories before publishing the selected
marker last. Before publication, failure returns `InitializationIncomplete`;
after a rename attempt, failure returns `PublicationUncertain`, without a handle
or silent deletion. Two tests,
`prepublication_errors_never_expose_selected_state_or_cleanup_partial_files` and
`postpublication_errors_return_no_handle_and_require_checked_reconciliation`,
exercise **nine selected boundaries**: seven before publication and two after it.
They inject errors after successful operations; they do not simulate every write,
rename or synchronization failure, torn writes or power loss. Checked reopening
can validate a complete visible selection after an uncertain result; it does not
retroactively prove a failed sync became durable.

**At the MB-03A checkpoint, next was MB-03B**, durable generation/accepted-tip
updates and explicit failure reconciliation. Its bounded implementation is
recorded below. Canonical development fork choice must precede durable branch
replacement, then membership-aware reorganization and peer reconstruction follow.
F08 is not globally closed: legacy behavior and later publication cases remain.
F09 network sessions, F10 biological uniqueness/privacy, F11 final rewards/economics,
human finality, real-device evidence and both launch gates remain open.

## Historical MB-03B append publication and checked reconciliation update

Published implementation
[`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c`](https://github.com/Ahmed-Sobeih/mirror/commit/ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c),
tree `06d7c3f4fa24d96c844e2301c76c01f804151192`, matches the locally tested source.
Its increment changes 14 files, with 3,568 insertions and 11 deletions. Publication
and verification are separate facts.
[Exact-source push CI 44649111203](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203)
and [PR CI 44649119266](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
both **SUCCEEDED**. Full push job `135999222947` logs confirmed checkout
`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c` and all workspace, documentation,
release, reference, example and allocation/runtime-data audit gates. Later
documentation-head CI must be checked separately; these source results do not
claim an untested documentation head passed.

**MB-03B is VERIFIED_COMPONENT for bounded accepted-child append and clean-state
reconciliation. MB-03 remains IN PROGRESS.**

The separate [append contract](STORAGE_APPEND_V2.md) specifies `AppendStoreV2` and
MSU2 storage profile 2. Immutable StoreV2/profile 1, legacy BlockStore and previous
network/block/state encodings remain unchanged. Initialization requires an absent
or empty directory and an already accepted complete ChainV2. There is no automatic
adoption, profile conversion, branch replacement or cleanup.

The increment adds **70 unique cases: 28 selector/vector, 31 filesystem integration,
eight private fault tests and three additional real-process ownership tests**.
Local Rust 1.98.0 verification passed **658 workspace tests**, one intentionally
ignored subprocess helper, and the same **three documentation tests**. Release
reruns passed core 121, personhood 234 plus the same three doctests, state 49,
consensus 23, chain 57 and storage 145. All eight independent references and eight
examples passed. These reruns and scenario iterations are not additional unique
tests. The independently inspected exact-source CI above passed the same gates;
CURRENT_STATUS.md records publication and subsequent checkpoint evidence.
Test-matrix section 11 maps the actual cases and distinguishes scenarios from
test counts. This is component verification, not an external security audit.

MSU2 is a new 280-byte envelope containing one unchanged 232-byte MSS2 snapshot,
an append generation and a separate integrity checksum. Each open requires the
independently chosen network, full height/tip/ledger-root checkpoint and generation.
Generation starts at zero for a newly initialized store, even if its initial
history has nonzero height. It increases once per local child publication and
cannot exceed selected height. A checksum, higher generation or pending marker
cannot choose its own authority. Old complete rolled-back state can still satisfy
old deliberately selected external pins; no global rollback/freshness discovery
or economic/human participation meaning is assigned to generation.

F08 gains bounded one-block publication. The candidate first passes actual
ChainV2 acceptance on a clone, with original mixed operations and all budgets.
An invalid child leaves memory and files unchanged and the writer usable. Before
preparation, the complete selected disk history is replayed against retained pins
and compared with the whole in-memory chain, ledger and counters. A disk mismatch
fences the handle. The private directory lock and stable root remain held.

The writer creates and synchronizes the new complete MRB2 file, synchronizes the
blocks directory, then creates and synchronizes a pending next-generation selector.
It verifies the root still names the held inode and compares the old selector's
exact bytes immediately before publication. Only after selector rename and root
synchronization succeed does memory advance and the fence clear. Preparation
errors retain artifacts and report incomplete update; after publication is attempted,
errors report uncertainty. An error may mean disk already selects the candidate.

Every mutating failure retains the fence. Normal chain/selection/checkpoint access
and subsequent appends reject while fenced; `last_confirmed_checkpoint` is explicitly
only the last confirmed memory state. This avoids presenting old memory as the
current durable selection after an uncertain rename. The candidate checkpoint in
an error identifies a locally validated candidate, not a completed publication.
No sleep, retry, stale-PID takeover or weaker ownership assertion bypasses the fence.

Writer open and consuming reconciliation fully validate the independently selected
clean store and synchronize the validated manifest, every block, block directory,
selected marker and root before returning a usable writer. Consuming reconciliation
retains the same directory descriptor throughout. Its failures return no handle.
Shared inspection is read-only and performs no resynchronization. These distinctions
avoid treating visible read-back after an earlier failed sync as sufficient permission
for blind continued writes; they do not retroactively prove what survived that sync.

**Residual staging remains an explicit availability limitation.** An old marker
plus a complete or partial unselected block, any pending marker, or unexplained
files rejects through normal open and reconciliation. Artifacts are retained;
no marker is automatically promoted, old history truncated or unknown file removed.
Thus the tested restart outcomes are a complete independently selected old/new clean
history or explicit rejection, not universal automatic crash recovery. MB-03C must
specify and verify an explicit repair operation before resolving those leftovers.

Bounds carry forward: at most 4,096 complete blocks, 16 MiB MRB2 and 1,048,576
cumulative state/configuration items. Successful selected encoded payload rises
only by the 48-byte envelope difference to **19,559,132 bytes**; it is not a resident
memory limit. An in-progress append can additionally retain a 280-byte pending marker.
Strict reopen rejects residual files before treating them as selected payload.
Nonregular files, static symlinks, hardlinks and accidental root replacement reject;
trusted parents/cooperating local processes remain assumptions rather than hostile
same-user namespace protection. Linux support is not certification of all mounts.

Private tests cover eleven append boundaries, eight reconciliation boundaries via
both consuming recovery and writer reopen, and nine initialization boundaries.
Other cases explicitly retain partial block/selector bytes, detect changed markers
and roots, check stale-pin rejection, and continue with another valid child only
after clean recovery. Three new pipe-ordered process cases check append-profile
writer Drop, process termination and shared-reader exclusion. The prior process
tests and ignored helper remain; parent test isolation preserves actual child
concurrency. These scenarios are not extra unique tests or actual power-loss,
torn-write, disk-firmware or comprehensive syscall-failure simulations.

**At the MB-03B checkpoint, next was MB-03C explicit residual-staging recovery**, then reviewed canonical
development fork choice, durable membership-aware reorganization and peer
reconstruction. F08 remains partly open in those scopes and for legacy behavior.
F09–F11, final human consensus, private matching, actual devices and Gate A/B remain
unmet. No founder reserve, marketing allocation, mint, fee or reward is enabled.

## Historical MB-03C explicitly authorized residual recovery update

**VERIFIED COMPONENT on Rust 1.98.0 with exact-source CI.** The source commit is
`e8302052ab724f59ff4a212dc64aea9575a6d4b3`, with tree
`dc9eb2c02436f0952ac54593ba8d02317e9d9314`, matching the locally tested tree.
Local gates passed **707 workspace tests**, **three documentation tests**, **194
storage release tests**, all **eight independent references** and **nine examples**;
one subprocess helper remains intentionally ignored outside its owning tests.
Exact-source [push run 44748718086](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748718086)
(job `136330677456`) and [PR run 44748724632](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748724632)
both completed successfully. The full 216,857-byte push log was inspected, including
the exact checkout SHA and successful workspace, release, reference, example and
audit steps. The 49 additions are mapped in section 12 of
[the test matrix](MEMBERSHIP_TEST_MATRIX.md). These runs verify the implementation
source; they do not establish a later documentation head's CI result.
[CURRENT_STATUS.md](../CURRENT_STATUS.md) records current publication evidence.

The [recovery contract](STORAGE_RECOVERY_V2.md) and
[recovery implementation](../crates/mirror-storage/src/v2/append/recovery.rs) add a
separate explicit operation to profile 2. Neither immutable `StoreV2` nor normal
append-profile open, inspection or clean reconciliation acquires hidden repair
behavior. No MNM2, MRB2, MSS2 or MSU2 byte, profile, checksum domain or fixture changes.
There is no on-disk intent, new journal or trusted cache.

### Independent intent limits repair authority

`AppendRecoveryRequestV2` owns the exact proposed child and independently retained
parent/candidate checkpoints. Its constructor checks network agreement, bounded
adjacent heights and generations, and header relationships. Both `Complete` and
`RetainParent` then replay the actual parent and accept the exact requested child
through `ChainV2` before any mutation. The executed candidate must equal all
external pins. Canonical old and new selectors are re-derived from those complete
histories, including manifest and block-byte metadata; a checksum-valid selector
with different metadata does not suffice.

The caller's request is authority to change the two named residual paths for that
candidate, not proof that an earlier process created those files. An empty file
or common prefix cannot prove provenance. Prefix equality limits the allowed bytes;
it cannot make a filesystem- or peer-suggested request trustworthy. Independent
intent must have been retained elsewhere or reestablished through an explicit
trusted operator decision. This component provides neither persistent custody of
that intent nor an operator interface. Loss of the independent request remains an
availability problem; a pending checksum cannot reconstruct missing authority.

### Explicit state choices and synchronized publication

With the exact parent selected, recovery recognizes no tail, an exact empty/proper
candidate prefix, or the exact full candidate; a pending selector is allowed only
when the full intended candidate file is present. The pending bytes must themselves
be an exact prefix or full encoding of the independently derived candidate MSU2.
Unknown filenames, extra heights, mismatched/trailing bytes, unsafe file types or
aliases, missing/corrupt parent history and unrecognized selected markers reject.
A selected candidate is accepted only with a complete exact child and no pending
selector. No holes in selected history are filled by guessing.

`Complete` first synchronizes the validated parent prerequisites, completes only
missing canonical suffixes without truncation, verifies and synchronizes each
completed file, synchronizes the block directory, and rechecks the old selector
and prepared artifacts before selector rename. The stable root is synchronized
before a final strict clean-state replay and resynchronization. An already selected
clean candidate is an idempotent success; no second child or generation is added.

`RetainParent` is permitted only while the exact parent remains selected. It checks
current safe file identities and exact observed bytes, removes only the known
pending selector, synchronizes the root, then removes only the known unselected
child and synchronizes the block directory. This order preserves recognized retry
states if cleanup is interrupted. It does not erase accepted history or roll back
an already selected candidate. A clean parent is an idempotent success after full
validation and synchronization. Both actions return the complete independently
selected ledger or no usable handle; final replay includes all accounts, membership,
permanent records, approvals, population and resource counters.

Static recovery acquires exclusive root ownership before inspection. Consuming
recovery retains the same held writer lock, including when the writer was fenced
by an earlier append failure. A read-only handle and an unrelated retained memory
checkpoint reject. Every error returns no handle; before publication it may leave
incomplete known staging, while publication uncertainty requires independent
revalidation. Root replacement, observed changed markers, newly unexplained layout,
changed prefixes and descriptor aliases are checked at the specified boundaries.
Those checks are defense in depth under trusted parents and cooperating processes,
not protection against arbitrary hostile same-user races.

### Resource and failure evidence boundaries

The candidate's complete intended bytes count toward the unchanged 16-MiB history
budget even when its disk file is only a short prefix. Parent count plus one remains
at most 4,096 blocks, and every replay/child transition uses the unchanged
1,048,576 cumulative state/configuration-item limit. No pruning, history eviction
or authoritative cached counters recover capacity. Distinct recognized encoded
input is at most **19,559,412 bytes**, including both selectors. A separate enforced
read meter permits at most **156,508,096 charged bytes** per recovery call, including
comparison reads, conservative final-loader charges and one-byte sentinels.
Neither value is total resident memory, syscall count, measured I/O, write volume
or an availability guarantee. Layout enumeration and metadata have their own finite
bounds; no scan/retry loop grows with unbounded residual data.

The new test inventory is **39 integration, nine private fault and one real-process
case**. The private suite checks 28 completion boundaries from three initial shapes
(84 scenario iterations), 21 retention boundaries, 16 already-selected
resynchronization boundaries, and both actions after 11 original append boundaries.
Other private cases change roots, markers, layout or prefix bytes; introduce an
alias after opening; and exhaust the read allowance before payload access. These
iterations remain nine tests. The process case exercises recovered-writer ownership
through child Drop and termination, preserving the existing parent-fork isolation.
No sleep, weakened ownership assertion or production retry is introduced.

Failure hooks inject errors after selected real operations; staged prefix fixtures
exercise partial-data shapes. These checks do not simulate arbitrary torn kernel
writes, physical power loss, firmware behavior or every syscall failure. A successful
retry establishes the currently replayed and synchronized result, never retroactive
proof that a prior failed synchronization succeeded. Linux local filesystems/Ubuntu
checks do not certify untested WSL, DrvFS, native Windows, NFS or SMB mounts.

F08 gains this bounded profile-2 recovery path, but legacy storage, arbitrary
corruption, interrupted initialization, missing/corrupt selected markers, unknown
artifacts, multi-child leftovers and durable reorganization remain outside it.
F09 network sessions, F10 biological uniqueness/privacy, F11 final economics and
human finality remain open. No issuance, founder reserve, marketing allocation,
real biometric capture or anonymity guarantee follows from recovery.
**At the MB-03C checkpoint, MB-03 remained IN PROGRESS and MB-03D was next:**
canonical development fork choice, before durable membership-aware replacement and
peer reconstruction. Gate A and Gate B remained UNMET.

## Historical MB-03D canonical development fork-choice update

**All required local acceptance gates passed.** Source:
`b9a2e870996fcafe1128dcb5cfdfb6b01b158ded`; source tree: `dd0a3fc29f429dcd4999261801c2bc7897d6aace`. The implementation and
[contract](FORK_CHOICE_V2.md) add comparison of complete accepted V2 branches,
without changing any existing format, dependency, fixture or storage behavior.
Rust 1.98.0 with locked dependencies passed **735 workspace runtime tests**
(707 retained + 28 new) and **six documentation tests** (three retained + three
new ownership/type compile-fail examples). One subprocess-only helper remains
intentionally ignored in ordinary enumeration and is invoked by its parent tests.

Release gates passed **121 core**, **234 personhood plus three doctests**,
**49 state**, **23 consensus**, **85 chain plus three doctests**, and **194 storage**
tests. All **eight independent reference checks** and **ten examples**, including
`fork_choice_v2_demo`, passed. **105 retained source, dependency, reference and
fixture files remain byte-identical.** Release repeats are not additional unique
tests. These local results are separate from exact-source and documentation-head
CI evidence:
Published-source [push CI 45703714223](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703714223)
and [PR CI 45703726306](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703726306)
both passed. Full push-job `139520118282` logs confirmed the exact implementation
checkout, **735 workspace tests, six compile-fail doctests, all required release
suites, eight independent references, ten examples and the allocation/runtime-data
audit**. The single intentionally ignored subprocess helper is invoked by parent
tests and is not another pass. Release reruns are not additional unique tests.
MB-03D is **VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON**. Subsequent
documentation-head CI is checked separately and recorded on PR #1.

### Independent identity and accepted inputs

`compare_branches_v2(expected, current, candidate)` requires an independently
selected configuration hash C and exact genesis hash. It checks current C, current
genesis, candidate C and candidate genesis in that order before any ancestry work.
Matching two inputs to each other alone would not select the intended network;
deriving expected identity from untrusted candidate data remains an application
trust-boundary failure. A different valid network can be selected only by changing
that independent caller choice explicitly.

Both inputs must be immutable borrows of already accepted `ChainV2` values. Their
private construction/append/replay paths established work, parent, time, body,
signature, complete ledger and resource invariants. The comparator cannot accept a
raw block, advertised height, claimed score, snapshot or storage marker in place
of an accepted chain. A rejected block cannot raise the accepted chain's score.
Comparison does not repeat full execution or turn those invariants into an unsafe
public setter.

### Fixed-target score, ties and ancestry

The current profile requires the same configured literal difficulty bits for every
accepted block. Within the independently selected identity, height is therefore a
normalized ordering of accumulated constant target-based work. Genesis contributes
the same common term to either branch. The comparator reads accepted height; it
never sums attacker-supplied scores, multiplies large integers or counts actual
mining attempts. Existing 4,096-block admission bounds the score to `0..=4095`.
A future variable-target or human-weighted profile must explicitly replace this
rule, rather than reusing height as an undocumented approximation.

A candidate wins exactly when its score is strictly larger. Equal scores keep the
input designated current, including when the candidate's realized hash is lower,
its timestamp is later or its membership/population counts differ. This is not a
global ordering of equal-score branches: two peers can retain different equal
branches. At the finite cap or a terminal timestamp, further extension can be
impossible, and this component supplies no eventual-convergence guarantee. Hash
rate dominance remains possible in this development PoW rule; this work does not
implement equal-human weight, rewards or MB-06 finality.

Common ancestry is the last matching prefix of actual accepted block hashes,
never agreement of account bytes, membership roots, claimed height or timestamps.
The comparator scans at most 4,096 pairs and hashes at most 8,192 fixed headers,
using constant additional state and stack hash inputs. It allocates no collection,
clones no branch, performs no disk I/O and executes no additional ledger steps.
The two inputs retain their separate encoded-history and cumulative-state budgets.
Existing collision-resistance and signature assumptions remain in force.

### Borrowed complete state and publication boundary

The private `BranchComparisonV2` result borrows both inputs. Safe Rust rejects
mutating either branch while that result remains in use. Its selected reference
points to the complete actual input branch: blocks, manifest, accounts, membership,
permanent records, approvals, population peaks and resource counters stay together.
Tests use genuine membership-divergent histories with identical account bytes;
a preference does not merge orphaned approvals or higher population peaks from
the other branch. Empty-block expiry remains part of the selected history too.

Public choice/relation enums and copied checkpoints remain diagnostic data. They
can be copied or become stale after the borrowing comparison is no longer used;
they are not publication certificates. Returning a branch reference cannot alter
an existing store's canonical selection. A future durable replacement API must
recheck identity, the current selected endpoint and the complete candidate while
holding exclusive stable-root ownership, then enforce its own publication and
interruption rules. MSU2 local generation is not branch weight, and MB-03C's
one-child recovery must not be stretched into an implicit reorganization API.

The new tests cover both input identities and error order, identical/prefix/
divergent relationships, sticky ties in both argument orders, genesis/deep common
ancestors, complete membership/counter retention, invalid-block exclusion and the
4,096-block cap. Compile-fail examples demonstrate immutable current and candidate
borrows plus rejection of a raw `BlockV2` argument. This is scoped engineering
verification, not an independent audit or evidence of deployed network convergence.

F06/F07 gain an explicit validated-branch preference boundary. F08 durable canonical
replacement, F09 peer sessions/reconstruction, F10 private biological matching,
F11 economics and human finality remain open in their stated scopes. Legacy V1
findings remain historical; no source finding is globally closed by this addition.
**At the MB-03D checkpoint, MB-03 remained IN PROGRESS and MB-03E was next:**
durable membership-aware branch replacement with reviewed publication/recovery
under the stable lock, followed by fresh-peer reconstruction.
**Gate A and Gate B remained UNMET.** No mint/grant, founder/marketing
privilege, actual biometric capture or anonymity guarantee was introduced.

## 1. Assets and trust boundaries

Protect canonical account balances/nonces; membership status/approval authority;
bootstrap and rule configuration; activation/population counters; future grant
uniqueness; the authoritative durable tip; availability; private evidence and keys.

Treat network frames, disk files, supplied config files, applicant evidence,
reviewer statements and advertised peer heights/roots as untrusted until the
appropriate checks complete. A locally trusted deployment manifest chooses a
network; it is not replaced by whichever config a peer advertises. The operator-
selected A1 control directory is trusted local custody of the full checkpoint and
one specific replacement intent after bounded validation/replay. Its record
checksums detect corruption but cannot authenticate an owner who can replace all
local state. The intended separate control/store path pair and network remain
operator inputs. Seed people are an explicit initial trust assumption, not proven
unique by self-signatures.

The initial development profile assumes synthetic evidence, a single explicitly
configured network, a bounded dataset and nonvaluable test balances. It makes no
claim of honest majority of real humans, private biometric matching, or finality.
Until MB-06, hash-rate dominance remains possible in the existing PoW foundation.

A device can provide measurements; an application binds a session; a matching
operator can report a comparison; humans attest to review; nodes enforce encoded
rules. None of those steps proves the truth of all previous steps. In particular,
valid signatures are not proof of biometric uniqueness or informed consent.

## 2. Historical source findings and bounded reproduction recipes

These are source-derived observations or explicitly labelled design inferences,
not claims of an executed exploit against a deployed network. Existing funds and
local chain files have not been accessed or modified.

| ID | Observation at originally reviewed source | Consequence / reproduction recipe | Planned owner |
|---|---|---|---|
| F01 | `RegistryRules::new` hashes network+threshold only; ordered seeds are supplied separately to `MembershipRegistry::new` | With identical network/threshold, reuse an applicant request across two different valid seed configurations. Rules/signing context does not distinguish them. Bootstrap membership differences can still change state roots; the missing protection is signature context binding. | MB-01B B1, R02–03 |
| F02 | `apply` and `apply_batch` have no record, pending-count or event-count limits; codec bounds one log to 4 MiB | Feed many distinct self-signed requests through repeated direct calls or many valid logs. The log limit is not a total registry/mutation budget. | MB-01B B3, R20–22 |
| F03 | Requests and approvals contain no chain anchor or height expiry | Preserve a valid request and approve it much later; source has no freshness deadline to reject it. Authorization expiry will not itself validate sensor freshness. | MB-01B B2, R09–12 |
| F04 | Approval immediately marks a member Active; the next event checks the mutable registry | With threshold 1, A approves B and B approves C in the same batch. This is allowed by the current library, but is unsuitable for the proposed parent-snapshot eligibility model. | MB-01B B3, R13–15 |
| F05 | `population_high_water` updates at each individual approval | Activation followed by withdrawal in one batch can record an intermediate peak. Current code has no block boundary; new end-of-block development accounting must not be confused with final monetary qualification. | MB-01B/02, R18 |
| F06 | V1 `ChainState` commits accounts only; `Block` carries signed transfers only; `apply_transaction` rejects other kinds | The library is not live personhood state. Simply adding an API or storing a second log would leave consensus blind to membership. | MB-02, R23–27 |
| F07 | V1 `from_persisted_blocks` checks genesis fields/PoW/state but receives no pinned exact expected genesis hash | Another valid nonce for the same genesis fields is not excluded there. V2 must pin exact genesis and bootstrap/config context, including on disk reopen, not rely only on peer Hello comparisons. | MB-02, R01/R03/R28 |
| F08 | `BlockStore` assumes one writer, uses `fs::read`, and performs two directory renames during replacement | No enforced writer lock, no read-byte cap, and errors after publication can leave in-memory and durable outcomes ambiguous. Recovery based on directory existence is not proof of complete old/new atomic publication on all filesystems. | MB-03, R29–30 |
| F09 | TCP receiver reads declared body before semantic header validation; timeouts are socket read/write timeouts; node accumulates candidate blocks | Payload bound does not establish complete-session limits, resistance to slow fragments, or bounded total candidate-chain memory. | MB-04, R31–32 |
| F10 | Request/key/evidence equality checks and public approval relationships are the implemented personhood boundary | Fresh keys/new commitments from one person are not detected; identities and relationships are linkable. Do not present this as private biological deduplication. | MB-05/09/12, R33 |
| F11 | Account executor deducts transfer fees and returns totals; it does not implement the agreed future reward distribution | Reusing that behavior as the final fee economy would silently choose monetary behavior. V2 integration uses explicitly zero-fee test transfers until MB-07 decisions. | MB-07, R08/R34–35 |

These rows preserve the original findings. In particular, F08 still describes the
unchanged legacy `BlockStore`; the separate MB-03A/B/C V2 mitigations above do not
retroactively close legacy findings or later repair/reorganization requirements.

Source anchors (relative links show current branch; original revision above fixes
which contents were reviewed):

- [Personhood rules, events, mutation and root](../crates/mirror-personhood/src/lib.rs)
- [MPH1 size limits and structural-only decoder](../crates/mirror-personhood/src/codec.rs)
- [Existing 26 personhood tests](../crates/mirror-personhood/src/tests.rs)
- [V1 transfer fields/signing preimage](../crates/mirror-core/src/transaction.rs)
- [120-byte V1 header](../crates/mirror-core/src/block.rs)
- [MRB1 codec](../crates/mirror-core/src/codec.rs)
- [Account-only execution and state](../crates/mirror-state/src/lib.rs)
- [Genesis, acceptance and fork comparison](../crates/mirror-chain/src/lib.rs)
- [Disk publication and recovery](../crates/mirror-storage/src/lib.rs)
- [Framed TCP read/write](../crates/mirror-network/src/lib.rs)
- [Hello/GetBlock/BlockData protocol](../crates/mirror-protocol/src/lib.rs)

## 3. Adversaries and required protections

**Malicious applicant:** substitutes evidence, submits replayed requests, changes
keys, creates simultaneous enrollments, floods pending state or cancels to reclaim
a grant. V2 authorization/context/limits can address some of these; real-human
matching and economic eligibility remain explicit external gates. Tombstones and
activation ordinals are not a complete recovery/biometric deduplication design.

**Dishonest or compromised reviewer:** approves false humans, signs twice, shares
keys, withdraws to manipulate counting, or colludes with the applicant. Enforce
signatures, exact-target binding, distinct reviewers, sequences, parent eligibility
and block-end checks. A coalition with valid keys can still lie; assignment,
challenges, availability and real-admission fault assumptions remain D-02.

**Malicious proposer/peer:** reorders or withholds operations, changes roots,
claims a false height/population, serves inconsistent branch data or mixes networks.
One executor, full commitments, exact config/genesis and bounded decode reject
invalid transitions. They do not establish censorship resistance, equal selection,
finality or progress against a hostile majority. Those belong to MB-06.

**Compromised capture/matching system:** replays recordings, swaps applicants,
returns stale/invented no-match reports or leaks evidence. Keep archives off-chain;
require explicit capture/session/profile/population binding and private access
controls in MB-08/09. A hash/signature alone does not establish capture provenance.
A matcher outage must not translate to NO_MATCH or automatic approval.

**Storage/process failure or competing writer:** crashes between publication steps,
loads an older valid snapshot, corrupts a file or races the writer. MB-03A/B provide
cooperative directory locking, bounded inputs, independently selected metadata,
full reconstruction and one-block append publication with failure fencing. Complete
clean selections can be revalidated and synchronized before further writes. Normal
opening still rejects residual staging. MB-03C's separate explicit API recognizes
only a retained parent/candidate request and exact authorized prefixes; it may
complete that child or remove its known unselected staging while the parent is
still selected. Unknown/corrupt/unsupported artifacts remain preserved and rejected.
E2a/E2b add whole-branch publication and explicit recognized replacement recovery;
A1 persists the exact local authority before mutation, and A2 exposes only explicit
finite attachment, observation, synchronization and recovery. Arbitrary repair,
all physical failure modes and hostile local-owner defenses remain unfinished.
A valid old record is not proof of latestness; the trusted caller must retain
independent network selection and full local branch/generation authority.

**Observer:** links applicant/reviewer keys, timing and recurring commitments.
This prototype has public relationships. Do not claim total anonymity or publish
stable fingerprint hashes, encrypted archives, evidence URLs or personal records.
Cryptographic commitments require an appropriate randomized construction outside
this register; the ledger cannot verify how an opaque commitment was produced.

## 4. Security invariants for remaining integration

- The same trusted config, parent and ordered operations produce identical roots
  on independent nodes, without device/matcher queries or local clock decisions.
- An invalid operation/block changes no balances, membership, sequences, indexes,
  activation counters, population records or authoritative tip.
- Context includes seed order/contents, economic test profile and all limits;
  changing any consensus rule requires an explicit new version/configuration.
- A person admitted in block h cannot authorize another admission in that block.
  A same-block withdrawal removes that signer's approval from final activation
  counting, regardless of the approval's earlier position.
- Only the selected canonical history contributes state/counters. Orphaned peaks
  and approvals are discarded together. Future finalized history is never replaced
  by the old longest-chain path.
- Untrusted disk metadata, peers and caches cannot choose their own trusted
  C/genesis or final height/tip/ledger root. A1's operator-owned record retains a
  previously confirmed full local checkpoint; external network identity and full
  selected replay/canonical metadata checks still apply on clean restart. No
  highest-generation scan replaces this authority or supplies global freshness.
- A peer may advertise an untrusted candidate endpoint, but that claim is only a
  transfer-consistency expectation. MB-03F1 derives its returned checkpoint from
  actual complete execution under independently selected C/genesis. It does not
  turn the peer claim into trusted replay selection. MB-03F2 separately compares
  that completed candidate with the independently pinned local current history
  before owned publication; a connection's completion is not a write permit.
- Fixed-header admission and the remaining owner allowance must pass before
  allocating or reading a peer-declared payload. Attempts and actual partial I/O
  remain charged across failures. One absolute deadline covers connect/accept,
  request, complete response and EOF; fragments cannot refresh it. Matching End
  and EOF both precede candidate release. These are finite-owner bounds, not
  process-global admission or preemptive CPU deadlines.
- An uncertain storage-publication error yields no usable writer. Initialization
  and reconciliation failures return no handle; append failure retains ownership
  but fences chain/selection access and further writes. Complete visible selection
  must be checked against retained external pins and resynchronized before writing;
  an error must not be relabeled proof that nothing reached disk. Normal opening
  preserves and rejects staging. Explicit recovery may change only the exact
  recognized residual paths authorized by independent parent/candidate pins and
  the exact child; mismatched or unexplained bytes stay untouched. It must never
  silently roll back an already selected candidate or return mixed ledger state.
- Persist and revalidate both exact accepted histories and local pins before
  storage mutation; persist confirmed selection before retiring that intent.
  Startup with intent must report unresolved work, not choose completion or repair.
  Candidate confirmation staging authorizes only an exact suffix and requires a
  clean selected candidate; a selected candidate cannot roll back. Control/store
  ownership uses stable separate roots and control-before-store lock ordering.
- Parse the complete bounded command before filesystem or socket effects. Attach
  control only to an already initialized store under all independent pins. Open
  ready A1 ownership before contacting a sync peer and keep both roots through
  receive/adoption. Status must distinguish uninspected intent from a verified
  current store and must not choose or perform recovery.
- Keep diagnostics distinct from durable outcomes. A failed output channel after
  init/sync/recover may hide an already completed operation; inspect status before
  another explicit action. Neither failed transport nor lost stdout authorizes an
  automatic retry, initialization, recovery or rollback.
- No runtime operation in this integration scope can mint or distribute grants.
  Unknown operation kinds cannot be treated as harmless payloads.
- No raw/encrypted biometric archive is intentionally accepted by the client/wire
  schema; arbitrary 32-byte fields are not a proof against malicious covert data.

## 5. Residual risks and launch gates

Resource caps can themselves be exhausted, denying honest access. Parent-snapshot
eligibility prevents same-block propagation but does not stop collusion over many
blocks. Request expiry bounds authorization age but does not refresh old readings.
State-root commitments detect disagreement, not fraudulent true-looking evidence.
Static source review and old passing tests do not establish the proposed fixes.

D-01/D-04 gate issuance and fees; D-02 gates real admission; D-03 gates human-round
consensus; D-05/D-06 gate sensitive evidence and real-device recognition. Devices,
biometrics, anonymity, identity/key recovery and inheritance are not silently implemented here.
Founder-led startup is explicitly centralized initial trust, with no claimed
permissionless security until the specified handoff and fault model are tested.

## 6. Original review outcome and current continuation

MB-01A design deliverables identify integration blockers, choose a compatibility
path, and map defenses to tests. They did not close F01–F11. The original next task
was MB-01B/B1. B1–B4 and MB-02A's Transfer/unified operation and BlockV2 codecs,
commitments and independent vectors are verified components. MB-02B adds atomic
ledger staging and combined commitments; MB-02C connects them to actual validated
genesis, blocks and complete externally selected replay. **MB-02 is a
VERIFIED_COMPONENT within that bounded in-memory scope.** Local and inspected
exact-source CI evidence is recorded above; later documentation checkpoints are
tracked separately in [CURRENT_STATUS.md](../CURRENT_STATUS.md).
MB-03A implements immutable selected-store initialization, cooperative writer
ownership, bounded pinned reopening and selected initialization-failure cases.
MB-03B adds the distinct append profile, complete one-child publication, failure
fencing and checked clean-store resynchronization. MB-03C adds explicitly authorized
completion or abandonment of recognized interrupted append staging. Implementation/test scope is
recorded above; exact publication/CI remains separate
[CURRENT_STATUS.md](../CURRENT_STATUS.md) evidence.
MB-03D adds the independently pinned, fixed-target comparison of complete accepted
branches described above. MB-03E1 adds bounded MSR2 metadata and pure preparation
under independent current-selection pins. MB-03E2a adds the bounded replacement
store described below, including full-state publication and exact old-copy cleanup.
MB-03E2b adds explicit recovery of recognized partial staging/cleanup, with both
exact accepted histories retained as authority.
MB-03F1 adds bounded peer-history framing and complete in-memory candidate
reconstruction. MB-03F2 adds finite TCP sessions, actual peer-process validation,
independent whole-branch adoption and explicit recovery with retained histories.
**MB-03F2, MB-03F and MB-03 are VERIFIED_COMPONENT in that finite scope.**
**MB-04A1 now supplies durable local node control and explicit restart recovery**,
preserving independent C/genesis and trusted current pins plus both exact histories
outside the strict store root before mutation. **MB-04A2 adds finite V2 operator
commands** for attachment-only init, read-only status, one sync attempt and explicit
recovery. **MB-04B1 adds finite fixed-history serving and draining shutdown** under
one retained controller and one cumulative F2 owner. B2's receive/adopt/serve
composition, B3's bounded explicit-peer acquisition and B4's finite repeated rounds
appear below with separate source and acceptance evidence. The B4 update records
the exact current continuation. Arbitrary corruption repair, indefinite reconnect,
broader scheduling, transaction pool, wallet,
human finality and Gate A/Gate B remain unfinished; MB-04 is still IN_PROGRESS.
Keep MPH1/MRB1 unchanged and carry findings forward until linked passing tests
demonstrate the required scope, rather than inferring complete deployment security.

## Historical MB-03E1 replacement-selection and preparation update

The [replacement contract](STORAGE_REPLACEMENT_V2.md) adds the fixed 280-byte MSR2
profile-3 record and a pure preparation API. The nested MSS2 record, both existing
stores and their recovery contracts remain unchanged. Focused verification passed
31 selection-codec cases and 21 preparation cases; all nine independent reference
checks passed. Two compile-fail examples cover the borrowed current and candidate
chains. Its historical full-suite, exact-source and CI evidence remains in
[MASTER_PLAN.md](../MASTER_PLAN.md), section 22; earlier green checkpoints do not
verify a newer implementation source.

MSR2 decoding requires independent configuration, genesis, height, tip, ledger-root
and local-generation expectations. A checksum-correct old record or competing
same-height membership branch cannot replace those pins. Exact length precedes
fixed-offset reads; unchanged nested bounds restrict height and generation to
4095. Initialization can use generation zero at any accepted height. Generation
counts proposed local publications, never work or human weight, and structural
validity does not prove any publication took place.

Preparation reuses accepted-branch comparison, checks the complete current
checkpoint/generation, and independently derives the entire canonical current
selector. Thus matching endpoint fields and recomputed public checksums cannot
hide altered manifest metadata or block-byte totals. Only a strictly preferred
accepted candidate produces a plan. The candidate retains its own exact manifest,
whole block history, accounts, membership, reservations, approvals, population
peaks and resource counters; no state is combined across branches.

The plan immutably borrows both complete histories and cannot mutate either input.
It performs no filesystem operation or durable selection, and copied metadata is
not a publication permit. It encodes bounded manifests during preparation, so this
step is not allocation-free. Independent pins and borrows do not establish global
freshness, hold a storage lock or prevent an unrelated store from changing.

At the E1 checkpoint, F08 therefore remained open for MB-03E2: stable-root ownership,
revalidation under that ownership, a bounded layout, synchronized publication,
cleanup interruptions and write fencing were still required. No crash-safety result
followed from in-memory preparation. E2a's narrower durable implementation follows;
F09–F11, human finality and the existing policy gates remain open.

## Historical MB-03E2a durable replacement and strict reconciliation update

The [replacement-store contract](STORAGE_REPLACEMENT_STORE_V2.md) specifies
`ReplacementStoreV2` around unchanged MSR2 profile-3 metadata. The implementation
adds explicit initialization, independently pinned writer reopening/shared
inspection, complete accepted-branch replacement, failure fencing and consuming
clean-state reconciliation. At that checkpoint it did **not** implement repair of
interrupted replacement staging or cleanup. Ordinary opening/reconciliation still
rejects and preserves those states; the separate E2b operation is recorded below.

Written coverage adds **49 test functions: 32 filesystem integration, 14 private
fault/read-budget and three process ownership tests**, mapped in
[test matrix section 15](MEMBERSHIP_TEST_MATRIX.md#15-mb-03e2a-durable-replacement-and-strict-clean-state-reconciliation).
This description does not infer a passing total or CI result. Exact source/tree,
observed full-suite results and publication evidence belong to
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md),
section 23. Existing reference fixtures, legacy store behaviors and policy gates
remain unchanged. This is an engineering review, not an independent security audit.

### Selected authority and complete state

Independently retained configuration/genesis, complete chain endpoint and local
generation remain mandatory. The clean generation pathname is computed from the
independent pin, not chosen by a directory scan or numerically greatest generation.
The entire canonical MSR2 must match the replayed chain, including exact manifest
metadata and block-byte totals. Public checksums and matching endpoint fields
cannot authorize different metadata or substitute for actual replay.

Replacement accepts a borrowed complete `ChainV2` candidate. It checks strict
existing fixed-target preference, fully replays current selected disk history under
the held lock, compares the complete current state, and repeats checked preparation
against that replay before writing. Whole candidate equality is checked before and
after selection publication and at final reconstruction. Accounts, membership,
permanent reservations, approvals, sequences, population history and resource
counters move together. Equal-account but different-membership tests specifically
guard against retaining losing-branch effects in the selected candidate.

Generation remains local bookkeeping: zero at initialization, one increment per
strictly preferred branch publication, no more than selected height. It is not
hash power, human weight, global freshness or finality. A larger directory or
well-formed pending record cannot select a branch. Sticky equal-score branches and
the existing finite-chain/timestamp limits remain separate consensus constraints.

### Stable ownership, publication and retirement

The root inode remains owned continuously. A clean root contains one selected
MSR2 and one exact generation directory. Replacement temporarily permits only
the checked current and next generations and one pending selector. The candidate's
own manifest and full block history are synchronized, along with their directories
and root, before selector publication. A same-root marker rename changes visible
selection; root synchronization establishes the required durable boundary before
any old-copy deletion.

After publication, the complete candidate is replayed again. The entire old
generation is independently replayed and compared with its exact accepted source
before the first deletion. Each old file is compared again immediately before
unlinking. Only prescribed exact files and subsequently empty directories are
removed; no wildcard or recursive removal exists. Required directory synchronization
follows block removal, blocks-directory removal, manifest removal and generation
removal. Full clean selection validation/synchronization precedes in-memory advance.

These steps remove only the old unselected branch copy. They do not prune selected
history, discard selected permanent reservations to obtain capacity, or preserve
an automatic old-branch archive. Callers requiring an archive retain it separately.
Source tests corrupt the candidate after publication or corrupt a late old block/
manifest and check that no earlier old file is deleted before these complete
preflight checks pass.

### Failure, retained data and the remaining recovery gap

Disk discrepancies and mutation failures fence all ordinary chain/selection access
and further replacements. Last-confirmed checkpoint is explicitly the old confirmed
memory state, not an assertion about the now-selected disk state. Errors after a
publication attempt, including cleanup failures, remain potentially published
outcomes. A caller must not assume failure means balances or selection stayed old.

Consuming reconciliation retains the same exclusive lock and returns a writer
only after complete independently pinned clean replay and synchronization. Any
error returns no usable writer. Ordinary open, inspection and reconciliation never
promote pending metadata, delete a second generation, finish partial files or roll
selection back. Unknown files, corrupt selected metadata and unexplained staging/
cleanup residue are preserved and rejected. This can deliberately leave a store
unavailable pending explicit recovery; a clean-state API is not a recovery promise.

**E2b must require both exact accepted old and candidate histories**, their
independently retained full checkpoints/generations and an explicit action. Once
cleanup removes any old file, its endpoint alone cannot authenticate the remaining
partial tree. The old exact history must have been retained or independently
reconstructed; a highest-generation scan or guessed old record cannot replace it.
An already selected new generation cannot be rolled back through a retain-old
action. Future recovery must compare every residual byte before mutation and keep
continuous ownership; it must not relax selected-history validity or old caps.

Individual file synchronization does not certify persistence of every directory
entry before the directory synchronization completes. Interrupted candidate staging
and old cleanup can therefore leave arbitrary subsets of prescribed files rather
than a prefix of block heights. The contract records this constraint for E2b,
while E2a rejects all such residue. Boundary injection and successful reopen tests
do not certify every physical power-loss or storage-device failure behavior.

### Bounded reads and scoped filesystem assumptions

At most two generation payloads and two selectors coexist: **39,118,264 encoded
bytes** at retained per-history caps. Every payload read uses a shared per-call
meter that charges the checked length plus a growth sentinel. The allowance is
**313,011,680 bytes** and is never reset within a call. Exhaustion or overflow
rejects before payload open. Root, generation and block enumeration have finite
entry bounds; individual lengths and aggregate MRB2 bytes are preflighted before
block payload allocation. A sparse oversized file is rejected without reading or
truncating its payload.

A successful replacement performs **five full actual-chain replays**, each under
the retained block/byte/state-work limits. Four further exact payload comparisons
and seven selector reads give a conservative **176,068,508-byte** successful-call
read upper bound. This is below the enforced allowance; it is not a resident-memory,
metadata-operation, latency or write-volume limit. No unbounded retry/replay loop
exists. Reconstructed state remains subject to the unchanged complete-chain caps.

Three additional tests use real cooperating processes to exercise writer/shared
ownership and the lifetime of a replaced/reconciled writer through descriptor drop
or process termination. Private fault hooks check ownership throughout the operation.
The existing trusted local owner/parent and cooperative Linux filesystem model
still applies. Static path/type/link checks and root-inode comparisons do not
establish hostile same-user race resistance. Untested WSL/DrvFS, native Windows and
network filesystems are not represented as supported or verified.

At the E2a checkpoint F08 was mitigated for the bounded successful replacement and
clean-reconciliation path, **not closed for arbitrary interruption repair or
deployment**. MB-03E2b, fresh-peer reconstruction, persistent V2 node integration,
human finality, private biological matching and the other policy/security gates
remained unfinished. E2b's subsequent bounded recovery scope is recorded below.
No founder/marketing privilege, monetary formula, mint/reward API, real biometric
capture or anonymity guarantee is introduced. MB-03 and Gate A/B remain incomplete.

## MB-03E2b explicit interrupted replacement recovery update

The [recovery contract](STORAGE_REPLACEMENT_RECOVERY_V2.md) and
[implementation](../crates/mirror-storage/src/v2/replacement_recovery.rs) add
`ReplacementStoreV2::recover_interrupted` and consuming `recover`, with explicit
`Complete` or `RetainOld` actions. Normal opening, shared inspection and clean
reconciliation retain their existing strict behavior. No MSR2/MSS2/MSU2, MNM2 or
MRB2 bytes, old fixture, dependency or monetary rule changes.

This section records the reviewed implementation boundaries. Exact source/tree,
actual test counts, release gates, reference checks, examples and CI results remain
separate [CURRENT_STATUS.md](../CURRENT_STATUS.md) evidence, with coverage mapped
in [the test matrix](MEMBERSHIP_TEST_MATRIX.md). It is not an external security audit.

### Both histories and independent intent constrain recovery

`ReplacementRecoveryRequestV2` immutably borrows both complete accepted histories
and retains independently supplied old/candidate checkpoints and generations.
Construction checks the request relation; recovery repeats network, strict
preference, exact endpoint and adjacent-generation checks under owned storage.
The candidate must be strictly preferred by the existing development rule, not
merely stored in a greater-numbered directory.

Both complete reference histories are required even when some old files have
already been retired. An old endpoint alone cannot authorize surviving old bytes
after deletion makes old replay impossible. Loss of the old reference remains an
availability problem: independent reconstruction must precede this API, and an
untrusted peer, residual marker or directory scan cannot supply the missing trust.
The API provides neither persistent custody of these references nor an operator
interface for recovering their authority.

The selected marker must equal a complete canonical old or candidate MSR2, including
its exact manifest and block-byte metadata. Recovery fully replays the selected
complete history and compares the entire result with its accepted reference.
Missing/corrupt selected history rejects; it is never filled from staging. Both
ledger components, all retained membership history, sequences, population and
resource counters come from the selected branch as one complete state.

### Residual bytes and directory ordering

Before any file mutation, bounded layout and length checks cover all recognized
paths, and every surviving residual byte is checked against independent canonical
encodings. Old-selected candidate staging can be any subset of prescribed files,
each an exact canonical prefix, including empty files. A pending selector prefix
requires the complete exact candidate; even an empty pending file carries that
prerequisite. Prefix equality limits contents but does not prove who created them.

Once the candidate is selected, it must be complete with no pending selector.
Old residue may contain arbitrary subsets of exact **complete** old files.
Partial old files reject. If the old blocks directory exists, the full old manifest
must also exist; manifest-only, empty and absent old-generation shapes are allowed.
Unknown entries, differing bytes and unsafe static aliases reject before removal.
An invalid late old file prevents deletion of an earlier valid old file during
the initial all-residue validation. Whole residue is rechecked after synchronization
and again before the cleanup phase; each named file is compared at removal.

Recovery synchronizes the validated selected history, its directories, selector
and stable root before any unselected-copy deletion. This is necessary even when
the new marker is already visible, because the prior call may have stopped after
rename but before root synchronization. `RetainOld` durably removes any pending
selector before deleting candidate staging; it cannot leave pending metadata
alongside a candidate it has made incomplete.

Cleanup synchronizes the old generation after blocks-directory removal and before
manifest unlink. Crucially, it also performs that synchronization when the blocks
directory was **already absent on recovery entry**. Observed absence after a restart
does not prove the earlier removal was synchronized. Reestablishing this ordering
prevents a retry from letting manifest deletion persist before the earlier
blocks-directory removal. Only exact named files and empty prescribed directories
are removed; the locked root is never replaced and no recursive deletion exists.

### Explicit retries, bounded resources and retained limits

`Complete` extends authorized prefixes through checked read/append descriptors,
without truncation, verifies/synchronizes complete candidate history, publishes the
canonical marker, then retires only authorized old residue. Already-selected
completion is idempotent. `RetainOld` operates only while old remains selected;
it never rolls the selected candidate back. A caller may choose completion after
an interrupted retention, because the still-selected old endpoint and both exact
histories remain the independent authorization.

Consuming recovery retains the existing writer lock continuously, including from
a fenced writer, and checks that its last-confirmed memory checkpoint is one of
the independently retained endpoints. An inspector or unrelated handle rejects.
All successful paths finish strict clean replay and synchronization before returning
an unfenced writer; every failure returns no handle. An error after publication
was attempted, or when the candidate was already selected, can represent a
published but unfinished result. There is no automatic rollback or unbounded retry.

The distinct encoded-input ceiling remains **39,118,264 bytes**, and one shared
**313,011,680-byte** allowance charges every payload read and growth sentinel,
including reads through existing prefix descriptors. Per-chain limits remain
4096 blocks, 16 MiB MRB2 and 1,048,576 cumulative state/configuration items.
Successful old-selected completion uses at most four full replays and
**293,447,607 charged bytes**; the contract derives smaller upper bounds for the
other two successful action paths. These are conservative upper bounds on those
finite paths, not total resident memory, metadata calls, latency or write volume.

Failure hooks and synthetic partial-file fixtures cover bounded recognized retry
states; they do not reproduce all torn writes, physical power loss or firmware
faults. Cooperative Linux locks and trusted local ownership/parents remain
assumptions. No hostile same-user race guarantee or newly certified WSL/DrvFS,
native Windows or network-filesystem support follows from these checks.

F08 gains explicit recovery of this one independently authorized replacement's
recognized staging and cleanup. Interrupted initialization, missing/corrupt
selected markers, unknown residue, arbitrary corruption repair, reference-history
custody, fresh-peer reconstruction and persistent node integration remain outside
the component. **At the E2b checkpoint, next was MB-03F**; its first bounded
framing/receiver slice is recorded below. Separate MB-04 node/operator work remains.
F09–F11, human finality, biological matching/privacy and Gate A/B remain open.
No founder/marketing privilege, issuance/reward API or real biometric data is added.

## MB-03F1 bounded peer-history framing and candidate reconstruction update

This section preserves the historical F1 checkpoint. The completed finite F2
transport/adoption component and its residual risks are recorded in the next section.

The [peer-history contract](PEER_HISTORY_V2.md), separate
[`mirror_protocol::v2` codec](../crates/mirror-protocol/src/v2.rs) and
[`mirror_network::v2` receiver](../crates/mirror-network/src/v2/mod.rs) define one
finite candidate-history transfer. The new `MPN2`/version-2 framing never falls
back to the existing `MIRR`/version-1 protocol. No V1 frame, previous V2 block,
manifest, signature domain or storage profile is reinterpreted.

This section records reviewed source behavior. Focused execution passed **24 codec
tests and 36 receiver tests**, and the new synthetic receiver example passed.
Full-suite/source-publication acceptance is recorded separately in
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and
[test-matrix section 17](MEMBERSHIP_TEST_MATRIX.md#17-mb-03f1-bounded-peer-history-framing-and-candidate-reconstruction).
Independent source review found no blocker in this component's stated boundary;
that review is not an external security audit or a completed network-deployment test.

### Peer claims cannot select the network or the canonical branch

`HistoryReceiverV2::new` takes independently selected configuration and exact
genesis hashes. A Begin frame carries an explicitly untrusted `HistorySummaryV2`:
network hashes, terminal height/tip/ledger root, block count, total complete MRB2
bytes and manifest length. The network fields must equal the caller's identity,
and `NetworkManifestV2::decode_for` independently verifies the received manifest
against that identity and checks every configured seed proof.

The remaining summary fields describe what the peer promises to send. They do
not establish latestness, canonical selection, finality or the biological truth
of a membership. In particular, the implementation does not manufacture a trusted
`ChainReplayExpectationV2` from those claims. It constructs actual private state
through `ChainV2::from_genesis` and `ChainV2::append`, then compares the computed
complete endpoint and byte/count accounting with the captured summary.

Original mixed-operation blocks are decoded and executed in their original order.
Every parent and operation anchor comes from the privately accepted candidate
branch; a peer root, membership-only log or separate account snapshot supplies no
replacement ancestry. Signatures, work, timestamps, roots, account/member rules
and cumulative state work remain checked by the existing acceptance path. Both
balances and membership history, including permanent records, approvals, sequences
and population counters, come from that one branch.

Completion returns a `ReceivedCandidateV2` containing the whole accepted `ChainV2`
and transfer statistics. It makes no selection against a local current branch and
performs no file access. Later integration must apply the existing independently
bound branch comparator and store publication/recovery rules. No storage generation
is present on this wire or accepted as a fork score or freshness indicator.

### Header admission precedes future transport allocation

Each fixed header has exactly 12 bytes: magic, version, kind and payload length.
Begin and End carry the same 152-byte summary. The single manifest frame carries
complete MNM2 bytes; each Block frame contains its height followed by complete
MRB2 bytes. Structural framing bounds are separate from manifest/block validity.

The receiver's consuming `begin_frame` checks exact header shape, version, kind,
current phase, per-kind length, configured block cap, remaining declared block
bytes, feasible minimum/maximum future blocks, frame count and total framed-input
accounting. Only then does it return a `PendingHistoryFrameV2` with a checked
payload length. A future socket adapter must obtain this value **before allocating
or reading the advertised payload**. The convenience `accept_frame` instead
receives an already supplied slice and cannot establish how its caller allocated it.

`accept_payload` requires the exact admitted length and uses borrowed framing
decode before semantic acceptance. A second Begin/manifest, wrong kind, duplicate
or reordered block, inconsistent height, invalid bytes, false terminal or changed
End summary returns an error and consumes the whole attempt. No partially executed
`ChainV2` can be recovered from a failed receiver through its public API. Dropping
a pending frame on an I/O failure similarly exposes no candidate.

Even after every claimed block has been accepted, `finish` still rejects until
the explicit matching End frame has been received. End defines the finite transfer
boundary; no socket EOF is used. Further input through a completed receiver rejects.
It is still a future adapter's responsibility to close failed/unfinished sessions
and define any later session or connection framing. F1 does not inspect bytes that
the caller never supplies, open sockets or enforce wall-clock deadlines.

### Finite input and execution budgets

The summary bounds its block count to `1..=4096` and requires height plus one to
equal that count. It permits at most **2,781,636 manifest bytes** and
**16,777,216 complete MRB2 bytes**. A block is at most the configured limit, whose
current hard ceiling is **1,048,576 bytes**. Once the manifest is checked, the
receiver tightens the claimed total to the configured child cap plus the exact
168-byte empty genesis. Its genesis payload is exactly 176 bytes including height.

For `N` blocks, `M` manifest bytes and `B` complete MRB2 bytes, the total is:

```text
frames = N + 3
wire bytes = 340 + M + B + 20*N
```

The inclusive bounds are **4,099 frames** and **19,641,112 wire bytes**. Products
used to establish feasible future block sizes are checked in `u64`, including
cases where an intermediate product exceeds 32-bit `usize`. Every accepted
history still obeys the existing **1,048,576 cumulative state/configuration-item**
limit. No history pruning, state cache or cap increase is used to finish a transfer.
These are finite encoded-input and abstract execution bounds, not resident-memory,
session-duration, bandwidth, peer-count or availability guarantees.

`HistoryFramesV2` borrows one immutable accepted chain and emits one bounded frame
at a time. Its captured summary cannot become stale through mutation of that
borrowed chain; the iterator stops permanently at error or completion. It does
not assemble an unbounded whole-transcript output or select a recipient's chain.

At the F1 checkpoint F09 gained a separate checked framing/admission boundary and
bounded in-memory candidate reconstruction. The historical V1 TCP implementation
was unchanged; socket allocation/read ordering, fragmented delivery, slow-peer
deadlines, connection/request quotas and actual peer-process integration still
required **MB-03F2**. Durable adoption and preservation of recovery references were
also part of that next slice. A passing codec or receiver alone
does not close T31/T32, MB-03F, MB-03 or either launch gate.

No founder/marketing privilege, monetary decision, biometric payload schema,
real device adapter, private duplicate matching, anonymity mechanism or final
human consensus is added by this component.

## MB-03F2 finite peer sessions and completed-candidate adoption update

This section preserves the historical F2 acceptance and its memory-only recovery
boundary. The separate A1 control layer and current restart evidence follow below.

Published source:
[`851193ea6428198441e6cdf29361b1885fae157b`](https://github.com/Ahmed-Sobeih/mirror/commit/851193ea6428198441e6cdf29361b1885fae157b),
tree `cf2e4bb9d29b250a000bb4680d1571e0355d1126`, parent
`f47b4306b13c4d079859472f2fdd919debd83eda`.
The [session contract](PEER_SESSION_V2.md) and
[adoption contract](CANDIDATE_ADOPTION_V2.md) specify the implemented boundaries.
[Test-matrix section 18](MEMBERSHIP_TEST_MATRIX.md#18-mb-03f2-finite-tcp-sessions-and-durable-candidate-adoption)
maps the actual cases. Exact-source CI and later documentation-head conclusions
belong to [CURRENT_STATUS.md](../CURRENT_STATUS.md); local passing tests are not
substituted for unobserved remote results.

Local Rust 1.98.0 locked-dependency gates passed **1,010 runtime tests** (951
retained plus **59 new**), **ten compile-fail doctests**, all required release
suites, **eleven independent references** and **fifteen synthetic examples**.
Two subprocess entry points are intentionally ignored by ordinary enumeration
and exercised by parent tests. Release reruns and boundary scenarios inside a
function are not additional unique tests. Source review found no blocker within
the stated finite component scope; this is not external security certification.

### F09: actual transport admission, lifetime and partial I/O

One fixed **72-byte MPQ2** request selects independently configured C/genesis.
The provider checks its immutable accepted chain against its own local identity,
validates the request and requires requester EOF before sending a response.
The requester sends no further data, receives one unchanged F1 transcript, checks
its matching End, requires response EOF and only then consumes `finish`. A trailing
byte, second request/response, early closure or missing terminal EOF rejects.
Correct network selection is not peer authentication or encryption.

The requester first reads the fixed **12-byte** F1 header. Consuming F1 admission
checks phase, exact/configured lengths and remaining transfer feasibility before
returning an allowed payload length. The entire payload must also fit the owner's
remaining I/O allowance before allocation. All actual mixed operations execute
through unchanged ChainV2 acceptance under independently selected network pins.
Receiving and semantic errors consume the attempt, close the owned connection
and return no partial candidate. No magic scan, resume, downgrade or automatic
retry is introduced.

One non-Clone mutable owner permits **at most one live connection**, **eight total
attempts**, **157,129,488 cumulative actual read/write bytes**, and one absolute
deadline of **at most 60 seconds per attempt**. Each syscall buffer is limited to
remaining capacity before I/O. Positive byte counts, including malformed input,
partial output and rejection sentinels, remain charged after failure. The one-byte
EOF probe requires unused capacity; zero-byte EOF does not spend it.

Connect/accept, request, data transfer, semantic checks and terminal EOF share one
`Instant` deadline. The remaining time is installed before every read/write syscall;
short reads/writes, interrupted calls and timeout wakeups cannot reset it. Checks
after successful I/O and execution prevent an expired result from becoming a
returned candidate. Tests exercise fragmented input, stalls, a drip-fed header,
missing EOF and a real nonreading peer that causes partial blocked writes.

This mitigates the F09 allocation/session defects for the **separate finite V2
path** and supports T31/T32's VERIFIED_COMPONENT status there. It does not alter
or retrospectively harden the historical V1 transport/node path. A caller can
create more owners or clone its own listener; global connection/rate limits,
backlog control, fair scheduling, retry/reconnect policy and multi-peer availability
still require persistent-node acceptance. Kernel/TCP overhead and unread buffered
traffic are outside application-byte accounting. Bounded synchronous replay is
not preemptible; an overrun is rejected afterward, not guaranteed to return at the
exact deadline. No RSS or hard real-time bound is established.

### F08/F07: local selection stays separate from received claims

`AdoptionAttemptV2::prepare` accepts only an owned replacement writer, independent
network identity, independently retained complete current checkpoint/generation,
and a completed accepted candidate. It rejects fenced/read-only handles, foreign
networks and stale current pins. It verifies canonical current metadata and uses
the existing whole-history comparator. Tied, identical or shorter histories retain
the current store without filesystem calls; that no-op does not newly audit disk
for out-of-band changes by the trusted local owner.

For a preferred branch, preparation retains one exact old-history clone, moves
the entire candidate, and retains both local checkpoints before publication.
Accounts, membership records, permanent reservations, approvals, sequences,
population history and resource counters stay with their own accepted branch.
The owned store rechecks actual disk history and preference before its existing
staging, synchronization, publication, exact cleanup and final replay. A complete
peer transcript supplies no reusable write permit, local-generation score, global
freshness or finality authority. Fresh initialization is explicit and separate;
failed existing-store opening never authorizes overwrite or automatic repair.

Publication failure privately retains the same store lock and both exact histories
and returns no ordinary writer. Explicit `recover` consumes that store into the
existing recovery API under the same lock. If recovery fails, no handle survives;
the exact inputs remain available for a later explicit retry that must acquire
fresh ownership and revalidate all residue. Candidate selection cannot be rolled
back with `RetainOld`; unknown/corrupt artifacts remain preserved and rejected.
The wrapper adds no new storage mutation or recovery primitive.

Actual provider processes demonstrate fresh reconstruction/init/reopen, tie
retention and preferred whole-membership-fork adoption with equal account bytes.
The preferred fork restores its own historical population peak **2**, replacing
the losing branch's **3**, while retaining its own complete memberships and
approvals. Rejected process traffic leaves selected-store files unchanged and
strictly reopenable. Synthetic recovery-residue tests cover the wrapper's retained
input/ownership behavior; the earlier store fault suites remain required. These
tests do not simulate every power loss or certify a filesystem.

### Restart-authority work identified at the F2 checkpoint

F2's retained recovery inputs are **memory**, not a durable intent journal. A process
crash can lose them even though independently pinned storage and explicit recovery
already exist. **At this checkpoint, MB-04A1 was next**: persist independent local
selection and exact old/candidate histories outside the strict store root before
mutation, with explicit authorized restart recovery. The A1 update below implements
that separate layer. Scanning a higher generation, importing peer claims or
silently discarding unknown residue still cannot substitute for valid authority.

**MB-03F2, MB-03F and MB-03 are VERIFIED_COMPONENT only within their finite
storage/peer-session scope.** At that checkpoint, persistent node/global admission,
transaction pool, wallet/operator controls, final human consensus, privacy/matching,
device support, policy-gated money and Gate A/B remained unfinished. A2's later
finite commands are described below. No biometric archive, production
enrollment rule, founder privilege, reward formula, or changed prior byte/hash
semantics is introduced by F2.


## MB-04A1 durable local node control and restart authority update

This section preserves the accepted A1 checkpoint; the later finite command
surface and its separate evidence follow in the A2 update.

Implemented contract: [NODE_CONTROL_V2.md](NODE_CONTROL_V2.md).
Runtime: [node owner and ordering](../crates/mirror-node/src/v2/mod.rs),
[bounded filesystem/ownership helpers](../crates/mirror-node/src/v2/files.rs), and
[pure control/intent codecs](../crates/mirror-node/src/v2/records.rs).
[Test-matrix section 19](MEMBERSHIP_TEST_MATRIX.md#19-mb-04a1-durable-node-control-and-restart-authority)
maps the actual cases to T28–T30 and the retained execution requirements.

Implementation source:
[`32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0`](https://github.com/Ahmed-Sobeih/mirror/commit/32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0),
tested tree `d79080968d822a2267ce5f1b1196efcf16d2e546`, parent
`37010bc5c5827f5a8dd886af0e79d079ebfe1d4e`.
[Exact-source PR CI 45861912935](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45861912935)
passed all configured gates. Full job `140054826270` was inspected; the actual
synthetic merge checkout `62437dc1c338156fb984349b791efc7a4ad34ebb` has no file differences from that source.
Later documentation-head evidence remains separate on PR #1.

Local Rust **1.98.0** locked-dependency gates passed **1,083 runtime tests**:
1,010 retained plus **73 new** (24 codec, 31 public integration, twelve process and
six returned-error/authority-check cases). **Eleven compile-fail doctests**, all
required release suites, **twelve references** and **sixteen examples** passed.
The new node release gate passed 73 runtime tests and one doctest. Three ignored
subprocess helpers are invoked by parent tests; scenarios and release reruns are
not extra unique tests. This is component evidence and implementation review,
not an external security audit or readiness for a real-money launch.

### F07/F08: trusted local custody, independently selected network

MNC2 is a separate exact **464-byte** local record containing the complete local
checkpoint/generation and unchanged canonical MSR2 metadata. Every entry point
still receives independently selected configuration C and genesis; no record,
peer or directory scan can choose that identity. Clean opening first loads the
trusted local record, then uses existing profile-3 storage to reconstruct its
complete selected history and compare both endpoint and all canonical metadata.
An authentic older record cannot authorize a different later selected store.
This does not discover global latestness or detect an operator deliberately
restoring a mutually consistent old control/store pair.

MNI2 carries both full accepted histories and their exact local records. Decoding
checks the bounded envelope and checksums, consumes each unchanged F1 history
through actual complete execution, then repeats strict replacement preparation.
Equal-account branches with distinct memberships cannot be interchanged. Tied,
identical or shorter branches cannot create replacement intent. Local generation
advances once per preferred publication; it is never a fork score.

The control directory is trusted operator state, separate and non-nested with the
strict store root. It contains no peer-selected path. The caller chooses the intended
path pair. Checksums provide corruption detection, not authentication against a
hostile owner, disk rollback or a compromised machine. The operator and parent
namespace remain material trust assumptions.

### F08/T30: stable ownership and bounded filesystem input

Each filesystem operation validates nonempty paths, rejects parent traversal,
symlink components, managed filesystem roots, nested/same paths and aliased root
inodes, and requires existing parents. Nonblocking cooperative Linux directory-
inode ownership is acquired **control first, then store**. Ready writers and
recovery hold exclusive control; status uses shared control and refuses an active
exclusive owner. A ready owner drops storage before control. No operation replaces
a live root inode, waits/retries for a lock or derives exclusion from PID bytes.
The node exposes no raw store writer that could omit its intent ordering.

New managed directories/files use private creation modes subject to umask;
existing permissions are not silently rewritten. Payloads must be regular,
non-symlink, single-link files. Metadata and observed bytes are checked before
and after reading; unexpected entries and unsafe artifacts remain preserved.
Names are bounded before traversal, and no recursive cleanup or highest-generation
selection is used. These checks do not stop hostile same-user races, privileged
mount changes or an uncooperative client bypassing the intended path pairing.

One MNI2 intent is bounded to **39,283,208 encoded bytes**, containing at most two
**19,641,112-byte** complete histories plus fixed metadata/checksum. One operation
shares a **314,269,392-byte control payload-read allowance**, including length-plus-
one sentinels and repeated exact-byte comparisons. Length is checked and charged
before payload opening/allocation. Storage separately retains its existing read
and replay budgets. Per-history chain/block/state limits remain unchanged.

These bounds do not establish a total resident-memory ceiling or a deadline.
Replaying both histories, readback, checksum buffers and the accepted old-chain
clone use additional memory. Publication/recovery has no socket deadline or hard
CPU preemption. Global admission, availability and persistent scheduling remain
separate work, including capacity exhaustion under honest or adversarial demand.

### T28/T29: intent precedes store changes and confirmation precedes retirement

Preferred adoption first constructs both accepted histories and exact local pins,
creates `intent.pending` without overwrite, writes/synchronizes it, then reads it
back, compares exact bytes and executes both histories again. Only after publishing
`intent.v2` and synchronizing the control directory does the owner recheck the
exact published layout/confirmed/intent bytes and call existing storage replacement.
The same store lock remains held. An I/O error after store selection can therefore
leave old confirmation with candidate-selected storage, but complete intent already
exists to explain and authorize that specific transition.

After storage returns a clean candidate, A1 writes/synchronizes the candidate's
`confirmed.pending`, verifies it, rechecks existing authority, and publishes
`confirmed.v2`. It verifies/resynchronizes confirmed data and the control directory
before removing only the authorized `intent.v2`. Final layout/confirmation checks
and directory synchronization precede return of a ready owner. Returned errors
consume ownership and provide no ordinary writer; no catch-all fallback initializes
or repairs storage. Explicit later recovery reacquires locks and repeats validation.

Clean restart resynchronizes its checked confirmed record and the control directory
and parent before returning a writer. This covers visible intent absence after a
failed final directory sync. Read-only status performs none of that synchronization
and does not manufacture an initialized control directory.

### Explicit recognized recovery and preserved rejection

Read-only status reports **Missing**, **Clean** or **RecoveryRequired**, or an error.
Clean means the selected store was fully replayed against the trusted record.
RecoveryRequired reports the confirmed/old/candidate pins from fully validated local
intent; it deliberately does not infer which branch the unresolved store currently
selects or choose an action. Ordinary opening rejects valid unresolved intent.

With confirmed old and a complete intent, explicit Complete/RetainOld delegates to
the existing narrowly authorized storage recovery using both exact histories. A
selected candidate cannot roll back. Recovery first re-reads and synchronizes exact
intent, control and parent, then rechecks all expected authority before storage can
change. A visible rename from a failed previous directory sync cannot skip this step.

Confirmed candidate, or confirmed old plus an exact candidate-confirmation prefix,
permits **Complete only** and requires an already clean candidate store. Confirmation
staging means store publication returned clean; it cannot newly authorize changing
an old-selected or residual store. The permitted prefix is exactly **0 through 464
bytes** of the candidate record justified by the full intent. Recovery appends only
the missing authorized suffix without truncation, verifies/synchronizes the complete
record and directory, then retires intent. A second interruption can extend that
same exact prefix without changing its authority or permitting rollback.

Any `intent.pending` is rejected and preserved, including a fully written valid one.
Partial initialization or orphan confirmation staging is not automatically repaired.
Unknown names, corrupt/overlong records, conflicting metadata, wrong exact prefixes,
unsafe files, or candidate confirmation with confirmation staging reject unchanged.
Recognized control layout does not widen the existing storage recovery layouts or
supply missing authorization for arbitrary corruption, staging promotion or cleanup.

### Actual process evidence and remaining scope

The new process harness performs actual SIGKILL at six initialization and sixteen
adoption boundaries, then exercises status, reopening and explicit recovery in fresh
helper processes. It covers real half-written records, the store/confirmation gap,
confirmation rename, intent retirement, repeated recovery interruption and exact
confirmation-suffix continuation. Competing ownership is tested across actual
processes. Read-only and rejected operations compare exact bytes and selected inode/
link-count/mtime metadata; operating-system access time is deliberately excluded.
Private callbacks are test orchestration only; production exposes no public or
environment-variable interruption switch. Parent waits are bounded and children
are killed/reaped on failure. Same-executable lock-owning tests serialize against
unrelated forks to prevent inherited descriptors extending another test's lock.

The public finite demonstration runs four independent processes through control
initialization, a kept tie, preferred membership-fork publication and strict reopen.
It preserves equal account bytes while verifying the selected branch's own complete
memberships, approvals, population history and resource counters. No real personal
records, keys or devices are involved.

These termination cases are not physical power-loss tests or certification of
storage hardware/filesystems. Verified cooperative Linux assumptions remain;
WSL, DrvFS and other untested filesystems are not declared verified. The prior V1
executable, storage/network implementation, earlier bytes and monetary behavior
are preserved: **140 of 142 retained runtime/dependency/reference/fixture files
are byte-identical**, with only the node manifest/lockfile's local personhood
**development** dependency edge changed. No external package versions change.

**MB-04A1 is VERIFIED_COMPONENT within finite durable-control/restart scope.**
**At the A1 checkpoint, next was MB-04A2 finite V2
init/status/sync-once/explicit-recover operator commands.** The A2 update below
records that surface. Persistent serving, reconnect/rejoin, multi-peer policy,
global admission, transaction pool and wallet remain unfinished. Final human
consensus, biological uniqueness, private evidence matching, economics, devices
and Gate A/B retain their later gates. No founder privilege, biometric archive,
real enrollment or monetary policy is added.

## MB-04A2 finite operator command update

Implemented contract: [NODE_OPERATOR_V2.md](NODE_OPERATOR_V2.md). Runtime:
[dispatch and ownership](../crates/mirror-node/src/bin/mirror-node-v2/main.rs),
[bounded argument parsing](../crates/mirror-node/src/bin/mirror-node-v2/args.rs),
[bounded fallible output](../crates/mirror-node/src/bin/mirror-node-v2/output.rs),
and [static diagnostics](../crates/mirror-node/src/bin/mirror-node-v2/errors.rs).
[Test-matrix section 20](MEMBERSHIP_TEST_MATRIX.md#20-mb-04a2-finite-v2-operator-commands)
maps the unit and actual-command cases to the retained requirements.

### Source and evidence boundary

Implementation source [eea42a403fef2a1442e553df2a6823b1b25da616](https://github.com/Ahmed-Sobeih/mirror/commit/eea42a403fef2a1442e553df2a6823b1b25da616),
tree `cda30cd7e8b564a82d09a46b7067012e7112e2dd`, extends parent
`957599df1b8a424ba333d0e0c10247b2807c3a09`; published implementation/contract file
count: **9**. Runtime accounting is **1,137**, including
**54 new** cases: 23 parser, four output, three diagnostic and
**24 actual-command process tests**. The node runtime field is
**127**, with its retained one compile-fail documentation case.
The required gate set passed eleven workspace compile-fail doctests, all release
suites, twelve references and sixteen synthetic examples. No new example/reference
gate, wire format or external dependency is introduced.

Exact-source PR CI run [45882757484](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45882757484),
full job `140125894368` and synthetic merge
checkout `7dbc9921785f52c4c0136c27dc5890f9b0188e2c` were inspected: all 43 steps passed, with full logs
confirming runtime/documentation counts and an exact checkout with no file differences
from implementation source. See [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
master-plan section 28. Later documentation-head checks remain separate.

Compatibility comparison found **153 of 154 retained source/reference/fixture
files byte-identical**. The node Cargo manifest alone adds
`default-run = "mirror-node"` to preserve the V1 invocation after adding the
separate executable. Cargo.lock and all prior runtime and fixture bytes are
unchanged. Component test counts are not an external audit, project-completion
percentage, physical-failure certificate or launch authorization.

### F07/F08: explicit operator input remains the authority boundary

All four operational commands require independent C/genesis and explicit trusted
control/store paths. There is no command configuration file, environment lookup,
working-directory fallback, highest-generation scan or peer-derived network
selection. Path trust, separate non-nested roots and control-before-store
cooperative Linux ownership remain A1's unchanged contract. Checksums and command
validation do not authenticate a hostile operator or prevent mutually consistent
rollback of control and store by that owner.

`init --attach-existing` additionally requires the full height/tip/ledger-root/
generation checkpoint and strictly reopens the existing profile-3 store. It
initializes only control. Failed opening is not a request to create storage,
import genesis or overwrite control. A1 may create an empty control directory
before a later executed failure, or leave partial initialization. Those states
are preserved and reported; complete syntax rejection, by contrast, happens before
any filesystem/socket effect. No fresh-node distribution or bootstrap policy is
silently introduced.

`status` performs no application writes, synchronization, mining, peer contact or
repair. Missing refers to control, not storage existence. Clean requires full
selected replay. Valid pending intent yields confirmed/old/candidate endpoints
with `store_selection=uninspected`; it does not claim the store selects old or
candidate, or that both recovery choices are available. Filesystem access time
may independently change during reads. Actual status tests compare bytes, layout,
inodes and modification times while deliberately excluding atime.

`recover` accepts only an explicit `complete` or `retain-old`. It delegates to the
unchanged A1 authority/layout rules. Candidate selection cannot roll back; candidate
confirmation or its exact pending prefix requires Complete against an already
clean candidate store. Unsupported initialization, intent staging, prefixes and
unknown bytes remain rejected. A clean node has no pending action to recover.
No failed action automatically switches to another one.

### F09/T30–T32: finite connection and ownership bounds

The entire command is validated before effects: at most 32 arguments, 4,096 UTF-8
bytes each and 16,384 cumulative bytes; exact hashes; canonical unsigned numbers;
mandatory values; no duplicates, unknown/cross-command options, control characters
or non-UTF-8 tokens. Initialization height is at most 4,095 and generation at most
that height. These are admitted-input bounds, not operating-system argv-allocation
or total-memory guarantees.

`sync-once` strictly opens the ready A1 controller before contacting a numeric
IP/nonzero-port peer. Missing, pending, malformed, wrong-network or busy local
state therefore causes no peer connection. Both roots stay owned through receipt
and adoption. A single F2 owner is configured for one attempt, a timeout of
1–60,000 ms and actual application I/O allowance of 1–157,129,488 bytes. There is
no listener, DNS resolution, discovery, retry, reconnect or second peer.

The unchanged F1 consuming receiver must accept the entire executed candidate and
matching End, and F2 must verify terminal EOF before A1 can adopt it. Identical, tied or shorter
histories retain the old selection. Preferred publication still persists both
histories before store mutation and confirms selection before retiring intent.
Full memberships, approvals and population/resource counters follow the selected
branch; equal account bytes cannot hide a different membership history.

The timeout covers F2's connect/request/receive/semantic-completion/EOF attempt,
excluding local opening/replay and subsequent durable adoption. Synchronous
validation is not preempted; late completion is rejected. Actual partial I/O and
rejection sentinels consume the byte allowance, which excludes storage reads,
RSS, kernel buffering and TCP/IP overhead. The F1 history ceiling and A1/storage
budgets remain independent. One finite command does not establish process-wide
provider admission or continuing availability across repeated invocations.

### T29/T30: a response is not a durable transaction receipt

Success/status uses bounded `key=value` records; failures use static categories
and advice without echoing raw paths, tokens or peer payloads. Exit 2 means
rejected command input before effects. Exit 1 covers operational failures;
status alone uses 3 for missing control and 4 for validated unresolved intent.
The output limit is 4,096 bytes, but a blocked stdout consumer has no deadline.
The ready owner is dropped before output-channel writes, so output backpressure
does not extend control/store ownership.

An init/sync/recover operation may already have completed when writing or flushing
its response fails. The diagnostic then reports `operation_may_have_completed=true`
and advises `inspect-status`. Neither a failed response nor a failed mutation can
be interpreted as proof that nothing reached storage, and neither authorizes an
automatic retry or different recovery action. A1 can also return an error after
intent retirement while a confirmed clean state already exists.

The actual binary test closes stdout's receiving endpoint **before child launch**,
observes `output_io`/exit 1 after successful initialization, and checks a new
status process plus strict complete-state reopening. This establishes the finite
binary's output-failure behavior rather than relying only on a synthetic writer.

### Process evidence and the next provider boundary

The new [command integration suite](../crates/mirror-node/tests/operator_commands_v2.rs)
launches actual executables against privately owned synthetic stores and bound
loopback listeners. It checks successful keep/adopt, malformed/truncated/trailing
traffic, terminal EOF/deadline and tiny-byte exhaustion, preserving selected state
on receive failure. A separate child is held in receive with both roots owned;
competing commands fail busy without contacting their listener. Actual kill/reap
then releases ownership, and the unchanged old history strictly reopens.

Pending recovery layouts in A2 are **constructed canonical fixtures** from the
existing records and storage APIs. They check command-level Complete/RetainOld/
no-rollback behavior. They are distinct from A1's retained actual SIGKILL evidence
at six initialization and sixteen adoption boundaries and repeated recovery stops.
A2's killed receive occurs before candidate publication. Listeners are bound
before launch; readiness and held streams use explicit channels; child/peer waits
are bounded, and children are killed/reaped on failure. Same-executable process
tests serialize to prevent unrelated inherited descriptors from extending locks.
These tests do not model physical power cuts, hostile local owners, every kernel/
filesystem or global resource exhaustion.

**At the A2 checkpoint, next was MB-04B1 — bounded V2 provider lifecycle and
shutdown, design first.**
The design must require explicit paths, network identity and listen address;
strictly clean A1 ownership before bind; and a fixed accepted history under held
ownership for the entire provider lifetime. Concurrent A2 calls must observe busy
ownership. Process-wide attempt, active-connection, aggregate-byte and lifecycle
budgets must not replenish when a new F2 owner is constructed. Shutdown must be
explicit and bounded, with tested admission stop, resource release and process
exit. There is no automatic initialization/recovery/reconnect, multi-peer policy,
pool or wallet in this next component.

MB-04 remains IN_PROGRESS; Gate A/B remain unmet. The final human consensus,
biological uniqueness/private matching, accessibility/device compatibility and
policy-gated economy retain their separate requirements. No founder privilege,
biometric archive, real enrollment or monetary-policy change is introduced here.

## MB-04B1 finite provider lifecycle and draining shutdown update

Checkpoint **MIRROR-MB04B1-001**, 2026-11-27. The
[provider contract](NODE_PROVIDER_V2.md) was written and accepted before
implementation. It extends the separate V2 executable with `serve` and a narrow
[F2 accepted-stream entry](PEER_SESSION_V2.md), retaining the existing checked
request/EOF/history exchange and A1's clean ownership boundary.

**MB-04B1 is VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN.**
All **38 local command gates** and **43 source-CI steps** passed. Full source-CI
logs confirm **1,167 runtime tests and eleven compile-fail doctests**, zero failures
and three retained parent-exercised ignored helpers. The actual synthetic checkout
`cdb00671509b4f2876d51bd677ab39d861833396` has no file differences from the implementation
commit. All **17 published implementation/test/contract/dependency/workflow files**
were read back byte-for-byte. Later documentation-head CI is recorded separately
on PR #1; source acceptance does not assert that a future head has passed.

### Source and evidence boundary

Implementation source:
[ed631eae05ebeebe2b3a58a433ab8a0cc5753150](https://github.com/Ahmed-Sobeih/mirror/commit/ed631eae05ebeebe2b3a58a433ab8a0cc5753150);
tree: `dc32d623bf6a8e5e3ab03023b699a729e0c630a1`;
parent checkpoint: `f507827aabcaecda85c02d66f0de6a10db8972ac`.
The source publication changes **17 implementation/contract/workflow files**.
Source PR CI is
[45904484131](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45904484131),
full job `140197823058`. Exact-source acceptance and later documentation-head CI
remain separate records in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
master-plan section 29.

The required workspace count is **1,167 runtime tests**, including
**30 new** functions: six parser tests, nine accepted-stream
transport tests and fifteen actual-provider process tests. Node records
**148 runtime tests plus one retained compile-fail doctest**;
network records **108 runtime tests plus two retained
doctests**. The gate set retains all eleven workspace compile-fail doctests,
three parent-exercised ignored helpers, twelve references, sixteen examples,
every required release suite and all 38 local command gates. All 43 CI steps
remain; the job budget alone changes from 15 to 20 minutes. Release reruns,
parameterized scenarios and child processes are not additional unique tests.
[Matrix section 21](MEMBERSHIP_TEST_MATRIX.md#21-mb-04b1-finite-provider-lifecycle-and-draining-shutdown)
maps the complete new test set to the retained rules.

Compatibility comparison preserves **153 of 161 retained source/reference/fixture
paths byte-for-byte**. Eight deliberate existing-file changes supply the F2
accepted-stream integration, finite command plumbing, node manifest/lockfile
dependency edge and CI duration. The Linux-target direct dependency
`libc = "=0.2.189"` uses the already locked package; no package version, source or
checksum changes. A1, storage, V1, independent references and frozen
protocol/storage/hash bytes remain unchanged. These tests establish the finite
component's behavior, not an audit, physical-failure certificate or launch gate.

### F07/F08: strict local authority precedes public admission

Every provider input is deliberate: trusted control/store paths, independent
C/genesis, numeric nonzero-port listen address and all four resource limits.
The existing 32-argument, 4,096-byte-token and 16,384-byte-total parsing bounds
remain. Syntax, canonical numbers and ranges are checked before filesystem or
socket work; no environment/configuration-file/stdin fallback or DNS is added.

Startup strictly opens clean A1 control before binding. Missing, pending,
corrupt, mismatched or busy local state cannot become a provider and never
causes implicit initialization, recovery or repair. The same controller and its
complete accepted history remain held until serving ends. It does not reload
history, reopen roots or give a peer selection authority between sessions.
Concurrent A2 `status` and `sync-once` observe busy ownership. Trusted parent
directories, cooperative Linux locks and the trusted local operator remain
required; provider bounds do not protect against a hostile owner replacing root
inodes or coordinating rollback of both control and store.

### F09/T21/T30–T32: one invocation has one cumulative transport allowance

The provider owns one nonblocking listener and one F2 session owner for
**1..8 admitted attempts**, **at most one active admitted stream** and
**1..157129488 total application bytes**. It checks stop, admission time and
remaining budgets before accepting, then checks stop/time again before passing
an accepted stream to F2. A stream caught by that second check is closed unread
without becoming an admitted attempt. A signal can arrive after the final check;
an attempt already admitted at that boundary drains under its original timeout.
Kernel-established connections may wait in a backlog without being application
sessions. Kernel buffers, TCP/IP traffic, other processes and unread bytes are
outside the counted allowance.

`PeerSessionsV2::serve_connection` consumes the supplied stream and installs its
shutdown guard before local network/budget preflight. It retains the same owner's
counters and shares the checked MPQ2 request, mandatory request EOF and complete
F1 frame-writing routine. No duplicate decoder, fallback or resumption of a
failed stream is introduced. Existing `serve_once` still includes its
listener wait within its original attempt deadline.

Invalid, wrong-network, short or trailing requests receive no history.
Malformed and partial I/O retain every actual positive byte and charged attempt;
a later successful client cannot refill either allowance or clear the last
rejection category. The original absolute deadline is checked through the
exchange, including request EOF and semantic completion. Short operations,
interruptions and drip-fed input do not restart it. Buffer sizes stay within
the remaining allowance, and a one-byte EOF probe needs unspent capacity.
Synchronous work is not preempted, so these caps are neither a hard CPU/RSS bound
nor a guarantee of continuing service against repeated hostile clients.

Local history, allocation, listener/setup and internal-limit failures are fatal
operational errors. Peer refusal and deadline outcomes are accounted rejected
sessions. On normal exit, completed plus rejected sessions equals admitted
attempts. The normal stop-reason priority is shutdown, admission timeout, wire
limit, then attempt limit. Limits and graceful stop exit 0 even if earlier peers
were rejected; local/output failures exit 1 and usage failures exit 2.

### T30/T32: graceful stop drains the original attempt

Scoped Linux SIGINT/SIGTERM handling sets only a sticky atomic stop flag; it
performs no output, socket or filesystem work inside the handler. The idle loop
checks in bounded waits. A signal does not abort an admitted session or reset
its timer: that session finishes or rejects under its original F2 deadline,
then no further session is admitted. Admission-window expiry has the same
final-drain rule. Signal dispositions are restored on normal and error returns.

Both the admission window and original attempt timeout must be **1..60000 ms**.
The admission clock starts immediately before binding, after strict A1 opening.
The configured serving allowance is the admission window plus at most one final
attempt timeout, at most **120000 ms** at the ceilings. Initial opening/replay and
final output are outside that interval. Synchronous work and scheduling remain
non-preemptible; this is not a hard total process-return deadline or a
process-independent availability guarantee.

### T29/T30: release precedes final output

The provider writes no readiness or progress records while it owns the roots.
It releases the accepted stream, listener and controller before writing its
single final record, bounded to 4,096 bytes. The record identifies the fixed
selected checkpoint, stop reason, actual attempts/completions/rejections, charged
application bytes and the latest static session error. A later writer can
change selection after ownership is released; the record identifies what this
invocation served, not a perpetually current checkpoint.

A completed provider session means the local checked writes and shutdown
completed. It does not prove that a requester validated, retained or durably
adopted the history. A failed output write/flush can occur after successful
serving; it reports `operation_may_have_completed=true` and `inspect-status`.
It never restarts serving or retries a mutation. A blocked consumer can still
delay process exit after every provider resource has been released.

### Actual-process observation and its limits

The [fifteen provider process tests](../crates/mirror-node/tests/provider_commands_v2.rs)
use the actual executable and synthetic complete histories. They exercise input
and local-state refusal before bind; bind failure; repeated fixed-history
transfers; charged malformed requests and aggregate partial output; exclusive
ownership; idle and active SIGINT/SIGTERM; admission expiry; forced termination;
closed stdout; and blocked final output. Complete-state comparisons include
accounts, memberships, approvals and population state. Only synthetic fixtures
and development keys are used; no real enrollment or biometric data is introduced.

The [bounded Linux helper](../crates/mirror-node/tests/support_provider_v2/mod.rs)
serializes ownership-sensitive parent cases and kills/reaps owned children on
failure. Synthetic responses have one absolute receive deadline and a byte cap;
each read uses the remaining duration. Busy assertions require the exact wrapped
`StorageErrorV2::Busy`. A numeric nonzero port is reserved and released immediately
before launch; an actual bind collision is a visible failure with no production
retry or test retry.

Readiness requires a child-owned listening socket. To distinguish an admitted
request from a queued handshake, observation matches the exact loopback TCP
tuple, established socket inode, child-owned FD and blocked read/receive syscall
on that FD, including the one-byte EOF probe where needed. This uses bounded
direct-parent `/proc` access and native Linux syscall constants, under the access
policy tested here. A restricted `/proc` mount fails explicitly. Successful TCP
handshake or an arbitrary sleep is not substituted for that admission evidence.
These checks establish the selected Linux child states; they do not test every
kernel, architecture, filesystem or scheduler.

The full-pipe case observes the child blocked in its real final stdout write
while its listener is gone and a newly launched status process succeeds.
Ownership release is therefore observed before output can finish. The separate
closed-stdout case checks the possible-completion diagnostic and no restart.
SIGKILL tests force release of a provider blocked in network receipt and verify
unchanged selected state. They establish neither graceful drain nor physical
power-loss safety. A1's retained process stops at durable publication boundaries
remain separate evidence; B1 does not add a storage recovery policy.

### Historical B1 continuation and remaining threats

The following records B1's next design scope. The B2 update below records
the resulting composition contract and its exact-source acceptance evidence.

**MB-04B2 begins with an explicit finite rejoin and serve coordination design.**
Specify one receive from a deliberate numeric peer, complete keep/adopt through
the existing authority, then serving of the resulting fixed history. Retain the
returned controller and one cumulative F2 attempt/byte owner across both phases;
do not silently reopen storage or replenish budgets.

Define stop behavior while receiving, before adoption, during synchronous
durable adoption and during serving. Preserve original deadlines and A1's
intent/confirmation ordering. A later bind, serving or output failure must
report that adoption may already have completed and direct explicit
inspection/recovery. Verification must use independent seed-provider, rejoining
node and requester processes/directories, then clean restart/rejoin, complete
state agreement, pending-state refusal and continued no-rollback/ownership checks.
That is the next design scope, not an implemented B1 capability.

No automatic reconnect/recovery, discovery, multi-peer scheduler, pool or wallet
is added. Full MB-04 remains IN_PROGRESS and Gate A/B remain UNMET. Existing
monetary, enrollment, combined ECG+pinky and AntKiss, accessibility and private
device/matching policies remain unchanged. Neither a signature nor synthetic
history agreement proves biological uniqueness, privacy or human finality.

## MB-04B2 finite rejoin and serve coordination update

Checkpoint **MIRROR-MB04B2-001**, 2026-11-27. The
[rejoin contract](NODE_REJOIN_V2.md) defines one explicit source receive, complete
keep/adopt through A1 and subsequent fixed-history service under retained
ownership. It was specified before implementation. This is a composition of
existing authority and transport boundaries, not a new peer trust, fork-choice,
storage-recovery or consensus rule.

**MB-04B2 is VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION.** All **38 local command gates** and
**43 exact-source CI steps** passed. Published source readback and the
actual synthetic checkout were verified against the implementation. Later
documentation-head CI is recorded separately on PR #1.

### Exact source and acceptance evidence

| Evidence | Recorded value |
|---|---|
| Acceptance state | VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION |
| Implementation source | `bbac1622be18c97ab640f6834a158e5378f1d172` |
| Source tree | `d724df618f6b45429abebaa3c24bb52ad742799b` |
| Parent checkpoint | `966286bec62ce422db6bc091c472d74659935344` |
| Source CI run / full job | [45927607784](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45927607784) / `140272547972` |
| Actual source-CI checkout | `e00fbb793f5eb417cc0f0cda1ed52443c5cef34d` |
| Published source/test/contract files read back | 13 |
| Workspace runtime / node runtime tests | 1,201 / 182 |
| New runtime / actual-process test functions | 34 / 27 |
| Retained paths unchanged / intentionally changed | 160 / 6 of 166 |

The 34 added tests comprise six parser, one diagnostic and 27 actual-process
functions. All 38 local gates and 43 exact-source CI steps passed, including
eleven compile-fail doctests, twelve independent references, sixteen synthetic
examples and every required release/workspace command. Published readback and
preservation checks passed. Later documentation heads and their CI remain
separate from the tested implementation source.

### F07/F08: local authority spans receive, adoption and serving

All ten command options are deliberate, including independent C/genesis,
control/store paths and numeric nonzero peer/listen addresses. Complete existing
argument, scalar and resource validation precedes signal installation, directory
ownership and network effects. No discovery, hidden configuration, DNS or failed-
open initialization is added. Strict A1 opening rejects pending, corrupt,
mismatched, missing or busy state before peer contact or bind; it never invokes
recovery. A shutdown request cannot turn failed opening into a ready controller.

The one request uses unchanged F2 header admission, complete accepted-history
reexecution, matching End and terminal EOF. A transport or candidate failure is
fatal before adoption, even if stop was requested; the command does not fall back
to serving old history. An incomplete response never enters comparison or disk
publication. The transport remains unauthenticated and unencrypted. Independent
network pins do not authenticate a peer or prove that its branch is globally latest.

After validated receipt, the consuming A1 keep/adopt path preserves complete
branch selection and intent-before-store/confirmation-before-retirement ordering.
The command retains `outcome.into_control()` directly for the fixed-history
service phase. It does not reopen either root, import peer generation, expose a
raw writer, graft losing-branch membership or refresh history during service.
Competing status/sync/recovery operations remain busy while exclusive ownership
is held. Cooperative Linux locking and trusted local parents/owners are unchanged.

### F09/T21/T30–T32: one cumulative allowance across phases

One `PeerSessionsV2` owner supplies the receive and all service attempts. Its
limits remain 1..8 attempts and 1..157129488 actual application bytes. Failed and
partial I/O retains its charges. The receive consumes the first attempt, leaving
at most seven service attempts. No new phase or helper constructs a replacement
owner or refunds allowance. There is at most one connected application stream;
receive and serving never overlap.

Check shutdown and remaining allowance before binding. Successful receive and
keep/adopt with `max-attempts=1` can stop normally without trying an occupied
listen address. EOF probing still needs unused capacity even when zero bytes
are returned. B1's pre-accept and post-accept stop/time checks, one admitted
stream and original-deadline drain remain in the shared private serving loop.
TCP handshakes/backlog, unread bytes, kernel overhead and other processes are not
application admissions or charged payload bytes.

The service admission window begins after keep/adopt, immediately before a
permitted bind. Network-phase allowances total one receive timeout, that window
and at most one final service timeout: at most 180000 ms under existing ceilings.
No active attempt's absolute deadline is renewed by fragments, signals or phase
changes. Initial opening, durable adoption, synchronous work, scheduling and
final output prevent a hard process-return or CPU/RSS guarantee. These finite
bounds also do not guarantee useful service against hostile or unavailable peers.

### T29/T30/T32: shutdown respects adoption admission

Install one scoped signal guard after parsing and retain its sticky flag across
opening, receipt, adoption and service. The handler performs no I/O, allocation,
recovery or process exit. Successful opening followed by a pre-request stop
returns unchanged verified selection with zero attempts and no listener. During
receive, allow unchanged F2 to finish or fail under its original deadline.

Check stop after validated receipt immediately before adoption admission. Stop
there discards the candidate, retains charged receive counters and never binds.
If the check admitted adoption, a later signal does not cancel any synchronous
intent/publication/confirmation step. After A1 success, stop prevents binding and
the normal result identifies the actual selected branch. During service, preserve
B1's final-drain behavior. Never clear stop, repeat the peer or infer rollback
from a missing final command response.

Any adoption error or later fatal bind/listener/service error carries conservative
possible-completion context and directs `inspect-status`. An adoption error
returns no ordinary controller and makes no selected-state assertion. A failure
can follow complete confirmation/intent retirement and does not necessarily mean
pending intent. A selected candidate cannot be rolled back by RetainOld. Explicit
status/open/recovery resolves only supported A1 states; no new repair policy or
automatic recovery is introduced.

### T29/T30: final records identify phase results without retaining ownership

Streams, listener and controller are released before formatting or writing the
single bounded normal record. Its 26 fields include previous/selected checkpoints,
receive status, choice and separate receive/service counters. Normal results obey
receive attempts plus serve attempts equals attempts, completed plus rejected
equals serve attempts, and receive bytes plus serve bytes equals wire bytes.
An unsuccessful receive produces an operational diagnostic, not a normal result.
Standalone B1 keeps its original fields and counters.

Final-output loss retains conservative possible-completion reporting for every
normal B2 outcome, including pre-adoption shutdown. A blocked stdout consumer
may delay exit after socket and owner release. No readiness/progress output holds
those resources. Diagnostics include only bounded static categories, not raw
paths, peer addresses, bytes or nested errors. Normal stops exit 0, operational
or output failures 1, usage failures 2; only status retains 3/4. Service completion
reports local transfer completion, not requester validation, durable adoption,
human finality or a perpetually current local checkpoint.

### Observed process boundaries and their limits

The new actual-process acceptance is in
[rejoin_commands_v2.rs](../crates/mirror-node/tests/rejoin_commands_v2.rs), with
bounded support in
[support_rejoin_v2](../crates/mirror-node/tests/support_rejoin_v2/mod.rs).
All 27 added process tests passed in the accepted source's debug/release suites,
covering independent seed/rejoin/requester processes and directories; complete
state agreement and clean restart/rejoin; pre-network refusal; retained ownership;
cumulative and EOF limits; keep/adopt choices; receive and serving shutdown;
later bind/output errors; output-release ordering; and explicit no-rollback.
Only synthetic development keys and histories are used. All 13 published source/
test/contract files were read back; 160 of 166 retained paths remain byte-identical.
Six existing private command modules change; lower layers, public APIs, dependencies,
workflow commands, references and all frozen encodings remain unchanged.

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

Before-adoption shutdown evidence must similarly identify the child's actual
receive socket/FD and blocked syscall, then release a valid terminal EOF or the
chosen failing response. Keep deliberate pending fixtures separate from actual
interrupted commands and from A1's existing publication-boundary failure tests.
Use bounded parent observations and kill/reap cleanup, without production hooks,
permission changes or hidden retries. SIGKILL and fresh replay demonstrate only
their tested process boundary; they do not certify arbitrary corruption repair,
physical durability or untested OS/filesystem behavior.

### Historical B2 continuation and remaining threats — MB-04B3

At the B2 checkpoint, next was **MB-04B3 — bounded explicit-peer failover, design first**.
Specify one finite ordered list of explicit numeric peers, at most one attempt
per entry, ending acquisition at the first complete validated candidate. Apply
exactly one existing keep/adopt and B2 fixed-history serving; a keep-current
choice also ends acquisition and does not justify a best-peer/latestness claim.

Before implementation, define the failure-advance allowlist, duplicate-address
treatment, list/argv caps, cumulative attempts/actual bytes and total network-phase
allowance, phase-specific stop/error handling and bounded per-peer diagnostics.
Retain one controller, signal guard and F2 owner. Local/adoption failures remain
fatal; exhausted peers without a validated candidate fail without serving-old
fallback. Actual-process acceptance must exercise failed-first/successful-second
acquisition, no later contact after validation, exhaustion and shutdown.

B3 is planned design only. Repeated-peer retry/backoff, discovery, concurrent
sessions, multiple durable adoptions, an indefinite daemon, pool and wallet remain
outside it. Full MB-04 remains IN_PROGRESS; Gate A/B and the human-consensus,
monetary, enrollment, private biometric matching and actual-device gates remain
unmet. Existing combined ECG/designated-pinky/supervised Ant Kiss and accessibility
preferences are neither altered nor validated by network composition tests.

## MB-04B3 bounded explicit-peer failover update

Checkpoint **MIRROR-MB04B3-001**, 2026-11-27. The design-first
[failover contract](NODE_FAILOVER_V2.md) introduces `rejoin-failover-serve`:
one bounded ordered acquisition prefix followed by the shared B2 adoption and
provider path. It changes neither peer trust nor canonical selection authority.

**VERIFIED_COMPONENT — BOUNDED EXPLICIT-PEER FAILOVER.**

| Evidence | Verified source and execution |
|---|---|
| Implementation source / tree | `3311151ab627b3512abed47be20adea2f0e6d66a` / `fc238ae2df95b7b2b3df2457d06f739048812c7e` |
| Parent checkpoint | `9ca7d99681877d014ae64aab31cb5eb22a957c35` |
| Source CI run / full job | [45933868084](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45933868084) / `140292436668` |
| Actual source-CI checkout | `0cada2958afe985bb5fcd3b683c57a40b01042ce` |
| Implementation/test/contract publication inventory | 12 files |
| Passing workspace / node runtime tests | 1,242 / 223 |
| New runtime / actual-process functions | 41 / 22 |
| Retained paths byte-identical / intentionally changed | 164 / 5 of 169 |

The focused combined run passed **62 binary tests and all 22 new process tests**
on its first execution. The 41 added functions comprise **six parser, eight
typed-policy, three output, one error-context, one diagnostic-fallback and 22
process tests**. Parameterized scenarios, child executions and release reruns are
not extra unique tests. Full acceptance passed **all 38 local gates, 1,242 runtime
tests, eleven compile-fail doctests, all seven release commands, twelve independent
references and sixteen examples**, with zero failures and three retained ignored
helpers exercised by parent tests. Node release passed 223 runtime tests and one
doctest. Exact-source CI passed all **43 steps**; its full logs confirm these counts
and every added test in both profiles. The actual synthetic checkout has no file
differences from the source, and all 12 published source files were read back
byte-for-byte. Later documentation-head CI is separate evidence recorded on PR #1.

### F07/F08/T28: operator authority and first-valid ordering

Effect-free parsing admits exactly one 1..4-entry numeric list, fixed slots and
the existing scalar/argv caps. Alias rejection includes equivalent IPv6 spellings
and IPv4-mapped endpoints; it does not identify physical machines. No peer is
discovered or authenticated. Trusted operator paths and independent C/genesis
remain authority. Pending, missing, corrupt, wrong-network or busy local state
refuses before any configured peer contact or bind; no recovery is inferred.

One A1 controller and one F2 owner remain held across acquisition and service.
The first complete candidate passing actual F2 validation and terminal EOF ends
acquisition, even when keep-current wins. A valid stale first peer can prevent
contact with a later better history: first-valid is not convergence, best-peer
selection or global latestness. The whole selected branch retains accounts,
membership, approvals, sequences and population; no cross-branch state is merged.

### F09/T21/T31/T32: closed refusal policy and finite resource consumption

Only the typed allowlist in the contract advances. Never treat the coarse
`invalid_peer_history` diagnostic as permission: nested allocation, arithmetic,
accounting, local-authority and resource failures can share it. Unlisted I/O,
manifest configuration, block-operation and ledger errors remain fatal. The eight
classifier tests construct both allowed and excluded typed errors; this is unit
classification evidence, not induced OOM or live coverage of every OS error kind.
Actual processes cover representative transport/malformed responses, nested
wrong-network refusal and excluded manifest-configuration failure.

Failed requests retain actual attempt/byte charges, discard their socket/candidate
and never resume framing. All peers failing produces operational failure without
serving the old branch. Partial responses and service rejections share the same
1..8 attempts and 1..157129488-byte owner. A payload larger than positive remaining
capacity is fatal; terminal EOF still needs one unused byte even when it charges
zero. The invalid-header process observes precisely 72+12=84 consumed bytes.

At most min(P,M) requests occur, with each original 1..60000-ms deadline intact.
Configured network-phase allowances are at most `min(P,M)*T+A+T` (360000 ms at
the ceilings), also bounded by `M*T+A`. These are qualified allowances, not hard
CPU, RSS or whole-process latency limits. Service begins after keep/adopt and
drains at most its admitted attempt under B1's unchanged deadline. Successful
last-allowed receipt can adopt and stop before binding an occupied address.

### T29/T30/T32: shutdown, uncertainty and reporting

One sticky signal guard spans every phase. An active request drains to failure
or full validation under its original deadline. Failed receipt plus a signal
preserves the error and prevents advancement; stop between failures likewise
retains failure. A validated candidate under a prior stop is discarded before
adoption admission, while its ordinal and earlier failure evidence remain.
Admitted adoption is synchronous and retains A1's ordering and no-rollback rules.

Every adoption or later fatal error keeps its category plus possible completion
and inspection-first guidance; it makes no unresolved selected-state assertion.
Optional acquisition evidence uses fixed static keys, separate receive counters,
and no endpoints/paths/nested text. Malformed optional internal evidence falls
back to the original safe diagnostic without losing completion uncertainty.
Normal 32..35-field records and errors fit the existing 4096-byte cap. Release
sockets, listener, both roots and signal guard before output; output loss cannot
justify repetition. Service and acquisition errors remain separate after success.

### Actual observations, retained evidence and limits

The new [22-process suite](../crates/mirror-node/tests/failover_commands_v2.rs)
uses unchanged B2 observations through [bounded support](../crates/mirror-node/tests/support_failover_v2/mod.rs).
Refused/malformed first -> real seed second -> B3 -> third requester runs with
independent roots and an explicit restart. Fresh strict reopening compares whole
state, including equal-account/different-membership forks; the seed retains local
generation 0 while adopters use 1. Held unused listeners establish no later
contact, and retained prior listeners detect unintended repeat contact.

Signal tests correlate the actual child socket endpoints/inode/FD and blocked
syscall before SIGINT/SIGTERM, including later successful receipt discarded at EOF.
Directory descriptor identities and competing busy commands are checked across
failed receipt, validated receipt and service. A full output pipe observes a
blocked write while the listener is gone and status/strict reopening succeeds.
SIGKILL during second partial receipt leaves old storage unchanged; post-adoption
bind/output failures leave the selected candidate and truthful uncertainty.

The retained B2 live-adoption SIGTERM and idle/active service-drain tests exercise
the newly shared post-receive path. They are not new B3 test functions or a claim
that each phase was newly forced through the B3 command. Pre-first-request stop
and the inter-request stop check retain source-level evidence. Finite observations
cannot sample every microscopic ownership interval. Pending fixtures differ from
actual interrupted processes; no observed signal establishes an exact fsync or
physical power-loss boundary. Process evidence is cooperative Linux IPv4, with
trusted local owners/parents; IPv6 parsing is separate coverage.

The 164 unchanged retained paths and 199 files frozen throughout source acceptance preserve earlier
tests/helpers, public A1/F2 APIs, lower layers, V1, dependencies, workflow and references.
Fixtures and wire/storage/hash formats remain; only five retained private command modules change.
No production hooks, privilege changes, real keys, biometric archives or new recovery/admission policies are introduced.

The later checkpoint updates nine status/handoff records and clarifies only the
contract's compatibility wording. Existing operational records are unchanged;
help lists the new command and options. No runtime/test file changes, and the
other 198 files frozen during source acceptance remain byte-identical.

### Historical B3 continuation and remaining threats — MB-04B4

At the B3 checkpoint, next was **MB-04B4 — bounded persistent-run lifecycle, design first**.

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

## MB-04B4 bounded persistent-run lifecycle update

Checkpoint **MIRROR-MB04B4-001**, 2026-11-27. The design-first
[run contract](NODE_RUN_V2.md) adds separate `run` with explicitly finite repeated
acquisition, keep/adopt and service. Repetition changes neither peer trust nor the
independent network, canonical branch or local durable authority boundaries.

**VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE.**

| Evidence | Verified source and execution |
|---|---|
| Implementation source / tree | `e768268fae2e4c5c9980cc74dc7a9973da56849c` / `daff130cc7652ea0946106f46a600c54671ecf18` |
| Parent checkpoint | `e421494a5a2ba28992ad581af4067618c38c0dae` |
| Source CI run / full job | [45941543950](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45941543950) / `140316760168` |
| Actual source-CI checkout | `e51ff7da8385981dd9d94e4329ed35861f2dc31f` |
| Implementation/test/contract publication inventory | 16 files |
| Passing workspace / node runtime tests | 1,281 / 262 |
| New runtime / actual-process functions | 39 / 19 |
| Retained paths byte-identical / intentionally changed | 164 / 8 of 172 |
| Frozen noncheckpoint files | 203 |

The first combined focused execution passed **82 binary tests and all 19 new
process tests**, with zero failures (86.184 seconds); no execution fixes were needed.

The 39 added functions comprise **six parser, five output, one Failure-context,
one diagnostic-fallback, seven runner and nineteen actual-process tests**.
Parameterized scenarios, child executions and release reruns are not extra unique
tests. Full acceptance passed **all 38 local gates, 1,281 runtime tests, eleven
compile-fail doctests, all seven release commands, twelve independent references
and sixteen examples**, with zero failures and three retained ignored helpers
exercised by parent tests. Node release passed 262 runtime tests and one doctest.
Exact-source CI passed all **43 steps**; full logs confirm these counts and every
added function in workspace and node release. The actual synthetic checkout has
no file differences from the source, and all 16 published source files were read
back byte-for-byte. Later checkpoint-head CI is separate evidence on PR #1.

### F07/F08/T28: finite operator authority spans every round

All twelve options are explicit. Run admits 1..4 rounds, one ordered 1..4-entry
numeric peer list and a 0..60000-ms inter-round wait under unchanged parser limits.
A peer may be retried only by entering an authorized later round. Each round
restarts at entry one; first fully validated history and terminal EOF end that
acquisition even when keep-current wins. A stale first source may repeatedly
prevent later contact. This is neither peer authentication nor convergence.

Strict clean A1 opening precedes any network effect. One controller, F2 session
owner and signal guard persist across all rounds and waits. Private retained
helpers return the same controller; no reopen, budget reset or overlapping listener
and acquisition loop is introduced. Waiting holds both roots but no listener.
Every selected state is one complete accepted history; accounts, membership,
approvals, sequences and population are not merged across forks. Missing, busy,
wrong-network, corrupt or pending state refuses without implicit recovery.

### F09/T21/T31/T32: repeated work still has one cumulative budget

B3's closed typed advancement policy remains unchanged. All-peer failure and
unlisted nested, authority, allocation, accounting and resource failures terminate
the invocation, including after successful earlier rounds. Failed/partial requests
and rejected service keep their actual attempt/byte charges. The invalid-header
fixture consumes exactly 72 request + 12 header = 84 bytes. Later terminal EOF
still requires one unused byte, even when the successful probe consumes zero.

Total transport remains 1..8 attempts and 1..157129488 bytes. Per-attempt timeout
and each service admission are 1..60000 ms. With M attempts, R rounds, timeout T,
admission A and wait W, configured network/admission/wait time is bounded by
`M*T+R*A+(R-1)*W`, at most 900000 ms. Final draining consumes an already counted
attempt. Synchronous validation, opening, adoption, scheduler delay and output
remain outside a hard whole-process latency/CPU/RSS guarantee.

After normal service return, priority is shutdown, wire limit, attempt limit,
then round limit. Only `admission-timeout` with no global stop advances. Stops
and errors skip the inter-round wait. A configured round count larger than the
attempt cap is valid authorization, not a promise that every round will execute.

### T29/T30/T32: admission and completion uncertainty remain distinct

Active receive/service retain their original deadlines under shutdown. A failed
receive plus a signal remains failure, even after earlier completion. A later
validated candidate stopped before adoption is discarded; the last completed
selection remains. Once any keep/adopt is admitted, all later fatal failures keep
possible completion and inspection-first guidance, including later receipt/wait.
Keep-current admission counts; a new acquisition cannot reset that annotation.

Normal output identifies initial, completed-round and final selected checkpoints
in exactly `30+16*C+7*I` fields. Checked phase totals and checkpoint continuity
precede formatting. Latest non-`none` receive/service errors remain across rounds.
Error evidence has fifteen invocation fields and eight optional current-acquisition
fields. Its `adoption-service` phase deliberately does not guess an unavailable
inner subphase. `last_completed_service_error` excludes an unreturned fatal phase;
no completed/rejected totals or unresolved selected pins are invented for it.
Malformed optional context preserves the original safe diagnostic and uncertainty.
Static-label CSVs contain no addresses/paths/nested text. Conservative 3918/3529
byte normal envelopes and help fit the existing 4096-byte buffer. All sockets,
listener, roots and signal guard drop before output. A stalled or lost output
record cannot authorize repeating work, automatic recovery or rollback.

### Actual-process evidence and limits

The new [nineteen-process suite](../crates/mirror-node/tests/run_commands_v2.rs)
uses [bounded run assertions](../crates/mirror-node/tests/support_run_v2/mod.rs).
A real seed, repeated run and requester use independent roots and an explicit
restart cycle. Other cases adopt two preferred histories and compare complete
freshly reopened state, exhaust the same attempt/byte owner across rounds, retain
prior errors, and establish no later contact with held unused listeners.
Root descriptor identities and busy competing commands span receipt, service and
wait. Finite observation is complemented by inspection of the retained-owner path.

The only retained helper change appends a wait observer that requires the child
listener to be gone and its actual nanosleep syscall to be visible. SIGINT/SIGTERM
then stop the wait without another round. Later receive tests observe the actual
socket/FD/read and mandatory-EOF phase; successful receipt is discarded or its
original deadline produces failure. Later service drains its admitted request
while a queued request gets no response. Live second adoption is established by
stopped child state, both roots and the exact canonical intent before queued
SIGTERM is released; it completes without binding an occupied service address.

Closed/full output pipes exercise uncertainty and owner release after two
adoptions. SIGKILL during later partial receipt leaves the prior completed
checkpoint unchanged. These are different observations from pending fixtures or
physical power loss. No test certifies a precise fsync boundary, every instruction
interval, hostile same-user races, untested filesystems or IPv6 TCP; numeric IPv6
admission is parser coverage. Constructed fatal-error tests do not induce OOM.
Seven private retained CLI modules and a 720-byte append-only helper are controlled
changes. Earlier test bodies, public APIs, lower layers, V1, dependencies, workflow,
references, fixtures, encodings and all monetary/enrollment/device policies remain.

### Historical B4 continuation and remaining threats

**Next: MB-04C1 — bounded local pending-operation pool. READY — DESIGN FIRST.**

Specify a local in-memory component for bounded signed `SignedOperationV2` inputs,
covering transfers and membership operations without changing their accepted bytes
or execution rules. Separate retained input, contextual validity, executable ordered
batches and canonical inclusion. Admission is neither spendability nor an inclusion
promise, and a pool must never acquire chain/store authority by holding a staged result.

Before implementation define independently named local entry, encoded-byte,
batch-size and cumulative validation/revalidation work caps. Check them before
input-sized copying or expensive validation. Canonical membership `max_pending`
is not a transaction-pool limit. Specify exact signed-operation-ID deduplication,
stable bounded ordering, full-pool behavior, expiry/anchor handling, conflicts and
removal. Transfer account and membership record sequences are separate domains;
do not collapse them into one actor/sequence namespace or invent fee replacement.

Bind evaluation to an actual immutable accepted `ChainV2` and its exact selected
parent checkpoint. Existing contextual/signature validation does not execute
balances or consume sequences; use the complete ordered mixed-operation ledger
stager to assess executability atomically. Define explicit reconciliation after
append or replacement, stale cached results, included/expired inputs, and handling
of gaps or future inputs. A narrow next-block-only first scope is possible, but
that choice and its failure behavior must be specified before code.

The pool must not bind staged ledger state, append or mine a block, sign, mutate
A1/store, persist, gossip, add submission CLI/API ingress or assume wallet custody.
Those remain separate increments. Verify duplicates, ordering, overspend/sequence
conflicts, mixed-operation late failure, caps and selected-parent changes without
mutating the accepted chain. Preserve zero-fee/no-mint development semantics;
production monetary, enrollment, device, privacy and accessibility decisions remain
at their existing unresolved gates. C1 is planned only. Broader scheduling, wallet,
operator APIs, full MB-04 and Gate A/B remain unfinished.

## MB-04C1 bounded local pending-operation pool update

**MB-04C1 is VERIFIED_COMPONENT — BOUNDED LOCAL PENDING-OPERATION POOL.**
C1 adds an in-memory library component for context-valid signed retention,
one atomic ordered batch evaluation and explicit selected-parent reconciliation.
B4's finite run rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown,
A2 operator commands, A1 durable control/recovery and MB-03's storage/peer components
retain their accepted boundaries. MB-04 remains IN_PROGRESS: finite operator pool
intake, persistence/gossip integration, wallet, wider APIs and broader scheduling
remain unfinished. Complete Ant Kiss remains MB-05 and final human consensus
MB-06. Gate A/B remain UNMET.

### Exact source and evidence boundary

Implementation source: **94207e713dd89f4d33586263c510591be0e78201**.
Tested source tree: **9c40a4852c8ab935cd7318d33d81d0720cd0dd15**.
Parent checkpoint: **35bbd0817843c35a42ff8c09b779391df331d53c**.
[Source PR CI 46086769527](https://github.com/Ahmed-Sobeih/mirror/actions/runs/46086769527)
**attempt 2** passed all **43 steps**. Full job `140804733638` logs confirm **1,333 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`17ece65483bd57dbfb3c6e11c64895ddf0afbbe2` has no file differences from the implementation source.
All **seven published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **208 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Selected-parent and publication authority

The new `mirror_node::v2::pool` component borrows an actual accepted `ChainV2`.
Its private binding contains configuration/genesis, height, tip and accepted ledger
root. Five deliberate local limits cap retained entries (at most 256), encoded
operation bytes (74,496), one batch's operations (64), complete framed block bytes
(19,048), and cumulative abstract work (67,108,864). Individual input is capped at
291 bytes before decoding/ID/signature work. Canonical membership `max_pending`
remains unrelated, and canonical operation/body limits apply independently.

Admission decodes the exact existing signed operation, deduplicates its complete
signed ID against retained entries and validates it for the selected next block.
Stable FIFO retention permits context-valid overspends, sequence gaps, competing
sequences and unknown membership targets. An exact duplicate preserves its position
even at full capacity; a distinct full-pool input never evicts another entry.
Admission does not scan history, so an already-included but still context-valid
input may be retained until evaluation/reconciliation. Future windows and invalid
expiry/anchor/signature/configuration reject. No fee replacement is invented.

Evaluation chooses exactly one requested nonempty FIFO prefix. Local count/framed-
byte and canonical complete-body preflight precede copying or execution. One full
`LedgerV2::stage_operations` call preserves original mixed order and atomic accounts/
membership semantics, including separate account and member sequence domains.
The staged ledger is discarded even on success; only immutable ordered input and
parent/revision observations are returned. Failures retain original indexed ledger
errors and leave all pool entries and accepted chain state unchanged.

Explicit reconciliation handles the same parent, append or accepted replacement
within the same configuration/genesis. It reserves complete work before history
ID hashing, contextual checks or temporary allocation, scans accepted history
once. Each input's removal reason prefers exact inclusion over contextual
invalidity; removal reports and survivors keep FIFO order. Context-valid stateful
blockers remain. Parent,
entries, exact bytes and revision publish together; budget/allocation/arithmetic
failure preserves the prior binding and semantic entries. Removed inputs are not
automatically resurrected after a replacement.

Work is cumulative for one non-Clone owner. Identification charges failed/duplicate
attempts, each evaluation charges the current parent-state size again, and every
reconciliation charges accepted history plus bounded retained-ID comparisons and
revalidation. No failure, clear, removal or parent change refunds work. The meter
is an explicit abstract budget, not CPU/RSS/wall-clock or a global rate guarantee.
Every successful reconciliation and actual entry mutation invalidates earlier
batch observations. Currentness checks the real chain, full parent, revision and
exact prefix; a different pool's different entries cannot match on revision alone.

The component never returns/binds staged state, appends/mines/prepares blocks,
signs, mutates A1/store, persists, gossips, exposes submission CLI/API ingress or
assumes wallet custody. Existing commands do not create a pool automatically.
Retained input, contextual validity, evaluated execution and canonical inclusion
remain separate. Actual block acceptance still checks complete blocks, timestamps,
work, roots and cumulative chain budgets. Zero-fee/no-mint development semantics
and all monetary, enrollment, privacy, device and accessibility decisions remain.

Admission checks context and possession of the signing key, not available funds,
unconsumed state, honest enrollment evidence or historical noninclusion. Holding a
batch cannot bind state or publish a block. A real accepted chain supplies ancestry;
equal height, balances or an externally supplied generic parent view cannot replace
the complete selected checkpoint. Same-network accepted replacement is explicit;
the pool never chooses a preferred branch or treats a removal report as a command.

### Resource bounds and denial-of-service limits

Local caps precede the corresponding copying/validation. Exact-ID checks at full
capacity remain charged; rejected signatures and late failed evaluations consume
their complete reserved work. Full ordered staging occurs once per evaluation;
parent-state charge is repeated for each attempt. Reconciliation includes actual
accepted-history bytes and bounded ID comparisons, not merely the pool size.
These are finite-owner abstract bounds. They do not rate-limit creation of new
owners, untrusted submission channels or process scheduling, and establish no
network-facing DoS resistance or hard allocator/CPU/RSS/wall-clock guarantee.

### Observed tests and residual integration work

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed, with all **217 tested source/checkpoint inputs
unchanged during execution**:

- **1,333 workspace/all-target runtime tests**, zero failures: 1,281 retained plus
  **52 new** — 31 admission/evaluation and 21 reconciliation/work integration tests.
  Parameterized cases, subprocesses and release reruns are not additional tests.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers remain exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **314+1**. All **twelve references** and **sixteen examples** passed.
- Both new focused suites passed on their first execution: **31 and 21** tests,
  with no execution-driven production/test corrections. Scoped formatting,
  whitespace and allocation/runtime-data checks passed.

The first full local workspace attempt and one unchanged targeted retry failed in
the retained B4 test's inter-round observer: it waited for requester durability
before looking for a 200 ms phase. Both captured runner reports completed two
legal rounds successfully. The narrowly repaired test starts the requester
concurrently, observes the real listener-free wait, verifies an owned SIGSTOP,
retains the owner/busy checks, awaits requester completion and resumes before the
second request. Original timing values, two-request outcomes and restart/state
assertions remain. The repaired targeted case and all required gates passed on
the final source. Failed attempt evidence is retained; production lifecycle and
C1 implementation/test semantics were not changed to suppress the failure.

The final source's first hosted CI attempt passed every test/release gate, then
was cancelled during the examples just after the configured 20-minute job limit.
Its log records only a generic cancellation; timing supports a budget-boundary
explanation but does not establish a more specific cause. That incomplete attempt
is preserved and not accepted. The unchanged source/workflow was rerun; the
successful complete attempt is identified in the source record above. The
20-minute workflow limit and all verification commands remain unchanged.

The new suites construct actual mined, manifest-pinned accepted chains with public
synthetic keys. They cover all four operation kinds, exact signed bytes/IDs,
malformed/signature/context refusals, duplicates at full capacity, actual 256-entry
and 64-operation limits, independent local/canonical quotas, framed batch bytes,
FIFO order/removal, retained sequence gaps/conflicts, receive-before-spend,
aggregate overspending, independent account/member sequences and mixed late failure.

Reconciliation evidence covers exact included IDs from earlier history, unchanged-
parent inclusion, included-before-expiry reporting, survivor order, same-height and
shorter replacements, changed/old anchors, newly future windows, wrong configuration
and distinct accepted genesis, no resurrection and stale/cross-pool batches.
Literal boundary checks verify successful/failed state-work charges, historical
scan/revalidation charges, partial admission reservations, sticky exhaustion,
atomic failed reconciliation and no refund from removal/clear/parent changes.

Complete chain equality and exact retained-state observations establish this
in-memory boundary. New C1 tests do not add command/process, socket, persistence,
wallet or power-loss evidence; the existing required process/recovery suites are
retained and rerun. Fallible bounded pool allocation paths are reviewed but induced
OOM is not tested. Abstract work units are not CPU-time, RSS or global DoS limits.
No runtime behavior is inferred merely from the written contract.

**210 of 212 existing repository files are byte-identical to the B4
checkpoint before the later nine-document handoff.** The sole existing Rust change
in production is one `pub mod pool;` export in the V2 node module; removing that
line reproduces its entire earlier body byte-for-byte. **68 of 69 retained
test/helper files** and **173 of 175 retained source/dependency/workflow/reference/
fixture paths** remain byte-identical. The sole retained test exception is the
documented B4 observation-coordination repair; all other 18 test bodies and every
byte outside that function in its file are preserved. Five new files add the pool,
two suites, an isolated synthetic fixture and the contract. Existing command implementations, A1/F2/storage,
lower-layer/V1/consensus/monetary bodies, Cargo manifests/lockfile, workflow commands
and its 20-minute budget, reference tools and frozen vectors are unchanged.
No wire, storage, hash, monetary, enrollment, device or accessibility policy changed.

### Historical C1 continuation and remaining threats

**Next: MB-04C2 — finite local pool intake and inspection. READY — DESIGN FIRST.**

Design the smallest explicit operator path that can feed already signed V2 inputs
into C1 and inspect one local result against a clean accepted A1 chain. Specify the
command, deliberate input source/framing and finite invocation lifetime before
implementation. Preserve C1's exact-parent binding, contextual-only admission,
FIFO batch semantics, independent limits and cumulative work; never treat a pool
receipt as accepted chain state or permission to publish a block.
Require independently selected configuration/genesis identity; signed input must
never select its own network or accepted parent.

Define argument, input-file/stream, item-count and total-byte ceilings before any
input-sized read/copy, and a closed policy for malformed framing, duplicates,
individual rejections, budget exhaustion and a blocked requested batch. Choose
all-or-nothing intake versus bounded per-item reporting explicitly; do not let a
decoder or reporting loop silently reset C1's owner allowance. Keep retained,
contextually admitted, evaluated and canonically included observations distinct.
Choose bounded regular-file intake with explicit file-type/symlink/EOF rules, or
specify absolute deadline, cancellation and terminal-EOF behavior for streams.
Byte and item limits alone do not bound a stalled pipe, FIFO or stdin read.

Define clean controller ownership and selected checkpoint capture, shutdown/error
semantics, bounded output and release-before-output behavior. Any A1 open/replay/
synchronization must retain its existing contract. Report the exact observed
parent and finite local outcome; do not imply durable pool retention across exit,
network propagation, inclusion or currentness after owner release. Test actual
operator processes, malformed/overlong input, independent caps, duplicate/order
handling, contextual/execution errors, output failure and unchanged canonical
history/control selections before claiming operator integration.
Opening the A1 writer to borrow its accepted chain includes ownership, replay and
synchronization; unchanged canonical selection does not turn it into the existing
read-only, no-synchronization `status` path.

No wallet signing/key custody, automatic mining/append, new monetary rule,
transaction gossip, pool persistence, remote submission endpoint or integration
into B4's retained network loop is implicitly authorized by this narrow step.
Those require separately specified increments. Keep existing commands, A1/F2/store
authority, consensus encodings and all policy gates unchanged. C2 is planned only;
broader scheduling, wallet, wider APIs, full MB-04 and Gate A/B remain unfinished.

Run every existing AGENTS.md gate, verify published source and exact-source CI,
then check the later checkpoint head separately and refresh this handoff.

## MB-04C2 finite local input and inspection boundary

**MB-04C2 is VERIFIED_COMPONENT — FINITE LOCAL POOL INTAKE AND INSPECTION.**
C2 adds the finite `pool-inspect` command around C1's accepted in-memory pool and
one clean A1 controller. C1 retention/evaluation/reconciliation, B4 finite run
rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown, A2 operator
commands and the existing storage/peer components retain their boundaries.
MB-04 remains IN_PROGRESS: wallet/key custody, pool persistence/gossip and retained
network-loop integration, wider APIs and broader scheduling remain unfinished.
Complete Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain
UNMET.

### Input trust, ownership and semantic limits

The separate finite `mirror-node-v2 pool-inspect` command requires thirteen
deliberate option/value pairs: the existing roots and independent configuration/
genesis pins, one input path, input byte/count caps, all five C1 pool limits and
an explicit evaluation count. The existing 32-argument, 4096-byte-token and
16384-total-byte ceilings remain. Every argument and scalar relation is checked
before effects; old commands reject C2-only options. No default network or stdin
fallback is introduced.

The file is a bounded local regular file of LF-terminated lowercase hex records
around unchanged signed V2 operation bytes. Empty input is valid; nonempty
records have 2..582 even hexadecimal characters and one LF. Blank/uppercase/CRLF/
odd/nonhex/truncated framing fails before A1 opens. Input caps are independently
1..149248 bytes and 1..256 records; every duplicate counts. Retention, batch and
abstract work retain C1's separate limits. Signed input cannot choose its network.

The input path rejects lexical `..`, stdin-like `-`, symlink parents/leaves,
nonregular objects and multiply linked files. Read-only `O_NOFOLLOW|O_NONBLOCK`
opening, pre/open/post device/inode/type/link/length/mtime/ctime checks and a
fallibly allocated length-plus-one EOF sentinel reject observed changes. The
descriptor closes before A1 opens. Parent namespaces remain trusted: metadata
checks are not hostile-writer snapshot isolation, and nonblocking open prevents
a raced FIFO's writer wait without imposing a regular-filesystem deadline.

One sticky signal guard, one clean `NodeControlV2::open_writer` and one C1 pool
span the invocation. The real immutable accepted chain remains owned throughout
admission/evaluation. A1 still replays and synchronizes; unchanged application
bytes and selected checkpoint do not mean no synchronization. Missing, pending,
corrupt, wrong-network or busy roots retain their existing refusal/recovery rules.

Complete framing precedes sequential FIFO admission. Any file, operation,
capacity or admission-work refusal ends with a static diagnostic, exit 1 and no
normal partial-success result. The ephemeral pool is discarded. Exact duplicates
preserve their retained position and charged identification work, including at
full retention. No failure or retry replenishes the owner allowance.

`--evaluate-count 0` skips evaluation explicitly. A positive count evaluates
exactly that deduplicated FIFO prefix once; a count above retained length is
blocked instead of shortened. Success returns an evaluated count/framed size.
Expected operation/body/ledger/count/byte/work refusal returns `evaluation=blocked`
with exit 1, a closed reason and an available zero-based FIFO ledger index.
Allocation, arithmetic and internal-state failures remain operational diagnostics.
Typed nested membership errors are classified without formatting their text.

One fixed 31-field report names the complete observed checkpoint, input/retention/
duplicate/work counters, every supplied limit and evaluation result. It explicitly
reports `canonical_inclusion=not-checked` and `pool_disposition=discarded`.
The formatter checks scalar, limit, identity, count and evaluation consistency;
normal output and diagnostics remain within 4096 bytes. No raw paths, input
bytes, unbounded operation-ID list or nested error strings are emitted.

Sticky shutdown is checked around bounded reads and successful A1/C1 calls;
already admitted synchronous work finishes or fails before another check. A real
returned error takes precedence over relabeling it as shutdown. The pool,
controller, input bytes and signal guard drop before formatting/stdout. Lost
output retains conservative completion uncertainty and never triggers a retry.

C2 does not reconcile or scan accepted history, retain a pool after exit, sign,
mine, prepare/append/adopt a block, initialize/recover, use a peer or integrate
with B4's run loop. Contextual retention, exact batch executability and canonical
inclusion remain separate. C1/A1 public APIs, lower-layer code/encodings, crate
versions, Cargo.lock, workflow commands and timeout remain unchanged. The sole
dependency access adjustment promotes the existing local mirror-personhood path
dependency for typed nested error classification; no new crate is introduced.

An operator-readable regular file is still untrusted content. Bounded framing
and C1 validation contain this invocation's admitted input; they do not protect
against hostile same-user namespace control or establish global transaction
validity. Signatures do not establish biological uniqueness or authorization to
publish a block. No real wallet/private-key or biometric intake is introduced.

### Exact acceptance and retained limits

Implementation source: **39948d40703ee832d528f662c04f6cd5df7d69e4**.
Tested source tree: **733e483ebaa0ecfba8737fd7fe2e1da12d4cd8f1**.
Parent checkpoint: **b488287ab47dba327d6b6d1efbde86213fb3fbfc**.
[Source PR CI 46097203497](https://github.com/Ahmed-Sobeih/mirror/actions/runs/46097203497)
**attempt 1** passed all **43 steps**. Full job `140831260936` logs confirm
**1,386 runtime tests and eleven compile-fail doctests**, zero failures and
three retained parent-exercised ignored helpers. Actual synthetic checkout
`d699c9be6e52114b2de1481e55d3b50f9a05fd4d` has no file differences from the implementation source.
All **12 published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **213 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed, with all **222 tested source/checkpoint inputs
unchanged during execution**:

- **1,386 workspace/all-target runtime tests**, zero failures: 1,333 retained
  plus **53 new**. Parameterized cases, subprocesses and release reruns are
  not additional tests.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers remain exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **367+1**. All **twelve references** and **sixteen examples** passed.

New executable test inventory:

- `crates/mirror-node/src/bin/mirror-node-v2/args.rs`: **7** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/errors.rs`: **1** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/main.rs`: **1** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/output.rs`: **7** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/pool_input.rs`: **8** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/pool_inspect.rs`: **9** new tests.
- `crates/mirror-node/tests/pool_commands_v2.rs`: **20** new tests.

Focused binary verification first passed all 114 then-registered tests. Source
review then tightened impossible typed-body representation/context/root errors
to operational invariants and added one classifier test, which passed separately;
the final full gates exercise all 115 binary tests. These are 33 new private
tests plus the retained 82-test binary suite.

The first process-suite run passed 19 of 20 functions. Its sole failure was the
test harness's optional Unix-domain socket creation returning EPERM on this host,
before the operator command ran. That complete failed log is retained. Only the
optional socket fixture and unused import were removed; missing-file, directory,
writerless FIFO, symlink-parent/leaf and hardlink checks remain. The final focused
suite passed all 20 functions in debug and release, with no ignored cases. Socket
path rejection is not claimed as observed process evidence. No production
behavior or retained test was weakened to accommodate the host limitation.

The new process suite covers full framing admission before a busy controller;
empty input; malformed operation encodings; exact maximum 149248-byte/256-record
input and distinct count/byte refusals; all four operation kinds; duplicates at
full retention and exact FIFO order; independent retention caps; signature/context
refusal; overspending, gaps/conflicts, funding order and a late mixed-operation
failure; literal cumulative work and body quotas; missing/wrong/busy/corrupt/
unknown/pending A1 state; restart; bounded records; broken stdout; and actual
blocked stdout with input and both authority owners released. Observed SIGKILL
there occurs after computation/resource release, not during admission or staging.

The initial publication `e2c1c68d72380bb4c111d6839119377febed52f2` was not accepted.
Its first full local workspace gate passed all 53 new tests but failed a retained
B2 test before its chosen SIGINT: the helper found that the child had already
exited. That failed path did not retain the child's diagnostic, so the exact
exit class is unknown. The test constructed/transmitted/replayed its candidate
inside a 1000 ms session, then started another 650 ms sleep after observing EOF
receive, without establishing that 650 ms remained. The complete failed gate
and initial publication evidence are preserved separately.

The final source repairs only that test's observation timing. It precomputes
the transcript before launch, anchors the 650 ms offset to completed-request
observation before transmission/replay, sleeps only the remaining offset and
freshly observes the same one-byte EOF receive immediately before SIGINT. The
unchanged 1450 ms elapsed bound uses that earlier epoch; renewing 1000 ms after
the signal would require at least 1650 ms. All three timing constants, the full
post-signal assertions, the other 26 test bodies and all helpers are unchanged.
One exact targeted debug execution passed after the repair. Production B2 and
C2 behavior were not changed. The full final gates and exact-source CI reported
above apply to this repaired source; the earlier publication's CI alone cannot
accept the later repair.

The initial publication's PR CI run `46095907470`, job `140827064527`, ended
cancelled during the workspace gate. Its complete archived log contains all 53
new passing test names and no reported test-failure marker, but not a complete
workflow. That partial run is not source acceptance. The exact final-source run
identified above is verified independently from its full log and actual checkout.

New process tests use actual accepted synthetic chains, real signed operations,
separate roots and bounded child cleanup. Full canonical state, selected
checkpoint, control/store/input bytes and strict reopening are checked. The
observed full-pipe stdout write establishes release of both authority owners and
the input descriptor before output. This evidence is local and finite; it does
not establish production network operation or a wall-clock bound on filesystem I/O.

Private deterministic checks cover bounded framing/read behavior and the closed
error/report boundary. Static unsafe-file cases are actual process evidence.
Concurrent hostile namespace mutation, induced allocator exhaustion and signals
at every internal synchronous validation/synchronization phase are not claimed
as observed process evidence. A1 replay/synchronization retains its accepted
contract; equal file contents do not establish absence of synchronization.

Preservation audit: **71 of 72 existing test/helper files** and
**67 pre-existing private test functions** in changed binary
modules are byte-identical. The exception is the single retained B2 deadline
observer described in the evidence; all **26 other test bodies**, the rest of that
file and its complete post-signal assertions are unchanged. All lower-layer
production source, C1/A1 public APIs,
frozen vectors, reference tools, monetary policy and runtime-data boundaries
remain unchanged. The existing mirror-personhood path dependency is promoted to
normal dependency access for typed nested error classification; no new crate or
version is added, and **Cargo.lock is unchanged**. The existing workflow commands,
pinned toolchain and 20-minute job limit are unchanged. The new contract was
written before implementation, with the dependency-access refinement recorded
before coordinator implementation. No monetary, enrollment, privacy, device or
accessibility choice was silently changed.

### Exact next increment

**Next: MB-04D1 — local wallet and signing boundary. READY — DESIGN FIRST.**

C2 now consumes already signed operations; the next useful MB-04 increment is
an explicit local wallet boundary that can eventually supply those inputs. Begin
with repository inspection and a written threat model/API/file-lifetime design,
then select the smallest reviewable implementation slice. D1 is not implemented
by this checkpoint, and choosing this next slice does not mark broader pool or
node integration complete.

Specify fresh operating-system randomness, key ownership and separate signing,
transport and membership roles; independently selected configuration/genesis;
the exact operation fields and user intent authorized by signing; and a truthful
distinction between constructing/signing bytes, pool admission, ledger execution
and canonical inclusion. Do not let an untrusted input choose its signer, network,
recipient, amount, sequence, anchor or authorization window silently.

Choose deliberate local secret input/output, authenticated protected storage,
file permissions/identity, bounded decoding/KDF work, backup/export/recovery,
cancellation and error semantics before implementation. Review established
cryptographic libraries and official documentation if new dependency support is
needed; pin exact reviewed versions and do not invent encryption or KDF schemes.
Document the limits of memory clearing, filesystem persistence and same-user
trust honestly. Keep real private keys, passwords, recovery seeds and biometric data
out of argv, logs, fixtures, repository files and diagnostics. Tests use fresh temporary
or explicitly synthetic public development material only; known demo keys are
not real wallets.

Keep signing local and explicitly authorized. Do not add automatic mining,
append/adoption, transaction gossip, pool persistence, a remote submission/signer
endpoint, default key selection or B4 loop integration as incidental work.
Existing canonical operation encodings, no-fee/no-mint development behavior,
monetary decisions, enrollment profiles and open privacy/accessibility/consensus
gates remain unchanged. Design separately any later persistence/gossip or retained
pool lifecycle across chain changes; C2 still discards its owner after each call.

Retain all existing gates and test bodies. Verify exact implementation source,
full local/CI evidence and published readbacks before the later nine-document
checkpoint; verify checkpoint-head CI separately and refresh `the-prompt`.
