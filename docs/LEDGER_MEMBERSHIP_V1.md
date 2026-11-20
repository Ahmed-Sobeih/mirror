# Membership-ledger integration — design revision 1

Original checkpoint: **MIRROR-MB01A-001**. Task: MB-01A.
Original status: **DESIGN DELIVERABLE, not implemented consensus**.
Original audited source: `db2861930cc0f01f735ccfba80a4f530f4d1ddd6`.

Current implementation update: **MB-01B/B1–B4 supply standalone membership;
MB-02A supplies codecs/commitments; MB-02B implements atomic combined ledger
execution/commitments; MB-02C implements pinned actual block acceptance and
complete finite MRB2 replay; MB-03A now implements immutable selected-chain
initialization, ownership and checked reopening; MB-03B adds a separate append
profile with one-child publication, failure fencing and clean-state reconciliation;
MB-03C adds explicit recovery of independently authorized one-child staging**.
[MEMBERSHIP_STATE_V2.md](MEMBERSHIP_STATE_V2.md) records B3 execution, explicit
retained-approval budgets and the implemented MPS2 commitment/cache boundary.
[REPLAY_V2.md](REPLAY_V2.md) specifies bounded membership-only MPH2 grouped replay.
Historical B4 source
`21e2272b5ac92c00e840d7aaa26a99570a5ec24f` passed
[CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870).
MB-02A implements TransferV2, the unified operation envelope and BlockV2
encoding/commitments with independent vectors; the frozen contract is
[CORE_V2_ENCODING.md](CORE_V2_ENCODING.md). The implemented combined state contract
is [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md); actual in-memory branch acceptance is
[CHAIN_V2.md](CHAIN_V2.md). The implemented immutable storage contract is
[STORAGE_V2.md](STORAGE_V2.md); the separate append contract is
[STORAGE_APPEND_V2.md](STORAGE_APPEND_V2.md). Its separate explicit recovery contract
is [STORAGE_RECOVERY_V2.md](STORAGE_RECOVERY_V2.md). **Next is MB-03D: canonical
development fork-choice specification and implementation**, before durable membership
reorganization and peer reconstruction. MB-03 remains in progress.
Staging a ledger alone does not establish block authority. The MB-02 in-memory
component is verified; durable/node integration, finality and Gate A/B remain
unfinished. The historical design-status statements below describe the original
MB-01A checkpoint; current publication and acceptance evidence is in CURRENT_STATUS.md
and the acceptance matrix.

This specifies the development integration design, not mainnet rules. `V1` in
this document's filename means design revision 1. The proposed wire formats are
**V2**, because MRB1/MPH1 already exist. Proposed constants below are explicitly
small-network development limits, not new user-approved economic or admission
policy. Follow [DECISIONS.md](DECISIONS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).

## 1. Source audit and selected integration path

Current APIs and the identified gaps are recorded in [THREAT_MODEL.md](THREAT_MODEL.md).
The standalone MembershipRegistry and the account-only legacy ChainState retain
their separate roots. Legacy V1 Block remains transfer-only. MB-02A's BlockV2 can
carry typed transfers and membership operations together; MB-02B's new LedgerV2
executes the complete ordered body into atomic combined state. MB-02C's ChainV2
accepts actual blocks only after checking pinned identity, ancestry, work and
executed roots. No legacy transfer payload is reinterpreted as an
enrollment and no historical state root changes meaning.

**Engineering choice:** add an explicitly selected synthetic development profile,
`membership-dev-v2`, with typed V2 operations, a combined ledger root and its own
configuration, genesis, protocol version and data directory. Preserve V1 APIs and
fixtures while V2 is built in new modules. Do not silently upgrade old databases.

| Input | Legacy V1 path | V2 library path; live transport integration remains planned |
|---|---|---|
| MRB1 blocks, version-1 header/transfer | Unchanged; existing tests/data preserved | Reject, never reinterpret |
| MPH1 standalone logs and signatures | Existing library/demo only | Reject as V2 events; no import granting rights |
| MRB2 blocks, V2 signed operations | Reject | Full bounded decode and state validation |
| MPH2 diagnostic logs | Reject | Replay only with trusted V2 config/chain anchors; not authoritative storage |
| Existing Alice/Bob development keys | Public fixtures, not founder credentials | Never converted into real users or grants |
| Fingerprint/ECG recordings or archives | No enrollment payload | Off-chain only; no archive field |

Both implemented V2 storage profiles refuse initialization in a nonempty legacy
directory; in-memory ChainV2 itself opens or migrates no directory. Its implemented
network manifest pins config hash and exact genesis hash. A different genesis
nonce is not accepted merely because it also satisfies the easy target. Existing
mainnet language in V1 comments is not evidence of a launched mainnet. Do not merge
or migrate merely to publish this design.

## 2. Configuration binding without a circular genesis dependency (R01–R03)

Use the existing SHA256d and strict Ed25519 verification primitives; no new crypto
algorithm is introduced here. All integers in canonical data are unsigned little
endian; hashes/keys retain their raw byte order. Every variable-length sequence has
an explicit count/length checked before allocation. No platform usize, float, JSON
ordering or wall-clock lookup belongs in signed/committed state.

`ConfigV2` commits to: format/profile versions; a nonzero 32-byte network domain;
fixed development PoW bits and genesis timestamp; approval threshold; a 32-byte
synthetic evidence-profile identifier; every execution/resource limit; the ORDERED
bootstrap descriptors; and sorted explicit synthetic account allocations.

A bootstrap descriptor is public key (32), enrollment nonce (32), evidence
commitment (32). Reject duplicate keys/nonces/evidence, zero nonce/evidence and
invalid counts. Order is intentional and committed; no invisible first-founder
privilege. Account allocations are sorted by address, unique even for zero
balances, and have checked sums. They are test fixtures, not economic issuance.

Compute `C = H("MIRROR_CONFIG_V2" || canonical_config_bytes)` BEFORE signing seeds.
Seed i signs `H("MIRROR_SEED_V2" || C || u32(i) || descriptor_i)`.
Seed member ID is `H("MIRROR_SEED_ID_V2" || C || u32(i) || descriptor_i)`.
Signatures, derived member IDs, roots, and the eventual genesis hash are NOT inputs
to canonical_config_bytes. This removes the dependency cycle. The constructor
recomputes C; the separate `verify_bootstrap` and `NetworkManifestV2::new` boundaries
verify every seed proof. They never accept an arbitrary caller assertion that a
supplied context hash represents a different configuration.

Initial roots then determine the genesis header; the deployment manifest pins its
exact valid nonce/hash. Reopening or syncing must verify BOTH C and pinned genesis.
All ordinary V2 operations sign C and a canonical-chain anchor. Seed proofs use
the separate seed domain and cannot be submitted as runtime operations.

Threshold-one/one-seed startup remains a synthetic bootstrap profile only. Neither
it nor a multi-seed example settles D-02, the production verifier policy. No seed
has a continuing administrator or forced-revocation key.

## 3. One typed operation envelope (R04–R08)

Proposed canonical unsigned prefix, in this exact order:

| Field | Bytes | Meaning |
|---|---:|---|
| operation_version | 2 | Exactly 2 |
| kind | 1 | Transfer=0, Enroll=1, Approve=2, Withdraw=3 |
| config_hash | 32 | Exactly C |
| anchor_height | 8 | Height of referenced canonical ancestor |
| anchor_hash | 32 | Exact ancestor hash |
| valid_from | 8 | Inclusive block height |
| valid_until | 8 | Inclusive block height |
| actor_public_key | 32 | Authorized signer |
| actor_sequence | 8 | Checked account or membership sequence |

Prefix = **131 bytes**. Append the fixed body below, then a 64-byte signature over
`H("MIRROR_OPERATION_V2" || unsigned_prefix || body)`.
There are no optional/trailing fields and no generic application payload.

| Kind | Body | Total signed bytes |
|---|---|---:|
| Transfer | recipient[32], amount u64, fee u64 | 243 |
| Enroll | applicant_nonce[32], evidence_commitment[32], evidence_profile[32] | 291 |
| Approve | target_enrollment_id[32] | 227 |
| Withdraw | own_member_id[32] | 227 |

Enrollment member ID is `H("MIRROR_ENROLLMENT_ID_V2" || entire_unsigned_enrollment)`;
therefore approval of that ID commits to the entire request, evidence and window.
Operational ID is `H("MIRROR_OPERATION_ID_V2" || complete_signed_operation)`.
Changing a field, kind, context, target or window invalidates the signature.
Public-key parsing is not signature validation. Retain strict signature checks.

Sequence spaces are separate: Transfer uses the sender account nonce; membership
operations use one membership sequence per registered actor key. Enroll requires
sequence 0 and an unused key, then stores next_sequence=1. Seeds start at 0.
Approve/Withdraw require exactly next_sequence and checked increment it only on
success. A rejected operation/batch consumes neither sequence. Same-verifier
approvals of one target remain single-count even with a new sequence. Sequence
exhaustion rejects, never wraps. Key recovery/rotation is disabled until specified.

V2 transfers are signed V2 messages, NOT wrapped V1 signatures. For this integration
profile, fee must be zero and issuance/rewards are disabled. This is a temporary
test profile, not a decision that production transfers will be free. Require
positive amount, sufficient balance and checked arithmetic; preserve self-transfer
behavior and nonce changes. No account balance is inferred from membership.

## 4. Anchor and authorization lifetime (R09–R12)

For candidate block at height h, require anchor_height < h, that anchor is in the
candidate PARENT's canonical ancestry, and h-anchor_height <= max_anchor_age.
An anchor on a competing branch fails even at the same height. In a chain reorg,
validate anchors against THAT candidate ancestry, not the old canonical tip.

Require anchor_height < valid_from <= h <= valid_until and a window span
valid_until-valid_from+1 <= max_validity_blocks. Use checked arithmetic. This
limits authorization replay; it does NOT prove when a biological reading was
captured. No device-clock claim or raw waveform influences deterministic execution.

For an enrollment, valid_until also expires its Pending record. Pending records
with valid_until < h become Expired during candidate block preparation, including
empty blocks. Expired/Revoked keys, IDs and evidence remain tombstones. A normal
active membership does not expire merely because its original application expired.

An approval must target a still-Pending request. Its valid_until must equal that
request's expiry, and valid_from may not precede the request's valid_from. Store
its verifier, accepted height, and complete signed approval or a canonically bound
representation sufficient for replay. Accepting an approval during its window does
not make an expired application activatable later. Withdraw is independently
signed/windowed and affects only the signer's Pending or Active membership.

Exact IDs/keys/evidence reuse prevention is not biological duplicate detection.
Fresh evidence for the same human is explicitly outside this synthetic profile.
The future off-chain matching receipt/assignment rules require D-02/D-05 and
MB-05/09. Do not accept a simulated matcher as real uniqueness evidence.

## 5. Block execution and eligibility timing (R13–R19)

The implemented chain derives its parent state, candidate height and stable
parent-chain view from its own accepted blocks, never a caller-chosen arbitrary
active-member count, peer-supplied ancestry or clock. MB-02B implements the
deterministic state-staging portion described in [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md).
It has no public unbounded `apply` bypass. MB-02C adds complete actual header, work,
operation-root and claimed-ledger-root acceptance under [CHAIN_V2.md](CHAIN_V2.md).
The following is a design summary; that contract freezes the precise preflight,
validation and publication order.

1. Preflight complete bytes/counts and cumulative resource limits; check format,
   context, actual parent/height, timestamp, configured target/work and operation root.
2. Clone/stage the complete ledger; snapshot the parent's Active member set.
3. Expire Pending requests deterministically at h. Apply ordered operations using
   checked sequence/state rules. Rejected operation => reject the entire block.
4. Approvers must be Active in the PARENT snapshot AND still Active in staged state.
   An enrollment activated by this block cannot approve another enrollment in it.
5. Collect approvals but defer activations until all operations have passed. At
   finalization, count distinct approvers Active both in parent and post-operation
   state. A verifier withdrawing anywhere in this block contributes no activation
   approval in this block, even if their approval came first. Do not retroactively
   deactivate memberships activated in earlier blocks when a verifier withdraws.
6. Activate qualifying Pending requests in order of their original canonical
   (enrollment_height, enrollment_operation_index). Store activation height and
   first-activation ordinal with checked increments. The order is reproducible,
   not a claim of fair global first-1,000 selection; D-01 still defines grant rights.
7. Compute end-of-block counts/roots. Compare the committed ledger root, then
   publish the complete candidate state only after all validation succeeds.

Transitions (all authorization/signature/window checks remain mandatory):

| Previous | Operation/condition | Result |
|---|---|---|
| Absent | Valid Enroll, unused key/evidence, room under limits | Pending |
| Pending | New valid Approve, below threshold | Pending |
| Pending | Sufficient surviving approvals at block end | Active |
| Pending | Expiry before block h | Expired tombstone |
| Pending/Active | Valid self Withdraw | Revoked tombstone |
| Active | Another approval for this target | Reject |
| Expired/Revoked | Approve, Withdraw or same-key re-enroll | Reject |
| Any | Forced revocation, recovery, death, inheritance, mint | Unsupported |

End-of-block `active_count` is derived from records; `pending_count` is also derived
and checked. `active_high_water = max(parent.active_high_water, final.active_count)`.
Transient intermediate operations cannot inflate that committed record. This
V2 DEVELOPMENT counter is not a final monetary qualification rule. `total_activations`
and each original activation ordinal remain recorded after ordinary withdrawal.
A reorg restores the selected branch's counters, not the maximum over every branch
ever observed. Actual qualified monetary population/maturity is still D-01.

## 6. Resource limits (R20–R22)

Proposed default synthetic profile; commit these values in C, validate ranges at
construction, test exact-limit and limit+1, and version any change. These are not
throughput promises or limits on eventual humanity:

| Limit | Proposed development value |
|---|---:|
| Complete encoded block | 1,048,576 bytes |
| All operations / membership operations per block | 256 / 256 |
| New enrollments per block | 64 |
| Single encoded operation | 1,024 bytes (exact kind lengths still required) |
| Pending memberships / total membership records | 1,024 / 65,536 |
| Account records | 65,536 |
| Bootstrap entries / approval threshold | <=1,000 / 1..64 and <= seed count |
| Anchor age / validity window | 32 / 256 blocks |
| Diagnostic log | 4,194,304 bytes |

Count tombstones toward total records. Expiry frees pending capacity, not historical
identity protections. Reject capacity exhaustion explicitly; never evict replay
protection to make space. These bounds constrain damage but do not solve denial of
service: a malicious enrollment flood can fill capacity. Admission quotas, fair
access and resource economics remain MB-04/05, not a hidden fee here.

B3 refinement: retained approvals are separately capped at 64 per record and at
configured max_records across all records. This closes quadratic approval-history
growth without changing MCF2 fields. Expired/revoked/active records retain their
approvals, and full capacity still permits withdrawal/expiry. These explicit
synthetic-profile budgets can halt new admissions; they do not guarantee progress.

B4 refinement: MPS2 snapshot bytes have a separate checked cap of
`90 + max_records * (366 + 267)`, default 41,484,378 bytes. MPH2 keeps the configured
4-MiB default log limit and adds diagnostic limits of 4,096 groups and 1,048,576
cumulative state items. The latter includes configuration descriptors/allocations
cloned at bootstrap and each staged block, retained records/approvals and prospective
record/approval additions. Exceeding a budget rejects; it does not evict history,
truncate a replay or lower an admission threshold. These are diagnostic work
limits, not final network size, throughput or monetary decisions.

Enforce limits on decoding, encoding, direct executor input, disk reads and peer
messages, before reservations/clones or expensive signature batches. An error
leaves accounts, membership, roots, counters and sequences unchanged.

## 7. Header, body and unified commitment (R23–R27)

V2 header fields, now frozen and implemented by MB-02A:
`version:u32 | C:32 | height:u64 | previous_hash:32 | operations_root:32 |
ledger_root:32 | timestamp:u64 | pow_bits:u32 | nonce:u64` = **160 bytes**.
Block bytes: `MRB2[4] | header[160] | count:u32 | repeated(u32 length, operation)`.
Empty block size = **168 bytes**. Reject unknown versions/kinds, bad exact lengths,
count/size inconsistencies and trailing bytes. Hash the header as
`H("MIRROR_BLOCK_V2" || header)`. MB-02A implements these byte/hash rules, but only
carries the supplied target and ledger-root fields. MB-02B computes the resulting
ledger state/root. MB-02C now checks actual PoW, timestamp, parent/height linkage,
pinned genesis and the header's claimed ledger root before accepting the block.
The design retains fixed-target PoW for temporary development; it is not the final
one-human-one-vote protocol.

Let leaf_i = H("MIRROR_OP_LEAF_V2" || u32(len(op_i)) || complete_signed_op_i).
`operations_root = H("MIRROR_OP_ROOT_V2" || u32(count) || merkle_root(leaves))`.
The count commitment is required: the current odd-leaf duplication convention
must not let lists of different lengths share a commitment unnoticed.

The implemented MB-02B combined ledger contains accounts, membership, and
disabled-economics state together. The following root computations and balance
conservation are specified in [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md); they are
executor behavior, not authority supplied by the MB-02A codecs.
`accounts_root = H("MIRROR_ACCOUNTS_V2" || u32(count) || sorted(address,balance,nonce))`.
Account records are sorted by raw address bytes and encode 32-byte address, u64
balance and u64 next account sequence. Explicit zero allocations and exhausted
accounts remain included; absent and retained-zero accounts are distinguishable.
Membership canonical encoding commits C, all counters, and sorted records including
request data, status, inclusion location, next membership sequence, activation
height/ordinal, and sorted approval evidence. Presence of optional values uses
explicit tagged encodings; absent is not confused with ordinal/height zero.
Derived key/evidence indexes must agree with records; replay rebuilds them.
`membership_root = H("MIRROR_MEMBERSHIP_STATE_V2" || canonical_membership_bytes)`.
`economics_root = H("MIRROR_ECONOMICS_DISABLED_V2" || initial_test_supply:u64)`;
check sum(account balances)==initial_test_supply under the zero-fee/no-mint profile.
`ledger_root = H("MIRROR_LEDGER_STATE_V2" || C || height:u64 || accounts_root ||
                 membership_root || economics_root)`.

MB-01B has frozen and independently verified full field ordering/optional encodings
for its V2 membership module. MB-02A does the same for Transfer and full block
bytes/operation commitments. MB-02B adds independently calculated mixed-ledger
preimages and vectors in `tools/reference_ledger_v2.py`; publication and acceptance
evidence belongs in CURRENT_STATUS.md.
Domains listed here are exact ASCII, no
terminating NUL. Any revision is explicit and updates the design and vectors.

Block creation, reception, replay, and competing-chain validation use the SAME
executor. Never update a side membership database independently of block acceptance.
Invalid late membership operations must roll back earlier transfers, and vice versa.

B4 MPH2 has a separate exact group digest because state roots alone do not commit
to every operation history. It is **not** the MRB2 Merkle operation root specified
above. Do not filter transfers out of a mixed block to feed B3/MPH2: membership
inclusion indices must remain positions in the original complete block. MB-02B
implements a consuming indexed membership stage that preserves those positions
while the ledger executes the complete ordered body. MPS2's accepted runtime
enrollment-index range is explicitly extended from `< max_membership_operations`
to `< max_operations`; serialized fields and domains are unchanged. Older decoders
may reject newly possible mixed positions. MPH2 remains membership-only. MB-02C
implements actual validated mixed-block replay; MB-03A/B now use that same replay
to reconstruct selected stored state without a cache-authority shortcut.

## 8. Persistence, transport and dependency boundaries (R28–R32)

Implemented dependency direction: crypto <- personhood V2 <- core V2, with state
V2 composing core and personhood. Chain V2 composes those layers and consensus V2.
Core reuses personhood's typed operations/configuration/parent-view interface;
personhood does not depend on core. State V2 now stages accounts+membership
atomically. Chain V2 now validates parent/config/state and actual block authority;
later node integration coordinates storage/transport. Do not create a
core/state/personhood dependency cycle.

MB-02 implements deterministic in-memory mixed execution, actual finite-branch
acceptance and full replay. MB-03A now persists an immutable selected MRB2 history
with explicit metadata and continuous directory ownership. MB-03B adds a separate
versioned append profile with one-child publication, failure fencing and complete
replay/resynchronization of clean selected states. Both require independently
retained network and full checkpoint pins; profile 2 additionally requires its
local generation. Neither a checksum nor a greater generation establishes latestness.
Snapshots remain disposable caches tied to a trusted block/root. Interrupted
pending/tail artifacts still reject through ordinary opening without deletion or
promotion. MB-03C separately accepts an explicit retained parent/candidate/exact-child
request and fully validates the parent and child before narrowly scoped completion
or parent retention. Unknown data remains untouched. Both profiles preserve the
stable locked root rather than swapping the directory. The legacy directory-swap mechanism is not claimed
to be a globally atomic transaction.

Planned transport integration must use a distinct V2 handshake (magic MIR2,
protocol version 2) with C, pinned genesis, height/tip and ledger root. Separate
listener/config/data selection; no fallback
from a rejected V2 message to legacy handling. BlockData includes height plus
bounded MRB2 bytes; advertised height/root is untrusted until independent execution.
Validate headers/kind sizes before reading/allocating bodies. Complete-message
and session budgets, multi-peer operation and anti-slow-peer tests belong to MB-04.

ChainV2 can validate independent branches of the same config/genesis, but supplies
no fork-choice or reorganization API. The existing legacy fixed-target fork choice
does not thereby become a V2 policy. Planned pre-MB-06 integration retains explicitly
development-only branch selection after complete validation; its durable publication
belongs to MB-03. MB-06 must prevent any reorg across finalized history. This design
does not make PoW a source of equal human voting power.

## 9. Privacy, real admission and economics stay gated (R33–R35)

Evidence commitments are opaque, randomized off-chain references; no raw/encrypted
fingerprint or ECG archive, public URL to one, personal ID, device secret, or matcher
database belongs in these operations. Fixed-size fields limit payload shape but
cannot prove a malicious sender never embeds private bytes. Client validation,
redaction and private-data tests remain required. Public member/reviewer keys and
relationships remain linkable; anonymity is not implemented.

Human reviewers and private matching will later supply session/snapshot-bound,
versioned evidence under explicitly authorized policies. This spec does NOT give
arbitrary "no match" receipts admission authority. Concurrent fresh-key enrollment
of the same person and consent/retention remain MB-05/09/12 gates.

No founder premine, marketing reserve, bootstrap payout, population mint, fee
redistribution or reward is enabled by V2 integration. Keep the separate agreed
monetary components; D-01/D-04 still block a combined executable monetary policy.

## 10. Historical implementation handoff and current acceptance

[Test matrix](MEMBERSHIP_TEST_MATRIX.md) maps R01–R35 to named planned assertions.
At the original MB-01A checkpoint, all V2 runtime cases were NOT_IMPLEMENTED.
Static design review was not evidence that those behaviors existed. The historical
slice plan below has now been completed for the standalone membership component;
MB-02A adds full mixed block representation and MB-02B adds atomic ledger
execution. MB-02C now adds pinned actual block authority and full finite-branch
replay; MB-03A/B now add selected storage and bounded append publication. MB-03C
adds explicit recognized-staging recovery. Canonical branch selection, durable
reorganization, peers and node integration remain unfinished.

Original MB-01B slices:

- B1: new versioned module, canonical configuration/seed context and bounded rules;
  preserve V1 vectors. Reject different seed lists/order/config with the same signatures.
- B2: signed typed membership operations, sequences, anchors and height windows.
- B3: bounded staged execution, expiry, parent eligibility and block-end activation;
  roots/codecs and tombstones. No direct unbounded mutation escape hatch.
- B4: full regression/adversarial suite and synthetic demo, debug/release CI.

The original MB-02 plan adds TransferV2, BlockV2 and the unified ledger executor with
mixed-batch atomicity and exact genesis validation. Its codec/commitment and atomic
executor slices and MB-02C actual branch acceptance are now implemented.
MB-03 adds durable multi-node integration.
Do not combine these milestones into an unreviewed rewrite or alter real policy
merely to make a test pass.

Historical B4 acceptance: 323 workspace tests (254 retained + 69 B4), 216 personhood
release reruns, one compile-fail documentation test, three independent references
and three synthetic examples pass. These scoped results complete MB-01B, not this
entire 35-rule integration design.

Historical MB-02A source is `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382`.
Local acceptance passed 403 workspace tests (323 retained + 80 new: 37 operation,
33 block and 10 independent-vector cases), one documentation test, 121 core release
reruns and 216 personhood release reruns plus its documentation case. Four
independent references and four synthetic examples passed.
[Exact-source push CI 44428924399](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399)
and [PR CI 44428936866](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428936866)
passed; exact checkout and full push-job logs were inspected. See CURRENT_STATUS.md
for the detailed acceptance record.
The precise next slice is **MB-03D**: canonical development fork-choice specification
and implementation, before durable branch replacement or peer reconstruction.
MB-03A now supplies explicit immutable initialization, cooperative ownership and
full checked reopening under independent network/checkpoint pins; its contract is
[STORAGE_V2.md](STORAGE_V2.md). MB-03B's separate
[append profile](STORAGE_APPEND_V2.md) stages one accepted child, synchronizes block
and selector publication, fences failed updates, and reconciles only clean selected
states by full replay and resynchronization. MB-03C's separate
[recovery operation](STORAGE_RECOVERY_V2.md) requires independent parent and candidate
pins, the exact complete child and an explicit action. It validates both histories,
completes only authorized byte prefixes or retains the still-selected parent, and
performs final strict replay/synchronization without trusting caches. It cannot
roll back a selected child or repair unknown data. MB-02B's atomic execution is
specified in [LEDGER_STATE_V2.md](LEDGER_STATE_V2.md); MB-02C's finite in-memory acceptance and
complete replay are specified in [CHAIN_V2.md](CHAIN_V2.md). CURRENT_STATUS.md records
current source and acceptance evidence. An MRB2 body-integrity check or a successful
staged ledger alone still does not establish actual block authority; ChainV2 adds
the complete checks. Reviewed canonical branch selection, durable membership
reorganization, peer/node operation and final human consensus remain later work.
Zero-fee/no-mint synthetic execution introduces no
founder allocation, marketing reserve, population issuance or bootstrap payout.
