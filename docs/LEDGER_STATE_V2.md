# V2 atomic ledger execution and commitments

Contract: **MIRROR-MB02B-001**, synthetic development profile only.
This extends the design in [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md)
and the formats in [CORE_V2_ENCODING.md](CORE_V2_ENCODING.md). Publication and
observed acceptance results belong in CURRENT_STATUS.md and MASTER_PLAN.md.
Membership transitions and MPS2 bytes remain specified in
[MEMBERSHIP_STATE_V2.md](MEMBERSHIP_STATE_V2.md).

The ledger stages accounts and membership together. It does not accept a real
block, establish canonical ancestry, issue rewards, prove human uniqueness or
process biometric readings. The separate MB-02C [chain acceptance layer](CHAIN_V2.md)
now validates actual blocks and complete finite MRB2 histories before publishing
this staged state. These component boundaries remain explicit below.

## 1. Immutable parent and candidate

`mirror_state::v2::LedgerV2` contains private account state and a
`MembershipRegistryV2`. `StagedLedgerV2` contains the complete candidate accounts
and staged membership state. Neither exposes an arbitrary state constructor,
mutable account map, membership setter or cache-to-runtime conversion.

`LedgerV2::bootstrap` accepts `VerifiedBootstrap`, requiring every configured seed
proof. It produces height-zero staged state using the exact immutable configuration,
its explicit synthetic allocations and the existing seed membership rules. There
is no special founder address, marketing allocation or implicit grant. Initial
synthetic supply is the checked sum already committed by configuration.

`LedgerV2::stage_operations` accepts a stable trusted `ParentChainViewV2` and the
complete ordered `SignedOperationV2` list. Successful staging returns both candidate
components together; failure returns no candidate. The immutable parent remains
unchanged in either case. Empty lists still advance height and expire pending
membership requests under the existing rules.

`StagedLedgerV2::bind_to_validated_block` attaches a caller-supplied hash and returns
the next immutable ledger. It is a trusted integration boundary, **not block
validation**. MB-02C's `ChainV2` first checks the actual block, pinned genesis,
parent, height, configuration, timestamp, target, work, operation commitment and
resulting ledger root, using only its accepted branch for ancestry. Its in-memory
acceptance and complete replay contract is [CHAIN_V2.md](CHAIN_V2.md). The original
ledger component example/tests continue to use explicitly synthetic parent hashes;
the separate chain example/tests use actual validated V2 block hashes.

The current block hash is external provenance and never part of the state root;
including it would create a cycle with the header's ledger-root commitment.

## 2. Original mixed-operation positions

Membership enrollment inclusion positions are indices in the full block. A transfer
at position zero means the following enrollment has index one, regardless of the
number of membership operations. Filtering transfers and renumbering the remainder
changes membership bytes, roots and potentially end-block activation order.

`MembershipRegistryV2::begin_indexed_block` receives a total full-block operation
count and an immutable borrowed slice of `(original_index, signed_membership_op)`.
Indices must be strictly increasing and less than the total count. Total,
membership and enrollment counts and complete byte budgets are checked before
candidate cloning, parent-Active-set allocation or signature verification.

This profile has exactly one omitted operation kind: Transfer=0, whose complete
signed size is **243 bytes**. The indexed membership projection charges its omitted
positions exactly, rather than treating them as zero-byte operations:

```text
complete_bytes = 168
              + (total_count - membership_count) * (4 + 243)
              + sum(4 + signed_membership_operation_length)
```

Checked arithmetic and configured max-operation/max-block sizes apply. This fixed
243-byte contract is shared with the core transfer format without introducing a
personhood-to-core dependency. Adding another omitted kind or variable transfer
size requires an explicit format/API review. The ledger separately preflights the
actual complete operation list with `validate_operation_limits_v2`, so projected
membership does not supply evidence that omitted transfers are valid.

The indexed builder is consuming and not cloneable. `apply_next(self, full_index)`
executes exactly the next planned membership operation at that full index. Any
error destroys that builder. `finish(self)` requires all planned membership
operations to have succeeded before end-block activation and invariant checks.
No intermediate registry or mutable state escapes. The ledger holds its account
candidate locally until this final membership result also succeeds.

The existing membership-only `stage_block` uses the same transition machinery
with contiguous indices and unchanged budget behavior. Existing MCF2, membership
operation, MPS2 and MPH2 bytes/domains and frozen reference fixtures are preserved.

### Explicit MPS2 index compatibility extension

For runtime enrollments, MPS2 structural validation now requires
`inclusion.operation_index < max_operations`, not `< max_membership_operations`.
The latter limits a count, not a position in a mixed block. Thus an index of six
can be valid with seven total operations and only three membership operations.
Seed positions retain their existing configuration-index rules.

No serialized field or hash domain changes. This is an explicit extension of the
accepted structural index range; an older decoder may reject these newly possible
mixed states. It does not turn MPS2 into trusted history. Decoding still returns an
untrusted snapshot wrapper with no conversion into executable state, and a root
must be compared with independently executed trusted state.

MPH2 remains membership-only and retains contiguous positions. It still rejects
Transfer=0. It cannot replay a mixed ledger by omitting transfers; complete validated
MRB2 replay is implemented by MB-02C's `ChainV2`. The separate implemented
[MB-03A storage layer](STORAGE_V2.md) uses that full replay when reopening an
immutable selected history, with independent network/checkpoint pins. Continued
tip updates now use the separate [MB-03B append profile](STORAGE_APPEND_V2.md),
which stages complete children and reconciles clean selected states by full replay.
Explicit interrupted-staging recovery remains MB-03C. The previous diagnostic
replay budgets remain unchanged.

## 3. Execution and error order

1. Preflight the complete body count, individual operation sizes, membership and
   enrollment quotas, and encoded block bytes, without cloning candidate state or
   verifying signatures. Construct only a bounded membership projection.
2. Preflight that projection and validate parent configuration, parent height,
   checked next height and exact canonical parent hash against stored provenance.
   Snapshot the parent's Active set, clone membership once and expire pending
   requests for the candidate height. Then clone the bounded account state.
3. Visit every original operation in order. Execute a transfer at its actual
   position or consume the next planned membership operation at that position.
   Each uses the same stable parent view for its context, anchor and window checks.
   A failing earlier transfer is not skipped to report a later membership error,
   and an earlier membership failure is not hidden by a later transfer error.
4. After the last operation, finalize membership using the existing parent-Active
   eligibility and final-state reviewer survival rules. Activate in original
   enrollment position order, update final-count high-water and check invariants.
5. Check account bounds and the checked balance sum against immutable initial
   synthetic supply. Return the complete staged result only after all checks pass.

Structural budget or parent-association errors precede per-operation execution.
Per-operation failures carry the full original index. End-block membership or
account invariant failures occur after ordered execution. There is no guarantee
that the earliest invalid signature outranks a structural budget failure.

Pending expiry, retained tombstones, history/approval budgets, independent membership
sequences and same-block review restrictions are preserved. No account/member state
is committed on failure, even after an earlier transfer, approval, withdrawal or
expiry succeeded inside the discarded candidate.

## 4. Account rules and disabled economics

An account has an unsigned 64-bit Nusa balance and an unsigned 64-bit next account
sequence. Addresses use the existing `MIRROR_ADDRESS_V1` derivation. Account and
membership sequences are separate even for the same signing key.

Every explicitly configured allocation is retained, including a zero balance.
An absent address reads as zero balance/sequence for lookup but is distinguishable
from an explicitly retained zero account. Receiving a positive transfer creates an
account if capacity permits. Once present, an account is never removed merely
because its balance reaches zero; its sequence and capacity use persist.

For each transfer, common context/window/anchor and strict signature validation,
positive amount and zero fee checks precede economic mutation. Require the exact
next account sequence and a checked sequence increment, then sufficient balance.
For a different recipient, debit and checked-add the credit, reserving capacity
when the recipient is absent. A self-transfer still requires a positive amount,
sufficient funds and the correct sequence; it changes only the next sequence.
It must not overwrite a debit with a credit or create/destroy funds.

Transfers do not require Active membership in this synthetic profile. No unapproved
rule prevents a pending, withdrawn or non-member key from spending its own account.
Membership approval or activation consumes no account sequence; a transfer consumes
no membership sequence. Tests explicitly distinguish these cases.

All arithmetic is checked. Account capacity, balance/sequence overflow or supply
invariant failure rejects the whole candidate. A recipient overflow cannot arise
from a valid conserved ledger whose total fits in u64, but the implementation still
checks it defensively. Private fault tests do not create a production state setter.

The invariant is `sum(all retained account balances) == initial_test_supply`.
There is no mint, fee burn, commission, mining reward, population issuance, bootstrap
payment or grant API. The user's 10-MRY bootstrap policy and ordinary 0.2-MRY
population allowance remain distinct unexecuted policy components; D-01 and D-04
are unresolved. Membership high-water counters do not release money.

## 5. Canonical commitment bytes

All integers below are unsigned little-endian. H is SHA256d. Domains are exact
ASCII without a length prefix or trailing NUL. Hashes and addresses are their
32 raw bytes, without display-byte reversal. These are commitment payloads, not
trusted disk snapshots or complete block encodings.

### Accounts

```text
account_payload = account_count:u32
               || for each address in ascending raw-byte lexicographic order:
                    address:32 || balance_nusa:u64 || next_account_sequence:u64
accounts_root = H("MIRROR_ACCOUNTS_V2" || account_payload)
```

Each record is exactly 48 bytes. Explicit zero allocations and exhausted accounts
are included. The empty payload is four zero bytes, not an empty string. Account
count and allocation arithmetic are bounded and checked before encoding.

### Membership and disabled economics

```text
membership_root = H("MIRROR_MEMBERSHIP_STATE_V2" || canonical_MPS2_bytes)
economics_payload = initial_test_supply:u64
economics_root = H("MIRROR_ECONOMICS_DISABLED_V2" || economics_payload)
```

MPS2 retains the exact existing field order, flags, retained signed operations,
approval ordering, processed height and counters. No runtime key/evidence indexes
or provenance tip hash are appended. Economics has no additional mutable field.

### Combined ledger

```text
ledger_payload = C:32 || processed_height:u64
               || accounts_root:32 || membership_root:32 || economics_root:32
ledger_root = H("MIRROR_LEDGER_STATE_V2" || ledger_payload)
```

The combined payload is exactly **136 bytes**. Configuration and height come from
the staged membership state, preventing separate account/member height inputs.
The root does not replace the block's independent operation commitment: different
valid histories can produce equal final account state, and a state root alone does
not establish the exact sequence of operations or the accepted block.

## 6. Verification and remaining integration

Independent standard-library Python calculations and committed synthetic vectors
must reproduce account bytes, membership bytes, component roots and combined roots
from explicit configuration and history. Rust reconstructs and executes those
operations independently. Existing reference files stay unchanged. Signature fixture
generation is independently verified; a failed Rust result never becomes a fixture.

Coverage includes full mixed indices, activation/expiry/withdrawal, separated
sequences, empty blocks, retained zero accounts, self-transfers, conservation,
capacity/exhaustion, actual original error order and rollback in both directions.
The synthetic example demonstrates execution and atomic failure, and explicitly
does not claim canonical block acceptance.

MB-02C now validates actual headers, exact manifest/genesis identity, work, accepted
ancestry and both roots before binding candidate provenance. Its complete MRB2
replay uses the same executor and a separately selected five-field checkpoint;
valid work over a wrong final ledger root rejects without changing either component.
See [CHAIN_V2.md](CHAIN_V2.md) for the finite resource profile and actual branch
authority, and CURRENT_STATUS.md for exact source/CI evidence.

The implemented [MB-03A storage contract](STORAGE_V2.md) initializes an immutable
accepted chain under single-writer ownership. Exclusive reopening and shared
read-only inspection require independently selected network and full checkpoint
pins and rebuild both ledger components by replaying every complete MRB2 block.
No cached account or membership state becomes a second authority. Exact source
verification and publication evidence are recorded in the checkpoint files.

The separate [MB-03B append contract](STORAGE_APPEND_V2.md) stages one complete
child through `ChainV2`, synchronizes block and selector publication, then advances
the accepted chain and its entire ledger together. Failure fences ordinary state
access and further writes; a clean selected state must be independently pinned,
fully replayed and resynchronized before a writer can resume. Cached account,
membership, population or resource counters never become recovery authority.

**Next is MB-03C:** explicit recovery of interrupted append staging, whose pending
or unexplained artifacts are currently preserved and rejected. Reviewed canonical
branch selection, durable membership reorganization and peer reconstruction follow;
the V2 node/API remains later work. In-memory acceptance and
bounded storage checks are not universal crash-safety or human-consensus
guarantees. Private matching, biometric capture, privacy and real admission remain
later gates.
