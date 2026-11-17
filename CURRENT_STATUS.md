# Current status

Planning checkpoint: **MIRROR-PLAN-001 — device-readiness roadmap recorded**.
Implementation checkpoint: **MIRROR-PH-001 — standalone registry verified**.

**Master roadmap:** [MASTER_PLAN.md](MASTER_PLAN.md).
**Next unfinished task:** **MB-01A — integration design, threat review and tests**.
Status: **READY, not implemented or in progress yet**.

## Publication observed before this planning update

Active branch: `work/personhood-register-v0.1`.
PR #1: https://github.com/Ahmed-Sobeih/mirror/pull/1 — open, ready for review,
not merged. `main`: `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
Observed branch head: `dd4c2a1a7c73368364a27557537864d8c9814a03`.
Verify live refs before continuing; never reset newer work to these observations.

## What changed in this planning turn

Added MASTER_PLAN.md with dependencies, decision gates, acceptance tests and a
progress table from the current register through integrated enrollment readiness.
Added docs/DEVICE_INPUT_CONTRACT.md as DRAFT requirements for interchangeable
fingerprint/ECG capture sources. Linked the roadmap from README/AGENTS and refreshed
NEXT_SESSION.md and the-prompt. Recorded the user-requested endpoint in decisions.

This turn changes documentation only. No Rust implementation, consensus encoding,
monetary decision, dependency, wallet balance or existing database changed.
Writing the plan does NOT complete any of MB-01A through MB-12.

## Verified implementation baseline

The existing development chain supports signed transfers, account state, fixed
simple PoW, persisted blocks, TCP sync and competing-history replacement.
The separate personhood state machine supports signed Pending/Active/Revoked
membership, distinct active approvals, terminal voluntary self-withdrawal,
exact request/key/evidence reuse rejection, failure-atomic event/batch execution,
activation/population high-water counters, state roots, bounded log codec/replay.
It is NOT connected to live block validation or peer membership synchronization.

Monetary components encode zero founder bonus/marketing reserve, 10 MRY for each
of the eligible first 1,000 activation ordinals, and checked ordinary allowance of
0.2 MRY per qualifying historical population maximum. There is NO mint/claim API
and NO combined ceiling: bootstrap-floor versus additive treatment is unresolved.

## Actual test evidence, separate from this planning publication

Source implementation: `3936d2a528e73444b667860f8987bbebcfe7ab78`.
Expanded verification: `b0e9b75aad28f3804fc3a35a0bde6bfa6807e2b3`, run 44181156418.
Later checkpoint: `685a6e6838e8fb8e3480131dc362afc953112702`, run 44181734498.
Latest pre-plan exact-head CI observed: `dd4c2a1a7c73368364a27557537864d8c9814a03`,
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44186275709 — SUCCESS.

Recorded results: 133 workspace/all-target tests, including 26 personhood and
5 monetary-policy tests; 26 personhood tests also pass in release mode; synthetic
enrollment/replay demo succeeds. Documentation-test command succeeds with zero
cases. Allocation/runtime-data audit succeeds. No new runtime test result is
implied by the plan; inspect exact-head CI for its publication commit separately.

## Gate definitions and remaining work

Gate A is a tested software integration release candidate using synthetic inputs
through documented device interfaces, not just the current standalone library.
Gate B additionally requires supported real devices and controlled capture/matching
validation. Neither is a public mainnet or a claim of absolute biometric uniqueness.
Both gates are currently UNMET. See the master plan for exact completion criteria.

Not implemented: actual biometric capture/matching, private matching architecture,
production verifier assignment and bootstrap transition, external revocation or
recovery, anonymous credentials, live membership blocks, human-round consensus,
grants/rewards/fees, device adapters or deployment acceptance.

Preserve existing local runtime data and the public Alice 100-MRY development
fixture; it is not a founder entitlement. Fresh keys/fresh evidence from the same
human remain undetectable by the register alone. The standalone event log is not
a rollback-protected authoritative chain. Use synthetic data; no biometric archive
belongs on-chain or in GitHub. Current public member keys/approval links are linkable.
