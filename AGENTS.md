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

## Living continuation prompt — explicit user requirement

The root file `the-prompt` is the user's copy-and-paste handoff into a fresh
conversation when this conversation fills up. Keep it standalone and current;
maintain it during the conversation, not only at a final handoff.

Before the final response to EACH Mirror-related user message, refresh
`the-prompt` on the verified active development branch. Include new decisions,
actual completed work, observed test results, publication/PR state, unresolved
questions, and the exact next unfinished action. Distinguish user decisions from
proposals. When a turn changes no implementation state, explicitly preserve the
last verified checkpoint and next task rather than inventing progress.

Read the current file before changing it; reconcile newer work and use the
current content SHA. Verify the saved file after writing. Keep CURRENT_STATUS.md,
NEXT_SESSION.md and decision records consistent whenever their subject changes.
Do not include secrets, biometric data or unrelated personal/project information.

A copied prompt can be stale. A new session must verify live repository metadata,
branches, PRs and CI before continuing. If the feature PR is unmerged, read its
checkpoint files on the active branch rather than assuming main contains them.
After a merge or branch change, refresh the prompt's branch/path accordingly;
never reset newer work to a copied snapshot or merge only to publish the prompt.

Record the last verified source/CI commit separately from documentation updates.
Do not label an untested new head as tested, and do not chase a self-referential
commit SHA for the prompt itself. Updates happen within the active conversation,
not as promised background work. If GitHub access or publication fails, state
that clearly and provide the current continuation text in the response instead
of claiming the repository file was refreshed.

At handoff, provide a direct link to the current branch's `the-prompt` so the user
can copy its entire contents into the next conversation.
