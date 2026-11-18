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

MPS2 state bytes are specified in MEMBERSHIP_STATE_V2.md for B4; the codec/root/replay
still require implementation and independent vectors. No unresolved user economic,
real-admission, privacy, recovery or human-consensus decision was selected by B3.
