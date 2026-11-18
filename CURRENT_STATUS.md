# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Latest implementation: **MIRROR-MB01B-B2-001 — signed authorizations VERIFIED**.
MB-01A design complete; MB-01B **IN_PROGRESS**, B1/B2 verified, B3/B4 pending.
**Next: MB-01B/B3 — bounded staged V2 membership execution. READY.**

## Exact source, publication and verification

Active branch `work/personhood-register-v0.1`; PR #1 open/unmerged:
https://github.com/Ahmed-Sobeih/mirror/pull/1
Main preserved at ad7ffd120fb451fe44cf59cb909176785b33bf8f.
B2 implementation: `af1dd66c4fb8811d8eab54a1c88a8bddae2ae80e`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44205064789 — SUCCESS.
Job 134468771581 full logs and step outcomes inspected. Rust 1.98.0, locked Cargo.
These checkpoint documents follow the tested source; inspect later exact-head CI
separately. Verify live refs/comments before continuing, never reset newer work.

## Completed B2

Added `crates/mirror-personhood/src/v2/operation.rs` and public exports, typed
Enroll/Approve/Withdraw, immutable signed messages, strict signature verification,
complete-request enrollment IDs and signed operation IDs. Exact prefix131 and
signed lengths291/227/227 are fixed in docs/OPERATIONS_V2_ENCODING.md.
Decoder enforces cap/version/membership kind/exact size before fixed-size copying.
Transfer=0 and legacy formats reject; no generic variable payload is introduced.

Context checks bind ConfigV2, trusted parent-view context, parent+1 candidate height,
recent exact canonical ancestor and inclusive authorization window using checked
arithmetic. Enrollment requires nonzero nonce/evidence, expected evidence profile
and sequence0. ParentChainViewV2 is a documented TRUSTED stable local view, not a
peer oracle. Tests use synthetic views; no actual chain resolver exists yet.

B1 remains intact: validated immutable configuration, bounded MCF2, all-seed proof
verification, noncircular context, independent fixed vectors. V1 types/encodings,
old chain/node, test balances, monetary helpers and dependencies are unchanged.

## Actual tests

- **203 workspace tests passed =164 retained +39 new B2 tests.**
- **96 personhood tests passed again in release =26 V1+31 B1+39 B2.** Reruns are not
  extra unique tests.
- Both independent Python reference checks pass. B2 byte/hash/ID calculation uses
  hashlib/struct; signature constants were independently generated locally with
  Python cryptography Ed25519 and matched/verified by Rust tests in Actions.
- Legacy enrollment/replay demo and new `operation_v2_demo` both execute. The latter
  accepts its synthetic authorization at candidate height1 and rejects it at9
  as expired. It performs no registry mutation and issues no MRY.
- Documentation-test command passes with zero doctest cases; allocation/runtime-
  data audit passes and Cargo.lock is unchanged.

Rust execution was in actual GitHub Actions; no local Rust compiler was available.
No new package dependency was added to Rust or CI Python. See test matrix section4.

## Important scope boundaries

B2 verifies signed/contextual messages, NOT valid registry transitions. It does not
consume/check current Approve/Withdraw sequences, establish target status/ownership,
check reviewer eligibility, enforce approval-to-request expiry alignment, detect
history reuse, expire records or activate members. A context-valid approval may
still be invalid against state; the test suite explicitly demonstrates this.
B3 must enforce those conditions atomically against the correct parent snapshot.

Matrix: T04 membership-only codecs, T05 membership binding, T06 target signatures
and T09 synthetic ancestry are verified subcases; T10 message windows verified.
Stateful T07/T11 and B3+ requirements remain pending. Do not mark all 35 cases PASS.
F01/F03 defenses are present in V2 context/message layer, not a deployed fix to the
legacy chain. Full genesis pinning/live ancestry, durable sync and replay are later.

No V2 membership executor, live blocks, human-round consensus, biometric matching,
anonymity, real enrollments, grants/rewards/fees or device adapters are enabled.
Gate A/B remain UNMET. Fresh keys/evidence from one person need external private
deduplication; public member/reviewer relationships remain linkable.

## Accepted policy and preserved data

No founder bonus or privileged marketing reserve. First1,000 qualifying humans
including founder have identical10-MRY bootstrap allocations; ordinary allowance
is0.2 MRY per qualifying historical population maximum; 1 MRY=10,000,000 Nusa.
Bootstrap-floor/additive D-01 and fee/reward D-04 remain undecided; no minting API.
No local mirror-data, genesis or wallet balance was touched. Known Alice100-MRY
fixture is development-only, NOT founder funds. No biometric archives/secrets in Git.

Continue NEXT_SESSION.md: B3 execution, B4 roots/replay, then MB-02 mixed ledger.
