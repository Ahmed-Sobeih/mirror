# V2 branch replacement: selection metadata and preparation

**MB-03E1. Updated 2026-11-26.** This contract freezes the MSR2 metadata format and
the pure, checked preparation API. It does **not** implement a replacement store,
filesystem layout, durable publication, on-disk intent or recovery procedure.
Those are the next increment, MB-03E2. Current acceptance evidence belongs to
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).

The existing [immutable store](STORAGE_V2.md), [append store](STORAGE_APPEND_V2.md)
and [explicit append recovery](STORAGE_RECOVERY_V2.md) keep their exact APIs,
encodings and behavior. MSR2 is separate profile 3; it does not turn either prior
store into a branch-replacement store. No implicit migration is introduced.

## 1. Authority and scope

The [accepted-chain contract](CHAIN_V2.md) establishes actual complete block and
ledger validity. The [development comparator](FORK_CHOICE_V2.md) checks two
accepted branches against independently chosen configuration/genesis identity,
finds their actual common ancestor and prefers a strictly higher candidate under
the fixed-target development rule. Every tie keeps the current branch.

MB-03E1 adds two boundaries:

1. `ReplacementSelectionV2` is bounded metadata for one selected snapshot and its
   local publication generation. Decoding requires an independent full checkpoint.
2. `ReplacementPlanV2` checks the complete current metadata against an accepted
   current chain and prepares canonical metadata for an accepted, preferred
   candidate. It retains immutable references to both complete histories.

Neither boundary acquires storage ownership or selects a globally canonical
history. Checksums are public integrity calculations, not signatures or selection
authority. A caller who derives its expected checkpoint from the record being
checked removes that selection protection. Independent pins do not themselves
discover global latestness or finality.

## 2. MSR2 encoding

MSR2 is exactly **280 bytes**. Integers are unsigned little-endian; hashes use
their raw byte order. The checksum domains below contain no terminating NUL.

| Offset | Field | Bytes / required value |
|---|---|---|
| 0 | Magic | 4 bytes `MSR2` |
| 4 | Encoding version | u16 = 2 |
| 6 | Storage profile | u16 = 3 |
| 8 | Local publication generation | u64 |
| 16 | Exact nested MSS2 snapshot | 232 bytes, unchanged version 2/profile 1/generation 0 |
| 248 | Outer checksum | 32 bytes: `SHA256d("MIRROR_STORAGE_REPLACEMENT_SELECTION_V2" || bytes[0..248])` |

The nested MSS2 keeps its existing layout and checksum domain. It includes the
full configuration/genesis/height/tip/ledger-root checkpoint, block count, exact
manifest length and checksum, and complete MRB2 byte total. No old byte range,
wire encoding, signing hash, ledger root or generation meaning is repurposed.
Bare MSS2 and MSU2 are not accepted by the MSR2 decoder, and neither prior decoder
accepts MSR2.

`ReplacementCheckpointV2` is independent caller input with this shape:

```rust,ignore
ReplacementCheckpointV2 {
    generation: u64,
    chain: ChainReplayExpectationV2,
}
```

Initialization metadata uses generation zero at any accepted height. Under this
profile, a successful strictly preferred replacement would advance generation by
one and height by at least one. A multi-block extension or a divergent replacement
counts as one publication. Therefore `generation <= height` is necessary. It does
not prove that any publication occurred. Construction from a chain accepts a
bounded caller-supplied generation; it does not invent a durable history.

The inner block count remains in 1..=4096, so accepted metadata height and generation
are at most 4095. Equal-height replacement, human-weighted selection or a change
to these limits requires an explicitly reviewed profile; do not silently change
this invariant or treat generation as chain score.

### Decoder checks

`ReplacementSelectionV2::decode_for(bytes, expected)` checks, in order:

1. Exact length, magic, encoding version and storage profile.
2. The complete outer checksum.
3. The unchanged MSS2 decoder, including its own checksum, all semantic bounds
   and every independently supplied `expected.chain` field.
4. Generation not exceeding the nested selected height.
5. Equality with `expected.generation`.

Every fixed-offset read follows the exact-length guard. The decoder performs no
input-sized allocation, manifest decoding, signature verification or block replay.
There is no unpinned decoder or public record-field mutation API.

`verify_manifest_bytes` checks exact length and SHA256d against the nested record.
This alone does not validate MNM2 seed proofs or any block. Full actual-chain replay
remains mandatory before treating a disk history as an accepted chain.

## 3. Checked in-memory preparation

```rust,ignore
prepare_replacement_v2(
    identity: ManifestIdentityV2,
    expected_current: ReplacementCheckpointV2,
    current_selection: &ReplacementSelectionV2,
    current: &ChainV2,
    candidate: &ChainV2,
) -> Result<ReplacementPlanV2<'_>, ReplacementPlanErrorV2>
```

The function performs these checks in order:

1. Reuse the accepted-branch comparator: current configuration/genesis, then
   candidate configuration/genesis, real common ancestry and fixed-target scores.
   Its ancestry scan is bounded even when a later current-pin check rejects.
2. Compare the actual current chain's complete checkpoint with the independently
   retained expected current chain checkpoint.
3. Compare the supplied current selector's complete checkpoint and generation
   with the same independent expected selection.
4. Derive canonical metadata from the actual current chain and expected generation;
   require equality of the **entire** supplied selector. Matching endpoint fields
   cannot conceal altered manifest length/checksum or block-byte totals, even if
   both public checksums were recomputed successfully.
5. Require `PreferCandidate`. Identical histories, ties and shorter candidates
   reject without advancing generation.
6. Increment generation with checked arithmetic, then derive the candidate's own
   canonical selector from the entire accepted candidate.

The candidate selector retains the candidate's exact manifest metadata. Matching
network identity is not permission to substitute the old branch's exact manifest
bytes. Complete MRB2 history, accounts, membership records, retained approvals,
population high-water values and resource accounting all remain those of the
candidate; no state is grafted from the old branch.

The resulting plan has private fields and no public constructor, decoder,
serialization or mutation operation. It exposes the independent identity, branch
relation, accepted common ancestor, both checkpoints/selectors and immutable
references to the exact current and preferred candidate chains. The borrows
prevent either chain from changing while the plan is still used. A copied
checkpoint or record is diagnostic metadata, not a durable publication permit.

Successful preparation encodes two bounded manifests while deriving metadata.
It does not clone the chains, reexecute them, allocate an input-sized selector
buffer or access the filesystem. It is not allocation-free. `GenerationOverflow`
is a defensive error, unreachable from valid accepted inputs under the current
4095 height/generation cap; tests do not claim a valid-chain overflow transition.

## 4. Reference and verification boundaries

`tools/reference_replacement_storage_v2.py` independently constructs MSR2 with
Python's standard library, using pinned preexisting chain and MSS2 vectors rather
than new Rust output. Nine cases cover genesis, ordinary generations, nonzero-height
initialization, a divergent branch and a multi-height replacement at generation 1.
The exact fixture is `crates/mirror-storage/tests/vectors/replacement_storage_v2.txt`.

Codec tests compare those bytes and exercise malformed framing, nested limits,
independent pins, legacy domains/profiles and checksum-correct semantic corruption.
Preparation tests use fully accepted actual histories, including equal account
bytes with different membership outcomes. Compile-fail examples check both borrowed
chain lifetimes. The synthetic `replacement_plan_v2_demo` prepares a genuine fork
replacement, round-trips the new metadata and performs no filesystem publication.

```bash
python3 tools/reference_replacement_storage_v2.py --check
cargo test -p mirror-storage --test replacement_selection_v2 --locked
cargo test -p mirror-storage --test replacement_plan_v2 --locked
cargo test -p mirror-storage --doc --locked
cargo run -p mirror-storage --example replacement_plan_v2_demo --locked
```

## 5. Next: MB-03E2 durable replacement and recovery

The following are requirements for the next design and implementation, **not APIs
or filesystem behavior implemented by E1**:

- Keep one stable root directory inode and cooperative exclusive ownership through
  revalidation, staging, selection publication, recovery and memory advancement.
  Never rename or replace the locked root to switch branches.
- Revalidate independent network identity, full selected checkpoint/generation,
  exact selected disk history and the actual candidate under that ownership.
  The E1 plan cannot authorize writes after an unrelated stale-disk check.
- Stage the candidate's complete exact manifest and accepted block history; sync
  the files and their containing directories before publishing a selector. Advance
  the in-memory selected state only after publication and root synchronization.
- Freeze a bounded layout, publication linearization point, allowed interrupted
  states, write fencing and explicit retry/recovery actions before implementation.
  Decide whether to use two reusable slots or bounded retained-generation directories.
  E1 commits to neither approach and assigns no path to a generation.
- Define cleanup interruption states and what retained history can be removed.
  Do not let a directory scan or the numerically greatest generation choose a branch.
  Unknown/corrupt bytes must remain preserved and rejected by ordinary opening.
- Recovery must require independently retained current/candidate pins and exact
  candidate history, with complete validation before mutation. A selected new
  generation cannot be rolled back by a retain-old action. Keep recovery ownership
  continuously; return no usable writer after an unresolved error.
- Specify aggregate enumeration, input, read/work and staging bounds. Existing
  chain caps remain 4096 blocks, 16 MiB of MRB2 and 1,048,576 cumulative state/config
  items. One manifest + history + MSR2 has a maximum encoded payload of 19,559,132
  bytes. Two manifests, two complete histories and two selectors total 39,118,264 bytes. These are
  planning arithmetic, not an implemented E2 disk/read/RSS budget; extra staging,
  repeated reads and cleanup must be accounted for in the final design.
- Exercise real divergent membership histories, failure at each publication stage,
  both authorized recovery outcomes, retries, no mixed-state exposure and preserved
  old-format compatibility. Then proceed to fresh-peer reconstruction.

MB-03, persistent V2 node integration and Gate A/B remain unfinished. This increment
does not change human enrollment, biological matching, money issuance, reward rules,
founder policy or the final human-weighted consensus design.
