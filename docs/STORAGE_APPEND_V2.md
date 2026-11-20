# V2 accepted-block append and checked reconciliation

**MB-03B. Updated 2026-11-20.** This document specifies the separately versioned
append-only storage profile implemented by `mirror_storage::v2::AppendStoreV2`.
The immutable `StoreV2` and MSS2 profile 1 contract in [STORAGE_V2.md](STORAGE_V2.md)
remain unchanged. [CHAIN_V2.md](CHAIN_V2.md) remains the authority for actual block,
ledger and membership validity. Current verification and publication evidence
belongs to CURRENT_STATUS.md and MASTER_PLAN.md, not to this format specification.

The scope is explicit initialization, one accepted child per append, checked
reopening, read-only inspection and reconciliation of a complete selected store.
It does not replace branches, repair residual staging, choose global finality,
automatically adopt an immutable store, or launch a V2 network daemon. A failed
append may leave a store that this slice deliberately refuses to reopen. Its files
are preserved for a later explicitly specified repair operation.

## 1. Selection authority and compatibility

Three facts have separate meanings:

1. `ManifestIdentityV2` independently selects configuration `C` and the exact
   genesis hash. It is not learned from an arbitrary manifest or peer.
2. `ChainReplayExpectationV2` independently selects the full finite branch by
   `C`, genesis, height, tip and ledger root. Complete replay must reproduce it.
3. `AppendCheckpointV2` additionally selects the local storage generation. A
   generation identifies local one-block advances since this store's initialization.
   It does not count humans, rewards, confirmations or global consensus rounds.

The external checkpoint has the exact public shape:

```rust,ignore
AppendCheckpointV2 {
    generation: u64,
    chain: ChainReplayExpectationV2,
}
```

The full expected append checkpoint must come from the caller's retained selection.
Reading it out of the selector currently being checked defeats this protection.
The record checksum detects inconsistent bytes; it does not authenticate a branch,
establish latestness or make an unknown generation authoritative. A stale checkpoint
rejects against a later selected marker. If the caller deliberately supplies old
pins and an entirely rolled-back matching store, this API cannot discover global
freshness or external history.

Initialization starts at generation zero from any already accepted finite
`ChainV2`, including a history with nonzero height. Each successful one-block append
increments both generation and chain height by one. Therefore a valid generation
never exceeds the selected height. The difference between height and generation
is the initialization height for that particular store. Existing finite chain caps
remain in force; generation is not permission to continue beyond them.
An uncertain append can have published the next marker despite returning an error;
only checked reconciliation can establish which independently selected generation
is visible and prepare it for further writes.

MSU2 is a distinct storage envelope. It embeds the exact existing immutable MSS2
record, including the inner profile 1 and inner generation zero, and carries its
own append profile and generation. The old MSS2 parser does not accept MSU2, and
the new append parser does not reinterpret a bare MSS2 record. No network,
configuration, signed operation, MRB2 block, ledger or membership encoding changes.

There is no in-place migration or automatic adoption. An explicitly selected,
validated chain can initialize a separate absent or empty append-store directory.
Doing so begins a new local generation sequence; it does not establish that this
directory supersedes another store. Existing source directories remain intact.

## 2. Public lifecycle and ownership

The public operations are:

```rust,ignore
AppendStoreV2::initialize(path, &accepted_chain)
AppendStoreV2::open_writer(path, identity, expected_append_checkpoint)
AppendStoreV2::inspect(path, identity, expected_append_checkpoint)
store.append(complete_child_block)
store.reconcile(expected_append_checkpoint)
```

Initialization accepts only an absent final directory with an existing parent,
or an explicitly empty directory. It never creates missing parents, chooses a
genesis, mines, imports an untrusted state cache or overwrites a populated store.

`open_writer` retains exclusive ownership. After complete validation it synchronizes
the selected files and containing directories before permitting appends. `inspect`
retains shared ownership and performs no application-level writes, repairs,
creation or synchronization. OS access-time effects are outside this no-write
description. Multiple inspectors may coexist and exclude a writer. An exclusive
writer excludes every other writer and inspector.

`append(&mut self, BlockV2)` returns the new `AppendCheckpointV2` only after durable
publication succeeds under the filesystem assumptions below. The caller must keep
that checkpoint if it wants to select precisely that generation on a later open.

`reconcile(self, AppendCheckpointV2)` consumes the handle, keeps its existing
exclusive directory lock throughout reconstruction and synchronization, and
returns a fresh usable handle only on success. It can choose an independently
retained complete old or candidate selection; it does not promote staging or roll
back the selected marker. On failure it returns no handle and preserves files.
A read-only handle cannot append or reconcile as a writer.

The immutable getters `chain()`, `selection()` and `checkpoint()` return errors
while the handle is fenced. `root()`, `access()` and `is_fenced()` remain available.
`last_confirmed_checkpoint()` exposes only the last successfully published
in-memory checkpoint. Following an uncertain append, it is not a claim about which
marker is currently selected on disk. There is no mutable chain accessor, raw
descriptor accessor, clone of the store handle, lock upgrade, forced takeover or
external state setter.

The implementation retains the same cooperative Linux root-directory inode lock
as profile 1. The locked root must never be renamed, replaced or removed. Parent
directories and local filesystem owners are trusted. Every process must honor the
advisory lock. Closing the handle closes its descriptor; actual kernel ownership
lasts until all descriptors for the same open file description have closed,
including descriptors temporarily inherited by a concurrent fork before exec.
There is no PID-file authority or stale-lock deletion.

Linux is the implemented operating-system boundary. It is not a filesystem
certification: correct directory locking, atomic same-directory rename and file/
directory synchronization are explicit requirements. The prior local checks used
tmpfs and workspace overlayfs; Ubuntu CI evidence is recorded per checkpoint.
WSL2 native Linux storage is intended but not directly exercised here. Native
Windows, DrvFS, NFS/SMB and other translated/network mounts remain unverified.
Static symlink, file-type, link-count and device/inode checks are defense in depth,
not a sandbox against hostile same-user namespace races.

## 3. Selected layout and bounded inputs

A clean selected root has exactly these three entries:

- `manifest.v2`: exact public MNM2 bytes with all seed proofs.
- `blocks/`: exactly the selected contiguous complete MRB2 files from height zero.
- `selection.v2`: the exact MSU2 append selection envelope.

Block names retain the 20-digit height plus `.mrb2` convention. There is no account,
membership, population, approval or resource-counter cache with independent runtime
authority. All of those values must be reproduced by the full accepted history.

`selection.pending` is reserved for staging a new marker. A newly written next-height
block can also remain unselected if an append fails before publication. Neither
artifact authorizes a branch. Normal open, inspection and reconciliation require
the strict clean layout: an extra tail block, pending marker, missing entry or
unrecognized file rejects and remains untouched. No directory scan selects the
largest height or generation.

### Exact MSU2 envelope

MSU2 is exactly **280 bytes**. All unsigned integers are little-endian; hashes
retain their ordinary raw byte order.

| Offset | Field | Size / required value |
|---|---|---|
| 0 | Magic | 4 bytes `MSU2` |
| 4 | Encoding version | u16 = 2 |
| 6 | Storage profile | u16 = 2 |
| 8 | Append generation | u64 |
| 16 | Exact nested MSS2 snapshot | 232 bytes, unchanged version 2/profile 1/generation 0 |
| 248 | Envelope checksum | `SHA256d("MIRROR_STORAGE_APPEND_SELECTION_V2" || bytes[0..248])`, 32 bytes |

The inner MSS2 carries the selected full chain checkpoint, block count, exact
manifest length, complete block-byte total and manifest/record checksums defined
in [STORAGE_V2.md](STORAGE_V2.md). Its generation remains zero because it describes
one immutable selected snapshot; the outer generation describes local append
history. No old byte range or hash domain is silently repurposed.

Decoder rejection order is exact length, magic, version, profile, outer checksum,
the complete nested MSS2 checks against `expected.chain`, generation not exceeding
selected height, and exact generation equality to `expected.generation`. An outer
checksum-correct envelope cannot bypass inner bounds or external branch selection.
The selection API has no unpinned decoder. A decoded selection is bounded metadata,
not an accepted chain; the full manifest and block replay remain mandatory.

### Input bounds

Root and block enumeration are bounded. Every selected file must be a regular,
non-symlink file with one link. All block counts and lengths are checked against
the selected metadata and configured limits before payload allocation. Reads
retain the checked-length-plus-one sentinel and opened identity checks from the
immutable profile. Length changes, excess entries and aggregate excess reject.

The complete selected history still permits at most 4,096 blocks including genesis,
16,777,216 bytes of complete MRB2, and 1,048,576 cumulative abstract state/configuration
items as defined by ChainV2. The manifest maximum remains 2,781,636 bytes. The MSU2
marker is 280 bytes, giving a maximum successful selected encoded payload of
19,559,132 bytes. This is not a total resident-memory ceiling: decoded structures,
staged execution, buffers and their separate limits also apply.

An in-progress valid append can temporarily retain both the old selected marker
and the new 280-byte pending marker. The new block is already included in the
candidate history's unchanged 16-MiB limit. These temporary bytes are not permission
to accept arbitrary tail files: strict reopening rejects residual staging before
reading it as selected payload.

All complete block bytes enter `ChainV2::replay_encoded`. Genesis, context, ancestry,
literal target, work, parent time, original mixed operation order, signatures,
accounts, membership and resulting roots must validate. Disk counters never replace
reexecution. History is never pruned or discarded to recover budget.

## 4. Initial publication

Initialization uses the new MSU2 marker at generation zero while preserving the
prior preparation-before-selection ordering:

1. Encode the already accepted chain and marker before filesystem mutation.
2. Validate/create the final root only, acquire exclusive ownership, and require
   an empty directory. Synchronize its existing parent directory.
3. Create `blocks/`; create the manifest and each block with `create_new`, fully
   write each file, and synchronize each file.
4. Synchronize `blocks/`, then the stable root directory.
5. Create, fully write and synchronize `selection.pending`.
6. Attempt the same-directory rename to `selection.v2`.
7. Synchronize the root again before returning an owning handle.

A failure before selection publication is attempted yields
`InitializationIncomplete`; after publication is attempted it yields
`InitializationUncertain`. No store handle is returned. Files are retained and
later checked opens still require an independent append checkpoint. Root creation,
ownership and initial layout checks may fail directly before preparation begins.

## 5. One-block append publication

An append is a complete accepted child transition, never a metadata-only update.

1. Require writer ownership and an unfenced handle. Clone the private accepted
   ChainV2 and run its normal full `append` validation on the supplied block.
   Rejection here leaves the original chain and files unchanged and the handle
   remains usable. Check generation advancement and prepare exact encoded bytes.
2. Reopen and fully replay the existing selected on-disk history under the same
   lock against the retained independent checkpoint. Require equality with the
   retained complete accepted chain and exact marker. Require a strict clean
   layout. Any disagreement fences the handle before returning an error.
3. Set the fence before the first mutating filesystem operation. Create the next
   height's block with `create_new`, write its complete canonical MRB2 bytes, and
   synchronize the file. Synchronize the blocks directory.
4. Create `selection.pending` with `create_new`, write the complete next-generation
   MSU2 marker and synchronize that file. Its inner checkpoint describes the whole
   candidate history and resulting combined ledger state.
5. Compare the current selected marker's exact bytes with the retained old marker
   before publication. Never overwrite an unexpected selected marker.
6. Mark publication as attempted, rename the pending marker over `selection.v2`,
   and synchronize the stable root directory.
7. Only after all fallible publication work succeeds, install the candidate chain
   and marker in memory, clear the fence and return the new append checkpoint.

Only the selected marker is replaced. Prior blocks and the manifest remain
unchanged, and the root inode continues to identify the held lock. Under the stated
filesystem assumptions, the new marker is published only after its complete block
data and directory entry have been synchronized.

## 6. Fencing and errors

| Situation | Result and state |
|---|---|
| Read-only handle calls append | `ReadOnly`; no files or accepted state change |
| Already fenced handle calls append | `Fenced`; no new write is attempted |
| Invalid actual candidate or exhausted chain budget | Validation error; original state and files unchanged |
| Checked generation cannot advance | `GenerationOverflow`; no publication |
| Existing selected bytes differ | `SelectionChanged` or checked storage/replay error; handle fenced |
| Mutation began but selector rename not attempted | `UpdateIncomplete` with candidate checkpoint; handle fenced; files retained |
| Selector rename attempted, or later synchronization fails | `PublicationUncertain` with candidate checkpoint; handle fenced; files retained |
| Checked reconciliation fails | `ReconciliationFailed`; no usable handle returned; files retained |

Errors may wrap the precise underlying storage, selection or ChainV2 validation
error. A candidate checkpoint carried by an incomplete/uncertain error is the
checkpoint of the locally validated candidate; it is not evidence that publication
completed. The caller can retain it alongside the old checkpoint for explicit
reconciliation.

Failure before rename may leave a partial or complete extra block and/or pending
marker. Failure after rename may leave the new selected marker even though the
method returned an error. Therefore the method never resumes writes using only
the old in-memory state. While fenced, accepted-chain and selection getters reject
instead of presenting that state as the current durable selection.

No production sleep, retry loop, explicit unlock or cleanup weakens these outcomes.
An error does not trigger a different consensus rule, lower quorum, cache restore,
new genesis, monetary grant or larger resource budget.

## 7. Checked restart and reconciliation

Reopening and consuming reconciliation use the independent identity and append
checkpoint, a strict clean selected layout and full replay. On a writer path, the
validated selected manifest and every block are synchronized, then the blocks
directory, selected marker and stable root are synchronized before a writable
handle is returned. A synchronization error cannot yield a usable writer.

This reestablishes the publication prerequisites for visible validated data under
the filesystem assumptions. It does not retroactively prove what survived an
earlier failed synchronization or certify the storage device against power loss.
Read-only inspection does not perform this durability preparation.

| Restart observation | Outcome |
|---|---|
| Complete old clean selection; caller selects old checkpoint | Full validation and writer resynchronization may return the old store |
| Complete new clean selection; caller selects candidate checkpoint | Full validation and writer resynchronization may return the new store |
| Marker and supplied generation/branch pins disagree | Reject; never infer a replacement expectation from disk |
| Old marker plus extra new-height block or pending marker | Reject and preserve; staging repair is not implemented here |
| New marker with missing, partial or invalid required data | Reject; never expose a mixed or partial ledger |
| Foreign, legacy, unknown or noncanonical layout | Reject and preserve; no automatic adoption or deletion |

In particular, a prepublication interruption can make the old otherwise-valid
prefix unavailable through this strict opening API until a future explicit repair
procedure resolves the residual artifacts. This availability limitation is
intentional and must not be described as automatic crash recovery.

## 8. Verification scope and remaining work

Acceptance evidence must cover successful empty and mixed-operation appends,
unchanged disk/memory after invalid candidates, exact ledger/counter reconstruction,
independent generation and branch selection, writer exclusion, bounded inputs,
and deterministic failure injection around writes, synchronization, publication
and reconciliation. Every uncertain live handle must refuse further appends.
Fault scenarios must observe complete selected old/new state or explicit rejection,
with retained-artifact assertions. Normal test counts must distinguish subprocess
helpers, repeated release runs and multiple scenarios inside one test.

The independent MSU2 reference must derive the new envelope without rewriting old
MSS2, manifest or actual-chain expected values to match Rust output. Existing
references, examples and tests remain required gates. Injected errors and process
ownership checks are not torn-write, filesystem power-loss or disk-firmware tests,
nor an external security audit. Exact results are recorded in the checkpoints.

Remaining storage work includes explicitly authorized partial-staging repair,
canonical development fork choice, durable membership-aware branch replacement,
and peer reconstruction. These need separately reviewed selection, publication,
retention and recovery rules. MB-03 remains in progress after this append slice;
MB-04 supplies the persistent V2 node, pool, wallet and operator interfaces.

No extra founder allocation, marketing reserve, bootstrap payout, population mint,
fee/reward formula, biometric archive, real device capture or biological uniqueness
guarantee is introduced. Human finality, private fingerprint/ECG matching, actual
admission and both readiness gates remain later work under the existing decisions.
