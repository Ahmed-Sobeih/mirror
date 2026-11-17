# Mirror development instructions

Read CURRENT_STATUS.md, MASTER_PLAN.md, NEXT_SESSION.md and docs/DECISIONS.md first.
Verify the live default branch, active PRs and recent commits; old chat checkpoints
are not authority over newer verified work. Preserve local unpushed changes.

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

## Roadmap execution — MIRROR-PLAN-001

MASTER_PLAN.md is the staged roadmap and progress tracker to device integration.
NEXT_SESSION.md points to its exact next unfinished task: currently MB-01B/B1.
MB-01A is complete as a DESIGN task: read docs/LEDGER_MEMBERSHIP_V1.md,
docs/THREAT_MODEL.md and docs/MEMBERSHIP_TEST_MATRIX.md before coding V2.
The 35 cases are PLANNED, not passing tests. Preserve V1 APIs/encodings and use
explicit new version domains rather than silently updating old signature bytes.
Use docs/DEVICE_INPUT_CONTRACT.md for the private capture interface, not as a claim
of existing hardware support. Later tasks remain planned.

For every completed task record implementation commit, acceptance evidence/CI,
remaining defects and next subtask in the master plan and checkpoint files.
Do not mark integration complete because standalone tests pass. Gate A is tested
software readiness with synthetic inputs; Gate B additionally requires actual
device/capture/matching evidence. Neither is a mainnet security guarantee.

User policy decisions in docs/DECISIONS.md override proposed technical defaults.
D-01 through D-06 in the plan are scoped decision gates, not reasons to stall all
work. Continue dependency-ready work when one package is blocked. Do not select a
monetary exception formula or collect real biometrics merely to pass a test.

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
current content SHA. Verify the saved file after writing. Keep MASTER_PLAN.md,
CURRENT_STATUS.md, NEXT_SESSION.md and decision records consistent when relevant.
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
