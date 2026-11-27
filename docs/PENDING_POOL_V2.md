# MB-04C1 — bounded local V2 pending-operation pool

Contract: **MIRROR-MB04C1-001**. Design selected on **2026-11-28**, before
implementation. This is a local in-memory component in `mirror-node::v2::pool`.
Acceptance, publication and the next increment are recorded separately in
CURRENT_STATUS.md and MASTER_PLAN.md. This contract alone is not test evidence.

## 1. Scope and authority

The pool retains bounded signed transfers and membership operations under an
actual immutable accepted `ChainV2`. It uses the existing `SignedOperationV2`
bytes, exact signed-operation IDs, contextual validation and complete ordered
`LedgerV2::stage_operations` execution. No encoding or execution rule changes.

Four observations remain distinct:

| Observation | Meaning |
|---|---|
| Retained input | A signed operation occupies one local FIFO slot. |
| Contextual admission | At admission/reconciliation it passed the existing signature, configuration, next-height window and accepted-ancestry checks. |
| Evaluated batch | One exact ordered prefix executed atomically against the stated accepted parent at that observation. |
| Canonical inclusion | The exact signed ID occurs in the explicitly selected accepted chain. Only reconciliation reports this fact. |

Admission does not check balances, consume any sequence, reserve funds, promise
inclusion or prove biological uniqueness. An evaluated batch is historical local
evidence, not a block or a publication permit. Actual block acceptance still
checks the complete block, timestamp, work, roots and cumulative chain budgets.
The pool never returns or binds `StagedLedgerV2`, calls `prepare_candidate` or
`append`, mines, signs, mutates A1/storage, persists, gossips, provides submission
CLI/API ingress, or assumes wallet/key custody. It neither owns nor clones the
accepted chain. Existing commands do not automatically create or drive a pool.

## 2. Deliberate local limits

`PendingPoolLimitsV2` supplies all five limits explicitly. `PendingPoolV2::new`
validates them before creating an empty owner bound to `chain.checkpoint()`.
Limits cannot be changed and the owner is not `Clone`.

| Field | Accepted range | What it bounds |
|---|---|---|
| `max_entries` | 1..256 | Retained signed operations. |
| `max_encoded_bytes` | 1..74,496 | Sum of retained canonical operation lengths, without block framing. |
| `max_batch_operations` | 1..64 and no greater than `max_entries` | Requested FIFO prefix length. |
| `max_batch_bytes` | 168..19,048 | Full encoded MRB2 length: 168 + sum(4 + operation length). |
| `max_work_units` | 1..67,108,864 | Cumulative abstract validation/revalidation work for this owner. |

The local per-input ceiling is **291 bytes**, the largest existing signed V2
operation. Current lengths remain transfer 243, enrollment 291 and
approval/withdrawal 227. A positive retained-byte limit may intentionally be too
small for any operation; that is a capacity refusal, not an invalid configuration.
Canonical per-operation and full-body limits still apply independently.
Canonical membership `LimitsV2::max_pending` is never a transaction-pool limit.

Encoded-byte limits do not measure struct size, allocator overhead, RSS or CPU
time. Fixed-size values and separately bounded temporary vectors add bounded
overhead. The accepted chain has its own independent bounds. These local hard
ceilings are development engineering choices, not consensus or production policy.

## 3. Parent binding and contextual admission

The binding includes configuration hash, genesis hash, height, tip hash and
accepted ledger root (`ChainReplayExpectationV2`). `admit` and `evaluate` require
the actual chain's complete checkpoint to equal that binding. A changed selected
parent returns `ParentChanged`; explicit reconciliation is required. A different
configuration or genesis returns `WrongNetwork`, including during reconciliation.
Height equality alone never establishes parent equality.

`admit(&mut self, chain: &ChainV2, bytes: &[u8])` proceeds in this order:

1. Check network/complete parent, then the 291-byte local and configured individual
   input caps. No input-sized copy, decoding, ID hash or signature check precedes
   these checks.
2. Reserve the identification work charge below; structurally decode the exact
   bytes, compute the existing complete signed-operation ID, and compare only
   against retained IDs.
3. An exact retained ID returns `Duplicate { id }` without moving the entry,
   changing retained bytes/revision, or checking its signature again. Identification
   work remains charged. Duplicates can be recognized while retention is full.
4. For a distinct ID, check entry and retained-byte capacity, checked next revision,
   and contextual work allowance before contextual verification or retention growth.
5. Validate the operation for the selected next height using the actual chain.
   Reserve vector capacity fallibly and append the unchanged typed operation and
   its ID only after success. Return `Retained { id }` and increment the revision.

Structural/signature/context/capacity failure preserves semantic retained state.
Work already reserved is not refunded. A full pool never evicts or replaces an
entry. Exact-ID deduplication is not a global history index or a replacement rule.

Admission deliberately does not scan accepted history. A still context-valid
operation already included in that history can therefore be retained. Ledger
evaluation rejects its consumed/conflicting state; explicit reconciliation, even
at the unchanged parent, removes its exact included ID. A contextual receipt is
not a claim that an operation has never appeared before.

This first scope accepts only input contextually valid for the selected **next
block**. A future `valid_from`, expired window, wrong/unknown/old anchor, wrong
configuration, malformed input or invalid signature rejects. No future-height
parking queue is created. Sequence gaps, overspends, same-domain competing
sequences and unknown membership targets may nevertheless pass contextual
admission and remain retained until execution, removal or reconciliation.

## 4. Stable order and a single atomic evaluation

The pool exposes immutable ordered operation/ID slices; their positions correspond.
`evaluate(chain, count)` requests exactly the first `count` retained entries.
Reject zero, unavailable entries, the local count/byte caps and canonical complete
body limits before copying a prefix or staging a ledger. No silent truncation,
sorting, conflict skipping, alternate-prefix retry, fee replacement or eviction.

After reserving the full evaluation charge, make bounded fallible copies for an
owned `EvaluatedBatchV2` and call `chain.ledger().stage_operations(chain, prefix)`
exactly once. Discard the staged ledger, including on success. Do not perform a
separate signature/context prepass that changes the existing original-order first
failure. Preserve the indexed `LedgerErrorV2` on failure, and leave all retained
entries, bytes, order and revision unchanged; only reserved work is consumed.

`EvaluatedBatchV2` exposes its parent checkpoint, revision, exact operations/IDs
and encoded block length through immutable getters. It is not consumed by any
state-changing pool API. `batch_is_current(chain, batch)` is an observational
comparison of actual chain checkpoint, pool binding, revision and the exact
current prefix operations/IDs. Check the prefix as well as revision so a receipt
from a different pool with different contents cannot falsely match. This check
does not waive actual block validation or promise future inclusion.

New retention, successful removal/clear and **every successful reconciliation**
invalidate earlier batch observations by incrementing the checked revision.
Duplicates, absent removal, empty clear and failed operations do not change it.
Evaluation/work usage alone does not change revision. No internal executable
cache survives or bypasses validation. Holding an old owned batch is permitted;
its parent/revision remain historical facts, and currentness can become false.

Transfer account sequences and membership record sequences remain separate.
The same actor can validly use sequence zero in both domains in one batch.
Only full mixed execution establishes spendability, sequence consumption,
parent-set review authority, pending expiry and block-end activation. Earlier
funding can enable a later spend; a valid early transfer followed by invalid
membership execution fails the entire selected batch.

## 5. Explicit atomic reconciliation and removal

`reconcile(&mut self, chain: &ChainV2)` accepts an immutable accepted parent of
the same configuration/genesis. It works for append, same-height/shorter/longer
replacement and the unchanged parent. It does not select a preferred branch.

Check identity and next revision, compute cheap bounded counts, then reserve all
reconciliation work before history-ID hashing, contextual checks or temporary
vector allocation. Reserve survivor and removal vectors fallibly before scanning.
If entries exist, scan accepted history once; compute each historical signed ID
once and mark exact matches against at most 256 retained IDs. Do not infer exact
inclusion merely from a consumed sequence or an enrollment identity.

Process retained entries in FIFO order. Remove included entries first; otherwise
run existing contextual validation against the new selected next height. Remove
newly context-invalid entries with the original typed error, including expiry,
anchor loss/age and windows now in the future after a shorter replacement. Keep
context-valid stateful blockers such as sequence conflicts/gaps or overspends.
There is no individual ledger execution or greedy scheduling during reconciliation.

Only after this complete pass, atomically replace retained vectors/byte total,
publish the new checkpoint and revision, and return `PoolReconciliationV2` with
previous/selected checkpoints plus bounded FIFO `PoolRemovalV2` observations.
Reasons are `Included` or `Context(UnifiedOperationErrorV2)`. A report with no
removals still establishes a new observation/revision. An empty pool skips history
hashing, but still charges one unit and changes its binding/revision on success.

Insufficient work, arithmetic or temporary-allocation failure leaves the previous
binding and retained semantic state intact. Any reserved work stays consumed.
Admission/evaluation against a changed parent continue to refuse until successful
reconciliation. Context rejection is a reported removal, not partial reconciliation.
No implicit resurrection occurs when a formerly included branch is abandoned;
the caller must explicitly resubmit an operation the pool no longer retains.

`remove(id) -> Result<bool, PendingPoolErrorV2>` removes that retained ID only,
updates exact bytes and preserves survivor order. `clear() -> Result<usize, ...>`
removes all retained entries and returns their count. Neither mutates accepted
history, rebinds the parent or refunds work. Unknown IDs and already-empty clear
are no-ops. These bounded housekeeping/read operations do not perform charged
validation; repeated calls are not a process-wide execution-time bound.

## 6. Cumulative work accounting

All arithmetic is checked. `work_used` is monotonically cumulative for one pool
owner; removal, clear, failure, successful evaluation and parent changes never
replenish it. A reservation beyond `max_work_units` returns `WorkLimit` before
the associated work and leaves that reservation unapplied. Earlier reservations
in the same admission attempt remain charged. Creating another owner is an
explicit new local budget, not a global rate limit or implicit retry.

Let `N` be retained entry count, `L` the submitted operation byte length, `B` the
chosen prefix length, `E` its summed encoded-operation lengths, `H` accepted
block count, `O` total accepted historical operation count and `W` accepted
canonical block bytes. Let `R` be retained encoded bytes. Define parent state
weight `S` as configuration seed count + allocation count + account count +
**4 * membership record count** + retained approval count. The record weight
accounts abstractly for record/key/evidence views and the parent active set.

| Reservation | Work units |
|---|---|
| Admission identification (even malformed/duplicate/full) | `1 + L + N` |
| Distinct admission contextual validation | `1 + L` |
| One evaluation (success or failure) | `1 + S + 4*B + E` |
| Reconciliation with entries | `1 + H + W + O*(N+1) + 3*N + R` |
| Reconciliation without entries | `1` |

Admission input/parent preflight and batch count/byte/canonical preflight refusals
do not reserve validation work. The complete reconciliation reservation covers
the entire accepted-history scan, one ID hash per historical operation, bounded
retained-ID comparisons, temporary survivor/removal capacity and contextual
revalidation of **all** retained entries, including those later found included.
It is conservatively reserved without refunds for skipped checks. Computing
these charges reads only bounded lengths/counts before expensive work.

This meter counts explicit abstract categories, not instructions, allocation bytes,
signature latency or a CPU/RSS/wall-clock guarantee. The single-stage policy is
essential: every later evaluation charges parent-state work again. It adds no
canonical population/monetary limit and does not reuse membership `max_pending`.

## 7. Verification and retained boundaries

Tests must use actual accepted `ChainV2` fixtures and preserve complete chain and
ledger state across every pool operation. Cover exact duplicates/full capacity,
order/removal, invalid signatures/windows/anchors, retained conflicts/gaps,
receive-before-spend, separate sequence domains, aggregate overspend, mixed late
failure, independent local/canonical limits, failed/repeated work consumption,
included/expired/anchor-invalid reconciliation, unchanged-parent inclusion,
same-height and shorter replacements, wrong network and stale/cross-pool batches.

Preserve existing test bodies, V1, A1/F2/storage, workflow/toolchain/dependencies,
reference tools and frozen formats/vectors. Run all existing 38 gates; record
actual new counts and exact-source acceptance separately from a later checkpoint
head. No new consensus encoding requires a new reference format in this increment.

Zero-fee/no-mint development semantics remain. Founder allocation, bootstrap and
ordinary allowance policy, combined ECG/designated pinky/human-supervised Ant Kiss,
privacy and accessibility proposals retain their existing decisions and open gates.
Pool ingress, transaction gossip, wallet, block production, broader scheduling,
full MB-04 and Gate A/B remain unfinished.
