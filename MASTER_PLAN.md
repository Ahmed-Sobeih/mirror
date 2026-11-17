# Mirror master plan — blockchain to biometric enrollment readiness

Roadmap: **MIRROR-PLAN-001** (2026-11-18).
Design checkpoint: **MIRROR-MB01A-001 — design deliverables complete**.
Latest implementation: **MIRROR-MB01B-B1-001 — V2 configuration/bootstrap verified**.
The V2 ledger, runtime membership and human consensus are not implemented yet.

## Start here: current position and next action

**Next task: MB-01B/B2 — signed V2 membership operations, canonical codecs,
ancestry anchors and authorization windows. Status: READY.**

B1 source: `809661b5e6a9338d65c15c45e540289df24fcc99`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44201393943 — SUCCESS.
164 workspace tests; 57 personhood tests rerun in release; independent Python
configuration vectors, legacy synthetic demo and allocation audit passed.
See section 11 for exact scope. B1 completion does not complete MB-01B as a whole.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[NEXT_SESSION.md](NEXT_SESSION.md), [decisions](docs/DECISIONS.md), and
[the-prompt](the-prompt). The active branch is `work/personhood-register-v0.1`;
PR #1 remains open and unmerged. Recheck live refs before acting. Do not restart
from an older copied prompt.

Design: [ledger integration](docs/LEDGER_MEMBERSHIP_V1.md),
[source threat review](docs/THREAT_MODEL.md), [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md),
and [implemented B1 encoding](docs/CONFIG_V2_ENCODING.md).

The original request established this plan and a maintained continuation pointer,
not a mainnet launch, hardware purchase, real biometric collection, or a change to
unresolved monetary policy. The work packages below are proposed engineering
steps implementing the accepted direction. Their technical choices still require
review, tests and, where identified, a user decision.

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
passed run 44194994506. B1's actual new runtime evidence is in section 11.

The historical suite had 133 workspace tests (26 personhood and 5 monetary-policy
included). B1 adds 31 tests, bringing the workspace to 164. Release reruns are NOT
additional unique tests. Documentation-test command passes with zero doctest cases.

Already demonstrated: development signed transfers, account state, fixed easy
PoW, persisted blocks, restart, TCP synchronization and competing-branch reorg.
Already implemented separately: signed pending/active/voluntarily-revoked
membership, distinct active approvals, exact request/key/evidence-reuse rejection,
atomic event batches, population/activation counters, a deterministic root, and
bounded event-log replay. See [library scope](docs/PERSONHOOD_V0_1.md).
B1 adds validated configuration and proof-checked bootstrap in separate V2 modules.

NOT implemented: biological deduplication, live membership-bearing blocks,
production reviewer assignment, human-round consensus, grants/fees/rewards,
anonymous membership proofs, biometric intake, or real-device integration.
The current prototype's public keys and approval relationships are linkable.

Preserve the historical public-key Alice 100-MRY development fixture and local
`mirror-data`. It is NOT a founder allocation. New network versions use separate
explicit configuration and data directories; no silent genesis reset or migration.

## 3. Fixed direction and decisions still needed

Accepted direction comes from [DECISIONS.md](docs/DECISIONS.md), not from an
implementation convenience: no extra founder allocation or privileged marketing
reserve; first 1,000 qualifying humans receive 10 MRY each including the founder;
0.2 MRY ordinary allowance per qualifying historical population maximum;
1 MRY = 10,000,000 Nusa; founder-led initial verification with no permanent special
powers; offline Ant Kiss with fingerprint/ECG inputs; hardware-independent
interfaces; human-limited participation with easy PoW subordinate to it.

| Gate | Unresolved choice | Blocks which work? |
|---|---|---|
| D-01 | Bootstrap exception as additive allowance versus temporary floor; maturation and qualifying population; false-admission correction | Enabling mint/grant accounting in MB-07, not membership integration |
| D-02 | Approval/review threshold and assignment; founder-led startup transition; appeals, recovery, suspension and revocation authority | Real admission in MB-05/MB-10; synthetic configurations remain explicitly experimental |
| D-03 | Consensus fault assumptions, participant set/selection, quorum/locking protocol, epoch changes and easy-PoW cost | Claiming human-based finality in MB-06 |
| D-04 | Reward eligibility/amounts, fees, commission calculation, funding when issuance capacity is exhausted | Paying protocol rewards in MB-07 |
| D-05 | Evidence custody, authorized search, private matching design, retention, deletion, consent and recovery | Processing real biometrics in MB-09/MB-12 |
| D-06 | Supported capture profiles, matching algorithms/versions, minimum quality, error thresholds, exceptions | Actual uniqueness acceptance in MB-12 |

Prepare concrete options when the relevant task is reached. Do not ask the user
to settle all of these before membership integration. Do not treat proposed defaults
as accepted policy or use permissive fallback behavior to pass a gate.

## 4. Progress tracker

Statuses: VERIFIED_COMPONENT, READY, TODO, IN_PROGRESS, BLOCKED, VERIFIED.
Only mark VERIFIED with deliverables, acceptance evidence and publication refs.
For a design-only task this means design acceptance, NOT runtime implementation.
A passing old suite is not proof that a newly planned task is implemented.

| ID | Work package | Status | Dependencies / exit gate |
|---|---|---|---|
| MB-00 | Existing chain and standalone personhood baseline | VERIFIED_COMPONENT | Evidence above; integration remains open |
| MB-01A | Integration design, threat review and test specification | VERIFIED — DESIGN ONLY | MIRROR-MB01A-001, section 10 |
| MB-01B | Harden the standalone registry against the agreed model | IN_PROGRESS — B1 VERIFIED; NEXT B2 | B2–B4 remain |
| MB-01B/B1 | Canonical config, seed context/proofs and bounded rules | VERIFIED | 809661b, CI 44201393943, section 11 |
| MB-01B/B2 | Signed membership operations, sequences, anchors/windows | READY — NEXT | B1; stateful sequence enforcement completes in B3 |
| MB-01B/B3 | Bounded staged execution, expiry, parent eligibility/activation | TODO | B2 |
| MB-01B/B4 | Full registry codec/root/replay and adversarial regression | TODO | B3 |
| MB-02 | Versioned membership operations in block execution | TODO | MB-01A/B |
| MB-03 | Durable state, synchronization and membership reorg/replay | TODO | MB-02 |
| MB-04 | Persistent node, transaction pool, wallet and operator API | TODO | MB-03; some interface work can run earlier |
| MB-05 | Ant Kiss enrollment lifecycle and bootstrap handoff | TODO | MB-02/04; D-02 for real admission |
| MB-06 | Human-limited rounds, certificates and easy-PoW participation | TODO | MB-03/04/05; D-03 |
| MB-07 | Bootstrap grants, population allowance, rewards and fees | TODO — POLICY-GATED | MB-05/06; D-01/D-04 |
| MB-08 | Device-neutral input contract and capture adapters | TODO | Draft contract exists; implementation can start after MB-01A |
| MB-09 | Protected evidence storage and duplicate-checking pipeline | TODO — PRIVACY-GATED | MB-05/08; D-05 |
| MB-10 | End-to-end verifier/application workflow | TODO | MB-04/05/08/09 |
| MB-11 | Hardening, deployment and Gate A acceptance | TODO | All MB-01 through MB-10 gates satisfied |
| MB-12 | Real device integration and controlled pilot | TODO — DEVICE-GATED | Gate A, compatible devices, D-05/D-06 |

Default execution order is MB-01A, MB-01B, MB-02 onward. MB-08 interface work can
be brought forward so device compatibility is known before purchases. If a policy
decision blocks one package, record it and continue another dependency-ready task.
Do not purchase devices or introduce live biometric collection merely to unblock CI.

## 5. Work packages and acceptance tests

### MB-01A — specify the integration before changing consensus

**Delivered:** [docs/LEDGER_MEMBERSHIP_V1.md](docs/LEDGER_MEMBERSHIP_V1.md),
[docs/THREAT_MODEL.md](docs/THREAT_MODEL.md), and
[docs/MEMBERSHIP_TEST_MATRIX.md](docs/MEMBERSHIP_TEST_MATRIX.md).
These are specifications, not deployed implementations.

Inspect current personhood, core encodings, account execution, chain acceptance,
network transport and storage, including outstanding PR #1 comments. Specify:
network/genesis/configuration binding; typed operations and signatures; ordering;
applicant/verifier nonces and replay scopes; exact block/state commitments;
resource bounds; bootstrap seed commitment; and when membership changes become
eligible for consensus or monetary accounting. Review pending-request spam,
stale evidence, cross-configuration approval reuse, withdrawal ordering and
replay assumptions. Reference the separate device-input draft rather than putting
biometric formats into the block body.

Choose a documented, versioned development-network integration path that preserves
the old MRB1 history. No automatic conversion of known test keys into real founder
credentials. Identify required policy choices without inventing them.

**Pass:** old/new version compatibility table, transition table, failure-atomicity
requirements and adversarial test cases reviewed against actual APIs. Every open
security assumption is listed; no implementation claim is made from the document.

### MB-01B — harden the registry foundation

**Deliver:** targeted fixes and regression tests from MB-01A in `mirror-personhood`.
Keep fresh-evidence biological deduplication explicitly external. Add resource
limits for application/pending state, bounded batch execution, approved evidence
freshness/context rules and deterministic errors. Any changes to signed bytes or
roots require a version and test-vector update, not silent reinterpretation.

Slices: B1 configuration/seed binding and bounded rules (VERIFIED); B2 signed
sequences/anchors/windows (NEXT); B3 staged expiry/activation and execution limits;
B4 canonical registry vectors, regression tests, replay/demo and debug/release checks.
Preserve the V1 implementation while introducing explicitly versioned V2 modules.

**Pass:** duplicate/forged/stale/out-of-order approvals, mismatched networks or seed
contexts, invalid withdrawals, overflow and oversized inputs fail without state
mutation. Activation and counters are deterministic; cancellation/recovery cannot
create a second grant opportunity. Recovery behavior is tested only once specified.

### MB-02 — membership becomes blockchain state

**Deliver:** versioned enrollment/approval/lifecycle transactions; canonical codecs;
atomic execution over account state plus membership state; a block commitment to
all consensus-relevant state and counters. Connect through core/state/chain,
without a second independent authoritative membership database.

Validation must not require a node to rerun floating-point biometric matching,
contact a central server, or decrypt an archive. It validates authorized,
versioned review evidence under the chosen admission rules. A matching receipt is
not a magical proof: its issuer set and trust model must be explicit.

**Pass:** a block carrying enrollment is accepted by two independent validators;
changing an approval, state root, issuer policy, counter, order or network context
is rejected. Failed mixed batches change neither money nor membership. Full replay
from the trusted genesis/configuration reproduces all roots. Legacy data is intact.

### MB-03 — persistence, synchronization and reorganizations

**Deliver:** one coherent durable commit/recovery boundary for blocks, accounts,
membership, counter indexes and eventual grant history; single-writer locking;
versioned migrations; corruption detection; bounded block/snapshot reads. Resolve
ambiguous I/O failures before continuing to write. Existing directory-swap tests
are a starting point, not evidence of universal crash safety.

**Pass:** kill/restart at each publication boundary yields an identifiable complete
old or new state, never mixed accounts/membership. A second writer is rejected.
A fresh peer downloads membership history and independently reconstructs it.
Test forks containing DIFFERENT enrollments/withdrawals, not only timestamps.
Orphaned approvals and population peaks do not survive as canonical state.

The population record is derived from canonical history; it cannot be raised by
any fork ever observed. Normal membership decline does not reduce it. Later,
MB-06 finality must prohibit reorganization of finalized history; legacy PoW reorg
support must not bypass finality certificates.

### MB-04 — usable, persistent network and wallet operations

**Deliver:** long-running node service; separate explicit initialize/status/run
operations; bounded transaction pool; reconnect/multi-peer synchronization; small
operator CLI/API for enrollment submission, status, transfers and diagnostics.
Use fresh random wallet keys, protected local storage, backup/recovery procedures
and distinct signer/transport/member roles. Development keys stay confined to demos.

**Pass:** at least three separate processes/directories share accepted transactions
and membership updates; restart/rejoin converges; duplicate transactions and
overspending do not enter canonical state. Read-only status does not mine or modify
the chain. Malformed frames, slow partial messages, oversized allocations, peer
floods and connection loss stay within documented bounds. Use complete-message
resource/deadline limits, not just a per-read timeout. Sensitive APIs are protected
and not publicly exposed by default; logs contain no secrets or biometric payloads.

### MB-05 — Ant Kiss admission and bootstrap lifecycle

**Deliver:** versioned enrollment sessions, applicant consent/key possession,
verifier eligibility/assignment, signed decisions tied to the exact session,
matching evidence and registry snapshot, and resumable provisional/review/active
outcomes. Implement a bounded review/appeal route and specified lifecycle changes;
unspecified death/inheritance or emergency powers remain disabled, not guessed.

Specify the first founder-led seed process and transition to ordinary distributed
review. The founder has no enduring bypass. Possessing a scanner does not buy more
votes. Define consistent finger positions and required modalities in a versioned
capture profile; no silent fingerprint-only approval when ECG is required.

**Pass:** applicant cannot pick all reviewers or reroll assignments freely; repeated
approvals have no extra weight; incomplete/unusable evidence never auto-activates.
Assignment retries do not create permanent gatekeeping. Simultaneous enrollments
using the same underlying person/evidence are serialized or rechecked against an
updated matching population before activation; two stale "no match" results are
not both blindly accepted. Colluding reviewers and fraudulent evidence are
explicit adversaries in the threat model, not assumed impossible.

### MB-06 — human-based consensus, not a fast-hardware race

**Deliver:** a specified and reviewed round-based consensus profile with an
explicit fault/network model, membership snapshot, proposer/validator eligibility,
locked votes, quorum certificates, safe timeouts, membership-set transitions and
restart protection against double-signing. Evaluate established BFT designs before
implementing a new variant [R2]. Equal eligible identities carry equal weight.

An initial bounded pilot may use the whole small active set; rotating committees
are a separately tested scaling change. Do not invent randomness as just a
miner-selectable previous hash. Specify anti-grinding selection, key-enrollment
cutoffs and committee-capture assumptions before enabling randomized committees.

Bind any easy PoW to identity, challenge, height/round and purpose; additional
nonces or devices do not create votes or rewards. Benchmark the cost on intended
ordinary hardware; never promise literal zero latency. State honestly that easy
work cannot rescue admission security after an attacker controls many identities.

**Pass:** simulations cover equivocation, stale votes, partitions, offline proposers,
delayed/reordered messages, clock disturbances, restarts and membership changes.
No conflicting finality under the stated fault bound. Missing quorum stalls or
changes round; a timeout NEVER permits a reduced safety quorum. Ordinary devices
can participate under measured timing assumptions. Finalized checkpoints are
verified by syncing nodes. A founder-only bootstrap has no decentralization claim.

### MB-07 — issuance and payment for useful participation

**Deliver:** after D-01/D-04, deterministic testnet monetary execution using integer
Nusa, no founder-specific path and no marketing premine. Specify the combined
bootstrap/ordinary ceiling, first-1,000 eligibility, qualifying population timing,
issuance pacing, grant claim uniqueness, fees, reward roles and budget exhaustion.

Keep money already issued, remaining authorized issuance and transferred fees
separate. A fixed proportion is an allowance, not a payout every block. Network
adoption and successful service are different inputs. Do not pay merely for
approving applicants or for generating self-transfers. Enrollment errors must not
silently unlock permanent monetary rights before qualification.

**Pass:** founder and member 1,000 have identical 10-MRY grant rules; member 1,001
gets no bootstrap grant; replays, recovered/rotated keys, revoked identities and
forks cannot double-claim. Population 1M -> 900k -> 2M produces ordinary allowance
200k -> 200k -> 400k MRY, with the separately approved exception applied exactly.
No premature reserve spending, counter overflow, reward above budget or fee double
counting. Exhausted issuance and zero-fee blocks have explicitly tested behavior.

No real grants are enabled until admission and qualifying-population gates pass.
Policy-blocked monetary work must not stop unrelated membership or adapter tests.

### MB-08 — device-neutral capture contract and adapters

**Deliver:** stabilize [DEVICE_INPUT_CONTRACT.md](docs/DEVICE_INPUT_CONTRACT.md);
implement a local adapter SDK/interface, capability negotiation, synthetic capture
source, documented file importer and adapter conformance tests. Isolate vendor
SDKs/parsers from consensus. No arbitrary "universal device" support claim.

Inputs must carry the fingerprint positions/format/quality metadata and ECG
waveform/timing/calibration metadata required by the capture profile. Do not treat
heart-rate numbers or a local fingerprint-unlock boolean as enrollment evidence.
The draft describes requirements, not a chosen manufacturer or guaranteed API.

**Pass:** at least two independently encoded synthetic sources normalize correctly;
wrong units, swapped modalities, truncated/oversized samples, unsupported profiles,
NaN/invalid readings and unplug/retry fail explicitly. File import is labelled
imported, not evidence of a witnessed live capture. Later real adapters can change
without changing monetary or block consensus code.

### MB-09 — protected evidence and private duplicate checking

**Deliver:** an off-chain enrollment service with protected local intake,
authenticated transport, encryption at rest, scoped access, key backup/rotation,
minimal retention and auditable deletion. Define authorized matching operators,
query abuse controls, availability, matching-population synchronization and the
on-chain evidence boundary. Never store archives or decrypting keys on public nodes.

Implement a matching-provider interface with versioned algorithms/capture profiles
and distinct results: NO_MATCH_WITHIN_TESTED_RULES, POSSIBLE_DUPLICATE,
INCONCLUSIVE, ERROR. No exact fingerprint hash masquerading as biometric matching.
Human review is required for uncertain matches [R1]. Specify what joint
fingerprint/ECG results mean; do not assume adding modalities proves uniqueness.

A trusted matcher may exist only as an explicitly labelled development harness.
It does not satisfy decentralized private matching. Before real admission, D-05
must identify who can inspect/compare data, their collusion threshold, and how
another operator can replace them. Evaluate threshold/MPC or other reviewed
approaches; do not invent new cryptography or hide a universal decrypting service.

**Pass:** wrong/revoked credentials cannot retrieve records; replayed or substituted
capture evidence fails session binding; matcher outage cannot produce approval;
receipts bind model/policy, evidence, registry population snapshot and freshness.
Concurrent duplicate tests pass. Canary payloads do not appear in blocks, public
peer traffic, logs, crash reports or CI artifacts. Deletion/backup tests document
what remains as a public membership record and what cannot be erased there.

### MB-10 — complete operator and applicant workflow

**Deliver:** a minimal enrollment interface (CLI or local UI first) showing capture
quality, retry/review status, assigned reviewers, signed decisions and membership
activation. Nodes receive only permitted typed records. Support clean disconnect,
resume and error reporting without manual database editing. Add read-only ledger
and membership status plus transfer/claim operations once authorized by MB-07.

**Pass:** scripted end-to-end flow using synthetic input: new application -> both
readings -> protected comparison -> required approvals -> on-chain activation ->
independent peer sync -> eligible participation -> authorized test payment ->
restart with identical state. Duplicate, inconclusive and failed-capture scenarios
leave membership and issuance unchanged. Wallet keys are never derived from ECG or
fingerprint data. Lost-key recovery cannot create another person or grant.

### MB-11 — hardening, packaging and Gate A release candidate

**Deliver:** bounded-load/fuzz/property tests; dependency/build review; threat-model
review; deterministic multi-node test harness; reproducible build instructions and
release artifacts; operator setup/backup/recovery guides; device integration guide;
a signed-off acceptance report listing residual risks and exact tested profiles.
Target Linux/WSL initially; test any additional target before advertising support.

**Pass:** all preceding software criteria and policy gates complete; full suite,
release-mode tests and integrated scenarios pass at the exact release commit.
Exercise bad peers, malformed enrollment input, storage interruption, network
partition/recovery, full issuance accounting and leaked-evidence checks. Set and
measure capacity/latency/availability targets; do not fabricate throughput claims.
Independent review is required before public-value deployment; unmet review is
recorded, not replaced by a test count. Keep devnet isolated from real assets.

Gate A report must state what real hardware and biometric recognition have NOT
been tested. If private matching or human consensus is still a stub, label the
result partial/integration-scaffold, not software-ready completed.

### MB-12 — actual devices and controlled pilot

**Deliver:** adapter per selected model/firmware/SDK, verified export permissions,
calibration/profile validation, supported-device matrix and approved pilot report.
The user supplies device choices; this plan does not authorize purchases or uploads
of real recordings into this chat/GitHub. Use the compatibility checklist first.

**Pass:** real captures traverse the same pipeline without consensus changes;
repeat enrollment across sessions and devices is tested. Measure false accepts,
false duplicate flags, failure-to-acquire and acquisition time under specified
conditions and population sizes. Include changed recordings, attempted replay,
operator error, new keys/new sessions and cross-device matching. Publish aggregate
results only, not recordings or participant identities. Small-pilot success does
not prove billion-user reliability. Unusable data goes to retry/review, never an
invented unique-human certificate. Any recognition claims remain conditional on
measured D-06 acceptance criteria and human review.

## 6. Final demonstration checklist

Do not mark Gate A/B complete with screenshots alone. Retain reproducible test
commands, machine-readable results and exact build hashes without private data.

- Separate nodes agree on accounts, membership root, activation order, supply and
  eligible participant set after enrollment, transfer, restart and synchronization.
- No approval, transaction, vote or reward can be counted twice for one entitlement.
- The enrollment workflow consumes both required modalities; no hardware is an
  admission authority and no single founder key can bypass mature-network rules.
- Concurrent duplicate attempts and stale matcher results cannot silently activate
  two memberships; errors/uncertainty are visible and reviewable.
- Easy PoW never increases voting/reward weight for faster devices.
- Partition tests preserve specified finality; finalized history is not replaced by
  the legacy longest-chain path. Unfinalized rollback is consistent across state.
- First-1,000 grants and ordinary allowance match the explicitly approved combined
  formula; fees are transfers, not extra supply; there is no hidden owner treasury.
- Biometric archives never enter public blocks, GitHub or logs; intake/matching
  access, retention/deletion and privacy limits are documented and tested.
- Devices satisfy the input contract and supported-device matrix; unsupported
  outputs fail clearly. Gate A uses synthetic inputs; Gate B records real tests.

## 7. Working method and continuation rules

Use small reviewable changes with tests; do not replace whole source files merely
to insert imports. Keep the active branch/PR explicit. Do not merge PR #1 simply
to publish design documents. Verify its state before the next task and preserve
any newer work. Follow pinned CI; add milestone-specific suites as needed.

For each task update this table with: state, acceptance evidence, implementation
commit, CI run, remaining defects, and next exact subtask. Maintain
CURRENT_STATUS.md, NEXT_SESSION.md and the-prompt together at each handoff. Every
Mirror-related reply refreshes the-prompt as requested; discussion-only turns do
not invent implementation progress. Store no secrets, biometric files or runtime
wallet data in repository records. Record failures honestly.

**Immediate next session:** MB-01B/B2, implement signed V2 membership envelopes,
canonical decoding, anchors and windows. Do not repeat B1 or MB-01A. Do not start
reward minting, change legacy genesis or purchase hardware.

## 8. Engineering references, not extra user decisions

Repository facts above come from the cited local checkpoint files and exact GitHub
refs. The task sequencing and acceptance requirements are proposed engineering
work, not capabilities supplied by these references. Reverify version-specific
APIs when implementation begins.

- [R1] NIST SP 800-63A-4, Identity Proofing Requirements:
  https://pages.nist.gov/800-63-4/sp800-63a/ial-general/
  Reference for protection/retention of enrollment evidence and human review of
  one-to-many biometric matches; not a claim of NIST certification or a decision
  to require government identity documents in Mirror.
- [R2] CometBFT consensus specification (primary project source):
  https://github.com/cometbft/cometbft/blob/main/spec/consensus/consensus.md
  Reference for reviewing round/vote/locking/commit and timeout distinctions;
  no library or protocol adoption is decided by this plan.
- [R3] WFDB signal-header specification:
  https://wfdb.io/spec/header-files.html
  Reference for explicit waveform sampling, units and calibration metadata;
  WFDB is a candidate import format, not mandatory device hardware or a biometric
  uniqueness algorithm. Mirror must not silently guess absent calibration.

## 9. Deferred beyond this endpoint

Social media/messenger/dApp execution, manufacturing a custom watch, mainnet value
launch, inheritance automation, full anonymity, global committee scalability and
post-quantum migration are not prerequisites for the first supported-device pilot.
Their hooks/privacy implications must not be ignored, but they must not expand the
current milestone without a recorded scope decision. Full anonymity remains a
goal, not a property of this linkable pilot register. Public operation requires
additional security, privacy and applicable legal review before real enrollment.

## 10. Historical MB-01A acceptance — MIRROR-MB01A-001

Source inspection at `db2861930cc0f01f735ccfba80a4f530f4d1ddd6` covered the
personhood implementation/tests/codec, transaction and block formats, account
executor, chain acceptance, storage, network transport and protocol, plus PR #1
comments. Threat findings F01–F11 distinguished observed code from proposed remedies.
Delivered compatibility/transition tables, config/seed design, signed layouts,
sequences/anchors/expiry, parent eligibility/block-end activation, resource budgets,
combined commitments and atomic replay requirements. T01–T35 were planned then.

Independent arithmetic checked prefix 131, header 160, empty block 168 and operation
lengths 243/291/227/227. No Rust or economic change was made in that design turn.
MB-01A VERIFIED means design deliverables, not implementation or security certification.

## 11. B1 acceptance — MIRROR-MB01B-B1-001

Source: `809661b5e6a9338d65c15c45e540289df24fcc99`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44201393943
Actual log inspected: job 134456099484. All steps passed on Rust 1.98.0.

Implemented `crates/mirror-personhood/src/v2/{config,encoding,limits}.rs` and
31 public-API tests in `crates/mirror-personhood/tests/config_v2.rs`. Canonical MCF2
unsigned configuration binds network, versions, PoW/timestamp, threshold, evidence
profile, all 12 limits, ordered seed descriptors and strictly sorted synthetic
allocations. Checked sums reject overflow and duplicate zero-balance addresses.
Context precedes seed signatures/IDs; every seed proof is strictly checked.
ConfigV2 is not a VerifiedBootstrap until all proofs pass; no founder bypass.

Independent standard-library Python reference generates full bytes/context/seed
hashes and IDs, compared to committed vectors and Rust results. Decoders enforce
count/byte caps before input-sized allocation, reject malformed/truncated/trailing
inputs, and rerun constructor checks. Max-count and one-beyond cases pass.

Results: 164 workspace tests (133 retained + 31 new); 57 personhood tests in release
(26 V1 + 31 B1); reference-vector check; legacy synthetic enrollment/replay demo;
documentation command with zero cases; allocation/runtime-data audit. Rust tests
ran in actual GitHub Actions, not in the local chat container (no Rust toolchain).

Matrix coverage: T02 verified for configuration/bootstrap binding; T03 construction/
proof parts and T20 configuration/count parts verified. Runtime membership privilege,
state execution/resource enforcement, operation expiry and block integration remain
B2/B3/MB-02. Do not report all 35 cases as passed. F01 is addressed only at the V2
config/bootstrap boundary; legacy V1 behavior and remaining findings are unchanged.

No node, genesis, wallet, monetary-policy source, Cargo dependencies/lockfile or
local runtime data changed. No real enrollment, grants or biometrics. PR remains
unmerged. Next B2, then B3/B4, then MB-02. Gate A and Gate B remain UNMET.
