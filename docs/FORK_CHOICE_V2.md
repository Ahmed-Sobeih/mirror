# V2 development fork choice

Contract: **MIRROR-MB03D-001**, fixed-target synthetic development profile,
2026-11-26. Public API: `mirror_chain::v2`.

This contract compares complete branches already accepted under the
[V2 chain rules](CHAIN_V2.md). It preserves the existing configuration, block,
ledger, membership and storage encodings. It adds no wire format, hash domain,
dependency, journal, currency issuance or human-finality rule.

## 1. Scope and trust boundary

`ChainV2` privately owns its accepted blocks, immutable manifest, complete executed
ledger and cumulative resource counters. Its constructors and append/replay paths
validate actual blocks before exposing that state. Fork comparison accepts these
already validated values; it does not accept a peer's advertised height, a raw
block list, decoded state cache, claimed work score or storage selection record.

The caller independently chooses `ManifestIdentityV2`, consisting of configuration
hash **C** and the exact genesis hash. Both branches must match that selection.
Merely checking that two untrusted inputs agree with one another would not pin
the intended network. Deriving the expected identity from an untrusted candidate
would remove this protection.

The result identifies a preferred branch within this pair under the current
development rule. It does not prove that either input is the latest branch in a
network, that no better branch exists, or that the choice is final. It performs
no network communication, disk access, storage publication or branch replacement.

## 2. Public API and error order

```rust
pub fn compare_branches_v2<'a>(
    expected: ManifestIdentityV2,
    current: &'a ChainV2,
    candidate: &'a ChainV2,
) -> Result<BranchComparisonV2<'a>, ForkChoiceErrorV2>;
```

`BranchSideV2` distinguishes `Current` and `Candidate` in these errors:

- `ForkChoiceErrorV2::WrongConfiguration { branch }`
- `ForkChoiceErrorV2::WrongGenesis { branch }`

Checks have this exact order, before scoring or ancestry comparison:

1. Current configuration hash equals the expected C.
2. Current genesis hash equals the expected genesis.
3. Candidate configuration hash equals the expected C.
4. Candidate genesis hash equals the expected genesis.

An error does not select a winner or modify either branch. The function relies on
the existing accepted-chain invariants after these identity checks. It does not
repeat signature verification, proof-of-work checks or ledger execution, and it
does not offer an unchecked construction path around those checks.

`BranchComparisonV2<'a>` has private fields and no public constructor or decoder.
Its read-only accessors are:

| Accessor | Return value |
|---|---|
| `identity()` | The independently selected `ManifestIdentityV2` |
| `relation()` | `BranchRelationV2` |
| `choice()` | `ForkChoiceV2` |
| `common_ancestor()` | The shared accepted `ChainReplayExpectationV2` |
| `current_score()` | Current normalized post-genesis score, `u64` |
| `candidate_score()` | Candidate normalized post-genesis score, `u64` |
| `current_checkpoint()` | The current input's complete checkpoint |
| `candidate_checkpoint()` | The candidate input's complete checkpoint |
| `selected_chain()` | An immutable reference to the chosen complete `ChainV2` |

The comparison borrows both inputs. Safe Rust prevents mutation of either input
while that borrowing result is still used. The selected reference points to the
actual input branch with its accounts, membership, permanent records, approvals,
population counters, manifest and resource accounting intact. No state is copied
from a sibling, combined across branches or installed independently of its blocks.

## 3. Fixed-target score

Every accepted block, including genesis, must use the literal `pow_bits` selected
by the configuration. That configuration is bound by C. Thus branches accepted
under the same selected identity use the same positive target and cannot change
difficulty within their histories. Another compact encoding of the same numeric
target is not interchangeable with the selected literal bits.

The normalized score is the number of accepted blocks **after genesis**:

```text
score(branch) = branch.height()
```

Genesis alone has score zero. Under the existing maximum of 4,096 complete blocks,
including genesis, the score is in `0..=4095`. Reading the accepted height requires
no score accumulation, large-integer arithmetic or multiplication. The comparator
does not compute `height + 1`, add untrusted scores, or introduce an overflow path.

This normalization preserves fixed-target work ordering. If every accepted block
at the selected target contributes the same positive target-derived work `W`, a
branch at height `h` has total `(h + 1) * W`, including the shared genesis. For
two branches under that identity, comparison of those totals is exactly comparison
of their heights. This is an ordering argument; the implementation does not compute
that product or claim to measure the actual number of attempts a miner performed.

An unusually low individual block hash, a larger nonce, a later timestamp, more
transactions, higher membership/population counters, richer accounts or a larger
local storage generation supplies no extra score. Two equal-height branches have
equal score regardless of those values. Easier fixed work does not make devices
equally fast or establish one-human-one-vote consensus; the later human consensus
profile remains a separate milestone.

The rule applies only to this fixed-target development profile. Introducing
variable-target acceptance would require an explicitly reviewed replacement for
this scoring contract; this comparator must not silently rank such histories by
height. A different configuration or genesis already rejects under the identity
checks, rather than being compared by nominal height across networks.

## 4. Common ancestry and branch relation

Both accepted branches contain the same independently selected genesis. The
comparator walks their accepted block hashes from height zero upward, stopping at
the first unequal hash or the end of the shorter branch. The last matching block
is the common ancestor. It returns that block's complete checkpoint: C, selected
genesis, actual height, actual block hash and accepted ledger root.

An account root, membership root, timestamp or matching tip height is not a
substitute for matching accepted block hashes. In particular, two branches can
have identical account bytes and different membership histories. The existing
hash and signature assumptions remain in force; this routine does not establish
collision resistance itself.

Relations and decisions are:

| `BranchRelationV2` | Meaning | `ForkChoiceV2` |
|---|---|---|
| `Identical` | All accepted hashes match and both branches end together | `KeepCurrent` |
| `CandidateExtendsCurrent` | Current is an exact proper prefix of candidate | `PreferCandidate` |
| `CurrentExtendsCandidate` | Candidate is an exact proper prefix of current | `KeepCurrent` |
| `Diverged` | Both continue beyond the common ancestor on different branches | `PreferCandidate` only if candidate score is strictly greater; otherwise `KeepCurrent` |

The common-ancestor checkpoint is the shared tip for identical branches, the
shorter tip for a proper prefix, or the final shared block for divergent branches.
Neither the relation nor the choice mutates either input.

## 5. Equal-score behavior and finite limits

A tie keeps the input designated as current. This prevents switching solely
because an equal-score competitor was presented. The result is deterministic
for the ordered `(current, candidate)` pair, but it is not a global total ordering
of equal-score branches. Reversing two equal-score divergent inputs retains the
other branch because that branch is now current.

Two peers can therefore keep different equally strong branches until a strictly
stronger accepted candidate is available. There is no tie breaker based on hash
luck, arrival timestamps supplied by a peer, membership count or an administrator.
At the finite chain cap, or when a tip has terminal timestamp `u64::MAX`, further
extension might be impossible. This component provides no guarantee that such a
tie will eventually resolve and establishes no finality or partition-liveness rule.

Comparison uses at most 4,096 block pairs and 8,192 fixed-size header hashes. It
uses constant additional state, allocates no heap collection, clones no branch,
and replays no ledger. Each input was already admitted under the independent
4,096-block, 16-MiB encoded-history and 1,048,576 cumulative state/configuration
item limits. Those remain per-input limits; comparing two accepted inputs does
not merge or relax their budgets. This is not a total process-memory bound.

## 6. Publication remains a separate operation

The selected chain reference includes the complete state associated with that
branch. Selection cannot preserve orphaned approvals, active statuses, sequences,
population peaks or resource counters from the other branch. Existing state roots
and full-history validation continue to define each candidate's executed state.

Nevertheless, this result is an in-memory comparison, not a durable commit token.
Public enum values and copied checkpoints are diagnostic data. They can be saved
or become stale after the borrowed comparison is no longer used; they do not
authorize a later store mutation on their own. There is no serialized comparison
record, mutable selected-chain accessor or API that adopts an arbitrary reported
score.

A future durable replacement API must retain exclusive store ownership and
revalidate the expected network, current selected endpoint and exact candidate
under that lock before publication. It must define its own interruption and retry
rules and preserve the stable root inode. It must not turn MSU2 generation into
branch weight, interpret a peer's preferred tip as validated history, or reuse
one-child append/recovery as an implicit reorganization procedure.

The implemented [immutable store](STORAGE_V2.md), [append profile](STORAGE_APPEND_V2.md)
and [explicit recovery](STORAGE_RECOVERY_V2.md) keep their existing APIs and strict
selection requirements. This comparison performs no filesystem operation and
changes none of their ownership or recovery assumptions.

## 7. Verification obligations and continuation

Acceptance evidence must identify actual tests and exact published source in the
checkpoint records. Required cases include:

- Identity rejection on either input side, with the documented error order.
- Genesis-only and identical histories, proper prefixes, extensions, and genuine
  divergent branches at shorter, equal and longer scores.
- Sticky ties in both argument orders, common ancestors at genesis and later
  heights, and the finite chain/timestamp boundaries.
- Frozen membership-divergent histories whose account bytes agree, with selection
  preserving the complete chosen membership and counters.
- No input mutation, forged score path, raw-block acceptance path or publication
  side effect; Rust ownership checks for the borrowed result.

Existing independent encoding/reference fixtures remain frozen. No new encoded
selection or score format is introduced by this contract. Existing workspace,
doctest, release and reference gates still apply, together with the synthetic
fork-comparison example and new targeted tests.

The next slice is durable membership-aware branch replacement, followed by
fresh-peer reconstruction. MB-03 as a whole remains unfinished. Persistent V2
node/pool/wallet/operator integration, human consensus, policy-gated issuance and
fees, private biometric matching and actual fingerprint/ECG inputs remain later
work. Gate A and Gate B are not established by this comparison component.
