# Mirror master plan — blockchain to biometric enrollment readiness

Roadmap: **MIRROR-PLAN-001** (established 2026-11-18; checkpoint 2026-11-26).
Design checkpoint: **MIRROR-MB01A-001 — design deliverables complete**.
Latest implementation: **MIRROR-MB03E2B-001 — explicit interrupted replacement recovery**.
MB-01B/B1–B4, MB-02A/B/C and MB-03A/B/C/D/E1/E2a remain verified within component scope.
MB-03E2b completes MB-03E2/E's local replacement/recovery component. Section 24 records
its acceptance. **MB-03 remains IN PROGRESS**; fresh-peer reconstruction and persistent
V2 node integration remain unfinished.

## Start here: current position and next action

**Next task: MB-03F — bounded fresh-peer V2 reconstruction. READY.**

Explicit recovery validates both independent histories and all residual bytes before
completion/retention, preserves root ownership, never rolls selected candidate back,
and returns a writer only after full clean replay/synchronization. Ordinary opening
remains strict. No existing encoding or economic policy changed.

Verification: **891 workspace tests, eight doctests, required release suites, nine references,
thirteen examples passed**. Section 24 records exact source/CI/scope. Gate A/B remain unmet.

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
[NEXT_SESSION.md](NEXT_SESSION.md), [decisions](docs/DECISIONS.md), and
[the-prompt](the-prompt). Active branch `work/personhood-register-v0.1`; PR #1 is
open and unmerged. Recheck live refs before acting; preserve newer/unpushed work.

Design: [ledger integration](docs/LEDGER_MEMBERSHIP_V1.md),
[threat review](docs/THREAT_MODEL.md), [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md),
[configuration](docs/CONFIG_V2_ENCODING.md), [signed membership](docs/OPERATIONS_V2_ENCODING.md),
[transfers/blocks](docs/CORE_V2_ENCODING.md), [atomic ledger](docs/LEDGER_STATE_V2.md),
[accepted chain](docs/CHAIN_V2.md), [development fork choice](docs/FORK_CHOICE_V2.md),
[immutable storage](docs/STORAGE_V2.md),
[append storage](docs/STORAGE_APPEND_V2.md), [explicit recovery](docs/STORAGE_RECOVERY_V2.md),
[replacement metadata/preparation](docs/STORAGE_REPLACEMENT_V2.md),
[explicit replacement recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[membership state](docs/MEMBERSHIP_STATE_V2.md),
[diagnostic replay](docs/REPLAY_V2.md).

The roadmap leads toward software that accepts device readings; it does not
initiate mainnet, hardware procurement, real biometric collection or an unresolved
monetary policy. Proposed technical choices must retain documented compatibility
and verified behavior; marked user decisions remain explicit gates.

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
MB-03D in section 21; MB-03E1 in section 22; MB-03E2a in section 23; MB-03E2b in section 24.

Historical suite: 133 workspace tests. B1 added 31 (164 total), B2 added 39 (203),
B3 added 51 (254), B4 added 69 (323), MB-02A added 80 (403), and MB-02B added
59 (462); MB-02C adds 61 (523); MB-03A adds 65 (588); MB-03B adds 70 (658);
MB-03C adds 49 (707); MB-03D adds 28 (735); MB-03E1 adds 52 (787);
MB-03E2a adds 49 (836); MB-03E2b adds 55 (891).
The subprocess-only storage
helper is explicitly ignored and not counted as a passed test. B4 added the first compile-fail doctest; MB-02B adds two consuming-stage
compile-fail cases, bringing documentation tests to three. MB-03D adds three
borrow/type-boundary cases, bringing documentation tests to six. MB-03E1 adds two
replacement-plan borrow cases, bringing documentation tests to eight.
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
current metadata. MB-03E2a now implements separate whole-branch disk replacement,
checked cleanup and clean reconciliation. Explicit interrupted recovery remains E2b.

NOT implemented: biological deduplication, persistent networked V2 membership blocks,
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
| MB-03 | Durable state, synchronization and membership reorg/replay | IN PROGRESS — NEXT MB-03F | Local publication/recovery complete within scope; bounded fresh-peer reconstruction and node integration remain |
| MB-03A | Explicit V2 storage identity, writer ownership and initial publication | VERIFIED_COMPONENT — IMMUTABLE STORE | 528c891; independent pins, complete replay, Linux directory ownership; section 18 |
| MB-03B | Durable accepted-tip updates and failure reconciliation | VERIFIED_COMPONENT — APPEND / CLEAN STATE | ecf5e76; separate MSU2, accepted-child publication, fencing and selected-state resynchronization; section 19 |
| MB-03C | Explicit interrupted-append staging recovery | VERIFIED_COMPONENT — EXPLICIT ONE-CHILD RECOVERY | e830205; independent request, complete replay, explicit completion/retention, bounded retry; section 20 |
| MB-03D | Canonical development fork-choice specification and implementation | VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON | Independent identity, normalized fixed-target score, keep-current ties, accepted common ancestry and whole-state borrowed selection; section 21 records source acceptance |
| MB-03E | Durable membership-aware branch replacement | VERIFIED_COMPONENT — LOCAL REPLACEMENT / RECOVERY | E1/E2a/E2b complete; actual peer/network integration remains MB-03F/04 |
| MB-03E1 | Replacement selection metadata and checked preparation | VERIFIED_COMPONENT — METADATA / IN MEMORY | Separate MSR2, independent current pins, canonical metadata comparison and complete candidate borrow; section 22 |
| MB-03E2 | Replacement store, publication and explicit recovery | VERIFIED_COMPONENT — LOCAL STORE | E2a publication/cleanup and E2b authorized recovery; sections 23–24 |
| MB-03E2a | Bounded durable replacement and clean reconciliation | VERIFIED_COMPONENT — REPLACEMENT / CLEAN STATE | Generation layout, full candidate publication, exact old cleanup, fencing and complete reopen; section 23 |
| MB-03E2b | Explicit interrupted replacement recovery | VERIFIED_COMPONENT — EXPLICIT RECOVERY | Exact old/candidate histories, independent pins, completion/retention and bounded retries; section 24 |
| MB-03F | Bounded fresh-peer V2 reconstruction | READY — NEXT | Versioned bounded transport; independent network selection; full actual history replay; accepted whole-state publication |
| MB-04 | Persistent node, transaction pool, wallet and operator API | TODO | MB-03; interfaces may start earlier |
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

**Next slice — MB-03F (READY):** bounded fresh-peer V2 reconstruction. Define an
explicit versioned protocol/receiver boundary, verify complete candidate manifests/
blocks under independently selected network identity, and publish only accepted
whole histories through current storage. Peer endpoints are claims, and local
storage generations are neither fork scores nor global freshness. Preserve genuine
membership-fork evidence and all current byte/state/work caps. NEXT_SESSION.md gives
the concrete starting slice and rejection/acceptance cases.

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

**Immediate next:** MB-03F bounded fresh-peer V2 reconstruction.
Reuse the new stable-root store and state table. Keep independently retained exact
old/candidate histories, full checkpoints/generations and explicit completion or
retention separate from untrusted residual bytes. Recheck selected history and all
residuals before mutation; keep ownership through final replay and synchronization.
Unknown artifacts and unpinned/corrupt selection remain preserved/rejected. Do not
roll back a selected candidate or weaken ordinary open to guess a repair. Fresh-peer
reconstruction follows. No money-policy or hardware work is needed now.

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

**Next: MB-03F — bounded fresh-peer V2 reconstruction. READY.**

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
