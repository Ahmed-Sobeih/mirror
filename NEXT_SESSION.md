# Next session — MB-01A

Master plan: [MASTER_PLAN.md](MASTER_PLAN.md).
Input boundary draft: [DEVICE_INPUT_CONTRACT.md](docs/DEVICE_INPUT_CONTRACT.md).
Planning checkpoint MIRROR-PLAN-001; implementation remains MIRROR-PH-001.

## Recover first

Verify live default branch, active branches/PRs, recent commits and exact CI refs.
At planning handoff PR #1 is open/unmerged and the active branch is
`work/personhood-register-v0.1`. Do not look only on main for the new documents.
Read AGENTS.md, CURRENT_STATUS.md, MASTER_PLAN.md, docs/DECISIONS.md,
docs/PERSONHOOD_V0_1.md, docs/DEVICE_INPUT_CONTRACT.md and the-prompt.

## Exact next work — READY, not yet implemented

**MB-01A: review current source and define versioned membership-ledger integration.**

1. Inspect personhood, core/block/transaction encoding, account-state execution,
   chain acceptance, storage and protocol/network code plus unresolved PR comments.
   Review admission ordering, context binding, evidence freshness, resource bounds,
   failure atomicity and replay/rollback assumptions. Do not rely on a chat summary
   as a substitute for the live source.
2. Produce `docs/LEDGER_MEMBERSHIP_V1.md` and `docs/THREAT_MODEL.md` with a concrete
   test matrix. Define typed operations, signatures/replay scope, bootstrap context,
   block/state-root commitment, accounting boundaries, limits and membership timing.
3. Choose and document a versioned development-network integration approach that
   preserves old MRB1/local data. No silent genesis rewrite, main reset or wallet
   conversion. Identify policy gates rather than silently deciding them.
4. Review the proposed compatibility/transition table and map each acceptance rule
   to an implementable test. Mark MB-01A VERIFIED only when its own document/test-map
   deliverables are complete, not when the old suite passes again.
5. Then proceed to MB-01B hardening and MB-02 implementation. Follow MASTER_PLAN.md
   thereafter. MB-08 interface work can be brought forward after MB-01A; unresolved
   reward economics does not block ledger or adapter work.

## Boundaries

Do not merge PRs merely to publish the plan. Do not claim membership or human
consensus is already integrated. No real grants/minting until D-01/D-04 are resolved;
no real biometrics until the privacy and device-pilot gates are met. No purchasing
or custom hardware work is part of the immediate blockchain task. Every device
must eventually satisfy a documented export/adapter contract; "whichever device"
is not a universal compatibility promise.

Keep the master plan task status/evidence, CURRENT_STATUS.md, NEXT_SESSION.md and
the-prompt synchronized. Refresh the-prompt before each Mirror-related final
response. Preserve tests/publication evidence separately from documentation edits;
if blocked, record it and select a dependency-ready task without inventing progress.
