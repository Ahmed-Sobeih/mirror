# Mirror decisions — resumed development

These records supersede earlier brainstorming, not existing historical block bytes.

## Accepted direction

1. No founder-specific allocation. The founder receives no extra funds or
   permanent administrative authority merely for founding Mirror.
2. First 1,000 qualifying humans: 10 MRY each, INCLUDING the founder on the same
   terms. This is an explicit 10,000-MRY bootstrap allocation.
3. No privileged preallocated marketing/airdrop treasury in the initial design.
   The early proposal of 1.5 billion founder MRY and 2 billion marketing MRY is
   retired. The earlier 12-billion fixed cap is not an active policy requirement.
4. Ordinary issuance allowance is 0.2 MRY per highest qualifying verified-human
   population reached. Population is not wallet/device/online-node count.
   Declines do not force burning/confiscation; returning to the same record does
   not unlock the same allowance again.
5. Nusa are subdivisions: 10,000,000 Nusa per MRY, not an additional supply.
6. Founder-led initial enrollment is allowed as an explicit bootstrap trust
   assumption. It is not a permanent founder veto or master credential.
7. Ant Kiss combines human review with proposed fingerprint/ECG duplicate checks.
   Hardware is interchangeable. The blockchain does not ingest biometric archives.
8. Human-limited participation is the intended final consensus. Easy PoW is
   secondary; faster hardware must not purchase more human slots. Current PoW
   devnet rules are not proof that this goal is implemented.
9. New-issuance rewards and transaction fees may fund useful participation. Rates,
   eligibility, role allocation and fee mechanics are not yet decided.

## Explicitly unresolved — do not silently choose while coding

- Whether the bootstrap exception is additive to ordinary allowance or a temporary
  supply floor; funding verification when the allowance is exhausted.
- Qualifying/mature population rules, false-admission correction, and when an
  enrollment affects real issuance.
- Production verifier assignment, approval quorum, founder bootstrap handoff,
  challenges, forced revocation, death reports, appeals and key recovery.
- Private duplicate-search design, biometric reliability, hardware trust,
  retention/deletion, consent, and enrollment exceptions.
- Human-round consensus, committee selection, randomness, deadlines, quorum
  safety, rewards and migration from the existing PoW chain.
- Privacy-preserving membership proofs. Prototype identities remain linkable.

## User input — flexible devices and minimal capture, 2026-11-26

The user reaffirmed flexible equipment: readings should come from interchangeable
devices in a human-supervised Ant Kiss procedure. Vendor/model-specific adapters
and documented file imports belong off-chain; consensus should consume authorized
enrollment outcomes and permitted commitments/approvals. Human oversight does not
remove the need for usable comparable readings or a shared duplicate-search process.

The user is considering **one anatomical fingerprint plus ECG**, possibly the
**left little finger**, to reduce invasive collection. Record this as a proposed
minimal profile, not a final choice of finger, sample count, threshold or proof
of sufficient uniqueness. No all-fingerprint collection requirement is adopted.
Repeat captures of the same finger can be evaluated for quality. Finger consistency,
reviewed exceptions, cross-profile continuity and joint matching performance remain
to be specified under D-05/D-06. Neither modality's failure implies permission to
create a new identity. Both readings remain protected off-chain.

The revised [device input draft](DEVICE_INPUT_CONTRACT.md) records these boundaries
and primary engineering references. This discussion adds no device support, changes
no consensus or monetary rule, and does not advance implementation past MB-03D.
At this discussion checkpoint, next remained MB-03E; the rough Gate A progress
estimate remained 20–25%.

## User proposal — accessible enrollment, 2026-11-26

The user requested preserving a future exception route for people unable to supply
the standard readings, including a missing designated finger or hands. The proposal
uses an accessibility/disability record with a description, a committee of **three
qualified people**, and deliberate controls on adding, changing or removing it.

[ACCESSIBLE_ENROLLMENT_PROPOSAL.md](ACCESSIBLE_ENROLLMENT_PROPOSAL.md) records the
idea separately from engineering recommendations. Committee size is proposed;
qualifications, assignment, approval threshold, alternatives and appeals remain open.
An accommodation explains the need for another procedure; it does not establish
uniqueness on its own. Prefer a minimal private explanation and ordinary public membership
outcome, with any public indicator subject to explicit privacy review. The user
has not authorized public medical descriptions or selected a replacement biometric.

Later design must preserve identity/claim continuity and allow reviewed corrections
without casual toggles or an irreversible inaccurate label. This is future work
under MB-05/08/09/10/12 and D-02/D-05/D-06. No current API, policy threshold, money
rule or implementation status changes. At this discussion checkpoint, next remained MB-03E.

## User clarification — combined initial enrollment profiles, 2026-11-26

The user favors ECG + one designated pinky fingerprint + human-supervised Ant Kiss
for initial standard enrollment. Where the fingerprint is unavailable, the proposed
combination is ECG + an explanation of the observed physical limitation + human
supervision, retaining the prior proposed exceptional review. The argument concerns
the combination; no single component is expected to establish uniqueness alone.

The user believes these combinations can be adequate for an initial launch and may
consider more fingerprints, even all fingers, later as population grows. Record
that preference accurately without turning the belief into measured readiness.
Exact capture, matching/fusion, committee and launch-acceptance rules remain open.
There is no new ten-finger requirement, automatic upgrade threshold or launch action.

Engineering recommendations: keep limitation details minimal/private; evaluate each
complete route separately against repeat enrollment with new keys, days and reviewers;
preserve cross-profile duplicate scope and membership/claim continuity. Shared
physical descriptions are context, not an independent uniqueness guarantee. Later
upgrades cannot retroactively validate earlier admissions. These refinements are in
[the accessibility proposal](ACCESSIBLE_ENROLLMENT_PROPOSAL.md) and
[device draft v0.3](DEVICE_INPUT_CONTRACT.md). At this discussion checkpoint, next
remained MB-03E; no code/test or progress milestone was added by this discussion.

## Implementation boundaries for this milestone

`monetary_policy` contains separate checked arithmetic components, zero founder
bonus and zero marketing reserve. It has NO mint or claim API and deliberately
NO combined total-supply formula.

`mirror-personhood` is an isolated experimental state machine. The approval
threshold is supplied in explicit development rules and bound into signatures;
this is not an agreed production quorum. Seed members have ordinary rights after
bootstrap. Voluntary self-revocation is the only implemented revocation operation;
no third party or founder can force-revoke another member through this API.

No founder wallet is created, no grants are paid, and no existing database,
genesis block, transaction format or wallet balance is rewritten.

## Planning instruction — MIRROR-PLAN-001

The user requested a repository plan to finish the blockchain/enrollment software
to the point of accepting fingerprint and ECG readings, with interchangeable
devices supplied later, and asked that the-prompt track position and continuation.
[MASTER_PLAN.md](../MASTER_PLAN.md) records the proposed engineering sequence;
[DEVICE_INPUT_CONTRACT.md](DEVICE_INPUT_CONTRACT.md) records draft compatibility
requirements. This instruction does not choose a manufacturer, approve real data
collection, settle the open monetary rules, or assert universal device support.
The roadmap separates synthetic software acceptance from actual-device pilot
acceptance. Its proposed algorithms, thresholds and policies are not user-approved
merely because they appear as tasks. At the original planning handoff, MB-01A was
next; that historical plan added no runtime capability or admission authority.

## Engineering design selection — MIRROR-MB01A-001

Following the user's instruction to continue, MB-01A reviewed current source and
published LEDGER_MEMBERSHIP_V1.md, THREAT_MODEL.md and MEMBERSHIP_TEST_MATRIX.md.
The selected implementation direction is a separate, explicitly initialized
synthetic membership-dev-v2 network with new typed encodings and combined state,
while preserving all legacy history. This is an engineering compatibility choice,
not a new launch or monetary decision. At that design checkpoint the next task was
MB-01B/B1; CURRENT_STATUS.md and NEXT_SESSION.md now track the live implementation.

V2 context binds seed order/contents, configuration and limits; proposed operation
windows, sequences and block-parent eligibility address identified source gaps.
Proposed end-of-block activation/counters and zero-fee/no-mint test transfers are
development semantics only. They do not settle qualifying monetary population,
real approval policy, production fee amounts, forced revocation or human consensus.
No source runtime behavior changed merely because this design was published.

## B3 engineering choices — MIRROR-MB01B-B3-001

The separately versioned synthetic registry now has bounded staged block execution.
Two implementation details were made explicit without changing MCF2 or B2 bytes:
at most 64 retained approvals per record, and at most configured max_records retained
approvals over all records. These independent finite development budgets retain
history even after expiry/withdrawal; exhaustion can stop new admissions and does
not weaken quorum or erase identity reservations. They are not production admission
policy, global scalability claims, or monetary limits.

Registry tip hash is external trusted provenance, excluded from canonical state
to avoid the future block-hash/root cycle. Binding a staged state to a hash is not
block validation. MB-02 must validate actual blocks/roots/genesis before that binding.
The runtime nonce is a signed request salt, not a global biometric identity; exact
ID/key/evidence checks remain the defined synthetic duplicate boundary.

At the B3 checkpoint, MPS2 state bytes were specified in MEMBERSHIP_STATE_V2.md;
the codec/root/replay and independent vectors remained B4 work. No unresolved user
economic, real-admission, privacy, recovery or human-consensus decision was selected
by B3.

## B4 engineering choices — MIRROR-MB01B-B4-001

B4 implements the frozen MPS2 encoding and commitment, a bounded untrusted cache,
and block-grouped MPH2 diagnostic replay. Source
`21e2272b5ac92c00e840d7aaa26a99570a5ec24f` passed
[exact-source CI](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870).
The MB-01B component is complete; it does not establish live V2 ledger authority.

MPH2 has an explicit diagnostic cap of 4,096 groups, including empty blocks, and
1,048,576 cumulative state items during replay. The work charge includes retained
records/approvals, prospective additions, and configuration descriptors/allocations
copied by immutable staging. These are bounded engineering work limits, not new
economic rules or production population limits. Exhaustion rejects the transcript;
it never deletes history, weakens quorum, or publishes a partial result.

The configured log limit remains 4,194,304 bytes by default. MPS2 has its separate
checked state bound, `90 + max_records * (366 + 267)`, or 41,484,378 bytes with
default limits. MCF2/B2 and legacy encodings remain unchanged.

A state root alone does not identify every operation history. Replay therefore
requires an independently trusted digest of each exact ordered operation group,
its block hash and its resulting membership root, plus an external final
configuration/height/tip/root expectation. The group digest is a diagnostic
membership-only commitment, not the future MRB2 operation Merkle root. Future
mixed blocks must preserve original operation positions; dropping transfers to
reuse a membership-only transcript would change semantics.

Decoded snapshots have no conversion into runtime membership state or a registry.
Verified replay still depends on a trusted caller supplying actual history; B4
does not implement the chain, durable tip or consensus that establishes that trust.
At the B4 checkpoint, MB-02A was next. Its subsequent implementation and MB-02B/C's
completed atomic ledger and actual block validation are recorded below. Durable
persistence was left to MB-03; the implemented MB-03A/B slices are recorded below.
No grant, fee, monetary exception formula, private matching, anonymity, real
admission policy or final human-consensus decision was selected by B4.


## MB-02A engineering choices — MIRROR-MB02A-001

Source `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382` implements the previously selected
V2 transfer/unified-operation and MRB2 format contract. Exact-source CI 44428924399
passed; MASTER_PLAN.md section 15 records the historical acceptance evidence.
At that checkpoint MB-02B atomic execution was next. No user economic, admission,
privacy or human-consensus decision changed.

Core depends on personhood's immutable config, membership operations and parent
view. Personhood still depends only on crypto; there is no reverse dependency.
V2 transfer senders retain MIRROR_ADDRESS_V1 derivation for compatibility with
configured synthetic account addresses. Authorization binds the frozen V2 prefix
and context; account sequences and membership sequences remain separate.

The ordered-operation commitment explicitly includes signed bytes and operation
count, eliminating the duplicate-last Merkle ambiguity without changing legacy
roots. All format APIs enforce configured budgets; the decoder completes structural
preflight before allocating the operation vector. The exact bytes and validation
boundaries are frozen in CORE_V2_ENCODING.md with independent reference fixtures.

Positive amount/zero fee are static development transfer checks only. A signed
transfer spends nothing until the ledger executes it. A block body with valid
signatures/commitment is still untrusted with respect to balances, sequences,
ledger root, ancestry, work/time, genesis and acceptance. Existing membership-only
MPH2 remains unchanged and rejects transfers. Mixed execution must preserve original
positions in the complete block rather than filtering transfers out. D-01/D-04,
all grant/reward implementation and real biometric/device work remain open.


## MB-02B engineering choices — MIRROR-MB02B-001

Source `420d7114ddf782469a6ace6ba0a2c4076fb93aaf` implements the selected atomic
accounts/membership development ledger. Exact-source push CI 44619155972 and PR
CI 44619172378 passed; MASTER_PLAN.md section 16 records the historical inspected
acceptance evidence. At that checkpoint MB-02C actual block/genesis/work/root
authority was next. These are implementation choices under the existing design,
not new user economic or admission decisions.

Full mixed-operation order is preserved through a consuming indexed membership
builder. Every omitted transfer is charged at its frozen 243-byte signed size plus
four-byte prefix. Runtime MPS2 enrollment indices are now bounded by max_operations
rather than the membership count cap. This explicit accepted-range extension changes
no serialized field/domain and grants no snapshot authority. MPH2 remains unchanged
and membership-only; actual mixed replay must use complete validated MRB2 bodies.

V2 retains every explicit zero account allocation and every spent account so consumed
sequences and capacity history persist. Transfers use exact account sequences,
positive amounts and zero fees; funded self-transfers advance one sequence without
changing balances. Membership sequence/status changes do not imply monetary grants
or new spending restrictions. Both components stage together and conserve initial
synthetic supply. Account/economics/combined roots are frozen independently in
LEDGER_STATE_V2.md; current block hash remains external trusted provenance.

No founder allocation, marketing reserve, bootstrap payout, population mint, fee or
reward formula was enabled. D-01/D-04 remain OPEN. Synthetic configured balances
are not a new currency policy. Private matching, real admission, anonymity, device
support and final equal-human consensus remain later gates. A staged ledger and a
caller-supplied hash do not establish that any actual blockchain block is valid.


## MB-02C engineering choices — MIRROR-MB02C-001

Source `db2cb58ba5a9a828ec949debd1b52b5915a66f3b` implements pinned actual V2 block
acceptance and complete bounded MRB2 replay. Exact-source push CI 44623282448 and
PR CI 44623296178 passed; CURRENT_STATUS.md records inspected evidence. MB-02 is
complete within its in-memory component scope. At that checkpoint MB-03A storage
identity/locking/recovery was next; its implementation is recorded below.
These are compatibility and implementation choices, not new
user economic, real-admission, privacy or human-consensus decisions.

MNM2 carries canonical MCF2, all ordered seed proofs and the selected exact genesis
hash. Decode requires an independently selected C/genesis pair; manifest bytes do
not choose local trust. Actual genesis still must independently validate. The
immutable accepted branch supplies operation ancestry. Full original mixed bodies
execute atomically and the resulting ledger root must match before actual-hash
binding/publication. Replay requires external C/genesis/height/tip/ledger-root
expectations; no cache conversion or membership-only history substitutes for it.

This finite synthetic component explicitly caps history at 4,096 blocks, 16 MiB
complete MRB2 and 1,048,576 cumulative abstract state/configuration items. The
exact checked charge formula is documented in CHAIN_V2.md. Exhaustion rejects;
there is no history eviction, hidden limit increase or production capacity claim.
Changes require reviewed profile/version semantics. These are not monetary or
human population caps. Existing B4 diagnostic limits remain separate.

Child timestamp is strictly greater than its parent, with exact configured genesis
time and no local clock/future ceiling. Thus an accepted u64::MAX timestamp cannot
have a successor. This is a deterministic development choice, not a final timing
or liveness policy. Actual work uses exact configured compact bits and the existing
V2 domain-separated header hash as a big-endian integer; accepted positive compact
forms are not silently redefined. The mining helper's one-million-attempt cap is
local, not a new consensus quota or equal-human reward rule.

A valid finite branch is not globally canonical or final. At the MB-02C checkpoint,
durable selected-tip publication, locking, restart/latestness authority, transport
and membership-aware reorganization remained MB-03/04; human consensus remained
MB-06. No founder/marketing
reserve, grant/mint, nonzero fee, reward distribution, biometric archive, real device
support or reliable biological deduplication was introduced. D-01/D-04 remain OPEN.


## MB-03A engineering choices — MIRROR-MB03A-001

Source [`528c8912c5a50cfd2e86856d345b29d6bb79d2c5`](https://github.com/Ahmed-Sobeih/mirror/commit/528c8912c5a50cfd2e86856d345b29d6bb79d2c5)
implements immutable selected-chain storage and checked reopening. Source tree:
`65004e73f4c19deedafdb8cf2874939df0ab367a`. Local verification passed 588 workspace
tests, including 65 new cases, with one ignored subprocess helper; three retained
compile-fail doctests, all seven independent references and all seven examples
also passed. Release reruns passed core 121, personhood 234 plus the same three
doctests, state 49, consensus 23, chain 57 and storage 75 tests. These reruns are
not additional unique tests. Exact-source CI and publication evidence are recorded
separately in CURRENT_STATUS.md and MASTER_PLAN.md; local success is not a CI claim.

The [storage contract](STORAGE_V2.md) chooses a separate explicit `StoreV2` lifecycle:
initialize an absent or empty directory from an already accepted `ChainV2`, reopen
an existing selection with exclusive ownership, or inspect it with shared read-only
ownership. Reopening requires both an independently selected C/genesis identity
and the matching full C/genesis/height/tip/ledger-root checkpoint. It retains exact
MNM2 manifests and complete MRB2 blocks and reconstructs accounts and membership
through normal whole-chain replay. No snapshot, stored counter, matching checksum
or peer-supplied selection becomes runtime authority.

MSS2 is a new fixed 232-byte storage record with version 2, profile 1 and generation
zero only. Its manifest and record checksums detect byte corruption; they do not
authenticate selection or discover freshness. Existing configuration, operation,
block and state bytes/domains remain unchanged. The finite 4,096-block, 16-MiB MRB2
and 1,048,576 cumulative state/configuration budgets are preserved. Disk metadata
and all file lengths are bounded before payload allocation; the maximum successful
encoded payload is 19,559,084 bytes, separately from runtime memory overhead.

This initial profile is Linux-only and uses nonblocking operating-system locks on
the unchanged root directory inode for each handle's lifetime. Cooperating
processes, a trusted parent tree/local owner and the documented filesystem locking,
rename and synchronization behavior are explicit assumptions. No lock-file PID
lease, stale takeover, hostile-owner race defense or verification of WSL/DrvFS is
implied. Initialization synchronizes prepared files and directories before the
selected marker is published. An uncertain publication returns no writable handle;
reconciliation requires retained independent pins. Partial stores are neither
silently repaired nor deleted, and legacy stores are never migrated automatically.

The immutable profile has no append, tip replacement, automatic repair, global
fork-choice or V2 node-daemon API. At the MB-03A checkpoint, MB-03B accepted-tip
updates and failure reconciliation were next; that separate implementation is
recorded below. The immutable profile remains frozen. Later integration cannot
recover capacity by discarding retained history or restoring an untrusted cache.

Only internal workspace dependencies were added; no external dependency changed.
All inputs remain public synthetic test material. No founder allocation, marketing
reserve, bootstrap payout, population mint, fee/reward formula, real biometric
archive, device support, anonymity or biological uniqueness was introduced.
Accepted user policies above are unchanged; D-01/D-04 remain OPEN and Gate A/B
remain unmet. MB-03A is a bounded storage component, not completion of MB-03 or a
universal crash-safety or mainnet-security guarantee.

## MB-03B engineering choices — MIRROR-MB03B-001

Published implementation:
[`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c`](https://github.com/Ahmed-Sobeih/mirror/commit/ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c).
Source tree: `06d7c3f4fa24d96c844e2301c76c01f804151192`, identical to the locally
tested tree. The source increment changes 14 files, with 3,568 insertions and
11 deletions. Local verification passed
658 workspace tests (588 retained + 70 new), with one explicitly ignored subprocess
helper, and three retained compile-fail doctests. The new cases are 28 selector,
31 append integration, eight private fault/reconciliation and three process tests.
Release reruns passed core 121, personhood 234 plus the same three doctests,
state 49, consensus 23, chain 57 and storage 145 tests. All eight independent
references and eight examples passed. Exact-source
[push CI 44649111203](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203),
job `135999222947`, and
[PR CI 44649119266](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
both **SUCCEEDED**. The full push-job log confirmed checkout
`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c` and all workspace, documentation,
release, reference, example and audit gates. This establishes **MB-03B
VERIFIED_COMPONENT — bounded append and clean-state reconciliation**.
Subsequent documentation-head CI must be checked separately; no result is inferred
from this source run. The prior MB-03A acceptance record above remains historical evidence.

The [append storage contract](STORAGE_APPEND_V2.md) adds a separate `AppendStoreV2`
lifecycle while preserving immutable `StoreV2` and every existing wire encoding.
MSU2 is a 280-byte envelope with version 2/profile 2, an outer local generation,
the exact 232-byte MSS2 snapshot, and a new domain-separated integrity checksum.
The nested MSS2 remains version 2/profile 1/generation zero. Initialization in an
absent or empty directory starts the outer generation at zero, including when the
input is an already accepted multi-block history. No existing profile is silently
adopted or migrated. Each subsequent accepted child advances height and generation
once, preserving the existing chain budgets.

The independently supplied `AppendCheckpointV2` selects both a complete chain
checkpoint and its local generation. It is not learned from an untrusted marker,
peer, pending file or largest observed height. Checksums and generation ordering
provide neither global freshness nor fork choice. All reopened accounts, membership,
approval history and resource counters come from complete actual-chain replay.

One-child append validates and stages the complete block before filesystem writes,
replays the current disk selection under retained pins, synchronizes the new MRB2
file and block directory, and writes/synchronizes the next MSU2 marker. It rechecks
the old selector before publishing the new marker by same-root rename, then
synchronizes the stable root before advancing memory. Directory ownership remains
continuous; the root is never swapped or removed.

Disk discrepancies and update I/O failures fence further writes and ordinary
chain/selection access. `last_confirmed_checkpoint` identifies only the last
successful memory publication. Errors after a publication attempt disclose that
disk may already select the candidate. Consuming `reconcile` retains the same
lock and returns an unfenced writer only after complete replay and resynchronization
of an independently selected clean history. A failed reconciliation returns no
usable handle. Writer reopening also resynchronizes; shared inspection never writes
or authorizes an append.

This slice deliberately has **strict clean-state reconciliation**: pending markers,
extra tail blocks, partial files and unknown artifacts reject and remain in place.
No data is deleted, pending candidate promoted, or generation selected merely to
make recovery succeed. **At the MB-03B checkpoint, next was MB-03C: explicit
interrupted-append staging recovery**, followed by reviewed canonical branch
selection, durable membership reorganization and peer reconstruction. MB-03
remained in progress at that checkpoint; MB-04's live
V2 node/API and MB-06 human finality remain later work.

The independent append reference derives new MSU2 bytes from pinned prior
chain/storage fixtures, including successive mixed/empty blocks and a distinct
membership fork. These synthetic checks do not establish reliable biometrics,
anonymity, final consensus, universal crash safety or device readiness. No monetary
policy, grant/mint API, nonzero fees, rewards, founder privilege or marketing reserve
is introduced. D-01/D-04 remain open, and Gate A/B remain unmet.

## MB-03C engineering choices — MIRROR-MB03C-001

Published implementation:
[`e8302052ab724f59ff4a212dc64aea9575a6d4b3`](https://github.com/Ahmed-Sobeih/mirror/commit/e8302052ab724f59ff4a212dc64aea9575a6d4b3).
Source tree: `dc9eb2c02436f0952ac54593ba8d02317e9d9314`, identical to the locally
tested tree. The source increment changes nine files, with 3,138 insertions and
one deletion. Local verification passed 707 workspace tests (658 retained + 49
new), with one explicitly ignored subprocess helper, and three retained
compile-fail doctests. The new tests are 39 recovery integration tests, nine
private fault/retry tests and one process-lock test. Release reruns passed core
121, personhood 234 plus the same three doctests, state 49, consensus 23, chain 57
and storage 194. All eight independent references and nine examples passed.
Exact-source
[push CI 44748718086](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748718086),
job `136330677456`, and
[PR CI 44748724632](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748724632)
both **SUCCEEDED**. The complete 216,857-byte push-job log confirmed checkout
`e8302052ab724f59ff4a212dc64aea9575a6d4b3`, the workspace/documentation/release
counts above, all eight references, all nine examples and the allocation audit.
This establishes **MB-03C VERIFIED_COMPONENT — bounded explicit interrupted-append
recovery**. Later documentation-head CI must be checked separately; no result is
inferred from these source runs.

The [explicit recovery contract](STORAGE_RECOVERY_V2.md) adds narrowly authorized
recovery to profile 2 without changing any existing encoding, checksum domain,
ordinary opening rule, dependency or independently frozen fixture. Current source,
local verification and exact-commit CI are recorded in CURRENT_STATUS.md and
master-plan section 20; the MB-03B evidence above remains historical.

The caller retains a parent checkpoint, candidate checkpoint and exact complete
child independently, then explicitly selects Complete or RetainParent. The two
checkpoints must describe one bounded child transition. Recovery replays the
complete parent through ChainV2, accepts the exact requested child and matches its
full resulting checkpoint before either action can mutate data. Signatures, state
roots, work and all resource counters retain their existing acceptance rules.

No on-disk journal or implicit discovery is introduced. A prefix match does not
prove who created an artifact, especially an empty file. The trusted caller's
specific retained request authorizes the two named paths; byte comparisons only
constrain that authority. The selected marker must match the exact canonical
parent or candidate. Only the single next-height block and candidate pending
selector may be absent or exact canonical prefixes under the documented state
table. Unknown bytes, unsupported combinations and unsafe paths reject unchanged.

Complete extends only a missing suffix, synchronizes prerequisites and prepared
files, compares the old marker/artifacts, publishes by the existing same-root
rename, then performs strict clean replay and synchronization. An already selected
clean candidate is idempotent success without another generation. RetainParent
works only while the parent is still selected: remove the authorized pending file,
synchronize the root, then remove the authorized unselected child and synchronize
its directory. It never rolls back a selected candidate or deletes accepted history.

Standalone recovery acquires exclusive ownership; consuming recovery keeps the
same existing writer lock. Both return a usable writer only after complete final
replay and synchronization; errors return no handle. Failure before publication is
reported as incomplete recovery. Once publication may have occurred, or an already
selected candidate cannot be reconciled, the outcome is explicitly uncertain.
Retry always revalidates under the same independent request, with no automatic
retry loop, higher-generation guess or arbitrary cleanup.

The recognized encoded-input maximum is 19,559,412 bytes, including the full
intended child and both selectors. A separate conservative 156,508,096-byte
per-call allowance charges payload reads and growth sentinels, including repeated
comparisons and final replay. Neither value is a resident-memory or write-volume
limit. Existing 4,096-block, 16-MiB MRB2 and cumulative state/configuration limits
remain unchanged. Cooperative Linux ownership and trusted local owners/parents
remain explicit assumptions; these checks are not a power-loss certification.

Missing/corrupt selected markers, corrupt accepted history, unexplained artifacts,
multiple residual children and interrupted initialization remain outside recovery.
The caller's request has no persistence or operator-selection UI in this slice.
The new recovery example uses only public fixtures and newly owned temporary paths;
all eight prior reference checks remain required and the example count becomes nine.

**At the MB-03C checkpoint, next was MB-03D: canonical development fork choice**,
before durable membership-aware branch replacement and peer reconstruction.
MB-03 remains in progress; the persistent V2 node/API and final human consensus
remain subsequent work. No monetary formula, mint/grant/nonzero fee/reward API,
founder privilege, marketing reserve, private biometric matching or actual-device
support is added. D-01/D-04 remain open and Gate A/B remain unmet.

## MB-03D engineering choices — MIRROR-MB03D-001

Source: `b9a2e870996fcafe1128dcb5cfdfb6b01b158ded`.
Source tree: `dd0a3fc29f429dcd4999261801c2bc7897d6aace`.
Focused verification passed 28 new integration cases, three new compile-fail
ownership/type cases and the synthetic `fork_choice_v2_demo` example.
Full checkpoint verification:

Published-source [push CI 45703714223](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703714223)
and [PR CI 45703726306](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703726306)
both passed. Full push-job `139520118282` logs confirmed the exact implementation
checkout, **735 workspace tests, six compile-fail doctests, all required release
suites, eight independent references, ten examples and the allocation/runtime-data
audit**. The single intentionally ignored subprocess helper is invoked by parent
tests and is not another pass. Release reruns are not additional unique tests.
MB-03D is **VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON**. Subsequent
documentation-head CI is checked separately and recorded on PR #1.

The [fork-choice contract](FORK_CHOICE_V2.md) adds pure comparison of two complete,
already accepted `ChainV2` values. The caller independently selects C and exact
genesis. The comparator checks current C, current genesis, candidate C and
candidate genesis in that order before scoring or ancestry work. Reported peer
height, arbitrary work scores, block proposals, caches and local storage markers
cannot substitute for accepted input types.

The fixed-target development profile uses accepted post-genesis block count as
its normalized score: `score = height`, bounded to 0..=4095. Every accepted block
within one selected identity has the same literal configured target. If its
positive work contribution is W, comparing `(height + 1) * W` is exactly comparing
heights. The implementation therefore needs no accumulated big integer, unchecked
work multiplication, target reinterpretation or new serialized score.

Only a strictly higher candidate score is preferred. Equal scores keep the input
designated as current. This makes no global tie-breaking or finality claim: two
peers can retain different equal-score branches until a strictly stronger accepted
branch appears, and a finite cap or terminal timestamp may prevent that. Lower
individual hash, nonce, timestamp, account value, membership/population counts and
storage generation contribute no extra weight. This is not the future equal-human
consensus or a reward rule.

Actual accepted hashes are compared forward to the first mismatch. The result
records the last common accepted block's full checkpoint and distinguishes identical
histories, either proper-prefix relationship and divergent branches. It borrows
both inputs immutably and exposes the complete selected chain by reference. All
accounts, membership, permanent reservations, approvals and resource counters stay
with their accepted history. No values are combined across branches and no state
is mutated, cloned into a new authority or published to storage.

The comparison performs at most 4,096 hash pairs / 8,192 fixed-header hashes,
uses constant additional state and allocates no collection or replayed ledger.
Its fields are private, and it has no public constructor or encoded certificate.
Copied enum/checkpoint data remains diagnostic and may become stale after the
borrowing result is no longer used. Future durable publication must revalidate
identity, current store selection and exact candidate under exclusive ownership.

Existing block/configuration/state/storage bytes, all independent references,
finite per-branch budgets and user policy remain unchanged. The example uses
synthetic frozen membership-divergent histories and performs no network or disk
publication. This slice adds no dependency, new hash domain or encoded format.

**At the MB-03D checkpoint, MB-03E was next:** durable membership-aware branch
replacement with reviewed publication, interruption and recovery rules, followed
by fresh-peer reconstruction.
It must retain the stable locked root and cannot treat a local MSU2 generation as
fork weight. MB-03 remains in progress; persistent V2 node/API and human finality
remain later work. No monetary formula, mint/grant/nonzero fee/reward API, founder
privilege, marketing reserve, biometric intake, private matching or anonymity is
introduced. D-01/D-04 stay open; Gate A and Gate B remain unmet.

## MB-03E1 engineering choices — MIRROR-MB03E1-001

The [replacement contract](STORAGE_REPLACEMENT_V2.md) freezes bounded selection
metadata and checked in-memory preparation before implementing disk replacement.
MSR2 is a separate 280-byte version-2/profile-3 envelope around the unchanged
232-byte MSS2 snapshot. It uses a distinct checksum domain and independent full
endpoint/generation expectations. Existing MSS2/MSU2 bytes, store APIs, recovery
behavior and reference fixtures remain unchanged; no implicit migration is added.

Local replacement generation starts at zero at any accepted height. Each strictly
preferred candidate would advance it once, even for a multi-block extension, while
height advances by at least one. The profile therefore requires generation no
greater than height, bounded to 4095. Generation is neither a branch score nor
proof of publication. Equal-height or future human-weighted selection requires an
explicitly reviewed profile instead of silently changing this meaning.

`prepare_replacement_v2` reuses actual accepted-branch comparison under independent
network identity, checks the retained current checkpoint/generation, then derives
and compares the entire canonical current selector. Endpoint equality alone cannot
authorize altered manifest metadata or block-byte accounting. Only a strictly
preferred accepted candidate yields a plan and canonical next-generation metadata.
The plan borrows both complete chains and keeps the candidate's exact manifest,
accounts, membership and counters together. It has no public constructor,
serialization or mutation path and performs no filesystem I/O.

This increment deliberately did not freeze an on-disk intent, directory layout,
publication linearization point or cleanup/recovery procedure. **At the E1
checkpoint, MB-03E2 was next:** those rules must be designed and implemented
together, retaining stable exclusive root ownership and revalidating current
selection plus the exact candidate before mutation. The E1 plan is not a reusable
disk-write permit or evidence of crash safety.

Focused verification passed 31 codec and 21 preparation tests; all nine reference
checks passed. Two compile-fail examples cover the borrowed input histories. Full
acceptance and publication/CI evidence belongs to
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md),
section 22, separately from the historical MB-03D source record above.

These are storage engineering choices. All accepted user decisions and unresolved
admission, accessibility, monetary and human-consensus policies remain unchanged.
No founder or marketing privilege, issuance/reward rule, device requirement or
biometric capture is introduced. MB-03, the persistent V2 node/API and Gate A/B
remain unfinished.

## MB-03E2a engineering choices — MIRROR-MB03E2A-001

The [replacement store contract](STORAGE_REPLACEMENT_STORE_V2.md) adds actual
whole-branch publication, independently pinned reopening, write fencing and strict
clean-state reconciliation to the separate profile-3 lifecycle. The existing MSR2
bytes, MSS2/MSU2 stores, MNM2/MRB2 encodings, previous recovery API, dependencies and
independent reference fixtures remain unchanged. No persisted intent format or
implicit migration is introduced. Historical E2a implementation, acceptance and
exact source/CI evidence remains in [MASTER_PLAN.md](../MASTER_PLAN.md), section 23;
[CURRENT_STATUS.md](../CURRENT_STATUS.md) tracks the latest increment separately.

The chosen layout uses canonical `generation-{generation:020}` directories under
one stable locked root. A clean store has exactly one selected generation and
`selection.v2`. During a replacement only the exact current and next generations
may coexist, with one optional `selection.pending`; there is no retained archive
of every generation. Generation remains local publication bookkeeping, starts at
zero at any accepted height and increments once for a strictly preferred complete
candidate. Directory names, checksums and the numerically largest generation do
not choose authority, mining weight, latestness or global finality.

The writer first validates independent network/current pins and complete accepted
branch preference, then replays and compares its exact selected disk history under
continuous ownership. It stages the candidate's own complete manifest and MRB2
history, synchronizes files and directories, rechecks prepared bytes and publishes
the MSR2 selection by same-root rename. Root synchronization follows before any old
file is retired. Both the selected candidate and the complete old directory are
checked before cleanup; only named byte-matching old files and now-empty prescribed
directories may be removed. No recursive deletion or selected-history pruning is
used. Final clean replay and synchronization precede the single in-memory advance
of the complete ledger, selector and generation.

Disk discrepancies and mutation failures fence state access and further writes.
The last-confirmed checkpoint describes memory only after failure; the disk may
already select the candidate. A failure once publication may have been attempted,
including cleanup failure, reports uncertain completion. Consuming reconciliation
holds the same exclusive root descriptor throughout and returns a writer only
after full replay and synchronization of a clean independently selected history.
Ordinary open, inspect and reconcile preserve and reject every pending selector,
second generation, partial cleanup or unknown artifact. They are not repair APIs.

Each accepted history keeps the existing 4096-block, 16-MiB complete MRB2 and
1,048,576 cumulative state/configuration-item limits. The two-generation layout
has a conservative distinct encoded payload ceiling of **39,118,264 bytes**.
Every permitted payload read, including a growth sentinel and repeated comparisons,
is charged against one conservative **313,011,680-byte per-call allowance**.
Enumeration and per-file limits remain separate. These bounds are not resident
memory, elapsed-time or physical power-loss guarantees. Cooperative Linux locking
and trusted local owners/parent paths remain explicit assumptions.

**At the E2a checkpoint, MB-03E2b was next: explicit recovery of interrupted
replacement staging and cleanup.** E2a recorded the state table but implemented
no completion/retention recovery entry point. The following E2b record now tracks
that implementation. Recovery must independently retain or reconstruct
**both exact accepted old and candidate histories**, their full checkpoints and
local generations, plus the explicit action. An old endpoint and candidate alone
cannot authenticate partially removed old history. Authorized residue may contain
an arbitrary subset of expected paths; file synchronization alone does not make
all directory entries durable. Every surviving byte must be checked against the
specific retained authority before mutation. A selected new generation cannot be
rolled back with a retain-old action. Missing/corrupt selection and interrupted
initialization remain separate unsupported cases.

The new synthetic `replacement_store_v2_demo` performs a real replacement from the
accepted fork at height 3/generation 0 to main height 4/generation 1, then opens and
reconstructs the entire selected state. This advances local storage only; it does
not finish MB-03 or provide fresh-peer reconstruction, a persistent V2 node, final
human consensus or an end-to-end enrollment flow. Gate A and Gate B remain unmet.
All accepted monetary and enrollment/accessibility decisions are preserved. No
founder or marketing privilege, grant/mint/fee/reward formula, biometric capture or
device requirement is added.

## MB-03E2b engineering choices — MIRROR-MB03E2B-001

The separate [replacement recovery contract](STORAGE_REPLACEMENT_RECOVERY_V2.md)
implements explicit completion or retention for recognized interrupted profile-3
replacement layouts. Ordinary open, inspect and clean reconciliation remain strict;
they never call recovery automatically. MSS2, MSU2, MSR2, MNM2 and MRB2 bytes,
existing store APIs, dependencies and independent reference fixtures remain frozen.
This increment introduces no journal, persisted intent, implicit migration or
directory-scanning authority.

`ReplacementRecoveryRequestV2` borrows **both complete accepted old and candidate
histories** and retains their full independently chosen checkpoints and adjacent
local generations. Construction checks the endpoints and strict branch preference;
the explicit recovery operation also checks the caller's independent network
identity. A partly removed old directory cannot reconstruct its own missing
authority. Matching byte prefixes constrain authorized content without proving who
created it. Independently retained histories and an explicit `Complete` or
`RetainOld` action authorize the prescribed paths.

Before changing a file, recovery validates the selected complete history and every
remaining permitted artifact. While old remains selected, candidate entries may
be arbitrary subsets with canonical byte prefixes, including empty files or absent
manifest entries. A pending selector, even an empty prefix, requires a complete
candidate. `Complete` finishes the authorized prefixes without truncating or
recreating existing files, synchronizes and validates the candidate before selection,
then performs checked old-copy cleanup. `RetainOld` removes only authorized staging
while preserving the fully validated selected old chain. Repeated allowed actions
are explicit and idempotent; generation still advances once per preferred branch.

After candidate selection, that complete history must remain present and no pending
selector is accepted. Recovery may finish removing arbitrary subsets of exact full
old files. If an old blocks directory survives, its exact old manifest must still
exist; later supported shapes are manifest-only, empty old generation or absent
old generation. Partial old files and unsupported cleanup ordering are rejected
unchanged. `RetainOld` cannot roll back an already selected candidate.

New-lock and consuming-writer recovery both preserve the stable root inode.
Consuming recovery keeps the same exclusive descriptor throughout and requires
the retained memory checkpoint to be one of the request's endpoints. A complete
final replay and synchronization precede returning an unfenced writer. Any error
returns no handle; interrupted recovery can be explicitly retried against the same
independent authority. Unknown artifacts, missing/corrupt selected markers and
interrupted initialization remain preserved and unsupported. Cooperative Linux
locking, trusted local owners/parents and finite per-history/payload-read limits
remain assumptions and boundaries, not physical power-loss certification.

Focused public verification passed **38 integration tests** and the new synthetic
`replacement_recovery_v2_demo`. The scenarios include genuine personhood forks
with identical account bytes, sparse staging and cleanup, canonical metadata,
wrong pins/history/network, complete state reconstruction, unsafe paths, strict
ordinary reopening, fenced consuming recovery and both explicit actions. Multiple
residual shapes inside a test are scenarios rather than additional tests. Full
local acceptance subsequently passed **891 runtime tests, eight retained doctests,
all required release suites, nine independent references and thirteen examples**,
including the corrected absent-old-blocks-directory synchronization ordering.
Exact source, private fault/retry, process-ownership and published CI evidence are
tracked in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and the
[master plan](../MASTER_PLAN.md), separately from the focused result above.

**At the E2b checkpoint, next was MB-03F: fresh-peer reconstruction**, then persistent
V2 node/operator integration under MB-04. E2b completes a bounded local recovery component, not
MB-03 as a whole or Gate A/B. It changes no monetary, grant, reward, fee, founder,
marketing, biometric, accessibility or human-consensus policy. No device support,
biological uniqueness, anonymity, public launch or real-money readiness is claimed.

## MB-03F1 engineering choices — MIRROR-MB03F1-001

MB-03F was split into a checked transfer/reconstruction component (F1), followed by
actual bounded peer-session and durable-adoption integration (F2). F1 implements
the separate [MPN2 framing and candidate receiver](PEER_HISTORY_V2.md). This split
alone did not mark network reconstruction or MB-03 as a whole complete. Verification
and exact source/publication evidence belong to CURRENT_STATUS and master section 25.

The new protocol uses its own magic, version and Begin/Manifest/Block/End kinds.
It does not reinterpret or negotiate fallback to the existing MIRR/version-1
messages. Begin and End carry the same bounded endpoint/count/byte claims. Those
claims describe the promised finite transfer; they do not independently choose a
network, canonical branch, current tip or finality. No local storage generation is
transmitted as a score or freshness signal. A valid shorter or losing branch can
still be a valid received candidate.

The caller supplies configuration/genesis identity independently. The receiver
uses NetworkManifestV2::decode_for, then actual ChainV2::from_genesis and append
for every complete original MRB2 block. It does not put peer claims into the
trusted ChainReplayExpectationV2 interface. Final tip/root/count/bytes must match
execution, and End must repeat the originally captured claim. Only finish exposes
the complete accepted candidate; later comparison and owned storage publication
remain separate operations. Mixed transaction and membership ordering is preserved.

Receiving ownership is consuming. Every rejected header, payload or complete
frame drops the attempt and its private partial chain. A pending-frame owner exposes
only a payload length already checked against kind, phase, configured limits and
remaining transfer budget, so a future transport can check before allocating.
There is no public partial-chain getter or clone of the continuation. End is the
explicit transfer boundary; further input to that receiver rejects. No EOF-based
termination, socket, deadline, retry scanning or persistent session is added in F1.
The sender borrows an immutable accepted ChainV2 and emits one bounded frame at a
time; after an encoding error it cannot emit a later terminal frame.

F1 preserves the existing 4,096-block, 16-MiB MRB2 and 1,048,576-state-item chain
limits. Its complete encoded transfer ceiling is 19,641,112 bytes in at most
4,099 frames, including both summaries, every header and height prefix. Genesis
has its exact empty-block size. The decoded configuration further restricts child
sizes, and every block header admission reserves the minimum/maximum space for
all remaining blocks. Bounds are per transfer, not resident-memory, wall-clock or
global connection limits. At this checkpoint, F2 was assigned session ownership,
deadlines, I/O failure handling and explicit accepted-branch adoption without
weakening these checks; its later completion is recorded below.

Two existing workspace crates, mirror-chain and mirror-core, become mirror-network
runtime dependencies. Existing mirror-consensus and mirror-personhood are test-only
dependencies for meaningful invalid-work/signature/ancestry and configured-limit
cases. No third-party dependency version changes. The old V1 code, earlier V2
formats/hash domains, monetary policy and stored history remain unchanged.
No enrollment rule, device adapter, biometric storage, founder privilege or reward
formula is selected by this networking component.

## MB-03F2 engineering choices — MIRROR-MB03F2-001

Implementation source:
[`851193ea6428198441e6cdf29361b1885fae157b`](https://github.com/Ahmed-Sobeih/mirror/commit/851193ea6428198441e6cdf29361b1885fae157b).
Tested tree: `cf2e4bb9d29b250a000bb4680d1571e0355d1126`.
Parent: `f47b4306b13c4d079859472f2fdd919debd83eda`.
Exact-source CI and any later documentation-head results belong to
[CURRENT_STATUS.md](../CURRENT_STATUS.md), separately from the local results below.

F2 implements [finite V2 TCP sessions](PEER_SESSION_V2.md) and
[complete-candidate adoption](CANDIDATE_ADOPTION_V2.md), composed with the unchanged
F1 receiver, accepted-history comparator and owned replacement/recovery APIs.
MB-03F2 is **VERIFIED_COMPONENT — FINITE TCP / DURABLE ADOPTION**;
MB-03F is **VERIFIED_COMPONENT — BOUNDED FRESH-PEER RECONSTRUCTION**; and MB-03 is
**VERIFIED_COMPONENT — V2 STORAGE / EXPLICIT RECOVERY / FINITE PEER SYNCHRONIZATION**.
These statuses identify the implemented finite component boundary. Persistent
restart authority, daemon lifecycle, global scheduling, reconnect/multi-peer policy,
pool, wallet and operator APIs remain MB-04. Gate A and Gate B remain unmet.

### Dedicated request and terminal socket boundary

One exact 72-byte MPQ2 request carries version 2, full-history request kind 1 and
independently selected configuration/genesis hashes. It carries no height, local
generation, signed checkpoint or personhood claim. The provider independently
checks both its accepted history and the request against its locally selected
identity before responding. MPQ2 is a separate preamble; frozen MPN2 F1 frames,
V1 MIRR bytes/APIs and legacy node commands remain unchanged.

The requester sends the one request and shuts down its write half. The provider
requires request EOF before it streams one immutable accepted history. The
requester must accept every unchanged F1 frame, including matching End, and then
observe response EOF before `finish` releases the candidate. One trailing byte
rejects in either direction. EOF alone cannot complete an unfinished history;
End without EOF cannot complete this dedicated socket session. All errors close
the owned socket and discard the receiver/pending owner. No scanning, resumption,
fallback, multiplexing, DNS or automatic reconnect is introduced.

### Finite owner budgets and absolute deadline

`PeerSessionsV2` is a non-Clone synchronous owner. Its validated configured bounds
permit 1..=8 attempts, positive cumulative actual read/write bytes up to
157,129,488, a positive per-attempt duration through 60 seconds, and at most one
connected/accepted stream at a time. The byte ceiling is
`8 * (19,641,112 + 72 + 2)`; all F1 per-transfer and chain limits still apply.
Every real positive read/write result is charged, including malformed input,
partial output and rejected trailing bytes. Failure never refunds consumed budget.
EOF is zero bytes, but its one-byte rejection probe needs one unspent byte of
capacity before reading. Buffer sizes cannot exceed the remaining allowance.

The deadline begins before connection establishment or accept waiting. Every
individual read/write syscall uses the original deadline's remaining time, and
post-syscall plus semantic-processing checks reject late completion. Short reads,
partial writes and Interrupted errors cannot renew the deadline. Fixed F1 headers
are checked through consuming admission, then the complete admitted payload must
fit the remaining owner allowance before allocation or reading.

Synchronous decoding/execution is not preempted. The bounds establish the stated
I/O and acceptance behavior, not a hard CPU-return deadline, resident-memory cap,
kernel-buffer cap, bandwidth guarantee or availability promise. Creating another
owner creates another explicit budget; this component is not a global process
admission policy or the future persistent node's scheduler. A provider success
means its local writes and half-close succeeded, not remote validation or adoption.

### Whole-candidate selection, publication and retained recovery

Adoption requires a completed actually executed candidate, owned writer,
independent C/genesis and the separately retained full current checkpoint plus
local generation. Preparation checks those inputs and canonical current metadata
without filesystem I/O. Tied, shorter and identical candidates keep the unchanged
store and do not increment generation or perform publication filesystem calls.

Only strict preference creates retained recovery intent. Before publication it
clones the exact old accepted history once, moves the received candidate without
cloning, and retains the independent identity and both full local checkpoints.
The candidate checkpoint is derived from accepted state and the next local
publication generation. The provider transmits no generation, and peer endpoint
claims never become independent current pins.

Publication calls the unchanged `ReplacementStoreV2::replace`, which repeats actual
disk/identity/preference validation under the held lock. The selected candidate
replaces the whole history: accounts, membership, permanent reservations, approvals,
sequences, population peaks and resource counters stay together, including on forks
whose account bytes match. The new local generation advances once per publication,
even across several heights.

Pure preparation rejection returns the unchanged owned inputs. Publication failure
privately retains the same store lock and both exact histories, exposing no normal
writer. Its last-confirmed checkpoint describes prior successful memory only;
the disk may already select the candidate. Explicit consuming recovery uses the
same lock and existing `Complete`/`RetainOld` rules. Recovery failure returns no
handle while preserving root and exact inputs; a later explicit retry acquires
fresh ownership and revalidates the same authorization. There is no implicit
retry, cleanup or promotion. A selected candidate can never be retained back to old.

Fresh initialization remains a separately requested operation with its own
independent network check. Failed opening never authorizes initialization,
overwriting existing data or storage-profile migration. The additional old-chain
clone and store replays are not covered by encoded-input memory claims. Adoption
runs after transport acceptance and is outside the socket session deadline.

**F2 recovery intent remains in memory only.** It is neither a persisted journal nor
an automatic history archive. A process restart cannot recover missing authority
from a higher generation, peer claim or damaged residue. Preserving independent
node selection and both exact histories across restart was assigned to the next
increment; A1's separate persistent wrapper is recorded below. The F2 API itself
retains its original memory-only recovery boundary.

### Verification and unchanged policy

Required local gates with Rust 1.98.0 and locked dependencies passed:
**1,010 workspace runtime tests** (951 retained plus 59 new), **10 doctests**,
all required release suites, **11 independent references** and **15 synthetic
examples**, including `peer_session_v2_demo`. Release runs and multiple scenarios
within a test are not additional unique tests. The real-process demonstration and
tests cover complete fresh reconstruction, fragmentation, premature closure,
stall/deadline rejection, oversized headers, changed End, invalid actual blocks,
equal-account membership ties/preferred forks, durable publication and strict
reopen. Adoption tests retain explicit recovery ownership and reject unknown or
corrupt residue. These are executed software scenarios, not physical power-loss
certification or an external security audit. Local results do not assert a CI run
or future documentation head has passed.

The only new runtime workspace dependency is mirror-storage in mirror-network;
no external dependency version changes. Existing F1/V1 protocols, prior V2 byte
formats/hash domains, storage/recovery implementations, monetary code and historical
fixtures are preserved. No founder or marketing privilege, grant, reward, fee,
biometric/accessibility requirement, admission rule or human-consensus policy is
selected by this work.

## Historical F2 continuation plan — MB-04A1

**At the F2 checkpoint, next was MB-04A1 — bounded persistent node control state
and restart recovery authority.** The plan below records that earlier boundary;
the completed A1 engineering record follows it. This was a technical continuation
plan, not a new user monetary or enrollment-policy decision. MB-04A2 operator
commands, daemon/reconnect/multi-peer scheduling, pool and wallet remained later work.

Specify and implement a separate bounded, versioned operator-owned control layer
outside the strict selected-store root. It must retain independently chosen
C/genesis, the confirmed complete selected checkpoint/generation, and at most one
pending replacement intent containing **both exact complete accepted old/candidate
histories and both local checkpoints**. Loading intent must bound input before
allocation, replay both histories and validate their exact authorized relationship.
Integrity checksums do not establish authority against a hostile owner who can
replace all local files. Define deterministic control/store ownership and lock order.

The required ordering is: persist, validate and synchronize exact intent before
any store mutation; invoke existing owned publication; persist and synchronize
confirmed current pins only after successful publication or explicit recovery;
retire intent only after those pins are durable. This must cover a restart after
candidate storage selection but before the node's confirmed pin was updated.

Startup/status must leave missing or invalid control state unchanged and clearly
distinguish uninitialized, invalid and recovery-required states. No highest-generation
scan, silent fallback initialization, automatic completion/retention or inferred
authorization is permitted. A separate
explicit recovery action selects Complete or RetainOld under the recorded exact
inputs; candidate selection still prohibits rollback. Initially reject/preserve
partial control initialization or partial intent unless a separately specified and
tested control-layer recovery rule authorizes that case.

A1 acceptance must exercise separate-process clean restart; read-only status for
missing/clean/recovery-required states; all independent pins; failure before durable
intent with no store mutation; each intent/store/confirmed-pin/retirement boundary;
both exact histories available after restart; explicit completion and retention;
selected-candidate no-rollback; competing ownership; overlong/corrupt/unsafe control
inputs; and the genuine equal-account/different-membership fork. Preserve frozen
storage/wire profiles rather than adding unrecognized files inside an existing
store. New control encoding and its reference evidence must be documented before
claiming acceptance. These are planned gates, not tests already passed by F2.

MB-04A2 then adds a small V2 init/status/sync-once/recover operator surface. Status
must never initialize, mine, synchronize peers or repair; sync-once must use F2's
existing bounded attempts and the A1 intent ordering. Do not bundle a long-running
daemon, transaction pool, signer/wallet or unresolved monetary/admission decision
into A1 merely to call it a node. No real keys, personal records, device captures or
hardware purchase are needed for this next software increment.


## MB-04A1 engineering choices — MIRROR-MB04A1-001

Implementation source:
[`32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0`](https://github.com/Ahmed-Sobeih/mirror/commit/32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0).
Tested source tree: `d79080968d822a2267ce5f1b1196efcf16d2e546`.
Parent checkpoint: `37010bc5c5827f5a8dd886af0e79d079ebfe1d4e`.
[Source PR CI 45861912935](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45861912935)
passed the configured gates; full job `140054826270` logs were inspected.
Its synthetic PR checkout `62437dc1c338156fb984349b791efc7a4ad34ebb` has no file differences from the
implementation source. Source acceptance and any later documentation-head CI
remain separate records in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and PR #1.

A1 implements [durable V2 node control and restart authority](NODE_CONTROL_V2.md):
**VERIFIED_COMPONENT — DURABLE V2 CONTROL / EXPLICIT RESTART RECOVERY**.
This is a finite library component and process harness. MB-04 remains in progress;
operator commands, persistent serving/reconnect/multi-peer scheduling, transaction
pool and wallet are not completed by this increment. MB-03/F2 remain verified in
their earlier scope. Gate A and Gate B remain unmet.

### Separate local authority and canonical records

`NodeControlV2` owns a trusted operator control directory outside the strict
profile-3 store root. The operator supplies both intended paths and independent
C/genesis. The control record retains the full selected chain checkpoint and local
generation for restart; a peer, checksum or higher directory never chooses them.
Control payloads contain public synthetic chain history, not private keys or raw or
encrypted biometric archives. Replacing every local file is outside the trusted
owner/parent-directory model; checksums do not authenticate a hostile owner.

MNC2 is a new exact 464-byte version-2/kind-1 local record: full checkpoint and
unchanged canonical MSR2, with a separate integrity checksum. MNI2 has a 952-byte
header containing both records and two explicit lengths, followed by both complete
unchanged MPN2 histories and a checksum. Maximum encoded intent is 39,283,208 bytes.
Its decoder binds the externally selected network identity, executes both whole histories,
checks exact canonical metadata and repeats existing strict preference and next-
generation preparation. No cached state or advertised root replaces replay.
The new checksum domains and offsets are frozen in the linked contract; existing
consensus, wire and storage domains are preserved.

The confirmed record's complete MSR2 metadata is compared with the actual replayed
store, including manifest/block-byte metadata, not only endpoint/generation fields.
A valid intent admits only its exact old or exact candidate confirmed record.
Identical, shorter and tied histories keep current; only a preferred complete
history creates intent. Balances, memberships, approvals, sequences, population
history and resource counters stay with one accepted branch.

### Ownership, durable ordering and explicit restart

Acquire cooperative Linux control ownership before storage ownership and release
storage first. The stable control/store root inodes are not replaced. Managed
payloads must be bounded regular single-link files under checked non-symlink paths.
The ready node's storage writer remains private; callers cannot extract it to bypass
intent persistence. New control initialization is explicit and requires an already
initialized store under independently retained full pins. Failed opening never
becomes implicit initialization or repair.

Preferred adoption consumes its ready owner and received candidate. It clones the
exact old accepted history once, moves the candidate, writes/synchronizes bounded
`intent.pending`, reads it back and replays both histories, then publishes `intent.v2`
and synchronizes the control directory. Exact layout, confirmed record and intent
are checked again before existing owned storage replacement. The candidate is then
confirmed through synchronized `confirmed.pending` publication and confirmed-file/
directory synchronization. Only afterward is `intent.v2` removed and that removal
synchronized. No new ordinary storage staging layout or recovery permission is added.

Startup is clean-only. Read-only status returns Missing, Clean, RecoveryRequired or
an error without creation, synchronization, peer contact or repair. RecoveryRequired
reports validated local intent and does not claim the store currently selects a
particular history. Missing, invalid or unknown state is never resolved by a scan
for the highest generation.

Explicit recovery reloads/replays both histories and resynchronizes exact intent,
control root and parent before any mutating storage recovery. Confirmed-old plus
complete intent permits the existing Complete/RetainOld rules against the actual
recognized store layout. RetainOld never rolls back a selected candidate.

Every `intent.pending`, incomplete control initialization, and unknown/corrupt/
overlong/unsafe artifact is preserved and rejected. A `confirmed.pending` prefix
of zero through 464 exact candidate bytes is recognized only with confirmed-old
and complete published intent. It marks the later confirmation phase: only Complete
is allowed and the store must already open as the exact clean candidate. Recovery
appends only the authorized suffix without truncation. Confirmed-candidate plus
intent likewise requires Complete and a strict candidate-store open. An old-selected
or residual store in these later states is inconsistent and rejects.

Visible bytes after a failed directory sync do not alone authorize retiring intent.
Recovery resynchronizes the selected confirmed record and directory before unlink;
clean writer opening resynchronizes confirmed control/root/parent, including after
a visible intent unlink whose final sync failed. Errors consume the node owner and
return no ordinary writer. **An error after intent retirement can leave an already
confirmed clean state**, so errors do not invariably mean a pending intent remains.
Use the defined read-only status and explicit open/recovery paths; do not guess
success, manufacture intent or automatically repair unknown control data.

The per-call control payload-read allowance is 314,269,392 bytes, shared by reads,
length-plus-one sentinels, synchronization reads and repeated exact comparisons.
Storage retains its separate existing budgets. Extra accepted chains, replay and
checksum buffers are not covered by an RSS guarantee. The component has no socket
or hard CPU deadline and supplies no global admission policy. Process termination
tests are not physical power-loss certification, an external audit or a statement
that WSL/DrvFS or untested filesystems are supported.

### Acceptance and preserved policy

The implementation checkpoint passed **1,083 workspace/all-target runtime tests**:
1,010 retained plus **73 new** (24 codec, 31 public integration, 12 separate-process
and six deterministic failure/revalidation tests). **Eleven compile-fail doctests**
passed, including the new consuming node-adoption ownership case. Required release
suites passed; the added `mirror-node` release gate has 73 runtime tests and one
doctest. Release reruns and multiple scenarios within a test are not additional
unique tests. All **twelve independent reference checks** and **sixteen examples**
passed, including the new control reference and `node_control_v2_demo`.

Coverage includes strict clean restart and read-only status, both exact histories
recovered from disk, complete network/checkpoint/metadata validation, malformed and
unsafe control, genuinely different membership forks with identical account bytes,
intent-before-store failures, explicit completion/retention, selected-candidate
rollback rejection, competing owners, and separate-process termination at each
control publication phase and during repeated recovery. Unknown partial intent
remains deliberately unsupported. Final command logs and exact-source CI evidence
belong to the checkpoint; published documentation revisions are verified separately.

Formatting, whitespace, preservation and allocation/runtime-data audits passed.
**140 of 142 retained runtime/dependency/reference/fixture files are byte-identical**.
The only changed retained files are the node Cargo manifest and lockfile, adding
one local test-only `mirror-personhood` dependency edge. No external version changes
or changes to retained runtime implementations, monetary rules, encodings, hash
domains or prior frozen fixtures are introduced. Earlier complete-chain, store,
recovery and transport tests remain applicable.

No founder or marketing reserve, reward/fee formula, admission/quorum choice,
biometric/accessibility requirement or final human-consensus policy is selected by
this component. The user's equal first-1,000 bootstrap policy and combined ECG/
pinky/Ant Kiss plus accessible-route proposals remain unchanged and retain their
later decision/evaluation gates. No actual personal records, device capture or
hardware purchase is needed for this software acceptance.

### Historical A1 continuation — MB-04A2

**At the A1 checkpoint, next was finite V2 init/status/sync-once/explicit-recover
operator commands.** The following was that continuation plan; the A2 engineering
record follows below. Build these through the accepted A1 library and existing F2
bounded peer-session APIs.
Keep network/path selection and initialization explicit; failed opening cannot
silently initialize or repair. Status remains read-only. Sync-once preserves F2
attempt/deadline limits and A1's complete durable-intent ordering, and unresolved
intent requires a separately requested recovery action. Verify command output,
failure exits, independent process restart and whole-branch state. Persistent
serving/reconnect/multi-peer scheduling, transaction pool and wallet remain later
MB-04 work; do not bundle them into this operator increment.

## MB-04A2 engineering choices — MIRROR-MB04A2-001

The separate `mirror-node-v2` executable implements finite explicit operator
commands through the existing A1 control and F2 receiver APIs. Its source contract
is [NODE_OPERATOR_V2.md](NODE_OPERATOR_V2.md); acceptance evidence belongs to
the [master plan](../MASTER_PLAN.md), section 28, and
[CURRENT_STATUS.md](../CURRENT_STATUS.md). This is a bounded component of MB-04,
which remains **IN_PROGRESS**. Provider lifecycle and scheduling, pool and wallet
remain unfinished; Gate A and Gate B remain unmet.

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

### Explicit finite commands and validation before effects

All operational commands require deliberate control/store directories and
independently selected C/genesis. `init` uses the allowed attach alternative:
mandatory valueless `--attach-existing` and independent full endpoint/generation
pins for an already initialized profile-3 store. It creates no chain or store,
chooses no bootstrap distribution and never falls back after a failed open.
Existing A1 initialization failures may leave empty/partial control state; the
operator command preserves that state and reports failure.

`status` calls only A1's read-only observation. Missing means missing control,
not missing storage. Valid pending status reports confirmed/old/candidate intent
pins with `store_selection=uninspected`, without selecting a recovery action or
claiming the current store endpoint. `recover` directly requests the explicit
`complete` or `retain-old` action and preserves A1's recognized-layout and
selected/confirmed-candidate no-rollback boundaries.

The parser admits at most 32 arguments after the executable name, 4,096 bytes
per token and 16,384 total bytes. It rejects non-UTF-8/control input, duplicates,
unknown/foreign-command flags, extras, missing values and malformed or overflowing
numbers/hashes. Hashes are exactly 64 ASCII hex characters; both cases are accepted
and output is lowercase. Unsigned decimal has no leading zero except `0`. Init
requires height at most 4,095 and generation no greater than height before A1 can
create a directory.
All command and session-limit validation completes before filesystem/socket work.
No environment/config-file fallback, DNS or implicit working-directory selection
is introduced. Standalone help performs no filesystem or network operation.

### One ready owner, one request and bounded reporting

`sync-once` requires an explicit numeric nonzero-port peer, 1–60,000 ms timeout and
positive F2 I/O allowance at most 157,129,488 bytes. It strictly opens the clean A1
controller before contacting the peer, holds that same owner throughout exactly one F2
request, then consumes it through A1 adoption. Missing, pending, malformed,
mismatched or busy local state rejects before TCP contact. Peer data never becomes
independent identity. Complete-branch comparison, transport validation, actual
I/O charging and durable intent/confirmation ordering remain in their existing
layers. There is no second attempt, reconnect, automatic recovery or action switch.

Output uses a fixed 4,096-byte `key=value` buffer. Completed sync reports the
choice, previous/selected full pins, one actual attempt and charged I/O bytes.
No success record is emitted for an operation that fails. Static diagnostic
categories omit raw arguments, paths, peer payloads and nested error text. Exit
0 means help, clean status or operation success; 1 means operational/output
failure; 2 means usage failure before effects; status alone uses 3 for missing
control and 4 for valid unresolved intent.

Writes and flushes are fallible. Output failure after init/sync/recover reports
`operation_may_have_completed=true` and directs status inspection without retry.
An A1 error after intent retirement may already leave confirmed clean state;
unknown/corrupt control is not automatically repaired. The session timeout excludes
opening, adoption and output, whose writes have no deadline. Argument/encoded-byte
bounds are not RSS or physical power-loss guarantees. Local ownership still assumes
cooperating Linux processes and trusted operator/parent paths.

### Acceptance checkpoint and preserved policy

Verified source acceptance records **1,137 passed** workspace/all-target runtime
tests, including 54 new tests: 30 binary tests (23 parser, four output,
three diagnostic) and 24 actual-command process tests. Required
documentation coverage remains eleven compile-fail doctests. The node release
suite records 127 runtime tests plus one doctest. All existing
Rust 1.98.0/locked workspace, documentation and release gates, twelve independent
references and sixteen synthetic examples all passed locally and in exact-source CI.
Release reruns and multiple scenarios within one test are not new unique tests.

Compatibility verification confirms **153 of 154 retained source/reference/
fixture files byte-identical**. The sole retained change is node
Cargo.toml's `default-run = "mirror-node"`, preserving V1's default invocation
after adding a second binary. Cargo.lock, dependencies, lower-layer/V1 source,
existing byte formats/hash domains and frozen fixtures remain unchanged. This
record accepts the tested implementation; later documentation-head CI is checked
and recorded separately.

No monetary formula, grant, reward, founder/marketing privilege, production
admission threshold or final human-consensus policy is selected. The user's
combined ECG/designated-pinky/human-supervised Ant Kiss preference and proposed
accessible exception route remain as recorded above, with private limitation
details, identity continuity and unresolved evaluation/decision gates. No real
keys, personal records, biometric capture or hardware purchase is needed here.

### Historical A2 continuation — MB-04B1

**At the A2 checkpoint, next was bounded V2 provider lifecycle and shutdown,
starting with an explicit design before implementation.** The following was that
continuation plan; the B1 engineering record follows below. Specify deliberate
paths, independent network identity and listen address, strict clean A1 opening
before bind, and a fixed accepted history served under the held owner. Current exclusive A1 ownership
makes concurrent A2 status/sync commands busy; the design must state that tradeoff.

Define process-level session, active-connection, aggregate-byte and lifecycle
bounds that survive creation of new F2 owners rather than silently resetting
allowances. Specify explicit idle and in-flight shutdown, stopping admission and
bounded resource/ownership release, with actual-process verification. These are
future design/implementation requirements, not a completed provider or daemon.
No auto-init/recovery, reconnect/multi-peer scheduling, transaction pool or wallet
is included in B1. The remaining MB-04 and Gate A/B work stays incomplete.

## MB-04B1 engineering choices — MIRROR-MB04B1-001

Recorded **2026-11-27**. The separate `mirror-node-v2 serve` command implements a
finite provider for one fixed accepted history, with explicit application
admission and draining shutdown. The design was reviewed before implementation;
its source contract is [NODE_PROVIDER_V2.md](NODE_PROVIDER_V2.md), with the additive
accepted-stream boundary in [PEER_SESSION_V2.md](PEER_SESSION_V2.md). Current
acceptance belongs to [CURRENT_STATUS.md](../CURRENT_STATUS.md) and the
[master plan](../MASTER_PLAN.md). Full MB-04 remains **IN_PROGRESS** and Gate A/B
remain **UNMET**. Finite rejoin/serve coordination, broader scheduling, pool and
wallet are separate unfinished work.

Implementation source: **ed631eae05ebeebe2b3a58a433ab8a0cc5753150**.
Source tree: **dc32d623bf6a8e5e3ab03023b699a729e0c630a1**.
Parent checkpoint: **f507827aabcaecda85c02d66f0de6a10db8972ac**.
Source CI: [45904484131](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45904484131),
job `140197823058`.

**MB-04B1 is VERIFIED_COMPONENT — FINITE PROVIDER / DRAINING SHUTDOWN.**
All **38 local command gates** and **43 source-CI steps** passed. Full source-CI
logs confirm **1,167 runtime tests and eleven compile-fail doctests**, zero failures
and three retained parent-exercised ignored helpers. The actual synthetic checkout
`cdb00671509b4f2876d51bd677ab39d861833396` has no file differences from the implementation
commit. All **17 published implementation/test/contract/dependency/workflow files**
were read back byte-for-byte. Later documentation-head CI is recorded separately
on PR #1; source acceptance does not assert that a future head has passed.

### One retained history and explicit admission

Require deliberate control/store paths, independently selected configuration and
genesis hashes, and one numeric nonzero-port listen address. Require explicit
max-attempts 1..8, max-io-bytes 1..157129488, timeout-ms 1..60000 and admission-ms
1..60000. Complete scalar/argument validation before effects, preserving A2's
32-argument, 4096-byte-token and 16384-total-byte bounds and its strict hash/decimal
syntax. There is no DNS, ephemeral-port discovery, stdin command channel or
environment/config-file default.

Install shutdown handling after parsing, then strictly open clean A1 state before
binding. Missing, invalid, pending, mismatched or busy state rejects without
listening. A failed open never initializes or repairs control/storage. This opening
retains A1's existing replay, exact selection checks and synchronization; it is
not the synchronization-free status operation.

Retain one ready controller and borrow its fixed complete chain for every peer.
Do not refresh history, adopt a candidate, reacquire roots or clone a substitute
history during serving. Concurrent A2 status/sync remains busy because A1 keeps
exclusive control/store ownership. One listener and a serial admission loop allow
at most one admitted connection at a time; no listener clone or parallel accept
is retained while F2 processes that connection.

Use one cumulative `PeerSessionsV2` for the invocation. Successful and failed
attempts share its unchanged attempt and actual read/write byte allowance. Check
stop, time and remaining budgets before accept, then stop/time again after accept.
A stream caught in that race is closed before request I/O and is not an admitted
F2 attempt. Kernel backlog entries, completed TCP handshakes and bytes not read by
the application are outside the application-admission count.

### Narrow accepted-stream addition and preserved transport

The new `serve_connection` entry consumes an already accepted `TcpStream` and
guards it before preflight. It applies the same owner budget rules, establishes
one original attempt deadline and checks the provider's independent identity
before request I/O. Refusal, including a depleted budget or wrong local identity,
shuts down and drops the connection; no reusable stream or refreshed budget returns.

Both provider entries use one checked request/EOF/frame routine. A valid 72-byte
MPQ2 request and request EOF precede every history response. The existing F1
Begin/Manifest/all Block/End sequence, successful write-half shutdown, byte charges
and original deadline checks are preserved. Invalid, wrong-network, incomplete or
trailing requests receive no history. `serve_once` still consumes its own listener
and includes its original listener wait in its deadline; `request` is unchanged.
Caller-side admission waiting belongs to the provider lifecycle, outside the new
accepted-stream entry's deadline.

This additive adapter and narrow shared-routine extraction are the explicit B1
exception to the prior lower-layer-source preservation instruction. They introduce
no new protocol/storage/hash format, decoder, peer authority or recovery behavior.
Existing F2 tests, references, frozen fixtures and examples remain required gates.

### Bounded admission, draining shutdown and truthful reporting

The admission clock starts immediately before binding, after clean A1 opening.
Linux SIGINT/SIGTERM set a sticky atomic stop flag only; the handler does not perform
I/O, allocation, formatting, locking, recovery or process exit. A startup stop is
honored after local opening before binding. Idle admission uses waits of at most
one millisecond, subject to scheduling. Once a connection is admitted, it completes
or rejects under its original F2 deadline while no parallel accept runs.

The same final drain applies if the admission window expires during a session.
The configured admission window plus at most one original attempt timeout totals
at most 120000 ms. These are synchronous deadline allowances, not instantaneous
cancellation, hard CPU preemption or a total process wall-clock guarantee. Initial
local opening/replay, final output and scheduler/kernel delays are not silently
included in a stronger promise. No shutdown path refreshes an F2 deadline.

Normal stop precedence is shutdown, admission timeout, wire limit, then attempt
limit. Rejected peers retain their charged attempts and actual input/partial-output
bytes. Completed plus rejected equals attempts in a normal final report. The most
recent static rejected-session category remains visible after later successful
peers. Fatal local/history/allocation/listener/internal failures return operational
errors; they are not downgraded to ordinary peer rejection.

Release streams, listener and controller before the final at-most-4096-byte
`key=value` record, preserving A1's store-before-control release. Restore signal
dispositions on success and failure. There is no readiness or progress output
while ownership is held, so a blocked output consumer cannot prolong those owners.
The record reports the fixed checkpoint, stop reason, attempts, completed/rejected
sessions, actual bytes and latest static session error. It does not certify remote
validation/adoption or prevent a later local writer from changing storage.

Normal requested shutdown or a configured limit exits 0, including runs containing
rejected peers. Operational or output failure exits 1; usage failure before effects
exits 2. Only existing status uses exits 3 and 4. Output failure can follow completed
serving, reports `operation_may_have_completed=true`, and directs status inspection
without restarting or retrying. An uncatchable process kill remains separate forced
termination, not evidence of graceful drain or physical power-loss safety.

### Acceptance evidence and controlled compatibility exceptions

The runtime acceptance count is **1,167**, including **30**
new tests. Node runtime coverage is **148** plus one retained
doctest; network runtime coverage is **108** plus two retained
doctests. Required gates include eleven compile-fail doctests overall, all required
release suites, twelve independent references, sixteen synthetic examples and
the full 38 local command gates. Release reruns, process children and multiple
scenarios inside a test are not additional unique tests.

Provider acceptance covers refusal before binding, full history and complete
ledger/membership transfer, actual cumulative allowances across successful and
rejected peers, original deadlines, idle and in-flight signals, admission expiry,
listener/control/store release, strict fresh reopening and output failure after
release. Adapter coverage includes preflight socket closure, unchanged request EOF,
partial-output accounting and mixed old/new methods on one lifetime owner.

Exactly **17 implementation/contract/workflow files** change in the source
publication. Compatibility comparison records **153 of 161 retained source,
reference and fixture paths byte-identical**, with eight intentional changes.
The node adds a Linux-target direct edge to already locked **libc 0.2.189** for the
signal boundary; no dependency package version, source or checksum changes. This
node manifest/lockfile edge is an explicit B1 exception. The CI job timeout changes
from 15 to 20 minutes while its toolchain and verification commands remain unchanged.
Those infrastructure exceptions do not alter verification requirements or count
as new runtime features.

V1 behavior and default invocation, A1 control/storage/consensus implementation,
all existing encodings/hash domains, references and frozen fixtures remain unchanged.
The F2 source adaptation is limited to the entry point and shared routine described
above. Verify published source readback and exact-source CI against that implementation.
Later documentation publication and its own CI must be recorded separately;
source acceptance does not
assert that a future documentation head has passed.

No new monetary formula, issuance, grant, reward, founder/marketing privilege,
production admission/quorum threshold, final human-consensus rule or device choice
is selected. The combined ECG/designated-pinky/supervised Ant Kiss preference and
accessible exception proposals retain their recorded privacy, identity-continuity
and evaluation/decision gates. No real keys, personal records, biometric archive,
capture or hardware purchase is introduced.

### Historical B1 continuation — MB-04B2

The following records the next design scope at the accepted B1 checkpoint. The
B2 engineering and exact-source acceptance evidence follow below.

**Next: finite rejoin and serve coordination, beginning with explicit design.**
Strictly open clean A1 control, receive at most one complete candidate through F2
from an explicitly chosen numeric peer, apply existing keep/adopt rules and serve
the resulting fixed complete history. Retain the returned ready controller through
serving and one cumulative F2 attempts/bytes owner across receive and serve. Do not
reset budgets, reacquire ownership or bypass complete validation between phases.

Define shutdown separately during receipt, before adoption, during synchronous
durable work and during serving. Preserve original session deadlines and A1's
intent/publication/confirmation ordering. Later bind, serving or output failure
must report that adoption may already have completed and direct explicit inspection
or recovery. A failure must not silently rerun the peer, initialize, recover or
pretend a durable operation was rolled back.

Use three independent processes and directories: seed -> rejoining node -> third
requester. Verify agreement of complete accounts, memberships, approvals, sequences
and population state, then strict fresh reopening and explicit clean restart/rejoin.
Include pending-state startup refusal, ownership continuity and no rollback of a
selected candidate. The design must account for the remaining shared attempt/byte
allowance before entering serving; a new phase never grants a fresh allowance.

No automatic reconnect/recovery, concurrent receive/serve phases, discovery,
multi-peer scheduler, pool or wallet belongs to B2. It remains planned work;
full MB-04 stays IN_PROGRESS, Gate A/B stay UNMET, and no unresolved monetary,
enrollment or device decision blocks this dependency-ready engineering design.

## MB-04B2 engineering choices — MIRROR-MB04B2-001

Recorded **2026-11-27**. The separate `mirror-node-v2 rejoin-serve` command
composes one explicitly selected peer receive, existing A1 keep/adopt and finite
serving of the resulting fixed complete history. The design contract was written
before implementation in [NODE_REJOIN_V2.md](NODE_REJOIN_V2.md). It reuses the
accepted [control](NODE_CONTROL_V2.md), [operator](NODE_OPERATOR_V2.md),
[provider](NODE_PROVIDER_V2.md) and [session](PEER_SESSION_V2.md) boundaries.

**MB-04B2 is VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION.** All **38 local command gates** and
**43 exact-source CI steps** passed. Published source readback and the
actual synthetic checkout were verified against the implementation. Later
documentation-head CI is recorded separately on PR #1.

| Evidence | Recorded value |
|---|---|
| Acceptance state | VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION |
| Implementation source | `bbac1622be18c97ab640f6834a158e5378f1d172` |
| Source tree | `d724df618f6b45429abebaa3c24bb52ad742799b` |
| Parent checkpoint | `966286bec62ce422db6bc091c472d74659935344` |
| Source CI run / full job | [45927607784](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45927607784) / `140272547972` |
| Actual source-CI checkout | `e00fbb793f5eb417cc0f0cda1ed52443c5cef34d` |
| Published source/test/contract files read back | 13 |
| Workspace runtime / node runtime tests | 1,201 / 182 |
| New runtime / actual-process test functions | 34 / 27 |
| Retained paths unchanged / intentionally changed | 160 / 6 of 166 |

### Explicit input and one retained authority path

Require all four common path/network options plus `--peer`, `--listen`,
`--max-attempts`, `--max-io-bytes`, `--timeout-ms` and `--admission-ms`.
Both addresses use the existing numeric nonzero-port syntax. Keep the existing
32-argument, 4096-byte-token, 16384-total-byte and canonical scalar checks before
effects; there is no implicit peer, DNS, environment/stdin/config fallback or
initialization. The total limits remain 1..8 attempts, 1..157129488 actual
application bytes and 1..60000 ms for each original attempt timeout and service
admission window.

Install the Linux signal guard once after parsing, then strictly open A1 before
either peer contact or binding. Missing, pending, corrupt, mismatched or busy
state fails unchanged; a signal cannot convert failed opening into readiness.
Retain the ready controller and its previous local checkpoint through at most
one unchanged F2 request. Any receive failure is operational failure, including
failure accompanied by a stop request. No old-history serving fallback, retry,
partial publication or implicit recovery follows.

After validated receipt and the pre-adoption stop check, consume the controller
and candidate through existing `NodeControlV2::adopt`. Copy `outcome.report()` and
retain `outcome.into_control()` for serving. Ties, identical and shorter branches
keep current; preferred candidates follow unchanged intent, replacement,
confirmation and retirement ordering. Serve only that returned fixed complete
history under the same ownership. A private shared provider loop avoids calling
standalone `serve` setup, reopening either root, clearing stop or replacing the
F2 owner. No A1/F2 public API or consensus/storage/wire/hash format changes.

### Shared transport limits and phase-specific shutdown

The receive charges the first attempt. One retained F2 owner keeps every actual
byte and attempt charge through serving. Check stop and remaining budgets before
binding. A one-attempt invocation may successfully keep/adopt then stop normally
without binding, even when the supplied listen address is occupied. There is no
reserved service allowance. The mandatory terminal EOF probe still needs one
unused byte of capacity, though zero-byte EOF is not charged.

Start the admission clock only after keep/adopt, immediately before permitted
binding. B1's serial admission, post-accept race check and original-deadline final
drain remain. The configured network-phase allowance is one receive timeout plus
the admission window plus at most one final service timeout: at most 180000 ms.
This excludes initial opening, durable adoption and final output, and is not hard
CPU preemption or a total process-return guarantee. Standalone B1 retains 120000 ms.

The same sticky signal guard spans every phase. Stop after successful opening
but before request gives zero attempts and unchanged verified selection. An active
receive drains under its original deadline. Stop observed after validated receipt
but before adoption admission discards the candidate and preserves the charged
receive. Once the adoption admission check succeeds, run A1 to success or error;
do not asynchronously cancel its durable work. After success, honor stop before
binding and report the actual returned selection. Serving keeps B1's stop and
final-drain behavior. No phase clears the flag or renews an active deadline.

### Truthful records, failure context and evidence limits

The normal record has the 26 fields specified in the rejoin contract, including
previous/selected local checkpoints, receive status, choice and separate phase
counters. Receive attempts plus serve attempts equals total attempts; completed
plus rejected sessions equals serve attempts; receive bytes plus serve bytes
equals total bytes. Local publication generations are not peer state or global
freshness. Complete accounts, memberships, approvals, sequences and population
state must come from the chosen accepted branch.

Any error after adoption admission carries conservative
`operation_may_have_completed=true` and `next=inspect-status`, including later
bind/listener/service failure. An adoption error exposes no ready controller and
makes no selected-state assertion. Errors can follow completed confirmation and
intent retirement; they do not necessarily imply pending intent or rollback.
RetainOld never rolls back a selected candidate. Existing commands keep their
prior diagnostic behavior through default-absent optional failure context.

Release streams, listener and controller before final formatting/output. Output
loss retains conservative possible-completion reporting for every normal B2
result, including pre-adoption shutdown. Normal stops exit 0, operational/output
failures 1 and usage failures 2; status alone retains 3/4. No output or error
authorizes automatic retry, initialization or recovery. Diagnostics remain
bounded and contain no raw paths, peer addresses, bytes or nested error text.

The 34 added tests comprise six parser, one diagnostic and 27 actual-process
functions. All required gates passed, retaining eleven compile-fail doctests,
twelve independent references, sixteen synthetic examples and every required
release suite. Release reruns, parameterized scenarios and child processes are
not additional unique tests. All 13 published implementation/test/contract files
match the accepted source; 160 of 166 retained source/dependency/workflow/reference/
fixture paths remain byte-identical. The six existing private command-module
changes leave lower-layer source, public APIs, manifests/lockfile, workflow,
references, fixtures and consensus/storage/wire/hash encodings unchanged.

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

### Historical B2 continuation — MB-04B3 bounded explicit-peer failover

At the B2 checkpoint, next was **MB-04B3 — bounded explicit-peer failover, design first**. Specify one finite ordered list of
explicit numeric peers, at most one attempt per entry, ending acquisition at the
first complete validated candidate. Then apply exactly one existing keep/adopt
and B2 fixed-history serving, even when that candidate keeps current.

Before implementation, define the failure-advance allowlist, duplicate-address
treatment, list/argv caps, cumulative attempts/actual bytes and total network-phase
allowance, phase-specific stop/error handling and bounded per-peer diagnostics.
Retain one controller, signal guard and F2 owner. Local/adoption failures remain
fatal; exhausted peers without a validated candidate fail without serving-old
fallback. Verify failed-first/successful-second acquisition, no later contact
after validation, exhaustion and shutdown through actual processes.

This is planned design only. No repeated-peer retry/backoff, discovery, concurrent
sessions, multiple durable adoptions, indefinite daemon, pool, wallet or policy
choice is included. Full MB-04 remains IN_PROGRESS and Gate A/B remain UNMET.
All accepted monetary, combined ECG/designated-pinky/supervised Ant Kiss,
accessibility, privacy and identity-continuity decisions/proposals retain their
existing gates. No real keys, personal records, biometric archive or hardware
purchase is introduced.

## MB-04B3 engineering choices — MIRROR-MB04B3-001

Recorded **2026-11-27**. The separate `rejoin-failover-serve` command adds a
finite ordered acquisition prefix before B2's shared keep/adopt/serve path.
[NODE_FAILOVER_V2.md](NODE_FAILOVER_V2.md) was specified before implementation.
These are narrow engineering choices; they select no monetary, enrollment,
consensus, device or accessibility policy.

**VERIFIED_COMPONENT — BOUNDED EXPLICIT-PEER FAILOVER.**

| Evidence | Verified source and execution |
|---|---|
| Implementation source / tree | `3311151ab627b3512abed47be20adea2f0e6d66a` / `fc238ae2df95b7b2b3df2457d06f739048812c7e` |
| Parent checkpoint | `9ca7d99681877d014ae64aab31cb5eb22a957c35` |
| Source CI run / full job | [45933868084](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45933868084) / `140292436668` |
| Actual source-CI checkout | `0cada2958afe985bb5fcd3b683c57a40b01042ce` |
| Implementation/test/contract publication inventory | 12 files |
| Passing workspace / node runtime tests | 1,242 / 223 |
| New runtime / actual-process functions | 41 / 22 |
| Retained paths byte-identical / intentionally changed | 164 / 5 of 169 |

The focused combined run passed **62 binary tests and all 22 new process tests**
on its first execution. The 41 added functions comprise **six parser, eight
typed-policy, three output, one error-context, one diagnostic-fallback and 22
process tests**. Parameterized scenarios, child executions and release reruns are
not extra unique tests. Full acceptance passed **all 38 local gates, 1,242 runtime
tests, eleven compile-fail doctests, all seven release commands, twelve independent
references and sixteen examples**, with zero failures and three retained ignored
helpers exercised by parent tests. Node release passed 223 runtime tests and one
doctest. Exact-source CI passed all **43 steps**; its full logs confirm these counts
and every added test in both profiles. The actual synthetic checkout has no file
differences from the source, and all 12 published source files were read back
byte-for-byte. Later documentation-head CI is separate evidence recorded on PR #1.

### Explicit list and closed advancement policy

Require all ten options, replacing B2's single `--peer` with one `--peers`
token containing 1..4 ordered numeric endpoints. Four fixed slots preserve the
32-argument/4096-token-byte/16384-total-byte caps. Reject empty, spaced, scoped,
malformed or duplicate endpoints before effects, including numeric IPv6 aliases
and mapped-IPv4 duplicates. Only duplicate comparison normalizes mapped addresses;
the actual connection retains the parsed address. Distinct endpoints do not prove
distinct machines or authenticated identities. Existing operational command
flags and records retain their semantics; help adds the command and options.
No DNS or hidden configuration is added.

Only the contract's closed typed `SessionErrorV2`/nested-error allowlist may
advance. Never decide using text or `invalid_peer_history`, which also represents
allocation and internal failures. Global session budgets, local authority,
allocation, accounting, unlisted I/O and unlisted nested errors remain fatal.
Nested manifest configuration, block-operation and ledger errors are deliberately
excluded. Some bad peer inputs therefore terminate B3; broadening the list needs
an explicit typed review and tests, not an automatic retry rule.

Strict A1 opening precedes every contact or bind. One controller, sticky signal
guard and F2 owner span all attempts and service. Each listed endpoint is tried
at most once; partial candidates and streams are discarded without refunds.
The first fully validated candidate ends acquisition even if identical, shorter
or tied and therefore kept current. No later peer is queried for a better branch.
Retain one complete branch's accounts, membership, approvals and population.

### Shared limits, stop precedence and truthful records

All receive and service calls share configured caps of 1..8 attempts and
1..157129488 actual bytes.
Each original timeout and service admission are 1..60000 ms. With P peers, M
attempts, timeout T and admission A, configured network-phase allowances are
bounded by `min(P,M)*T+A+T`, at most 360000 ms, and separately by `M*T+A`.
The final drain is not an extra allowed attempt. Opening, adoption, synchronous
work, scheduling and output prevent a hard total wall-clock guarantee.

After an eligible failure, terminal precedence is stop, exhausted wire, exhausted
attempts, then exhausted peers; fatal typed errors take precedence over those
checks. Preserve the observed error class and report the terminal reason separately.
A signal during a failed request cannot turn it into normal success or permit
another contact. Zero-attempt pre-request stop remains normal; validated receipt
under stop is discarded before adoption admission. Admitted adoption finishes or
fails through unchanged A1. Successful receipt on the last allowance can adopt
and stop without binding, including an occupied listen address.

Normal output retains B2's 26 fields plus five acquisition fields and one static
result per configured entry: 32..35 fields, at most 4096 bytes. Errors attach the
bounded acquisition trace and receive counters without selected-state assertions.
Service and acquisition errors remain separate, and earlier failures remain after
success. An invalid optional internal trace falls back to the original safe base
diagnostic, retaining class, next action and possible-completion context.
Every adoption or later fatal error uses inspection-first possible completion;
output loss never authorizes repetition. Release all owners/sockets before output.

### Evidence scope and preservation

New actual processes exercise refused or malformed first peer, real seed second,
B3 adoption/service and a third requester, with independent directories and an
explicit restart cycle. Complete reopened state agrees, including a preferred
fork with equal account bytes but different membership. Other cases cover no
later contact, startup refusal, shared limits, rejected service, phase-specific
signals, unchanged root descriptors, post-adoption bind/output failure and forced
death during the second receive. The measured invalid-header exchange is 84
bytes: 72 request plus 12 consumed response-header bytes, not all offered bytes.

The retained B2 live-adoption SIGTERM and service-drain cases exercise the shared
post-receive routine; they are not new B3 test functions or new per-phase process
observations. B3's new signal tests identify actual sockets/FDs/blocked syscalls
during acquisition. Zero-attempt stop and the inter-request flag check retain
source-level evidence rather than a claimed artificial process phase. Tests use
Linux IPv4; numeric IPv6 admission is parser coverage. Neither process death nor
successful replay proves a physical power-loss/fsync boundary.

The five changed retained private modules are args, errors, main, output and
rejoin. B2's post-receive extraction preserves its behavior and old process suite.
All 199 noncheckpoint files were frozen throughout implementation-source acceptance. Earlier tests/helpers,
public A1/F2 APIs, V1/lower-layer source, manifests/lockfile, workflow, references,
fixtures and wire/storage/hash encodings remain unchanged. The original user
policy records above remain authoritative, including combined enrollment and
accessibility proposals and their unresolved gates.

The later checkpoint updates nine status/handoff records and clarifies only the
contract's compatibility wording. Existing operational records are unchanged;
help lists the new command and options. No runtime/test file changes, and the
other 198 files frozen during source acceptance remain byte-identical.

### Historical B3 continuation — MB-04B4 bounded persistent-run lifecycle

At the B3 checkpoint, next was **MB-04B4 — bounded persistent-run lifecycle, design first**.

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

## MB-04B4 engineering choices — MIRROR-MB04B4-001

Recorded **2026-11-27**. The separate `run` command performs explicitly bounded
repeated acquisition, keep/adopt and service rounds while retaining one controller
and cumulative transport owner. [NODE_RUN_V2.md](NODE_RUN_V2.md) was specified
before implementation; its saved preimplementation SHA-256 is
`d041c117dbdfafe933e25fbd0cabe4ec0ad8887295eefd4a8e3a4a6538544b53`.
These engineering choices select no monetary, enrollment, consensus, device or
accessibility policy, and do not turn a finite invocation into an indefinite daemon.

**VERIFIED_COMPONENT — BOUNDED PERSISTENT-RUN LIFECYCLE.**

| Evidence | Verified source and execution |
|---|---|
| Implementation source / tree | `e768268fae2e4c5c9980cc74dc7a9973da56849c` / `daff130cc7652ea0946106f46a600c54671ecf18` |
| Parent checkpoint | `e421494a5a2ba28992ad581af4067618c38c0dae` |
| Source CI run / full job | [45941543950](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45941543950) / `140316760168` |
| Actual source-CI checkout | `e51ff7da8385981dd9d94e4329ed35861f2dc31f` |
| Implementation/test/contract publication inventory | 16 files |
| Passing workspace / node runtime tests | 1,281 / 262 |
| New runtime / actual-process functions | 39 / 19 |
| Retained paths byte-identical / intentionally changed | 164 / 8 of 172 |
| Frozen noncheckpoint files | 203 |

The first combined focused execution passed **82 binary tests and all 19 new
process tests**, with zero failures (86.184 seconds); no execution fixes were needed.

The 39 added functions comprise **six parser, five output, one Failure-context,
one diagnostic-fallback, seven runner and nineteen actual-process tests**.
Parameterized scenarios, child executions and release reruns are not extra unique
tests. Full acceptance passed **all 38 local gates, 1,281 runtime tests, eleven
compile-fail doctests, all seven release commands, twelve independent references
and sixteen examples**, with zero failures and three retained ignored helpers
exercised by parent tests. Node release passed 262 runtime tests and one doctest.
Exact-source CI passed all **43 steps**; full logs confirm these counts and every
added function in workspace and node release. The actual synthetic checkout has
no file differences from the source, and all 16 published source files were read
back byte-for-byte. Later checkpoint-head CI is separate evidence on PR #1.

### Explicit repeated-round authorization

Require all twelve flag/value pairs: B3's ten options plus `--max-rounds` 1..4
and `--round-wait-ms` 0..60000. One ordered `--peers` token retains 1..4 numeric
endpoints, original parsed connection addresses and alias rejection. The existing
32-argument/4096-token-byte/16384-total-byte parser caps remain; the ordinary run
has 25 tokens. Earlier commands reject these round flags. Independent round/peer
counts may exceed the total attempt allowance; earlier exhaustion is deliberate.
No DNS, discovery, automatic initialization/recovery or hidden configuration enters.

Each round starts again at the first configured entry, trying an entry at most
once in that round. B3's closed typed advancement policy is unchanged: unlisted
nested, authority, allocation, accounting and resource errors remain fatal.
The first complete validated candidate ends that acquisition even when it is
identical, shorter or tied and keep-current wins. Repeated stale-first selection
is permitted; no convergence, best-peer or global-latestness claim follows.

### One retained owner and bounded advancement

Strict clean A1 opening precedes contact or bind. One controller, signal guard and
F2 owner span all rounds and waits. The private provider and post-receive helpers
return the same ready controller on success; old command wrappers keep their prior
behavior. Each listener ends before the next acquisition or wait. There is no
concurrent receive/service loop, owner reopening or replenished session allowance.
Only a normal return from admitted keep/adopt and service completes a round;
service may perform zero attempts. All-peer failure always ends the invocation.

After a completed round, global stop priority is shutdown, exhausted wire,
exhausted attempts, then round limit. Only a completed `admission-timeout` service
with no global stop enters the bounded wait and next round. A final stop or error
never pays an irrelevant configured wait. Signals interrupt the wait without
opening another peer or listener. Fixed arrays retain at most four round reports.

Receive and service share 1..8 attempts and 1..157129488 actual bytes, including
failed and partial traffic. Every timeout and per-round service admission stays
within 1..60000 ms. For M attempts, R rounds, timeout T, admission A and wait W,
the configured network/admission/wait allowance is `M*T+R*A+(R-1)*W`, at most
900000 ms. No new global deadline is introduced. Opening, synchronous validation,
durable adoption, scheduling and output prevent a hard whole-process time bound.
Terminal EOF still needs one unspent byte of capacity although zero-byte EOF is
not charged; a last-allowed valid receive can adopt and stop without binding.

### Sticky uncertainty and fixed checkpoint evidence

A failed receive plus shutdown remains an error after earlier successful rounds.
A validated candidate stopped before admission is discarded, leaving the last
completed selection. Once any round admits keep/adopt, every later fatal error
retains its category, `operation_may_have_completed=true` and `next=inspect-status`,
including later acquisition, wait or service failures. A keep-current admission
also makes that context sticky; beginning another round cannot erase it.

Normal records distinguish the initial checkpoint, each completed round and final
selected checkpoint. They contain exactly `30+16*C+7*I` fields, with C completed
rounds, optional incomplete acquisition I and C+I<=4. Checked totals, prefix and
ordinal relationships, static labels and whole-checkpoint continuity precede output.
Latest non-`none` receive/service errors survive later success. Fixed peer-result
CSVs expose labels only. Independent-width envelopes are 3918 and 3529 bytes;
help and every record retain the unchanged 4096-byte buffer.

Error context has fifteen fixed invocation fields and eight optional current
acquisition fields. Cumulative service usage is total minus receive usage. A
combined `adoption-service` phase makes no unsupported inner-phase claim, and
`last_completed_service_error` refers only to returned service reports. Fatal
service without a report supplies no completed/rejected-session assertion or
selected pins. Malformed optional evidence falls back to the original safe base
diagnostic. Drop listener, sockets, both roots and signals before final output;
blocked or failed output never authorizes repeated work or implicit rollback.

### Evidence scope and preserved decisions

The process suite asserts real seed -> repeated run -> requester, explicit restart,
two preferred whole-history adoptions, repeated first-valid keep-current,
round-local failover ordinals and invocation-wide attempt/byte exhaustion. It
compares complete strict-reopened state, including membership-distinct histories,
and verifies retained root descriptors across acquisition, service and waits.
Signals follow actual child socket/FD/syscall observations, a listener-free sleep,
or stopped state with the exact second-round canonical intent. Later bind/output
failure and SIGKILL during later partial receipt remain separately evidenced.

The wait observer is appended to the retained test helper; its original bytes
and all prior test bodies remain. Seven retained private CLI modules change,
plus that append-only helper; public APIs, lower layers, V1, dependencies,
workflow, references, fixtures and wire/storage/hash encodings are preserved.
Linux IPv4 process cases do not establish IPv6 transport, every microscopic
ownership interval, induced OOM, an exact fsync boundary or physical power loss.
Original monetary, combined-enrollment, device and accessibility records above
remain authoritative; MB-04 is IN_PROGRESS and Gate A/B remain unmet.

### Exact next increment

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
