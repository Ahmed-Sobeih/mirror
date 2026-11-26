# V2 replacement store: durable publication and strict reconciliation

**MB-03E2a. Updated 2026-11-26.** This contract extends the separate profile-3
[MSR2 metadata and preparation contract](STORAGE_REPLACEMENT_V2.md) with a bounded
filesystem store. It defines initialization, independently pinned reopening,
whole-branch replacement, failure fencing and clean-state reconciliation. Explicit
repair of interrupted staging or cleanup is **MB-03E2b, not implemented by E2a**.
Current implementation and verification evidence belongs to
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).

No existing MSS2/MSU2 encoding, immutable/append store API, old recovery rule,
network manifest, block encoding or reference fixture is repurposed. There is no
implicit migration, persisted intent or new checksum domain in this increment.

## 1. Authority and ownership

The caller independently selects the network configuration hash, exact genesis,
full current chain checkpoint and local generation. Reopening never learns these
expectations from a file or peer. A complete accepted candidate is checked against
the same network identity and existing strict fixed-target branch preference.
The [pure preparation API](STORAGE_REPLACEMENT_V2.md#3-checked-in-memory-preparation)
is reused under the store's ownership; a previously copied plan is not permission
to publish after the selected disk state has changed.

`ReplacementStoreV2` holds the same cooperative Linux root-directory inode lock
throughout its lifetime. Writers are exclusive; inspection handles are shared and
cannot replace or reconcile. Reconciliation consumes the writer and retains its
existing descriptor continuously. There is no lock release/reacquire window,
lock-file lease, PID takeover, exposed descriptor or explicit unlock shortcut.

The root directory is never renamed, removed or replaced by this API. Its local
owner and parent tree remain trusted. Static symlink, file-type, hardlink and
observed inode checks are defense in depth; this is not protection against a
hostile same-user process racing every path operation. Correct kernel locking,
same-root rename and file/directory synchronization are explicit filesystem
assumptions. Tests do not establish WSL/DrvFS, native Windows or network-filesystem
support or certify physical power-loss behavior. Inherited fork descriptors retain
the existing last-reference lock lifetime rule.

## 2. Bounded layout

The clean root contains exactly:

- `selection.v2`: the unchanged **280-byte MSR2** record.
- `generation-{generation:020}`: the exact independently selected generation
  directory, with 20 decimal digits, including leading zeroes.

That generation directory contains exactly `manifest.v2` and `blocks/`. The block
directory contains precisely the selected complete history, using the existing
20-digit height plus `.mrb2` filename convention. Manifest and block bytes are the
candidate's exact existing MNM2/MRB2 encodings. A generation path is derived from
the independently pinned checkpoint, not from choosing the greatest directory
name. Generation is local publication bookkeeping, not fork score or freshness.

Initialization starts at generation zero at any accepted bounded chain height.
Replacement by a strictly preferred complete candidate advances generation once,
even if the candidate skips several heights. E1's `generation <= height <= 4095`
invariant remains unchanged. The clean layout does not retain all old generation
directories: at most the current generation `g` and the candidate `g + 1` coexist
while a replacement is being prepared or its old data is being retired.

During one operation only, the root may also contain `selection.pending`. Thus the
maximum recognized root width is four entries: two generation directories and two
selectors. Each generation has at most the two prescribed entries; each block
directory has at most 4096 entries. No arbitrary nesting, aliases, unselected
generation scan, extra block height, backup directory or unknown file is ignored.

**Ordinary open, inspect and reconcile accept only the clean layout.** Any pending
selector, second generation, incomplete directory or unexplained entry is rejected
and preserved. None of these operations deletes data, resumes a write, chooses a
newer generation or fills a hole in selected history. Missing/corrupt selected
markers and interrupted initialization are not repaired by E2a.

## 3. Initialization and reopening

The public lifecycle is:

```rust,ignore
ReplacementStoreV2::initialize(root, &accepted_chain)
ReplacementStoreV2::open_writer(root, identity, expected_checkpoint)
ReplacementStoreV2::inspect(root, identity, expected_checkpoint)
store.replace(identity, expected_current_checkpoint, &accepted_candidate)
store.reconcile(expected_checkpoint)
```

`replace` borrows the candidate, so the caller retains it after either success or
failure. `reconcile` consumes the handle. Read-only accessors expose the root,
access mode, fencing flag, and last-confirmed checkpoint; ordinary chain, selector
and checkpoint access is fallible while fenced. No mutable chain or selector is
exposed. A caller preparing for later explicit recovery must also retain or be
able to independently reconstruct the complete old history; the last-confirmed
checkpoint alone does not contain it.

Explicit initialization operates only in an absent or empty final directory whose
parent already exists. It derives canonical MSR2 generation zero and encodes the
bounded accepted history before filesystem changes. It acquires the stable root
lock, persists the root's parent entry, creates the generation and its complete
manifest/block history, and synchronizes every file, `blocks/`, the generation
directory and the root before publishing the selection. The staged selector is
written and synchronized, renamed to `selection.v2`, and the root synchronized
again before a writable handle is returned. Existing or partial stores are not
overwritten. Failure returns no handle and preserves observed files.

Opening or inspection checks independent network/endpoint/generation pins, strict
layout and all lengths before unbounded payload reads are possible. It decodes
the exact pinned MSR2, verifies its exact manifest checksum/length, validates the
MNM2 manifest and every seed proof, and replays every actual MRB2 block through
`ChainV2` against the complete independently retained checkpoint. The derived
canonical selector must equal the entire disk record. A valid public checksum or
matching endpoint alone cannot hide different manifest metadata or byte totals.

Reconstructed accounts, membership records, permanent reservations, retained
approvals, sequences, population history and resource counters all come from full
execution of the selected history. A cached root or account balance is never an
alternate source of runtime authority. Writer reopening additionally synchronizes
the verified selected files and directories and rechecks root/selector identity
before returning. Shared inspection performs no application writes or repair;
operating-system access-time updates are outside that guarantee.

## 4. Whole-branch replacement ordering

The operation requires a writable, unfenced handle, an independently retained
current checkpoint and an accepted complete candidate. The sequence is:

1. Check the retained current expectation, independently selected network and
   existing actual-chain branch comparison. Derive exact current/candidate MSR2
   metadata and the full candidate manifest/block encodings. Candidate validation
   errors before disk revalidation leave the current handle usable.
2. Fence ordinary state access and writes. Verify the held root inode and fully
   replay the strict selected on-disk history under the retained current pins.
   Compare its complete chain and selector with the current in-memory selection.
   A discrepancy rejects without attempting publication.
3. Create only the absent canonical `g + 1` directory and stage the candidate's
   exact whole manifest and every block. Use exclusive file creation. Synchronize
   files, the candidate blocks directory, candidate generation directory and root.
   The original selected generation remains complete and unchanged.
4. Write and synchronize the canonical candidate `selection.pending`. Recheck the
   root, old selector and prepared candidate bytes. Mark publication as attempted
   before renaming the pending selector over `selection.v2`.
5. Synchronize the stable root after the rename. This is the required durable
   selection boundary; the rename itself is the visible marker switch. Memory has
   not advanced yet, and the old complete history still exists.
6. Verify the selected complete candidate and preflight/compare the entire exact
   old generation against the accepted old history before deleting any old file.
   Remove only its canonical, byte-matching block files, with identity/content
   checks at removal. Synchronize the old blocks directory, remove that now-empty
   directory, and synchronize the old generation directory.
7. Recheck and remove only the exact old manifest, then synchronize the old
   generation directory. Remove the now-empty old generation directory and
   synchronize the stable root. Never use recursive deletion or remove a differing
   file merely because it occupies an expected pathname.
8. Complete required clean-layout verification and synchronization. Advance the
   in-memory chain, selector and generation together only after all fallible work
   succeeds. The accepted candidate's full state replaces the old full state; no
   accounts, membership history or resource counters are grafted across branches.

The old history is retired only after the complete candidate is durably selected.
This is removal of an unselected branch copy, not pruning any part of the complete
selected chain or recovering capacity by evicting selected membership history.
There is no automatic old-branch archive and no rollback promise after publication.
Callers that need a branch archive must retain it independently outside this layout.

## 5. Failure fencing and strict reconciliation

A disk discrepancy or mutation failure keeps the handle fenced. Normal chain,
selection and checkpoint access, and additional replacements, reject. The explicitly
named last-confirmed checkpoint reports only the last successfully published memory
state. It does not assert that disk still selects that history after a failed call.

Errors before selection publication is attempted report an incomplete update.
After a rename may have been attempted, including cleanup/synchronization failures,
the result is uncertain with respect to successful completion: disk may already
select the candidate, while memory still retains the old confirmed state. Do not
treat an error as proof that the current selector or balances stayed old.

Consuming reconciliation retains the same exclusive root lock, uses independent
network/current expectations, and requires a clean complete selected layout. It
fully replays and synchronizes that selected history before returning an unfenced
writer. It can reconcile a completed publication whose final call failed; it
cannot repair staged or partially cleaned layouts. Any error returns no handle.
Ordinary reopening follows the same clean-only restriction. Unknown, corrupt or
unsupported artifacts are preserved, not cleaned up to obtain a successful result.

## 6. Interrupted states and the next explicit recovery increment

The following table freezes the boundary between E2a and **planned E2b**. E2a
rejects every non-clean row during ordinary opening/reconciliation. It implements
no `Complete` or `RetainOld` recovery entry point. A future recovery operation must
validate all independently authorized input and residual bytes before mutation.

| Selected marker | Candidate generation | Old generation | Required future treatment |
|---|---|---|---|
| Exact old MSR2 | Absent | Exact complete old history | Clean old state; an explicit retained request may begin/resume replacement |
| Exact old MSR2 | Partial authorized staging; no pending selector | Exact complete old history | Explicit completion or removal of only authorized candidate staging |
| Exact old MSR2 | Exact complete candidate, pending absent or canonical byte prefix | Exact complete old history | Explicit completion or retention of still-selected old history |
| Exact new MSR2 | Exact complete candidate; no pending selector | Exact complete old history or an authorized deletion subset | Explicit completion of old-copy cleanup; never retain/roll back to old |
| Exact new MSR2 | Exact complete candidate; no pending selector | Absent | Clean new state; explicit completion is idempotent |
| Exact old MSR2 | Incomplete candidate with any pending selector | Any | Reject and preserve: selector preparation requires complete candidate prerequisites |
| Exact new MSR2 | Missing/incomplete candidate or any pending selector | Any | Reject and preserve: selected history is never repaired from staging |
| Missing, partial, corrupt, foreign or unpinned marker | Any | Any | Reject and preserve; no greatest-generation fallback |

“Authorized staging” means only the exact named candidate paths and byte prefixes
of their independently supplied canonical encodings; no mismatched byte, oversized
file or unknown name is authorized. Surviving staged files need not form a prefix
of block heights: syncing an individual file does not by itself establish durability
of every directory entry before the directory synchronization completes. Each
surviving file must be checked independently. A pending selector is permitted only
when the entire exact candidate is already complete.

An authorized deletion subset may contain any subset of the exact complete old
block files, including empty/missing prescribed directories as permitted by the
cleanup ordering. A process interruption can leave an ordinal deletion prefix;
physical persistence before a directory sync need not do so. Retained files must
still match their full canonical bytes; partial or differing old files are not
authorized by a cleanup request. Both cases remain bounded by the two exact
generation names and independent histories. No observed filename or prefix proves
who created an artifact.

**E2b requires both complete accepted old and candidate histories**, as well as
independently retained network, full checkpoints and generations, and an explicit
completion/retention action. An old checkpoint plus candidate alone cannot
authenticate the contents of an old branch after some old files have been deleted.
The exact old history can instead be independently reconstructed against its
retained pins before recovery; it cannot be inferred from missing/corrupt residue.
The selected old history must replay fully from disk before prepublication recovery.
After the new marker is selected, only exact complete selected-candidate replay and
strict comparison against the independently retained old reference can authorize
cleanup. A selected new generation must never be rolled back by `RetainOld`.

These retained references are caller authority, not a new persisted journal. Safe
recovery must keep the same exclusive root lock, recheck artifacts before mutation,
return no writer on unresolved failure and make allowed repeated actions explicit.
No recovery promise or operator interface exists merely because this state table
has been written. Interrupted initialization remains a separate unsupported case.

## 7. Separate resource bounds

Every accepted history retains existing `ChainV2` limits: at most **4096 blocks**,
**16,777,216 MRB2 bytes**, **1,048,576 cumulative state/configuration items**, and
at most **2,781,636 MNM2 manifest bytes**. Complete candidate budgets are checked
before staging; a short residual prefix does not grant capacity for a larger history.

One full MSR2-selected encoded payload is at most **19,559,132 bytes**. At most two
full manifests/histories and two selectors coexist, giving a conservative distinct
encoded payload bound of **39,118,264 bytes**. No third generation, additional
selector or recovery journal is included or permitted by this profile.

A separate conservative per-call payload-read allowance is:

```text
MAX_REPLACEMENT_READ_BYTES_V2
    = 8 * (39,118,264 + 8,192 + 4)
    = 313,011,680 bytes
```

Every bounded payload read is charged its checked permitted length plus one growth
sentinel before reading. The allowance includes repeated selector comparisons,
full replay, candidate verification and cleanup comparisons. It is not reset within
a call or an unbounded retry loop. Exhaustion rejects. A conservatively prepaid
bounded replay is allowed only if its maximum read/sentinel charge is explicit.

The implemented successful-call control flow has these separate upper bounds.
Let `P = 2,781,636 + 16,777,216 = 19,558,852` be one history's encoded payload,
and `F = 4096 + 1 = 4097` its maximum manifest/block file count. A whole-generation
payload pass charges at most `P + F`; an MSR2 read charges at most `280 + 1`.

| Successful operation | Full actual-chain replays | Complete payload passes / selector reads | Conservative charged read upper bound |
|---|---|---|---|
| Initialization | 2 | 4 passes / 4 selectors | 78,252,920 bytes |
| Shared inspection | 1 | 1 pass / 1 selector | 19,563,230 bytes |
| Writer reopening or clean reconciliation | 1 | 2 passes / 3 selectors | 39,126,741 bytes |
| Whole-branch replacement | 5 | 9 passes / 7 selectors | 176,068,508 bytes |

The five replacement replays validate current disk history, the staged candidate,
the candidate after publication, the old history before cleanup, and the final
clean selection. Additional exact byte passes check the prepared candidate before
publication, all old files before deletion, each old file at deletion and the final
selected files after synchronization. All share one per-call read meter. The
accepted-chain replay budget is enforced on each execution; this finite control
flow does not repeatedly replay until a condition happens to pass.

Root/generation/block directory enumeration has its own finite entry limits.
Metadata preflight checks each canonical filename, regular-file identity, individual
length and aggregate complete MRB2 bytes before block payload allocation. Reads
use checked length-plus-one sentinels; no new unbounded `fs::read` path is allowed.
Existing actual replay/work budgets apply independently to each complete execution.
One two-generation layout contains at most 8200 prescribed entries across the
root, generation directories and blocks directories (4 + 4 + 8192). A pass rejects
on an additional entry rather than traversing an unbounded tree. Repeated bounded
preflight passes are separate from the payload-read allowance; 8200 is not a claim
about the total number of metadata operations during a replacement.

These bounds are not resident-memory limits, total write-volume measurements,
metadata syscall budgets, elapsed-time guarantees or physical power-loss evidence.
The operation may hold both accepted chains, replayed state and bounded encoded
buffers concurrently. Existing per-history state limits are not doubled inside a
single chain to make replacement succeed.

## 8. Acceptance scope

Verification must cover real divergent histories, including equal account bytes
with different membership/reservations/population history; independent wrong/stale
pins; byte-correct selectors with inconsistent metadata; all initialization,
publication, cleanup and synchronization fault boundaries; fenced access; strict
reopen/reconcile behavior; unsafe/unknown paths; bounded reads; unchanged old
profiles; and actual concurrent-process ownership. Exact source/CI counts and any
unimplemented recovery coverage must be recorded separately from this contract.

E2a advances durable local branch replacement. It does not finish MB-03, supply
fresh-peer selection, integrate a persistent V2 node, settle human consensus or
implement devices, private matching, monetary issuance or rewards. No founder or
marketing privilege is introduced. Gate A and Gate B remain unmet.
