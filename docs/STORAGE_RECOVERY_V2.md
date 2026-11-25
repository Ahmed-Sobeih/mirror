# V2 explicit recovery of an interrupted append

**MB-03C. Updated 2026-11-20.** This contract extends the explicit profile 2
`AppendStoreV2` lifecycle with recovery of one independently authorized child
append. The existing [append contract](STORAGE_APPEND_V2.md), immutable
[profile 1 contract](STORAGE_V2.md), and [actual-chain rules](CHAIN_V2.md)
retain their existing byte formats and normal-open behavior.

This is a narrowly specified recovery operation. It neither discovers a canonical
branch nor repairs arbitrary filesystem corruption. An explicit caller chooses
whether to complete one retained candidate or keep its parent while the parent
is still selected. Unknown data, unsupported states and mismatched expectations
are rejected and preserved. The current acceptance evidence belongs to
CURRENT_STATUS.md and MASTER_PLAN.md; this document defines required behavior.

## 1. Independent authority and exact request

The caller supplies all of the following independently of the files being repaired:

- `ManifestIdentityV2`: the selected configuration hash and exact genesis hash.
- A retained parent `AppendCheckpointV2`, including its storage generation and
  full configuration/genesis/height/tip/ledger-root checkpoint.
- A retained candidate `AppendCheckpointV2`, with the same network identity and
  exactly one greater height and generation.
- The exact complete candidate `BlockV2`.
- An explicit recovery action, `Complete` or `RetainParent`.

The public request has this construction and inspection interface:

```rust,ignore
AppendRecoveryRequestV2::new(parent, candidate, block)
request.parent()
request.candidate()
request.block()
```

The request owns its candidate and exposes no mutable setters. Construction checks
the request's structural relationships; it does not establish that the candidate
was accepted, that an append ran, or that any files belong to the request. Actual
recovery must replay the complete parent and validate the proposed child through
`ChainV2`, including the exact resulting candidate checkpoint and all resource
charges, before changing any file. These checks apply to both recovery actions.

The request is **trusted caller authority for this recovery attempt**. The caller
must retain the intended parent, candidate and exact child independently, or
reestablish that authority through an explicit trusted operator decision. Reading
expectations out of an arbitrary selected/pending marker or accepting a peer's
suggested repair removes this protection. This component does not persist the
caller's recovery authority or provide an operator interface for choosing it.

A prefix match constrains what bytes this request may change; it is not proof of
historical origin. In particular, an empty file or common prefix cannot prove that
a previous Mirror invocation created it. The trusted caller explicitly authorizes
the two named staging paths for this exact candidate under the trusted-owner
filesystem model. No checksum, filename, generation or partially written file
chooses the request or the action.

There is no new journal, intent file, storage profile, checksum domain or wire
encoding. Existing MNM2 manifest, MRB2 block, MSS2 and MSU2 bytes remain unchanged.
The independently frozen block and selector fixtures remain the byte authority.
If future recovery requires an on-disk intent with stronger provenance or different
failure states, it requires a separately specified format and reference vectors.

## 2. Public entry points and ownership

```rust,ignore
AppendStoreV2::recover_interrupted(root, identity, &request, action)
store.recover(&request, action)
```

The first operation opens an existing profile 2 store and holds exclusive
root-directory ownership throughout inspection, validation, mutation and final
reconciliation. It never initializes a missing store or migrates another profile.

The consuming operation uses the writer's **same existing lock**; it does not
release and reacquire ownership. The handle's last confirmed checkpoint must equal
the request's parent or candidate. A read-only handle cannot recover. A previously
fenced writer can use this operation because recovery does not trust its old
in-memory chain as the current disk state.

Success returns one unfenced, exclusively owned `AppendStoreV2` whose complete
selected state has been replayed and synchronized. Every error returns no usable
handle. Closing the failed operation releases its own lock descriptor; inherited
descriptors retain the existing last-reference lifetime rule. There is no raw
descriptor, lock upgrade, PID takeover or explicit unlock escape.

The root inode stays stable. All processes must honor the cooperative Linux lock;
the local owner and parent path remain trusted. Existing checks reject static
symlinks, unsuitable file types, hardlinked managed files and observed inode
replacement. They do not claim protection against a hostile same-user process
that changes the namespace concurrently. Correct file/directory synchronization
and atomic same-directory rename remain explicit filesystem assumptions.

Normal `open_writer`, `inspect` and clean-state `reconcile` remain strict: they do
not acquire hidden cleanup or recovery behavior. The immutable `StoreV2` remains
unmodified. This recovery API is for profile 2 only.

## 3. Recognized layout and provenance checks

The root must contain exactly `manifest.v2`, `blocks/`, `selection.v2`, and
optionally `selection.pending`. A missing, partial, corrupt or foreign selected
marker is rejected. Interrupted initialization is outside this contract.

The blocks directory must contain every complete parent block from height zero
through the independently selected parent height, and at most one additional
file at the exact candidate height. Names retain the canonical 20-digit height
and `.mrb2` suffix. No extra height, noncanonical name or unexplained root entry
is ignored or removed.

The selected marker must equal the exact canonical parent MSU2 selector or the
exact canonical candidate selector after full replay derives those values. A
checksum-correct marker with different metadata is still a mismatch. The local
generation cannot establish freshness, finality or permission to select a branch.

The two possible residual files are classified only against independently
authorized canonical bytes:

- The next-height block is absent, an exact proper prefix of the candidate's
  complete MRB2 encoding, or that exact complete encoding.
- `selection.pending` is absent, an exact proper prefix of the candidate's
  canonical 280-byte MSU2 encoding, or that exact complete encoding.

Proper prefixes include the empty byte string. A file longer than its expected
encoding, a single differing byte, extra trailing bytes, an unsafe file identity
or an unknown file is rejected before mutation. A complete alternative valid
block or selector does not become repairable merely because it is valid in some
other context. There is no truncation, replacement of a mismatched file, cache
restore or best-effort branch selection.

The supported observations are deliberately narrower than arbitrary combinations:

| Selected marker | Candidate file | Pending selector | `Complete` | `RetainParent` |
|---|---|---|---|---|
| Exact parent | Absent | Absent | Resume the explicit candidate append | Resynchronize and retain parent |
| Exact parent | Empty/proper prefix | Absent | Append only the missing block suffix, then publish | Remove only that authorized prefix, then retain parent |
| Exact parent | Exact complete candidate | Absent | Prepare and publish the candidate selector | Remove only the authorized unselected block |
| Exact parent | Exact complete candidate | Empty/proper prefix or exact complete candidate selector | Complete selector if needed, then publish | Remove pending selector, then unselected block |
| Exact candidate | Exact complete candidate | Absent | Replay/resynchronize; idempotent success | Reject; selected history is not rolled back |
| Exact parent | Absent or partial candidate | Any pending selector | Reject and preserve | Reject and preserve |
| Exact candidate | Missing or partial candidate | Any | Reject and preserve | Reject and preserve |
| Exact candidate | Exact complete candidate | Any pending selector | Reject and preserve | Reject and preserve |
| Anything else | Any | Any | Reject and preserve | Reject and preserve |

Allowing parent plus no residual files to `Complete` is an explicit resumption of
the retained append request, not evidence that a previous write occurred. Requiring
a complete block whenever a pending selector exists preserves the preparation
ordering of the original append and the recovery operation. A selected candidate
with incomplete required data indicates an unsupported state; recovery never fills
holes in already selected history.

## 4. Bounded inspection and validation

Recovery does not relax the existing finite chain caps:

- At most 4,096 complete blocks including genesis; a parent admitting a child has
  at most 4,095 blocks.
- At most 16,777,216 bytes of complete MRB2 history, including the **full intended
  candidate**, even if only a small prefix is currently on disk.
- At most 1,048,576 cumulative abstract state/configuration items under the
  unchanged ChainV2 accounting rules.
- At most 2,781,636 manifest bytes, and exactly 280 bytes per complete MSU2 selector.

Root enumeration admits at most four known entries. Block enumeration admits only
the bounded parent count and optional one-child tail. File metadata, individual
lengths and aggregate complete-history bytes are checked before block payload
allocation. Reads retain the checked-length-plus-one sentinel and opened-file
identity checks. Pending data is bounded to 280 bytes, and the optional tail is
bounded to the exact requested block's length.

The maximum distinct recognized encoded payload, with the full candidate and both
selectors present, is **19,559,412 bytes**, exported as
`MAX_APPEND_RECOVERY_INPUT_BYTES_V2`. It is not a resident-memory ceiling or the
total I/O of all comparisons, repeated validation and synchronization.

A separate enforced per-call allowance bounds repeated payload reads:

```text
MAX_APPEND_RECOVERY_READ_BYTES_V2
    = 8 * (19,559,412 + 4,096 + 4)
    = 156,508,096 bytes
```

Every bounded read is charged for its allowed payload length plus one sentinel
byte before reading. The existing final strict clean-state load is charged an
up-front conservative full selected-input ceiling plus per-file sentinels; its
subsequent selector comparisons are charged separately. The allowance may charge
more than the bytes actually returned. It is neither a measured I/O total nor a
resident-memory, metadata-operation or write-volume limit. Enumeration and metadata
checks retain their separate finite entry bounds. Exhaustion rejects instead of
resetting the meter or retrying internally. Recovery contains no retry loop or
scan that can grow with an unbounded set of residual files.

Before any mutation, recovery must:

1. Check independently supplied identities, checkpoint adjacency and candidate
   header relationships, and acquire/retain exclusive ownership of the stable root.
2. Preflight the exact supported layout and all file lengths.
3. Decode the pinned MNM2 manifest and verify every seed proof.
4. Replay all complete parent MRB2 blocks through `ChainV2::replay_encoded` against
   the full independent parent checkpoint.
5. Accept the exact requested child through `ChainV2::append`, checking every
   normal ancestry, height, timestamp, work, body, signature, account, membership,
   root and budget rule. Match the full independent candidate checkpoint.
6. Derive the exact canonical old and new selectors; verify the current marker
   and the optional residual bytes under the table above.
7. Recheck the selected marker and relevant observed bytes before mutation and
   immediately before publication or removal. A changed observation is an error.

No stored account, membership, approval, population or resource counter replaces
reexecution. Neither action discards accepted history to regain capacity.

## 5. Completing the candidate

If the exact candidate is already selected with a clean complete layout, recovery
performs full checked reconciliation and synchronization without changing its
generation or appending it a second time.

While the exact parent is selected:

1. Synchronize the validated parent prerequisites under the retained lock before
   publishing a child. Recheck root identity and the exact parent selector.
2. If the candidate file is absent, create only its exact canonical path with
   exclusive creation. If it is a known prefix, open without truncation and verify
   the observed identity, length and prefix again before extending it.
3. Append only the missing canonical suffix. Verify the completed block bytes,
   synchronize the file, then synchronize `blocks/`. A failed partial write can
   leave only a prefix of the same authorized candidate.
4. Create or similarly finish the known candidate `selection.pending`. Verify
   its complete canonical bytes and synchronize it.
5. Recheck the stable root, unchanged parent selector and prepared candidate
   prerequisites. Mark publication as attempted before renaming the pending
   selector over `selection.v2`.
6. Synchronize the stable root after publication.
7. Require a clean complete candidate layout, perform full selected replay and
   synchronization, and only then return an unfenced writer at the candidate
   checkpoint. No fallible publication work may follow a successful return.

There is no truncation, rewriting of a differing prefix or deletion of parent
blocks. The manifest, prior accepted blocks and locked root are never replaced.

## 6. Retaining the parent

`RetainParent` is permitted only when the exact parent is still selected. It is
not a rollback of a published candidate or a branch-reorganization API.

After full validation of both parent and authorized candidate:

1. Recheck root identity and the exact parent selector.
2. If a known candidate pending selector exists, recheck its exact observed bytes
   and safe current regular-file identity, remove only that file, then synchronize
   the root directory.
3. Recheck the parent selector. If the known unselected candidate file exists,
   recheck its exact observed bytes and safe current regular-file identity, remove
   only that file, then synchronize `blocks/`.
4. Require a clean parent layout, replay it again against the independent parent
   checkpoint, synchronize its files and directories, and return an unfenced
   writer at that same checkpoint.

Deleting pending metadata before the candidate block preserves the recognized
state table if recovery itself is interrupted. An already clean selected parent
is an idempotent success after full validation and synchronization. Successful
retention does not increment a generation, erase an accepted block, pay a reward,
or change the parent's accounts or membership state.

## 7. Errors, restart and repeated interruption

`AppendRecoveryErrorV2` preserves the distinction between rejection before repair,
an incomplete recovery operation, and publication uncertainty. Errors inside an
operation use `RecoveryIncomplete` before candidate publication is attempted.
They use `PublicationUncertain` once a rename may have been attempted or when the
candidate was already selected at entry and its reconciliation fails. No error
returns a usable writer or silently chooses a different action. The caller retains
the exact request and explicitly retries or investigates.

Before a candidate-selector rename is attempted, a failed completion can leave a
recognized parent plus absent/prefix/full candidate data and an optional known
pending selector only after the block is complete. Once rename may be attempted,
the selected marker may be old or new; an error cannot assert that the old marker
is still selected. Retrying `Complete` recognizes the complete candidate as an
idempotent result. Retrying `RetainParent` against that candidate rejects.

A failed retention can leave the parent with both known residual files, just the
unselected candidate, or no residual files. Retrying the same action validates the
new observation before continuing. A failed directory synchronization does not
prove that a preceding creation/removal/rename persisted; revalidation and a new
successful synchronization establish only the current checked boundary.

If any observed bytes stop matching the retained request, ordinary safety rules
apply: reject and preserve. Missing or corrupt selected markers, corrupt parent
blocks, unknown artifacts, multi-block leftovers, interrupted initialization and
selected-candidate corruption require separate explicit investigation. This API
does not guess which bytes are authoritative or delete data merely to make a
store reopen.

## 8. Verification and scope

Acceptance must cover both entry points, both actions, every supported state-table
row, complete ledger/counter equality after recovery, exact encoded bytes, wrong
network/parent/candidate/generation pins, unsafe or unknown paths, bounded inputs,
and a real next append after recovery. Fault tests must interrupt preparation,
extension, file/directory synchronization, deletion, selector comparison,
publication and final reconciliation, then retry under the same independent
request. Check continued lock ownership for consuming recovery and actual
cross-process writer exclusion without sleeps or weakened assertions.

Expected bytes come from the existing independent MRB2/MSU2 references; no fixture
may be changed to agree with an implementation bug. Existing strict normal-open
and immutable-storage tests remain required. A synthetic runnable recovery example
must use only its own new temporary directory and public development fixtures.

Injected failures and process-ownership checks do not simulate arbitrary torn
kernel writes, storage-controller caches or physical power loss. The existing
Linux/filesystem assumptions remain explicit; untested WSL/DrvFS, native Windows
and network filesystems gain no certification from this work.

MB-03D's separate [development comparator](FORK_CHOICE_V2.md) now compares complete
accepted branches under independently selected network identity. It performs no
storage mutation and cannot turn this one-child recovery into a reorganization.
MB-03 remains in progress. Next is MB-03E durable membership-aware branch replacement
with reviewed publication/interruption/recovery rules, followed by fresh-peer
reconstruction. No monetary rule, founder allocation,
marketing reserve, mint/grant/reward API, human-finality rule, real biometric input
or anonymity guarantee is introduced. Gate A and Gate B remain unmet until their
separate acceptance requirements are demonstrated.
