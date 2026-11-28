# B2: signed membership operations and contextual authorization

Checkpoint MIRROR-MB01B-B2-001. Source implementation acceptance requires the
exact CI evidence in CURRENT_STATUS.md. No membership execution, money or devices
are enabled by this module. Read with LEDGER_MEMBERSHIP_V1.md and CONFIG_V2_ENCODING.md.

Current integration update: **MB-02A implements TransferV2, the unified operation
envelope and MRB2 codecs/commitments in `mirror_core::v2`**; see
[CORE_V2_ENCODING.md](CORE_V2_ENCODING.md). This B2 membership contract remains
unchanged. B3/B4 implement standalone membership execution and diagnostic replay;
MB-02B now implements [atomic combined-ledger execution](LEDGER_STATE_V2.md).
**Next is MB-02C: actual V2 block acceptance and replay**. MB-02 remains IN_PROGRESS.
Exact-source acceptance and CI status are recorded in CURRENT_STATUS.md.

## Frozen bytes

Integers are unsigned little-endian; domains are ASCII without NUL. Prefix:

| Offset | Field | Bytes |
|---|---|---:|
| 0 | version = 2 | 2 |
| 2 | kind: Enroll=1, Approve=2, Withdraw=3 | 1 |
| 3 | validated configuration context C | 32 |
| 35 | anchor height | 8 |
| 43 | exact anchor block hash | 32 |
| 75 | valid_from inclusive | 8 |
| 83 | valid_until inclusive | 8 |
| 91 | actor public key | 32 |
| 123 | actor membership sequence | 8 |
| 131 | kind-specific body | 96 or 32 |

Enroll body: nonce[32], evidence commitment[32], evidence profile[32].
Approve body: complete target enrollment ID[32]. Withdraw body: own member ID[32].
A 64-byte Ed25519 signature follows. Complete sizes are 291, 227 and 227 bytes.
No variable payload, optional field or archive is supported. Transfer=0 is now
implemented by the MB-02A unified/Transfer decoders, with a 243-byte signed
envelope. It remains rejected by this membership-only decoder, including a
correctly sized Transfer envelope. MB-02A reuses the common prefix without changing
membership bytes or reinterpreting old signatures. The final prefix field is a
membership sequence here; Transfer uses its separate account sequence. Signing
either field does not consume or validate a current state sequence.

Signature message = SHA256d("MIRROR_OPERATION_V2" || complete unsigned bytes).
Operation ID = SHA256d("MIRROR_OPERATION_ID_V2" || complete signed bytes).
Enrollment ID = SHA256d("MIRROR_ENROLLMENT_ID_V2" || complete unsigned Enroll).
Thus changing the enrollment's lifetime, context or signer changes its target ID.
Signature changes change operation ID but not enrollment ID; an ID never proves
that its signature is valid. Approve and Withdraw have no enrollment ID.

## API boundaries

`OperationHeaderV2` and `MembershipActionV2` are construction inputs.
`SignedMembershipOperationV2` holds private immutable copies. `sign` only confirms
that the chosen key matches the header and signs the requested bytes. `from_parts`
and `decode` produce UNTRUSTED signed values. Decoder checks configured size cap,
version, membership kind and exact length before fixed-size field copying; it
performs neither signature nor state validation. Encoder has only fixed-size
inputs and can produce no larger record than 291 bytes; ConfigV2 guarantees its
single-operation limit is at least that large. No unbounded batch API is added.

`verify_signature` uses existing strict Ed25519 verification including weak-key
rejection. This proves possession of the signer key, not eligibility of a reviewer,
validity of a biometric claim, or acceptance of a membership change.

`validate_for_parent(config, parent_view)` checks, in deterministic order:
size; operation C; parent-view C; checked parent_height+1; strictly past anchor and
maximum age; non-inverted, non-overflowing inclusive span and maximum lifetime;
anchor before valid_from; inclusion height inside the window; enrollment's nonzero
nonce/evidence, configured evidence profile and sequence zero; exact canonical
ancestor lookup; strict signature. There is one bounded ancestor lookup and no
wall-clock, network, device or matcher I/O in the implementation.

A valid ConfigV2 is not an authoritative network manifest. ParentChainViewV2 is a
TRUSTED interface: implement it using the locally validated, stable candidate
parent ancestry and pinned config/genesis. Its methods must describe one immutable
view for the whole validation/block. Do not construct it from a peer's unchecked
hash map or advertised height. During competing-branch validation, use THAT branch.
B2 supplies synthetic test views, not a live chain resolver or genesis pinning.

## Stateful boundary: standalone B3 execution and future mixed-ledger execution

B2 does not consume or compare current membership sequences for Approve/Withdraw,
check target existence/status/lifetime alignment, verify signer eligibility or
self-withdrawal ownership, detect reuse against history, expire records, count
approvals, activate humans, or execute blocks. Even a correctly signed approval
with sequence u64::MAX or an unknown target can pass B2 context checks: B3 MUST
reject it against registry state before mutation. This boundary is explicitly
tested, not a claim that such a transition is acceptable. Enrollment sequence zero
is a stateless necessary condition only; unused-key/history checks remain B3.

Authorization lifetime is not evidence of fresh sensor capture. Fresh-key,
fresh-evidence duplicate humans remain undetectable without the private admission
system. Public keys/reviewer links remain linkable. Opaque fields are not proof
against malicious covert data; no raw/encrypted biometric archives are authorized.

## Evidence

`crates/mirror-personhood/tests/operations_v2.rs` covers exact encodings; every truncation; oversized,
trailing, wrong-kind/version and legacy input; bytewise message/signature tampering;
wrong signing/weak keys; domain separation; context and changed-seed configuration;
parent/anchor/window edges including u64 boundaries; required evidence metadata;
request and operation IDs; and deferred stateful checks. No test calls a live node.

`tools/reference_operations_v2.py` independently builds preimages/hashes/IDs with
Python struct/hashlib. Signature constants were independently generated using
Python cryptography Ed25519 with public test seed [1;32]; Rust tests generate and
strictly verify those exact signatures. Fixture: tests/vectors/operations_v2.txt.
CI requires no new Python package: the reference check is standard-library only.
The local reference generation does not imply local Rust execution.

At the historical B2 checkpoint, T04 membership lengths/decoding and T06
cryptographic target binding were partial rows; Transfer and stateful reuse checks
were later work. B3/B4 subsequently supplied the standalone membership-state and
diagnostic-replay subcases. MB-02A now adds 37 operation, 33 block and 10 independent
vector tests in `crates/mirror-core/tests`, covering Transfer and all four kinds in
the unified envelope without changing this membership-only API. The current
coverage map is in [MEMBERSHIP_TEST_MATRIX.md](MEMBERSHIP_TEST_MATRIX.md).

T09/T10 contextual checks still rely on trusted parent views. Actual candidate
ancestry and accepted-block replay remain MB-02C, with durable recovery in MB-03.
T07 transfer sequence-state safety is now implemented in MB-02B's ledger executor;
signing an account-sequence field or passing MB-02A body integrity alone does not
consume it or validate balances. Mixed execution uses positive zero-fee transfers,
separate account/member sequences and no additional membership spending restriction.
Passing body integrity still does not imply that separate context/execution checks
ran. Neither Gate A nor Gate B follows from these component results.
