# V2 membership state and block execution

Checkpoint: **MIRROR-MB01B-B4-001**. Synthetic development profile only.
This document preserves the B3 state model/execution and its frozen MPS2 layout.
B4 adds the canonical codec, commitment and untrusted cache boundary. Grouped
diagnostic replay is specified in [REPLAY_V2.md](REPLAY_V2.md). Exact published
acceptance evidence is recorded in CURRENT_STATUS.md. See [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md),
[CONFIG_V2_ENCODING.md](CONFIG_V2_ENCODING.md), [OPERATIONS_V2_ENCODING.md](OPERATIONS_V2_ENCODING.md)
and [MEMBERSHIP_TEST_MATRIX.md](MEMBERSHIP_TEST_MATRIX.md).

No runtime membership transition proves biological uniqueness. Fingerprint/ECG
capture and private duplicate matching remain external, later work. Only synthetic
commitments/public development keys belong in examples. There is no monetary API,
grant, founder privilege, anonymous credential or live V2 block network in B3.

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

Successful staging returns a new `StagedMembershipStateV2`; the previous registry
is unchanged. `bind_to_validated_block` associates a staged snapshot with its block
hash only **after the trusted outer chain executor accepts that block**. This is a
trusted integration boundary, not a block-validation function. It does not verify
PoW, a header, genesis pinning or a committed state root. B3 tests use synthetic
block hashes; MB-02 must implement the actual validated association.

The current block hash is deliberately excluded from committed membership state:
the future block header commits to the ledger/membership root, so including that
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

1. Check context and parent association, derive candidate height with checked
   parent-height+1, and preflight operation/count/byte budgets before signature
   work or staging input-sized state.
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
every branch observed. Actual ledger/reorg execution remains MB-02/03.

## 4. Bounded synthetic execution

Use the configuration's committed operation, membership-operation, new-enrollment,
per-operation, pending and total-record limits. The planned MRB2 byte budget is
`168 + sum(4 + signed_operation_length)` with checked arithmetic. B3 has only
membership operations; MB-02 must apply the combined budget to the full mixed block.
No declared count substitutes for checking the actual supplied slice.

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
ancestry; MB-02/03 supplies actual block/root/storage authority. It must not accept
a peer-supplied snapshot merely because its individual signatures verify.

## 6. Historical B3 acceptance evidence

B3 implementation `b6f1233d038901aa0a13c8ddac9afbc34418e846` passed exact-source
push CI [44317174000](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317174000),
job 134846238261; full logs and checkout inspected. The same source tree also passed
locally. There are 51 new B3 tests (45 integration + 6 internal arithmetic/invariant),
254 workspace tests total, and 147 personhood release reruns. Both existing independent
references, all three demos, documentation command (zero cases), and preservation
audits pass. This document's MPS2 layout remains a B4 byte contract, not a tested
state codec/root. See CURRENT_STATUS.md and the test matrix for scoped evidence.

Relevant verified B3 cases are the runtime portions of T03/T06/T07/T11–T20/T22.
B4 root/codec/replay evidence, mixed-ledger execution, actual network reorgs, device
readiness and Gate A/B remain pending. Tests/review are not a security certification.

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
checks in REPLAY_V2.md. MB-02/03 still must supply actual block/root/storage authority.

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
or real chain genesis is produced. Inclusion indices here are membership-only;
MB-02 must preserve full mixed-block positions when adding transfers.
