# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03F1-001 — bounded V2 peer-history framing and reconstruction**.
Updated **2026-11-26**. Prior MB-01/02 and MB-03A/B/C/D/E components remain verified.
**MB-03F1: VERIFIED_COMPONENT — FRAMING / IN-MEMORY CANDIDATE RECONSTRUCTION.**
**MB-03F and MB-03 remain IN PROGRESS. Next: MB-03F2 — peer sessions and durable adoption.**
Gate A and Gate B remain UNMET.

## Exact source and publication

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


Prior acceptance remains in master sections 18–24. This increment adds fourteen
source/contract/workflow files or file changes (2,904 inserted lines, zero deleted),
including framing, receiver/sender, focused tests, one reference, one example and
the normative [peer-history contract](docs/PEER_HISTORY_V2.md). Local storage and
recovery remain separate verified components; F1 does not change their authority.

## Delivered behavior

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


## Bounds and remaining limits

MPN2 has a separate 12-byte header, version 2, and four known message kinds.
Begin and End each contain an exact 152-byte summary. There is no new signed
checkpoint, hash domain or local storage generation on the wire. For N complete
blocks, M manifest bytes and B MRB2 bytes, exact accounting is N + 3 frames and
`340 + M + B + 20*N` total wire bytes.

| Per-transfer bound | Maximum |
|---|---:|
| Complete blocks including genesis | 4,096 |
| Canonical MNM2 manifest bytes | 2,781,636 |
| Complete MRB2 bytes across all blocks | 16,777,216 |
| One complete MRB2 block | Configured cap, at most 1,048,576 bytes |
| Frames | 4,099 |
| Total framed wire bytes | 19,641,112 |
| Actual cumulative chain state/configuration items | 1,048,576 |

Genesis remains exactly 168 MRB2 bytes. After decoding the configuration, the
receiver rejects impossible declared totals using that fixed genesis and the
configured child-block limit. Before each payload, checked arithmetic reserves
both minimum and maximum possible space for all remaining blocks. Exact final
counts are required; unused declared capacity does not complete a transfer.

These are encoded-input and abstract-work limits, not resident-memory, latency,
bandwidth, connection-count or availability guarantees. F1 opens no sockets,
implements no deadlines/session lifecycle and performs no canonical storage
publication. End is a framing boundary, not an EOF or connection-close rule.


## Verification

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


## Exact next action

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
