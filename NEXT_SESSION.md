# Next session — MB-01B/B1

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md).
Design: [LEDGER_MEMBERSHIP_V1.md](docs/LEDGER_MEMBERSHIP_V1.md).
Risks/tests: [THREAT_MODEL.md](docs/THREAT_MODEL.md) and
[MEMBERSHIP_TEST_MATRIX.md](docs/MEMBERSHIP_TEST_MATRIX.md).

## Recover the live state

Verify default branch, active branches, PR #1, recent commits and exact CI.
At handoff the feature branch is work/personhood-register-v0.1, PR #1 open and
unmerged; main is still the old development foundation. Read AGENTS.md,
CURRENT_STATUS.md, MASTER_PLAN.md, this file, docs/DECISIONS.md, the three MB-01A
documents, docs/PERSONHOOD_V0_1.md, docs/DEVICE_INPUT_CONTRACT.md and the-prompt.
Newer verified work wins over this snapshot. Do not redo MB-01A or reset changes.

## Exact next implementation slice — MB-01B/B1

1. Inspect the current personhood/crypto APIs and the MB-01A configuration design.
   Add a clearly separate V2 module (for example v2/config.rs); retain all current
   V1 types, MPH1 codec and fixed vectors. Do not modify MRB1 or legacy node data.
2. Implement bounded rules, canonical unsigned configuration and ordered bootstrap
   descriptors, context derivation, seed ID derivation and seed proof verification.
   Compute context before seed signatures/roots/genesis to avoid a circular hash.
   Keep personhood independent of mirror-core: use raw fixed-size address descriptors
   or another reviewed acyclic boundary for synthetic account allocations.
3. Freeze exact config encoding/domain strings and independently calculate fixed
   vectors. Context must change when network, seed contents/order, threshold,
   profile, limits or synthetic genesis allocations change. Invalid bootstrap,
   duplicates, unsorted accounts and checked-sum overflow must reject.
4. Implement test-matrix T02/T03 and configuration portions of T20. Changing valid
   seed configurations must invalidate old seed/event authorization context. No
   privileged founder path. Proposed small dev limits are not production policy.
5. Run workspace/all-target, doc command, release personhood tests and synthetic
   demo in pinned CI. Publish code, actual results and updated checkpoint records.
   Mark B1 complete only on its tested commit, not merely because its API exists.

## Then continue, do not claim all hardening from B1

B2: V2 membership signed envelopes, sequence spaces, anchor resolver and height
windows. B3: bounded staged block execution, expiry, parent-snapshot eligibility,
block-end activation, tombstones, counters and roots. B4: complete codec/reference
vectors, atomicity/adversarial tests and demonstration. All are unimplemented now.

MB-02 then adds versioned transfer/block codecs and one atomic ledger state for
accounts+membership. MB-03 adds durable locking/replay/sync/reorg coverage. Do not
substitute an independent side membership log for consensus-integrated state.

Monetary D-01/D-04 do not block this code, but do block real grants/minting/rewards.
Biometrics remain synthetic/off-chain; no devices are selected, purchased or
claimed supported. Real admission and human consensus remain separate gates.

Before the final response to every Mirror message, refresh the-prompt and relevant
master/status/decision records with actual progress, known risks and the exact next
unfinished slice. Preserve completed design work and verify published contents.
