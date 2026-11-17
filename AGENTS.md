# Mirror development instructions

Read CURRENT_STATUS.md, NEXT_SESSION.md and docs/DECISIONS.md first. Verify the
live default branch, active PRs and recent commits; old chat checkpoints are not
authority over newer verified work. Preserve the user's local unpushed changes.

Use reviewable feature branches and exact base commits. Do not reset main, delete
runtime data, rewrite old genesis, or silently change consensus encodings. The
user prefers work directly in GitHub, not repeated fragile paste/replace scripts.

No founder premine/extra allocation. No privileged marketing/airdrop reserve.
First 1,000 qualifying humans receive the same proposed 10-MRY bootstrap grant,
including the founder. Ordinary allowance is 0.2 MRY per qualifying historical
population maximum. Combined bootstrap ceiling is unresolved: no assumed formula.

Raw OR encrypted biometric archives, personal identifiers and real keys must
never be committed. Tests use synthetic evidence/public development keys only.
Do not claim signatures/evidence commitments prove biological uniqueness or
anonymity. Device selection is outside the current blockchain task.

Existing fixed-PoW consensus remains a development foundation, not the final
human-weighted design. Library personhood tests are not deployed PoH. Record
unimplemented parts and trust assumptions explicitly. Never claim mainnet-ready,
crash-proof, audited, or unconditionally immutable from passing unit tests.

Run `cargo test --workspace --all-targets --locked`, documentation tests and the
membership demo. Pin CI toolchain/dependencies; do not print secrets. Distinguish
code written, tests actually passed, published branch, PR and merge status.

Maintain CURRENT_STATUS.md, NEXT_SESSION.md and the-prompt at each handoff.
