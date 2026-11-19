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
At the B4 checkpoint, MB-02A was next. Its subsequent implementation is recorded
below; MB-02B/C still integrate the ledger and real block validation, followed by
MB-03 persistence.
No grant, fee, monetary exception formula, private matching, anonymity, real
admission policy or final human-consensus decision was selected by B4.


## MB-02A engineering choices — MIRROR-MB02A-001

Source `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382` implements the previously selected
V2 transfer/unified-operation and MRB2 format contract. Exact-source CI 44428924399
passed; CURRENT_STATUS.md records the acceptance evidence. MB-02B atomic execution
is next. No user economic, admission, privacy or human-consensus decision changed.

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
