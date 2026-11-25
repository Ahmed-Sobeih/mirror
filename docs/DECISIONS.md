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
Next remains MB-03E; the rough Gate A progress estimate remains 20–25%.

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
rule or implementation status changes. Next remains MB-03E.

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
[device draft v0.3](DEVICE_INPUT_CONTRACT.md). Next remains MB-03E; no code/test or
progress milestone is added by this discussion.

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
remains in progress; MB-04's live
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

**Next is MB-03E: durable membership-aware branch replacement**, with reviewed
publication, interruption and recovery rules, followed by fresh-peer reconstruction.
It must retain the stable locked root and cannot treat a local MSU2 generation as
fork weight. MB-03 remains in progress; persistent V2 node/API and human finality
remain later work. No monetary formula, mint/grant/nonzero fee/reward API, founder
privilege, marketing reserve, biometric intake, private matching or anonymity is
introduced. D-01/D-04 stay open; Gate A and Gate B remain unmet.
