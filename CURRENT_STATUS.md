# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03A-001 — immutable V2 storage VERIFIED_COMPONENT**.
Updated: **2026-11-20**. MB-01A, MB-01B/B1–B4, MB-02A/B/C and MB-03A are
complete within their recorded component scope. **MB-03 remains IN PROGRESS.**
**Next: MB-03B — durable accepted-tip updates and failure reconciliation. READY.**
Local gates and corrected exact-source CI passed. The resolved test-isolation issue is recorded below.

## Exact source and publication

Active branch: `work/personhood-register-v0.1`; [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
is open and unmerged. Main is preserved at `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
Published MB-03A implementation:
[`528c8912c5a50cfd2e86856d345b29d6bb79d2c5`](https://github.com/Ahmed-Sobeih/mirror/commit/528c8912c5a50cfd2e86856d345b29d6bb79d2c5).
Its tree `65004e73f4c19deedafdb8cf2874939df0ab367a` is identical to the locally
verified source tree. The source increment changes 16 files: 3,355 insertions and
2 deletions. It adds the V2 storage component, contract, fixtures, reference,
example and tests; it preserves legacy execution and historical bytes.

[Corrected-source push CI 44639322362](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44639322362),
job `135965942902`, and [PR CI 44639329630](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44639329630)
both **SUCCEEDED**. Full push-job logs were inspected: checkout matched
`528c8912c5a50cfd2e86856d345b29d6bb79d2c5`, and every workspace, documentation,
release, reference, example and audit gate passed. Verification date: 2026-11-20.

The initial source `2370ff5df3a73ff874a8fb0031df9e2a98775a43` passed PR CI
44638347110 but failed push run 44638337326 in a release ownership test. That
failure was reproduced locally: an unrelated test's fork briefly inherited a lock
descriptor. The correction isolates the five parent process tests without changing
production behavior, sleeps, retries or ownership assertions. All 200 repeated
release executions then passed; the corrected workspace, docs and storage release
gates also passed. Repetitions are not additional unique tests. Final acceptance
uses the corrected exact-source run, not the earlier successful PR result.

Checkpoint documentation follows the implementation. Verify any later documentation
head independently; an older successful source run does not establish a newer head's
result. Never reset newer work to a checkpoint or merge only to publish handoffs.
Historical MB-02C evidence remains in master-plan section 17; section 18 records
the MB-03A increment.

## What MB-03A implements

**Explicit initialization and checked reopening.** `mirror_storage::v2::StoreV2`
initializes an absent or explicitly empty directory from an already accepted
`ChainV2`, opens an existing selection with exclusive ownership, or inspects it
with shared ownership. Full finite histories, including genesis-only histories,
are supported. Every reopen requires independently supplied network identity and
full checkpoint pins. Reading a status never creates a genesis, imports a cache,
repairs a directory or selects a network from a peer.

**One owner and read-only inspection.** Linux directory-inode locks remain held
for the lifetime of each store handle. An exclusive owner rejects competing writers
and inspectors; multiple shared inspectors may coexist and exclude a writer.
There is no lock file, PID takeover or stale-lock cleanup. Closing a handle closes
its descriptor; the lock releases only after all descriptors referring to that open
file description close. A concurrent fork can briefly prolong ownership until an
inherited descriptor closes. Inspection performs no application-level file writes;
ordinary OS access-time effects are excluded.

**A bounded immutable storage format.** Profile 1 stores exact public MNM2 metadata,
complete MRB2 block files, and a 232-byte MSS2 selection record at generation zero.
The record binds configuration, genesis, selected height/tip/root, counts, lengths
and integrity checksums. It cannot choose authority or establish freshness.
Missing, empty, partial, legacy, foreign, inconsistent and oversized inputs have
explicit rejection behavior. All files remain in place on rejection.

**Preparation before publication.** Initialization writes and synchronizes the
manifest and every block, synchronizes directories, then stages and publishes the
selection marker last. Preparation failures retain partial files and return no
store handle. An error after publication is attempted reports uncertainty and
requires checked reconciliation against the caller's retained external selection.
There is no automatic cleanup, promotion of a pending marker or retry that overwrites
existing data.

**Full ledger reconstruction.** Reopening bounds every file and the complete history
before reading block payloads, then calls `ChainV2::replay_encoded`. Actual genesis,
ancestry, work, signed operations, transfer/membership execution, roots and cumulative
budgets are revalidated. Different membership forks reopen under their separate
selected checkpoints. No stored account cache, population counter or approval cache
becomes a second authority.

The normative [storage contract](docs/STORAGE_V2.md) records exact bytes, publication
order, error semantics, bounds and filesystem assumptions. [CHAIN_V2.md](docs/CHAIN_V2.md)
continues to govern actual block and ledger validity. Previous configuration,
operation, block, state and manifest encodings remain unchanged.

## Observed local and exact-source CI verification

| Check | Result |
|---|---|
| Workspace/all-target tests | **588 passed = 523 retained + 65 new** |
| New cases | **24 selection codec + 34 filesystem integration + 5 ownership/process + 2 initialization fault tests** |
| Initialization fault scenarios | **9 boundaries within the 2 fault tests**, not 9 additional tests |
| Subprocess helper | **1 explicitly ignored helper entry** in ordinary enumeration; exercised by process tests |
| Documentation tests | **3 retained compile-fail cases passed** |
| Core/personhood release reruns | **121 / 234 tests passed**, plus the same 3 personhood doctests |
| State/consensus/chain release reruns | **49 / 23 / 57 tests passed** |
| Storage release rerun | **75 tests passed**; release reruns are not new unique tests |
| Independent references | **All 7 passed**, including `reference_storage_v2.py` |
| Synthetic examples | **All 7 passed**, including `storage_v2_demo` |
| Formatting, diff, preservation and dependency checks | **Passed** |

The new reference independently derives MSS2 from the frozen actual-chain fixture.
Filesystem cases cover exact full-history replay, branch-specific membership,
external pins, preserved legacy/partial data, unsafe aliases, nonregular files,
corruption and both per-file and aggregate read limits. The aggregate case rejects
17 individually bounded 1-MiB sparse files before any block decoding. Process tests
exercise actual independent descriptors/processes and release after termination.

These checks are not an external security audit or a power-loss simulation. Linux
tmpfs and workspace overlayfs were exercised locally; Ubuntu source CI passed
separately. WSL2's Linux-native filesystem is intended but was not directly tested;
native Windows, DrvFS, network filesystems and malicious local-owner path replacement
are not validated guarantees. Only an internal storage-to-chain dependency was added;
no external package changed. Runtime directories, legacy APIs/genesis and user policy
were preserved.

## Exact boundary and next work

MB-03A persists **one immutable selected finite history** and checks it on reopen.
It has no append, accepted-tip update, replacement, automatic repair, global fork
choice or V2 node-daemon API. `open_writer` grants exclusive ownership; it does not
expose an update operation. Profile 1 accepts only generation zero.

MB-03B must first specify a separately versioned update profile and any explicit
migration, then implement durable accepted-tip updates with staged accepted blocks,
file/directory synchronization, coherent selector publication and failure fencing.
An ambiguous I/O result must prohibit further writes until reconciliation. Tests
must distinguish complete old/new selected states or explicit rejection at restart.
Preserve the stable locked root and the existing immutable profile. Canonical fork
choice, durable membership reorganization and peer reconstruction follow; MB-04
provides the persistent V2 node and operator APIs.

Existing chain caps remain **4,096 blocks**, **16 MiB of complete MRB2 history** and
**1,048,576 cumulative abstract state/configuration items**. Storage does not raise
these caps, prune identity history or trust persisted resource counters. Child time
is strictly greater than parent time; no wall-clock/future ceiling exists. These
finite synthetic rules are not production scalability or human-consensus claims.

No extra founder allocation or privileged marketing reserve exists. First 1,000
qualifying humans, including the founder, share the agreed 10-MRY bootstrap policy,
not yet paid. Ordinary allowance is 0.2 MRY per qualifying historical population
maximum; 1 MRY = 10,000,000 Nusa. D-01 combined bootstrap treatment and D-04 rewards/
fees remain unresolved. No mint, grant, nonzero fee or reward was enabled. The
historical public Alice 100-MRY fixture is not a founder entitlement.

Fingerprint/ECG capture, private duplicate matching, real admission, anonymous
membership proofs and human-round consensus remain later work. No raw or encrypted
biometric archives belong on-chain or in GitHub. Exact key/evidence equality is
not biological uniqueness; public review metadata is linkable.
**Gate A and Gate B remain UNMET.**
