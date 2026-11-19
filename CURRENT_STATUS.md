# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB02A-001 — V2 transfers and mixed block formats VERIFIED_COMPONENT**.
Updated: **2026-11-19**. MB-01A design, MB-01B/B1–B4 and MB-02A are complete
within their recorded scope. MB-02 remains **IN_PROGRESS**.
**Next: MB-02B — atomic accounts + membership execution and combined ledger roots. READY.**

## Exact source, publication and verification

Active branch: `work/personhood-register-v0.1`; [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
remains open and unmerged. Main preserved at `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
MB-02A implementation: `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382`.
Source tree: `930aa22bf9810525372fd31926812c1bde6dcf81`.
[Exact-source push CI 44428924399](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399): **SUCCESS**.
Job `135240927609` full logs inspected: exact checkout, all tests, references,
examples and allocation/runtime audit passed.
[PR CI 44428936866](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428936866)
also passed.

The identical complete source tree passed local Rust 1.98.0 validation with locked
dependencies. These checkpoint documents follow that implementation; inspect their
later exact-head CI separately. Do not confuse a documentation commit with the
implementation whose evidence is recorded here, or reset newer work to this snapshot.

Previous B4 implementation: `21e2272b5ac92c00e840d7aaa26a99570a5ec24f`,
[CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870).
Its historical acceptance evidence remains in master-plan section 14. The latest
source retains and retests all existing B1–B4 behavior.

## What MB-02A completes

**Signed V2 transfers and unified operations.** `mirror_core::v2` now represents
Transfer, Enroll, Approve and Withdraw in one ordered operation list. A signed
transfer is exactly 243 bytes, with the existing V2 configuration/anchor/window
prefix, recipient, integer-Nusa amount, fee and strict Ed25519 signature. Membership
bytes remain 291/227/227. Transfers use account sequences; membership sequences
remain separate. Sender addresses retain the existing address namespace.

Transfer context validation checks the trusted parent configuration, recent canonical
anchor, inclusive authorization window, positive amount, zero development fee and
signature. It does not check funds or consume a nonce. Unified decoding preserves
the exact existing membership bytes. The old membership-only decoder and MPH2
continue to reject Transfer=0.

**V2 block format.** An immutable header is 160 bytes and commits configuration,
height, parent, operation root, claimed ledger root and work/time fields. MRB2 is
the header plus an ordered length-prefixed body; an empty block is 168 bytes.
The new block hash has an explicit domain. Every operation's signed bytes, position
and count contribute to the operation commitment. An explicit count wrapper prevents
the duplicate-last-leaf ambiguity of the underlying Merkle convention.

**Bounded parsing and integrity.** Decode preflights complete bytes, counts, exact
operation lengths/kinds and membership/enrollment quotas before allocating the body
vector. Direct construction, encoding, root calculation and integrity validation
also enforce the chosen configuration's budgets. Body-integrity checks establish
configuration agreement, the operation root and strict signatures.

**Independent evidence.** A Python standard-library reference computes five operation
fixtures, seven blocks and nine header variations. Fixed signatures were separately
generated and verified using Python cryptography; the committed checker requires
only the standard library. Rust independently recreates bytes, signatures and hashes.
All ten Rust vector tests matched on their first run; no failed Rust result became
an expected value. The synthetic mixed-block example verifies round-trip, signatures,
context and tamper rejection while explicitly leaving ledger/work claims unvalidated.

See [the frozen encoding contract](docs/CORE_V2_ENCODING.md) and the scoped
[test matrix](docs/MEMBERSHIP_TEST_MATRIX.md).

## Actual acceptance evidence

| Check | Observed local and exact-source CI result |
|---|---|
| Workspace/all-target tests | **403 passed = 323 retained + 80 MB-02A** |
| MB-02A tests | **37 operations + 33 blocks + 10 independent vectors** |
| Workspace documentation tests | **1 retained compile-fail case passed** |
| Core release reruns | **121 unit/integration tests passed** |
| Personhood release reruns | **216 unit/integration tests + the same 1 doctest passed** |
| Independent Python references | **All 4 passed**: config, membership operations, state/replay, transfers/blocks |
| Synthetic examples | **All 4 passed**: registry, operation_v2, state_v2, block_v2 |
| Changed Rust formatting and diff checks | **Passed** |
| Preservation/allocation/runtime audit | **Passed**; only the explicit internal core-to-personhood dependency was added |

Release reruns are not additional unique tests. Independent implementation cross-reviews
found no blocking issue. These reviews and checks are not an external security audit.
No external dependency version changed; Cargo.lock adds only the internal dependency.

## Exact boundary and next work

**These are V2 formats and verification components, not an accepted V2 ledger.**
A block can pass body integrity while claiming an incorrect ledger root, invalid
work, wrong ancestry or a transfer that cannot be executed. Those fields must be
checked by the future state/chain executor. Tests explicitly demonstrate this
boundary; decoding or verifying a signature never grants authority.

MB-02B must stage accounts and membership atomically, preserve original full-block
operation indices including transfers, compute account/economics/combined roots,
and prove rollback and conservation of synthetic supply. Current membership-only
`stage_block` and MPH2 cannot be fed a filtered mixed list: doing so changes committed
inclusion positions. Specify an indexed/full-block execution boundary first.
MB-02C then validates actual parents, headers, PoW, roots and pinned genesis;
MB-03 adds durable coherent publication and multi-node recovery/reorganizations.

B3/B4 remain unchanged: exact sequences, permanent key/evidence/ID reservations,
expiry on empty blocks, self-only withdrawal, parent eligibility, end-block
activation and high-water counters. MPS2 decode remains untrusted with no runtime
conversion. MPH2 still requires independently trusted grouped history and an exact
expected checkpoint. Its diagnostic group digest is not the MRB2 operation root.
Trusted parent/replay views must come from stable locally validated ancestry.

Legacy source/encodings/genesis, existing vectors and runtime data are preserved.
Alice's public 100-MRY fixture is development data. No extra founder allocation
or privileged marketing reserve exists. First 1,000 qualifying humans, including
the founder, have the same agreed 10-MRY bootstrap allocation, not yet paid.
Ordinary allowance remains 0.2 MRY per qualifying historical population maximum;
1 MRY = 10,000,000 Nusa. D-01 combined bootstrap treatment and D-04 fees/rewards
remain unresolved. This slice issues no money and activates no new members.

Live V2 membership networking, final human-round consensus, private biometric
matching, anonymity, real enrollment and device support remain unfinished.
Fresh keys/evidence still require private human deduplication; public member/review
metadata remains linkable. **Gate A and Gate B remain UNMET.**
