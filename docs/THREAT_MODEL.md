# Mirror membership integration threat model

Checkpoint: **MIRROR-MB01A-001**. Scope: MB-01A source review and V2 design.
Source inspected at `db2861930cc0f01f735ccfba80a4f530f4d1ddd6`; PR #1 comments
were reviewed and contained checkpoint/roadmap notes, not an independent security
review. This is an engineering review, not an audit certification.

Read with [the proposed integration specification](LEDGER_MEMBERSHIP_V1.md) and
[the test matrix](MEMBERSHIP_TEST_MATRIX.md). No issue below is represented as fixed
merely because a mitigation is documented. At the original design checkpoint,
no V2 acceptance test had run.

The preceding sentence describes the original design checkpoint. Current B1–B4
and MB-02A evidence is in CURRENT_STATUS.md and MEMBERSHIP_TEST_MATRIX.md. The updates below
record component-level mitigations without rewriting historical findings or
claiming that live ledger, consensus or storage integration already exists.

## Historical B3 implementation update

Source `b6f1233d038901aa0a13c8ddac9afbc34418e846`, exact-source push CI44317174000
and PR CI44317184524 passed. B3 addresses F02/F04/F05 for the separate V2 in-memory
executor: bounded direct inputs and retained approval history, parent-snapshot
eligibility, deferred activation, same-block withdrawal effects, final population
high-water and complete candidate discard on failure. B2 authorization windows are
now enforced during state execution, with pending expiry and permanent tombstones.
Legacy V1 is preserved, not retrospectively relabeled hardened.

The supplied parent view and bind_to_validated_block remain trusted integration
boundaries, not proofs of actual chain validity. Finite approval/history budgets
can halt admissions and do not solve availability. A valid old log/snapshot is not
authoritative latest state. At that checkpoint, B4 roots/replay, MB-02 ledger
authority, MB-03 durable reorgs, human consensus and real private biological
matching remained unfinished. B4 component progress is recorded next.
No finding is declared globally closed or externally audited from these tests.

## Historical B4 implementation update

Source `21e2272b5ac92c00e840d7aaa26a99570a5ec24f` passed
[push CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870)
and [PR CI 44404351436](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404351436).
The exact push checkout and full logs were inspected. There are 69 new B4 tests
(26 codec, 10 independent vector, 31 replay and two private arithmetic cases),
323 workspace tests total, 216 personhood release reruns and one compile-fail
documentation test. Three references, three examples and preservation checks pass.

Canonical MPS2 commits the complete membership records and counters while excluding
external tip-hash provenance. The decoder bounds input, reconstructs indexes,
checks metadata/accounting and verifies retained signatures. Its untrusted wrapper
cannot expose a runtime state or registry. Internally coherent tampered withdrawal
status can still pass structural decoding; only comparison against independently
trusted state or full verified replay establishes the intended history.

MPH2 retains empty blocks and exact group boundaries. Replay re-executes B3 and
requires independently trusted block hashes, exact ordered-group digests and every
intermediate membership root, plus an external final C/height/tip/root expectation.
Tests include commuting approvals and alternate valid withdrawal bytes that yield
the same state root: they still reject against the trusted group digest. A valid
old prefix does not satisfy a newer expected checkpoint. The provider of these
trusted values remains an explicit caller boundary, not a deployed chain validator.

The default log cap is 4 MiB, distinct from the default 41,484,378-byte snapshot
bound. Diagnostic replay additionally caps groups at 4,096 and cumulative state
items at 1,048,576, including configuration clones and empty-block scans. Bounds
are checked before input-sized copies or repeated state execution; failures return
no final registry. This constrains work but does not guarantee honest admission,
continued availability, fixed execution time or unbounded whole-chain replay.

F06/F07 actual block/ledger authority, F08 durable publication, F09 network session
limits and F10 private biological matching remain unresolved in their stated
scope. The exact diagnostic group digest is not the future MRB2 operation root.
Future mixed blocks must retain transfers and original inclusion positions; they
cannot be replayed by filtering operations into the current membership-only log.
No biometric uniqueness, anonymity, currency issuance or human consensus follows
from these passing component tests. MB-01B is complete only in that bounded scope.

## MB-02A implementation update

Published source `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382` adds 80 tests
(37 operation, 33 block and 10 independent-vector cases). Local verification passed
403 workspace tests and one documentation test, 121 core release tests, and 216
personhood release tests plus its documentation case. Four reference checks and
four synthetic examples passed.
[Exact-source push CI 44428924399](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399)
and [PR CI 44428936866](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428936866)
passed. The push checkout and full job log were inspected; CURRENT_STATUS.md
records the exact acceptance evidence.

The new `mirror_core::v2` component adds Transfer and a unified four-kind operation
envelope, a 160-byte header and bounded MRB2 bodies. Strict signatures and separate
domains bind Transfer fields. The block operation root binds count, order, record
lengths and complete signed bytes, including the odd-leaf duplicate-last case.
Construction, encoding, decoding and root calculation enforce complete byte,
operation, membership and enrollment limits. Independent Python vectors guard the
actual Transfer/header/block preimages and preserve existing membership bytes.
See [CORE_V2_ENCODING.md](CORE_V2_ENCODING.md) and test-matrix section 7.

This supplies F06's missing mixed-body representation and integrity component;
**it does not close F06 or F07**. `verify_integrity` checks bounds, configuration,
the operation root and signatures. It does not execute balances/membership,
consume account sequences, validate the supplied ledger root, check actual
parent/height/timestamp/PoW or pin an authoritative genesis. Even positive amount,
zero-fee and authorization-window checks belong to the separate operation-context
method and must be invoked by the future executor. Its parent view remains a
trusted caller boundary. Body integrity cannot authorize a canonical state update.

F08/F09 durable publication and network-session limits remain unresolved; an
in-memory decoder's bounds are not disk-read, session or global availability
guarantees. F10 private biological matching and F11 the final fee/reward economy
also remain open. Original mixed-operation positions are now represented and
committed, but B3/MPH2 must not be fed a transfer-filtered list as though it were
equivalent execution. **MB-02B combined-ledger execution is next; MB-02 overall
remains IN_PROGRESS.** No launch gate, human uniqueness, anonymity, currency
issuance or device readiness is established by this component milestone.

## 1. Assets and trust boundaries

Protect canonical account balances/nonces; membership status/approval authority;
bootstrap and rule configuration; activation/population counters; future grant
uniqueness; the authoritative durable tip; availability; private evidence and keys.

Treat network frames, disk files, supplied config files, applicant evidence,
reviewer statements and advertised peer heights/roots as untrusted until the
appropriate checks complete. A locally trusted deployment manifest chooses a
network; it is not replaced by whichever config a peer advertises. Seed people
are an explicit initial trust assumption, not proven unique by self-signatures.

The initial development profile assumes synthetic evidence, a single explicitly
configured network, a bounded dataset and nonvaluable test balances. It makes no
claim of honest majority of real humans, private biometric matching, or finality.
Until MB-06, hash-rate dominance remains possible in the existing PoW foundation.

A device can provide measurements; an application binds a session; a matching
operator can report a comparison; humans attest to review; nodes enforce encoded
rules. None of those steps proves the truth of all previous steps. In particular,
valid signatures are not proof of biometric uniqueness or informed consent.

## 2. Observed source findings and bounded reproduction recipes

These are source-derived observations or explicitly labelled design inferences,
not claims of an executed exploit against a deployed network. Existing funds and
local chain files have not been accessed or modified.

| ID | Observation at audited source | Consequence / reproduction recipe | Planned owner |
|---|---|---|---|
| F01 | `RegistryRules::new` hashes network+threshold only; ordered seeds are supplied separately to `MembershipRegistry::new` | With identical network/threshold, reuse an applicant request across two different valid seed configurations. Rules/signing context does not distinguish them. Bootstrap membership differences can still change state roots; the missing protection is signature context binding. | MB-01B B1, R02–03 |
| F02 | `apply` and `apply_batch` have no record, pending-count or event-count limits; codec bounds one log to 4 MiB | Feed many distinct self-signed requests through repeated direct calls or many valid logs. The log limit is not a total registry/mutation budget. | MB-01B B3, R20–22 |
| F03 | Requests and approvals contain no chain anchor or height expiry | Preserve a valid request and approve it much later; source has no freshness deadline to reject it. Authorization expiry will not itself validate sensor freshness. | MB-01B B2, R09–12 |
| F04 | Approval immediately marks a member Active; the next event checks the mutable registry | With threshold 1, A approves B and B approves C in the same batch. This is allowed by the current library, but is unsuitable for the proposed parent-snapshot eligibility model. | MB-01B B3, R13–15 |
| F05 | `population_high_water` updates at each individual approval | Activation followed by withdrawal in one batch can record an intermediate peak. Current code has no block boundary; new end-of-block development accounting must not be confused with final monetary qualification. | MB-01B/02, R18 |
| F06 | V1 `ChainState` commits accounts only; `Block` carries signed transfers only; `apply_transaction` rejects other kinds | The library is not live personhood state. Simply adding an API or storing a second log would leave consensus blind to membership. | MB-02, R23–27 |
| F07 | V1 `from_persisted_blocks` checks genesis fields/PoW/state but receives no pinned exact expected genesis hash | Another valid nonce for the same genesis fields is not excluded there. V2 must pin exact genesis and bootstrap/config context, including on disk reopen, not rely only on peer Hello comparisons. | MB-02, R01/R03/R28 |
| F08 | `BlockStore` assumes one writer, uses `fs::read`, and performs two directory renames during replacement | No enforced writer lock, no read-byte cap, and errors after publication can leave in-memory and durable outcomes ambiguous. Recovery based on directory existence is not proof of complete old/new atomic publication on all filesystems. | MB-03, R29–30 |
| F09 | TCP receiver reads declared body before semantic header validation; timeouts are socket read/write timeouts; node accumulates candidate blocks | Payload bound does not establish complete-session limits, resistance to slow fragments, or bounded total candidate-chain memory. | MB-04, R31–32 |
| F10 | Request/key/evidence equality checks and public approval relationships are the implemented personhood boundary | Fresh keys/new commitments from one person are not detected; identities and relationships are linkable. Do not present this as private biological deduplication. | MB-05/09/12, R33 |
| F11 | Account executor deducts transfer fees and returns totals; it does not implement the agreed future reward distribution | Reusing that behavior as the final fee economy would silently choose monetary behavior. V2 integration uses explicitly zero-fee test transfers until MB-07 decisions. | MB-07, R08/R34–35 |

Source anchors (relative links show current branch; audited revision above fixes
which contents were reviewed):

- [Personhood rules, events, mutation and root](../crates/mirror-personhood/src/lib.rs)
- [MPH1 size limits and structural-only decoder](../crates/mirror-personhood/src/codec.rs)
- [Existing 26 personhood tests](../crates/mirror-personhood/src/tests.rs)
- [V1 transfer fields/signing preimage](../crates/mirror-core/src/transaction.rs)
- [120-byte V1 header](../crates/mirror-core/src/block.rs)
- [MRB1 codec](../crates/mirror-core/src/codec.rs)
- [Account-only execution and state](../crates/mirror-state/src/lib.rs)
- [Genesis, acceptance and fork comparison](../crates/mirror-chain/src/lib.rs)
- [Disk publication and recovery](../crates/mirror-storage/src/lib.rs)
- [Framed TCP read/write](../crates/mirror-network/src/lib.rs)
- [Hello/GetBlock/BlockData protocol](../crates/mirror-protocol/src/lib.rs)

## 3. Adversaries and required protections

**Malicious applicant:** substitutes evidence, submits replayed requests, changes
keys, creates simultaneous enrollments, floods pending state or cancels to reclaim
a grant. V2 authorization/context/limits can address some of these; real-human
matching and economic eligibility remain explicit external gates. Tombstones and
activation ordinals are not a complete recovery/biometric deduplication design.

**Dishonest or compromised reviewer:** approves false humans, signs twice, shares
keys, withdraws to manipulate counting, or colludes with the applicant. Enforce
signatures, exact-target binding, distinct reviewers, sequences, parent eligibility
and block-end checks. A coalition with valid keys can still lie; assignment,
challenges, availability and real-admission fault assumptions remain D-02.

**Malicious proposer/peer:** reorders or withholds operations, changes roots,
claims a false height/population, serves inconsistent branch data or mixes networks.
One executor, full commitments, exact config/genesis and bounded decode reject
invalid transitions. They do not establish censorship resistance, equal selection,
finality or progress against a hostile majority. Those belong to MB-06.

**Compromised capture/matching system:** replays recordings, swaps applicants,
returns stale/invented no-match reports or leaks evidence. Keep archives off-chain;
require explicit capture/session/profile/population binding and private access
controls in MB-08/09. A hash/signature alone does not establish capture provenance.
A matcher outage must not translate to NO_MATCH or automatic approval.

**Storage/process failure or competing writer:** crashes between publication steps,
loads an older valid snapshot, corrupts a file or races the writer. Require locked,
versioned stores, bounded reads, reconstructible canonical state, explicit tip
commitment and failure-injection tests. A valid old log is not proof of latestness.

**Observer:** links applicant/reviewer keys, timing and recurring commitments.
This prototype has public relationships. Do not claim total anonymity or publish
stable fingerprint hashes, encrypted archives, evidence URLs or personal records.
Cryptographic commitments require an appropriate randomized construction outside
this register; the ledger cannot verify how an opaque commitment was produced.

## 4. Security invariants for the next implementation

- The same trusted config, parent and ordered operations produce identical roots
  on independent nodes, without device/matcher queries or local clock decisions.
- An invalid operation/block changes no balances, membership, sequences, indexes,
  activation counters, population records or authoritative tip.
- Context includes seed order/contents, economic test profile and all limits;
  changing any consensus rule requires an explicit new version/configuration.
- A person admitted in block h cannot authorize another admission in that block.
  A same-block withdrawal removes that signer's approval from final activation
  counting, regardless of the approval's earlier position.
- Only the selected canonical history contributes state/counters. Orphaned peaks
  and approvals are discarded together. Future finalized history is never replaced
  by the old longest-chain path.
- No runtime operation in this integration scope can mint or distribute grants.
  Unknown operation kinds cannot be treated as harmless payloads.
- No raw/encrypted biometric archive is intentionally accepted by the client/wire
  schema; arbitrary 32-byte fields are not a proof against malicious covert data.

## 5. Residual risks and launch gates

Resource caps can themselves be exhausted, denying honest access. Parent-snapshot
eligibility prevents same-block propagation but does not stop collusion over many
blocks. Request expiry bounds authorization age but does not refresh old readings.
State-root commitments detect disagreement, not fraudulent true-looking evidence.
Static source review and old passing tests do not establish the proposed fixes.

D-01/D-04 gate issuance and fees; D-02 gates real admission; D-03 gates human-round
consensus; D-05/D-06 gate sensitive evidence and real-device recognition. Devices,
biometrics, anonymity, recovery and inheritance are not silently implemented here.
Founder-led startup is explicitly centralized initial trust, with no claimed
permissionless security until the specified handoff and fault model are tested.

## 6. Original review outcome and current continuation

MB-01A design deliverables identify integration blockers, choose a compatibility
path, and map defenses to tests. They did not close F01–F11. The original next task
was MB-01B/B1. B1–B4 are verified standalone components; MB-02A's Transfer/unified
operation and BlockV2 codecs, commitments and independent vectors are complete and
verified in exact-source CI. **Current next: MB-02B**, deterministic combined-ledger
execution.
Actual chain authority remains MB-02C; MB-02 overall remains IN_PROGRESS and
Gate A/Gate B remain unmet. Durable recovery and multi-node membership reorgs remain
MB-03. Exact-source CI acceptance is recorded in CURRENT_STATUS.md.
Keep MPH1/MRB1 unchanged and carry findings forward until linked passing tests
demonstrate the required scope, rather than inferring complete deployment security.
