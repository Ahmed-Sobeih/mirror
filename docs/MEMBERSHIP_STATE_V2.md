# V2 membership state and block execution

Original codec checkpoint: **MIRROR-MB01B-B4-001**. Synthetic development profile only.
This document preserves the B3 state model/execution and its frozen MPS2 layout.
B4 adds the canonical codec, commitment and untrusted cache boundary. Grouped
diagnostic replay is specified in [REPLAY_V2.md](REPLAY_V2.md). Exact published
acceptance evidence is recorded in CURRENT_STATUS.md. See [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md),
[CONFIG_V2_ENCODING.md](CONFIG_V2_ENCODING.md), [OPERATIONS_V2_ENCODING.md](OPERATIONS_V2_ENCODING.md)
and [MEMBERSHIP_TEST_MATRIX.md](MEMBERSHIP_TEST_MATRIX.md).

No runtime membership transition proves biological uniqueness. Fingerprint/ECG
capture and private duplicate matching remain external, later work. Only synthetic
commitments/public development keys belong in examples. There is no monetary API,
grant, founder privilege, anonymous credential or live V2 block network in this
component. B4 completes MB-01B; MB-02A supplies the transfer/block formats in
[CORE_V2_ENCODING.md](CORE_V2_ENCODING.md). MB-02B now implements indexed membership
staging and atomic combined execution in [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md).
MB-02C now establishes actual finite-branch block authority and complete MRB2
replay in [CHAIN_V2.md](CHAIN_V2.md). The implemented [MB-03A storage layer](STORAGE_V2.md)
initializes an immutable selected chain and reopens it through complete MRB2
replay under independent network/checkpoint pins. Continued tip updates, failure
recovery, networking and reorganization remain later integration. Membership-only
behavior and MPS2 bytes/domains are preserved; section 5 documents the explicit
extension of MPS2's accepted inclusion-index range for mixed blocks.

## 1. State and trusted chain association

`MembershipStateV2` is an immutable snapshot with private fields. It contains the
validated configuration, processed height, records, derived key/evidence indexes
and counters. A `MembershipRegistryV2` associates that snapshot with a tip hash as
**external provenance**, not as part of the canonical membership bytes.

Bootstrap accepts only `VerifiedBootstrap`, so every configured seed has already
proved possession under the exact configuration. Seeds become Active at height 0,
with membership sequence 0 and activation ordinals 1..N in configuration order.
The seed's inclusion position is `(0, configuration_index)`. Runtime seeds obey
the same approval/withdrawal rules as other Active members.

`stage_block` accepts an immutable trusted parent-chain view and an ordered bounded
slice of signed membership operations. The view must have the same configuration,
height and canonical parent hash as the registry's stored association. Its ancestry
must be locally validated and stable throughout execution. A peer's unchecked
height/hash map is not such a view.

`begin_indexed_block` additionally accepts a total full-block operation count and
an immutable membership projection with strictly increasing original indices. Its
consuming builder applies each next planned membership operation at that full
position and finalizes only after all planned operations succeed. It never
authenticates the omitted transfers. MB-02B's ledger executor separately preflights
and executes the complete mixed list, preserving operation order and discarding
both account and membership candidates on any failure. The complete contract and
byte charging are in [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md).

Successful staging returns a new `StagedMembershipStateV2`; the previous registry
is unchanged. `bind_to_validated_block` associates a staged snapshot with its block
hash only **after the trusted outer chain executor accepts that block**. This is a
trusted integration boundary, not a block-validation function. It does not verify
PoW, a header, genesis pinning or a committed state root. B3 tests use synthetic
block hashes. MB-02C's separate `ChainV2` now supplies the actual validated
association by checking complete blocks and the combined ledger root before
binding; its acceptance path uses only its own accepted branch for ancestry.

The current block hash is deliberately excluded from committed membership state:
the V2 block header commits to the combined ledger root, so including that
same block hash in its root would create a circular dependency. No unbounded
single-event mutation API or setter for state fields is provided.

## 2. Records, indexes and retained history

Each record stores its member ID, origin, status, inclusion position, next membership
sequence, optional activation data and accepted approvals. Origin is either the
configured seed descriptor/index or the complete signed enrollment operation.
Public key, evidence commitment and request nonce are derived from that origin.

Statuses are Pending, Active, Revoked and Expired. Pending requests have no
activation data. Active records have an activation height and positive ordinal.
Revoked records retain activation data if they were previously Active; a withdrawn
Pending request has none. Expired records originated as pending enrollments and
have no activation data. Active membership does not expire with its old application.

Every retained record participates in the key and evidence indexes, including
Revoked and Expired tombstones. Member IDs, public keys and evidence commitments
cannot be reused. Seeds consume record capacity and reserve their key/evidence
references. Runtime enrollment nonces are signed and nonzero; this profile does
not add global nonce uniqueness beyond the existing bootstrap configuration rule.
Fresh keys and fresh evidence from one person remain outside these equality checks.

An approval stores its reviewer public key, accepted height and complete signed
Approve operation. Its target must equal the containing record's enrollment ID;
the stored reviewer must equal the signed actor key. Each reviewer contributes at
most one retained approval per request. Duplicate approvals reject even with a
new otherwise-valid sequence. Accepted approvals are retained after withdrawal,
expiry and activation; they are not evicted to reclaim replay-protection capacity.

## 3. One atomic transition per candidate block

1. Preflight operation/count/byte budgets, then check context and parent
   association and derive candidate height with checked parent-height+1. All
   precede signature work or staging input-sized state.
2. Snapshot the parent's Active set. Stage the state and expire Pending requests
   whose `valid_until < candidate_height`, including on empty blocks. Expiry frees
   pending capacity, not records, approvals or identity indexes.
3. Process operations in their supplied order. Reuse B2 strict signature, context,
   canonical ancestry and inclusive height-window checks. Enroll requires unused
   ID/key/evidence and sequence 0, then creates Pending with next sequence 1.
4. Approve requires an existing Pending target, an actor Active both in the parent
   and staged state, the exact next actor sequence, a previously unused reviewer
   for this request, matching target expiry and no earlier `valid_from` than the
   request. The actor cannot approve their own request.
5. Withdraw requires the signer's own Pending or Active record and exact next
   sequence. Unknown, foreign, Expired and already-Revoked targets reject.
   Successful Approve/Withdraw checked-increment the actor sequence once; sequence
   exhaustion never wraps. There is no forced withdrawal or founder override.
6. After all operations succeed, count approval weight using reviewers Active in
   **both** the parent snapshot and final staged state. Activate eligible requests
   in original `(enrollment_height, operation_index)` order. New members cannot
   approve in this same block. A reviewer's withdrawal removes their weight from
   new activations, even when their approval came earlier or in an earlier block.
   Already-Active targets never cascade into revocation.
7. Assign each first activation its checked next ordinal and this block's height.
   Derive/check record counts and indexes. Set high-water to the maximum of the
   parent's high-water and the final Active count; transient intermediate counts
   do not increase it. Return the new staged snapshot only after all checks pass.

Any failure discards the entire candidate, including expiry, earlier approvals,
sequence changes, new indexes, counters and inclusion metadata. The original
registry remains usable and unchanged. Order is meaningful: Withdraw then Enroll
can free a pending slot, while Approve then Enroll cannot free it before finalization.

`total_activations` and old activation ordinals survive ordinary withdrawal.
These are development counters, not production monetary qualification. A future
reorg must restore the chosen branch's snapshot and counters, not the maximum over
every branch observed. Atomic mixed execution is now MB-02B behavior; actual
finite-branch acceptance/replay is MB-02C behavior. MB-03A storage reconstructs one
immutable selected branch from that complete history. It neither combines
counters from competing branches nor replaces the selection. Accepted-tip updates
and their failure recovery are next in MB-03B; canonical branch selection and
durable reorganization remain subsequent integration.

## 4. Bounded synthetic execution

Use the configuration's committed operation, membership-operation, new-enrollment,
per-operation, pending and total-record limits. The complete MRB2 byte budget is
`168 + sum(4 + signed_operation_length)` with checked arithmetic. The original B3
path contains only membership operations. MB-02B preflights the actual full mixed
list and its indexed membership stage also charges each omitted transfer position
as `4 + 243` bytes. Total operation count, membership count and enrollment count
are distinct limits. No declared count substitutes for checking the actual supplied
slice; the indexed projection alone grants no authority to omitted operations.

Two explicit B3 profile rules bound retained approvals without changing MCF2 bytes:

- At most **64 approvals per record**, equal to `MAX_APPROVAL_THRESHOLD`.
- At most **`config.limits().max_records` approvals across the entire registry**.
  This is a separate retained-approval count with the same configured upper bound;
  it is not subtracted from membership-record capacity.

The aggregate bound prevents record-count-squared approval storage. Both limits
include approvals attached to Active, Revoked and Expired records. Exact limits
are allowed; additions exceeding them reject without consuming sequences.
Full history/approval capacity still permits withdrawal and empty-block expiry.

These are finite development resource rules, not admission or availability
guarantees. At a full per-record approval budget, a request whose reviewers have
withdrawn may be unable to acquire replacement approvals. Exhausting aggregate
history can also stop further admissions. B3 never silently deletes history or
weakens the quorum to escape this condition. Admission access/recovery and larger
production capacities require explicit later policy/profile work.

## 5. Canonical MPS2 layout

**B4 implements the exact bytes frozen by B3**, with independent Python preimages
and complete Rust comparisons. MCF2, existing B2 signed messages, V1 state/logs and
legacy blocks remain unchanged. This is membership serialization, not an MRB2
block, combined ledger root or source of trusted canonical history.

All integers below are unsigned little endian. Hashes, keys and complete signed
operations preserve their existing raw order/encoding. Tags accept only listed
values; counts, exact lengths and end-of-input must be checked before allocation.

### Snapshot prefix — 90 bytes

| Order | Field | Bytes |
|---|---|---:|
| 1 | Magic `MPS2` | 4 |
| 2 | State format version, exactly 2 | 2 |
| 3 | Configuration hash C | 32 |
| 4 | Processed height | 8 |
| 5 | Active count | 8 |
| 6 | Pending count | 8 |
| 7 | Active high-water | 8 |
| 8 | Total activations | 8 |
| 9 | Retained approval count | 8 |
| 10 | Record count | 4 |

Append records strictly sorted by raw member-ID bytes. No duplicate IDs are allowed.

### Each record

| Order | Field | Bytes |
|---|---|---:|
| 1 | Member ID | 32 |
| 2 | Origin tag: Seed=0, Enrollment=1 | 1 |
| 3 | Seed body: configuration index u32, descriptor (96 bytes); or complete signed Enroll | 100 or 291 |
| 4 | Status: Pending=0, Active=1, Revoked=2, Expired=3 | 1 |
| 5 | Inclusion height | 8 |
| 6 | Inclusion operation index; seed configuration index at height 0 | 4 |
| 7 | Next membership sequence | 8 |
| 8 | Activation presence: absent=0, present=1 | 1 |
| 9 | If present: activation height u64, ordinal u64 | 0 or 16 |
| 10 | Retained approval count | 4 |
| 11 | Approval entries sorted by raw reviewer public key | count × 267 |

A seed record without activation payload is 159 bytes; an enrollment record without
activation payload is 350 bytes. Present activation adds 16 bytes. Seed states must
always retain their initial activation data even after self-withdrawal.

### Full-block inclusion-index bound (MB-02B)

For a runtime enrollment, `inclusion.operation_index` is the position in the full
ordered block, including transfers. Its structural MPS2 bound is now
`< config.limits().max_operations`. This explicitly extends the former
`< max_membership_operations` check: the membership limit caps a count, not a
position. For example, index 6 can be valid in a seven-operation block containing
only three membership operations. Seed positions retain their configuration-index
rules at height zero.

No field, serialized byte order or hash domain changes. Existing membership-only
fixtures remain unchanged. An older decoder may reject newly possible mixed
positions; the extension is not a claim of identical decoder acceptance sets.
MPS2 remains an untrusted snapshot until compared with independently executed
trusted state. MPH2 remains membership-only and cannot reconstruct a mixed ledger
by dropping transfers or renumbering membership operations.

### Approval entries and commitment

Each approval entry is reviewer key (32 bytes), accepted height u64, and the complete signed
Approve operation (227 bytes). It is 267 bytes. Duplicates/non-increasing reviewer keys reject.

Do not duplicate derived key/evidence/nonce fields. Derived indexes, implementation
containers and tip-hash provenance are not serialized. Rebuild indexes and verify
their consistency, counters, origins/IDs, statuses, positions, sequences, approval
bindings and activation metadata against trusted configuration and replay.

The commitment domain is exact ASCII with no terminator:

`H("MIRROR_MEMBERSHIP_STATE_V2" || canonical_snapshot_bytes)`

The largest per-record base is 366 bytes and approval entry 267 bytes. Thus a
conservative snapshot bound is `90 + max_records × (366 + 267)` under this
profile's aggregate approval cap, evaluated with checked arithmetic. The default
maximum is 41,484,378 bytes. This snapshot bound differs from `max_log_bytes`, which
bounds the separate diagnostic log; do not conflate the two formats or allow
input counts to allocate beyond the state limits.

Decoding internally coherent signed records does not prove their complete history,
canonical tip or freshness. Snapshots remain disposable caches bound to a trusted
validated block/root. B4 replay requires trusted configuration and candidate
ancestry. MB-02C supplies actual block/root authority by executing full MRB2 history;
MB-03A checked storage reopening uses the same complete replay with independent
network/checkpoint pins. It does not restore authoritative state from MPS2. A
peer-supplied snapshot must never gain authority merely because its individual
signatures verify.

## 6. Historical B3 acceptance evidence

B3 implementation `b6f1233d038901aa0a13c8ddac9afbc34418e846` passed exact-source
push CI [44317174000](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317174000),
job 134846238261; full logs and checkout inspected. The same source tree also passed
locally. There are 51 new B3 tests (45 integration + 6 internal arithmetic/invariant),
254 workspace tests total, and 147 personhood release reruns. Both existing independent
references, all three demos, documentation command (zero cases), and preservation
audits passed. At that checkpoint this document's MPS2 layout was a B4 byte contract,
not a tested state codec/root. B4 acceptance is recorded in section 8 below.

Relevant verified B3 cases are the runtime portions of T03/T06/T07/T11–T20/T22.
B4 root/codec/replay and mixed-ledger execution were still pending then. B4,
MB-02B and MB-02C subsequently implemented those components and actual finite-branch
acceptance/replay; durable network reorgs, device readiness and Gate A/B remain
pending. Consult CURRENT_STATUS.md for current source and acceptance evidence.
Tests/review are not a security certification.

The preceding evidence describes the B3 checkpoint. B4 adds the implementation
below; historical B3 counts are not current totals.

## 7. B4 codec and cache authority boundary

`MembershipStateV2::encode()` returns bounded canonical bytes; `root()` returns
their domain-separated SHA256d commitment. Both return an explicit error if the
private state invariants, checked size or output allocation fail. Neither encodes
the registry's external block-hash association. The same state bound to two
different synthetic hashes has identical bytes and membership root.

`UntrustedMembershipSnapshotV2::decode(config, bytes)` first enforces actual byte,
declared count, canonical order and aggregate approval budgets. It reconstructs
derived indexes, checks origins, IDs, inclusion positions, status/activation data,
counters and exact sequences inferable from retained outgoing approvals plus the
enrollment/withdrawal baseline. It checks signed metadata windows/profile and
activation timing, then verifies all retained enrollment and approval signatures.
No signature work precedes complete cheap parsing/accounting checks.

This wrapper exposes only encoding, root, height, record count and
`verify_against(&MembershipStateV2)`. It has **no conversion or accessor yielding
a runtime state, staged state or registry**. A compile-fail documentation example
guards private state access. `verify_against` compares every record, counter and
rebuilt index with independently trusted execution state; success does not return
an object reconstructed from the cache.

These checks cannot recover missing historical authorization. In particular, the
snapshot retains withdrawal status and resulting sequence, not the complete signed
withdrawal. A manufactured but internally coherent revoked record may pass cache
decoding while failing comparison to the trusted state. Root/signature/internal
consistency alone do not establish history, canonical ancestry or latestness.
Replay uses VerifiedBootstrap and the same B3 executor with the external trust
checks in REPLAY_V2.md. The separate MB-02C chain API supplies actual block/root
authority from pinned genesis and full accepted MRB2 history. It never restores
authoritative runtime state from this cache. MB-03A reopens an immutable selected
store through that full MRB2 replay path; it adds no cache-to-runtime conversion.

The independent standard-library reference `tools/reference_state_v2.py --check`
constructs five complete MPS2 snapshots from explicit synthetic record fixtures:
genesis, Pending, activation/Pending withdrawal, empty-block expiry, and Active/seed
withdrawal. It commits every status, retained approval and optional activation,
including present height zero. Operation signatures were generated separately with
Python Ed25519 using public test seeds; the checker needs no Python package.
Rust signs and executes the same history and compares every byte/root without
using a Rust-produced expected value. The same fixture independently defines MPH2
bytes, group commitments and trusted intermediate roots for replay verification.

Acceptance tests are in `tests/state_codec_v2.rs`, `tests/state_vectors_v2.rs` and
`tests/replay_v2.rs`. Consult the current checkpoint for actual test counts and CI.
All data are synthetic; no device capture, uniqueness decision, money, new consensus
or real chain genesis is produced. The original B4 vectors use membership-only
indices. MB-02B's additional independent ledger vectors preserve full mixed-block
positions; they do not reinterpret the old fixtures or extend MPH2 to transfers.

## 8. Historical B4 acceptance and current integration

Published implementation: `21e2272b5ac92c00e840d7aaa26a99570a5ec24f`.
Source tree: `31dfafa463935c640cd5cb31b88588575fc06d4b`.
Exact-source [push CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870)
passed; job 135155424668 full logs and exact checkout were inspected.
[PR CI 44404351436](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404351436)
also passed. The same source tree passed locally with Rust 1.98.0 and locked
dependencies before publication.

- **69 new B4 tests:** 26 codec, 10 independent-vector, 31 replay and two private
  work-budget arithmetic cases.
- **323 workspace tests passed:** 254 retained + 69 B4.
- **216 personhood release tests passed again.** These are reruns, not new tests.
- **One compile-fail documentation test passed**, protecting the cache's private
  runtime state boundary.
- All three independent references, all three synthetic examples, changed-file
  formatting and allocation/runtime-data/preservation checks passed.

MPH2 replay also enforces 4,096 groups and 1,048,576 cumulative state items, including
configuration clone entries, under its separate default 4-MiB log limit. The default
MPS2 maximum remains 41,484,378 bytes. Full details and trust assumptions are in
[REPLAY_V2.md](REPLAY_V2.md); reaching a limit rejects without erasing retained history.

This completed the **MB-01B membership component** at the B4 checkpoint. MB-02A
has since implemented TransferV2, unified operations and BlockV2 codecs/commitments
with independent vectors. MB-02B now stages accounts and membership together under
[LEDGER_STATE_V2.md](LEDGER_STATE_V2.md), with independent account/member/combined
commitments, retained zero accounts, separate sequence spaces and zero-fee/no-mint
conservation. Membership activation and high-water counters release no funds.

MB-02C now implements [actual block acceptance and complete MRB2 replay](CHAIN_V2.md)
for a bounded in-memory branch selected by independent manifest/checkpoint pins.
The implemented [MB-03A storage contract](STORAGE_V2.md) adds immutable selected-chain
initialization, exclusive reopening and shared read-only inspection. Every reopen
reexecutes the whole chain under independently supplied network/checkpoint pins.
**Next is MB-03B:** accepted-tip updates, generation authority, publication failure
reconciliation and restart behavior. Peer reconstruction and durable membership-aware
reorganization follow; the V2 node/API remains later work.
Diagnostic group commitments are not MRB2 operation roots, MPH2 remains
membership-only, and filtered indices must never replace original mixed-block
positions. Combined staging and caller-supplied hash binding alone still do not
establish actual block authority; ChainV2 performs the additional checks. No real
currency, biological uniqueness, anonymity or final human consensus is enabled
here. CURRENT_STATUS.md records current source and acceptance evidence.
