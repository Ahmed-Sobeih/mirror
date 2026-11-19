# V2 storage initialization and checked reopening

**MB-03A / MIRROR-MB03A-001. Updated 2026-11-20.** This document freezes storage
profile 1: one explicitly selected, immutable finite V2 chain. The implementation
is `mirror_storage::v2`; [CHAIN_V2.md](CHAIN_V2.md) remains the actual block and
ledger authority. Current verification/publication evidence is in CURRENT_STATUS.

This slice initializes a new store from an already accepted `ChainV2`, exclusively
reopens it, or inspects it with shared ownership. Every reopen reconstructs the
complete chain through normal acceptance. **It has no append, tip-update,
replacement, automatic repair, global fork-choice or V2 node-daemon API.** Those
remain MB-03B and subsequent integration work. Legacy `BlockStore` and node recovery
are separate and unchanged; their automatic create/repair behavior is never called.

## 1. External authority and lifecycle

The public methods are:

```rust,ignore
StoreV2::initialize(path, &accepted_chain) -> Result<StoreV2, StorageErrorV2>
StoreV2::open_writer(path, identity, checkpoint) -> Result<StoreV2, StorageErrorV2>
StoreV2::inspect(path, identity, checkpoint) -> Result<StoreV2, StorageErrorV2>
```

`ManifestIdentityV2` independently pins configuration `C` and exact genesis hash.
`ChainReplayExpectationV2` independently pins those same two values plus final
height, tip hash and combined ledger root. The two arguments must agree before
filesystem access. Never derive these trusted expectations from the untrusted
record or peer being checked. An obsolete but deliberately selected checkpoint
still selects an old valid history; this API does not discover global freshness.

Initialization accepts only an absent final directory under an existing parent,
or an explicitly supplied empty directory. It never creates missing parents,
mines a genesis, imports a decoded state cache, grants funds or chooses a network
from an arbitrary peer manifest. Source bytes are encoded from a valid private
`ChainV2` before filesystem mutation. No files from an existing nonempty store are
overwritten. Initialization of the same complete store again is an error.

`open_writer` and `inspect` require an existing selected store. Neither creates,
mines, repairs, renames or deletes anything. All returned handles retain their
directory lock, accepted chain and selection record for their lifetime. They
provide only immutable `chain()`, `root()`, `selection()` and `access()` getters.
There is no `Clone`, mutable chain accessor or raw descriptor accessor.

| Observed directory | Behavior |
|---|---|
| Missing final directory | Open/inspection return `MissingDirectory`; explicit init may create it |
| Missing parent | All paths reject; no recursive creation |
| Empty directory | Open/inspection return `EmptyDirectory`; explicit init may use it |
| Legacy `.mrb` history, at root or under `blocks` | Reject as legacy; no reinterpretation or migration |
| Unrecognized user files | Reject as foreign or invalid layout; retain them |
| Recognized preparation files without `selection.v2` | `PartialInitialization`; no cleanup or promotion |
| Selected record with missing/extra entries | Reject invalid layout; no fallback to empty/genesis |
| Malformed, oversized or contradictory selected data | Reject exact format/length/budget/validation error |
| Different selected network or checkpoint | Reject identity/selection mismatch |
| Conflicting live directory lock | `Busy`; no waiting, takeover or stale-PID cleanup |

Unknown layouts are not exhaustively classified as every historical application
format. Enumeration itself is bounded; a directory with too many unrelated entries
can fail `InvalidLayout` before classification. All rejection paths retain data.

## 2. Directory ownership and platform assumptions

Profile 1 currently operates only on **Linux**; other operating systems return
`UnsupportedPlatform`. It uses Rust's standard nonblocking `File::try_lock` and
`try_lock_shared` on an opened **root directory inode**. The descriptor is private,
uncloned and retained by `StoreV2`. Exclusive ownership excludes both other writers
and shared inspectors; multiple shared inspectors may coexist and exclude writers.
Dropping the handle or terminating its process closes its descriptor and releases
the operating-system lock. There is no lock file, owner-PID authority, stale lease
or lock-path deletion. No upgrade/downgrade API is provided.

The store root must never be renamed, replaced or removed while any handle is
alive. All participants must honor these advisory locks. Parent directories and
the local filesystem owner are trusted. Static symlink components, symlink managed
files/directories, nonregular selected files and hardlinked managed files reject.
Opened file/directory device and inode are checked against path metadata; this is
defense in depth, **not race-free protection against a hostile same-user process
swapping paths during operations**. No descriptor-relative `openat2` sandbox or
cross-process malicious-owner defense is claimed.

Actual local tests ran on Linux tmpfs and workspace overlayfs. CI uses Ubuntu.
WSL2's Linux-native filesystem is the intended user environment, but was not
directly exercised here. Native Windows, `/mnt/e` DrvFS, NFS/SMB and other network
or translated filesystems have not been validated. The code gates the operating
system, not filesystem mount types; deployment must honor these assumptions.
Filesystem/kernel/hardware must implement directory locking, atomic same-directory
rename and file/directory synchronization correctly. No weaker silent fallback is
used when an operation errors.

Read-only means no application-level writes, file creation, repair or mutation of
stored bytes. Ordinary reads/locks may affect OS access times or transient kernel
state. Holding an inspection handle prevents a writer for the handle's lifetime;
this is not a concurrent live-status endpoint for an active writer daemon.
Writer mode currently establishes exclusive ownership; it does not promise that
filesystem permissions or a read-only mount will allow future write operations.

## 3. Exact layout and selection encoding

The complete root contains exactly three entries:

- `manifest.v2`: exact existing `MNM2` public manifest bytes, including every seed proof.
- `blocks/`: exactly the selected contiguous blocks from zero through selected height.
- `selection.v2`: exact `MSS2` record described below, published last.

Each block name is its height as exactly 20 decimal digits followed by `.mrb2`,
for example `00000000000000000000.mrb2`. Each file contains its exact existing
`MRB2` bytes. No cached account/membership state, trusted resource counter or
biometric archive is stored. Extra tail blocks, other filenames and extra root
entries reject. `selection.pending` exists only during initial preparation; its
presence never authorizes selection and it is not silently removed on open.

`MSS2` is exactly **232 bytes**. Integers are unsigned little-endian; hashes retain
their ordinary raw byte order. No trailing bytes, alternate profile or generation
is accepted by this initial implementation.

| Offset | Field | Size / required value |
|---|---|---|
| 0 | Magic | 4 bytes `MSS2` |
| 4 | Encoding version | u16 = 2 |
| 6 | Storage profile | u16 = 1 |
| 8 | Generation | u64 = 0 |
| 16 | Configuration hash | 32 bytes `C` |
| 48 | Genesis hash | 32 bytes |
| 80 | Selected height | u64 |
| 88 | Selected tip hash | 32 bytes |
| 120 | Selected ledger root | 32 bytes |
| 152 | Block count, including genesis | u32 |
| 156 | Exact manifest length | u32 |
| 160 | Sum of complete selected MRB2 lengths | u64 |
| 168 | Manifest checksum | `SHA256d(exact MNM2 bytes)`, 32 bytes |
| 200 | Record checksum | `SHA256d("MIRROR_STORAGE_SELECTION_V2" || bytes[0..200])`, 32 bytes |

Checksums detect corruption, not authenticity, freshness or branch choice. The
external checkpoint is mandatory even when a checksum is valid. Full manifest
proof verification and full chain replay remain mandatory after metadata checks.
No existing network, header, operation or state hash domain changes.

Metadata bounds are: block count 1..=4096, height exactly count minus one, manifest
length 46..=2,781,636, and complete block-byte total between 168 times count and
16,777,216. Manifest and block decoders apply their stronger structural/configured
limits afterward. Generation other than zero rejects rather than inventing update
semantics. Existing chain state/config budget 1,048,576 is recomputed by replay;
it is not restored from metadata or raised by storage.

## 4. Initialization publication and errors

Under exclusive directory ownership, preparation proceeds in this order:

1. Encode accepted source data, validate the path, create the final root only if
   absent, acquire its exclusive lock and confirm it is empty.
2. Synchronize the existing parent directory so the root entry can be durable.
3. Create `blocks/`; write `manifest.v2` and every block using `create_new`.
   Each file is fully written and `sync_all` succeeds before continuing.
4. Synchronize `blocks/`, then the root directory, persisting the prepared entries.
5. Write exact `selection.pending` with `create_new` and synchronize its file.
6. Rename `selection.pending` to `selection.v2` in the same root directory.
7. Synchronize the root directory again, then return the owning store handle.

No existing root or populated block directory is swapped. The stable root inode
continues to identify the lock. Under the stated filesystem assumptions, the
selection marker cannot deliberately authorize preparation that has not already
passed file/directory synchronization. The final marker may be absent after a
crash before publication, or present and subject to complete checked reopen.

Before a selection rename is attempted, a preparation error is returned as
`InitializationIncomplete`. Once publication is attempted, any rename or later
error returns `PublicationUncertain`. No store handle is returned in either case;
partial data remains untouched. Failed root creation/ownership/layout checks
return their direct errors before file preparation. If only an empty root remains,
explicit initialization may reuse that empty directory; no partial files are deleted.

After an uncertain result, the caller must reopen/inspect against its **retained
external identity and checkpoint**. Complete replay may succeed or report an
explicit error; directory existence, a checksum or a leftover pending marker alone
cannot authorize continued writes. This slice has no additional write API with
which to continue blindly. Partial/corrupt stores require a later explicitly
specified recovery operation or operator reconciliation, not automatic deletion.
Reopening proves the visible selected history validates; it cannot retrospectively
prove that an earlier failed synchronization will survive a subsequent power loss.

## 5. Bounded reopening and complete reconstruction

After path checks and lock acquisition, root enumeration stops at a small fixed
entry bound. Exact layout is required. Read and decode the 232-byte selection
against the external checkpoint, then bound/read/checksum/decode the exact manifest
against the external identity and all seed proofs.

Next scan at most the declared block count plus the first disqualifying entry.
Reject missing/extra/noncanonical heights, aliases and nonregular files. Preflight
every file length against the configured block limit and the aggregate 16-MiB
history ceiling **before allocating or reading block payloads**. The total must
equal the selection record. Each read rechecks opened identity/length, reserves a
bounded buffer and reads at most its preflight length plus one sentinel byte.
Shortening or growth rejects instead of escaping the budget.

Successful selected encoded payload is bounded by
232 + 2,781,636 + 16,777,216 = **19,559,084 bytes**. This is not a total resident-memory
limit: buffers, sentinel bytes, decoded blocks, configuration and replayed ledger
state have overhead and their separate existing bounds. No unbounded `fs::read`,
silent history eviction, enlarged budget or partial success is used here.

All complete block bytes then enter `ChainV2::replay_encoded`. Genesis, actual
ancestry, proof of work, operations, signatures, ordered atomic transfers and
membership, roots, retained records and cumulative budgets are revalidated.
Only a complete replay matching the external tip and ledger root can be returned.
Two membership forks can reopen independently under different expectations; that
does not select a global canonical chain or demonstrate a durable reorganization.

## 6. Verification and remaining work

`tools/reference_storage_v2.py --check` independently derives MSS2 from the frozen
prior actual-chain fixture, checking that fixture's existing digest first. The new
selection fixture was generated before Rust comparisons. Genesis, full mixed
history and a genuine membership fork have independent expected records. No prior
fixture is regenerated to agree with new code.

Tests exercise codec boundaries and checksum-correct contradictions, every external
pin, actual multi-block filesystem replay, absence of automatic legacy migration,
read-only byte/name/mtime preservation, truncation/corruption/oversize, missing and
extra files, static aliases, independent descriptors/processes, process termination
and lock release. Initialization injection covers preparation boundaries and both
post-publication boundaries. These deterministic injected errors and killed lock
holders do **not** emulate power loss, torn kernel writes, disk firmware failures
or every filesystem. No universal crash-safety certification is claimed.

Run the bounded `storage_v2_demo` for explicit initialization, second-writer
rejection, complete reopening and independently selected membership-fork recovery
in newly created temporary directories. It never opens user runtime stores and
does not update the legacy node.

**Next MB-03B:** define and implement accepted-tip updates/generation authority,
staging, failure reconciliation and restart behavior under continued single-writer
ownership. Preserve this profile and all old bytes. Specify canonical development
fork choice before any branch replacement, then add durable membership-aware
reorganization and peer reconstruction. MB-04 supplies the persistent V2 node/API.
Human finality, real fingerprint/ECG admission and private matching remain later
gates. No economics, founder privilege, fee, mint or grant is introduced here.
