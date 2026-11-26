# V2 durable node control and restart authority

Implemented contract: **MB-04A1**, in the `mirror-node` library's `v2` module.
This document specifies the source behavior. Executed acceptance results, exact
implementation publication and later documentation-head CI are recorded separately
in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and the
[master plan](../MASTER_PLAN.md).

The component retains independently selected network identity and a complete local
checkpoint, and persists both accepted histories before one preferred branch can
replace durable storage. A process restart with unfinished intent requires an
explicit operator recovery action. The existing V1 executable remains separate.
There is no V2 command-line surface or persistent server in this increment.

## 1. Trust and ownership

Every entry point receives independently selected `ManifestIdentityV2`: the full
configuration hash C and genesis hash. The operator also supplies separate control
and profile-3 store paths. The control format does not discover, encode or select
a filesystem path. The operator is responsible for choosing the intended pair.

The control directory is trusted local operator state. Its confirmed record is the
retained full checkpoint and local generation used on restart. Its intent is local
authorization for the one specific old/candidate transition it records. The network
identity remains an external input; neither a peer, a higher generation directory
nor a valid stored checksum chooses it. Checksums detect corruption and inconsistent
bytes. They cannot authenticate authority against an owner able to replace all local
state. Local generation is neither branch score nor global freshness or finality.

Control and store paths must be nonempty, separate and non-nested. Checked path
components must be directories; symbolic links, `..`, a managed filesystem root
and existing control/store roots referring to the same directory inode reject.
Missing parents are not created. The local owner and parent directories are trusted.
These static path and inode checks do not provide protection against hostile
same-user namespace races or privileged mount manipulation.

All filesystem entry points support the verified Linux locking profile. They acquire
a nonblocking kernel lock on the stable control-directory inode **before** any store
lock. Ready and recovery operations use exclusive control ownership; read-only
status uses shared ownership. Existing storage then applies its own ownership rules.
Contention returns an error; there is no lock wait/retry loop. A ready owner's store
is dropped before its control lock. No operation renames or replaces a live root
directory inode. Kernel descriptor ownership, not a PID or lock-file payload,
determines exclusion.

`NodeControlV2` keeps its `ReplacementStoreV2` private. It exposes a borrowed accepted
chain and copied checkpoint, but no method extracts an ordinary storage writer.
`adopt` consumes the ready owner and received candidate. A failure returns an error
without an ordinary writer; owned descriptors are released. Recovery reacquires
ownership explicitly and revalidates persisted authority. All clients must cooperate
with these ownership and path-pair conventions.

## 2. Confirmed MNC2 record

`ControlRecordV2` is exactly **464 bytes**. All integers are unsigned little-endian;
hash fields are their exact 32-byte representation. Version is 2 and kind is 1.

| Offset | Length | Field |
|---:|---:|---|
| 0 | 4 | Magic `MNC2` |
| 4 | 2 | Version, 2 |
| 6 | 2 | Kind, 1 |
| 8 | 8 | Local replacement generation |
| 16 | 32 | Configuration hash C |
| 48 | 32 | Genesis hash |
| 80 | 8 | Selected post-genesis height |
| 88 | 32 | Selected tip hash |
| 120 | 32 | Selected complete ledger root |
| 152 | 280 | Unchanged canonical MSR2 replacement-selection record |
| 432 | 32 | MNC2 checksum |

The checksum is SHA256d of the exact ASCII bytes
`MIRROR_NODE_CONTROL_RECORD_V2` followed immediately by bytes `[0, 432)`, without
a delimiter or terminator. The nested MSR2 and MSS2 checksums retain their existing
domains and encodings. There is no padding or permitted trailing data.

The explicit 144-byte checkpoint repeats fields contained in MSR2. The decoder
passes this complete checkpoint to the existing MSR2 decoder; disagreement in any
duplicated field rejects. This avoids reading private offsets out of the old codec.
The embedded contract enforces its own counts, lengths, generation bound and
checksum rules. In particular, generation cannot exceed selected height.

`ControlRecordV2::from_chain(identity, chain, generation)` derives metadata from a
complete accepted history and checks independent network identity. `decode_for`
checks exact length, magic/version/kind, outer checksum, independent identity and
the nested record. Decoding this fixed record alone does not replay a history or
assert current store contents. Whenever a clean store is opened or inspected, the
node compares **both its full checkpoint and complete canonical MSR2 selection**
with the control record after existing storage has replayed the selected history.
Matching endpoint fields alone are insufficient.

## 3. Two-history MNI2 intent

`IntentRecordV2` owns both exact accepted histories and their local checkpoints.
Its constructor requires the old checkpoint, old chain, candidate chain and
independent network identity. It repeats the existing `prepare_replacement_v2`
checks: correct networks, complete old checkpoint and metadata, strictly preferred
candidate, and exactly one local generation increment. Identical, tied and shorter
branches cannot construct replacement intent.

MNI2 contains two unchanged complete MPN2 frame streams. Let `a` be the old-stream
length and `b` the candidate-stream length.

| Offset | Length | Field |
|---:|---:|---|
| 0 | 4 | Magic `MNI2` |
| 4 | 2 | Version, 2 |
| 6 | 2 | Kind, 1 |
| 8 | 464 | Old MNC2 record |
| 472 | 464 | Candidate MNC2 record |
| 936 | 8 | `a`, old complete MPN2 stream length |
| 944 | 8 | `b`, candidate complete MPN2 stream length |
| 952 | `a` | Old Begin/Manifest/all blocks/End stream |
| `952 + a` | `b` | Candidate Begin/Manifest/all blocks/End stream |
| `952 + a + b` | 32 | MNI2 checksum |

The fixed header is **952 bytes**. Total encoded length is exactly `984 + a + b`.
Both declared lengths must be positive and at most **19,641,112 bytes**, so the
complete intent ceiling is **39,283,208 bytes**. No third history, padding or
trailing bytes are accepted. The checksum is SHA256d of the exact ASCII bytes
`MIRROR_NODE_REPLACEMENT_INTENT_V2` followed immediately by the whole prefix before
the final checksum.

The decoder checks envelope length, framing and both declared bounds before
history execution. It verifies the outer checksum and both MNC2 records under the
external identity. For each stream, the consuming F1 receiver admits every fixed
header before accepting the exact bounded payload. Every original block executes
against that history's own actual accepted ancestry; a matching End and consuming
`finish` are required. The enclosing exact stream length supplies the local-file
boundary. It does not change the separate terminal socket EOF requirements in F2.

After both replays, the decoder repeats strict replacement preparation and compares
both complete derived canonical control records with the encoded records. A valid
checksum, claimed endpoint or internally coherent byte total cannot substitute for
actual history execution. The returned intent exposes no partially decoded chain.
It retains complete balances, membership, approvals, sequences, population history
and resource counters for both histories; no state is grafted across them.

`validate_confirmed` accepts only exact equality with the old or candidate control
record and returns `ConfirmedSideV2::Old` or `Candidate`. Its `request` method borrows
both histories for the existing replacement-recovery API. These pure codec methods
perform no filesystem operation and do not publish or recover a store themselves.

## 4. Control directory and entry points

The control directory is outside the strict store root. Its names are:

| Name | Purpose |
|---|---|
| `confirmed.v2` | Exact current MNC2 checkpoint record |
| `intent.v2` | Complete MNI2 authorization for at most one replacement |
| `intent.pending` | New intent being written and validated; never accepted by startup/recovery |
| `confirmed.pending` | Initial control staging, or an authorized candidate-confirmation prefix |

New control directories are created with mode `0700`; new managed files use
`create_new` and mode `0600`, subject to the process umask. Existing trusted-directory
permissions are not rewritten. Managed payloads must be regular, non-symlink files
with a single link. Unknown entries and unsafe files reject and remain preserved.
Enumeration admits at most four names before rejecting the fifth; accepted startup
layouts contain at most the three recognized published/confirmation names.

| API | Implemented behavior |
|---|---|
| `NodeControlV2::initialize(control, store, identity, expected)` | Explicitly creates control for an **already initialized** profile-3 store under a caller-retained full checkpoint/generation. It does not initialize or repair storage. |
| `NodeControlV2::open_writer(control, store, identity)` | Opens only clean control and a clean fully replayed matching store, then resynchronizes confirmed control before returning a ready owner. |
| `NodeControlV2::status(control, store, identity)` | Read-only application behavior: no creation, synchronization, peer contact or repair. Reports Missing, Clean or RecoveryRequired, or a validation/ownership/I/O error. |
| `node.adopt(received)` | Consumes a ready owner and complete F2 candidate. Keeps current without filesystem calls on ties/shorter/identical histories; otherwise persists intent before existing storage replacement. |
| `NodeControlV2::recover(control, store, identity, action)` | Explicitly reloads/replays and resynchronizes complete intent, then applies the permitted Complete or RetainOld path. |

For otherwise valid paths, absent or empty control is `Missing`. The store contents
do not turn it into initialized control. `Clean` requires a matching fully replayed
clean store. `RecoveryRequired` reports the confirmed, old and candidate checkpoints
from validated local intent. It deliberately does **not** assert which history the
store currently selects, inspect or repair its unresolved layout, or decide an
action. Invalid or inconsistent control returns an error instead of a guessed
status. A valid unresolved intent makes ordinary writer opening fail with
`RecoveryRequired`.

Initialization acquires control before store ownership, requires an empty control
directory, and opens the existing store against the independent full expected pin.
It writes/synchronizes `confirmed.pending`, verifies its exact bytes and codec,
renames it to `confirmed.v2`, synchronizes the control directory and synchronizes
its parent before returning. Failed initialization never falls back to store
creation, replacement or repair. An empty directory left before staging remains
uninitialized; partial initialization staging remains preserved and rejected.

## 5. Preferred adoption ordering

One ready owner holds both locks throughout successful adoption:

1. Check the in-memory store's full canonical control metadata and compare the two
   accepted histories under the independent identity. A keep-current result returns
   the same selected checkpoint and generation without filesystem operations.
2. For strict preference, clone the exact old accepted chain once, move the received
   candidate, and construct the validated two-history intent before publication.
   Require only `confirmed.v2` and verify its exact bytes.
3. Create `intent.pending` without overwriting anything, write its complete bounded
   bytes and synchronize the file. Read it back under the shared per-call control
   allowance, compare exact bytes, decode/replay both histories and compare them with
   the original accepted inputs.
4. Require the exact expected names, rename the completed pending file to
   `intent.v2`, and synchronize the stable control directory. Recheck names, the
   confirmed bytes, exact published intent and locked root before any store change.
5. Invoke existing `ReplacementStoreV2::replace` under the same store ownership.
   Its complete validation, staging, selection, cleanup, replay and synchronization
   rules remain unchanged. Its successful checkpoint must equal the intent candidate.
6. Write the candidate record to a newly created `confirmed.pending`, synchronize
   and verify it, recheck the prior confirmed bytes, then rename it over
   `confirmed.v2`. Verify and synchronize the selected confirmed file and control
   directory before retiring intent.
7. Recheck the exact intent and expected names, remove only `intent.v2`, and
   synchronize the control directory. Only then return the ready candidate owner.

An error consumes the owner and returns no writer. If store publication may have
started, complete durable intent already exists. Neither the error value nor an
old last-confirmed checkpoint is a claim about current disk selection. No failure
automatically chooses completion, retention, cleanup or a retry.

## 6. Recognized restart states and explicit recovery

The following table separates control-layout recognition from the store checks
needed to return a ready owner. `old` and `candidate` mean exact whole MNC2 records
from a completely decoded/replayed intent.

| Control layout | Startup/status | Explicit recovery |
|---|---|---|
| Absent or empty directory | Missing | No authority; reject |
| Only valid `confirmed.v2` | Clean only if strict store replay and all canonical metadata match | No pending intent; reject recovery |
| Confirmed old + complete `intent.v2` | RecoveryRequired | Complete or RetainOld through existing bounded replacement recovery, subject to actual recognized store layout |
| Confirmed candidate + complete `intent.v2` | RecoveryRequired | Complete only, with a strict clean candidate-store open |
| Confirmed old + complete intent + `confirmed.pending` equal to a candidate-record prefix | RecoveryRequired | Complete only, with a strict clean candidate-store open; append only the exact missing suffix |
| Any `intent.pending`, including a complete valid one | Invalid; preserve | Reject; no incomplete-intent recovery is introduced |
| `confirmed.pending` without complete published intent | Invalid initialization/confirmation residue; preserve | Reject |
| Confirmed candidate plus `confirmed.pending` | Invalid; preserve | Reject |
| Foreign, corrupt, overlong, conflicting or unknown bytes/names | Invalid; preserve | Reject |

The permitted candidate-confirmation prefix may contain **0 through 464 bytes**,
including an empty file or the full exact record. It is authorized only by the
complete persisted intent while `confirmed.v2` still names old. The presence of
confirmation staging means store publication already returned clean; recovery must
therefore open the exact clean candidate. Old-selected or residual store layouts
in this state are inconsistent and reject before control completion. A prefix does
not by itself establish its origin or permit changing storage.

With confirmed old and no confirmation staging, recovery delegates to the unchanged
explicit replacement-recovery contract using both exact accepted histories and pins.
Its recognized candidate-staging and old-cleanup layouts remain bounded and
byte-authorized. A selected candidate cannot roll back through RetainOld. No higher
directory, selector checksum, peer endpoint or scan can supply missing authority.

Before mutating storage during recovery, the node re-reads and synchronizes the exact
`intent.v2` file, synchronizes the control directory and its parent, then rechecks
the complete expected layout and exact confirmed/intent/pending bytes. This also
handles a visible intent rename whose earlier directory synchronization failed.

After storage returns a fully checked selected history, the node compares its
complete checkpoint and MSR2 metadata with the appropriate intent record. Candidate
confirmation staging is extended by appending the authorized suffix without
truncation. The completed pending file is synchronized and verified before rename.
Whether a new confirmation rename is needed or the correct record is already
visible, the selected confirmed file and control directory are resynchronized
before intent removal. RetainOld likewise resynchronizes the retained confirmed
record before retiring intent.

If an intent unlink is visible after a failed final directory synchronization,
clean `open_writer` fully reopens storage and resynchronizes `confirmed.v2`, the
control directory and its parent before returning another writer. Read-only status
does not perform that synchronization. Recovery failure returns no owner; a later
explicit call starts with fresh locks, input validation and bounded reads. Repeated
interruption cannot authorize truncating a foreign prefix or rolling back a
confirmed/selected candidate.

## 7. Resource and compatibility boundaries

| Bound | Value and scope |
|---|---|
| One MNC2 record | Exactly 464 encoded bytes |
| One complete history in intent | At most 19,641,112 MPN2 bytes |
| One MNI2 intent | At most 39,283,208 encoded bytes |
| Control payload-read allowance | **314,269,392 bytes per call**, shared across all control reads and exact-byte comparisons |
| History execution | Existing per-history chain/F1 limits, including 4,096 blocks, 16 MiB of MRB2 and 1,048,576 state/configuration items |

The control read allowance is `8 * (39,283,208 + 464 + 2)`. Before opening each
payload for reading, metadata length is checked against the appropriate fixed or
maximum length, then **length plus one sentinel byte** is charged to the same
allowance. A bounded read checks exact observed length and file identity before
and after access. Synchronization reads and repeated comparisons use that allowance
as well. Storage separately enforces its own existing read/replay budgets.

These are encoded-input and abstract-work bounds, not combined resident-memory,
allocation-success, wall-clock or availability guarantees. Decoding retains both
accepted histories; adoption readback and checksum construction also allocate
bounded temporary buffers. There is no socket deadline on control publication or
recovery. This finite component does not supply global connection or memory admission.

This layer composes the existing [peer history](PEER_HISTORY_V2.md),
[TCP sessions](PEER_SESSION_V2.md), [fork choice](FORK_CHOICE_V2.md),
[replacement storage](STORAGE_REPLACEMENT_STORE_V2.md) and
[explicit replacement recovery](STORAGE_REPLACEMENT_RECOVERY_V2.md). It preserves
their prior byte formats/hash domains, profile-3 store layout and accepted-state
rules. MNC2/MNI2 add separate local control domains; they do not change consensus,
money, issuance, enrollment, biometric/device handling or V1 behavior.

## 8. Verification scope and next work

The codec suite checks independently generated complete records/intents, corruption,
all checkpoint fields, lengths and generations, matching whole histories, invalid
actual ledger roots in either history, and valid-checksum metadata that disagrees
with execution. The independent Python reference packs the new local formats from
the established synthetic chain/history and storage references without consuming
Rust output.

Filesystem integration and private process tests exercise clean restart, read-only
status, malformed/unsafe control, full-pin mismatch, retained and preferred genuine
membership forks, explicit completion/retention, no selected-candidate rollback,
and competing owners. Deterministic private boundary hooks let parent tests stop
separate processes during initialization, intent writing/publication, after store
publication, during confirmed-record writing/rename and during intent retirement,
including repeated interruptions during recovery. Production has no public or
environment-variable mechanism to enable those hooks. The finite demonstration
also uses independent processes for initialized control, status, tie retention,
preferred adoption and strict reopening.

These process termination scenarios do not emulate physical power loss or certify
filesystem/device persistence behavior. Cooperative Linux operation, trusted local
parents/owner and the existing storage assumptions remain material. WSL, DrvFS and
other untested filesystems are not declared verified by this contract. Final executed
counts and exact-source CI belong to the checkpoint; this specification is not an
external security audit or Gate A/B acceptance.

**Next: MB-04A2 — separate V2 init/status/sync-once/explicit-recover operator
commands.** The implemented A1 library and finite harness are their foundation.
Persistent serving, reconnect/rejoin scheduling, multi-peer policy, transaction
pool, wallet and final human consensus remain later work.
