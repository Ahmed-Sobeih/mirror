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
