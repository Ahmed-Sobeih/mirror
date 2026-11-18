# Mirror master plan — blockchain to biometric enrollment readiness

Roadmap: **MIRROR-PLAN-001** (established 2026-11-18; checkpoint 2026-11-19).
Design checkpoint: **MIRROR-MB01A-001 — design deliverables complete**.
Latest implementation: **MIRROR-MB01B-B4-001 — membership codec/root/replay VERIFIED**.
MB-01B is **VERIFIED_COMPONENT**: B1 configuration, B2 messages, B3 bounded execution,
B4 canonical state commitment, untrusted snapshots and trusted grouped replay.
V2 ledger/blocks, durable membership synchronization and human consensus remain unfinished.

## Start here: current position and next action

**Next task: MB-02A — TransferV2, unified operation envelope, BlockV2 codec and
commitments with independent vectors. Status: READY.**

B4 published source: `21e2272b5ac92c00e840d7aaa26a99570a5ec24f`.
Source tree: `31dfafa463935c640cd5cb31b88588575fc06d4b`.
Local Rust 1.98.0 validation: 323 workspace tests; 216 personhood release reruns;
one compile-fail doctest; three references, three synthetic demos and audit PASS.
Published tree matches the locally tested source. Exact-source push CI 44404337870
passed with job 135155424668 full logs inspected; PR CI 44404351436 passed too.
See section 14 for scoped evidence. Neither Gate A/B nor MB-02 is complete.
Checkpoint documentation follows that source; check its later exact-head CI separately.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[NEXT_SESSION.md](NEXT_SESSION.md), [decisions](docs/DECISIONS.md), and
[the-prompt](the-prompt). The active branch is `work/personhood-register-v0.1`;
PR #1 remains open and unmerged. Recheck live refs before acting. Do not restart
from an older copied prompt.

Design: [ledger integration](docs/LEDGER_MEMBERSHIP_V1.md),
[source threat review](docs/THREAT_MODEL.md), [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md),
[configuration encoding](docs/CONFIG_V2_ENCODING.md),
[signed operations](docs/OPERATIONS_V2_ENCODING.md),
[state/execution layout](docs/MEMBERSHIP_STATE_V2.md), and
[grouped replay](docs/REPLAY_V2.md).

The original request established this plan and a maintained continuation pointer,
not a mainnet launch, hardware purchase, real biometric collection, or a change to
unresolved monetary policy. The work packages below are engineering steps toward
the accepted direction. Technical choices require review/tests and, where marked,
a user decision. No proposed default silently becomes production policy.

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
passed run 44194994506. B1/B2/B3/B4 evidence is in sections 11/12/13/14.

Historical suite: 133 workspace tests. B1 added 31 (164 total), B2 added 39 (203),
B3 added 51 (254), and B4 added 69 (323). B4 also adds one compile-fail doctest.
Release reruns are NOT additional unique tests. Both device-readiness gates remain UNMET.

Already demonstrated: development signed transfers, account state, fixed easy
PoW, persisted blocks, restart, TCP synchronization and competing-branch reorg.
Already implemented separately: signed pending/active/voluntarily-revoked
membership, distinct active approvals, exact request/key/evidence-reuse rejection,
atomic event batches, population/activation counters, a deterministic root, and
bounded event-log replay. See [library scope](docs/PERSONHOOD_V0_1.md).
B1 adds validated configuration/proof-checked bootstrap. B2 adds signed fixed-size
membership messages and contextual authorization checks against a trusted view.
B3 adds bounded immutable V2 execution, private indexes/tombstones, exact sequences,
expiry, parent-eligible approvals, end-of-block activation and counters. B4 adds
canonical V2 roots, an untrusted snapshot wrapper and bounded replay against trusted
per-height/checkpoint expectations. These components are not yet live V2 blockchain membership.

NOT implemented: biological deduplication, live membership-bearing blocks,
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

Statuses: VERIFIED_COMPONENT, READY, TODO, IN_PROGRESS, BLOCKED, VERIFIED.
Only mark VERIFIED with deliverables, acceptance evidence and publication refs.
For design-only work this means design acceptance, NOT runtime implementation.

| ID | Work package | Status | Dependencies / exit gate |
|---|---|---|---|
| MB-00 | Existing chain and standalone personhood baseline | VERIFIED_COMPONENT | Integration remains open |
| MB-01A | Integration design, threat review and test specification | VERIFIED — DESIGN ONLY | Section 10 |
| MB-01B | Harden registry foundation | VERIFIED_COMPONENT — B1/B2/B3/B4 | Live ledger integration remains MB-02 |
| MB-01B/B1 | Canonical config, seed proofs and bounded rules | VERIFIED | 809661b, CI 44201393943, section 11 |
| MB-01B/B2 | Signed membership messages, codecs, ancestry/windows | VERIFIED — MESSAGE LAYER | af1dd66, CI 44205064789, section 12 |
| MB-01B/B3 | Bounded staged execution, sequences, expiry, eligibility/activation | VERIFIED — IN MEMORY | b6f1233, local suite and CI 44317174000 PASS, section 13 |
| MB-01B/B4 | Full registry codec/root/replay and adversarial regression | VERIFIED — COMPONENT | 21e2272, CI 44404337870, section 14 |
| MB-02 | Versioned membership operations in block execution | READY — NEXT MB-02A | MB-01A/B complete; integration not yet implemented |
| MB-02A | TransferV2, unified envelope, BlockV2 codecs/commitments | READY — NEXT | Independent byte/hash vectors; preserve prior encodings |
| MB-02B | Atomic accounts + membership with full mixed-operation indices | TODO | MB-02A; no-fee/no-mint development semantics |
| MB-02C | Pinned V2 genesis, actual block acceptance and replay | TODO | MB-02B; trusted ancestry/root/PoW binding |
| MB-03 | Durable state, synchronization and membership reorg/replay | TODO | MB-02 |
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
mixed-operation positions. The diagnostic digest is not the future MRB2 Merkle root.

### MB-02 — membership becomes blockchain state

**Deliver:** versioned membership and transfer operations, canonical codecs, atomic
accounts+membership execution, and one commitment to all consensus state/counters.
Connect core/state/chain without a second authoritative side membership database.

Implement in three bounded slices:

- **MB-02A (NEXT):** TransferV2 kind 0 (243 signed bytes), unified operation envelope,
  160-byte header, bounded MRB2 codec (168-byte empty block), domain-separated block
  hash/count-bound Merkle operation commitment and independent reference vectors.
  Preserve membership signed bytes; avoid a core/personhood dependency cycle.
- **MB-02B:** stage accounts and membership together, preserve original full-block
  operation indices including transfers, checked positive zero-fee/no-mint transfers,
  separate account/member sequences and canonical combined roots. Test whole-ledger
  rollback after a late failure and conservation of initial synthetic supply.
- **MB-02C:** explicit manifest and pinned exact genesis, actual block/header/root/
  parent/configuration/PoW validation, stable candidate ancestry and accepted-block
  replay. Only validated blocks may bind staged membership provenance.

B3/MPH2 currently accept membership operations only. Filtering transfers out of
mixed blocks would change inclusion metadata and potentially activation order.
Specify indexed/full-block staging and version/extend diagnostic replay explicitly,
or replay complete validated MRB2 blocks. Do not reinterpret existing MPH2 bytes.
Structural codecs do not establish ledger validity or canonical-chain authority.

Validation must not rerun floating-point biometrics, query a central matcher, or
decrypt an archive. Validate authorized versioned evidence with explicit issuer
trust rules; a receipt is not mathematical proof of a true biometric claim.

**Pass:** two independent validators accept the same membership-bearing block;
changed approvals, roots, policies, counters, order or context reject. Invalid
mixed batches change neither money nor membership. Replay reproduces roots;
legacy history remains intact. Freeze block/ledger vectors independently.

### MB-03 — persistence, synchronization and reorganizations

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

### MB-10 — complete operator/applicant workflow

**Deliver:** minimal CLI/local UI showing capture quality/retries, assigned reviews,
signed decisions and activation; resumable sessions and clear errors without manual
database edits. Read-only status and transfers/claims only when MB-07 authorizes.

**Pass:** synthetic application -> both readings -> private comparison -> required
approvals -> on-chain activation -> independent peer sync -> eligible participation
-> authorized test payment -> restart with identical state. Duplicate/inconclusive/
failed capture changes neither membership nor issuance. Keys never derive from
biometrics. Lost-key recovery cannot create another person or grant.

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

**Immediate next:** MB-02A TransferV2/unified operations/BlockV2 codecs and commitments
with independent vectors. Reuse verified B1–B4; MB-02B/C then integrate accounts and
actual block authority, MB-03 durable multi-node state. No minting or hardware work now.

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
**Next: MB-02A**, followed by atomic mixed-state execution and actual block authority.
Gate A/B remain UNMET. [NEXT_SESSION.md](NEXT_SESSION.md) contains the exact next slice.
