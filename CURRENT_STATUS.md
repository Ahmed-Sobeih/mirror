# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB01B-B4-001 — V2 membership codec, commitment and replay VERIFIED**.
Updated: **2026-11-19**. MB-01A design and MB-01B/B1–B4 components are complete.
MB-01B is **VERIFIED_COMPONENT**; live V2 ledger integration remains unfinished.
**Next: MB-02A — TransferV2, unified operation envelope and BlockV2 codecs/commitments. READY.**

## Exact source, publication and verification

Active branch: `work/personhood-register-v0.1`; [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
remains open and unmerged. Main preserved at `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
B4 implementation: `21e2272b5ac92c00e840d7aaa26a99570a5ec24f`.
Source tree: `31dfafa463935c640cd5cb31b88588575fc06d4b`.
[Exact-source push CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870): **SUCCESS**.
Job `135155424668` full logs inspected: exact checkout, tests, references, examples
and allocation/runtime-data audit. [PR CI 44404351436](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404351436)
also passed. Rust 1.98.0 with committed locked dependencies.

The identical complete source tree passed local validation before publication.
These checkpoint documents follow that implementation; inspect their later exact-head
CI separately. Do not reset newer work or confuse a documentation commit with the
implementation commit whose evidence is recorded here.

## What B4 completes

**Canonical membership state:** `v2/state_codec.rs` implements the frozen MPS2 bytes
and `MIRROR_MEMBERSHIP_STATE_V2` commitment. It includes configuration, height,
records, sequences, inclusion positions, activation metadata, retained signed
approvals and counters. Records/reviewers have canonical ordering. Derived indexes
and external block-hash provenance are excluded, avoiding a block/root hash cycle.

**Untrusted snapshots:** decoding checks bounded lengths/counts, tags, ordering,
indexes, accounting, sequences and retained signatures. It returns only
`UntrustedMembershipSnapshotV2`, with no conversion to runtime membership state.
`verify_against` compares it against an independently trusted executed state.
A coherent cache cannot prove historical withdrawals, ancestry or freshness.
A compile-fail doctest enforces the wrapper's private state boundary.

**Grouped replay:** `v2/replay.rs` implements bounded MPH2 logs retaining every block,
including empty blocks. Replay uses verified bootstrap and the B3 executor. A trusted,
stable chain view supplies configuration, block hashes, exact-operation group
commitments and intermediate membership roots. The caller also supplies the expected
configuration/height/tip/root checkpoint. All must match before a registry is returned.
A failure at the final executed block publishes no partial registry.

Exact group commitments detect different signed histories that can produce the same
membership state, including reordered independent approvals and alternate valid
withdrawal windows. They are diagnostic commitments, **not MRB2 operation roots**.
Trusted chain metadata must come from independently validated history, never from
the replay log or a peer's advertised values. B4 tests use synthetic trusted views;
MB-02/03 must supply actual block authority and durable canonical-tip recovery.

**Bounded work:** the default MPS2 snapshot bound is 41,484,378 bytes. The configured
diagnostic log limit is separately 4 MiB. MPH2 additionally permits at most 4,096
block groups and 1,048,576 accounted state items across initialization/staging,
including copied configuration descriptors and allocations. Checked charging occurs
before state cloning. These diagnostic limits do not promise global scalability.

**Independent fixtures:** a Python standard-library reference computes five full
snapshots/roots, four replay groups and all five group commitments including genesis.
Nine signatures were separately generated/verified with Python cryptography using
public test keys; the committed checker needs only the standard library. Rust
independently signs, executes and reproduces all expectations. No Rust failure was
copied into an expected vector. The existing state demo now verifies full replay,
cache agreement and rejection of an old prefix and altered history.

## Actual acceptance evidence

| Check | Observed result |
|---|---|
| Workspace/all-target tests | **323 passed = 254 retained + 69 B4** |
| B4 tests | **26 snapshot + 10 independent-vector + 31 replay + 2 private arithmetic** |
| Workspace documentation tests | **1 compile-fail case passed** |
| Personhood release reruns | **216 unit/integration tests + the same 1 doctest passed** |
| Independent Python references | **All 3 passed**: config, operations, state/replay |
| Synthetic examples | **All 3 passed**: registry, operation_v2, state_v2 |
| Changed Rust formatting and diff checks | **Passed locally** |
| Preservation/allocation/runtime audit | **Passed**; no dependency or Cargo.lock changes |

Release reruns are not additional unique tests. Personhood total 216 consists of
26 V1 + 31 B1 + 39 B2 + 51 B3 + 69 B4 cases. Review caught and fixed configuration-copy
work accounting and strengthened a late-failure test to reach execution at height 4.
This implementation review is not an external security audit.

[The test matrix](docs/MEMBERSHIP_TEST_MATRIX.md) records scoped membership
commitment, cache and replay coverage; block/network integration rows remain open.
Mixed balances/membership, actual block validation, durable replacement and monetary
qualification portions remain MB-02/03 or later; all 35 rows are not complete.

## Preserved execution, policy and remaining boundaries

B3 transition rules remain unchanged: exact actor sequences, permanent key/evidence/
ID reservations, expiry on empty blocks, self-only withdrawal, parent eligibility,
end-of-block activation in original inclusion order and final-count high-water.
Retained approval budgets remain 64 per record and configured max_records in total.
No history eviction, forced withdrawal or founder override was introduced.

Legacy V1, MCF2/B2 bytes and vectors, node/genesis, monetary helpers, dependencies and
runtime data are preserved. Alice's public 100-MRY fixture is development data.
No extra founder allocation or privileged marketing reserve. First 1,000 qualifying
humans, including founder, have the same agreed 10-MRY bootstrap grant (not yet issued);
ordinary allowance is 0.2 MRY per qualifying historical population maximum.
1 MRY = 10,000,000 Nusa. D-01 combined bootstrap treatment and D-04 fees/rewards
remain unresolved. These components activate synthetic membership and pay nothing.

No live V2 membership blocks/P2P, human-round consensus, real biometric capture/
matching, anonymous credentials, grants/fees or device adapters are enabled.
Fresh keys and fresh evidence from one person require future private deduplication;
public member/review links remain linkable. Gate A and Gate B remain **UNMET**.

Follow [NEXT_SESSION.md](NEXT_SESSION.md): MB-02A types/codecs first, then MB-02B
atomic accounts + membership and MB-02C pinned genesis/chain validation. Preserve
original full-block operation indices when adding transfers; do not filter mixed
blocks into the membership-only MPH2 format and silently change their meaning.
