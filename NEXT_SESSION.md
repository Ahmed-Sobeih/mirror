# Next session

1. Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md and
   docs/PERSONHOOD_V0_1.md. Verify live branches, PRs and exact tested commits.
2. Continue the experimental membership milestone before inventing more policy.
   Review admission ordering, evidence freshness, resource bounds and replay
   assumptions; add missing negative tests.
3. Specify versioned membership operations and a membership-state commitment for
   ledger integration. Preserve old devnet compatibility or explicitly version a
   new development network; never silently change genesis/transaction bytes.
4. Integrate registry execution into candidate validation/replay, then test
   synchronization and reorgs where membership histories differ. The standalone
   event log is not itself a blockchain and has no trusted latest-tip mechanism.
5. Leave actual grant/mint/reward implementation blocked on the combined bootstrap
   exception/ordinary ceiling decision and qualifying population rules.
6. Keep hardware/biometrics off-chain and use synthetic evidence only. Do not add
   the old founder premine or a privileged marketing reserve.

Work directly in GitHub on a reviewable branch, keep the user informed, run CI,
record actual results and update the-prompt before handing off.
