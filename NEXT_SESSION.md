# Next session — MB-02A

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated 2026-11-19.
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/LEDGER_MEMBERSHIP_V1.md,
docs/CONFIG_V2_ENCODING.md, docs/OPERATIONS_V2_ENCODING.md,
docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md, docs/THREAT_MODEL.md,
docs/MEMBERSHIP_TEST_MATRIX.md and the latest the-prompt.

## Recover live state

Verify default/active branches, PR #1, recent commits/comments and exact CI.
At handoff branch `work/personhood-register-v0.1` is active; PR #1 is open/unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`. Read feature-branch records,
not old main. Preserve concurrent/unpushed work and runtime data. Never reset to a
copied checkpoint or merge merely to publish a handoff.

B4 implementation `21e2272b5ac92c00e840d7aaa26a99570a5ec24f`, tree
`31dfafa463935c640cd5cb31b88588575fc06d4b`, passed exact-source push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870
Job 135155424668 full logs/checkout inspected: 323 workspace tests, 1 doctest,
216 personhood release reruns plus that doctest, all 3 references/3 examples,
allocation/runtime audit. PR CI 44404351436 passed too. The identical source tree
passed local Rust 1.98.0 checks. Inspect later documentation-head CI separately.
MB-01A and MB-01B/B1–B4 are complete within their component scope. Do not redo B4.

## Exact next slice — MB-02A types, codecs and commitments

The next deliverable is a bounded, independently specified representation of a
V2 block containing both transfers and membership operations. It precedes live
ledger execution. Use sections 3/7/8 of LEDGER_MEMBERSHIP_V1.md and preserve the
already frozen MCF2, membership-operation and MPS2 encodings.

1. Inspect current crate dependencies and APIs before choosing placement. Avoid a
   cycle between mirror-core and mirror-personhood. Share primitive encoding
   helpers or choose an integration layer deliberately; do not rewrite the legacy
   types or move all code just to make an import compile.
2. Implement immutable signed TransferV2 and a unified operation envelope. Transfer
   uses kind 0, the existing 131-byte V2 prefix, recipient[32], amount u64, fee u64
   and signature[64]: exactly 243 bytes. Preserve Enroll/Approve/Withdraw bytes
   (291/227/227), signature domain and operation IDs. Transfer uses the account
   sequence; membership retains its separate sequence. Context/signature success
   alone must not claim sufficient balance, actor rights or current nonce.
3. Implement the specified 160-byte V2 header and bounded MRB2 codec: magic[4],
   header[160], count u32, repeated length-prefixed complete signed operations;
   an empty block is 168 bytes. Bind C, height, parent, operation root and combined
   ledger root. Enforce configuration/count/byte bounds before allocation on both
   decoding and direct construction. Unknown tags, wrong lengths, trailing bytes,
   overflow and malformed signatures must have explicit rejection boundaries.
4. Implement the specified domain-separated block hash and count-bound operation
   Merkle commitment. Freeze any still-ambiguous byte/order rules explicitly in
   the specification before coding. Produce independent reference preimages and
   hashes for empty, single, odd and mixed operation lists. Test order, signatures,
   count changes and truncation. Never bless an unexplained failed Rust hash.
5. Keep structural decoding, signature/context validation and full state/block
   acceptance separate. MB-02A alone cannot validate balances, combined roots,
   actual ancestry, PoW, pinned genesis or canonical-chain selection. No decode
   constructor may grant membership rights or instantiate an authoritative ledger.
6. Add meaningful codec/signature/vector tests and a synthetic example if useful.
   Preserve all prior fixtures. Run workspace, docs, release, reference checks,
   examples, formatting of changed files and preservation audit. Publish the exact
   source, inspect CI, then update master/status/matrix/next/the-prompt.

Do not add rewards, minting, device APIs or real data to satisfy this slice. D-01
and D-04 stay open. MB-02's zero-fee/no-mint transfer profile is development-only.

## Mandatory integration boundary after MB-02A

B3's `stage_block` and B4's MPH2 are currently **membership-only**. Their operation
positions are original positions within that profile. Mixed MRB2 blocks must retain
the original full-block indices, including transfers, for inclusion metadata and
activation order. Filtering transfers out and replaying the remaining operations
would silently change committed membership state. Specify an indexed staging API
or full-block integration path before MB-02B; update applicable index bounds/tests
without reinterpreting existing signed bytes or diagnostic history.

MPH2 group digests are diagnostic exact-operation commitments, not the MRB2 Merkle
root. It accepts membership operations only and must keep rejecting Transfer=0.
Version/extend replay explicitly or replay complete validated MRB2 blocks; do not
reuse the current diagnostic format as authoritative mixed-block history.

## Following slices

- **MB-02B:** stage accounts and membership atomically under full-block limits;
  checked positive transfers with zero fees, no issuance, separate sequences,
  canonical account/economics/combined roots and exact mixed-operation positions.
  Test that a late failure changes neither component, balances conserve initial
  test supply, empty blocks expire Pending records, and roots reproduce independently.
- **MB-02C:** explicit V2 manifest and pinned genesis, actual header/root/PoW/context/
  parent validation, stable candidate ancestry and complete accepted-block replay.
  Only validated blocks may bind staged membership provenance. Test competing
  histories with different memberships. Keep legacy initialization and data intact.
- **MB-03:** writer locking, durable coherent publication/recovery, peer sync and
  reorganizations across money and membership. Passing in-memory replay is not
  crash safety, latest-tip discovery or distributed consensus.

The master plan's later policy, privacy, human-round, operator and device gates
remain unchanged. Gate A/B are not complete. No procurement/real capture is needed.

## Existing boundaries to preserve

`VerifiedBootstrap` requires every seed proof. Seeds have ordinary runtime rights.
`bind_to_validated_block` attaches trusted provenance; it does not validate real
blocks. Keep tip hash outside canonical state. B2 and replay parent views must be
stable locally validated ancestry, never a peer oracle.

MPS2 decoding returns `UntrustedMembershipSnapshotV2` without runtime conversion.
Only comparison with trusted execution can establish cache agreement; historical
withdrawals and freshness cannot be inferred from a plausible snapshot. MPH2 replay
requires trusted per-height hash/group digest/root and exact expected C/height/tip/root.
Bounded history caps are 4,096 groups and 1,048,576 charged items including copied
configuration, separate from configured 4-MiB log and default 41,484,378-byte snapshot.
Preserve all tombstones/history and the existing approval capacities; never evict.

Before every Mirror final response, refresh the-prompt and relevant checkpoints
using actual progress, exact source/CI and next action. Read before editing, preserve
concurrent changes, publish with SHA guards, verify readback. Separate source evidence
from later documentation commits and avoid chasing self-referential SHAs.
