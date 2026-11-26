# Mirror membership integration threat model

Historical design checkpoint: **MIRROR-MB01A-001**. Scope: MB-01A source review
and V2 design. Current implementation update: **MB-03E2a durable whole-branch
replacement and strict clean-state reconciliation**, below. Exact acceptance evidence is
tracked in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and master-plan section 23.
Source inspected at `db2861930cc0f01f735ccfba80a4f530f4d1ddd6`; PR #1 comments
were reviewed and contained checkpoint/roadmap notes, not an independent security
review. This is an engineering review, not an audit certification.

Read with [the proposed integration specification](LEDGER_MEMBERSHIP_V1.md) and
[the test matrix](MEMBERSHIP_TEST_MATRIX.md). No issue below is represented as fixed
merely because a mitigation is documented. At the original design checkpoint,
no V2 acceptance test had run.

The preceding sentence describes the original design checkpoint. Current B1–B4,
MB-02A/B/C and MB-03A/B/C/D/E1 evidence, plus the current MB-03E2a verification status,
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
fencing under stable cooperative ownership. Explicit interrupted replacement
recovery, arbitrary corruption repair, peer integration and human finality remain unfinished;
source publication/CI is tracked separately.

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
network; it is not replaced by whichever config a peer advertises. Seed people
are an explicit initial trust assumption, not proven unique by self-signatures.

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
Arbitrary repair, all physical failure modes, durable reorganization and hostile
local-owner defenses remain unfinished. A valid old record is not proof of latestness;
the trusted caller must retain independent full branch and generation checkpoints.

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
- Disk metadata, peers and caches cannot choose their own trusted C/genesis or
  final height/tip/ledger root. Reopen must reconstruct the complete selected
  history within existing byte, count and state-work budgets.
- An uncertain storage-publication error yields no usable writer. Initialization
  and reconciliation failures return no handle; append failure retains ownership
  but fences chain/selection access and further writes. Complete visible selection
  must be checked against retained external pins and resynchronized before writing;
  an error must not be relabeled proof that nothing reached disk. Normal opening
  preserves and rejects staging. Explicit recovery may change only the exact
  recognized residual paths authorized by independent parent/candidate pins and
  the exact child; mismatched or unexplained bytes stay untouched. It must never
  silently roll back an already selected candidate or return mixed ledger state.
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
**Next: MB-03E2b explicit interrupted-replacement recovery**, followed by
fresh-peer reconstruction. Repair of partial staging/cleanup, human finality,
a live V2 daemon and Gate A/Gate B remain unfinished.
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

## MB-03E2a durable replacement and strict reconciliation update

The [replacement-store contract](STORAGE_REPLACEMENT_STORE_V2.md) specifies
`ReplacementStoreV2` around unchanged MSR2 profile-3 metadata. The implementation
adds explicit initialization, independently pinned writer reopening/shared
inspection, complete accepted-branch replacement, failure fencing and consuming
clean-state reconciliation. It does **not** implement repair of interrupted
replacement staging or cleanup. Those states remain rejected and preserved until
an independently authorized E2b recovery operation exists.

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

F08 is mitigated for this bounded successful replacement and clean-reconciliation
path, **not closed for arbitrary interruption repair or deployment**. MB-03E2b,
fresh-peer reconstruction, persistent V2 node integration, human finality, private
biological matching and other existing policy/security gates remain unfinished.
No founder/marketing privilege, monetary formula, mint/reward API, real biometric
capture or anonymity guarantee is introduced. MB-03 and Gate A/B remain incomplete.
