# Current status

Checkpoint: MIRROR-PH-001 (implementation awaiting its new CI run).
Base observed: `ad7ffd120fb451fe44cf59cb909176785b33bf8f` on `main`.
Working branch: `work/personhood-register-v0.1`.

The original devnet passed GitHub Actions run 44178257308 at commit
`1fae7c604d234d9beb74f41da5ebd02317ce862a` before membership changes.
Do not confuse that baseline success with validation of the new code.

Implemented in this branch: isolated signed membership registry, active approval
checks, terminal voluntary withdrawal, population/activation counters,
deterministic state root, bounded log codec/replay, synthetic example and tests;
separate monetary-policy calculations with zero founder bonus/marketing reserve.

Not implemented: biometric matching, assigned verifier selection, human-round
consensus, membership transactions in the live chain, grants/fees/rewards,
anonymity, external revocation/appeals/recovery, mainnet genesis.

Existing chain files/genesis/keys/balances are untouched. The public Alice 100-MRY
dev fixture is not the founder allocation. New policy supersedes old financial
brainstorming, not historical bytes. See docs/DECISIONS.md and NEXT_SESSION.md.
