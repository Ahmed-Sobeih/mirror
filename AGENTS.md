# Mirror development instructions

Read CURRENT_STATUS.md, MASTER_PLAN.md, NEXT_SESSION.md and docs/DECISIONS.md first.
Verify the live default branch, active PRs and recent commits; old chat checkpoints
are not authority over newer verified work. Preserve local unpushed changes.

Use reviewable feature branches and exact base commits. Do not reset main, delete
runtime data, rewrite old genesis, or silently change consensus encodings. The
user prefers work directly in GitHub, not repeated fragile paste/replace scripts.

No founder premine/extra allocation. No privileged marketing/airdrop reserve.
First 1,000 qualifying humans receive the same 10-MRY bootstrap allocation,
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

Run `cargo test --workspace --all-targets --locked`, documentation tests, release
personhood tests, both independent reference checks and both membership examples.
Pin CI toolchain/dependencies; do not print secrets. Distinguish written code,
actual passing tests, published branch, PR and merge status. Release reruns are
not new tests. Verify exact source commits, not just an older green check.

## Roadmap execution — MIRROR-PLAN-001

MASTER_PLAN.md is the staged roadmap and progress tracker to device integration.
NEXT_SESSION.md points to the next unfinished task: currently MB-01B/B3.
MB-01A design, B1 configuration/bootstrap and B2 signed message checks are verified.
B3 stateful execution, B4 registry roots/replay and actual ledger integration remain.
Read docs/LEDGER_MEMBERSHIP_V1.md, docs/CONFIG_V2_ENCODING.md,
docs/OPERATIONS_V2_ENCODING.md, docs/THREAT_MODEL.md and docs/MEMBERSHIP_TEST_MATRIX.md.
Coverage is scoped: B2 context/signature success does not establish reviewer rights,
target ownership/status or current sequences. Never use it alone to activate a
membership. The trusted parent view must come from locally validated stable ancestry.

Preserve V1 APIs/encodings and frozen MCF2/B2 vectors. New signed/root byte semantics
require explicit versioning/documentation and independent reference calculations.
Use docs/DEVICE_INPUT_CONTRACT.md for the private capture interface, not as a claim
of existing hardware support. Later tasks remain planned.

For every completed task record implementation commit, acceptance evidence/CI,
remaining defects and next subtask in the master plan and checkpoint files.
Do not mark integration complete because standalone tests pass. Gate A is tested
software readiness with synthetic inputs; Gate B additionally requires actual
device/capture/matching evidence. Neither is a mainnet security guarantee.

User policy decisions in docs/DECISIONS.md override proposed technical defaults.
D-01 through D-06 are scoped decision gates, not reasons to stall all work.
Continue dependency-ready work when a package is blocked. Do not choose a monetary
exception formula or collect real biometrics merely to pass a test.

## Living continuation prompt — explicit user requirement

The root `the-prompt` is the user's copy/paste handoff into a fresh conversation.
Keep it standalone and current during the conversation, not only at final handoff.
Before the final response to EACH Mirror-related message, refresh it on the verified
active branch. Include decisions, actual completed work, observed test results,
publication/PR state, unresolved questions and the exact next unfinished action.
Distinguish user decisions from proposals. On discussion-only turns preserve the
last verified checkpoint and next task rather than inventing progress.

Read current files before editing, reconcile newer work and use current content
SHAs/expected-head guards. Verify publication. Keep master/status/next/decisions
consistent. Include no secrets, biometrics or unrelated personal/project details.
A stale prompt never overrides newer live branches/PRs/CI. If unmerged, read feature
branch records, not main. After a merge/branch change update links; never reset newer
work or merge only to publish the handoff. Record tested source separately from
documentation updates; do not label an untested head tested or chase self-referential
SHAs. Work occurs in the current conversation, not promised background execution.
If publication fails, state that and provide the continuation text instead.
At session switch provide the direct live file link for copying its full contents.
