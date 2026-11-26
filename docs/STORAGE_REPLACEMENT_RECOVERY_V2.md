# Explicit interrupted replacement recovery — V2 profile 3

This contract defines **MB-03E2b**, the explicit recovery operation for the
[generation replacement store](STORAGE_REPLACEMENT_STORE_V2.md). The published
implementation, exact-source verification and next task are recorded in
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).

The existing MSR2/MSS2/MSU2, MNM2 and MRB2 encodings remain unchanged. Recovery
introduces no journal, persisted intent, new checksum domain, automatic migration
or inference that the greatest observed generation is authoritative. The ordinary
`open_writer`, `inspect` and `reconcile` operations continue to require a clean,
complete selected layout; they never invoke recovery implicitly.

## 1. Independent authority and retained histories

A recovery caller supplies all of the following:

- The independently selected network configuration hash and exact genesis.
- Independently retained complete old and candidate checkpoints, including each
  local generation, height, tip and combined ledger root.
- Both complete accepted `ChainV2` histories, with their own exact manifests,
  blocks, accounts, membership records, permanent reservations, approvals,
  sequences, population history and resource counters.
- The explicit action `Complete` or `RetainOld`.

The candidate must be strictly preferred under the existing fixed-target
[development fork-choice rule](FORK_CHOICE_V2.md). Its local generation must be
exactly old generation plus one; each generation remains bounded by its selected
height. Both supplied histories must match the independently retained checkpoints
and network. Exact canonical metadata is derived from the accepted histories,
including manifest checksums, lengths and total block bytes. A byte-correct public
checksum cannot authorize inconsistent history metadata.

The request borrows both accepted histories. It is not serialized into the store,
and checkpoints alone cannot replace either history. In particular, after some
old files have been removed, the old endpoint cannot authenticate the contents of
the surviving files. If the caller has not retained the old accepted object, it
must independently reconstruct it against its retained pins before this API can
authorize cleanup. Missing or corrupt residue is not that reconstruction source.

The relevant lifecycle is:

```rust,ignore
let request = ReplacementRecoveryRequestV2::new(
    old_checkpoint,
    candidate_checkpoint,
    &accepted_old,
    &accepted_candidate,
)?;

ReplacementStoreV2::recover_interrupted(root, identity, &request, action)?;
store.recover(&request, action)?;
```

The first entry point acquires a new exclusive root lock. The second consumes a
writer, including a fenced writer, without releasing and reacquiring its lock.
Its last-confirmed memory checkpoint must match one of the supplied endpoints.
An inspection handle cannot recover. Success returns a fully replayed,
synchronized, unfenced writer; any error returns no handle. The borrowed histories
remain available to the caller after either result.

## 2. Ownership and filesystem assumptions

Recovery holds the same cooperative Linux kernel lock on the stable root directory
inode throughout validation, modification, final replay and the returned writer's
lifetime. It never renames, removes or replaces that root, exposes its descriptor,
uses a PID as ownership evidence, or scans for a replacement lock owner.

The local owner and parent directory tree are trusted. All processes must respect
the lock. Static type, symlink, hardlink and observed file-identity checks provide
defense in depth; they do not make pathname traversal safe against a hostile
same-user process racing every operation. Correct same-root rename and file and
directory synchronization remain filesystem assumptions. Fault injection does not
certify physical power-loss behavior or establish WSL/DrvFS, native Windows or
network-filesystem support. The existing inherited-descriptor lock lifetime rule
continues to apply.

## 3. Finite recognized layouts

Only four root names are potentially authorized: `selection.v2`, optional
`selection.pending`, and the two exact independently pinned
`generation-{generation:020}` directories. Each generation contains at most
`manifest.v2` and `blocks/`; every block name is an exact selected 20-digit height
plus `.mrb2`. An unknown path is rejected, not traversed or deleted.

`selection.v2` must equal the entire canonical old or candidate MSR2. Missing,
partial, corrupt, foreign or inconsistent selected metadata rejects unchanged.
Neither pending bytes nor a generation name can substitute for this marker.

| Selected marker | Old generation | Candidate generation | Pending marker | Allowed action |
|---|---|---|---|---|
| Exact old | Complete accepted old history | Absent | Absent | Complete or RetainOld |
| Exact old | Complete accepted old history | Authorized partial staging | Absent | Complete or RetainOld |
| Exact old | Complete accepted old history | Complete exact candidate | Absent or canonical candidate-selector prefix | Complete or RetainOld |
| Exact candidate | Authorized old cleanup subset, or absent | Complete accepted candidate | Absent | Complete only |
| Exact candidate | Absent | Complete accepted candidate | Absent | Complete is idempotent |

Every other combination rejects and preserves the observed files. In particular,
a pending selector with an incomplete candidate is unsupported, and selected
candidate history is never repaired from partial staging. Interrupted initialization
has no valid old selected endpoint and remains outside this recovery API.

### Candidate staging while old remains selected

The candidate generation, its prescribed subdirectory and individual files may be
absent. Any existing file must be a byte prefix of its exact canonical encoding;
the empty prefix and the full encoding are allowed. A mismatched byte, overlong
file, unexpected height or noncanonical name rejects before modification.

Surviving block files may form an arbitrary subset of authorized heights. A file
sync alone does not make every prior directory entry durable, so recovery does not
assume that surviving files form an increasing-height prefix. The candidate
manifest may also be absent while some named block files survive. Each surviving
file is checked independently against the accepted candidate.

An existing `selection.pending`, even an empty one, requires the complete exact
candidate generation. Its bytes must be a canonical candidate MSR2 prefix. This
preserves the prerequisite that selector preparation follows complete candidate
staging and synchronization.

Prefix equality constrains authorized contents; it does not establish who created
an empty or common prefix. The independently retained request and explicit action
authorize work on these exact paths.

### Old cleanup after candidate selection

Surviving old files must match their complete canonical bytes. Partial old files
are never an authorized deletion residue. Surviving block heights may be any
subset of the old history, including none, because several unlinks may precede
the block-directory sync.

The cleanup order imposes additional constraints:

1. If old `blocks/` exists, the complete exact old manifest must also exist. The
   directory may contain any subset of complete old blocks, including zero.
2. With old `blocks/` absent, the old generation may contain only the complete
   manifest, or be empty.
3. The old generation may be entirely absent.

The manifest cannot disappear while `blocks/` survives: the implemented retirement
order synchronizes removal of `blocks/` in the generation directory before removing
the manifest. Recovery rejects that contradictory shape. It never recreates old
files to make cleanup possible and never rolls selected candidate state back to
old through `RetainOld`.

## 4. Validation before modification

Before the first filesystem mutation, recovery validates the request, independent
network and checkpoint pins, strict preference and generation transition, bounded
names and lengths, canonical selected marker, and every surviving residual byte.
It fully replays the selected complete history through `ChainV2` and compares the
entire accepted result with the corresponding supplied reference. The checks cover
both ledger components and the complete resource accounting.

An invalid later residual file must prevent deletion of an earlier valid file.
An unknown artifact must not be removed to make validation succeed. Layout and
selected-marker observations are rechecked at operation boundaries, and named
files are compared again before extension or removal. These checks share the
single per-call payload-read allowance.

## 5. Completing a replacement

When old remains selected, `Complete` creates only missing authorized directories
and files and extends only verified candidate byte prefixes. It never truncates
an existing file or overwrites a mismatching prefix. The complete candidate's
files, block directory, generation directory and stable root are synchronized.
The completed history is fully replayed and compared with the exact accepted
candidate before selector publication.

Recovery completes or creates the canonical pending selector, synchronizes it,
rechecks the old selection and prepared candidate, then renames the pending marker
over `selection.v2` and synchronizes the stable root. A failure after a publication
attempt may leave the candidate selected; it must not be reported as proof that
old remains selected.

Whether the call published the candidate or found it already selected, recovery
reestablishes the durability of the complete selected candidate and its marker
before deleting any old file. A previous attempt may have stopped immediately
after rename and before root sync; observed selection alone does not establish
that synchronization succeeded.

Only prevalidated complete old files are retired, with an exact comparison before
each unlink. The old block directory is synchronized and removed only when empty;
its removal is synchronized in the old generation before manifest removal. The
remaining manifest is checked and removed, the generation synchronized and removed
only when empty, and the stable root synchronized. There is no recursive deletion,
wildcard deletion or automatic old-branch archive.

If the old blocks directory is already absent on entry, recovery still synchronizes
the old generation **before removing its manifest**. The observed absence could
result from an earlier unlink whose generation-directory sync failed. Skipping
this sync on retry could let manifest removal persist before blocks-directory
removal, leaving an unsupported state after a second interruption.

`Complete` on a clean already-selected candidate is idempotent: it still fully
validates and synchronizes the selected history and returns that same checkpoint.
It neither increments generation again nor mints a new fork-choice score.

## 6. Retaining the still-selected old history

`RetainOld` is permitted only while the exact old marker remains selected. The
complete old history is replayed and its durability reestablished before candidate
staging is discarded.

If a pending selector exists, recovery compares and removes it, then synchronizes
the stable root **before removing any candidate file**. This ordering makes an
interruption during retention recognizable on retry: a pending selector must never
remain alongside a candidate made incomplete by this operation.

Only the authorized candidate staging files are removed. They may be incomplete
canonical prefixes, so their observed authorized bytes are checked before removal.
Named directories are synchronized and removed only when empty. The root is
synchronized after the candidate generation disappears. The old marker is never
rewritten, and the generation remains the old generation.

Retaining a clean old state is idempotent. Repeating `Complete` after an interrupted
retention may restage the independently authorized candidate; repeating `RetainOld`
after candidate selection rejects. No automatic action is chosen from a filename
or error message.

## 7. Final acceptance and failures

Both actions finish with the existing strict clean-layout loader, complete actual
chain replay against the chosen independent pins, synchronization and exact selected
file checks under uninterrupted ownership. Only then does the API return a writer
with the complete replayed chain, selector and generation together.

Any failure returns no handle and preserves the remaining state for inspection or
another explicit bounded recovery call. There is no automatic retry loop, implicit
rollback or cleanup of unrecognized bytes. A later attempt revalidates the entire
request and observed residual layout; a previously successful preflight is not
permission to skip current checks. Normal reopening and reconciliation remain
strict even after this explicit recovery API is available.

## 8. Separate finite resource bounds

Each supplied or replayed history retains the existing independent limits:
**4096 blocks**, **16,777,216 MRB2 bytes**, **1,048,576 cumulative state/configuration
items**, and **2,781,636 MNM2 manifest bytes**. A short prefix does not permit an
oversized complete candidate. Per-history limits are never doubled to admit a
single larger history.

The two prescribed generations and at most two selectors retain the conservative
**39,118,264-byte distinct encoded-payload ceiling**. All repeated bounded payload
reads, prefix comparisons, exact comparisons and final replay reads share the
unchanged **313,011,680-byte per-call read allowance**. A permitted length plus one
growth sentinel is charged before the payload read. Exhaustion rejects; the meter
is not reset between recovery phases.

Let `H = 2,781,636 + 16,777,216 + 4,097 = 19,562,949` be a conservative
whole-generation payload pass, including one growth sentinel for the manifest and
each of 4096 blocks. Let `S = 280 + 1 = 281` be one MSR2 read, including its
sentinel. Partial residual reads are conservatively charged as full passes in the
following successful-call bounds:

| Entry state and action | Full actual-chain replays | Maximum charged payload reads | Conservative byte bound |
|---|---|---|---|
| Old selected, Complete | 4 | `15H + 12S` | 293,447,607 |
| Candidate selected, Complete | 2 | `9H + 6S` | 176,068,227 |
| Old selected, RetainOld with pending marker | 2 | `11H + 10S` | 215,195,249 |

The old-selected completion path replays the initial old history, completed staged
candidate, selected candidate after publication, and final clean candidate. It
also compares all initial residue, both branches before publication and cleanup,
every opened prefix and completed file, each retired file, and the final selected
bytes. Already-selected completion omits staging and publication. Retention adds
an all-residue comparison before pending removal, then rechecks all residue after
that removal becomes durable; its initial and final replays both execute old.
Without a pending marker, retention needs no more than `9H + 6S`.

These finite path bounds fit within the unchanged enforced read allowance. Reads
through a prefix's checked read/append descriptor are charged to that same meter;
they are not an unmetered alternate path. There is no complete payload rescan
inside a per-block write or removal loop. Additional metadata checks and directory
synchronization are separate operations, with no payload reads in these counts.

Root enumeration is bounded at four entries, each generation at two entries, and
each block directory at its independently accepted height count, at most 4096.
Unexpected entries reject without an unbounded traversal. The maximum recognized
two-generation shape contains 8200 prescribed entries across these directories;
repeated bounded scans are separate from the payload-read meter.

These are encoded-input, read and execution bounds, not total resident-memory,
metadata-system-call, write-volume, elapsed-time or physical-durability guarantees.
The caller may hold both accepted histories alongside bounded encoded buffers and
replayed state. Verification records distinguish tests and examples from those
filesystem assumptions.

## 9. Acceptance scope

Acceptance covers completion, retention, idempotence and retry at every implemented
failure boundary; arbitrary surviving candidate and old subsets; incomplete
selectors; unknown/corrupt contents; independent pins and histories; genuine
membership forks; ordinary-open strictness; resource exhaustion; and actual
independent-process ownership. Exact evidence and any remaining unsupported cases
belong in the checkpoint files rather than inferred from this contract.

This operation recovers one independently authorized local branch replacement.
Next, **MB-03F** covers bounded fresh-peer candidate import and reconstruction under
independently selected network identity. Persistent V2 node/operator integration
remains **MB-04**. Device capture,
private biometric matching, monetary issuance/rewards and final human consensus
remain separate work. No founder or marketing privilege is introduced.
