# MB-02A: V2 transfer and block encoding contract

Original checkpoint: **MIRROR-MB02A-001**. This document preserves the implementation
contract for the verified MB-02A component. Historical source 35ad0cf passed
exact-source CI 44428924399;
CURRENT_STATUS.md and the test matrix record scoped acceptance evidence.
No balance/membership execution, accepted
V2 chain, genesis, PoW proof, currency issuance or device support follows from a
value decoding or passing body-integrity checks.

Read with [the integration design](LEDGER_MEMBERSHIP_V1.md),
[configuration](CONFIG_V2_ENCODING.md), [membership operations](OPERATIONS_V2_ENCODING.md),
[membership state](MEMBERSHIP_STATE_V2.md), [diagnostic replay](REPLAY_V2.md), and
the implemented [MB-02B atomic ledger](LEDGER_STATE_V2.md). Ledger staging executes
a complete mixed body; the separate [MB-02C chain layer](CHAIN_V2.md) now establishes
actual block authority within a bounded in-memory branch. The implemented
[MB-03A storage layer](STORAGE_V2.md) initializes an immutable selected history and
reopens it through complete chain replay with independent network/checkpoint pins.
The separate [MB-03B append profile](STORAGE_APPEND_V2.md) now adds one-child
publication, failure fencing and clean-state reconciliation. Its MSU2 storage
envelope does not change any MRB2 operation, header or body encoding. MB-03C adds
[explicit interrupted-append recovery](STORAGE_RECOVERY_V2.md) using those same
canonical bytes and independently retained parent/candidate authority.
Current publication and acceptance evidence is recorded in CURRENT_STATUS.md.

## 1. Dependency and compatibility boundary

New types live in `mirror_core::v2`. `mirror-core` depends on `mirror-personhood`
for the existing immutable V2 configuration, membership operation and trusted
parent-view types. `mirror-personhood` continues to depend only on `mirror-crypto`;
there is no reverse dependency or cycle. This is one internal workspace dependency,
not an external dependency upgrade.

Legacy V1 core, node, storage, consensus and network paths remain unchanged. MCF2,
B2 signed membership bytes, MPS2 fields/domains and MPH2 bytes remain unchanged.
MB-02B explicitly extends MPS2's accepted runtime enrollment-index bound to
`< max_operations`, preserving positions in full mixed blocks; this is a structural
acceptance extension without new encoded fields. See [membership state](MEMBERSHIP_STATE_V2.md).
The unified decoder accepts Transfer=0; the existing membership-only decoder and MPH2
still reject it. No automatic MRB1 migration, genesis replacement or mixed-version
fallback is introduced.

All integers are unsigned little-endian. Hash and key fields use raw bytes with
no reversal. All listed domains are exact ASCII without a terminating NUL.
`H(x)` always means SHA256d(x). No raw or encrypted biometric archive is permitted.

## 2. TransferV2 and the unified operation envelope

All four operation kinds share the frozen 131-byte prefix from B2:

| Offset | Field | Length |
|---|---|---:|
| 0 | operation version, exactly 2 | 2 |
| 2 | kind: Transfer 0, Enroll 1, Approve 2, Withdraw 3 | 1 |
| 3 | configuration context C | 32 |
| 35 | anchor height | 8 |
| 43 | anchor block hash | 32 |
| 75 | valid_from, inclusive | 8 |
| 83 | valid_until, inclusive | 8 |
| 91 | actor public key | 32 |
| 123 | actor sequence | 8 |

Transfer's body and authentication follow:

| Offset | Field | Length |
|---|---|---:|
| 131 | recipient account address | 32 |
| 163 | amount in integer Nusa | 8 |
| 171 | fee in integer Nusa | 8 |
| 179 | Ed25519 signature | 64 |

The unsigned transfer is 179 bytes; the complete signed transfer is **243 bytes**.
Enroll/Approve/Withdraw remain exactly **291/227/227 bytes**. There is no second
outer tag, arbitrary payload, embedded account balance or appended metadata.

Signing hash: `H("MIRROR_OPERATION_V2" || complete_unsigned_operation)`.
Operation ID: `H("MIRROR_OPERATION_ID_V2" || complete_signed_operation)`.
The signature is Ed25519 over the 32-byte signing hash. Strict verification rejects
invalid/weak keys and invalid/noncanonical signatures via existing mirror-crypto.

The sender is derived from the signed actor key using the existing address function:
`H("MIRROR_ADDRESS_V1" || actor_public_key)`. The address namespace is deliberately
retained for compatibility with configured synthetic account addresses; the signed
V2 operation and configuration provide the new authorization context. The body
contains no independently chosen sender address. A raw recipient address needs no
signature/public-key decoding. No ownership of the recipient is asserted.

`TransferBodyV2` and `OperationHeaderV2` are construction inputs. SignedTransferV2
copies them into private immutable fields. Signing checks the signing key agrees
with the header and signs the requested intent; it does not spend money. `from_parts`
and structural decoding can represent invalid intents, including zero amounts or
nonzero fees, for later explicit validation.

`SignedOperationV2` is an ordered typed envelope over either SignedTransferV2 or an
unchanged SignedMembershipOperationV2. Its encoded value is exactly the complete
inner operation. Dispatcher checks cap/version/kind/exact length before fixed-size
reads. It delegates hashes, IDs and signature/context validation to the appropriate
type and preserves each membership operation byte for byte.

Transfer `validate_for_parent` enforces the same configuration, checked parent+1,
strictly past recent canonical anchor and inclusive authorization-window rules as
B2. It also requires positive amount and fee zero for the selected synthetic
profile, then strict signature verification. Its parent view must be stable locally
validated candidate ancestry. The method performs no wall-clock, device, storage,
matcher or network I/O.

Transfer sequence belongs to the sender ACCOUNT; membership sequences remain
separate. Current nonce equality/exhaustion, sufficient funds, account capacity,
self-transfer arithmetic and whole-ledger supply conservation are implemented by
the MB-02B executor, not this operation codec. Explicit zero allocations and
exhausted accounts remain in its commitments; membership never implies a balance.
A valid signature/context with amount or sequence u64::MAX is not permission to
mutate state. Membership eligibility is not silently added as a transfer rule.
The zero-fee profile does not settle production fees, rewards or D-01/D-04.

## 3. Exact BlockHeaderV2 and MRB2 layout

The header is **160 bytes**:

| Offset | Field | Length |
|---|---|---:|
| 0 | block version, exactly 2 (u32) | 4 |
| 4 | configuration context C | 32 |
| 36 | height | 8 |
| 44 | previous block hash | 32 |
| 76 | operations root | 32 |
| 108 | combined ledger root | 32 |
| 140 | timestamp | 8 |
| 148 | compact PoW target bits | 4 |
| 152 | nonce | 8 |

The header constructor uses version 2 and immutable private fields. It produces
an untrusted candidate value; arbitrary claimed roots, height and work fields are
not acceptance certificates. Header decode requires the exact size/version.
`block_hash = H("MIRROR_BLOCK_V2" || header_bytes)`.

A complete block is:

`MRB2[4] || header[160] || operation_count:u32 || repeated(operation_len:u32, signed_operation)`.

The count starts at byte 164; body records start at byte 168. An empty block is
**168 bytes**. No nested magic, padding, optional tags, implicit count or trailing
bytes are allowed. All four signed-operation kinds can occur in their original
order. Their positions are indices in the full list, including transfers.

## 4. Count-bound operation commitment

For each complete signed operation at index i:

`leaf_i = H("MIRROR_OP_LEAF_V2" || operation_len:u32 || operation_bytes)`.

The intermediate Merkle tree uses the existing Mirror convention, without changing
legacy code: empty tree is H(empty); a single leaf is its own root; pairs become
H(left || right); an odd last leaf at any level is duplicated before pairing.

`operations_root = H("MIRROR_OP_ROOT_V2" || operation_count:u32 || merkle_root(leaves))`.

The count wrapper distinguishes an odd-length list from a longer list formed by
duplicating its last operation, even when their bare Merkle roots coincide. Every
signature byte, operation field, position and count is covered. An operation ID is
not substituted for the domain-separated leaf. These roots differ intentionally
from the diagnostic MPH2 group commitment and legacy transaction root.

The standalone operations-root function obeys the same configured total/member/
enrollment/operation/complete-block byte budgets as the block constructor, before
allocating a leaf list or copying an input-sized operation list.

## 5. Bounds and validation stages

ConfigV2 is validated and immutable, but its existence alone is not proof of the
intended network manifest. The caller selects the network identity outside
untrusted input; MB-02C's manifest/chain APIs enforce its configuration and exact
genesis pin. Enforce max_block_bytes, max_operations,
max_membership_operations, max_new_enrollments and max_operation_bytes on construction,
encoding, decoding, commitment construction and body-integrity checking.

Use checked sums/products and exact kind sizes. Before constructing body vectors,
decode checks the full byte cap, magic, header version, count limits, actual remaining
bytes, every record length/kind and aggregate membership/enrollment caps. Reject
truncation, impossible declared counts, unknown kinds/versions, oversize and trailing
bytes. Empty input and plausible headers do not bypass full-block minimum size.
Encoding under a stricter configuration must rerun its budgets.

The API boundaries are deliberately separate:

| API | Establishes | Still unchecked |
|---|---|---|
| Header/block construction or decode | Exact format and applicable bounds | Signatures, commitments and all state/chain authority |
| Operation verify_signature | Strict signature over signed intent | Context, profile, balances, nonce and member rights |
| Operation validate_for_parent | Context, signature, anchor/window; transfer's static profile | Current account/member state and acceptance |
| Block verify_integrity | Bounds, header/operation C, operation root and signatures | Static transfer profile, ancestry, height linkage, time, target/work, ledger root, genesis and state execution |
| LedgerV2 stage_operations (MB-02B) | Atomic account/member execution, parent association, separate sequences, balances, conservation and computed roots | Actual header/genesis/work/time validation, claimed-root comparison and canonical block/chain authority |
| ChainV2 from_genesis/append/replay (MB-02C) | Pinned genesis, actual accepted branch ancestry, complete header/work/body/execution/root checks and atomic in-memory acceptance | Durable publication/recovery, fork choice/finality, network operation and real human admission |
| StoreV2 initialize/open_writer/inspect (MB-03A) | Immutable generation-zero storage, lifetime exclusive/shared ownership, bounded exact MNM2/MRB2 reads and full replay against independent network/checkpoint pins | Continued tip publication, update/failure recovery, branch replacement, peer synchronization, V2 node operation and real human admission |
| AppendStoreV2 initialize/append/reopen/reconcile (MB-03B) | Separate MSU2 profile, one accepted child per publication, continuous ownership, failure fencing and complete replay/resynchronization of clean externally selected states | Explicit recovery of interrupted staging, branch replacement, peer synchronization, V2 node operation, global freshness/finality and real human admission |
| AppendStoreV2 recover_interrupted/recover (MB-03C) | Explicit Complete/RetainParent for recognized one-child staging; independent request pins/exact child; full parent/child validation; bounded reads; retained exclusive ownership and final strict replay/synchronization | Missing/corrupt selected-marker or initialization recovery, unknown artifacts, rollback of a selected child, global fork choice/finality, durable branch replacement, live V2 networking and real admission |

No method in `mirror_core::v2` mutates balances, consumes sequences, activates membership,
publishes a canonical block, mines genesis, or opens a chain database. The separate
MB-02B state module stages ledger execution; MB-02C's chain and consensus modules
now validate actual work/header/genesis/ancestry and the claimed resulting root.
A signature-valid block with a deliberately claimed ledger root can pass body
integrity; this remains an explicit tested boundary. `ChainV2` additionally executes
the body and rejects a wrong claimed ledger root even when the work is valid.

## 6. Independent evidence and next work

`tools/reference_blocks_v2.py` computes transfer and block bytes/hashes independently
using Python's standard library. Public test signatures are separately generated
and verified with Python cryptography; the committed checker requires no extra
Python package and does not call Rust. Fixtures cover empty, single, odd, mixed,
reordered and duplicated-last lists, and field changes. Rust independently rebuilds,
signs and checks these values. Never use a failed Rust result as a new expectation.

MB-02B now stages accounts and membership atomically using the complete ordered
operation list. Its consuming indexed membership stage retains full positions and
charges omitted transfer slots at their exact 243-byte signed size plus length
prefix, while the ledger preflights the actual body. The original membership-only
stage_block remains compatible. MPH2 still cannot replay a filtered mixed list
without losing positions and account effects; it continues to reject Transfer=0.

[LEDGER_STATE_V2.md](LEDGER_STATE_V2.md) freezes account/economics/combined-state
commitments and zero-fee/no-mint execution. Successful staging is not acceptance of
a block or a supplied ledger-root field. [CHAIN_V2.md](CHAIN_V2.md) specifies the
implemented MB-02C manifest, actual block acceptance and complete MRB2 replay.
It preserves these frozen formats and validates each candidate using its own
accepted ancestry. [STORAGE_V2.md](STORAGE_V2.md) specifies the implemented MB-03A
immutable selected-chain initialization, exclusive reopening and shared read-only
inspection. It retains exact MNM2 and complete MRB2 bytes, reconstructs both ledger
components through `ChainV2`, and does not trust a decoded state cache.

[STORAGE_APPEND_V2.md](STORAGE_APPEND_V2.md) adds a separate MSU2 append envelope
around an unchanged MSS2 snapshot. Full accepted blocks are synchronized before
selection publication and in-memory advancement. Failed writes fence further
updates; clean selected states require independent generation/checkpoint pins and
full replay/resynchronization. These ordinary paths preserve and reject residual
staging. [STORAGE_RECOVERY_V2.md](STORAGE_RECOVERY_V2.md) adds a separate explicit
operation that may finish exact canonical candidate prefixes or retain the parent
while it is still selected. Unknown bytes and unsupported states remain rejected.
It changes no MRB2, MSS2, MSU2 or operation encoding.

**Next is MB-03D:** canonical development fork-choice specification and implementation,
before durable membership reorganization and peer reconstruction. The V2 node/API
remains later work; the existing node daemon and legacy paths have not been migrated.
Refer to CURRENT_STATUS.md for current source and acceptance evidence.
