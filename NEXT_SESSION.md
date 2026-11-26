# Next session — MB-03F2

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-26**.

## Recover current state first

Read AGENTS.md, CURRENT_STATUS.md, the-prompt, MASTER_PLAN.md's tracker and section 25,
docs/DECISIONS.md and docs/PEER_HISTORY_V2.md. Verify live refs, PR #1, recent commits/
comments and exact-source CI. Preserve newer/unpushed work and runtime data. Never
reset to this checkpoint.

Implementation source: **43ef08ca6d3e7878b7dab2d2355a7265d41a0351**.
Tested tree: **7521e7906f2a64cee3a777a9aa158ddcba4e2078**.
Parent: **ce5203aa78717134a753f014cca0753a4519eb08**.
[Source push CI 45739035829](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45739035829)
and [source PR CI 45739040653](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45739040653)
both passed. Full push-job `139639209001` logs confirmed the exact source checkout
and all required workspace, documentation, release, reference, example and audit
gates. Later documentation-head CI is checked separately and recorded on PR #1;
these source results do not assert a future head is tested.
PR #1 remains open and unmerged on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.


MB-03F1 is verified within framing/in-memory reconstruction scope. Earlier local
comparison/storage/recovery components remain verified. MB-03F and MB-03 remain
IN PROGRESS; Gate A/B remain UNMET.

## Exact next work

**Next: MB-03F2 — bounded V2 peer sessions and durable candidate adoption. READY.**

1. Read [the F1 wire/receiver contract](docs/PEER_HISTORY_V2.md), the existing V1
   TCP/node path and accepted V2 chain/comparison/storage contracts. Specify an
   explicit V2 session/request boundary; preserve frozen V1 and F1 bytes.
2. Select C/genesis from independent local configuration, and keep the provider's
   accepted history immutable for one transfer. Read the fixed header first,
   call consuming `begin_frame`, then allocate/read only its admitted payload.
   Drop a pending owner and close/discard the attempt on I/O failure.
3. Enforce bounded requests/connections, cumulative bytes and an absolute session
   deadline. Repeated short reads or resets of individual socket timeouts must
   not extend an incomplete transfer forever. Specify End and subsequent session
   behavior explicitly; do not infer candidate completion from EOF.
4. Exercise actual TCP peer processes, fragmented input, early close, stalled or
   oversized input, changed claims and invalid blocks. Failed reception returns
   no candidate and cannot mutate the selected store.
5. Only after complete `finish`, compare the actual accepted candidate with the
   locally selected current history under independent C/genesis. Keep current
   on ties/shorter histories. Use existing owned storage publication for a preferred
   branch; explicit fresh initialization stays separate from existing-store open.
   Retain both exact histories and local pins needed for explicit recovery. Peer
   generations, height/root claims and transport completion are not write permits.
6. Demonstrate a fresh peer rebuilding complete state, plus a real transfer across
   equal-account/different-membership histories. Verify branch comparison, durable
   publication and strict reopen against independently retained local pins.

F1 protocol/receiver acceptance is complete; MB-03F and MB-03 remain in progress.
Do not mark network integration complete from an in-memory fixture. Persistent
daemon, transaction pool, wallet and operator APIs remain MB-04; complete Ant Kiss
is MB-05 and final human consensus MB-06. Production fee/reward formulas, private
matching and devices retain later decision/acceptance gates. No monetary decision,
hardware purchase or real biometric collection blocks MB-03F2.


## Preserve the completed receiver boundary

The separate V2 protocol now carries one finite candidate history: Begin, its
complete canonical manifest, genesis and every child block in order, then an
explicit matching End. The caller selects the expected configuration hash and
genesis independently before accepting any peer data. Every advertised height,
tip, root and byte count is a claim to check, not trusted replay authority.

The receiver validates the manifest and seed proofs, then reconstructs the branch
through actual `ChainV2::from_genesis` and `append`. All original mixed operations
execute against that branch's own ancestry. Accounts, memberships, permanent
reservations, approvals, sequences, population history and resource counters stay
together. Computed terminal state and exact counts must match the captured claim.
A valid shorter or losing branch is still a candidate; receiving does not select
it as canonical, establish global freshness or change a store.

Each receive operation consumes ownership. A checked 12-byte header must pass
phase, per-kind, configured and remaining-transfer limits before a pending owner
exposes its permitted payload length. Consuming that owner requires exactly those
payload bytes. Errors destroy continuation; partial accepted state has no public
getter or clone. All blocks alone are insufficient: only a matching End and a
successful consuming `finish` release the complete `ReceivedCandidateV2`.

The sender borrows one immutable accepted history, derives its summary from that
history, and yields one frame at a time. It retains no all-history output buffer
and stops permanently after completion or an encoding error. The convenience
receiver for an already allocated complete frame cannot establish allocation
safety for its caller; a future stream adapter must use checked header admission
before allocating or reading the declared payload.


Read [PEER_HISTORY_V2.md](docs/PEER_HISTORY_V2.md) for exact frame layouts, offsets,
byte arithmetic and caps. `ReceivedCandidateV2::chain` exposes a completed accepted
history; `into_chain` consumes that result. No receiver/pending clone or partial-
chain getter exists. Peer claims cannot seed trusted `ChainReplayExpectationV2`.

## Preserve the recovered storage boundary

The following E2b behavior remains unchanged by F1.

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

The E2b recovery increment changed no encoding, hash domain, dependency, monetary
rule or prior store behavior; F1 preserves that recovery behavior.
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

Read [recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[replacement store](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[MSR2/preparation](docs/STORAGE_REPLACEMENT_V2.md),
[fork choice](docs/FORK_CHOICE_V2.md), [chain](docs/CHAIN_V2.md),
[ledger](docs/LEDGER_STATE_V2.md), [threat model](docs/THREAT_MODEL.md) and
[test matrix](docs/MEMBERSHIP_TEST_MATRIX.md). Keep both reference histories outside
the store lifecycle; no implicit archive, inferred intent or cache authority exists.


## Actual verification and required gates

Local verification used Rust **1.98.0** and locked dependencies:

- **951 workspace/all-target runtime tests passed**, zero failures: 891 retained
  plus **60 new** (24 protocol-codec and 36 receiver/sender integration tests).
  Boundary scenarios inside one test function are not additional unique tests.
- **Nine compile-fail doctests passed**, eight retained plus one proving that a
  receiver cannot be reused after a consuming failure.
- Required release gates passed: personhood **234 runtime + 3 doctests**, core
  **121**, state **49**, consensus **23**, chain **85 runtime + 3 doctests**,
  storage **350 runtime + 2 doctests**, protocol **38**, and network
  **40 runtime + 1 doctest**. Release reruns are not new unique tests.
- All **ten independent Python reference checks** and **fourteen synthetic
  examples** passed. One subprocess helper remains intentionally ignored during
  ordinary enumeration and is invoked by the parent ownership tests.
- Scoped Rust formatting, whitespace and allocation/runtime-data audits passed.
  **120 of 124 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent. Existing protocol/network libraries only add the V2 module
  exports/documentation. Their legacy implementation is otherwise byte-identical.
  Network dependencies and Cargo.lock add four local workspace edges; no external
  dependency version changes. Prior encoding fixtures and monetary code are unchanged.

The independent Python reference rebuilds the existing synthetic chain and packs
the new headers, summaries, height prefixes and transcript accounting itself.
It neither runs Rust nor uses Rust output as its oracle. The complete-transcript
test hash is a reference assertion, not a new cryptographic domain on the wire.

The fourteenth example, `history_receiver_v2_demo`, reconstructs the accepted main
history and a genuine competing membership history from separate in-memory frames:

| Reconstructed history | Height | Active / pending / historical peak | Frames / wire bytes | MRB2 bytes / state items |
|---|---:|---|---|---|
| Main | 4 | 1 / 0 / 3 | 8 / 5,406 | 4,330 / 75 |
| Competing branch | 3 | 1 / 1 / 2 | 7 / 4,493 | 3,437 / 58 |

It also checks equal-account branches whose membership histories differ at height
3, with the existing comparator keeping current on a tie. Main tip remains
`33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d` and ledger root
`785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.

Receiver tests cover missing/repeated/reordered frames and blocks, unfinished
phases, wrong network/seed proofs, premature or changed End, false endpoints,
invalid actual work/roots/signatures/branch anchors and pre-payload bounds.
The example itself opens no TCP connection and publishes no store. Review and
tests establish this component's stated behavior, not an external security audit
or production network readiness.


Run every AGENTS.md gate using Rust 1.98.0 and locked dependencies: workspace
all-target tests and doctests; release personhood/core/state/consensus/chain/storage/
protocol/network; ten Python references; fourteen examples; allocation/runtime
audit; scoped formatting and whitespace. Confirm exact implementation source and
CI, then check any later documentation head separately. Do not count reruns as
new tests or turn test counts into a readiness percentage.
