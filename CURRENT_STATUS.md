# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB02C-001 — pinned V2 block acceptance VERIFIED_COMPONENT**.
Updated: **2026-11-20**. MB-01A, MB-01B/B1–B4 and MB-02A/B/C are complete within
their recorded component scope. **MB-02 is VERIFIED_COMPONENT — IN MEMORY.**
**Next: MB-03A — explicit V2 storage identity, writer locking and recovery contract. READY.**
MB-03 covers durable publication, synchronization and membership-aware reorganization.

## Exact source and publication

Active branch: `work/personhood-register-v0.1`; [PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1)
is open and unmerged. Main preserved at `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
MB-02C implementation: [`db2cb58ba5a9a828ec949debd1b52b5915a66f3b`](https://github.com/Ahmed-Sobeih/mirror/commit/db2cb58ba5a9a828ec949debd1b52b5915a66f3b).
Source tree: `3fa1d3b44abb885898cc256f9525355025649a0a`, identical to the locally
verified tree. [Exact-source push CI 44623282448](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623282448).
**SUCCESS**. Job `135910328823` full logs were inspected: exact source checkout,
all test/reference/example gates and the allocation/runtime audit passed.
[PR CI 44623296178](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623296178)
also **SUCCESS**.

Checkpoint documentation follows the source implementation. Inspect later
exact-head CI separately; do not label a newer documentation head tested merely
because the implementation passed. Never reset newer work to this checkpoint.
Previous MB-02B source `420d7114ddf782469a6ace6ba0a2c4076fb93aaf` and its evidence
remain in master-plan section 16; section 17 records the MB-02C increment.

## What this milestone implements

**A selected network and exact genesis.** The bounded public MNM2 manifest carries
canonical configuration, every ordered seed proof and an exact genesis hash.
Decoding requires an independently selected configuration/genesis identity. A
valid signature set does not authorize replacing that choice. Genesis still has
to pass its field, empty-body, work and independently executed bootstrap-root
checks; another valid mining nonce cannot silently replace the selected genesis.

**Complete actual-block acceptance.** `mirror_chain::v2::ChainV2` owns its accepted
blocks and combined ledger. It checks full-body budgets, version/configuration,
height, actual parent, strictly increasing timestamp, exact configured target,
actual work and ordered operation root. It then executes the original complete
mixed body through the atomic ledger and compares the calculated ledger root.
Only after success does it bind the actual hash and install block/state/counters.
A valid-work block claiming a false final root leaves the entire chain unchanged.

**Actual candidate-branch ancestry.** Signed anchors resolve only against the
chain's private accepted prefix. The outer acceptance API does not accept an
external parent oracle, forged ledger or decoded membership cache. Two valid
branches can contain different admissions/withdrawals; accepting one branch
neither elects it globally canonical nor gives it finality.

**Complete bounded replay.** Typed and encoded MRB2 histories rebuild through that
same acceptance path, preserving transfers, membership, empty blocks and original
indices. An external expectation pins C, genesis, final height, tip and ledger
root. Old valid prefixes, corrupt intermediate blocks, wrong roots and foreign
branch anchors reject. Selecting an obsolete expectation still cannot discover
latestness; durable selected-tip authority is MB-03.

**Independent actual-chain evidence.** A sixth Python reference generated the
manifest, 20 signatures, actual mined ancestors, five main blocks and a real
membership fork before the Rust comparisons. All nine new vector tests matched
on their first run. The example runs two separate in-memory validators, accepts
mixed blocks, rejects a re-mined false root on both, and reproduces the complete
selected history. It does not start a network or write durable chain data.

The exact [chain contract](docs/CHAIN_V2.md) freezes validation order, manifest
bytes, target handling, timestamp semantics, replay expectations and resource
charges. Existing ledger/operation/configuration bytes and previous fixtures remain.

## Observed local and exact-source CI verification

| Check | Result |
|---|---|
| Workspace/all-target tests | **523 passed = 462 retained + 61 new** |
| New cases | **15 work + 14 manifest + 20 chain integration + 3 private resource + 9 independent vectors** |
| Documentation tests | **3 retained compile-fail cases passed** |
| Core release rerun | **121 tests passed** |
| Personhood release rerun | **234 tests + the same 3 doctests passed** |
| State release rerun | **49 tests passed** |
| Consensus/chain release rerun | **23 consensus + 57 chain tests passed** |
| Independent references | **All 6 passed**; the new reference also regenerated/verified all 20 signatures locally |
| Synthetic examples | **All 6 passed**, including two-validator actual-block acceptance/replay |
| Changed Rust formatting and diff checks | **Passed** |
| Preservation and dependency audit | **Passed**; only an internal chain-to-personhood dependency was added |

Release reruns are not new unique tests. Independent reviews found no blocking
discrepancy in the manifest, work, accepted ancestry, execution order, root binding,
resource accounting or tests. These checks are not an external security audit.
No external dependency changed. Legacy behavior, frozen previous vectors, historical
genesis/runtime data and monetary constants are preserved. Runtime data is not tracked.

## Exact boundary and next work

MB-02's in-memory membership-bearing block execution is complete. MB-03 is next;
its first slice must freeze a separate V2 storage/publication/recovery contract and
implement explicit initialization, pinned identity, one writer and bounded reads
for opening/replay. Later MB-03 slices add coherent accepted-tip publication,
failure recovery, peer reconstruction and genuine membership reorganization. The legacy node has
not been silently upgraded into a V2 network daemon.

This finite component accepts at most **4,096 blocks**, **16 MiB of complete MRB2
history**, and **1,048,576 cumulative abstract state/configuration items**. Capacity
exhaustion rejects rather than deletes history. Timestamps depend only on the
parent and increase strictly; no future-time ceiling exists, so `u64::MAX` has no
successor. These explicit synthetic limits/time semantics need reviewed evolution
for a production network. Fixed easy work is not equal-human consensus.

No extra founder allocation or privileged marketing reserve exists. First 1,000
qualifying humans, including the founder, share the agreed 10-MRY bootstrap policy,
not yet paid. Ordinary allowance is 0.2 MRY per qualifying historical population
maximum; 1 MRY = 10,000,000 Nusa. D-01 combined bootstrap treatment and D-04 rewards/
fees remain unresolved. No mint, grant, nonzero fee or reward was enabled. The
historical public Alice 100-MRY fixture is not a founder entitlement.

Fingerprint/ECG capture, private duplicate matching, real admission, anonymous
membership proofs and human-round consensus remain later work. Exact key/evidence
equality is not biological uniqueness; public review metadata is linkable.
**Gate A and Gate B remain UNMET.**
