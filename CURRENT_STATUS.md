# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Latest implementation: **MIRROR-MB01B-B3-001 — staged membership execution VERIFIED**.
MB-01A design complete; MB-01B **IN_PROGRESS**, B1/B2/B3 verified, B4 pending.
**Next: MB-01B/B4 — canonical V2 registry commitment, bounded codecs and replay. READY.**

## Exact source, publication and verification

Active branch `work/personhood-register-v0.1`; PR #1 open/unmerged:
https://github.com/Ahmed-Sobeih/mirror/pull/1
Main preserved at ad7ffd120fb451fe44cf59cb909176785b33bf8f.
B3 implementation: `b6f1233d038901aa0a13c8ddac9afbc34418e846`.
Source tree: `e98daf435335e31440e6c0bf14f28f5928f556c7`.
Exact-source push CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317174000 — SUCCESS.
Job 134846238261 full logs inspected, including checkout of the exact implementation
SHA, all test results, reference checks, three demos and allocation/runtime audit.
PR CI 44317184524 also passed. Rust 1.98.0, committed locked dependencies.

The same complete source tree passed local checks before publication. No local Rust
limitations remain for this turn. These checkpoint documents follow the tested
implementation; inspect their later exact-head CI separately. Never reset newer work.

## Completed B3

`v2/state.rs` adds immutable MembershipStateV2, bounded staged block execution,
Pending/Active/Revoked/Expired records, exact current sequences, permanent used-key/
evidence/ID reservations, approvals, original inclusion positions and activation
height/ordinals. Bootstrap requires VerifiedBootstrap and gives seeds ordinary
runtime rights. There is no forced withdrawal, founder override or monetary API.

Every candidate checks direct-input operation/count/byte budgets and parent context,
height and exact tip association, then rechecks B2 signatures/ancestry/windows.
Expiry runs on empty blocks too. Approvals need an existing Pending target, exact
request expiry, correct current sequence and an eligible distinct reviewer.
Withdrawal affects only the signer's own Pending/Active membership.

All activation waits until block end. Reviewers must be Active in the parent and
survive all operations. A newly activated member cannot approve within that block;
a withdrawn reviewer's earlier approval cannot cause a new activation. Previously
Active targets are not cascade-revoked. Activation order follows original canonical
inclusion position, and high-water uses final Active count only. Failed candidates
leave the ENTIRE original registry unchanged, including expiry and sequences.

New explicit synthetic-profile budgets: at most 64 retained approvals per record,
and at most configured max_records retained approvals across the whole registry.
These are separate capacities, not deducted from record count. No history eviction
or relaxed quorum at capacity. Exhaustion can stop admissions; withdrawal/expiry
remain available. This bounds development storage, not global scalability or DoS.

`docs/MEMBERSHIP_STATE_V2.md` freezes the state/optional-tag ordering for B4.
No V2 state root or replay codec is implemented yet. Stored tip hash is external
provenance, excluded from future canonical state bytes to prevent a hash cycle.
`bind_to_validated_block` is an explicit TRUSTED caller boundary: it does not
validate a real block, PoW, root, genesis manifest or finality. MB-02 must do that.

## Actual tests

- **254 workspace tests passed = 203 retained + 51 new B3 tests.**
- New B3 tests: **45 public integration tests + 6 private arithmetic/invariant tests**.
- **147 personhood tests passed again in release = 26 V1 + 31 B1 + 39 B2 + 51 B3**.
  Release reruns are not additional unique tests.
- Both independent configuration/operation Python references pass. Frozen B1/B2
  bytes and vectors are unchanged; B3 claims no new state-hash vector yet.
- All three demos pass: registry_demo, operation_v2_demo and new state_v2_demo.
  B3 demo shows activation, withdrawal removing approval weight, whole-candidate
  rejection and successful retry. It issues no currency or accepted network block.
- Documentation-test command passes with zero doctest cases. New source/test/demo
  formatting passed locally. Allocation/runtime-data audit passes; Cargo.lock unchanged.
- Independent implementation/test review found no blocking defect; an existing
  stale-sequence test was strengthened to avoid a duplicate-approval false positive.
  This review and these tests are not an external security audit.

Matrix rows are scoped: B3 covers runtime portions of T03/T06/T07/T11–T20/T22.
Root/codec/replay, mixed balances, actual canonical reorg and monetary qualification
portions remain B4/MB-02/03 or later. Do not count all 35 rows as complete.

## Preserved policy and remaining boundaries

Legacy V1, B1/B2 source/encodings/vectors, existing node/genesis, monetary helpers,
dependencies and local runtime data remain unchanged. Alice's public 100-MRY fixture
is development-only, not founder funds. No extra founder/marketing allocation.
First 1,000 qualifying humans, including founder, have identical 10-MRY bootstrap
allocations; ordinary allowance is 0.2 MRY per qualifying historical maximum.
1 MRY = 10,000,000 Nusa. D-01 bootstrap floor/additive and D-04 fees/rewards remain
undecided; B3 activates synthetic membership only and pays nothing.

No live V2 membership-bearing blocks/P2P, final human-round consensus, real
biometric capture/matching, anonymous credentials, grants/fees or device adapters
are enabled. Fresh keys and fresh evidence from one person still require private
external deduplication. Public keys/review links remain linkable. Gate A/B UNMET.

Continue NEXT_SESSION.md: B4 commitments/codecs/replay, then MB-02 unified ledger.
