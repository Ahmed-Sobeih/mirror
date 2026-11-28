# Mirror master plan — blockchain to biometric enrollment readiness

Roadmap: **MIRROR-PLAN-001**. Planning deliverables complete.
Latest implementation: **MIRROR-MB04C1-001 — bounded local pending-operation pool**.
Acceptance and exact source/publication evidence are recorded in section 33.

**MB-04C1 is VERIFIED_COMPONENT — BOUNDED LOCAL PENDING-OPERATION POOL.**
C1 adds an in-memory library component for context-valid signed retention,
one atomic ordered batch evaluation and explicit selected-parent reconciliation.
B4's finite run rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown,
A2 operator commands, A1 durable control/recovery and MB-03's storage/peer components
retain their accepted boundaries. MB-04 remains IN_PROGRESS: finite operator pool
intake, persistence/gossip integration, wallet, wider APIs and broader scheduling
remain unfinished. Complete Ant Kiss remains MB-05 and final human consensus
MB-06. Gate A/B remain UNMET.

## Start here: current position and next action

**Next: MB-04C2 — finite local pool intake and inspection. READY — DESIGN FIRST.**

Design the smallest explicit operator path that can feed already signed V2 inputs
into C1 and inspect one local result against a clean accepted A1 chain. Specify the
command, deliberate input source/framing and finite invocation lifetime before
implementation. Preserve C1's exact-parent binding, contextual-only admission,
FIFO batch semantics, independent limits and cumulative work; never treat a pool
receipt as accepted chain state or permission to publish a block.
Require independently selected configuration/genesis identity; signed input must
never select its own network or accepted parent.

Define argument, input-file/stream, item-count and total-byte ceilings before any
input-sized read/copy, and a closed policy for malformed framing, duplicates,
individual rejections, budget exhaustion and a blocked requested batch. Choose
all-or-nothing intake versus bounded per-item reporting explicitly; do not let a
decoder or reporting loop silently reset C1's owner allowance. Keep retained,
contextually admitted, evaluated and canonically included observations distinct.
Choose bounded regular-file intake with explicit file-type/symlink/EOF rules, or
specify absolute deadline, cancellation and terminal-EOF behavior for streams.
Byte and item limits alone do not bound a stalled pipe, FIFO or stdin read.

Define clean controller ownership and selected checkpoint capture, shutdown/error
semantics, bounded output and release-before-output behavior. Any A1 open/replay/
synchronization must retain its existing contract. Report the exact observed
parent and finite local outcome; do not imply durable pool retention across exit,
network propagation, inclusion or currentness after owner release. Test actual
operator processes, malformed/overlong input, independent caps, duplicate/order
handling, contextual/execution errors, output failure and unchanged canonical
history/control selections before claiming operator integration.
Opening the A1 writer to borrow its accepted chain includes ownership, replay and
synchronization; unchanged canonical selection does not turn it into the existing
read-only, no-synchronization `status` path.

No wallet signing/key custody, automatic mining/append, new monetary rule,
transaction gossip, pool persistence, remote submission endpoint or integration
into B4's retained network loop is implicitly authorized by this narrow step.
Those require separately specified increments. Keep existing commands, A1/F2/store
authority, consensus encodings and all policy gates unchanged. C2 is planned only;
broader scheduling, wallet, wider APIs, full MB-04 and Gate A/B remain unfinished.

Run every existing AGENTS.md gate, verify published source and exact-source CI,
then check the later checkpoint head separately and refresh this handoff.

Local and exact-source CI passed **1,333 runtime tests, eleven doctests, all
required release suites, twelve references and sixteen examples**. These are
component acceptance results, not a readiness percentage.

**Progress estimate, reaffirmed in discussion on 2026-11-26: roughly 20–25% of
Gate A, software readiness for device integration.** This is a broad engineering
judgment, not a measured percentage, task/test-count calculation, time forecast or
security guarantee. The existing foundations are substantial; the remaining
network/operator, Ant Kiss, human-consensus, monetary-policy, private intake/matching
and end-to-end acceptance work keeps the full milestone in its early stages.
Gate A uses synthetic readings; actual compatible-device capture and recognition
acceptance remain Gate B. This discussion changes no implementation status or next
task and is not a claim of an automatic percentage increase from an individual slice.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 33, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [pending pool](docs/PENDING_POOL_V2.md),
[run lifecycle](docs/NODE_RUN_V2.md), [failover](docs/NODE_FAILOVER_V2.md),
[rejoin](docs/NODE_REJOIN_V2.md), [provider](docs/NODE_PROVIDER_V2.md),
[operator](docs/NODE_OPERATOR_V2.md), [control](docs/NODE_CONTROL_V2.md),
[sessions](docs/PEER_SESSION_V2.md), [ledger](docs/LEDGER_STATE_V2.md),
[chain](docs/CHAIN_V2.md), [threat model](docs/THREAT_MODEL.md),
[acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md) and [the-prompt](the-prompt).
Recheck live refs and preserve newer/unpushed work and runtime data.

## 1. What completion means

### Gate A — software-ready for device integration

A reproducible, versioned development network can execute the complete enrollment
and transaction flow using synthetic input through the same documented interfaces
that a supported fingerprint/ECG adapter will use. It includes real ledger
integration, defined admission rules, the bounded human-participation consensus
profile, wallet/node operations, protected off-chain enrollment handling, matching
interfaces, and negative/security tests. Accepted monetary rules are implemented
for test currency only after their remaining policy decisions are resolved.

The output is a release candidate, an operator guide, an adapter contract, and an
acceptance report identifying exact commits and tests. It is NOT a public mainnet
or a demonstration of reliable biological uniqueness. Simulator tests do not
establish recognition accuracy, liveness, sensor authenticity or device support.

### Gate B — controlled real-device pilot accepted

After the user supplies compatible devices, each actual adapter, capture profile,
and matching implementation passes the device acceptance tests in MB-12. Repeated
captures, duplicate attempts and human review are evaluated with approved,
consenting participants and a defined retention/security procedure. The result
applies only to the tested models, firmware, population and conditions.

Public launch, permissionless global enrollment, guaranteed anonymity, billion-user
capacity, and independent security assurance require further evidence. They are
not quietly bundled into the phrase "ready to accept readings."

### The architectural boundary

```
Fingerprint scanner + ECG device (or one combined instrument)
  -> local capture adapter / enrollment application
  -> protected OFF-CHAIN quality checking, template processing and duplicate search
  -> signed, session-bound review evidence + assigned human approvals
  -> ON-CHAIN membership transitions, commitments, eligibility and accounting
  -> human-limited consensus participation and authorized test-currency rewards
```

No raw OR encrypted biometric archive is published to the blockchain or GitHub.
The device measures; software assists comparison; humans review evidence; nodes
validate the admission rules. No sensor or matching service unilaterally mints a
human identity. A signature proves authorization, not the truth of a biometric
claim. Encryption alone is not a private duplicate-search protocol.

## 2. Verified baseline, not starting from zero

Historical pre-plan branch: `dd4c2a1a7c73368364a27557537864d8c9814a03`.
Observed `main`: `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
PR: https://github.com/Ahmed-Sobeih/mirror/pull/1
Pre-plan baseline CI: run 44186275709 — SUCCESS.
Plan commit `db2861930cc0f01f735ccfba80a4f530f4d1ddd6` passed run 44189324502;
that source was reviewed for MB-01A. Design commit `80b57b973e3d9cfb9aa57a20a10fea478440874b`
passed run 44194994506. B1/B2/B3/B4 evidence is in sections 11/12/13/14;
MB-02A evidence is in section 15; MB-02B in section 16; MB-02C in section 17;
MB-03A in section 18; MB-03B in section 19; MB-03C in section 20;
MB-03D in section 21; MB-03E1 in section 22; MB-03E2a in section 23; MB-03E2b in section 24;
MB-03F1 in section 25; MB-03F2 in section 26; MB-04A1 in section 27; MB-04A2 in section 28;
MB-04B1 in section 29; MB-04B2 in section 30; MB-04B3 in section 31;
MB-04B4 in section 32; MB-04C1 in section 33.

Historical suite: 133 workspace tests. B1 added 31 (164 total), B2 added 39 (203),
B3 added 51 (254), B4 added 69 (323), MB-02A added 80 (403), and MB-02B added
59 (462); MB-02C adds 61 (523); MB-03A adds 65 (588); MB-03B adds 70 (658);
MB-03C adds 49 (707); MB-03D adds 28 (735); MB-03E1 adds 52 (787);
MB-03E2a adds 49 (836); MB-03E2b adds 55 (891); MB-03F1 adds 60 (951);
MB-03F2 adds 59 (1,010); MB-04A1 adds 73 (1,083); MB-04A2 adds 54 (1,137);
MB-04B1 adds 30 (1,167); MB-04B2 adds 34 (1,201); MB-04B3 adds 41 (1,242);
MB-04B4 adds 39 (1,281); MB-04C1 adds 52 (1,333).
The subprocess-only storage
helper is explicitly ignored and not counted as a passed test. B4 added the first compile-fail doctest; MB-02B adds two consuming-stage
compile-fail cases, bringing documentation tests to three. MB-03D adds three
borrow/type-boundary cases, bringing documentation tests to six. MB-03E1 adds two
replacement-plan borrow cases, bringing documentation tests to eight. MB-03F1 adds
one consuming-receiver compile-fail case, bringing documentation tests to nine.
MB-03F2 adds one consuming-publication case (ten); MB-04A1 adds one consuming-controller case (eleven).
Release reruns are NOT additional unique tests. Both device-readiness gates remain UNMET.

Already demonstrated in the legacy V1 chain: development signed transfers,
account state, fixed easy PoW, persisted blocks, restart, TCP synchronization and
competing-branch reorg.
Already implemented separately: signed pending/active/voluntarily-revoked
membership, distinct active approvals, exact request/key/evidence-reuse rejection,
atomic event batches, population/activation counters, a deterministic root, and
bounded event-log replay. See [library scope](docs/PERSONHOOD_V0_1.md).
B1 adds validated configuration/proof-checked bootstrap. B2 adds signed fixed-size
membership messages and contextual authorization checks against a trusted view.
B3 adds bounded immutable V2 execution, private indexes/tombstones, exact sequences,
expiry, parent-eligible approvals, end-of-block activation and counters. B4 adds
canonical V2 roots, an untrusted snapshot wrapper and bounded replay against trusted
per-height/checkpoint expectations. MB-02A now provides signed transfers, unified
operation ordering and bounded V2 block encodings/commitments. MB-02B now executes
accounts and membership atomically with original mixed indices, checked transfers,
independent sequences and combined roots. MB-02C now validates actual complete
blocks from a pinned manifest/genesis, uses its accepted branch for ancestry, and
replays complete MRB2 histories against external selected checkpoints. MB-03A now
initializes and reopens those immutable selected histories with bounded disk inputs,
cooperative directory ownership and full accounts/membership replay. MB-03B adds
a separately versioned accepted-child append API, write fencing and checked
resynchronization of complete selected histories. MB-03C adds separately invoked
recovery of recognized interrupted one-child staging under independently retained
intent. Ordinary opening still rejects staging; unknown data remains untouched.
MB-03D adds pure comparison of complete accepted branches, independent network
identity checks, fixed-target normalized scoring, keep-current ties and actual
common ancestry. Its immutable result preserves all accounts, membership and
resource accounting with the chosen branch; it does not publish that choice.
MB-03E1 derives separately versioned replacement metadata and privately borrows
the complete preferred candidate after checking independent current pins and exact
current metadata. MB-03E2a implements separate whole-branch disk replacement,
checked cleanup and clean reconciliation. MB-03E2b implements explicit interrupted
replacement recovery with both independently retained accepted histories and bounded
completion/retention. MB-03F1 adds versioned bounded framing, pre-payload header
admission and complete in-memory candidate reconstruction. MB-03F2 supplies finite TCP sessions and explicit durable adoption of received
candidates. MB-04A1 persists local control/restart authority; MB-04A2 supplies
finite explicit operator commands. MB-04B1 supplies a finite fixed-history provider
with cumulative budgets and draining shutdown. MB-04B2 adds finite rejoin/serve
coordination under the same controller and transport allowance. MB-04B3 adds
bounded explicit-peer acquisition with a closed typed advancement policy and
the same post-receive path. MB-04B4 adds finite repeated rounds retaining the
same roots, signal guard and cumulative allowance. MB-04C1 adds bounded local
signed-operation retention, atomic ordered evaluation and explicit parent
reconciliation. Broader scheduling and operator pool intake remain MB-04.

NOT implemented: biological deduplication, broader persistent network scheduling,
production reviewer assignment, human-round consensus, grants/fees/rewards,
anonymous membership proofs, biometric intake, or real-device integration.
The prototype's public keys and approval relationships are linkable.

Preserve the historical public-key Alice 100-MRY development fixture and local
`mirror-data`. It is NOT a founder allocation. New network versions use separate
explicit configuration and data directories; no silent genesis reset or migration.

## 3. Fixed direction and decisions still needed

Accepted direction comes from [DECISIONS.md](docs/DECISIONS.md), not implementation
convenience: no extra founder allocation or privileged marketing reserve; first
1,000 qualifying humans receive 10 MRY each including founder; 0.2 MRY ordinary
allowance per qualifying historical population maximum; 1 MRY = 10,000,000 Nusa;
founder-led initial verification with no permanent special powers; offline Ant Kiss
with fingerprint/ECG inputs; interchangeable hardware; human-limited participation
with easy PoW subordinate to it.

| Gate | Unresolved choice | Blocks which work? |
|---|---|---|
| D-01 | Bootstrap exception additive versus temporary floor; maturation and qualifying population; false-admission correction | Mint/grant accounting in MB-07, not membership integration |
| D-02 | Approval/review threshold and assignment; founder-led startup transition; appeals, recovery, suspension and revocation authority | Real admission in MB-05/MB-10; synthetic profiles remain experimental |
| D-03 | Consensus fault assumptions, participant set/selection, quorum/locking, epoch changes and easy-PoW cost | Human-based finality in MB-06 |
| D-04 | Reward eligibility/amounts, fees, commissions and funding after issuance exhaustion | Protocol rewards in MB-07 |
| D-05 | Evidence custody, authorized search, private matching, retention/deletion, consent and recovery | Real biometrics in MB-09/MB-12 |
| D-06 | Capture profiles, matching algorithms/versions, quality/error thresholds, exceptions | Uniqueness acceptance in MB-12 |

Prepare options when each task is reached; do not require all policy choices before
membership integration. Do not use permissive fallback to pass a blocked gate.

## 4. Progress tracker

Statuses: IMPLEMENTED, VERIFIED_COMPONENT, READY, TODO, IN_PROGRESS, BLOCKED, VERIFIED.
Only mark VERIFIED with deliverables, acceptance evidence and publication refs.
For design-only work this means design acceptance, NOT runtime implementation.
An IMPLEMENTED component with passed local gates still has its source-publication
and exact-source CI evidence recorded separately before full acceptance.

| ID | Work package | Status | Dependencies / exit gate |
|---|---|---|---|
| MB-00 | Existing chain and standalone personhood baseline | VERIFIED_COMPONENT | Integration remains open |
| MB-01A | Integration design, threat review and test specification | VERIFIED — DESIGN ONLY | Section 10 |
| MB-01B | Harden registry foundation | VERIFIED_COMPONENT — B1/B2/B3/B4 | Accepted-block integration delivered by MB-02; live node remains MB-03/04 |
| MB-01B/B1 | Canonical config, seed proofs and bounded rules | VERIFIED | 809661b, CI 44201393943, section 11 |
| MB-01B/B2 | Signed membership messages, codecs, ancestry/windows | VERIFIED — MESSAGE LAYER | af1dd66, CI 44205064789, section 12 |
| MB-01B/B3 | Bounded staged execution, sequences, expiry, eligibility/activation | VERIFIED — IN MEMORY | b6f1233, local suite and CI 44317174000 PASS, section 13 |
| MB-01B/B4 | Full registry codec/root/replay and adversarial regression | VERIFIED — COMPONENT | 21e2272, CI 44404337870, section 14 |
| MB-02 | Versioned membership operations in block execution | VERIFIED_COMPONENT — IN MEMORY | A/B/C complete; durable canonical publication and network integration remain MB-03/04 |
| MB-02A | TransferV2, unified envelope, BlockV2 codecs/commitments | VERIFIED_COMPONENT — FORMAT LAYER | 35ad0cf, CI 44428924399, section 15 |
| MB-02B | Atomic accounts + membership with full mixed-operation indices | VERIFIED_COMPONENT — STAGED LEDGER | 420d711, CI 44619155972, section 16 |
| MB-02C | Pinned V2 genesis, actual block acceptance and replay | VERIFIED_COMPONENT — ACCEPTED BRANCH | db2cb58, CI 44623282448, section 17 |
| MB-03 | Durable state, synchronization and membership reorg/replay | VERIFIED_COMPONENT — FINITE V2 STORAGE / RECOVERY / PEER FLOW | A–F complete within finite component scope; persistent node control/restart authority and operations remain MB-04 |
| MB-03A | Explicit V2 storage identity, writer ownership and initial publication | VERIFIED_COMPONENT — IMMUTABLE STORE | 528c891; independent pins, complete replay, Linux directory ownership; section 18 |
| MB-03B | Durable accepted-tip updates and failure reconciliation | VERIFIED_COMPONENT — APPEND / CLEAN STATE | ecf5e76; separate MSU2, accepted-child publication, fencing and selected-state resynchronization; section 19 |
| MB-03C | Explicit interrupted-append staging recovery | VERIFIED_COMPONENT — EXPLICIT ONE-CHILD RECOVERY | e830205; independent request, complete replay, explicit completion/retention, bounded retry; section 20 |
| MB-03D | Canonical development fork-choice specification and implementation | VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON | Independent identity, normalized fixed-target score, keep-current ties, accepted common ancestry and whole-state borrowed selection; section 21 records source acceptance |
| MB-03E | Durable membership-aware branch replacement | VERIFIED_COMPONENT — LOCAL REPLACEMENT / RECOVERY | E1/E2a/E2b complete; F2 supplies finite received-candidate adoption; persistent node remains MB-04 |
| MB-03E1 | Replacement selection metadata and checked preparation | VERIFIED_COMPONENT — METADATA / IN MEMORY | Separate MSR2, independent current pins, canonical metadata comparison and complete candidate borrow; section 22 |
| MB-03E2 | Replacement store, publication and explicit recovery | VERIFIED_COMPONENT — LOCAL STORE | E2a publication/cleanup and E2b authorized recovery; sections 23–24 |
| MB-03E2a | Bounded durable replacement and clean reconciliation | VERIFIED_COMPONENT — REPLACEMENT / CLEAN STATE | Generation layout, full candidate publication, exact old cleanup, fencing and complete reopen; section 23 |
| MB-03E2b | Explicit interrupted replacement recovery | VERIFIED_COMPONENT — EXPLICIT RECOVERY | Exact old/candidate histories, independent pins, completion/retention and bounded retries; section 24 |
| MB-03F | Bounded fresh-peer V2 reconstruction | VERIFIED_COMPONENT — FINITE PEER FLOW | F1 complete history reconstruction plus F2 TCP sessions, real-process evidence and explicit durable adoption; section 26 |
| MB-03F1 | Versioned bounded history framing and consuming receiver | VERIFIED_COMPONENT — FRAMING / IN MEMORY | 60 new runtime tests, 9 total doctests and required local gates; publication/CI separate; section 25 |
| MB-03F2 | TCP-session integration and durable candidate adoption | VERIFIED_COMPONENT — FINITE TCP / DURABLE ADOPTION | 59 new runtime tests, 10 total doctests, exact source CI, 11 references and 15 examples; section 26 |
| MB-04 | Persistent node, transaction pool, wallet and operator API | IN_PROGRESS — A1/A2/B1/B2/B3/B4/C1 VERIFIED | Next C2 design for finite local pool intake/inspection; broader scheduling, wallet and APIs remain |
| MB-04A1 | Durable V2 control state and restart authority | VERIFIED_COMPONENT — DURABLE CONTROL / EXPLICIT RESTART | Separate bounded two-history intent, independent confirmed pins, synchronized publication and explicit process recovery; section 27 |
| MB-04A2 | Separate V2 operator lifecycle commands | VERIFIED_COMPONENT — FINITE OPERATOR COMMANDS | Explicit attach-existing init, read-only status, one receive/adopt and explicit recovery; section 28 |
| MB-04B1 | Finite V2 provider lifecycle and draining shutdown | VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN | One fixed accepted history, retained ownership, cumulative attempts/bytes, original deadlines and resource release before output; section 29 |
| MB-04B2 | Finite rejoin and serve coordination | VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION | One retained controller and cumulative budget, phase-specific stop/error handling, three-process restart/rejoin; section 30 |
| MB-04B3 | Bounded explicit-peer failover | VERIFIED_COMPONENT — BOUNDED EXPLICIT-PEER FAILOVER | One-to-four ordered peers, closed typed advancement, cumulative allowance and first-valid keep/adopt/service; section 31 |
| MB-04B4 | Bounded persistent-run lifecycle | VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE | One-to-four explicit rounds, retained roots/signals, cumulative budgets, bounded waits and sticky completion context; section 32 |
| MB-04C1 | Bounded local pending-operation pool | VERIFIED_COMPONENT — BOUNDED LOCAL POOL | Contextual signed retention, one atomic FIFO stage, cumulative work and explicit accepted-parent reconciliation; section 33 |
| MB-04C2 | Finite local pool intake and inspection | READY — NEXT, DESIGN FIRST | Specify explicit bounded signed-input source, independent identity, controller ownership and truthful finite reporting |
| MB-05 | Ant Kiss admission lifecycle and bootstrap handoff | TODO | MB-02/04; D-02 for real admission |
| MB-06 | Human-limited rounds, certificates and easy-PoW participation | TODO | MB-03/04/05; D-03 |
| MB-07 | Bootstrap grants, population allowance, rewards and fees | TODO — POLICY-GATED | MB-05/06; D-01/D-04 |
| MB-08 | Device-neutral inputs and adapters | TODO | Draft contract exists; can start after MB-01A |
| MB-09 | Protected storage and private duplicate-checking pipeline | TODO — PRIVACY-GATED | MB-05/08; D-05 |
| MB-10 | End-to-end verifier/application workflow | TODO | MB-04/05/08/09 |
| MB-11 | Hardening, deployment and Gate A acceptance | TODO | MB-01 through MB-10 gates |
| MB-12 | Actual devices and controlled pilot | TODO — DEVICE-GATED | Gate A, compatible devices, D-05/D-06 |

Default order is MB-01A, MB-01B, MB-02 onward. MB-08 interfaces can be brought
forward to establish compatibility before purchases. When policy blocks one
package, record the blocker and continue dependency-ready work. No procurement
or live biometric collection merely to unblock CI.

## 5. Work packages and acceptance tests

### MB-01A — specify integration before changing consensus

**Delivered:** [ledger specification](docs/LEDGER_MEMBERSHIP_V1.md),
[threat model](docs/THREAT_MODEL.md), and [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md).
Inspect actual personhood, core encodings, account execution, chain acceptance,
networking/storage and PR comments. Specify network/genesis/configuration binding,
typed signatures, ordering, replay scope, commitments, bounds and eligibility timing.
Review spam, stale evidence, cross-configuration reuse and rollback assumptions.
Choose explicit versioned integration preserving MRB1/local data; never convert
known test keys into real founder credentials. Keep biometric formats off-chain.

**Pass:** compatibility/transition tables, atomicity requirements and adversarial
assertions reviewed against actual APIs. List open assumptions, not implementation
claims. Design evidence is in section 10.

### MB-01B — harden the registry foundation

**Deliver:** targeted fixes and regression tests in versioned `mirror-personhood`
modules, with biological deduplication explicitly external. B1 configuration/seed
binding, B2 signed messages/windows, B3 staged execution and B4 commitments/codecs/
grouped-block replay are VERIFIED within component scope. See sections 11–14.
Preserve V1 APIs/vectors and MCF2/B2 signed bytes; any semantic/encoding change
requires an explicit revision and independently recomputed vectors.

**Pass:** forged/replayed/stale/out-of-order events, wrong context, invalid
withdrawals, overflow and excessive input fail without mutating state. Pending,
total records and block budgets apply at every execution path. Activation/counters
are deterministic. Recovery behavior is tested only after it is specified; no
invented recovery or economic right may be added to complete the milestone.

B3 now enforces actor eligibility, target status/ownership, exact sequences with
checked increments, request/approval window alignment, retained identity references,
expiry, parent-set reviewer eligibility and block-end activation/counters. It returns
a staged immutable candidate; rejection or success never mutates the parent.
There is no public unbounded event-application bypass. B2 context-valid unknown
targets/exhausted sequences are rejected by this stateful boundary.

B4 preserves the [frozen state layout](docs/MEMBERSHIP_STATE_V2.md), bounds snapshot
encoding/decoding, and reproduces independent state vectors. Its untrusted cache has
no runtime conversion. MPH2 replay retains membership-only block boundaries/indices,
including empty blocks, and verifies trusted per-height hashes, exact-operation group
digests and roots plus an expected final checkpoint. It never infers authority or
freshness from a self-consistent log/snapshot. Block-hash provenance remains outside
canonical state. MB-02 supplies actual validated block/root association and full
mixed-operation positions. The diagnostic digest is not the MRB2 Merkle root implemented in MB-02A.

### MB-02 — membership becomes blockchain state

**Deliver:** versioned membership and transfer operations, canonical codecs, atomic
accounts+membership execution, and one commitment to all consensus state/counters.
Connect core/state/chain without a second authoritative side membership database.

Implemented in three bounded slices; all are verified within component scope:

- **MB-02A (VERIFIED_COMPONENT):** TransferV2 kind 0 (243 signed bytes), unified operation envelope,
  160-byte header, bounded MRB2 codec (168-byte empty block), domain-separated block
  hash/count-bound Merkle operation commitment and independent reference vectors.
  Preserve membership signed bytes; avoid a core/personhood dependency cycle.
- **MB-02B (VERIFIED_COMPONENT):** stage accounts and membership together, preserve original full-block
  operation indices including transfers, checked positive zero-fee/no-mint transfers,
  separate account/member sequences and canonical combined roots. Test whole-ledger
  rollback after a late failure and conservation of initial synthetic supply.
- **MB-02C (VERIFIED_COMPONENT):** explicit MNM2 manifest and pinned exact genesis,
  actual block/header/root/parent/configuration/PoW validation, private accepted
  candidate ancestry and bounded complete MRB2 replay against an external checkpoint.
  Only fully validated blocks bind provenance through this acceptance API.

The original B3 stage_block and MPH2 remain membership-only. MB-02B adds an indexed
consuming stage preserving full positions and exact omitted-transfer byte budgets.
MPS2 now explicitly uses max_operations as its runtime inclusion-index bound.
Complete mixed history now replays from validated MRB2 through ChainV2, never
filtered into MPH2 or renumbered; durable integration remains MB-03. Preserve previous bytes and untrusted-cache rules.
Structural codecs do not establish ledger validity or canonical-chain authority.

Validation must not rerun floating-point biometrics, query a central matcher, or
decrypt an archive. Validate authorized versioned evidence with explicit issuer
trust rules; a receipt is not mathematical proof of a true biometric claim.

**Pass:** two independent validators accept the same membership-bearing block;
changed approvals, roots, policies, counters, order or context reject. Invalid
mixed batches change neither money nor membership. Replay reproduces roots;
legacy history remains intact. Freeze block/ledger vectors independently.

### MB-03 — persistence, synchronization and reorganizations

**First slice — MB-03A (VERIFIED_COMPONENT):** separate immutable profile1 storage
now provides explicit initialization, exclusive reopening, shared read-only
inspection, independently pinned network/checkpoint selection, bounded file inputs
and complete ChainV2 replay. Manifest and blocks are synchronized before final
generation-zero selection publication. Partial data is retained; ambiguous results
return no owning handle and require checked reconciliation. The stable root inode
provides cooperative Linux ownership. Legacy recovery is never invoked.

**Second slice — MB-03B (VERIFIED_COMPONENT):** a separate explicit append profile
uses 280-byte MSU2 around unchanged MSS2 metadata and an independently selected local
generation. One complete child is validated, the old disk chain rechecked, and new
block/selector publication synchronized before memory advances. Errors fence the
handle. Writer reopening and consuming reconciliation resynchronize fully validated
clean selected history while retaining stable ownership. Strict normal opening
preserves and rejects residual tail/pending/unknown data; it does not repair it.

**Third slice — MB-03C (VERIFIED_COMPONENT):** explicit one-child staging recovery
uses independently retained parent/candidate checkpoints, the complete intended
child and a selected Complete or RetainParent action. It fully replays the parent
and accepts the child before extending or removing only authorized matching staging.
Prefix equality constrains bytes but does not prove origin; authority comes from the
external request. Completion and retention synchronize in a defined order, retain
exclusive root ownership and support retries after recognized interruptions. A
selected child is never rolled back. Ordinary opening remains strict; unknown or
ambiguous artifacts remain preserved/rejected. Existing formats remain unchanged.

**Fourth slice — MB-03D (VERIFIED_COMPONENT):** the
[development comparator](docs/FORK_CHOICE_V2.md) compares only complete accepted
ChainV2 histories. It independently checks both configuration/genesis identities
before comparing normalized fixed-target scores and accepted hashes. Strictly
higher candidate height wins; ties keep current. Its private borrowed result
returns the actual common ancestor and the complete preferred branch, with no
state mutation, replay, encoded score or durable publication. Section 21 records
the source and verification evidence.

**Fifth slice — MB-03E1 (VERIFIED_COMPONENT):** the separate
[MSR2/preparation contract](docs/STORAGE_REPLACEMENT_V2.md) freezes profile-3 bytes
and checks full current metadata against accepted state before preparing a strictly
preferred complete candidate. It has no filesystem store or publication/recovery API.

**Sixth slice — MB-03E2a (VERIFIED_COMPONENT):**
[the replacement store](docs/STORAGE_REPLACEMENT_STORE_V2.md) performs real whole-branch
publication with at most two generation directories, unchanged MSR2, complete disk
revalidation, continuous root ownership, synchronized marker publication, exact old
copy cleanup and final replay before memory advance. Disk errors fence access;
clean-only reopening/reconciliation preserves all residual data. Section 23 records
source and acceptance. All previous formats and stores retain their boundaries.

**Seventh slice — MB-03E2b (VERIFIED_COMPONENT):**
[explicit replacement recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md) validates
both retained histories and all bounded residue before completing or retaining old.
It preserves same-root ownership, exact full state, bounded retry and final replay/
synchronization; selected candidate cannot roll back. Section 24 records acceptance.
Interrupted initialization, arbitrary corruption repair and global freshness remain
outside this component. No new journal/encoding or policy choice was introduced.

**Eighth slice — MB-03F1 (VERIFIED_COMPONENT — FRAMING / IN MEMORY):** separate
[MPN2 framing and consuming reconstruction](docs/PEER_HISTORY_V2.md) check headers
before exposing payload lengths and execute complete original MRB2 histories under
independent network selection. Exact Begin/End consistency and final accounting
precede candidate release. Section 25 records local acceptance; source publication
and CI remain separate. This slice performs no socket or storage I/O.

**Ninth slice — MB-03F2 (VERIFIED_COMPONENT — FINITE TCP / DURABLE ADOPTION):**
[bounded peer sessions](docs/PEER_SESSION_V2.md) enforce independent MPQ2 selection,
header-first F1 admission, actual cumulative I/O/attempt budgets, one absolute
deadline and matching End plus EOF. [Explicit adoption](docs/CANDIDATE_ADOPTION_V2.md)
checks the owned local store and whole accepted candidate, keeps ties/shorter
histories unchanged, retains both exact recovery histories before publication and
uses the existing owned store/recovery APIs. Separate-process tests demonstrate
fresh reconstruction, genuine equal-account membership forks and strict durable
reopen. Section 26 records exact acceptance. MB-03 is complete within this finite
component scope; MB-04A1 now adds persisted node control and restart authority, recorded in section 27.

**Deliver:** coherent durable commit/recovery for blocks/accounts/membership,
counter indexes and eventual grant history; writer locking; explicit migrations;
corruption detection; bounded block/snapshot reads. Reconcile ambiguous I/O before
writing again. Current directory-swap tests are not universal crash-safety proof.

**Pass:** fault injection at every publication boundary yields identifiable complete
old/new state or explicit failure, never mixed accounts/membership. Reject a second
writer. A fresh peer independently reconstructs membership history. Forks must
contain DIFFERENT admissions/withdrawals, not only timestamps. Orphaned approvals
and population peaks do not survive as canonical state.

High-water population is derived from canonical history, not every observed fork.
Ordinary membership decline does not reduce it. After MB-06, no legacy PoW reorg
path may override finalized certificates/history.

### MB-04 — usable persistent network and wallet

**First slice — MB-04A1 (VERIFIED_COMPONENT — DURABLE CONTROL / EXPLICIT RESTART):**
[Node control](docs/NODE_CONTROL_V2.md) retains independent confirmed pins and
one complete two-history intent in a separate locked directory. Intent is durable
before store mutation; confirmed pins are durable before intent retirement.
Fresh processes require explicit recovery and cannot roll back a selected candidate.
Source and process/failure acceptance are in section 27.

**MB-04A2 (VERIFIED_COMPONENT — FINITE OPERATOR COMMANDS):**
[Operator commands](docs/NODE_OPERATOR_V2.md) add explicit attach-existing init,
read-only status, one bounded receive/adoption and explicit recovery through A1/F2.
Exact source and actual-command process acceptance are in section 28.

**MB-04B1 (VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN):**
[Provider lifecycle](docs/NODE_PROVIDER_V2.md) serves one retained accepted history
with finite aggregate admission/byte limits and original-deadline drain on
SIGINT/SIGTERM or admission expiry. Exact source and process evidence are in section 29.

**MB-04B2 (VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION):**
[Rejoin lifecycle](docs/NODE_REJOIN_V2.md) receives at most one complete candidate,
uses existing keep/adopt and serves the selected fixed branch under retained control,
signal state and cumulative F2 allowance. Exact-source acceptance is in section 30.

**MB-04B3 (VERIFIED_COMPONENT — BOUNDED EXPLICIT-PEER FAILOVER):**
[Explicit-peer failover](docs/NODE_FAILOVER_V2.md) acquires from an ordered
one-to-four peer list under a closed typed failure policy, ending at the first
validated candidate before the shared B2 keep/adopt/service path. Exact source
and process acceptance are in section 31.

**MB-04B4 (VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE):**
[Run lifecycle](docs/NODE_RUN_V2.md) repeats up to four explicit acquisition,
keep/adopt and service rounds with retained roots and one cumulative allowance.
Bounded waits and sticky completion context span rounds. Exact source and
process acceptance are in section 32; indefinite scheduling remains unfinished.

**MB-04C1 (VERIFIED_COMPONENT — BOUNDED LOCAL POOL):**
[The pending pool](docs/PENDING_POOL_V2.md) retains context-valid signed inputs,
stages one bounded FIFO prefix atomically and reconciles exact inclusion/context
against the actual selected accepted parent. Independent retention/batch/work
caps do not grant chain/store authority. Exact acceptance is in section 33.

**Next is MB-04C2**, finite local pool intake and inspection, design first.
Specify bounded signed-input framing, independent identity, clean ownership,
finite reporting and actual-process evidence. Wallet, transaction gossip,
block production and broader operator APIs remain separate later work.

**Deliver:** long-running node; explicit init/status/run; bounded pool; reconnect
and multi-peer sync; operator CLI/API for enrollment, transfers and diagnostics.
Fresh random wallet keys, protected storage and backup/recovery; separate signer,
transport/member roles. Known development keys remain demos only.

**Pass:** at least three independent processes/directories share accepted events;
restart/rejoin converges; duplicate transactions/overspending never become canonical.
Read-only status neither mines nor initializes. Malformed frames, slow fragments,
peer floods and connection loss stay within complete-message/session byte/time
bounds, not merely per-read timeouts. Protect sensitive APIs; no secrets/biometrics
in diagnostics. Never trust advertised heights/roots/populations without replay.

### MB-05 — Ant Kiss admission and bootstrap lifecycle

**Deliver:** versioned sessions, consent/key possession, eligible assigned verifiers,
signed decisions tied to session/evidence/matching-population snapshot, provisional
and review outcomes, bounded appeals and specified lifecycle changes. Unspecified
death/inheritance/emergency powers stay disabled.

Define founder-led seeding and handoff to ordinary distributed review without
enduring founder bypass. Scanner ownership buys no extra votes. Capture profiles
specify consistent fingers and required modalities; no silent removal of ECG.

**Pass:** applicants cannot select all reviewers or reroll freely; repeated approvals
have no extra weight; unusable/incomplete evidence never auto-activates. Retries
avoid permanent gatekeeping. Concurrent enrollments of the same person are
serialized/rechecked against updated population so two stale no-match receipts
cannot blindly create two memberships. Colluding reviewers remain explicit threats.

**Future accessibility design:** the user's
[accessible-enrollment proposal](docs/ACCESSIBLE_ENROLLMENT_PROPOSAL.md) covers
unavailable standard readings, a proposed committee of three qualified people and
controlled changes to an accommodation record. Qualification, assignment, quorum,
alternative evidence and appeals remain open under D-02/D-05/D-06. Prefer a minimal
private explanation. A disability description does not establish uniqueness;
alternative procedures must preserve the same membership and claim history.
Changes must be reviewable and correctable, without a casual enrollment bypass.
This records future work and changes no current admission rule.

The user's latest preferred initial combinations are ECG + a designated pinky +
human supervision, and ECG + private observed capture limitations + reviewed Ant
Kiss where a fingerprint is unavailable. Evaluate the complete procedures; do not
require each ingredient to prove uniqueness independently. Launch sufficiency is
a hypothesis pending D-06 evidence. More fingerprints later remain an option, with
versioned profiles, shared duplicate scope and the same membership/claim history.

### MB-06 — human-based consensus, not a hardware race

**Deliver:** reviewed round-based profile with explicit fault/network assumptions,
membership snapshot, eligibility, locking/voting, quorum certificates, safe timeouts,
set transitions and restart double-sign protection. Evaluate established BFT designs
before inventing a variant [R2]. Equal eligible humans carry equal weight.

A bounded initial pilot may use the full small active set. Committee scaling needs
anti-grinding randomness, key cutoffs and capture assumptions; do not use a
miner-selected previous hash as an unexplained fair lottery. Bind small PoW to
identity/challenge/height/round/purpose; extra nonces/devices buy no votes/rewards.
Benchmark ordinary hardware; no zero-latency claim. Easy PoW does not rescue
admission after an attacker obtains many accepted identities.

**Pass:** equivocation, partitions, offline proposers, delayed/reordered messages,
clock changes, restarts and membership changes preserve the stated safety bound.
Missing quorum stalls/changes round; timeout NEVER lowers quorum. Sync verifies
finalized checkpoints. Founder-only startup makes no decentralization claim.

### MB-07 — issuance and compensation

**Deliver:** after D-01/D-04, integer-Nusa testnet accounting with no founder path
or marketing premine. Specify combined exception/ordinary ceiling, first-1,000
eligibility, qualifying population, pacing, unique claims, fees/roles and exhaustion.
Separate issued money, permitted unissued capacity and transferred fees. Population
allowance is not a payout every block. Do not reward merely approving applicants
or creating self-transfers; admission errors must not silently unlock real issuance.

**Pass:** founder/member 1,000 share 10-MRY eligibility; member 1,001 gets no bootstrap
grant. Replay/recovery/key rotation/revocation/forks cannot double-claim. Ordinary
allowance for 1M -> 900k -> 2M is 200k -> 200k -> 400k MRY, plus exactly the separately
approved bootstrap treatment. No reserve pre-spending, overflow, budget overspend
or fee double count. Test exhausted issuance and zero-fee behavior explicitly.
Real grants remain gated on admission/maturity. Monetary blockers do not stop
independent membership/adapter work.

### MB-08 — device-neutral capture and adapters

The user's 2026-11-26 clarification favors flexible brands/models and a minimal
human-witnessed capture procedure. One anatomical finger plus ECG, possibly the
left little finger, is a proposed profile to evaluate. The exact position and
exception/migration rules remain open; all-fingerprint collection is not assumed.
Consensus consumes authorized outcomes; the private workflow consumes the readings.

**Deliver:** stabilize [DEVICE_INPUT_CONTRACT.md](docs/DEVICE_INPUT_CONTRACT.md);
local SDK/interface, capability negotiation, synthetic source, documented importer
and conformance suite. Isolate vendor parsers/SDKs from consensus. No universal
hardware compatibility promise or implicit vendor selection.

Require specified finger positions/format/quality and actual ECG waveform, timing,
units/calibration metadata. Heart rate and fingerprint-unlock booleans are not
substitutes. **Pass:** two differently encoded synthetic sources normalize;
wrong units/modalities, malformed/truncated/oversized input, NaNs, unsupported
profiles, disconnects/cancellation/retry fail explicitly. Imported files are not
labelled witnessed live capture. Adapters can change without monetary/block changes.

Future accessibility profiles must represent unavailable modalities explicitly.
They require their own approved procedure and evidence; a missing reading is not
a successful result or permission to skip an existing required modality.

### MB-09 — protected evidence and private duplicate checking

**Deliver:** protected off-chain intake, authenticated transfer, encrypted storage,
scoped access, key rotation/backup, minimal retention and auditable deletion.
Specify authorized operators, query abuse, availability, snapshot synchronization
and public evidence boundaries. No archives/decrypting keys on public nodes.

Versioned matcher outcomes: NO_MATCH_WITHIN_TESTED_RULES, POSSIBLE_DUPLICATE,
INCONCLUSIVE, ERROR. Hash equality is not biometric matching. Review uncertain
matches [R1]; specify joint modality interpretation, not guaranteed uniqueness.
A trusted matcher is only a labelled development harness, not decentralized private
matching. D-05 must identify access/collusion assumptions and replacement operators;
evaluate reviewed threshold/MPC approaches without inventing cryptography or hiding
a universal decryption service.

**Pass:** wrong/revoked credentials cannot retrieve evidence; replay/substitution
fails session binding; outages cannot approve. Bind receipts to model/policy,
evidence, population snapshot and freshness. Concurrent duplicate tests pass.
Canary recordings stay out of blocks, peer messages, logs, crash reports and CI.
Document deletion/backup limits and permanent public membership metadata.

For the future accessibility route, protect minimum accommodation explanations
off-chain and keep normal/exception enrollments in the same duplicate-search scope.
Review any public exception indicator for information it reveals; do not promise
unimplemented anonymity.

### MB-10 — complete operator/applicant workflow

**Deliver:** minimal CLI/local UI showing capture quality/retries, assigned reviews,
signed decisions and activation; resumable sessions and clear errors without manual
database edits. Read-only status and transfers/claims only when MB-07 authorizes.

**Pass:** synthetic application -> both readings -> private comparison -> required
approvals -> on-chain activation -> independent peer sync -> eligible participation
-> authorized test payment -> restart with identical state. Duplicate/inconclusive/
failed capture changes neither membership nor issuance. Keys never derive from
biometrics. Lost-key recovery cannot create another person or grant.

Future accessibility work includes understandable alternative-route requests,
review status, correction and appeal, including existing members whose original
capture method becomes unavailable. It must not create a second identity or
reduce ordinary participation rights merely because an accommodation is needed.

### MB-11 — hardening, deployment and Gate A

**Deliver:** fuzz/property/load tests, dependency/build and threat review,
deterministic multi-node harness, reproducible builds/artifacts, setup/backup/
recovery and device-integration guides, exact-commit acceptance report with risks.
Initially Linux/WSL; test other targets before advertising support.

**Pass:** all prior software criteria/policy gates complete; exact-release full and
release-mode suites and integrated scenarios pass. Exercise malformed peers/input,
storage interruption, network partitions, accounting and data-leak checks. Measure
capacity/latency/availability; no invented throughput. Independent review is required
before public-value deployment, not replaced by a test count. Keep devnet isolated.

Gate A discloses untested real hardware/recognition. Stub matching or consensus
means partial scaffold, not completed software readiness.

### MB-12 — actual devices and controlled pilot

**Deliver:** adapter per supplied model/firmware/SDK, export permissions, calibration/
profile validation, support matrix and approved pilot report. No procurement or
uploading real recordings into chat/GitHub is authorized by this plan.

**Pass:** real captures use the same pipeline without consensus edits. Measure
repeat enrollment, false accepts/duplicate flags, acquisition failure/time under
stated conditions/population. Test replay, operator error, new keys/sessions and
cross-device matching. Publish aggregates, not recordings/identities. Small-pilot
success is not billion-user reliability. Unusable data means retry/review, never
an invented uniqueness certificate. Claims require measured D-06 criteria.

Approved alternative accessibility procedures need their own measured evaluation
and cross-route duplicate-enrollment checks. The proposed committee and synthetic
exception scenarios alone cannot establish biological uniqueness.

## 6. Final demonstration checklist

Retain reproducible commands, machine-readable results and exact build hashes,
not screenshots alone. No private data in acceptance evidence.

- Separate nodes agree on money, membership, activation order, supply and eligible
  set after enrollment, transfer, restart and synchronization.
- One entitlement cannot count twice as an approval, transaction, vote or reward.
- Required modalities are present; hardware/founder cannot bypass mature rules.
- Concurrent duplicates, stale receipts and uncertainty do not silently activate.
- Faster hashing buys no extra human voting/reward weight.
- Partitions preserve specified finality; legacy reorg cannot replace finalized
  history; unfinalized rollback is consistent across all components.
- Grants/allowance follow the approved combined formula; fees transfer money and
  create no supply; no hidden owner treasury.
- Biometric archives stay out of public blocks/Git/logs; custody/access/retention
  and deletion are documented/tested.
- Device support matches the tested profiles; unsupported inputs fail clearly.
  Gate A uses synthetic input; Gate B has real-device evidence.

## 7. Working method and continuation

Small reviewable changes with exact tested commits. Do not merge merely to publish
records, replace source wholesale to add imports, reset main or overwrite runtime
data. Check live refs/comments before changing files and preserve concurrent work.
For each milestone record deliverables, tests, source/CI refs, defects and next task.
Keep CURRENT_STATUS, NEXT_SESSION, the-prompt, tracker and acceptance matrix coherent.
Refresh the-prompt for every Mirror response, including discussion-only changes.
Do not count documentation publication as runtime evidence or chase self-referential
checkpoint SHAs. No background-work promise; disclose failed publication.

**Next: MB-04C2 — finite local pool intake and inspection. READY — DESIGN FIRST.**

Design the smallest explicit operator path that can feed already signed V2 inputs
into C1 and inspect one local result against a clean accepted A1 chain. Specify the
command, deliberate input source/framing and finite invocation lifetime before
implementation. Preserve C1's exact-parent binding, contextual-only admission,
FIFO batch semantics, independent limits and cumulative work; never treat a pool
receipt as accepted chain state or permission to publish a block.
Require independently selected configuration/genesis identity; signed input must
never select its own network or accepted parent.

Define argument, input-file/stream, item-count and total-byte ceilings before any
input-sized read/copy, and a closed policy for malformed framing, duplicates,
individual rejections, budget exhaustion and a blocked requested batch. Choose
all-or-nothing intake versus bounded per-item reporting explicitly; do not let a
decoder or reporting loop silently reset C1's owner allowance. Keep retained,
contextually admitted, evaluated and canonically included observations distinct.
Choose bounded regular-file intake with explicit file-type/symlink/EOF rules, or
specify absolute deadline, cancellation and terminal-EOF behavior for streams.
Byte and item limits alone do not bound a stalled pipe, FIFO or stdin read.

Define clean controller ownership and selected checkpoint capture, shutdown/error
semantics, bounded output and release-before-output behavior. Any A1 open/replay/
synchronization must retain its existing contract. Report the exact observed
parent and finite local outcome; do not imply durable pool retention across exit,
network propagation, inclusion or currentness after owner release. Test actual
operator processes, malformed/overlong input, independent caps, duplicate/order
handling, contextual/execution errors, output failure and unchanged canonical
history/control selections before claiming operator integration.
Opening the A1 writer to borrow its accepted chain includes ownership, replay and
synchronization; unchanged canonical selection does not turn it into the existing
read-only, no-synchronization `status` path.

No wallet signing/key custody, automatic mining/append, new monetary rule,
transaction gossip, pool persistence, remote submission endpoint or integration
into B4's retained network loop is implicitly authorized by this narrow step.
Those require separately specified increments. Keep existing commands, A1/F2/store
authority, consensus encodings and all policy gates unchanged. C2 is planned only;
broader scheduling, wallet, wider APIs, full MB-04 and Gate A/B remain unfinished.

Run every existing AGENTS.md gate, verify published source and exact-source CI,
then check the later checkpoint head separately and refresh this handoff.

## 8. Engineering references, not user policy

Repository facts use exact local checkpoint refs. Proposed engineering acceptance
requirements are not capabilities supplied by references. Reverify APIs at use.

- [R1] NIST SP 800-63A-4: https://pages.nist.gov/800-63-4/sp800-63a/ial-general/
  Evidence protection/retention and human match review; no NIST certification or
  government-document requirement is implied.
- [R2] CometBFT specification: https://github.com/cometbft/cometbft/blob/main/spec/consensus/consensus.md
  Round/vote/locking/timeout reference, not decided protocol adoption.
- [R3] WFDB: https://wfdb.io/spec/header-files.html
  Sampling/units/calibration metadata reference, not mandatory equipment or a
  uniqueness algorithm. Never silently guess missing calibration.

## 9. Deferred beyond this endpoint

Social/messaging/dApps, custom-watch manufacture, public-value mainnet, inheritance,
full anonymity, global committees and post-quantum migration are outside this
first pilot endpoint. Do not ignore their hooks but do not silently expand scope.
Linkable pilot membership is not anonymity. Public real enrollment needs additional
security, privacy and applicable legal review.

## 10. Historical MB-01A acceptance — MIRROR-MB01A-001

Source `db2861930cc0f01f735ccfba80a4f530f4d1ddd6` and PR comments were inspected:
personhood/crypto/state, wire formats, chain, storage and network. Delivered
compatibility/transition/commitment design, source findings F01–F11 and T01–T35
planned assertions. Arithmetic checked 131-byte prefix, 160-byte header, 168-byte
empty block and signed lengths 243/291/227/227. No Rust/economic change occurred.
VERIFIED meant design deliverables, not security certification or implementation.

## 11. Historical B1 acceptance — MIRROR-MB01B-B1-001

Source `809661b5e6a9338d65c15c45e540289df24fcc99`, CI 44201393943, job 134456099484.
Implemented immutable validated ConfigV2, checked LimitsV2, bounded MCF2 codec,
ordered seed descriptors/proofs/IDs and checked sorted synthetic allocations.
Context binds every field before seed signing, without a hash cycle. All seed
proofs are required; first seed has no exception. Valid config is not verified
bootstrap or authoritative peer-provided network data.

31 tests plus independent hashlib/struct vectors passed. Workspace 164; release
personhood 57 reruns; legacy demo/docs command/audit passed. T02 config/proof binding
verified; T03 construction and T20 config bounds partial rows only. V1 unchanged.
No node, monetary/dependency/lockfile/runtime data changed or real admission enabled.

## 12. Historical B2 acceptance — MIRROR-MB01B-B2-001

Source `af1dd66c4fb8811d8eab54a1c88a8bddae2ae80e`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44205064789 — SUCCESS.
Job 134468771581 full logs inspected. Rust 1.98.0 with committed Cargo.lock.

Implemented `v2/operation.rs`: immutable signed Enroll/Approve/Withdraw, exact
131-byte prefix and 291/227/227 signed sizes; bounded membership-only decoder;
strict signatures; complete-request enrollment IDs and signed operation IDs;
configuration binding; checked height/window/ancestor-age rules and a documented
trusted parent-chain resolver. Transfer=0 rejects. No state or money mutation API.

39 tests in `tests/operations_v2.rs` include bytewise message/signature tampering,
all truncations, size/kind/version edges, changed-seed contexts, weak keys,
seed/V1 signature separation, orphan/future/unknown anchors, inclusive windows,
u64 overflow and independent vectors. Standard-library reference computes bytes/
hashes/IDs; signature constants were generated separately with Python cryptography
and matched/strictly verified by Rust. No local Rust compiler was available.

Results: **203 workspace tests = 164 retained + 39 B2**. **96 personhood release
reruns = 26 V1 + 31 B1 + 39 B2**, not new unique tests. Both Python references,
legacy demo, new authorization/expiry demo, doc command (zero cases) and allocation/
runtime-data audit pass. The B2 demo accepts the synthetic message at height 1 and
rejects it at height 9 as expired, without activating membership or issuing money.

T04 membership portions, T05 membership signature binding, T06 target binding and
T09 synthetic-parent checks are verified subcases; T10 window checks verified at
the message layer. Transfer/stateful reuse/real-ancestry integration remain later.
Sequence fields are signed, not consumed. Unknown target or exhausted approval
sequence can pass contextual checks; B3 MUST reject against state. Parent resolver
is trusted input, not proof of real ancestry. Neither Gate A/B nor MB-01B overall
is complete. No V1 encoding/config vector, monetary rule, node or local data changed.


## 13. Historical B3 acceptance — MIRROR-MB01B-B3-001

Published implementation: `b6f1233d038901aa0a13c8ddac9afbc34418e846`.
Published tree: `e98daf435335e31440e6c0bf14f28f5928f556c7`, matching the locally
tested source. Exact-head push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317174000 — SUCCESS.
Push job 134846238261 full logs and exact checkout were inspected. PR CI
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317184524 also passed;
its job 134846284741 logs were inspected too.

Implemented `v2/state.rs`: immutable MembershipStateV2/RegistryV2, verified
bootstrap, retained records/key/evidence indexes, bounded staged block execution,
exact actor sequences, expiry even in empty blocks, self-withdrawal, parent-Active
and surviving reviewer checks, deferred activation in original inclusion order,
checked counters and final-state high-water. Failure discards the complete candidate.

The trusted parent view must match configuration, height and stored tip association.
`bind_to_validated_block` attaches external provenance only; it does not validate a
real block, PoW, genesis or state root. The tag is excluded from canonical state to
avoid circular block/root hashing. Actual outer-chain validation remains MB-02.

All configured membership/count/byte limits apply to direct block input. Explicit
synthetic approval budgets are 64 retained approvals per record and, separately,
at most configured max_records approvals across the whole registry. History is
never evicted; at full capacity withdrawal and expiry remain available. These
finite limits do not guarantee continued admission after capacity exhaustion.
They do not change MCF2/B2 bytes or establish a production reviewer policy.

**51 new tests**: 45 in `tests/state_v2.rs`, six private arithmetic/invariant tests
in `src/v2/state_tests.rs`. Local Rust 1.98.0/locked-dependency validation passed:
**254 workspace tests = 203 retained + 51 B3**, and **147 personhood release
reruns = 96 retained + 51 B3**. Release reruns are not extra unique tests.
Documentation tests pass with zero cases; both independent B1/B2 reference checks,
all three synthetic demos, formatting of new Rust files and allocation/runtime-data/
lockfile audit pass. The new `state_v2_demo` demonstrates admission, withdrawn-reviewer
weight removal, complete rejection and successful retry without money or real people.

The [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md) maps actual tests to scoped T03/
T06/T07/T09 and T11–T20/T22 behavior. Private fault injection covers otherwise
unreachable sequence/height/ordinal exhaustion and damaged accounting indexes;
there is no production mutation API for constructing those corrupt states.

At the B3 checkpoint, B4 codec/root/replay was NOT implemented. State field ordering/tags and capacity
arithmetic are frozen in [MEMBERSHIP_STATE_V2.md](docs/MEMBERSHIP_STATE_V2.md),
without claiming independent root vectors or snapshot trust yet. Live ledger
blocks, mixed money/membership atomicity, durable branch recovery, private matching,
human-round consensus, fee/grant policy and device readiness remain later tasks.
Legacy bytes/data, founder policy and all unresolved economic choices are preserved.


## 14. B4 acceptance — MIRROR-MB01B-B4-001

Published implementation: `21e2272b5ac92c00e840d7aaa26a99570a5ec24f`.
Published tree: `31dfafa463935c640cd5cb31b88588575fc06d4b`, matching the locally
tested source. Exact-source push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870 — **SUCCESS**.
Job 135155424668 full logs and exact checkout inspected. PR CI
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404351436 also passed.

Implemented `v2/state_codec.rs` canonical MPS2 bytes/root and bounded untrusted
snapshot decoding, plus `v2/replay.rs` MPH2 grouped replay through the B3 executor.
The snapshot wrapper exposes no runtime-state conversion. Structural invariants
and retained signatures cannot prove historical withdrawals or canonical authority;
`verify_against` compares the entire cache with independently trusted executed state.

Replay binds verified bootstrap, every group's height/hash/exact signed-operation
commitment/intermediate membership root and a trusted expected C/height/tip/root.
Empty blocks retain expiry semantics. Stable metadata is fetched once per height.
Different histories that yield the same state are still rejected by the group digest.
Incorrect final executed state returns no partial registry. No disk writes occur.
Actual validated blocks and canonical-tip authority remain MB-02/03.

MPH2 enforces configured log/operation/block limits, plus 4,096 block groups and
1,048,576 state items charged before initialization/staging. Accounting includes
copied seed descriptors and synthetic allocations as well as records/approvals/new
record work. Checked arithmetic rejects overflow. The default 41,484,378-byte MPS2
snapshot bound is separate from the 4-MiB diagnostic log limit. Capacity rejection
never evicts history or weakens eligibility; these are finite development budgets.

`tools/reference_state_v2.py` independently computes five canonical snapshots/roots,
a four-group log and five group digests including genesis. Nine fixed signatures
were separately generated and verified with Python cryptography; the committed
checker requires only the standard library. Rust independently signs and executes.
All expected bytes/roots matched on first run. No failed Rust output became a vector.

**69 new B4 tests:** 26 snapshot, 10 independent-vector, 31 replay and 2 private
work-arithmetic cases. Actual local and exact-source CI results:

- **323 workspace/all-target tests = 254 retained + 69 B4, all passed.**
- **216 personhood release unit/integration reruns**, all passed.
- **1 compile-fail doctest** passed in the workspace doc command and again in release.
- All **3 independent references**, all **3 synthetic examples**, allocation/runtime
  audit and locked dependency checks passed. Changed Rust formatting and preservation
  audit passed locally; no legacy/dependency/MCF2/B2 byte or vector changes.

The state demo now verifies complete replay/cache agreement and rejects an old
prefix and altered history against trusted expectations. Review corrected omitted
configuration-copy work charges and strengthened the late-failure regression to
reach height 4 execution. This review is not an external security audit.

MB-01B is complete as a verified component foundation. No actual MRB2 ledger,
private deduplication, human-round consensus, anonymity, monetary grants/fees or
hardware support was enabled. Policy and Alice's historical fixture are preserved.
At the B4 checkpoint, the next task was MB-02A. Section 15 records its subsequent
completion. Gate A/B remain UNMET. NEXT_SESSION.md is the live continuation pointer.


## 15. MB-02A acceptance — MIRROR-MB02A-001

Published implementation: `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382`.
Tree: `930aa22bf9810525372fd31926812c1bde6dcf81`, identical to the locally tested tree.
[Exact-source push CI 44428924399](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399)
passed; job 135240927609 full logs and source checkout inspected.
[PR CI 44428936866](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428936866)
also passed. Later checkpoint-document commits have separate exact-head CI.

Implemented in mirror_core::v2: immutable signed TransferV2, unified typed
operations, immutable 160-byte block headers and bounded MRB2 bodies. Transfer
is 243 signed bytes; membership bytes stay 291/227/227; empty blocks are 168 bytes.
The existing address namespace remains, account/member sequences stay separate,
and core reuses personhood's config/operations/parent-view types through one
acyclic internal dependency. No external dependency version changed.

The operation root commits complete signed bytes, order and count. Count binding
separates odd-leaf lists from duplicate-last variants with the same bare Merkle
root. Header hashing uses its explicit V2 domain. Decoder preflight validates all
byte/count/record/subcount budgets before allocating the body vector. Construction,
encoding, root calculation and integrity checks independently enforce limits.

Strict signatures, config agreement and an exact operation root establish body
integrity only. Transfer context validation adds anchors/windows and the positive,
zero-fee static profile. Neither function establishes balances, current sequences,
full state execution, combined ledger roots, actual PoW/ancestry, pinned genesis or
canonical-chain acceptance. Boundary tests explicitly cover these omissions.

Independent tools/reference_blocks_v2.py supplies five operations, seven complete
blocks and nine header variations. Five public test signatures were independently
generated/verified with Python cryptography; the committed checker is standard-
library-only. Rust independently constructs and signs fixtures. All ten vector
tests matched on their first run; no failed Rust expectation was copied into fixtures.

**80 new tests: 37 operations + 33 blocks + 10 independent vectors.** Actual local
and exact-source CI results: **403 workspace tests**, **1 retained compile-fail
doctest**, **121 core release tests**, **216 personhood release tests plus the same
doctest**, all **4 references**, all **4 examples**, and allocation/runtime audit
passed. Changed-file formatting and preservation checks passed locally. Release
reruns are not extra unique tests. Reciprocal implementation reviews found no
blocking issue; this is component review, not an external security audit.

The new example builds a synthetic mixed block, checks round-trip/context/body
integrity and tamper rejection, and explicitly reports no executed balance/member
changes or accepted block. Legacy source/genesis/runtime data and prior MCF2/B2/
MPS2/MPH2 vectors remain unchanged; lib.rs adds only new-module wiring. Cargo.lock
adds only core's internal personhood dependency. No economic decision, founder
allocation, reserve, grant, real biometric or device interface was added.

**At the MB-02A checkpoint, next was MB-02B.** Its subsequent implementation is
recorded in section 16. The planned work was to stage accounts and membership, retaining original
full-block operation indices including transfers, and independently commit the
combined ledger. Current membership-only staging/MPH2 must not silently consume
filtered mixed blocks. MB-02C actual block authority and MB-03 durability follow.
MB-02 remains IN_PROGRESS; Gate A/B remain UNMET.


## 16. MB-02B acceptance — MIRROR-MB02B-001

Implemented 2026-11-20. Source `420d7114ddf782469a6ace6ba0a2c4076fb93aaf`, tree
`6871ad0fbc178bd76262a26416a7d255d9fcbd66`. Exact-source
[push CI 44619155972](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619155972)
**SUCCESS**; job `135895834032` full logs and checkout inspected.
[PR CI 44619172378](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619172378)
also **SUCCESS**. PR #1 remains open/unmerged; main is unchanged. Documentation
checkpoints follow this implementation and have separate exact-head CI.

`LedgerV2` and `StagedLedgerV2` combine immutable accounts and membership. Full-body
limits and exact parent association precede bounded cloning. Operations execute in
original full-block order; an error discards both components, including prior
transfers, approvals, sequence increments or expiry. Finish activates membership
under the preserved B3 rules, checks account capacity and conserves initial supply.
Transfers require positive amounts, zero fees, exact account sequences and checked
arithmetic. Funded self-transfers consume one sequence without changing balances.
Explicit zero and spent accounts retain capacity/sequence history. Membership status
does not add a new restriction on spending owned test balances.

Personhood's non-cloneable consuming indexed builder preserves mixed enrollment
positions and charges omitted transfers exactly as 4+243 bytes. No intermediate
state or failed partial candidate can be published. The original membership-only
path shares transition machinery without changing its old vectors. MPS2's runtime
index bound now uses max_operations rather than the membership count cap; the
explicit range extension changes no layout/domain. Snapshot decoding remains
untrusted. MPH2 remains membership-only, including existing replay budgets.

Accounts commit sorted address/balance/account-sequence records, including zeros.
An immutable initial-supply economics root and existing MPS2 root feed the combined
C/height/accounts/membership/economics commitment. Tip hash is external provenance.
[LEDGER_STATE_V2.md](docs/LEDGER_STATE_V2.md) freezes bytes, error order, capacities
and the boundary between staged execution and actual block acceptance.

The independent Python reference computes five explicit 1,000-Nusa states plus an
alternate valid mixed ordering. Sixteen signatures were independently generated
and verified before Rust; all eight vector comparisons matched on the first run.
No failed Rust output became an expected value. The synthetic ledger example shows
successful mixed execution and rollback in both directions; it accepts no actual
block, real enrollment or valuable currency.

Local and exact-source CI: **462 workspace tests = 403 retained + 59 new**.
New cases are **18 indexed staging, 28 ledger integration, 5 private arithmetic/
invariant faults and 8 independent vectors**. **Three compile-fail doctests** pass.
Release reruns: core **121**, personhood **234** plus the same **3** doctests, state
**49**. All **five independent references**, **five examples**, formatting and
preservation/allocation audits passed. Cross-reviews found no blocking discrepancy;
these are not an external security audit or a proof of mainnet readiness.

Legacy source behavior, previous vectors, monetary constants, historical genesis
and runtime data are preserved. Only one internal state-to-personhood dependency
was added; no external package changed. No founder bonus, reserve, grant, mint,
fee/reward policy, biological duplicate matcher or device integration was enabled.

**At the MB-02B checkpoint, next was MB-02C.** Actual genesis/work/ancestry/root
acceptance and complete MRB2 replay were still incomplete then; section 17 records
their subsequent implementation. A supplied provenance hash alone remains weaker
than that validation. Durable canonical recovery remains MB-03. Gate A/B remain unmet.


## 17. MB-02C acceptance — MIRROR-MB02C-001

Implemented 2026-11-20. Source `db2cb58ba5a9a828ec949debd1b52b5915a66f3b`, tree
`3fa1d3b44abb885898cc256f9525355025649a0a`, identical to the locally tested tree.
[Exact-source push CI 44623282448](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623282448)
**SUCCESS**; job `135910328823` full logs and exact checkout inspected.
[PR CI 44623296178](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623296178)
also **SUCCESS**. PR #1 remains open/unmerged; main is unchanged. Documentation
checkpoints follow the implementation and require separate exact-head evidence.

`NetworkManifestV2` adds bounded MNM2 public metadata: exact MCF2 bytes, every ordered
seed proof and an exact genesis hash. Decoding requires an independently selected
configuration/genesis identity. Genesis must still satisfy its actual fields,
empty body, configured target/work and executed bootstrap ledger root. Another
valid nonce cannot replace the selected genesis. No peer or self-consistent cache
can select the manifest's authority implicitly.

`ChainV2` owns accepted full blocks and the combined ledger. Complete-body and
aggregate budgets precede context/height/parent/time checks, exact configured work,
operation commitment and original-order atomic ledger execution. The claimed root
must match the calculated combined state before actual-hash binding and installation.
No separate signature prepass masks an earlier execution failure. Failed acceptance
leaves blocks, ledger, manifest and counters unchanged. Candidate preparation is
read-only; every mined proposal still requires complete acceptance.

The private accepted prefix supplies every operation anchor. Complete typed/encoded
MRB2 replay validates through the same path, including empty blocks, against external
C/genesis/height/tip/ledger-root expectations. A corrupt intermediate block, old
valid prefix, wrong root or foreign-fork anchor cannot produce an accepted output.
The caller still chooses the desired finite branch; latestness, canonical choice,
durable recovery and finality are separate remaining tasks.

[CHAIN_V2.md](docs/CHAIN_V2.md) freezes manifest bytes, acceptance/error order and
resource/time semantics. Inclusive caps are 4,096 blocks including genesis, 16 MiB
complete MRB2 and 1,048,576 cumulative abstract state/configuration items. Genesis
charges twice seed-plus-allocation entries; each child charges configuration,
current accounts/records/retained approvals and incoming Transfer/Enroll/Approve.
Checked cap exhaustion rejects without deleting history. This is a finite synthetic
profile, not a production capacity benchmark or monetary population limit.
Timestamps strictly increase over the parent with no clock/future ceiling;
`u64::MAX` therefore has no successor. Work uses the exact configured compact bits,
raw big-endian V2 header hash and a separate local bounded mining helper. It is
still development work, not final equal-human consensus or mining-reward policy.

The sixth independent Python reference was frozen before Rust execution: exact
manifest and 20 independently generated/verified signatures, five actual mined
main blocks, an empty expiry block and a genuine membership-divergent branch.
The fork removes reviewer A, leaving C pending where the main branch activates C,
while account states match. Alternate valid genesis, re-mined wrong ledger root
and cross-fork anchor fixtures fail at their intended boundaries. All nine Rust
vector tests matched on first execution; expected bytes were not adjusted to Rust.
The new example runs two separate in-memory validators, accepts the same complete
mixed block, rejects a re-mined false root on both, and reconstructs full encoded
history to the separately retained checkpoint. No network or durable store is run.

Local and exact-source CI: **523 workspace tests = 462 retained + 61 new**.
New cases: **15 work + 14 manifest + 20 public chain + 3 private resource + 9 vectors**.
**Three retained compile-fail doctests** passed. Release reruns: core **121**,
personhood **234** plus the same **3** doctests, state **49**, consensus **23**,
chain **57**. All **six independent references**, **six examples** and allocation/
runtime audits passed. Focused formatting and preservation checks passed locally.
Exact/+1 resource arithmetic is covered; a maximum-million-item stress run is not
claimed. Independent cross-reviews found no blocking discrepancy; this is not an
external security audit. Release reruns do not add unique tests.

Legacy source behavior, runtime data, prior fixtures and economic constants remain
unchanged. One internal chain-to-personhood dependency was added; no external
package changed. No special founder or marketing allocation, grant/mint/reward/fee
formula, private matcher or device interface was enabled. D-01/D-04 remain open.

**At the MB-02C checkpoint: MB-02 VERIFIED_COMPONENT — IN MEMORY; next was MB-03A.** Freeze separate V2 storage
identity, selected-tip metadata, locking and recovery, then implement explicit
initialization/opening and writer ownership. Subsequent MB-03 work must make actual
block/state publication durable, handle I/O faults and synchronize/reorganize
membership-bearing histories across nodes. **Gate A/B remain UNMET.**

## 18. MB-03A acceptance — MIRROR-MB03A-001

Implemented source **528c8912c5a50cfd2e86856d345b29d6bb79d2c5**, tree
**65004e73f4c19deedafdb8cf2874939df0ab367a**, on the active feature branch.
PR #1 remains open/unmerged and main is unchanged. Exact-source push CI is
**44639322362**, job **135965942902**; source PR CI is **44639329630**.
Both source runs SUCCEEDED. Full push-job logs were inspected and matched the exact
source checkout and all gates. Documentation follows source and has separate exact-head verification.

The frozen [storage contract](docs/STORAGE_V2.md) and separate
`mirror_storage::v2::StoreV2` implement **immutable selected-history storage**.
Explicit initialization takes an already accepted finite ChainV2; existing
nonempty/partial/foreign/legacy stores are not overwritten or automatically repaired.
Writer reopening and shared inspection require independently retained C/genesis
and full height/tip/ledger-root expectations. Every file is bounded and the complete
actual MRB2 history is replayed through normal acceptance before a handle returns.
Accounts, membership, permanent records and cumulative budgets are reproduced;
no snapshot or persisted counter can become authoritative state.

New MSS2 is exactly 232 bytes, encoding version 2/profile 1/generation 0. It records the
selected checkpoint, count/length information and integrity checksums for exact
MNM2/record bytes. It preserves all existing formats/hash domains and carries no
biometric archive, real private key or economic privilege. Successful encoded input
is bounded by 19,559,084 bytes; this is not total resident memory. Complete MRB2 limits
remain 4,096 blocks / 16 MiB and existing replay work limits remain unchanged.

The root directory's private un-cloned Linux descriptor holds an exclusive writer
or shared reader lock for handle lifetime. Real separate-process tests prove
exclusion, coexistence, release on Drop while process stays alive, and release on
process termination without lock files/PID cleanup. Initialization synchronizes
files and containing directories before publishing selection.v2 last through a
prepared marker; an uncertain publication returns no handle. A checked reopen
reconciles visible content against external expectations. Static path aliases are
rejected under a trusted parent/cooperating-process assumption; hostile same-user
namespace changes are excluded. The live root inode must not be replaced.

Local acceptance: **588 workspace passes = 523 retained + 65 new**, comprising 24 selection
codec/vector, 34 filesystem, 5 lock/process and 2 initialization-fault tests. One subprocess
helper is explicitly ignored by normal runs and invoked by parent tests, not counted
as a pass. The 2 fault tests cover 9 injected boundaries; they are not actual power-loss
or comprehensive syscall-failure simulation. The 3 existing compile-fail doctests
passed. Release reruns passed core 121 / personhood 234 / state 49 / consensus 23 / chain 57 /
storage 75 (including 10 legacy), plus the same 3 doctests and 1 ignored helper. All 7 references
and 7 examples passed. Focused formatting, legacy/fixture/runtime preservation and
dependency checks passed. Release reruns are not additional unique tests.

The seventh Python reference independently derived genesis/main/membership-fork
selection records from the unchanged MB-02C fixture before Rust comparison. New
fixture SHA-256: `ab25051450c08774528fa14c253072a7856824051f526a2b7ea331adc34902ef`.
The example initializes/reopens exact full selected history, rejects a second
writer and another branch's checkpoint, and independently recovers a membership
fork in newly created temporary stores. It does not launch a V2 network or reorg
an existing store. Independent implementation reviews found no blocking issue
within the stated assumptions; they are not an external security audit.

Linux tmpfs/overlayfs behavior was directly exercised locally. WSL2 native Linux
storage is intended but not directly verified; Windows/DrvFS/NFS/SMB remain
unverified. OS gating does not detect mount types. Successful post-error inspection
does not retroactively prove a failed sync survives future power loss. The bounded
root-entry scan may report InvalidLayout before a more specific legacy/foreign
classification; it still preserves data.

Source changes: 16 files, 3,355 insertions / 2 deletions. Only one internal storage-to-chain
dependency was added; no external package changed. Prior crate source, independent
fixtures, node behavior, genesis/runtime data and economic rules remain intact;
legacy storage only gains a separate V2 module. Source CI now includes the seventh
reference, storage release tests and the new demonstration.

Initial source `2370ff5df3a73ff874a8fb0031df9e2a98775a43` passed PR run 44638347110
but failed push run 44638337326 in a release ownership test. Local reproduction
confirmed that one test's concurrent fork could temporarily inherit another test's
lock descriptor. The corrected source serializes only the five parent test cases,
preserving their real concurrent child checks and immediate assertions; no production
retry, sleep or lock behavior changed. All 200 repeated release executions passed,
then the corrected workspace/docs/storage-release gates and the full exact-source
CI passed. Repetitions are not additional unique tests. Last-descriptor ownership,
including fork inheritance, is now explicit in the storage contract and handoff.

**At the MB-03A checkpoint: VERIFIED_COMPONENT — IMMUTABLE STORE; next was MB-03B.**
No accepted-tip advancement, durable reorg, V2 peer/node integration, mint, reward,
real device admission or anonymity has been delivered by this slice. D-01/D-04
remain open; no founder bonus or privileged marketing reserve was added.
**Gate A and Gate B remain UNMET.**


## 19. MB-03B acceptance — MIRROR-MB03B-001

Implemented source **ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c**, tree
**06d7c3f4fa24d96c844e2301c76c01f804151192**, on `work/personhood-register-v0.1`.
PR #1 remains open/unmerged; main remains ad7ffd120fb451fe44cf59cb909176785b33bf8f.
[Source push CI 44649111203](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203),
job **135999222947**, and [PR CI 44649119266](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
both SUCCEEDED. Full push-job logs were inspected and confirmed this exact source
checkout and all test/reference/example/audit gates. The published tree matches the
locally checked tree. Documentation follows this source with separate exact-head CI.

### Delivered behavior and authority

The [append contract](docs/STORAGE_APPEND_V2.md) and `AppendStoreV2` introduce explicit
profile-2 initialization, one-child append, exclusive writer reopen, shared inspection
and consuming clean-state reconciliation. They preserve the immutable StoreV2 profile
and all prior network, transaction, block, ledger, membership and manifest bytes.
Initialization accepts only an absent or empty root from a fully accepted ChainV2.
No legacy/profile-1 adoption or in-place migration occurs.

MSU2 is exactly **280 bytes**: magic, version 2, profile 2, local u64 generation, exact
unchanged 232-byte MSS2 snapshot, and a SHA256d checksum under the new
MIRROR_STORAGE_APPEND_SELECTION_V2 domain. The outer generation starts at zero even
for a nonzero initial chain height, then advances with every published child. It
never exceeds height. An uncertain return may already have selected that generation.
Expected generation and full C/genesis/height/tip/root pins are independent caller
inputs; no checksum, pending file or highest generation chooses trust or freshness.

Append validates a cloned complete child through ChainV2 before disk mutation, then
fully replays the current selected disk history under the held lock against retained
pins and compares all state/counters with memory. Bad candidate validation leaves the
store usable and unchanged; disk discrepancy fences it. Publication creates/writes/
synchronizes the new block, synchronizes blocks/, creates/writes/synchronizes the
pending selector, compares the old marker, renames the pending marker and synchronizes
root. Only after that does memory advance. The locked root inode is never replaced.

Any uncertain write leaves the owning handle fenced and blocks normal chain/selection/
checkpoint access and further append. The explicitly named last_confirmed_checkpoint
is prior confirmed memory, not a claim about disk after failure. Error values distinguish
incomplete preparation from publication uncertainty and carry the locally validated
candidate checkpoint for the caller to retain. Consuming reconcile retains the same
exclusive lock while fully replaying the independently selected history and syncing
all selected files/directories; it returns an unfenced writer only after success.
Writer reopen likewise resynchronizes. Shared inspection performs no application write,
sync or repair. Unknown and partially prepared states are preserved and rejected.

### Actual verification

**658 workspace passes = 588 retained + 70 new**, plus three retained compile-fail
doctests and one intentionally ignored subprocess helper. The new tests are:

- 28 exact selector/independent-vector and negative-format cases.
- 31 filesystem/append integration cases using previously frozen actual blocks.
- Eight private initialization, append, failure-fencing and reconciliation tests.
- Three real subprocess ownership cases added to the five retained parent tests.

The private tests exercise 11 append boundaries, eight reconciliation boundaries
through both writer-reopen and consuming-reconcile entry points, nine initialization
boundaries, partial block/selector files, stale pins, root replacement and concurrent
selector changes before publication. These scenarios and release repeats are not
additional unique tests. Cases assert complete old/new selected state or rejection,
retained bytes, fence persistence and no memory advancement before successful sync.

Release reruns passed core (121), personhood (234 plus the same three doctests),
state (49), consensus (23), chain (57) and storage (145), with the same helper
ignored during ordinary enumeration.
All **eight independent references** and **eight synthetic examples** passed locally
and in exact-source CI. The final workspace check passed after the last comment/
example wording edits. Focused formatting and diff checks passed. Ninety retained
source/dependency/reference/fixture files were compared byte-for-byte; all dependencies,
prior fixtures and runtime data remain unchanged.

The new Python reference generated MSU2 genesis, sequential main-history, selected
nonzero-height initialization and membership-fork vectors independently before Rust
comparison. Fixture SHA-256 is
`d6e597fdac8c5be081a54019149ebbe79651817848139d51bc691681b96df668`.
No expected value was changed to agree with Rust output. New example append_v2_demo
publishes and reopens heights 1–4, reproducing complete accounts/membership/counters,
rejects a wrong-height candidate and second writer, and performs consuming clean
reconciliation. It uses only new owned temporary directories and public fixtures.

The three new process cases cover a child writer dropping its store while remaining
alive, writer termination and shared child/local inspection, each followed by checked
reopen and a real accepted child append. All eight parent tests retain the mutex
isolation required by the earlier fork-descriptor issue. No sleep, production retry,
assertion weakening or explicit unlock was added. Two source/contract reviews found
no blocking discrepancy under the stated assumptions; this is component verification,
not an independent security audit or a power-loss/storage-firmware certification.

### Preserved boundaries and continuation

Source increment: **14 files, 3,568 insertions and 11 deletions**. No dependency change.
Existing storage locking gains only a private stable-root identity check used by the
new profile. Old immutable example wording now points to the separate append profile;
its behavior is unchanged. The old immutable executable store and all prior wire/hash
semantics are retained. CI adds the eighth reference and eighth example.

The selected profile-2 input ceiling is **19,559,132 bytes**, distinct from resident
memory and staging overhead. Chain limits remain 4,096 blocks, 16 MiB of MRB2 bytes
and 1,048,576 cumulative state/configuration items. There is no history eviction, authoritative account/cache
restore or change to consensus/economic rules. Linux cooperating processes and trusted
local owners/parents, correct locking/atomic rename/file+directory sync remain explicit
assumptions; WSL, DrvFS, native Windows and network mounts are not established by Ubuntu CI.
Inherited fork descriptors can keep a lock alive until their last reference closes.

**MB-03B VERIFIED_COMPONENT — BOUNDED APPEND / CLEAN-STATE RECONCILIATION.**
**At the MB-03B checkpoint, MB-03 remained IN PROGRESS and MB-03C was next.**
Extra tail blocks, pending markers and partial/unknown files were rejected and
preserved. An interrupted preparation could therefore make a selected prefix
unavailable through normal open pending an explicit recovery facility. MB-03C was
to resolve recovery authority, bounded inspection and interruption/retry semantics
without silently reinterpreting MSU2 or deleting unexplained data. Its delivered
scope is recorded in section 20; normal opening continues to reject staging.
Canonical fork selection, durable membership-aware reorganization and peer reconstruction
follow. MB-04 node/pool/wallet/operator APIs and MB-06 human finality remain later work.
No founder/marketing privilege, mint, grant, nonzero fee/reward, real biometric capture,
private matching or anonymity was added. D-01/D-04 remain OPEN; Gate A/B remain UNMET.

## 20. MB-03C acceptance — MIRROR-MB03C-001

Implemented source **e8302052ab724f59ff4a212dc64aea9575a6d4b3**, tree
**dc9eb2c02436f0952ac54593ba8d02317e9d9314**, on `work/personhood-register-v0.1`.
PR #1 remains open/unmerged; main remains ad7ffd120fb451fe44cf59cb909176785b33bf8f.
[Source push CI 44748718086](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748718086),
job **136330677456**, and [PR CI 44748724632](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748724632)
both **SUCCEEDED**. Full push-job logs were inspected: the checkout is exactly this
source, and every required test, reference, example and audit step passed. The
published tree equals the locally verified source tree. Documentation follows in a
separate commit whose exact-head CI is checked and recorded separately in PR #1.

### Delivered behavior and authority

The [recovery contract](docs/STORAGE_RECOVERY_V2.md) and
`mirror_storage::v2` recovery API add an explicit way to resolve recognized
interrupted one-child appends. Static `AppendStoreV2::recover_interrupted` takes
exclusive ownership of an existing store. Consuming `AppendStoreV2::recover`
keeps the same root lock, including when the original writer has been fenced.
Neither entry point initializes, adopts, migrates or chooses a store from its files.

`AppendRecoveryRequestV2` holds independently retained parent and candidate
checkpoints plus the exact complete candidate `BlockV2`. The checkpoints must
describe adjacent height and local generation, with the same configuration and
genesis. The request constructor checks structural consistency; actual recovery
fully replays the selected parent and accepts the child through normal `ChainV2`
rules. It derives exact old/new MSU2 metadata, including manifest commitment and
byte totals, rather than treating a valid checksum as authority.

The caller explicitly chooses one of two actions:

- **Complete** extends only missing canonical suffixes of the authorized child
  and pending selector, synchronizes preparation, publishes the intended candidate,
  then performs strict clean replay and synchronization before returning a writer.
  Repeating completion after the candidate was selected does not advance generation.
- **RetainParent** is permitted only while the parent remains selected. It removes
  only matching authorized pending/tail artifacts, synchronizes removals in the
  specified order, and replays/resynchronizes the parent. A selected child is never
  rolled back. Full candidate validation is still required for this action.

Supported parent-selected layouts are: no staging; an empty, proper-prefix or
complete next child without pending selection; or a complete child with an empty,
proper-prefix or complete intended pending selector. Candidate-selected completion
requires the complete child and no pending marker. Pending selection with a missing
or partial child, unknown bytes, extra heights, unsafe aliases, wrong pins and
corruption of selected history are rejected without cleanup.

Prefix equality, especially an empty/common prefix, cannot establish who created
a file. The independently retained caller request authorizes these exact staging
paths; byte matching limits what may be changed. This component does not persist
that request or derive it from peers, damaged files, a checksum or a high generation.
It checks safe current file identity, not original inode provenance from some
earlier process. Hostile same-user races remain outside the trusted-owner model.

### Publication, retry and resource boundaries

Recovery first synchronizes the fully validated parent manifest/history/marker and
directories. Completion opens and checks existing prefixes without truncation,
appends only the missing canonical suffix, rechecks descriptor identity and exact
bytes, and synchronizes the prepared child and blocks directory. It prepares and
checks pending selection similarly and rechecks the selected marker and complete
artifacts immediately before rename. The stable root is synchronized after
publication. Retention removes pending selection and synchronizes root before
removing the unselected child and synchronizing blocks/.

Success requires the existing strict clean loader, complete `ChainV2` replay and
equality of accounts, membership, permanent records and resource counters, followed
by selected-file/directory synchronization. Any error returns no usable handle.
Incomplete preparation/cleanup is distinct from uncertain publication; once
publication was attempted, or the candidate was already selected at entry, an
error does not imply that the parent remains selected. The retained request permits
explicit retry only when the resulting layout is one of the recognized states.

There is no new format, profile, journal, domain, fixture or dependency. Ordinary
`open_writer`, `inspect` and clean `reconcile` remain strict and do not invoke
recovery. The root inode is not replaced. No selected block is deleted, no foreign
bytes are truncated or overwritten, and no cached state replaces replay.

The distinct recovery input ceiling is **19,559,412 encoded bytes**, including
the complete intended child even if its current file is empty and both selectors.
The enforced conservative per-call read allowance is **156,508,096 bytes**.
Every payload/sentinel read is charged before reading, including repeated prefix
and marker comparisons and the final strict loader phase. Metadata/enumeration is
also bounded. These ceilings are neither RSS limits nor write-volume promises.
Existing chain caps remain 4,096 blocks, 16 MiB of complete MRB2 bytes and 1,048,576
cumulative abstract state/configuration items. No history is evicted to fit them.

### Actual verification

**707 workspace tests passed = 658 retained + 49 new**, plus three retained
compile-fail doctests. One subprocess-only helper remains intentionally ignored
during ordinary enumeration; parent tests invoke it explicitly. The 49 added
functions are **39 integration tests, nine private fault tests and one real-process
parent test**. Boundary iterations and release reruns are not extra unique tests.

The nine private tests cover 28 completion boundaries over three initial layouts
(84 scenarios), 21 retention boundaries, 16 already-selected synchronization
boundaries, and all 11 existing append-failure boundaries with both actions. Other
cases cover root/selection/layout changes after validation, altered prefixes before
extension or removal, changes before publication, aliases introduced after opening,
and exact input/read-budget accounting. Successful retries are followed by the next
ordinary append to demonstrate a usable returned writer.

Integration cases exercise supported partial/complete states, idempotence, wrong
or stale pins, invalid candidate work and ledger roots, unknown data, unsafe paths,
sparse oversized files, aggregate bounds before payload decoding, checksum-correct
but inaccurate selector totals, and continued strict ordinary opening. They compare
complete reconstructed state, including membership-divergent branches with equal
account bytes and empty-block expiry effects.

The real-process test starts a child that completes recovery and holds its returned
writer. Concurrent parent recovery, writer opening and shared inspection must each
return `Busy`. Both handle-drop-while-child-alive and child-termination scenarios
are followed by checked reopening, stable-root verification and the next real block
append/reopen. It retains the existing helper/mutex isolation without sleeps,
production retries or weakened ownership assertions.

Local and exact-source CI release checks passed: core **121**, personhood **234**
plus the same three doctests, state **49**, consensus **23**, chain **57**, and
storage **194** with the same one helper ignored. All **eight independent Python
references** and **nine synthetic examples** passed. The new `recovery_v2_demo`
completes a partial block, retries completion without a new generation, retains
the parent from a fenced writer with matching staging, then appends/reopens again.
Its final tip is
`36b8ccbfdaa75ae33f3e522f7f636ddf02ff2c6b4c23df4ea694d207ada01c6b`
and final ledger root is
`3f0c9f7ef212a7cae71717004d7a0206f027109490be8dcd07cba032ce32f86e`.
Only synthetic fixtures and invocation-owned temporary directories are used.

Source increment: **9 files, 3,138 insertions and 1 deletion**. Focused formatting,
diff and dependency checks passed locally. **99 retained source/dependency/reference/
fixture files** were compared byte-for-byte and remain identical. Two source/contract
reviews found no blocking discrepancy under the stated cooperative ownership model.
This evidence does not simulate physical power loss, certify firmware or establish
recovery for every storage failure.

### Preserved limits and exact continuation

**MB-03C VERIFIED_COMPONENT — EXPLICIT ONE-CHILD RECOVERY.**
**At the MB-03C checkpoint, MB-03 remained IN PROGRESS and MB-03D was next.**
That next slice was to freeze score arithmetic, overflow, ties and common-ancestor
behavior for fully accepted branches with the same independent configuration/genesis,
after inspecting the fixed-literal-target profile. It was to avoid hash luck, local
storage generation, peer claims or population as branch-selection authority, preserve
complete candidate state, and remain separate from durable branch replacement.
Section 21 records its subsequent implementation. Durable replacement,
membership-aware peer reconstruction and persistent node/API work remain unfinished.

Recovery for interrupted initialization, missing/corrupt selected history, or lost
independent intent is not implemented. Linux cooperating processes, trusted local
owners/parents and correct locking/rename/file+directory synchronization remain
assumptions. WSL, DrvFS, native Windows and network mounts have not been verified by
Ubuntu CI. No human consensus, mint/grant/nonzero fee/reward, founder/marketing
privilege, real biometric intake, private matching or anonymity was added.
D-01/D-04 remain OPEN. **Gate A and Gate B remain UNMET.**

## 21. MB-03D acceptance — MIRROR-MB03D-001

Implemented source: **b9a2e870996fcafe1128dcb5cfdfb6b01b158ded**.
Tested source tree: **dd0a3fc29f429dcd4999261801c2bc7897d6aace**.
Branch: `work/personhood-register-v0.1`. PR #1 remains open and unmerged;
main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Published-source [push CI 45703714223](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703714223)
and [PR CI 45703726306](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703726306)
both passed. Full push-job `139520118282` logs confirmed the exact implementation
checkout, **735 workspace tests, six compile-fail doctests, all required release
suites, eight independent references, ten examples and the allocation/runtime-data
audit**. The single intentionally ignored subprocess helper is invoked by parent
tests and is not another pass. Release reruns are not additional unique tests.
MB-03D is **VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON**. Subsequent
documentation-head CI is checked separately and recorded on PR #1.

The local results below describe the frozen implementation. Source publication and
exact-source CI evidence belong above; later documentation-head CI is checked
separately. The preceding handoff was
`c3de6f4be32fd2c22523f06674a96295da8ec48e`; its earlier push/PR checks do not
establish acceptance of this new comparison code.

### Delivered behavior and authority

The [fork-choice contract](docs/FORK_CHOICE_V2.md) and
`mirror_chain::v2::compare_branches_v2` compare two complete, already accepted
`ChainV2` values. The caller independently selects `ManifestIdentityV2`, which pins
configuration hash C and the exact genesis hash. The function checks current C,
current genesis, candidate C and candidate genesis in that order before scoring
or ancestry comparison. Wrong configuration/genesis errors identify the input
side. Raw blocks, peer heights, state caches and arbitrary claimed scores cannot
substitute for the accepted-chain types.

Every accepted block uses the same literal target configured by C. The normalized
score is therefore the accepted post-genesis block count, equal to height and
bounded to `0..=4095`. For a positive constant target-derived contribution W,
comparing total work `(height + 1) * W` is exactly comparing heights. This is an
ordering argument: the implementation performs no work multiplication, accumulated
large-integer arithmetic or score decoding. It does not measure actual mining
attempts. A strictly higher candidate score wins; all ties retain current.

An unusually low block hash, a larger nonce, a later timestamp, more money,
membership/population counts and local storage generation add no weight. These
rules apply only to the fixed-target development profile. A future variable-target
or human-weighted profile requires a separately reviewed scoring contract. Easy
PoW does not make hardware equally fast or establish equal-human consensus.

Accepted hashes are compared from genesis until the first mismatch or the shorter
history ends. The result distinguishes identical histories, either proper-prefix
direction and genuine divergence. Its common ancestor is the complete checkpoint
of the last matching accepted block, including its actual hash and ledger root.
Equal account bytes or membership roots alone cannot establish shared ancestry.

`BranchComparisonV2` has private fields, no public constructor and no encoded
certificate. It immutably borrows both inputs; safe Rust prevents either input
from changing while that result remains in use. Its selected-chain reference is
the complete original accepted branch, including manifest, blocks, accounts,
membership, permanent reservations, approvals, separate sequences, population
history and resource counters. It combines no state across forks and mutates
neither input.

Copied choices/checkpoints are diagnostics and can become stale. They are not
durable commit tokens. A later storage operation must independently recheck network
identity, the exact current selection and the complete candidate under exclusive
ownership before publishing. This comparison performs no disk or network operation
and does not change append/recovery behavior.

### Actual local verification

Full local gates passed with Rust 1.98.0 and locked dependencies:

| Gate | Observed result |
|---|---|
| Workspace, all targets | **735 passed**, 0 failed; 707 retained + 28 new integration tests |
| Workspace documentation tests | **6 compile-fail cases passed**; 3 retained + 3 new |
| Release core | **121 passed** |
| Release personhood | **234 tests + 3 retained doctests passed** |
| Release state | **49 passed** |
| Release consensus | **23 passed** |
| Release chain | **85 tests + 3 new doctests passed** |
| Release storage | **194 passed** |
| Independent reference scripts | **All 8 passed** |
| Synthetic examples | **All 10 passed**, including `fork_choice_v2_demo` |
| Formatting, dependencies and preservation | Passed; **105 retained files byte-identical** |

The existing subprocess helper is intentionally ignored in workspace/storage
enumeration and is invoked by parent tests; it is not another passed test.
Release reruns and multiple assertions/scenarios within one test are not additional
unique tests.

The 28 new integration tests cover independently wrong network identities on each
side with exact error precedence; genesis-only and identical branches; both prefix
directions; shorter, equal and longer genuine forks; ties in both argument orders;
common ancestors at genesis and deeper heights; rejected peer blocks; terminal
timestamps; and complete histories at the 4,096-block cap. Frozen forks with equal
account bytes and different membership verify that selection preserves the exact
chosen approvals, permanent records, population history and resource counters.
Empty-block expiry remains attached to its own branch. The three new compile-fail
doctests reject mutation of either borrowed input and a raw `BlockV2` passed where
an accepted `ChainV2` is required.

The new synthetic example compares two accepted height-3 histories with equal
account bytes and different membership. It keeps current on the tie, then accepts
an empty child on the candidate and prefers that complete height-4 branch. The
common ancestor remains height 1. Observed selected tip:
`74ace822b0cbdc059fc8cfef336f5f3c88668a76ca437107443af0279860f666`.
Ledger root:
`2654699af4ccfeeffb0fca1b7b9ade6a76201134285de1737bb10cc964cbe0fd`.
Active / pending / historical high-water counts are **1 / 1 / 2**;
encoded history bytes / cumulative state items are **3,605 / 73**.
The example uses public frozen synthetic fixtures and publishes no store.

Preservation checks compare against the preceding handoff. **105 existing source,
dependency, reference and fixture files remain byte-identical.** The existing
chain acceptance implementation is unchanged; its module gains the new exports
and documentation. CI adds the tenth example. No dependency, encoded format,
hash domain, reference vector, fixture or storage profile changes. Two independent
source/contract reviews found no blocking discrepancy under the accepted-chain
invariants. These reviews and software tests are not an external security audit.

### Finite scope and exact continuation

The comparator performs at most **4,096 block-pair comparisons / 8,192 fixed-header
hashes**. It uses constant additional state, allocates no heap collection, clones
no branch and replays no ledger. Each input independently retains the existing
4,096-block, 16-MiB MRB2 and 1,048,576 cumulative state/configuration-item budgets;
comparison neither merges nor relaxes them. Those are not total process-memory
limits.

Keep-current ties are deterministic for the ordered inputs, not a unique global
winner. Peers can retain different equal-score tips until a strictly stronger
accepted branch appears. At a finite cap or terminal timestamp, extension may be
impossible. This component establishes no eventual-convergence, partition-liveness
or human-finality guarantee.

**At the MB-03D checkpoint: VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON;
MB-03 remained IN PROGRESS and MB-03E was next.** Section 22 records the later E1 slice.
Freeze a reviewed storage layout and publication/recovery contract before writing
replacement code. Retain independent network/current checkpoint and generation
pins, full candidate validation/comparison under the stable exclusive root lock,
bounded staging and reads, and explicit failure/retry intent. Publish all accounts,
membership, permanent history, approvals, sequences, population peaks and resource
counters with the selected history; orphaned effects must not survive. Unknown or
ambiguous staging remains preserved/rejected. Do not silently reuse one-child
Complete/RetainParent as a generic reorganization operation or change MSS2/MSU2
meaning. Any new persisted profile/bytes require versioned documentation and
independent vectors.

Acceptance must cover genuine membership-divergent forks, exact state after
reopening, shorter/equal/rejected candidates, wrong current/network pins, every
publication/recovery boundary, repeated recovery, finite bounds and real-process
exclusion. Fresh-peer reconstruction follows. MB-04 persistent node/pool/wallet/
operator interfaces and MB-06 human consensus remain later work.

No issuance formula, founder allocation, marketing reserve, grant, mint, nonzero
fee/reward API, actual biometric input, private matching or anonymity guarantee is
introduced. D-01/D-04 remain OPEN. **Gate A and Gate B remain UNMET.**


## 22. MB-03E1 acceptance — MIRROR-MB03E1-001

**Updated 2026-11-26.** Implemented replacement selection metadata and checked pure
preparation as the first bounded slice of MB-03E. At this historical checkpoint,
the actual replacement store, publication, cleanup and recovery remained MB-03E2. The unchanged prior comparator
and storage contracts remain in sections 18–21.

### Source and publication

Implementation source: **349edf5a6f453f365fa55480f6117c240876b0e1**.
Tested tree: **b88c9cafc8bf417d5609743bacb1ab278070bd0e**.
Parent: **3da75720319094f1b91b4a337b45cfdeb6869e7b**.
Source scope: **10 files, 2,210 insertions, no deletions**. The increment adds two
source modules, two test targets, an independent vector/reference, a synthetic
example and normative contract, with additive module exports and two CI steps.

[Source push CI 45721188614](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45721188614)
and [source PR CI 45721198833](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45721198833)
both passed. Full push-job `139579639094` logs confirmed the exact source checkout,
787 workspace tests, eight compile-fail doctests, every required release suite,
nine independent references, eleven examples and the allocation/runtime-data audit.
The follow-up documentation commit is separately checked; do not infer evidence
for an unknown future head from the implementation results. PR #1 stays open and
unmerged on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented contract

[STORAGE_REPLACEMENT_V2.md](docs/STORAGE_REPLACEMENT_V2.md) freezes:

- Exact **280-byte MSR2**, version 2/profile 3: local u64 generation, unchanged
  nested 232-byte MSS2 and distinct replacement-selection checksum domain.
- Independent full configuration/genesis/height/tip/ledger-root/generation pins.
  Decode checks framing and outer checksum, complete nested MSS2, generation bound,
  then generation equality. No unpinned decoder or input-sized allocation.
- `prepare_replacement_v2`: check both independent network identities via the
  accepted comparator; current endpoint; selector endpoint/generation; entire
  canonical current metadata; strict preference; checked generation and canonical
  candidate metadata. Full equality prevents checksum-correct manifest/byte-total
  changes from passing merely because endpoint fields match.
- Private `ReplacementPlanV2` borrows both accepted chains and exposes the exact
  whole preferred candidate. The candidate keeps its own manifest and complete
  accounts, membership, permanent records, approvals, sequences, population history
  and resource counters. Neither input changes; safe Rust enforces borrow lifetime.
- Initialization metadata may use generation zero at any accepted height. A
  multi-height extension or divergent replacement increments generation only once.
  Strictly increasing preferred height implies generation <= height <= 4095.
  Equal-height or future human-weighted selection requires a reviewed profile.
- No store, lock, filesystem mutation, serialization of a durable intent or recovery
  entry point. Preparation encodes two bounded manifests, so it is not allocation-free;
  it clones/reexecutes no chain. Checksums/generation are not authority or freshness.

The independent Python reference derives nine cases from pinned old actual-chain
and MSS2 fixtures, not new Rust results. New fixture SHA256:
`8caebea700b0aeb8b7999ead6c6850e047df6430c874b0931f87f26c90158ac0`.
Reference script SHA256:
`e2e9580d6ceb6051bcfa4a93494340dabcca53dcaaa496aee551e9138b597e08`.

### Local acceptance

Rust **1.98.0**, locked dependencies; every required gate passed.

| Gate | Observed result |
|---|---|
| Workspace, all targets | **787 passed**, 0 failed: 735 retained + 31 codec + 21 preparation tests |
| Documentation | **8 compile-fail cases passed**: 6 retained + 2 new borrow checks |
| Personhood release | **234 runtime + 3 doctests passed** |
| Core release | **121 passed** |
| State release | **49 passed** |
| Consensus and chain release | **23 consensus + 85 chain + 3 chain doctests passed** |
| Storage release | **246 runtime + 2 doctests passed** |
| Independent reference scripts | **9 passed**, including the new MSR2 vectors |
| Synthetic examples | **11 passed**, including replacement preparation |
| Preservation and audits | Scoped formatting/whitespace pass; **108 retained files byte-identical**; no dependency/lockfile/old-vector or monetary change; no tracked runtime data |

One intentionally ignored subprocess helper remains invoked by parent storage
ownership tests and is not an additional pass. Release reruns and multiple cases
inside one test are not additional unique tests. The only existing Rust file
changed is `mirror-storage/src/v2/mod.rs`, with documentation and exports only.

Codec coverage includes every independent vector, malformed framing/checksums,
checksum-correct nested semantics, full external pins, metadata maxima, generation
bounds, old profiles/domains and manifest checks. Preparation coverage includes
real divergent histories with identical accounts but different memberships,
complete candidate pointer/state/counters, untouched inputs, every current pin,
stale generations, foreign genesis, error precedence, checksum-correct metadata
forgery, ties/shorter rejection, high initialization and multi-block generation.
`GenerationOverflow` is defensive and unreachable for valid accepted chains under
the cap; no valid-chain overflow transition is claimed as exercised.

Two read-only implementation/contract reviews found no blocking discrepancy. They
are not external security audits. No power-loss experiment or new filesystem
certification is claimed.

### Synthetic observation

`replacement_plan_v2_demo` independently pins and replays fork height **3**, local
generation **0**, and main height **4**. The preferred replacement prepares local
generation **1**, sharing accepted ancestor height **1**. Its exact MSR2 bytes match
the independent vector and pinned decode.

Candidate tip: `33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`.
Candidate ledger root: `785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.
Active / pending / high-water: **1 / 0 / 3**.
Encoded MRB2 bytes / cumulative state items: **4,330 / 75**.
Both input chains remain unchanged. No filesystem selection or durable replacement
occurs in the example.

### Remaining boundaries and next action

**At the E1 checkpoint: VERIFIED_COMPONENT — METADATA / IN MEMORY. MB-03/E
remained IN PROGRESS; next was MB-03E2. Section 23 records later E2a delivery.**

Freeze the stable-root disk state machine and cleanup before writes. Choose a
bounded retained layout explicitly; E1 assigns no paths to generations. Under
continuous exclusive ownership recheck independent pins, exact selected disk
history, canonical metadata and complete accepted candidate. Synchronize candidate
files/directories before marker publication, synchronize the root before memory
advances, and define fencing/uncertainty, every recognized interruption and explicit
completion/retention. Never roll back an already selected candidate through retain-old.
Unknown or corrupt data stays preserved/rejected; ordinary opening is not repair.

Retain actual chain/replay caps and separately bound staging, enumeration and repeated
payload reads. One complete MSR2-selected payload is at most 19,559,132 bytes;
two manifests/histories/selectors total 39,118,264 bytes. This is planning arithmetic,
not an implemented E2 read/staging/RSS allowance. New persisted intent/layout bytes
require their own reviewed contract and independent reference. Prior one-child
recovery must not be silently reused as generic reorganization.

Fresh-peer reconstruction follows durable replacement. Persistent V2 node/wallet,
full Ant Kiss, human consensus, economics, private intake/matching and integrated
hardening remain unfinished. No founder/marketing allocation, grant, fee/reward API,
new biometric collection or device requirement is introduced. D-01/D-04 stay open;
**Gate A and Gate B remain UNMET**. The rough 20–25% Gate A estimate is unchanged.

## 23. MB-03E2a acceptance — MIRROR-MB03E2A-001

**Updated 2026-11-26. VERIFIED_COMPONENT — REPLACEMENT / CLEAN RECONCILIATION.**
E2 is deliberately split into durable publication/clean reconciliation (E2a) and
explicit interrupted staging/cleanup recovery (E2b). E2a is actual disk behavior;
At this historical checkpoint E2b remained unimplemented; section 24 records its
later acceptance. MB-03 and Gate A/B remain unfinished.

### Exact source and publication

Source increment: **9 files, 3,165 insertions and 10 deletions**. It adds the store,
private/public tests, one example and normative lifecycle contract, with additive
exports/process cases, one CI example step and a link from the retained E1 contract.

Implementation source: **9ad479d7c89a5b68e28e752c28b174f9885f16c5**.
Tested tree: **bc91bc43e0446be3fbffec1c3da0601cf12619c8**.
Parent: **b0582a41fa9286ebe7e8f29d8c5ab91b8ee366f4**.
[Source push CI 45726543378](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45726543378)
and [source PR CI 45726549735](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45726549735)
both passed. Full push-job `139597729216` logs confirmed the exact source checkout and
all required workspace, documentation, release, reference, example and audit gates.
Later documentation-head CI is checked separately; do not treat these source runs
as evidence for an unknown future head. PR #1 remains open and unmerged on
`work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented contract

`ReplacementStoreV2` now initializes, inspects, reopens and replaces complete
accepted V2 branches on disk. The selected generation lives in
`generation-{generation:020}` with its exact MNM2 manifest and full MRB2 history;
unchanged MSR2 selects it. A clean store contains exactly that directory and
`selection.v2`. At most the current and next generation coexist during publication,
with one optional `selection.pending`. No directory scan chooses authority.

A replacement rechecks independent network/current checkpoint/generation pins,
strict preference, the entire selected disk history and canonical metadata under
continuous exclusive ownership of the stable root inode. It stages, synchronizes
and fully validates the complete candidate, publishes the marker and synchronizes
the root, then revalidates the selected candidate and **all old bytes before any
old deletion**. Cleanup removes only exact named unselected files and empty
directories. It never replaces the locked root or recursively deletes a tree.

Final clean replay, synchronization and exact comparisons precede the single memory
update. Accounts, memberships, permanent reservations, retained approvals, sequences,
population high-water and resource counters all come from the selected branch.
No orphaned state is grafted into it. The candidate input is borrowed and remains
available if the operation fails.

Disk-phase failures fence ordinary state access and subsequent replacements.
The last-confirmed checkpoint describes prior memory only. Any error after a
publication attempt, including cleanup failure, may leave the new branch selected.
Consuming `reconcile` retains the same lock and returns a writer only after full
validation/synchronization of a **clean** independently selected layout.
Interrupted staging/cleanup is rejected and preserved by normal open/inspect/reconcile.

The encoded selected-payload ceiling is **19,559,132 bytes**; two histories and
two selectors are bounded by **39,118,264 bytes**. Every payload read and growth
sentinel is charged to one **313,011,680-byte** allowance per call. A replacement
performs five bounded full replays plus exact byte comparisons, with a conservative
successful-read bound of **176,068,508 bytes**. These are not RSS or latency bounds.
The unchanged per-chain caps are 4096 blocks, 16 MiB MRB2 and 1,048,576 cumulative
state/configuration items. Cooperative Linux locks and trusted local owners/parents
remain assumptions; WSL/DrvFS/native Windows/network filesystems are not newly verified.

The new normative [store contract](docs/STORAGE_REPLACEMENT_STORE_V2.md) fixes the
lifecycle, bounded layout and failure states. It distinguishes a visible selector
rename, successful root synchronization, and completed cleanup/final synchronization.
Existing MSR2/MSS2/MSU2, MNM2/MRB2 and every independent fixture remain unchanged.
No journal, migration, automatic repair, new consensus score or dependency is added.

### Verification

Local verification used Rust **1.98.0** and locked dependencies:

- **836 workspace/all-target runtime tests passed**, zero failures: 787 retained
  plus **49 new** (32 public integration, 14 private failure/read-budget and three
  cross-process ownership tests). Multiple injected boundaries within one function
  are scenarios, not additional tests.
- **Eight retained compile-fail doctests passed**. No new doctest count is claimed.
- Required release gates passed: personhood **234 runtime + 3 doctests**; core
  **121**; state **49**; consensus **23**; chain **85 runtime + 3 doctests**;
  storage **295 runtime + 2 doctests**. Release reruns are not new unique tests.
- All **nine independent Python references** and **twelve synthetic examples**
  passed. Existing fixtures, formats, dependencies and Cargo.lock are unchanged.
- One subprocess-only helper remains intentionally ignored during ordinary
  enumeration and explicitly invoked by parent tests; it is not an extra pass.
- Scoped formatting/whitespace and monetary/runtime-data audits passed.
  **114 of 116 retained source/dependency/reference/fixture files are byte-identical**
  to the parent. The two changed retained Rust files add module exports/docs and
  replacement-profile cases to the existing process harness. No runtime data is tracked.

Two read-only implementation/contract reviews found no blocker. These checks are
not an external security audit or physical power-loss certification.

The new `replacement_store_v2_demo` performs an actual synthetic disk switch from
fork height **3 / generation 0** to main height **4 / generation 1**, removes the
unselected old generation, then reopens and reconstructs the exact candidate.
Tip: `33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`.
Ledger root: `785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.
Active/pending/high-water: **1/0/3**. MRB2 bytes/state items: **4330/75**.
A separate integration case switches branches with identical account bytes but
different membership history, checking the whole chain and all counters after reopen.

The 14 private test functions include traced failure injection at every encountered
initialization, replacement and clean-reconciliation boundary; full old memory and
fenced APIs; root lock continuity; partial candidate data; interrupted old cleanup;
postpublication candidate/old corruption before any deletion; and cumulative read
allowance/sentinel overflow guards. The 32 public tests include independent pins,
whole fork reconstruction, equal-account/different-membership state, strict layouts,
unknown staging, file types/aliases/lengths and cross-profile rejection. Three new
process cases exercise writer/shared ownership, real replacement/reconciliation,
drop and termination without replacing the root inode. Test scenarios are not
physical durability proofs or evidence of hostile-owner race resistance.

### Exact next action

**At the E2a checkpoint, next was MB-03E2b.** Section 24 records its delivery.
The following retained requirements were its original next-action brief:

Use [the store contract](docs/STORAGE_REPLACEMENT_STORE_V2.md), especially section 6.
Retain independent identity, old/new full checkpoints and generations, **both exact
accepted old and candidate histories**, and an explicit completion/retention action.
Old checkpoint plus candidate alone cannot authenticate partly deleted old files.
Reconstruct any missing reference history independently against its retained pins;
never infer it from corrupt or missing residue.

Before mutation, fully validate the request, strict preference/generation transition,
the actually selected complete history and every allowed residual name/byte. Support
only the documented bounded shapes. Before directory synchronization, surviving
staged files or old cleanup files may be arbitrary subsets rather than ordered
height prefixes. Each candidate residual must match its authorized canonical byte
prefix; every surviving old file must match its full exact bytes. A pending selector
requires a complete candidate. A selected candidate must be complete and cannot be
rolled back by retaining old.

Keep the same exclusive root ownership through consuming recovery, final replay,
synchronization and returned writer. Errors return no writer. Define idempotent
completion and retention, retry every failure boundary, preserve unknown/corrupt
artifacts and missing/corrupt selected markers, and retain finite input/read/work
budgets. Existing one-child recovery is not generic branch recovery. Interrupted
initialization stays outside this recovery scope unless separately reviewed.

Then proceed to fresh-peer reconstruction; persistent node/pool/wallet/operator APIs
remain MB-04, complete Ant Kiss MB-05 and human consensus MB-06. No monetary-policy
choice, device purchase or real biometric collection is needed for E2b.

Accepted founder/marketing, bootstrap, population allowance, device/accessibility
and private-record policies are unchanged. D-01/D-04 remain open; no issuance or
fee/reward API is introduced. The rough 20–25% Gate A estimate is unchanged.

## 24. MB-03E2b acceptance — MIRROR-MB03E2B-001

**Updated 2026-11-26. VERIFIED_COMPONENT — EXPLICIT INTERRUPTED REPLACEMENT RECOVERY.**
MB-03E2/E local replacement/recovery components are complete within their stated
scope. MB-03 and Gate A/B remain unfinished.

### Exact source and publication

Implementation source: **99bf0da86469f5832fe9e5c855e9729676618887**.
Tested tree: **d20a433805287d16765e2c9d01957056e8cfef2a**.
Parent: **5706ed4b4d5035aba8dc5be274bb7a35f048bee7**.
[Source push CI 45731894201](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45731894201)
and [source PR CI 45731901717](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45731901717)
both passed. Full push-job `139615676064` logs confirmed the exact
source checkout and all required workspace, documentation, release, reference,
example and audit gates. Later documentation-head CI is checked separately and
recorded on PR #1; these source results do not assert a future head is tested.
PR #1 remains open and unmerged on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Source increment: **10 files, 3,587 insertions, 24 deletions**. Recovery, private/public
tests, independent-process cases, one example, its CI step and normative contract;
prior store logic and all previous encodings remain unchanged.

### Implemented behavior

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

### Bounds and limitations

No encoding, hash domain, dependency, monetary rule or prior store behavior changes.
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

### Actual verification

Local verification used Rust **1.98.0** and locked dependencies:

- **891 workspace/all-target runtime tests passed**, zero failures: 836 retained
  plus **55 new** (38 public recovery integration, 15 private failure/retry/read-
  budget, and two independent-process ownership tests). Boundary scenarios within
  one function are not additional tests.
- **Eight retained compile-fail doctests passed**; no new documentation-test count.
- Required release gates passed: personhood **234 runtime + 3 doctests**, core
  **121**, state **49**, consensus **23**, chain **85 runtime + 3 doctests**,
  storage **350 runtime + 2 doctests**. Reruns are not new unique tests.
- All **nine independent Python reference checks** and **thirteen synthetic
  examples** passed. One subprocess helper remains intentionally ignored during
  ordinary enumeration and is invoked by the parent process tests.
- Scoped formatting, whitespace and monetary/runtime-data audits passed.
  **117 of 120 retained source/dependency/reference/fixture files are byte-identical**
  to the parent. Changed retained Rust files add module exports/docs and recovery
  cases to the existing process harness. Cargo.lock, dependency manifests, previous
  encoding fixtures, monetary code and legacy runtime source are unchanged.

Root and independent implementation review identified and corrected the absent-old-
blocks synchronization edge before final verification. Read-budget charging was
also moved before opening existing-prefix payloads. Final independent review found
no remaining blocker in the scoped component. This is not an external security
audit, physical power-cut test or proof of hostile-owner race resistance.

The thirteenth example, `replacement_recovery_v2_demo`, constructs synthetic
sparse staging, completes the preferred branch, reopens and checks full state,
repeats completion, rejects rollback and separately retains old. It uses controlled
filesystem residue, not an actual physical power cut. Recovered candidate is height
4 / generation 1; tip
`33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`, ledger root
`785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.
Active/pending/high-water is **1/0/3**, encoded block bytes/state items **4330/75**.
Tests also recover equal-account branches with different membership histories,
compare complete ChainV2 values and preserve all branch-specific counters.

### Exact next action

**At the E2b checkpoint, next was MB-03F — bounded fresh-peer V2 reconstruction.**
Section 25 records its first framing/receiver slice. The current next task is F2.

Read the existing V1 protocol/network/node interfaces and accepted V2 chain/storage
contracts before designing the new wire boundary. Add an explicitly versioned V2
path; do not reinterpret legacy bytes, initialize an existing store implicitly,
or treat a peer's advertised checkpoint as accepted state.

Start with a concrete bounded protocol/receiver slice: independently selected
network configuration/genesis, strict message versions/kinds/lengths, a bounded
manifest and exact complete block bytes, bounded request counts and byte totals,
and an explicit terminal candidate checkpoint. Treat peer-provided height, tip and
root as claims to verify by complete actual ChainV2 replay. The caller's independent
network selection remains authority; local storage generations are never peer
fork scores or freshness evidence. Select only fully accepted branches through
the existing comparator before durable publication under existing root ownership.

Preserve all history/state/work caps and membership-aware replay. Reject wrong
networks, malformed/truncated/oversized frames, missing/duplicate/out-of-order
blocks, changed manifests/tips, invalid signatures/work/roots and interrupted
transfers without accepting a partial candidate or mutating the selected store.
Keep existing valid stored data and both old/candidate histories available for
publication recovery. No cache/root, peer counter or filtered membership log can
stand in for the complete original mixed-operation blocks.

Demonstrate independent fresh-peer reconstruction across genuine membership forks,
including equal account bytes with different membership, reservations and high-water;
verify complete rebuilt state and persist/reopen it under independent local pins.
Separate protocol-codec, receiver/replay and actual peer-process evidence; do not
mark the whole network complete from a standalone decoder or in-memory fixture.
Persistent daemon, transaction pool, wallet and operator APIs remain MB-04;
complete Ant Kiss is MB-05, final human consensus MB-06. Production fees/rewards,
private matching and actual devices retain their own later decision/acceptance gates.
No monetary choice, hardware purchase or real biometric collection blocks MB-03F.

## 25. MB-03F1 acceptance — MIRROR-MB03F1-001

**Updated 2026-11-26. VERIFIED_COMPONENT — BOUNDED FRAMING / IN-MEMORY RECEIVER.**
The first MB-03F slice is complete within its framing and complete candidate
reconstruction boundary. **MB-03F and MB-03 remain IN PROGRESS.** Actual TCP-session
integration and durable adoption of a received candidate remain MB-03F2; persistent
daemon, wallet and operator APIs remain MB-04. Both device-readiness gates are unmet.

### Source and publication evidence

The implementation source is published; its exact source/tree/parent and verified
CI evidence are recorded in [CURRENT_STATUS.md](CURRENT_STATUS.md). Source push and
PR CI both passed, and full push-job logs confirmed the exact checkout and every
configured test/reference/example/audit gate. A later documentation-only head is
checked separately and recorded on PR #1; source results do not assert future-head
verification. PR #1 remains open and unmerged; recheck live refs before continuing
and preserve newer/unpushed work.

The [peer-history contract](docs/PEER_HISTORY_V2.md) defines the new boundary;
[test-matrix section 17](docs/MEMBERSHIP_TEST_MATRIX.md#17-mb-03f1-bounded-peer-history-framing-and-candidate-reconstruction)
maps the codec and receiver cases, and the [threat review](docs/THREAT_MODEL.md)
records the remaining transport/adoption risks. Engineering source review found no
blocker within this stated component; it is not an external security audit.

### Implemented framing and authority

The new `mirror_protocol::v2` uses separate **MPN2/version-2** framing. Its exact
12-byte header selects Begin, Manifest, Block or End and declares a bounded payload.
Begin and End each contain the same 152-byte `HistorySummaryV2`; Manifest contains
complete canonical MNM2 bytes, and Block contains its u64 height plus complete
original MRB2 bytes. All integers use little-endian encoding and stored hash bytes
are unchanged. There is no new signature/root hash domain, fallback to MIRR/V1,
storage generation, snapshot authority or human-consensus certificate on this wire.

Every summary field begins as an untrusted finite-transfer claim. The caller
independently supplies C/genesis to `HistoryReceiverV2::new`; the peer cannot choose
those pins through Begin, its manifest or a claimed endpoint. Begin must match that
identity, and `NetworkManifestV2::decode_for` checks the actual configuration,
independently selected genesis and all configured bootstrap proofs.

The receiver decodes each complete original block, checks its expected wire/header
height, and executes actual `ChainV2::from_genesis`/`append`. Accepted ancestry,
target/work, timestamps, original mixed-operation positions, signatures, account/
membership rules and the combined ledger root all come from that one candidate.
Peer assertions never become a trusted `ChainReplayExpectationV2`. No account
snapshot, membership-only event projection or cache can substitute for original
mixed blocks and the retained execution/resource accounting.

### Consuming admission and complete candidate boundary

`begin_frame(self, header)` validates exact header shape, kind/phase, per-kind and
configured lengths, feasible future byte totals, and aggregate frame/wire caps.
Only then is a `PendingHistoryFrameV2` returned with its checked `payload_len`.
The future transport must use that value before allocating/reading the body.
`accept_payload(self, bytes)` requires exactly that length and consumes the pending
owner; every error discards continuation and exposes no partial chain. Neither
receiver nor pending owner is Clone. The full-frame convenience API cannot establish
how its caller allocated an already supplied frame.

After the final actual block, the computed endpoint and count/byte totals must match
the captured Begin. The chain remains private until the identical End summary and
exact final frame/wire accounting pass. Consuming `finish` then returns one complete
`ReceivedCandidateV2`. Missing End is failure even if every block arrived, and further
input through a completed receiver rejects. F1 uses no socket EOF; a later transport
must define its session/connection lifetime and discard unfinished attempts.

The accepted candidate retains full accounts, membership history, permanent
reservations, approvals, independent sequences, population counters and resource
charges. It is not necessarily newer, preferred or globally canonical. Existing
independently bound branch comparison and publication/recovery remain separate
acceptance boundaries. `HistoryFramesV2` borrows one immutable accepted chain,
preserves original manifest/block bytes, emits one bounded frame at a time and is
fused after an error or completion.

### Finite budgets and preserved behavior

For N blocks, M manifest bytes and B complete MRB2 bytes, the transfer is exactly
`N + 3` frames and `340 + M + B + 20*N` bytes. Its inclusive absolute bounds are:

| Resource | Maximum |
|---|---:|
| Blocks, including genesis | 4,096 |
| Complete MNM2 manifest bytes | 2,781,636 |
| Complete MRB2 history bytes | 16,777,216 |
| Frames, including Begin/Manifest/End | 4,099 |
| Total framed-input bytes | 19,641,112 |
| Existing cumulative state/configuration work items | 1,048,576 |

Configured per-block/account/member/operation limits remain in force, with the
existing hard block ceiling of 1,048,576 bytes. The exact 168-byte empty genesis
tightens the manifest-aware total to `168 + (N - 1)*configured_block_cap`; its
height-prefixed payload is exactly 176 bytes. Every subsequent admitted payload
must leave a feasible minimum/maximum remainder, and the last block must exhaust
the claimed total. Intermediate products and sums use checked u64 arithmetic,
including products that exceed 32-bit usize. These are encoded-input and abstract
work limits, not RSS, peer-count, bandwidth, latency or availability guarantees.

Preservation checks found **120 of 124 retained Rust/dependency/reference/fixture
files byte-identical** to the preceding checkpoint. The four changed retained files
are Cargo.lock, the network manifest, and the network/protocol library roots. V1
library changes only add the new module exports/docs. The lockfile adds four local
path-dependency edges and changes no external dependency version. Prior encodings,
frozen vectors, old genesis/development funds, runtime data, monetary policy and all
three storage profiles remain preserved. No sensor data, real credentials or new
issuance/reward mechanism is added.

### Actual local verification

Rust **1.98.0**, locked dependencies:

| Gate | Verified result |
|---|---|
| Workspace, all targets | **951 runtime tests passed**, zero failures: 891 retained plus 60 new |
| Documentation | **9 compile-fail doctests passed**, including the new consuming-receiver ownership case |
| Personhood release | **237 passing outcomes:** 234 runtime and 3 doctests |
| Core release | **121 runtime tests passed** |
| State release | **49 runtime tests passed** |
| Consensus + chain release | **111 passing outcomes:** 108 runtime and 3 doctests |
| Storage release | **352 passing outcomes:** 350 runtime and 2 doctests |
| Protocol + network release | **79 passing outcomes:** 78 runtime and 1 doctest |
| Independent Python references | **All 10 passed**, including `reference_peer_history_v2.py --check` |
| Synthetic examples | **All 14 passed**, including `history_receiver_v2_demo` |

The new 60 runtime functions are **24 protocol codec/vector cases and 36 receiver
integration cases**. Scenario loops, repeated fixtures, release reruns and example
assertions do not count as additional unique tests. One subprocess-only storage
helper remains intentionally ignored during ordinary enumeration and is invoked
by its parent process tests. Required verification now includes the separate
protocol/network release gate, tenth independent reference and fourteenth example;
all previous gates remain required.

The new in-memory example reconstructs independently frozen complete main and
membership-fork histories and compares exact ChainV2 values. Equal account bytes
at height 3 retain different membership histories; the existing fixed-work comparator
keeps current on that tie. The height-4 main candidate reports **8 frames / 5,406
wire bytes**, **4,330 MRB2 bytes / 75 state-work items**, with active/pending/high-water
**1/0/3**. The example opens no socket and changes no selected store. Receiver tests
separately reject malformed/incomplete/reordered transfers, false endpoints, foreign
network/manifests, impossible lengths, wrong work/roots, re-mined invalid signatures
and valid signatures anchored in a different branch. These are in-memory results,
not evidence of socket deadlines, actual peer-process interruption or disk adoption.

### Exact next action

**At the F1 checkpoint, next was MB-03F2 — TCP sessions and durable adoption.**

Build actual bounded peer sessions around the F1 split header/payload API. Check
each fixed header before allocating or reading its declared body; enforce request,
connection, total-byte and session-time limits; handle fragmentation, premature
closure and malformed traffic without partial acceptance or receiver resumption.
Explicit End remains the finite-history boundary; define connection/session behavior
without replacing that boundary with EOF.

Use independently selected network identity for each complete reconstructed
candidate. Compare only accepted whole branches against the selected local state,
then durably publish through existing ownership, fencing, clean-opening and explicit
recovery rules. Retain the old/candidate histories and their independent full
checkpoints/generations for interrupted adoption. Rejected or unfinished transfer
must preserve the current selected store; no peer claim may authorize repair or
provide global freshness.

Demonstrate separate real peers, genuine membership forks, complete state/counter
agreement and durable reopen, with transport failures and publication/recovery
failures kept distinct. Do not mark T31/T32 or all MB-03F complete from codec/receiver
tests. Node/operator flows, Ant Kiss, human consensus, unresolved monetary decisions,
protected biometric matching and actual devices retain their later gates. No new
money-policy choice, hardware purchase or real biometric collection blocks F2.

## 26. MB-03F2 acceptance — MIRROR-MB03F2-001

**MB-03F2, MB-03F and MB-03 are VERIFIED_COMPONENT within finite V2 storage,
explicit recovery and peer-synchronization scope.** This closes the planned MB-03
component sequence, not the persistent V2 node. F2 retains recovery authority only
in memory; persisted operator intent and restart orchestration remain MB-04A1.
Session limits are per owner, not a persistent node's global service policy.
Daemon/reconnect/multi-peer/pool/wallet operations remain MB-04; complete Ant Kiss
MB-05 and final human consensus MB-06. Money, protected intake/matching and actual
devices retain their later decision/acceptance gates. Gate A and Gate B remain UNMET.

### Exact source and publication

Implementation source: **851193ea6428198441e6cdf29361b1885fae157b**.
Tested source tree: **cf2e4bb9d29b250a000bb4680d1571e0355d1126**.
Parent checkpoint: **f47b4306b13c4d079859472f2fdd919debd83eda**.
[Source PR CI 45750810878](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45750810878)
passed all configured gates. Full job `139677872753` logs were inspected. Its
synthetic PR merge checkout `e74753737b608408a0f424d5b9c290805fa08752` has no file
differences from the implementation source; this is verified source-tree evidence,
not an assumption based on an older green run. All sixteen published source/contract/
workflow files were read back byte-for-byte against the locally tested contents.
Later documentation-head CI is checked separately and recorded on PR #1; these
source results do not assert a future head is tested.
PR #1 remains open and unmerged on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented session and adoption boundaries

The new one-shot V2 TCP path requests and reconstructs one complete accepted
history under independently selected configuration/genesis. MPQ2 is an exact
72-byte request: magic, version 2, kind 1, C and genesis. The requester half-closes;
the provider validates the request and its EOF before responding. Existing F1
Begin/Manifest/Block/End bytes and every earlier wire/hash domain remain unchanged.

The receiver calls consuming F1 header admission before payload allocation, then
checks the whole admitted payload against the session owner's remaining budget.
Every original block executes against its own accepted branch ancestry. A matching
End, terminal EOF, consuming finish and final deadline check are all required before
a candidate becomes public. Incomplete, malformed, oversized or trailing input
closes the attempt and returns no partial candidate. Reception writes no store.

One non-Clone synchronous owner permits at most eight sequential attempts, one
live stream at a time, and 157,129,488 cumulative actual read/write bytes. Failed
attempts and partial I/O remain charged. Each attempt has one positive absolute
deadline, at most 60 seconds, including connect/accept, every individual read/write,
request/response EOF and checks after semantic work. Slow fragments cannot reset
that deadline. EOF probing requires one unused byte of allowance; EOF itself
charges zero bytes. Limits are per owner, not global peer admission or bandwidth.
CPU validation is bounded but not preempted; late results are rejected afterward.

Explicit adoption consumes a completed candidate and an independently pinned owned
replacement store. It checks both network identities, the full current checkpoint/
generation and canonical metadata, then compares actual accepted whole histories.
Identical, shorter and tied candidates keep current without filesystem calls.
A strictly preferred candidate uses the existing owned replacement publication.
Before any publication, the attempt retains an exact old-chain clone, the moved
candidate, independent identity and both local checkpoints. Complete balances,
memberships, permanent records, approvals, sequences, population history and resource
counters always follow one history; no losing-branch state is grafted into it.

Publication failure retains the same store lock and both exact histories but exposes
no ordinary writer. Recovery is explicit: Complete or RetainOld invokes the existing
consuming recovery under that same lock. Recovery failure returns no handle while
retaining the exact inputs/root for a separately requested retry. A selected candidate
cannot roll back. Fresh initialization remains a separate explicit operation;
failed existing-store opening never falls back to initialization or repair.

These recovery inputs are in memory, not a crash-persistent intent journal.
Adoption runs outside the socket deadline. Extra accepted histories and replay
allocations are not covered by encoded-input ceilings. Storage's cooperative Linux
locking, trusted local owner/parents, fencing, full replay and unsupported-layout
rules remain unchanged. Persistent node control/restart authority is the next task.

### Actual verification

Local verification used Rust **1.98.0** and locked dependencies:

- **1,010 workspace/all-target runtime tests passed**, zero failures: 951 retained
  plus **59 new** (24 session integration, one private blocked-write deadline,
  23 adoption, nine real-process integration and two independent-vector tests).
  Parameterized scenarios and subprocess invocations are not extra unique tests.
- **Ten compile-fail doctests passed**, nine retained plus one proving that a
  prepared adoption cannot be published twice.
- Required release gates passed: personhood **234 runtime + 3 doctests**, core
  **121**, state **49**, consensus **23**, chain **85 runtime + 3 doctests**,
  storage **350 runtime + 2 doctests**, protocol **38**, and network
  **99 runtime + 2 doctests**. Release reruns are not new unique tests.
- All **eleven independent Python reference checks** and **fifteen synthetic
  examples** passed. Two subprocess helpers are intentionally ignored by ordinary
  enumeration and exercised by their parent tests.
- Scoped formatting, whitespace and allocation/runtime-data audits passed.
  **129 of 132 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent. The existing F1 runtime body is unchanged; its module only gains
  exports/documentation. The network manifest and lockfile add one local storage
  dependency edge, with no external package/version changes. Monetary code, V1,
  previous encodings and frozen fixtures remain byte-identical.

The new independent Python reference packs MPQ2 itself and derives four complete
request/response transcripts from the established independent Python history
generator. It does not invoke Rust or accept Rust output as its oracle. Transcript
hashes are test assertions, not new wire fields or cryptographic domains.

The finite `peer_session_v2_demo` launches three separate provider processes over
TCP and verifies explicit initialization, a kept tie, preferred whole-branch
publication and strict reopen with independently retained local pins:

| Selected result | Height / generation | Accounts | Active / pending / historical peak | MRB2 bytes / state items |
|---|---|---:|---|---|
| Fresh main history, strict reopen | 4 / 0 | 4 | 1 / 0 / 3 | 4,330 / 75 |
| Equal-height membership fork kept current | 3 / 0 | 4 | 3 / 0 / 3 | 3,206 / 58 |
| Preferred membership fork, strict reopen | 4 / 1 | 4 | 1 / 1 / 2 | 3,605 / 73 |

Each result retains two recorded approvals. Fresh main reception uses eight F1
frames / 5,406 response bytes, or 5,478 actual application bytes per endpoint with
the request. The genuine membership forks have identical account bytes; equality
checks cover the full selected state, not just balances or a reported root.

Successful fragmented process transfer reconstructs the complete history.
Failed separate-process transfers include early close/stall, an oversized fixed
header without a body, changed End and an independently frozen
actual wrong-ledger-root block. Failure preserves every selected-store byte.
Drip reads, blocked writes, missing EOF, repeated requests and cumulative quotas
are covered by the transport tests. This is tested component behavior and an
implementation review, not an external security audit or physical power-loss proof.

### Exact next action

**At the F2 checkpoint, next was MB-04A1 — durable V2 control/restart authority.**

1. Read the accepted F2 session/adoption contracts and profile-3 store/recovery
   contracts. Specify a separate bounded operator-owned control directory outside
   the strict store root, with deterministic ownership/lock order. Preserve V1 paths.
2. Retain independently selected C/genesis and the complete confirmed local
   checkpoint/generation. Loading a record must not derive authority from a peer,
   selected-file checksum alone, or the highest observed generation.
3. Specify at most one pending replacement intent containing BOTH complete accepted
   old/candidate histories, exact local endpoint/generation pins and network identity.
   Fully validate bounded input and actual replay when loading it; no authoritative
   cache, unknown-artifact cleanup or path-derived authorization.
4. Persist, validate and synchronize that exact intent BEFORE any store publication.
   After confirmed publication/recovery, persist/synchronize the confirmed checkpoint
   before retiring the intent. Test the window in which disk selected candidate but
   the separate confirmed checkpoint still names old.
5. Startup/status must report an unresolved intent as recovery-required. Completion
   or retention is explicit; RetainOld cannot roll back a selected candidate. Unknown,
   corrupt, overlong, unsafe or unsupported partial control artifacts reject and stay
   preserved. Do not silently expand storage recovery or its accepted layouts.
6. Exercise independent process restart, read-only status, wrong/stale local pins,
   failed intent writes before store mutation, interruption at defined publication/
   checkpoint/retirement boundaries, exact same membership-fork state, and exclusion
   of a competing operator. Termination tests are not physical power-loss evidence.

Keep this first increment focused on the durable control-state component and a
finite restart harness. **MB-04A2** then exposes separate V2 init/status/sync-once/
explicit-recover operator commands. A persistent server, reconnect/rejoin scheduler,
global connection admission, multi-peer policy, pool and wallet remain later MB-04.
No monetary, device, biometric or final human-consensus choice blocks A1.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies: workspace tests,
doctests, all required release suites, eleven references, fifteen examples, scoped
formatting/whitespace and preservation/allocation/runtime-data audits. Keep exact
source CI separate from later documentation-head CI and refresh the handoff.

## 27. MB-04A1 acceptance — MIRROR-MB04A1-001

**MB-04A1 is VERIFIED_COMPONENT — DURABLE CONTROL / EXPLICIT RESTART.**
MB-03 remains complete within its finite storage, recovery and peer-flow scope.
MB-04 remains IN_PROGRESS: operator commands, persistent serving, reconnect and
multi-peer scheduling, pool and wallet are separate work. Complete Ant Kiss remains
MB-05 and final human consensus MB-06. Gate A and Gate B remain UNMET.

Implementation source: **32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0**.
Tested source tree: **d79080968d822a2267ce5f1b1196efcf16d2e546**.
Parent checkpoint: **37010bc5c5827f5a8dd886af0e79d079ebfe1d4e**.
[Source CI 45861912935](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45861912935)
passed every configured gate. Full job `140054826270` logs were inspected and confirm
1,083 runtime tests and eleven compile-fail doctests. Its actual checkout
`62437dc1c338156fb984349b791efc7a4ad34ebb` is verified against the implementation's exact source tree.
All sixteen published implementation/contract/workflow files were read back
byte-for-byte against the locally tested source. Later documentation-head CI is
checked separately and recorded on PR #1; these results do not assert a future
head is tested. PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented control and restart boundary

A separate `mirror_node::v2` library now preserves trusted local restart authority
outside the strict profile-3 store. The operator supplies separate non-nested
control/store paths and independently chosen C/genesis. Both roots use cooperative
Linux directory-inode locks, always control first and store second. The ready
controller exposes no raw store writer; failures consume it and return no writer.

`NodeControlV2::initialize` explicitly attaches control to an already initialized
store under independent full checkpoint/generation pins. It never initializes or
repairs a store after failed opening. `open_writer` accepts only clean control and
strictly replays the selected store. `status` performs no application writes or
synchronization: absent/empty control is Missing, verified clean storage is Clean,
and fully replayed pending intent is RecoveryRequired. Invalid input rejects.
RecoveryRequired reports local intent and last-confirmed pins; it does not inspect
or assert the store's current selected endpoint.

MNC2 is an exact 464-byte confirmed record with full local checkpoint and unchanged
MSR2 canonical metadata. MNI2 has a 952-byte header, both complete unchanged MPN2
history streams and a checksum; its maximum is 39,283,208 bytes. Loading binds the
external network identity, executes both complete histories, checks both canonical
records and reuses existing strict replacement/generation rules. Checksums detect
corruption; they cannot defeat a hostile operator who replaces all trusted state.

Consuming adoption accepts a completed F1/F2 candidate. Ties, shorter and identical
histories keep the current controller without filesystem calls. Strict preference
retains an old-chain clone and moves the complete candidate into intent. The exact
intent is written, synchronized, read back, fully replayed, renamed and its control
directory synchronized BEFORE invoking existing storage replacement. The ready
writer rechecks confirmed bytes, intent and exact layout immediately before that
mutation. Accounts, memberships, approvals, sequences and population history always
come from one complete accepted branch.

After storage returns clean success, the exact new confirmed record is staged,
synchronized, renamed and the directory synchronized. Only then is the exact intent
removed and that absence synchronized. A retry reestablishes intent durability
before storage recovery and confirmed durability before retiring intent; visible
renames alone are insufficient. Clean writer reopening also resynchronizes the
confirmed record/control directory/parent before another adoption.

Startup never completes or retains pending intent automatically. Explicit recovery
selects Complete or RetainOld through existing storage rules. Once candidate
confirmation has begun, only Complete with a strictly clean candidate store is
permitted. Old-selected storage contradicting candidate confirmation rejects before
mutation. A selected candidate cannot roll back.

`intent.pending`, incomplete control initialization, unknown entries, corrupt data,
overlong files and unsafe links are preserved and rejected. The only recoverable
confirmation staging is an exact 0..464-byte candidate-record prefix beside valid
old confirmation and complete intent; explicit Complete extends it without truncation.
Candidate confirmation plus remaining intent is also explicitly completable. A
failure after intent retirement may leave an already confirmed clean state; errors
do not always leave intent and do not promise arbitrary corruption repair.

Control reads check type, length, inode and hardlink count, allocate only after
checked length-plus-one admission and charge one 314,269,392-byte per-call allowance.
Storage keeps its separate bounds. Encoded sizes are not RSS/time guarantees;
history copies, checksum buffers and replay consume additional memory. Parent paths
and local operators remain trusted. Process termination tests are not physical
power-loss certification or hostile-same-user race protection.

### Executed acceptance

Local acceptance used Rust **1.98.0** and locked dependencies:

- **1,083 workspace/all-target runtime tests passed**, zero failures: 1,010 retained
  plus **73 new** (24 record/vector, 31 public controller, 12 real-process parent
  tests and six internal failure/mutation tests). Parameterized boundary scenarios
  and child invocations are not extra unique tests.
- **Eleven compile-fail doctests passed**, ten retained plus a consuming-controller
  test preventing a second adoption through the same owner.
- All required release suites passed: personhood 234 runtime + 3 doctests, core 121,
  state 49, consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network 99 + 2,
  and node **73 runtime + 1 doctest**. Release reruns are not new unique tests.
- All **twelve independent Python references** and **sixteen examples** passed.
  Three subprocess helpers are intentionally ignored by ordinary enumeration and
  exercised by their parent tests.
- Scoped formatting, whitespace and allocation/runtime-data audits passed.
  **140 of 142 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent. The only retained changes are node's Cargo manifest and lockfile,
  adding one local test dependency on personhood. No external versions change.
  V1, storage/network runtime source, monetary code and earlier fixtures are unchanged.

The independent Python reference derives six confirmed records and four complete
intents from established independent history/selection generators, not Rust output.
It checks both new checksum domains, exact layouts and bounds.

Actual child processes are killed at all sixteen adoption and six initialization
boundaries, then fresh processes perform status/open/explicit recovery. Additional
tests interrupt recovery again, including a twice-extended confirmation prefix.
The error-return suite covers the same adoption boundaries, fifteen recovery-error
scenarios and changed control authority immediately before store writes or intent
retirement. Every child has bounded readiness/exit and kill/reap ownership.

The finite `node_control_v2_demo` uses four independent processes:

| Phase | Height / generation | Accounts | Active / pending / peak | MRB2 bytes / state items |
|---|---|---:|---|---|
| Initialize trusted control | 3 / 0 | 4 | 3 / 0 / 3 | 3,206 / 58 |
| Restart and keep membership-fork tie | 3 / 0 | 4 | 3 / 0 / 3 | 3,206 / 58 |
| Restart and adopt preferred membership fork | 4 / 1 | 4 | 1 / 1 / 2 | 3,605 / 73 |
| Fresh process strict reopen | 4 / 1 | 4 | 1 / 1 / 2 | 3,605 / 73 |

Every phase retains two recorded approvals. The two forks have identical account
bytes and different membership histories; full-state equality is checked. This is
finite component acceptance and implementation review, not an external audit.

### Next increment identified at the A1 checkpoint

**At the A1 checkpoint, next was MB-04A2 — separate finite V2 operator commands.**

1. Read the accepted node-control, F2 session/adoption and profile-3 recovery
   contracts. Specify and implement an explicit V2 command surface alongside the
   preserved V1 executable behavior: init, status, sync-once and explicit recover.
2. Require deliberate control/store paths and independently selected network
   identity. Define strict bounded argument/config parsing and clear diagnostics;
   do not derive trusted identity from a peer or promote the highest generation.
3. Init must separately initialize accepted synthetic storage and its A1 control
   state, or explicitly attach control using independent full pins. Report partial
   initialization; never turn a failed open into init, repair or overwrite.
4. Status must be read-only and distinguish missing, invalid, clean and
   recovery-required outcomes. It must not mine, contact a peer, create directories,
   synchronize files or repair. Keep pending intent distinct from current-disk truth.
5. Sync-once must use the existing bounded F2 TCP receiver and consuming A1 adoption.
   Recovery takes explicit Complete or RetainOld and preserves every A1 no-rollback,
   durability and unknown-artifact boundary. No automatic retry/recovery loop.
6. Test actual command processes, argument rejection and exit outcomes, read-only
   status, independent identities, successful finite synchronization and restart,
   unresolved intent and explicit recovery. Preserve synthetic fixtures and avoid
   real keys, biometrics or live monetary issuance.

Keep A2 focused on finite operator usability. Persistent serving, reconnect/rejoin
scheduling, global admission, multi-peer policy, transaction pool, wallet and
signer remain later MB-04 work. No monetary, device or human-consensus decision
blocks this increment.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies, including the
node release suite, twelve references and sixteen examples. Keep source acceptance
separate from later documentation-head CI and refresh all checkpoint records.


## 28. MB-04A2 acceptance — MIRROR-MB04A2-001

**MB-04A2 is VERIFIED_COMPONENT — FINITE V2 OPERATOR COMMANDS.**
MB-04A1 remains verified for durable local control and explicit restart recovery;
MB-03 remains complete within its finite storage/recovery/peer scope. MB-04 remains
IN_PROGRESS: persistent serving, reconnect/rejoin and multi-peer scheduling,
transaction pool, wallet and wider operator APIs are separate work. Complete Ant
Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain UNMET.

### Exact source and publication

Implementation source: **eea42a403fef2a1442e553df2a6823b1b25da616**.
Tested source tree: **cda30cd7e8b564a82d09a46b7067012e7112e2dd**.
Parent checkpoint: **957599df1b8a424ba333d0e0c10247b2807c3a09**.
[Source PR CI 45882757484](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45882757484)
passed all **43 steps**. Full job `140125894368` logs were inspected and confirm
**1,137 runtime tests and eleven compile-fail doctests**, with zero failures.
Its actual synthetic PR checkout `7dbc9921785f52c4c0136c27dc5890f9b0188e2c` has no file differences
from the implementation commit. All **9 published implementation/contract
files** were read back byte-for-byte against locally tested content.
Later documentation-head CI is checked separately and recorded on PR #1; source
acceptance does not assert that a future head is tested. PR #1 remains OPEN and
UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented operator contract

A separate `mirror-node-v2` executable composes the accepted A1 controller
and F2 peer receiver. The existing V1 executable source is unchanged; the package
sets `default-run = "mirror-node"` to preserve its prior default invocation.

| Command | Delivered behavior |
|---|---|
| `init --attach-existing` | Initialize only the separate control state for an **already initialized** profile-3 store, under independent C/genesis and all height/tip/ledger-root/generation pins. No store creation, mining, overwrite or failed-open fallback. |
| `status` | Call only A1 read-only status. Missing control exits 3, clean verified selection exits 0, validated pending intent exits 4; invalid/unsafe/busy state returns operational failure. |
| `sync-once` | Open and retain a clean ready controller before connecting, request exactly one candidate with one bounded F2 owner, then consume it through A1 adoption. No retry or automatic recovery. |
| `recover --action complete\|retain-old` | Forward the explicit action to A1's existing fully checked recovery. Selected/confirmed candidates cannot roll back. |

Every operational command requires explicit `--control-dir`, `--store-dir`,
`--config-hash` and `--genesis-hash`. The complete argument list is validated before
filesystem/socket operations: at most 32 arguments after argv[0], 4,096 UTF-8 bytes
per token and 16,384 total bytes. Reject duplicate/unknown/cross-command options,
non-UTF-8/control input, malformed exact-64 hexadecimal hashes, noncanonical
unsigned decimal, overflow and invalid init height/generation. Init height is
0..4095 and generation cannot exceed height. No env/stdin/file-config fallback.

Sync requires a numeric peer address with nonzero port, an explicit 1..60000 ms
timeout and positive I/O allowance up to 157,129,488 bytes. Its one F2 attempt
retains all prior header admission, replay, matching End and terminal EOF checks.
The timeout covers the session, not local open/replay, adoption or stdout; encoded
I/O bounds are not RSS, CPU preemption or global admission guarantees.

Output uses at most 4,096 bytes of deterministic `key=value` records. It reports
network and full checkpoints, successful sync choice/attempts/actual wire bytes,
or static non-echoing diagnostic categories. Pending fields use confirmed/old/
candidate labels with `store_selection=uninspected`; they do not assert current
store selection or infer which recovery actions are permitted. Usage exits 2
before effects; operational/output failures exit 1. Help exits 0 without paths.
Only status uses missing/pending exits 3/4.

Output failure after a mutating command can follow durable completion. The process
reports `operation_may_have_completed=true` and directs inspection instead of
retrying. Owners are released before stdout writing. Partial initialization and
unsupported/corrupt/unknown artifacts remain preserved under A1's rules.

See [NODE_OPERATOR_V2.md](docs/NODE_OPERATOR_V2.md) for the full grammar,
input/output bounds, exit codes and failure semantics. A2 changes no A1/F2
runtime behavior, prior format, dependency or monetary/enrollment policy.

### Executed acceptance

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,137 workspace/all-target runtime tests**, zero failures: 1,083 retained plus
  **54 new** (23 parser, four bounded-output, three diagnostic and **24 actual-command
  process tests**). Parameterized cases, child invocations and reruns are not new tests.
- **Eleven compile-fail doctests**, unchanged from the accepted A1 baseline.
- All required release suites: personhood 234 runtime + 3 doctests, core 121,
  state 49, consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network 99 + 2,
  and node **127 runtime + 1 doctest**. Release reruns are not extra unique tests.
- All **twelve independent Python references** and **sixteen synthetic examples**.
  Three existing subprocess helpers remain intentionally ignored in ordinary
  enumeration and are exercised by their parent tests.
- Scoped formatting, whitespace, allocation/runtime-data and preservation checks.
  **153 of 154 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent checkpoint. Only node's Cargo.toml changes, to preserve V1 default
  invocation. Cargo.lock, every earlier runtime source, wire/hash/storage format,
  monetary implementation, reference and frozen fixture remain unchanged.

Actual executable processes verify all full init pins and refused overwrite,
read-only missing/clean/invalid/pending snapshots, rejection before effects,
zero TCP connections when local state is unready, and complete accepted state after
real TCP keep/adopt. The preferred fork has identical account bytes but different
membership history; accounts, memberships, approvals and population state are
checked through strict fresh reopening.

Invalid/truncated/missing-End/trailing histories, missing terminal EOF and an
actual one-byte session allowance reject. Explicit completion/retention, candidate
no-rollback and unknown/partial artifact preservation are tested. Canonical pending
recovery layouts are deliberate record/storage fixtures, distinct from A1's
retained actual publication-boundary process-termination evidence.

One real sync child is killed while blocked in reception: it holds both directory
owners until killed/reaped, competing commands fail before connecting, ownership
then releases and selected state remains unchanged. A deterministically closed
stdout after successful init returns an output error; a fresh status process proves
the clean completed initialization. These are finite component observations, not
physical power-loss certification, an external audit or Gate A/B acceptance.

### Next increment identified at the A2 checkpoint

**At the A2 checkpoint, next was MB-04B1 — bounded V2 provider lifecycle and shutdown.**
B1 is now delivered in section 29; the following preserves the historical scope.

Begin with an explicit design contract, then implement only the accepted first
provider slice through existing A1/F2 components:

1. Require deliberate control/store paths, independently selected C/genesis and
   one numeric listen address. Strictly open clean A1 state before binding. Missing,
   invalid, pending or busy local state must refuse startup without listening.
2. Specify one held controller and a fixed complete accepted history for that
   lifecycle. The current exclusive owner makes concurrent A2 status/sync return
   busy; document that behavior instead of weakening A1 locking or pretending the
   provider sees live updates. Define listener ownership around F2's consuming
   `serve_once` API before implementing a repeated accept path.
3. Define process-level admitted-attempt, active-connection, aggregate application
   byte and lifecycle-duration bounds. Recreating session owners/listeners must
   not replenish them. Failed attempts stay charged. Kernel backlog, TCP/IP overhead
   and other processes remain outside application-admission claims.
4. Define explicit idle and in-flight shutdown: stop acceptance, keep the existing
   attempt deadline, complete or reject bounded work, then release sockets and
   directory ownership. Synchronous validation is not instantly preemptible; do
   not promise immediate shutdown without a separately designed cancellation path.
5. Use real command processes to verify startup refusal before bind, malformed and
   slow peers, admission exhaustion, both shutdown states, lock release and strict
   fresh-process reopening. Preserve whole-history validation and finite resource
   assumptions.

This is **next work, not behavior supplied by A2**. Do not bundle automatic
initialization/recovery, reconnect/rejoin or multi-peer scheduling, a transaction
pool, wallet/signer or unresolved monetary/admission/device choices into the first
provider slice. No policy decision blocks specifying these dependency-ready limits.

Run all AGENTS.md gates with Rust 1.98.0 and locked dependencies; preserve twelve
references, sixteen examples and every required release suite. Record exact source
acceptance separately from later documentation-head CI and refresh the handoff.

## 29. MB-04B1 acceptance — MIRROR-MB04B1-001

Updated **2026-11-27**.

**MB-04B1 is VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN.**
MB-04A2 remains verified for finite operator commands, A1 for durable local control
and explicit restart recovery, and MB-03 for its finite storage/recovery/peer scope.
MB-04 remains IN_PROGRESS: rejoin-and-serve coordination, broader reconnect and
multi-peer scheduling, transaction pool, wallet and wider operator APIs remain.
Complete Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain UNMET.

### Exact source and publication

Implementation source: **ed631eae05ebeebe2b3a58a433ab8a0cc5753150**.
Tested source tree: **dc32d623bf6a8e5e3ab03023b699a729e0c630a1**.
Parent checkpoint: **f507827aabcaecda85c02d66f0de6a10db8972ac**.
[Source PR CI 45904484131](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45904484131)
passed all **43 steps**. Full job `140197823058` logs were inspected and confirm
**1,167 runtime tests and eleven compile-fail doctests**, with zero failures.
Its actual synthetic PR checkout `cdb00671509b4f2876d51bd677ab39d861833396` has no file
differences from the implementation commit. All **17 published implementation,
test, contract, dependency and workflow files** were read back byte-for-byte against
locally tested content. Source acceptance: **PASSED — local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; source
acceptance does not assert that a future head is tested. PR #1 remains OPEN and
UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented provider contract

The existing `mirror-node-v2` executable now has an explicit finite `serve`
command. All A2 commands and the V1 default invocation remain available. The
complete contract is [NODE_PROVIDER_V2.md](docs/NODE_PROVIDER_V2.md).

| Boundary | Delivered behavior |
|---|---|
| Startup | Validate all arguments, install scoped Linux signal handling, then strictly open ready A1 control before binding. Missing, pending, corrupt, wrong-network or busy state refuses startup without listening. |
| Fixed selection | Retain one controller and its complete accepted history. Concurrent status/sync commands are busy; no history refresh or controller reacquisition occurs between clients. |
| Admission | One nonblocking listener, one retained F2 owner and at most one active admitted stream. Failed sessions keep their charged attempts and actual read/write bytes. |
| Shutdown | SIGINT/SIGTERM stop new admissions. An already admitted session completes or rejects under its original absolute F2 deadline. Admission expiry follows the same final-drain rule. |
| Final output | Release streams, listener and both directory owners before one bounded result record. Report full selected pins, reason, charged attempts/bytes and completed/rejected counts. |

Require explicit control/store paths, independent config/genesis hashes, numeric
nonzero-port `--listen`, `--max-attempts` 1..8, `--max-io-bytes` 1..157129488,
`--timeout-ms` 1..60000 and `--admission-ms` 1..60000. The existing parser caps remain
32 arguments after argv[0], 4,096 UTF-8 bytes per token and 16,384 total. No defaults,
DNS, environment/stdin/config fallback or side effects before complete validation.

The admission clock starts after A1 opening, immediately before bind. Check stop,
admission time, remaining bytes and attempts before accept, and check stop/time
again after accept. A raced stream closes before F2 and is not an admitted attempt.
A successful TCP handshake or kernel-backlog entry is not application admission.
One owner retains all attempt/byte charges across clients; creation of a fresh owner
cannot replenish this invocation's budget because the loop never creates one.

`PeerSessionsV2::serve_connection` adds an accepted-stream entry with an immediate
shutdown/drop guard, including preflight refusals. It shares the checked request,
EOF and history-output routine with `serve_once`; the latter retains its original
pre-accept deadline and consumed-listener ordering. The 72-byte MPQ2 request,
mandatory request EOF, complete F1 history and all original encodings remain.

The handler only sets a sticky atomic stop flag and prior handlers are restored on
exit. Final drain retains at most one original attempt timeout after the admission
window; configured ceilings total 120,000 ms. This is deadline-aware synchronous
behavior, not a hard process wall-clock guarantee: processing and scheduling are
not preempted, and initial open/replay plus final output lie outside that interval.
A1 opening may synchronize; it is distinct from read-only status. SIGKILL is forced
termination and does not establish graceful drain or physical power-loss safety.

Ordinary stop priority is shutdown, admission timeout, wire limit, then attempt
limit. Completed plus rejected equals charged attempts; the last static session
error persists across later successes. Normal limits/signals exit 0 even with
rejected peers, operational/output failures exit 1, and invalid commands exit 2.
Only status uses exits 3/4. Peer input errors can consume a bounded attempt; local
history/allocation/setup/listener errors terminate rather than masquerading as peer
rejections. Diagnostics echo no raw paths, addresses or peer bytes.

There is no readiness/progress output while resources are held. Final records stay
within 4,096 bytes. Output failure may follow completed serving and reports
`operation_may_have_completed=true` with explicit status inspection, without retry.
Provider completion reports local transfer completion, not remote validation,
durable adoption, global latestness or human finality.

### Executed acceptance

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,167 workspace/all-target runtime tests**, zero failures: 1,137 retained
  plus **30 new** (six parser, nine accepted-stream adapter and fifteen
  actual provider-process tests). Parameterized cases, subprocesses and release
  reruns are not extra unique tests.
- **Eleven compile-fail doctests**, unchanged from A2. Three retained subprocess
  helpers remain intentionally ignored in ordinary enumeration and are exercised
  by their parent tests.
- Required release suites: personhood 234 runtime + 3 doctests, core 121, state 49,
  consensus 23, chain 85 + 3, storage 350 + 2, protocol 38,
  network **108 + 2**, node **148 + 1**.
- All **twelve independent Python references** and **sixteen synthetic examples**;
  scoped formatting, whitespace and allocation/runtime-data checks also passed.

The new process tests establish refusal before bind; complete fixed-history
responses across clients; exact successful/rejected counts and cumulative bytes;
malformed/wrong-network/short/trailing/missing-EOF handling; idle and active
SIGINT/SIGTERM; unchanged in-flight deadlines; admission expiry and final drain;
competing commands busy with zero outbound sync contact; forced termination;
strict fresh reopening; and closed or blocked final output.

The blocked-output case fills a CLOEXEC pipe before launch, observes the child
actually blocked writing fd 1, then proves its listener is gone and a new status
process succeeds **while the provider child is still blocked**. Only afterward does
the parent close the reader and observe the output error. This verifies release
before output, rather than only release after process exit.

Admission/readiness evidence ties bounded Linux `/proc` socket-inode/descriptor
and blocked-syscall observations to the specific child and connection, including
the EOF probe. It uses no production readiness hook, ptrace or permission change.
Parents use bounded waits, byte/descriptor caps and kill/reap cleanup. The receive
helper has one original five-second deadline; direct store ownership requires the
exact wrapped Busy error. Unsupported observation fails visibly; these process
results apply to the tested Linux environment, not every OS/filesystem.

**153 of 161 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Eight deliberate changes are the workflow, lockfile, F2 session
module, node manifest and four existing V2 command modules. All earlier tests,
references, frozen fixtures, V1/A1/storage/consensus and monetary code remain.
The node adds a direct Linux-target edge to already locked `libc` 0.2.189; no package
version, source or checksum changes. CI timeout rises from 15 to 20 minutes because
the prior run took about 14 minutes 30 seconds and this slice adds process tests
in debug and release. Every prior verification command/assertion remains required.
No wire/hash/storage encoding, enrollment/device policy or runtime data changed.

### Next increment identified at the B1 checkpoint

**At the B1 checkpoint, next was MB-04B2 — finite rejoin and serve coordination.**
B2 is now delivered in section 30; the following preserves the historical design scope.

Specify one deliberate lifecycle before implementation:

1. Require explicit control/store paths, independent C/genesis and one chosen
   numeric incoming peer. Strictly open A1, receive at most one complete F2
   candidate, then apply the existing consuming keep/adopt rules.
2. Retain the returned ready controller and serve its resulting fixed complete
   history. Do not release/reacquire ownership between adoption and serving or
   accept another branch while serving. Define listener startup and failure order.
3. Use one cumulative F2 attempt/actual-byte budget across reception and serving.
   Neither phase nor a new owner may replenish it. Specify how the incoming attempt
   affects remaining service capacity; keep all failed charges and original
   deadlines. There is no concurrent receive/serve or automatic reconnect.
4. Specify shutdown before reception, after validated receipt but before adoption,
   during durable adoption and during serving. Durable publication is not
   asynchronously cancellable. A later bind or output failure must truthfully
   report that adoption may already have completed and direct explicit inspection.
5. Verify three independent processes/directories: seed provider -> rejoining node
   -> third requester. Include explicit clean restart/rejoin and complete accounts,
   membership, approvals and population agreement, plus pending-state refusal,
   ownership, cumulative limits and selected-candidate no-rollback.

B2 is planned work, not supplied by B1. Automatic initialization/recovery, peer
discovery, multi-peer scheduling, pool, wallet and unresolved monetary/enrollment/
device choices remain outside this increment. Full MB-04 and Gate A/B remain open.
No policy decision blocks this dependency-ready design.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies. Preserve twelve
references, sixteen examples and all required release suites. Record exact source
acceptance separately from later documentation-head CI and refresh the handoff.

## 30. MB-04B2 acceptance — MIRROR-MB04B2-001

**MB-04B2 is VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION.**
MB-04B1 remains verified for finite serving and draining shutdown, A2 for operator
commands, A1 for durable local control and explicit restart recovery, and MB-03
for its finite storage/recovery/peer scope. MB-04 remains IN_PROGRESS: broader
reconnect and multi-peer scheduling, transaction pool, wallet and wider operator
APIs remain. Complete Ant Kiss remains MB-05 and final human consensus MB-06.
Gate A/B remain UNMET.

### Exact source and publication

Implementation source: **bbac1622be18c97ab640f6834a158e5378f1d172**.
Tested source tree: **d724df618f6b45429abebaa3c24bb52ad742799b**.
Parent checkpoint: **966286bec62ce422db6bc091c472d74659935344**.
[Source PR CI 45927607784](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45927607784)
passed all **43 steps**. Full job `140272547972` logs were inspected and confirm
**1,201 runtime tests and eleven compile-fail doctests**, with zero failures.
The actual synthetic PR checkout `e00fbb793f5eb417cc0f0cda1ed52443c5cef34d` has no file differences from
the implementation commit. All **13 published implementation, test and
contract files** were read back byte-for-byte against locally tested content.
Source acceptance: **PASSED — all 38 local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; this
source acceptance does not assert that a future head is tested. PR #1 remains
OPEN and UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented rejoin contract

The separate `mirror-node-v2` executable adds `rejoin-serve`. It performs
at most one explicit complete receive, existing A1 keep/adopt, then finite serving
of the resulting fixed history. The contract was written before implementation:
[NODE_REJOIN_V2.md](docs/NODE_REJOIN_V2.md). Existing A2 commands, B1 `serve` and
the V1 default invocation remain available.

| Boundary | Delivered behavior |
|---|---|
| Startup | Complete argument validation, one signal guard, strict clean A1 opening before either outgoing contact or bind. Missing, pending, corrupt, wrong-network or busy state refuses. |
| Receive | One unchanged F2 request, actual complete history execution, matching End and mandatory terminal EOF. A failure is fatal even if shutdown was requested; there is no serve-old fallback. |
| Adoption | A stop observed after validated receipt but before adoption admission discards the candidate. Once admitted, synchronous A1 keep/adopt finishes or fails through its unchanged durable ordering. |
| Ownership | Retain `outcome.into_control()` directly. The same controller, signal state and F2 attempt/byte allowance span both phases; no reopen, reset or concurrent receive/serve. |
| Service | Check shutdown and exhausted budgets before binding. Serve the selected whole branch with B1 admission and original-deadline drain. A last-allowed receive can adopt and then stop without binding. |
| Reporting | Release sockets and both directory owners before a bounded 26-field result. Report previous/selected pins, actual choice and separate receive/service usage plus combined totals. |

All ten options are deliberate: separate control/store paths, independent C/genesis,
one numeric nonzero-port `--peer`, numeric nonzero-port `--listen`, `--max-attempts`
1..8, `--max-io-bytes` 1..157129488, `--timeout-ms` 1..60000 and `--admission-ms`
1..60000. Existing 32-argument, 4096-byte-token and 16384-total-byte parser bounds
remain, with no DNS, environment/stdin/config fallback or effects before validation.

Receipt charges one attempt and its actual I/O. Service uses only the remainder.
An allowance equal to the 72-byte request plus complete response bytes is still
insufficient: the terminal EOF probe needs one unspent byte of capacity even though
zero-byte EOF is not charged. The service admission clock starts after adoption,
immediately before bind when permitted. Receive timeout plus admission window plus
at most one final service timeout totals at most 180000 ms at configured ceilings.
This excludes opening, durable adoption and output and is not hard CPU preemption
or a complete wall-clock guarantee. Standalone B1 retains its 120000-ms allowance.

The post-receive stop check admits adoption. Signals after that check do not cancel
A1 or imply rollback. Every adoption error or later fatal bind/listener/service error preserves
its static error class and reports `operation_may_have_completed=true` with
`next=inspect-status`; no error asserts unresolved selected pins. Final output loss
uses the existing conservative rule, including normal pre-adoption shutdown.
No implicit initialization, recovery, retry or rollback is introduced.

Normal results obey receive_attempts + serve_attempts = attempts,
completed_sessions + rejected_sessions = serve_attempts, and receive_wire_bytes +
serve_wire_bytes = wire_bytes. A validated candidate discarded on shutdown reports
`receive_status=validated`, `choice=not-attempted` and the unchanged verified
selection. Normal limits/signals exit 0; operational/output failures exit 1 and
usage errors exit 2. Only status retains exits 3 and 4. No result claims peer
adoption, global latestness, human finality or protection against a later local writer.

### Executed acceptance

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,201 workspace/all-target runtime tests**, zero failures: 1,167 retained
  plus **34 new** (six parser, one diagnostic and
  **27 actual rejoin-process tests**). Parameterized cases,
  subprocesses and release reruns are not extra unique tests.
- **Eleven compile-fail doctests**, unchanged; three retained subprocess helpers
  remain intentionally ignored in ordinary enumeration and exercised by parent tests.
- Required release suites: personhood 234 runtime + 3 doctests, core 121, state 49,
  consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network **108 + 2**,
  node **182 + 1**.
- All **twelve independent Python references** and **sixteen synthetic examples**;
  scoped formatting, whitespace and allocation/runtime-data checks also passed.

The 27 added actual-process tests cover complete seed `serve` -> `rejoin-serve`
-> third `sync-once` transfer with independent directories and an explicit second
restart/rejoin cycle. Fresh strict reopening compares full histories, account
bytes, membership records/bytes, retained approvals, sequences, population and
resource counters. The seed keeps local generation 0 while adopters advance to 1;
the restart keeps those local generations and complete state unchanged. The
preferred fork retains equal account bytes but different membership state.

Further cases cover effect-free parsing; local missing/pending/corrupt/mismatched/
busy refusal before contact or bind; identical/shorter/tied keep-current; shared
attempts and actual bytes; the exact terminal-EOF capacity boundary; rejected and
partial serving; malformed/trailing/incomplete/wrong-network sources; idle/active
service drain; original deadlines and admission timing; post-adoption bind/output
uncertainty; and selected/confirming/confirmed no-rollback fixtures. Constructed
pending layouts are explicitly separate from actual interrupted processes.

Bounded Linux observations identify the actual child's socket endpoints, inode,
descriptor and blocked syscall. SIGINT/SIGTERM during complete or partial receipt
permit original-deadline completion/failure; validated completion under a prior
stop discards the candidate before adoption. Directory descriptors and competing
busy commands are checked across controlled receive/service phases; source review
also verifies direct retained-controller flow. These observations do not claim
to sample every hypothetical microscopic ownership gap.

The adoption test extends a valid synthetic history by 128 mined empty blocks.
An inotify event only prompts SIGSTOP. While the actual child is verified stopped,
the test requires the complete canonical intent, independently validates both
histories and checkpoints, confirms both owners remain held and no listener exists,
then verifies caught, unblocked SIGTERM is pending before SIGCONT. Adoption finishes,
retires intent and reopens the candidate at generation 1, followed by normal shutdown
despite an occupied listen port. This observation passed in debug and release; it
establishes signal generation during live adoption, not a particular fsync boundary.

A full-pipe case observes the child blocked on final stdout while its listener is
gone and fresh status plus strict writer opening already succeed. SIGKILL cases
separately preserve the old branch during reception and the new branch during
adopted service. Existing A1 publication-failure/process tests remain the evidence
for their exact durable substeps. Zero-attempt stop before initial request is not
artificially forced by this new process suite; that branch retains source-level
lifecycle evidence. Process tests use Linux IPv4 loopback; numeric IPv6 parsing
has CLI coverage. No test adds a production hook, permission change, hidden retry,
power-loss guarantee or untested-filesystem claim.

**160 of 166 retained source/dependency/workflow/reference/fixture
paths remain byte-identical**; 6 existing private V2 command
modules change deliberately. New code adds the rejoin composition and its process
suite/helper. Earlier tests, lower-layer/V1/A1/F2/storage/consensus/monetary source,
public APIs, Cargo manifests and lockfile, all twelve references, frozen fixtures,
workflow commands and its 20-minute job budget remain unchanged. No wire/storage/hash
encoding, enrollment/device policy, dependency selection or runtime data changes.

### Next increment identified at the B2 checkpoint

**Next: MB-04B3 — bounded explicit-peer failover. READY — DESIGN FIRST.**

Specify a separate finite acquisition command before implementation:

1. Accept a small, bounded ordered list of explicitly chosen numeric peers, with
   at most one request per entry. Define list/argv bounds and duplicate-address
   treatment; preserve all existing commands and their behavior.
2. Retain one A1 controller, one sticky signal guard and one cumulative F2
   attempt/actual-byte owner. Define the failure-advance allowlist: only selected
   pre-adoption peer/transport failures may advance. Local authority, allocation,
   internal-limit and adoption failures remain fatal.
3. Stop acquisition at the first fully validated candidate, then perform exactly
   one existing keep/adopt and B2 fixed-history serving. A valid keep-current
   candidate also ends acquisition. Do not imply best-peer selection or global
   latestness. Exhausted peers without a validated candidate fail without serving
   the old branch as a fallback; successful receipt exhausting shared allowance
   retains B2's normal no-bind stop.
4. Specify cumulative network-phase time, unchanged in-flight deadlines, stop
   between and during requests, bounded per-peer diagnostics without raw peer
   disclosure, and truthful possible-completion context after adoption admission.
5. Verify unavailable/malformed first peer -> valid second peer -> third requester,
   no later peer contact after validation, all-peer failure, shared limit exhaustion,
   pending refusal, shutdown and complete-state restart/reopening through actual
   processes and independently owned directories.

This is a planned narrow scheduler step under MB-04, not an implemented capability.
Repeated-peer reconnect/backoff, discovery, parallel sessions, multiple durable
adoptions, an indefinite daemon, pool and wallet remain later work. No monetary,
enrollment, consensus or device policy is selected. Full MB-04 and Gate A/B remain open.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies. Preserve twelve
references, sixteen examples and all required release suites. Record exact-source
acceptance separately from later documentation-head CI and refresh the handoff.

## 31. MB-04B3 acceptance — MIRROR-MB04B3-001

**MB-04B3 is VERIFIED_COMPONENT — BOUNDED EXPLICIT-PEER FAILOVER.**
B2 remains verified for finite rejoin/serve coordination, B1 for provider/draining
shutdown, A2 for operator commands, A1 for durable control/explicit recovery and
MB-03 for its finite storage and peer scope. MB-04 remains IN_PROGRESS: repeated
run scheduling, transaction pool, wallet and wider operator APIs remain. Complete
Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain UNMET.

### Exact source and publication

Implementation source: **3311151ab627b3512abed47be20adea2f0e6d66a**.
Tested source tree: **fc238ae2df95b7b2b3df2457d06f739048812c7e**.
Parent checkpoint: **9ca7d99681877d014ae64aab31cb5eb22a957c35**.
[Source PR CI 45933868084](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45933868084)
passed all **43 steps**. Full job `140292436668` logs confirm **1,242 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`0cada2958afe985bb5fcd3b683c57a40b01042ce` has no file differences from the implementation source.
All **12 published implementation/test/contract files** were read back byte-for-byte.
The later checkpoint updates nine status/handoff records and clarifies one
contract compatibility wording: existing operational records remain unchanged; the help
catalogue gains the new command and options. It changes no runtime/test file,
and the other 198 files frozen during source acceptance remain byte-identical.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; this
source record does not assert that a future documentation head is tested.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented failover contract

The separate `mirror-node-v2 rejoin-failover-serve` command accepts one to four
ordered numeric peers in a single required `--peers ADDRESS[,ADDRESS...]` token.
The other nine deliberate options, 32-argument/4096-token-byte/16384-total-byte
limits and effect-free validation remain. Reject empty/malformed/scoped lists,
duplicate numeric endpoints including IPv4-mapped aliases, DNS and zero ports.
Original parsed connection addresses are retained; no distinct-physical-node
or peer-authentication claim follows from endpoint checks.

Strict clean A1 opening precedes all contact and binding. One controller, sticky
signal guard and cumulative F2 owner span a finite ordered acquisition prefix,
at most one admitted keep/adopt and fixed-history service. Each configured entry
is requested at most once. Classify the typed session/history error before
sanitizing it: only the explicit contract allowlist advances. Allocation,
authority, accounting/global-resource and all unlisted errors remain fatal,
including ambiguous nested configuration/operation/ledger failures. A failed
receive plus shutdown remains an operational error, with no next-peer contact.

The first complete validated candidate permanently ends acquisition, including
identical/shorter/tied keep-current choices. Matching End and terminal EOF remain
mandatory. The shared B2 post-receive routine handles adoption admission, existing
A1 durability, retained-controller service and checked phase deltas. Stop before
admission discards a validated candidate; admitted adoption finishes or fails.
All-peer failure never serves the old branch as a fallback.

Configured caps remain 1..8 total receive/service attempts and 1..157129488
actual I/O bytes;
each original attempt timeout and post-adoption service admission remain
1..60000 ms. Failed/partial traffic is charged without refund or replenishment.
The EOF probe needs one unused byte even though zero-byte EOF is not charged.
A final allowed successful receive can adopt then stop normally without binding.
For P peers, M attempts, timeout T and admission A, conservative configured phase
allowances are min(P,M)*T+A+T (at most 360000 ms) and M*T+A. All service attempts
remain within M; these are not hard wall-clock/CPU/RSS guarantees. Local opening,
adoption, nonpreemptible processing, scheduling and output retain their qualifications.

Normal output has 32..35 fields: B2's 26 fields plus bounded acquisition totals,
first validated ordinal, latest receive error, terminal acquisition reason and
one static result per configured entry. Service rejection evidence stays separate.
Errors attach sanitized acquisition evidence without selected-pin assertions.
Every adoption error and later fatal setup/service failure retains its category,
possible-completion warning and inspect-status guidance. Invalid optional trace
formatting cannot erase that core diagnostic. All sockets/owners/signal guards
are released before final output; output loss never repeats the operation.

### Executed acceptance

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed with **199 frozen noncheckpoint files unchanged**:

- **1,242 runtime tests**, zero failures: 1,201 retained plus **41 new** — six
  parser, eight typed-policy, three output, one error-context, one diagnostic-fallback
  and **22 actual-process tests**. Scenarios, subprocesses and release reruns are
  not additional unique test functions.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers are exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **223+1**. All **twelve references** and **sixteen examples** passed.
- Scoped formatting, whitespace and allocation/runtime-data checks passed.

Separate refused-first and malformed-first tests run a real seed executable,
the B3 executable and a third requester with independent roots and a second
explicit restart cycle. Strict reopening compares complete histories, accounts,
membership records/bytes, approvals, sequences, population and resource counters;
the preferred fork has equal accounts but different membership. First validated
identical/shorter/tied candidates stop acquisition and leave later peers untouched.

Actual cases cover all four peers failing, eligible malformed/wrong-network/
incomplete/trailing/missing-EOF transfers, and a constructed nested manifest
configuration error that remains fatal without later contact. Fixed invalid-header
exchanges establish 84 charged bytes (72 request + 12 consumed header). Exact
EOF-capacity and payload-larger-than-positive-remainder cases verify shared bytes;
last-allowed receive succeeds without binding an occupied address. Later successful
acquisition and service retain distinct earlier rejection evidence.

Bounded Linux observations identify the child socket endpoints/inode/FD and blocked
syscall before SIGINT/SIGTERM. They verify original deadlines, failed-receive stop,
later validated-candidate discard, both exact directory descriptors across two
receives and service, and busy competing commands with no outgoing contact.
Bind/closed-output failure preserves adopted state and possible-completion context.
Blocked final stdout releases the listener/owners before the write finishes;
SIGKILL during a partial second receive preserves old state. Unused listeners
remain reserved until exit when absence of later contact is asserted.

The retained B2 live-adoption signal and service-drain/no-rollback tests exercise
the same extracted post-receive path; they are separate retained evidence, not
22 additional B3 scenarios. No new production hook, permission change, induced
OOM, hidden retry, power-loss or untested-filesystem guarantee is introduced.
Pre-first-request zero-attempt and brief between-request shutdown branches retain
source-level evidence. Process tests use Linux IPv4; numeric IPv6 parsing is separate CLI coverage.

**164 of 169 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Five existing private command modules change: args, errors,
main, output and rejoin. New files add the coordinator, process suite/helper and
contract. All earlier test functions and process helpers, public/lower-layer/V1/
A1/F2/storage/consensus/monetary source, Cargo manifests/lockfile, references,
fixtures, workflow commands and 20-minute budget remain unchanged. No wire,
storage or hash encoding, enrollment/device or monetary policy changed.

### Next increment identified at the B3 checkpoint

**Next: MB-04B4 — bounded persistent-run lifecycle. READY — DESIGN FIRST.**

Design a finite run mode that can repeat acquisition, keep/adopt and service
rounds while retaining one controller, one sticky signal guard and one cumulative
transport allowance. Choose the operator interface during that design; do not
assume another rejoin-* command or change the accepted B3 command.

Before implementation define explicit round and repeated-peer authorization,
bounded inter-round waits, whole-invocation attempt/byte/time limits, first-valid
ordering across rounds, all-peer-failure policy and shutdown at every phase.
The existing private provider consumes/releases the controller, so specify an
ownership-returning round boundary without dropping/reacquiring either root.
Reacquisition pauses serving; no concurrent listener/request loop is implied.
Repeatedly choosing a stale but valid first peer is possible and must not be
described as convergence or global latestness.

Once any round admits adoption, every later fatal error must retain truthful
possible-completion context, even during a later receive or wait. Distinguish
the initial checkpoint, each completed round and the final selected checkpoint;
bound all reports and preserve resource release before output. Verify actual
multiple rounds, whole-state reopening, cumulative exhaustion, ownership and
shutdown before declaring that lifecycle implemented.

B4 is planned only. Indefinite operation, discovery, parallel scheduling, pool,
wallet and policy choices are not silently included. Full MB-04 and Gate A/B
remain unfinished; all monetary, enrollment/device and accessibility decisions
and proposals retain their existing gates.

## 32. MB-04B4 acceptance — MIRROR-MB04B4-001

**MB-04B4 is VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE.**
B3's explicit-peer failover, B2's rejoin/serve, B1's provider/draining shutdown,
A2's operator commands, A1's durable control/recovery and MB-03's finite storage
and peer components remain verified within their accepted scope. MB-04 remains
IN_PROGRESS: a pending-operation pool, wallet, wider operator APIs and broader
network scheduling remain unfinished. B4 is finite and does not complete the
production persistent-node work. Complete Ant Kiss remains MB-05 and final human
consensus MB-06. Gate A/B remain UNMET.

### Exact source and publication

Implementation source: **e768268fae2e4c5c9980cc74dc7a9973da56849c**.
Tested source tree: **daff130cc7652ea0946106f46a600c54671ecf18**.
Parent checkpoint: **e421494a5a2ba28992ad581af4067618c38c0dae**.
[Source PR CI 45941543950](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45941543950)
passed all **43 steps**. Full job `140316760168` logs confirm **1,281 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`e51ff7da8385981dd9d94e4329ed35861f2dc31f` has no file differences from the implementation source.
All **16 published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **203 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented run contract

The separate `mirror-node-v2 run` command requires all twelve deliberate
options. It uses B3's ordered one-to-four numeric peer list and existing common,
listen, attempt, byte, timeout and admission options, plus `--max-rounds` in 1..4
and `--round-wait-ms` in 0..60000. The old 32-argument, 4096-byte-token and
16384-total-byte limits remain. All validation precedes effects; existing commands
reject the new flags and `run` rejects the single-source `--peer` flag.

Strict clean A1 opening occurs once. One controller, sticky signal guard and
cumulative F2 owner span every acquisition, admitted keep/adopt, service phase and
wait. Successful private round helpers return the same owner after closing their
listener. Earlier commands remain dropping wrappers. Neither root is reopened or
released between rounds; at most one stream is active and acquisition pauses service.

Each entered round restarts the same ordered list at entry one, with one request
per entry and B3's unchanged closed typed advancement policy. Checked round-local
deltas determine peer ordinals; the actual cumulative owner enforces budgets.
First complete validated history, including terminal EOF, ends acquisition even
when stale or tied. All-peer or fatal failure ends the invocation even after
earlier successful rounds, without another-round retry or serving an old branch.

A round completes only after validated receipt, admitted successful keep/adopt and
normal service return. Service may consume the remaining allowance; no attempt is
reserved for a later round. Overall stop priority is shutdown, wire-limit,
attempt-limit, round-limit. Only a normal service admission expiry with allowance
and rounds remaining permits a bounded signal-aware wait and another acquisition.
Waits hold the roots but no listener or stream, and never occur after a final stop.

The unchanged invocation caps are 1..8 total attempts and 1..157129488 actual I/O
bytes, with each attempt timeout and per-round service admission in 1..60000 ms.
Failed/partial traffic is charged without refund. The terminal EOF probe still
needs one unused byte. For total attempts M, timeout T, round limit R, admission A
and wait W, `M*T + R*A + (R-1)*W <= 900000 ms` bounds configured network,
admission and wait allowances. Opening, synchronous replay/adoption/processing,
scheduling and output are separate; this is not a hard wall-clock, CPU or RSS bound.

Shutdown before adoption discards a validated candidate; admitted A1 completes or
fails. An active service request drains under its original deadline. Once any
round admits keep/adopt, every later fatal error preserves its original class,
possible completion and inspect-status guidance, including errors in later receipt
or bookkeeping. Keep-current admission also establishes that sticky context.

The fixed normal record has `30 + 16*C + 7*I` fields for completed rounds C and
optional incomplete acquisition I, with C+I<=4. It distinguishes initial, each
completed and final selected checkpoints, checked phase totals and retained prior
errors. Error context adds fifteen fixed keys and eight for a current acquisition;
it does not assert a selected checkpoint or unreturned service outcome counts.
Invalid optional evidence falls back to the safe original diagnostic. All sockets,
roots and signal guards are dropped before final output; output loss never repeats work.

### Executed acceptance

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed with **203 frozen noncheckpoint files unchanged**:

- **1,281 runtime tests**, zero failures: 1,242 retained plus **39 new** — six
  parser, five output, one failure-context, one diagnostic-fallback, seven lifecycle
  unit tests and **19 actual-process tests**. Parameterized cases, subprocesses and
  release reruns are not additional unique test functions.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers are exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **262+1**. All **twelve references** and **sixteen examples** passed.
- The focused first execution passed **82 binary tests and all 19 new process
  tests**. Scoped formatting, whitespace and allocation/runtime-data checks passed.

Actual processes cover two preferred adoptions under unchanged root descriptors,
complete reopened history/accounts/membership/counters, three real executables and
an explicit restart, repeated stale first-valid selection, and actual four-round
execution with one eight-attempt allowance. A three-round sequence with earlier
failed-peer and rejected-service attempts, ending in clean receipt/service, proves
earlier errors remain visible and peer ordinals restart independently of prior usage.

The suite exercises first/later all-peer or fatal refusal, keep-current followed
by a later fatal error, cumulative receive/service bytes and terminal-EOF exhaustion,
observed inter-round waits under SIGINT/SIGTERM, later validated-candidate discard,
original-deadline failure, later active-service drain, second-adoption bind failure,
closed/blocked final stdout, and SIGKILL during a later partial receive. A live
second-adoption SIGTERM case observes and verifies the actual A1 intent before
queuing shutdown; the admitted operation finishes without binding afterward.

Linux observations identify owned directory descriptors and relevant socket/blocked
syscall or verified intent phases. Busy competing commands and unused listeners
check retained ownership and absence of later contact. Conservative independent-
value output envelopes format to exactly **3918** and **3529 bytes**; these are
synthetic width bounds, not simultaneously reachable runtime counter combinations.
Validated normal records and additive help stay within the unchanged 4096-byte cap.

Tests use Linux IPv4; numeric IPv6 parsing is separate CLI evidence. Pre-first-round
and narrow between-request signal races retain source-level coverage. New SIGKILL
evidence concerns a later partial receive, not forced death at every A1/fsync or
service boundary. Process death and replay do not prove physical power-loss safety.
No production test hook, induced OOM, hidden retry or untested-filesystem guarantee
is introduced. First-valid repeated rounds do not prove convergence or latestness.

**164 of 172 retained source/dependency/workflow/reference/fixture paths remain
byte-identical.** Seven existing private command modules change intentionally:
args, errors, failover, main, output, provider and rejoin. The sole retained test
helper change appends an observed-wait method; all previous helper bytes remain an
exact prefix. All **62 earlier private unit-test bodies** and every other retained
test/helper file remain byte-identical. Four new files add the coordinator,
process suite/helper and contract; four existing command contracts gain additive
cross-references. Public/lower-layer/V1/A1/F2/storage/consensus/monetary source,
Cargo manifests/lockfile, reference tools, frozen fixtures, workflow commands and
the 20-minute CI budget remain unchanged. No wire, storage or hash encoding,
enrollment/device or monetary policy changed.

### Historical B4 continuation

**Next: MB-04C1 — bounded local pending-operation pool. READY — DESIGN FIRST.**

Specify a local in-memory component for bounded signed `SignedOperationV2` inputs,
covering transfers and membership operations without changing their accepted bytes
or execution rules. Separate retained input, contextual validity, executable ordered
batches and canonical inclusion. Admission is neither spendability nor an inclusion
promise, and a pool must never acquire chain/store authority by holding a staged result.

Before implementation define independently named local entry, encoded-byte,
batch-size and cumulative validation/revalidation work caps. Check them before
input-sized copying or expensive validation. Canonical membership `max_pending`
is not a transaction-pool limit. Specify exact signed-operation-ID deduplication,
stable bounded ordering, full-pool behavior, expiry/anchor handling, conflicts and
removal. Transfer account and membership record sequences are separate domains;
do not collapse them into one actor/sequence namespace or invent fee replacement.

Bind evaluation to an actual immutable accepted `ChainV2` and its exact selected
parent checkpoint. Existing contextual/signature validation does not execute
balances or consume sequences; use the complete ordered mixed-operation ledger
stager to assess executability atomically. Define explicit reconciliation after
append or replacement, stale cached results, included/expired inputs, and handling
of gaps or future inputs. A narrow next-block-only first scope is possible, but
that choice and its failure behavior must be specified before code.

The pool must not bind staged ledger state, append or mine a block, sign, mutate
A1/store, persist, gossip, add submission CLI/API ingress or assume wallet custody.
Those remain separate increments. Verify duplicates, ordering, overspend/sequence
conflicts, mixed-operation late failure, caps and selected-parent changes without
mutating the accepted chain. Preserve zero-fee/no-mint development semantics;
production monetary, enrollment, device, privacy and accessibility decisions remain
at their existing unresolved gates. C1 is planned only. Broader scheduling, wallet,
operator APIs, full MB-04 and Gate A/B remain unfinished.

## 33. MB-04C1 acceptance — MIRROR-MB04C1-001

**MB-04C1 is VERIFIED_COMPONENT — BOUNDED LOCAL PENDING-OPERATION POOL.**
C1 adds an in-memory library component for context-valid signed retention,
one atomic ordered batch evaluation and explicit selected-parent reconciliation.
B4's finite run rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown,
A2 operator commands, A1 durable control/recovery and MB-03's storage/peer components
retain their accepted boundaries. MB-04 remains IN_PROGRESS: finite operator pool
intake, persistence/gossip integration, wallet, wider APIs and broader scheduling
remain unfinished. Complete Ant Kiss remains MB-05 and final human consensus
MB-06. Gate A/B remain UNMET.

### Exact source and publication

Implementation source: **94207e713dd89f4d33586263c510591be0e78201**.
Tested source tree: **9c40a4852c8ab935cd7318d33d81d0720cd0dd15**.
Parent checkpoint: **35bbd0817843c35a42ff8c09b779391df331d53c**.
[Source PR CI 46086769527](https://github.com/Ahmed-Sobeih/mirror/actions/runs/46086769527)
**attempt 2** passed all **43 steps**. Full job `140804733638` logs confirm **1,333 runtime
tests and eleven compile-fail doctests**, zero failures and three retained
parent-exercised ignored helpers. Actual synthetic checkout
`17ece65483bd57dbfb3c6e11c64895ddf0afbbe2` has no file differences from the implementation source.
All **seven published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **208 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

### Implemented local pool contract

The new `mirror_node::v2::pool` component borrows an actual accepted `ChainV2`.
Its private binding contains configuration/genesis, height, tip and accepted ledger
root. Five deliberate local limits cap retained entries (at most 256), encoded
operation bytes (74,496), one batch's operations (64), complete framed block bytes
(19,048), and cumulative abstract work (67,108,864). Individual input is capped at
291 bytes before decoding/ID/signature work. Canonical membership `max_pending`
remains unrelated, and canonical operation/body limits apply independently.

Admission decodes the exact existing signed operation, deduplicates its complete
signed ID against retained entries and validates it for the selected next block.
Stable FIFO retention permits context-valid overspends, sequence gaps, competing
sequences and unknown membership targets. An exact duplicate preserves its position
even at full capacity; a distinct full-pool input never evicts another entry.
Admission does not scan history, so an already-included but still context-valid
input may be retained until evaluation/reconciliation. Future windows and invalid
expiry/anchor/signature/configuration reject. No fee replacement is invented.

Evaluation chooses exactly one requested nonempty FIFO prefix. Local count/framed-
byte and canonical complete-body preflight precede copying or execution. One full
`LedgerV2::stage_operations` call preserves original mixed order and atomic accounts/
membership semantics, including separate account and member sequence domains.
The staged ledger is discarded even on success; only immutable ordered input and
parent/revision observations are returned. Failures retain original indexed ledger
errors and leave all pool entries and accepted chain state unchanged.

Explicit reconciliation handles the same parent, append or accepted replacement
within the same configuration/genesis. It reserves complete work before history
ID hashing, contextual checks or temporary allocation, scans accepted history
once. Each input's removal reason prefers exact inclusion over contextual
invalidity; removal reports and survivors keep FIFO order. Context-valid stateful
blockers remain. Parent,
entries, exact bytes and revision publish together; budget/allocation/arithmetic
failure preserves the prior binding and semantic entries. Removed inputs are not
automatically resurrected after a replacement.

Work is cumulative for one non-Clone owner. Identification charges failed/duplicate
attempts, each evaluation charges the current parent-state size again, and every
reconciliation charges accepted history plus bounded retained-ID comparisons and
revalidation. No failure, clear, removal or parent change refunds work. The meter
is an explicit abstract budget, not CPU/RSS/wall-clock or a global rate guarantee.
Every successful reconciliation and actual entry mutation invalidates earlier
batch observations. Currentness checks the real chain, full parent, revision and
exact prefix; a different pool's different entries cannot match on revision alone.

The component never returns/binds staged state, appends/mines/prepares blocks,
signs, mutates A1/store, persists, gossips, exposes submission CLI/API ingress or
assumes wallet custody. Existing commands do not create a pool automatically.
Retained input, contextual validity, evaluated execution and canonical inclusion
remain separate. Actual block acceptance still checks complete blocks, timestamps,
work, roots and cumulative chain budgets. Zero-fee/no-mint development semantics
and all monetary, enrollment, privacy, device and accessibility decisions remain.

### Executed acceptance

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed, with all **217 tested source/checkpoint inputs
unchanged during execution**:

- **1,333 workspace/all-target runtime tests**, zero failures: 1,281 retained plus
  **52 new** — 31 admission/evaluation and 21 reconciliation/work integration tests.
  Parameterized cases, subprocesses and release reruns are not additional tests.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers remain exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **314+1**. All **twelve references** and **sixteen examples** passed.
- Both new focused suites passed on their first execution: **31 and 21** tests,
  with no execution-driven production/test corrections. Scoped formatting,
  whitespace and allocation/runtime-data checks passed.

The first full local workspace attempt and one unchanged targeted retry failed in
the retained B4 test's inter-round observer: it waited for requester durability
before looking for a 200 ms phase. Both captured runner reports completed two
legal rounds successfully. The narrowly repaired test starts the requester
concurrently, observes the real listener-free wait, verifies an owned SIGSTOP,
retains the owner/busy checks, awaits requester completion and resumes before the
second request. Original timing values, two-request outcomes and restart/state
assertions remain. The repaired targeted case and all required gates passed on
the final source. Failed attempt evidence is retained; production lifecycle and
C1 implementation/test semantics were not changed to suppress the failure.

The final source's first hosted CI attempt passed every test/release gate, then
was cancelled during the examples just after the configured 20-minute job limit.
Its log records only a generic cancellation; timing supports a budget-boundary
explanation but does not establish a more specific cause. That incomplete attempt
is preserved and not accepted. The unchanged source/workflow was rerun; the
successful complete attempt is identified in the source record above. The
20-minute workflow limit and all verification commands remain unchanged.

The new suites construct actual mined, manifest-pinned accepted chains with public
synthetic keys. They cover all four operation kinds, exact signed bytes/IDs,
malformed/signature/context refusals, duplicates at full capacity, actual 256-entry
and 64-operation limits, independent local/canonical quotas, framed batch bytes,
FIFO order/removal, retained sequence gaps/conflicts, receive-before-spend,
aggregate overspending, independent account/member sequences and mixed late failure.

Reconciliation evidence covers exact included IDs from earlier history, unchanged-
parent inclusion, included-before-expiry reporting, survivor order, same-height and
shorter replacements, changed/old anchors, newly future windows, wrong configuration
and distinct accepted genesis, no resurrection and stale/cross-pool batches.
Literal boundary checks verify successful/failed state-work charges, historical
scan/revalidation charges, partial admission reservations, sticky exhaustion,
atomic failed reconciliation and no refund from removal/clear/parent changes.

Complete chain equality and exact retained-state observations establish this
in-memory boundary. New C1 tests do not add command/process, socket, persistence,
wallet or power-loss evidence; the existing required process/recovery suites are
retained and rerun. Fallible bounded pool allocation paths are reviewed but induced
OOM is not tested. Abstract work units are not CPU-time, RSS or global DoS limits.
No runtime behavior is inferred merely from the written contract.

**210 of 212 existing repository files are byte-identical to the B4
checkpoint before the later nine-document handoff.** The sole existing Rust change
in production is one `pub mod pool;` export in the V2 node module; removing that
line reproduces its entire earlier body byte-for-byte. **68 of 69 retained
test/helper files** and **173 of 175 retained source/dependency/workflow/reference/
fixture paths** remain byte-identical. The sole retained test exception is the
documented B4 observation-coordination repair; all other 18 test bodies and every
byte outside that function in its file are preserved. Five new files add the pool,
two suites, an isolated synthetic fixture and the contract. Existing command implementations, A1/F2/storage,
lower-layer/V1/consensus/monetary bodies, Cargo manifests/lockfile, workflow commands
and its 20-minute budget, reference tools and frozen vectors are unchanged.
No wire, storage, hash, monetary, enrollment, device or accessibility policy changed.

### Exact next increment

**Next: MB-04C2 — finite local pool intake and inspection. READY — DESIGN FIRST.**

Design the smallest explicit operator path that can feed already signed V2 inputs
into C1 and inspect one local result against a clean accepted A1 chain. Specify the
command, deliberate input source/framing and finite invocation lifetime before
implementation. Preserve C1's exact-parent binding, contextual-only admission,
FIFO batch semantics, independent limits and cumulative work; never treat a pool
receipt as accepted chain state or permission to publish a block.
Require independently selected configuration/genesis identity; signed input must
never select its own network or accepted parent.

Define argument, input-file/stream, item-count and total-byte ceilings before any
input-sized read/copy, and a closed policy for malformed framing, duplicates,
individual rejections, budget exhaustion and a blocked requested batch. Choose
all-or-nothing intake versus bounded per-item reporting explicitly; do not let a
decoder or reporting loop silently reset C1's owner allowance. Keep retained,
contextually admitted, evaluated and canonically included observations distinct.
Choose bounded regular-file intake with explicit file-type/symlink/EOF rules, or
specify absolute deadline, cancellation and terminal-EOF behavior for streams.
Byte and item limits alone do not bound a stalled pipe, FIFO or stdin read.

Define clean controller ownership and selected checkpoint capture, shutdown/error
semantics, bounded output and release-before-output behavior. Any A1 open/replay/
synchronization must retain its existing contract. Report the exact observed
parent and finite local outcome; do not imply durable pool retention across exit,
network propagation, inclusion or currentness after owner release. Test actual
operator processes, malformed/overlong input, independent caps, duplicate/order
handling, contextual/execution errors, output failure and unchanged canonical
history/control selections before claiming operator integration.
Opening the A1 writer to borrow its accepted chain includes ownership, replay and
synchronization; unchanged canonical selection does not turn it into the existing
read-only, no-synchronization `status` path.

No wallet signing/key custody, automatic mining/append, new monetary rule,
transaction gossip, pool persistence, remote submission endpoint or integration
into B4's retained network loop is implicitly authorized by this narrow step.
Those require separately specified increments. Keep existing commands, A1/F2/store
authority, consensus encodings and all policy gates unchanged. C2 is planned only;
broader scheduling, wallet, wider APIs, full MB-04 and Gate A/B remain unfinished.

Run every existing AGENTS.md gate, verify published source and exact-source CI,
then check the later checkpoint head separately and refresh this handoff.
