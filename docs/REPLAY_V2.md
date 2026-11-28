# B4: bounded grouped membership replay

Original checkpoint **MIRROR-MB01B-B4-001**. MPH2 is a synthetic diagnostic format for
reconstructing the membership component. It is not an accepted blockchain block,
a peer synchronization protocol, a durable latest-tip store, or a proof of real
human uniqueness. Read with [membership execution and MPS2](MEMBERSHIP_STATE_V2.md),
[signed operations](OPERATIONS_V2_ENCODING.md), and [configuration](CONFIG_V2_ENCODING.md).

MB-02B now implements the separate [atomic mixed ledger](LEDGER_STATE_V2.md).
MPH2 remains membership-only: its bytes, group commitments, contiguous membership
positions and acceptance rules are unchanged. A successful mixed ledger stage is
not an accepted block or an MPH2 replay. MB-02C now implements the separate
[actual-block acceptance and full MRB2 replay API](CHAIN_V2.md) for finite in-memory
branches; it does not change MPH2 or make an MPS2 cache authoritative. The
implemented [MB-03A storage layer](STORAGE_V2.md) initializes an immutable selected
history and reopens it by replaying every complete MRB2 block under independent
network/checkpoint pins. The separate [MB-03B append profile](STORAGE_APPEND_V2.md)
adds one-child publication and checked reconciliation of clean selected histories.
MB-03C adds [explicit interrupted-append recovery](STORAGE_RECOVERY_V2.md) under
independent parent/candidate pins and exact requested child bytes. None uses this
diagnostic transcript as storage authority or alters its frozen bytes.
The [MB-03D comparator](FORK_CHOICE_V2.md) accepts only complete validated ChainV2
branches, not this transcript or an MPS2 cache, and performs no diagnostic replay.
CURRENT_STATUS.md records current publication and acceptance evidence.

## 1. Independent trust inputs

`MembershipReplayV2::decode` returns an untrusted immutable transcript. Parsing
does not check signatures, grant reviewer rights, or recover authoritative state.
`MembershipReplayV2::replay` requires all of the following separately:

- `VerifiedBootstrap`: the exact validated configuration and every verified seed
  proof. The log cannot select another configuration or supply seed authority.
- `ReplayChainViewV2`: one stable history produced by an independently validated
  chain. At every height from zero through the requested final height it supplies
  the exact block hash, exact membership-operation group commitment, and expected
  membership state root. Its configuration must match the verified bootstrap.
- `ReplayExpectationV2`: a caller-selected tuple of configuration, final height,
  tip hash, and membership root. The caller must select this checkpoint outside
  the untrusted transcript; using fields from the transcript itself is circular.

Do not implement the trusted view using a peer's advertised hashes/roots or the
same diagnostic file that is under examination. A malicious view can authorize a
malicious history; the trait is an explicit integration trust boundary. B4 uses
synthetic independently executed fixtures. MB-02C's separate `ChainV2` now validates
actual MRB2 headers, roots and pinned genesis using its own accepted ancestry.
It does not implement an automatic MPH2 `ReplayChainViewV2` adapter. MB-03A storage
reopening uses actual MRB2 replay with separately selected pins, not a diagnostic
view inferred from this log. MB-03B now applies the same complete replay to append
publication prechecks and clean-state reconciliation, with separately pinned local
generations. MB-03C applies actual-chain replay and acceptance to both independently
selected endpoints before its explicit recovery actions. MB-03D now compares
complete accepted branches under independent identity; no diagnostic transcript
chooses that authority or supplies a work score.

The explicit expected tuple prevents an old valid prefix from satisfying a newer
requested checkpoint. It does not discover which checkpoint is latest. An
explicitly requested historical checkpoint can still be replayed deliberately.

## 2. Why exact operation commitments are necessary

A state root alone does not commit to every operation's exact bytes and ordering.
Two independent approvals may commute and produce the same sorted approval
archive. Two valid signed withdrawals with different authorization windows may
produce identical final membership records, because the record retains the
withdrawn status and sequence rather than the withdrawal message itself.

Therefore each trusted height includes an exact operation-group digest in addition
to its block hash and membership root. Replay rejects changed bytes or ordering
even when the resulting MPS2 state would be identical. The digest is a diagnostic
membership-only commitment; it is **not** the MRB2 ordered-operation Merkle
root implemented in [CORE_V2_ENCODING.md](CORE_V2_ENCODING.md). The current format
cannot replay mixed transfer/membership blocks by simply
dropping transfers: doing so changes membership inclusion indices and may change
ledger execution. MB-02B supplies indexed execution within the full mixed body,
but does not change this replay representation. Actual mixed-chain replay uses
MB-02C's complete validated MRB2 histories, preserving their original positions
and combined state semantics. That API requires independently selected C, genesis,
height, tip and ledger root. MB-03A/B use that API when reopening exact stored
MNM2/MRB2 history; MB-03C also uses it during explicit recovery, retaining the same
external selection requirement. Do not reinterpret MPH2 as such an integration
or replace its diagnostic group roots
with MRB2 roots.

MPS2's MB-02B structural enrollment-index bound is explicitly extended to
`< max_operations` from `< max_membership_operations`, without changing snapshot
fields or hash domains. That allows full mixed positions; it does not provide
missing transfer history to MPH2 or authorize a snapshot as runtime state.

## 3. MPH2 canonical bytes

Integers are unsigned little endian. Hashes and signed operations use raw bytes.
No padding, ignored optional data, trailing bytes, or unknown version/kind is
accepted. MCF2, B2 operation bytes, MPH1 and legacy blockchain formats are unchanged.

### Prefix: exactly 74 bytes

| Offset | Field | Bytes |
|---|---|---:|
| 0 | ASCII magic `MPH2` | 4 |
| 4 | Format version, exactly 2 | 2 |
| 6 | Configuration hash C | 32 |
| 38 | Claimed genesis block hash | 32 |
| 70 | Group count N | 4 |

The transcript starts from the verified bootstrap, not from an arbitrary snapshot.
Genesis is represented by the prefix and external bootstrap/metadata, not by an
ordinary group. N=0 is a valid genesis-only diagnostic transcript.

### Each group: 44 bytes plus operations

| Order | Field | Bytes |
|---|---|---:|
| 1 | Height | 8 |
| 2 | Claimed block hash at that height | 32 |
| 3 | Operation count | 4 |
| 4 | For each operation: signed-byte length, then exact signed bytes | 4 + length |

Group heights are exactly `1, 2, ..., N`, in that order. Height zero, gaps,
duplicates, reorderings and overflow-like values reject. Empty groups are retained
and consume the same group budget: they advance height and can expire pending
requests. Do not merge groups, discard empty groups, or flatten into MPH1's
immediate-activation semantics.

Signed operation lengths are the frozen B2 values: Enroll=291, Approve=227,
Withdraw=227. Other lengths/kinds, including Transfer=0 (implemented in the separate
core V2 envelope), reject in MPH2.

### Exact group commitment

`SHA256d("MIRROR_MEMBERSHIP_REPLAY_GROUP_V2" || C || height:u64 || count:u32 || entries)`

Each entry is `signed_length:u32 || complete_signed_operation`. The ASCII domain
has no terminator. Count, order, signature bytes, height and configuration all
contribute to the digest. The claimed block hash is external provenance and is
checked separately against the trusted view.

At genesis the trusted group commitment is this same function with height=0 and
zero operations. The bootstrap state root must also match independently trusted
genesis metadata. A matching genesis hash alone is insufficient.

## 4. Bounded construction and decoding

All direct constructors, encoders and decoders enforce the same applicable
configuration budgets before copying input-sized containers. They enforce:

- The configured `max_log_bytes` for the whole encoded transcript; default 4 MiB.
- At most **4,096 groups**, including empty groups, in this diagnostic profile.
- Configured operation count, membership-operation count, new-enrollment count,
  and signed-operation byte limits for every group.
- The B3 prospective block charge `168 + sum(4 + operation_length)` against
  `max_block_bytes`, even though MPH2's diagnostic group header itself is smaller.

Counts must fit actual remaining bytes before count-sized reservation: at least
44 bytes for every group and at least 231 bytes for every declared operation.
Exact kind-specific lengths and subsequent group minimum sizes remain checked.
Arithmetic uses checked additions/multiplications, and allocation failure is
reported where explicit reservations are performed. No attacker-declared u32/u64
count is used as an unchecked allocation size or iteration endpoint.

The MPS2 snapshot limit is separate: `90 + max_records * (366 + 267)` under the
aggregate approval budget. A snapshot does not acquire the log's smaller 4-MiB
limit, and the log cannot borrow the larger snapshot budget.

### Replay work budget

Small empty groups can repeatedly scan a large retained registry. Byte/group
limits therefore do not by themselves provide an adequate execution budget.
The diagnostic replay also charges at most **1,048,576 cumulative state items**:

1. Charge bootstrap seed record count plus the number of configuration bootstrap
   descriptors and synthetic allocation entries before building/hashing genesis.
2. Before each B3 stage/clone, charge current record count plus retained approval
   count plus the number of Enroll and Approve operations in the proposed group,
   plus the configuration bootstrap-descriptor and allocation counts again.

This includes prospective records/reviews that the transition may add and the
configuration vectors that the immutable-state clone copies at each step. A large
zero-value allocation list is still charged even though no money changes during
membership replay. Checked accumulation rejects overflow and an addition exceeding the
budget; an exact-limit charge succeeds. The charge is a work proxy, not a precise
CPU-time or memory measurement. It adds no enrollment privilege or monetary rule.

These finite limits make MPH2 a bounded diagnostic transcript, not storage for
unbounded whole-chain history. Exceeding a budget returns an error; it never
truncates the history, deletes tombstones/approvals, lowers quorum, or claims a
partial result is current. MB-03A's immutable store and MB-03B's separate append
store preserve the complete-chain budgets; neither streams MPH2 into authority or
enlarges this diagnostic profile. MB-03C's separate recovery retains those limits
and adds its own finite residual-input and repeated-read allowances. Larger
production profiles require their own later design and acceptance.

## 5. Replay procedure and failure behavior

1. Recheck transcript bounds and exact configuration/bootstrap equality. Require
   the expected final height to equal N before any history lookup. No loop runs
   to an unbounded caller-supplied expected height.
2. Read each trusted height exactly once and cache the metadata. Check claimed
   block hashes and exact group commitments, including empty genesis. Require the
   expected tip/root to equal the final trusted metadata.
3. Bootstrap via `MembershipRegistryV2::bootstrap(VerifiedBootstrap)`. Compare the
   reconstructed MPS2 root with the trusted genesis root before binding its hash.
4. Execute each group through B3's only bounded staged transition. The cached
   parent view exposes only heights at or below the current parent; it cannot
   make a future cached block a valid operation anchor. B2 signatures, anchors,
   windows and B3 rights/sequences/lifecycles are rechecked during execution.
5. Compare the complete resulting MPS2 root at **every** height, then bind that
   accepted candidate to its independently trusted block hash. All intermediate
   registries remain private to the replay call.
6. Return the final registry only after the entire requested history succeeds.

An invalid signature or transition fails even if the supplied operation-group
digest matches those exact invalid bytes. A trusted digest is not an exemption
from execution rules. Any late mismatch/failure returns no registry; the verified
bootstrap, transcript and caller's prior registry remain unchanged for retry.
There is no database, file, network, matcher or device I/O in this operation.

Replay reproduces exact records, original inclusion positions, activation heights
and ordinals, permanent key/evidence/ID reservations, next sequences, approval
archives, statuses and final population counters. It does not infer biological
uniqueness, anonymity, human consensus, economic qualification or money issuance.

## 6. Acceptance coverage

`tests/replay_v2.rs` independently constructs trusted fixtures using B3 execution
and checks full equality after replay. Its cases cover lifecycle/tombstones,
empty-block expiry, exact byte layouts and group commitment inputs, all
truncations, trailing data, invalid kinds/versions/context, huge counts/lengths,
height gaps/order, exact/+1 byte/group/count budgets, intermediate metadata,
genesis binding, caller-selected freshness, signatures/anchors/sequences,
same-block eligibility, immutable rejection and retry.

Two regressions specifically establish why an operation digest is mandatory:
reordering commuting approvals and replacing a valid withdrawal window produce
the same MPS2 root but fail with `GroupCommitmentMismatch`. Negative execution
tests assert the relevant error rather than passing because a preceding unrelated
check failed. Metadata caching and no-history-lookup for impossible heights are
also checked. A public replay regression checks 65,536 synthetic zero-value
allocation entries with tiny empty groups, accepts the last in-budget transcript,
and rejects the next group before another expensive configuration copy. Private
runtime tests cover work-budget exact/+1/overflow arithmetic.

Independent fixed state/group preimages and hashes are in the B4 reference fixture
and `tools/reference_state_v2.py`; `tests/state_vectors_v2.rs` checks the Rust
implementation against them. Repository checkpoint files record the actual final
test counts, publication commit and CI result. These tests are component evidence,
not an external security audit or proof of later block/disk integration. MB-03A's
separate implemented storage boundary is specified in [STORAGE_V2.md](STORAGE_V2.md);
its source verification and publication evidence belong to the current checkpoint.
The separate [MB-03B contract](STORAGE_APPEND_V2.md) adds bounded accepted-tip
publication, failure fencing and clean-state replay/resynchronization. The separate
[MB-03C contract](STORAGE_RECOVERY_V2.md) adds explicit recovery of recognized
one-child staging, requiring full parent/child execution under independent pins
and final clean replay/synchronization. Unknown bytes remain preserved and rejected.
The [MB-03D comparison contract](FORK_CHOICE_V2.md) uses accepted block hashes and
normalized fixed-target scores, preserving the whole selected branch without mutation.
Next is **MB-03E: durable membership-aware branch replacement**, with reviewed
publication/recovery rules, followed by fresh-peer reconstruction. No V2 node
service, real human admission or monetary issuance follows from this diagnostic API.
