# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03E2B-001 — explicit interrupted replacement recovery**.
Updated **2026-11-26**. Prior MB-01/02 and MB-03A/B/C/D/E1/E2a components remain verified.
**MB-03E2b, MB-03E2 and MB-03E: VERIFIED_COMPONENT — LOCAL REPLACEMENT / RECOVERY.**
**MB-03 remains IN PROGRESS. Next: MB-03F — bounded fresh-peer reconstruction.**
Gate A and Gate B remain UNMET.

## Exact source and publication

Implementation source: **99bf0da86469f5832fe9e5c855e9729676618887**.
Tested tree: **d20a433805287d16765e2c9d01957056e8cfef2a**.
Parent: **5706ed4b4d5035aba8dc5be274bb7a35f048bee7**.
[Source push CI 45731894201](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45731894201)
and [source PR CI 45731901717](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45731901717)
both passed. Full push-job `139615676064` logs confirmed the exact
source checkout and all required workspace, documentation, release, reference,
example and audit gates. Later documentation-head CI is checked separately and
recorded on PR #1; these source results do not assert a future head is tested.
PR #1 remains open and unmerged on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Prior acceptance remains in master sections 18–23. This increment changes ten source/
contract/workflow files (3,587 insertions, 24 deletions), including recovery, tests,
one example and the normative recovery contract. Earlier source histories and
frozen on-disk encodings remain preserved.

## Delivered behavior

The separate explicit recovery API now accepts independently retained old and
candidate checkpoints/generations, BOTH exact accepted histories, and an explicit
`Complete` or `RetainOld` action. The request immutably borrows the histories;
construction validates their exact relationship and recovery repeats selection
checks under independently selected network identity and exclusive root ownership.

Before mutation, recovery bounds all names/lengths, checks the exact canonical
selected MSR2, fully replays selected history, and verifies every surviving residue.
While old remains selected, candidate files may be any authorized subset of heights,
with exact canonical byte prefixes; the manifest may be absent. Pending-selector
prefixes require the entire candidate history to be complete. Once candidate is
selected, its history must be complete, pending must be absent, and every remaining
old file must contain its exact full bytes. A surviving old blocks directory requires
the old manifest; manifest-only, empty generation, and absent old generation are
recognized cleanup states. Unknown/corrupt/overlong/unsafe artifacts reject unchanged.

Completion extends authorized prefixes without truncation, synchronizes and fully
validates candidate history, checks both exact generations, publishes/synchronizes
the selector, and rechecks all residue before deleting old files. Selected-candidate
recovery also reestablishes selected-file/directory/selector/root durability before
old deletion. Retention removes and synchronizes a pending marker before removing
candidate staging. Both actions remove only exact prescribed unselected files and
empty directories; no recursive cleanup or locked-root replacement occurs.

An already-absent old blocks directory still requires old generation-directory
synchronization BEFORE old manifest unlink. Otherwise an earlier interrupted
directory removal could remain nondurable. The implementation and regression
trace cover that ordering, including interruptions of recovery itself.

Final strict clean replay, equality and synchronization precede the returned writer.
Consuming recovery retains the same lock, including from a fenced handle, and requires
its retained memory checkpoint to match one supplied endpoint. Every error returns
no handle. A publication attempt may already have selected candidate. Repeated
completion/retention is explicit and bounded; actions can change while old is still
selected, but `RetainOld` NEVER rolls back a selected candidate. Ordinary open,
inspect and reconcile remain clean-only and never call recovery implicitly.

Accounts, memberships, permanent reservations, approvals, sequences, population
history and replay/resource counters always come from one fully accepted history.
Copied roots, cache state, filenames and local generations never substitute for
independent selection or complete execution.

See [the recovery contract](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md) and
[the replacement store](docs/STORAGE_REPLACEMENT_STORE_V2.md).

## Bounds and remaining limits

No encoding, hash domain, dependency, monetary rule or prior store behavior changes.
MSR2 remains 280 bytes, version 2/profile 3, with the unchanged nested MSS2.
The distinct two-history/two-selector payload ceiling is **39,118,264 bytes**;
all payload reads and growth sentinels share one **313,011,680-byte** allowance.
With one conservative history pass H=19,562,949 and selector S=281 bytes:

| Recovery path | Full replays | Conservative charged reads |
|---|---:|---:|
| Old selected → Complete | 4 | 15H + 12S = 293,447,607 bytes |
| Candidate selected → Complete | 2 | 9H + 6S = 176,068,227 bytes |
| Old selected → RetainOld, pending present | 2 | 11H + 10S = 215,195,249 bytes |

Every history retains 4096 blocks, 16 MiB complete MRB2 and 1,048,576 cumulative
state/configuration items. These are not RSS, latency or physical durability bounds.
Cooperative Linux locking and trusted local owners/parents remain assumptions;
WSL/DrvFS, native Windows and network filesystems are not newly certified.
Interrupted initialization, missing/corrupt selected markers and unknown artifacts
remain unsupported and preserved. Recovery needs both histories; it cannot infer
missing authorization from damaged residue or a peer's higher generation.

## Verification

Local verification used Rust **1.98.0** and locked dependencies:

- **891 workspace/all-target runtime tests passed**, zero failures: 836 retained
  plus **55 new** (38 public recovery integration, 15 private failure/retry/read-
  budget, and two independent-process ownership tests). Boundary scenarios within
  one function are not additional tests.
- **Eight retained compile-fail doctests passed**; no new documentation-test count.
- Required release gates passed: personhood **234 runtime + 3 doctests**, core
  **121**, state **49**, consensus **23**, chain **85 runtime + 3 doctests**,
  storage **350 runtime + 2 doctests**. Reruns are not new unique tests.
- All **nine independent Python reference checks** and **thirteen synthetic
  examples** passed. One subprocess helper remains intentionally ignored during
  ordinary enumeration and is invoked by the parent process tests.
- Scoped formatting, whitespace and monetary/runtime-data audits passed.
  **117 of 120 retained source/dependency/reference/fixture files are byte-identical**
  to the parent. Changed retained Rust files add module exports/docs and recovery
  cases to the existing process harness. Cargo.lock, dependency manifests, previous
  encoding fixtures, monetary code and legacy runtime source are unchanged.

Root and independent implementation review identified and corrected the absent-old-
blocks synchronization edge before final verification. Read-budget charging was
also moved before opening existing-prefix payloads. Final independent review found
no remaining blocker in the scoped component. This is not an external security
audit, physical power-cut test or proof of hostile-owner race resistance.

The thirteenth example, `replacement_recovery_v2_demo`, constructs synthetic
sparse staging, completes the preferred branch, reopens and checks full state,
repeats completion, rejects rollback and separately retains old. It uses controlled
filesystem residue, not an actual physical power cut. Recovered candidate is height
4 / generation 1; tip
`33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`, ledger root
`785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.
Active/pending/high-water is **1/0/3**, encoded block bytes/state items **4330/75**.
Tests also recover equal-account branches with different membership histories,
compare complete ChainV2 values and preserve all branch-specific counters.

## Exact next action

**Next: MB-03F — bounded fresh-peer V2 reconstruction. READY.**

Read the existing V1 protocol/network/node interfaces and accepted V2 chain/storage
contracts before designing the new wire boundary. Add an explicitly versioned V2
path; do not reinterpret legacy bytes, initialize an existing store implicitly,
or treat a peer's advertised checkpoint as accepted state.

Start with a concrete bounded protocol/receiver slice: independently selected
network configuration/genesis, strict message versions/kinds/lengths, a bounded
manifest and exact complete block bytes, bounded request counts and byte totals,
and an explicit terminal candidate checkpoint. Treat peer-provided height, tip and
root as claims to verify by complete actual ChainV2 replay. The caller's independent
network selection remains authority; local storage generations are never peer
fork scores or freshness evidence. Select only fully accepted branches through
the existing comparator before durable publication under existing root ownership.

Preserve all history/state/work caps and membership-aware replay. Reject wrong
networks, malformed/truncated/oversized frames, missing/duplicate/out-of-order
blocks, changed manifests/tips, invalid signatures/work/roots and interrupted
transfers without accepting a partial candidate or mutating the selected store.
Keep existing valid stored data and both old/candidate histories available for
publication recovery. No cache/root, peer counter or filtered membership log can
stand in for the complete original mixed-operation blocks.

Demonstrate independent fresh-peer reconstruction across genuine membership forks,
including equal account bytes with different membership, reservations and high-water;
verify complete rebuilt state and persist/reopen it under independent local pins.
Separate protocol-codec, receiver/replay and actual peer-process evidence; do not
mark the whole network complete from a standalone decoder or in-memory fixture.
Persistent daemon, transaction pool, wallet and operator APIs remain MB-04;
complete Ant Kiss is MB-05, final human consensus MB-06. Production fees/rewards,
private matching and actual devices retain their own later decision/acceptance gates.
No monetary choice, hardware purchase or real biometric collection blocks MB-03F.
