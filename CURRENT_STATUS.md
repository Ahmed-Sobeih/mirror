# Current status

Checkpoint: **MIRROR-PH-001 — standalone registry milestone verified**.
Working branch: `work/personhood-register-v0.1`.
Publication: pull request #1, https://github.com/Ahmed-Sobeih/mirror/pull/1
The PR is not merged. Default `main` was preserved at
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`; verify live refs before continuing.

## Verified implementation

The personhood placeholder is replaced with an isolated signed membership state
machine: Pending/Active/Revoked states, distinct currently active approvals,
terminal voluntary self-withdrawal, exact request/key/evidence-reuse rejection,
failure-atomic individual/batch transitions, activation/population high-water
counters, canonical state roots, bounded event-log codec and validated replay.

Monetary-policy components separately encode zero extra founder allocation, zero
privileged marketing reserve, the same 10-MRY amount for first-1,000 eligible
activation ordinals, and a checked 0.2-MRY ordinary allowance per qualifying
historical population maximum. There is NO mint/claim API and NO combined ceiling
formula: bootstrap-floor versus additive treatment remains unresolved.

## Actual verification evidence

Source implementation commit: `3936d2a528e73444b667860f8987bbebcfe7ab78`.
Expanded CI commit: `b0e9b75aad28f3804fc3a35a0bde6bfa6807e2b3`.
GitHub Actions run: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44181156418
All steps passed using Rust 1.98.0 and the committed lockfile:

- Workspace/all-target tests: **133 passed**, including **26 personhood** and
  **5 new monetary-policy** tests. The original personhood placeholder was replaced.
- Documentation-test command passed (no doctest cases currently defined).
- Personhood release-mode tests: **26 passed**.
- Synthetic enrollment/replay example ran successfully; it issues no MRY.
- Allocation/runtime-data audit passed; the lockfile was not rewritten by Cargo.

Baseline before implementation also passed in run 44178257308. The task environment
had no local Rust toolchain; compile/runtime claims above come from actual GitHub
Actions runs, not local or hypothetical execution. This documentation-only
checkpoint follows the tested source; verify latest PR CI for its exact head.

## Preserved and not yet implemented

No node source, historical genesis, existing database, public development key,
wallet balance, block format or PoW consensus rule was changed. Alice's 100-MRY
public test fixture is explicitly documented as NOT a founder entitlement.

Not yet implemented: biometric matching, production verifier assignment, challenge
or external revocation/recovery, anonymity, membership operations in live block
validation/P2P/reorgs, human-round consensus, grant issuance, rewards and fees.
The library's duplicate checks cannot identify the same human using fresh keys and
new evidence. The standalone log also needs a trusted latest-tip/root boundary;
it is not itself a blockchain or rollback-protected production database.

See docs/DECISIONS.md, docs/PERSONHOOD_V0_1.md and NEXT_SESSION.md.
