# V2 network manifest, block acceptance and complete replay

Contract: **MIRROR-MB02C-001**, synthetic development profile, 2026-11-20.
Implementation: `mirror_chain::v2` and `mirror_consensus::v2`.
This contract composes [MCF2 configuration](CONFIG_V2_ENCODING.md),
[MRB2 blocks](CORE_V2_ENCODING.md), [signed operations](OPERATIONS_V2_ENCODING.md)
and the [atomic ledger](LEDGER_STATE_V2.md). Their frozen bytes and domains remain
unchanged. It does not migrate or reinterpret legacy blocks or data directories.

## 1. Authority and scope

`ChainV2` owns a finite, fully validated branch, its complete executed ledger, and
an immutable network manifest. Its private accepted blocks supply every ancestry
lookup used by its own validation. An external `ParentChainViewV2`, peer assertion,
state snapshot, reported height or claimed root cannot replace that ancestry.

There are two explicit choices outside untrusted input:

- Network selection: configuration hash **C** and the exact genesis block hash.
- Replay selection: C, genesis hash, final height, final block hash and ledger root.

These choices identify the intended network and finite branch. They do not discover
the latest global tip, establish finality, choose a canonical competing branch or
prove human uniqueness. Choosing both expectations from the untrusted input would
remove this protection. An application must retain its trusted selection separately.
`ChainV2` remains an in-memory acceptance API. The separate implemented
[MB-03A storage layer](STORAGE_V2.md) initializes one immutable selected history,
reopens it under exclusive ownership, or inspects it under shared ownership.
Reopening requires independent network and full checkpoint pins and reconstructs
every complete MRB2 block through this API. The separate
[MB-03B append profile](STORAGE_APPEND_V2.md) now publishes one accepted child at a
time, fences writes after failures, and replays/resynchronizes clean selected
states. Its local generation also requires independent selection. MB-03C's
[explicit recovery API](STORAGE_RECOVERY_V2.md) replays the parent and accepts the
exact independently retained child through these same rules before either completing
that candidate or retaining its still-selected parent. It does not select a branch.

## 2. Public network manifest — MNM2

All integers are unsigned little-endian. Hashes and signatures use their exact
stored byte order, with no reversal. Fields appear in this exact order:

| Field | Bytes | Meaning |
|---|---:|---|
| Magic | 4 | ASCII `MNM2` |
| Manifest version | 2 | `2` |
| Configuration length | 4 | Length of the complete following MCF2 encoding |
| Configuration | variable | Exact canonical MCF2 bytes, including seed order and allocations |
| Seed-proof count | 4 | Exactly the configured bootstrap descriptor count |
| Ordered seed proofs | `64 * count` | One complete Ed25519 signature for each descriptor, in configuration order |
| Genesis hash | 32 | Exact selected V2 genesis block hash |

Total size is `46 + configuration_length + 64 * proof_count`. MCF2 is at most
2,717,590 bytes; at most 1,000 seed proofs are accepted. Consequently the absolute
manifest bound is **2,781,636 bytes**. Trailing bytes, truncation, unknown magic or
version, oversized lengths/counts, and missing/extra/reordered/invalid proofs
reject. No fallback format or optional unchecked seed exists.

`NetworkManifestV2::new(config, proofs, genesis_hash)` explicitly selects a network
and verifies **every** ordered seed proof under that complete configuration. It
does not accept a genesis block or attest to the biological identity of a seed.
`decode_for(bytes, ManifestIdentityV2 { config_hash, genesis_hash })` requires the
independently selected network identity. It checks the entire framing/count/size
budget before configuration/proof allocations and signature work, checks the
external genesis pin, decodes MCF2, checks C, and verifies every seed proof.

The manifest is public network metadata. It contains no private keys, fingerprint
or ECG archive, custody URL, biometric template or executable economic policy.
It adds no manifest hash domain: the selected identity is the pair `(C, genesis)`.
There is no public unpinned decode API and no manifest setter on an accepted chain.

## 3. Exact genesis

`genesis_template_v2(VerifiedBootstrap)` creates an unaccepted nonce-zero proposal.
Its roots are calculated from the verified bootstrap and the existing encoders.
Work search and explicit manifest selection precede `ChainV2::from_genesis`.

| Header/body field | Required value |
|---|---|
| Version | V2 (`2`) |
| Configuration | Exact C from the selected manifest |
| Height | `0` |
| Previous hash | 32 zero bytes |
| Operations | Empty; configured allocations/seeds are not genesis operations |
| Operations root | The existing V2 empty-operation root |
| Ledger root | Independently executed `LedgerV2::bootstrap(manifest.bootstrap())` root |
| Timestamp | Exact configured `genesis_timestamp` |
| Difficulty bits | Exact configured `pow_bits` |
| Nonce | Any value satisfying work **and the exact selected genesis hash** |

Acceptance preflights body and chain budgets, checks context/height/parent/time and
empty body, validates the exact target and actual work, compares the manifest hash,
checks the operation root, executes bootstrap, and compares the ledger root before
binding that actual block hash. Even a deliberately pinned malformed genesis must
pass every check. Another correctly mined nonce is a different genesis and rejects.
An accepted chain always contains at least this one block.

## 4. Fixed-target V2 work

`validate_header_pow_v2(header, expected_bits)` requires **literal equality** between
header bits and trusted configured bits. It then uses the existing compact-target
decoder and compares:

```text
big_endian_integer(SHA256d("MIRROR_BLOCK_V2" || exact_160_byte_header)) <= target
```

No digest reversal is applied. Existing compact positive/noncanonical forms that
MCF2 already accepts remain supported when configured; another compact encoding of
the same numeric target is still rejected if its literal bits differ. A miner
cannot select an easier target in a child block. There is no retargeting here.

`mine_header_v2(header, expected_bits, max_attempts)` searches consecutive nonces,
including the input nonce, and changes no other field. Its explicit local budget
must be in `1..=1_000_000`. A valid solution on the last allowed attempt or at
`u64::MAX` succeeds. After a failed `u64::MAX` attempt, nonce exhaustion wins over
budget exhaustion; nonce never wraps. Callers can make another bounded search
request. This helper cap is not a consensus limit on other miners' attempts.
`MineResultV2` is data, not a certificate; acceptance verifies the work again.

Easy fixed work is a development rule. It is not encryption, equal hardware
participation, human voting, economic reward assignment or final human consensus.

## 5. Child validation and atomic publication

`append(BlockV2)` performs these steps in order:

1. Preflight all full-body operation/count/byte limits and cumulative branch work.
2. Check version/C, checked next height, actual accepted parent hash, and timestamp.
3. Check exact configured bits and actual work over the complete header.
4. Recompute and compare the ordered full signed-operation commitment.
5. Stage the **original complete body** through `LedgerV2::stage_operations`, using
   only this branch's accepted ancestry. Signatures, contexts and state rules run
   in original operation order; no separate signature pass masks an earlier error.
6. Compare the complete calculated accounts/membership/economics ledger root with
   the claimed header root. Valid work cannot excuse a wrong root.
7. Reserve the block slot, bind the successful staged ledger to the actual hash,
   and install the block, ledger and resource counters together.

Every returned error leaves accepted blocks, ledger, manifest and resource counters
unchanged. No incomplete candidate is exposed. This is in-memory failure atomicity,
not a disk transaction or a claim that allocation failure/process termination is
recoverable. Infallible standard-library allocations can still terminate a process.

`prepare_candidate(timestamp, operations)` checks the same body/state budgets and
stages the complete body against the accepted parent. It returns an unaccepted
nonce-zero block with calculated roots, and does not mutate the chain. Even after
successful mining, `append` must validate it independently. Original positions
include transfers; neither filtering into MPH2 nor renumbering membership is valid.
Empty accepted blocks still process expiry, height and the combined state root.

**Timestamp rule:** a child timestamp must be strictly greater than its parent's.
Genesis must match configuration exactly. No local clock, median-time calculation
or future-time ceiling is consulted. Thus a valid accepted timestamp of `u64::MAX`
admits no successor timestamp. This deliberately finite synthetic rule is not a
production liveness/time-manipulation policy; that policy requires explicit review.

## 6. Complete replay and immutable ancestry

`ChainV2::replay` accepts typed full blocks. `replay_encoded` accepts slices of complete
MRB2 bytes and uses the bounded MRB2 decoder. Neither accepts MPH2 membership-only
groups or an MPS2 cache as authoritative ledger state. Both rebuild from the selected
manifest/genesis through the same `from_genesis` and `append` validation path.

The required `ChainReplayExpectationV2` has exactly five fields:

```text
config_hash, genesis_hash, height, tip_hash, ledger_root
```

Replay bounds total count and all complete input lengths before input-sized cloning
or decoding. It verifies expected C/genesis and the implied final height, validates
each actual block in order, then compares final tip/root. A bad intermediate block,
late wrong root, incomplete prefix or wrong expected branch returns no chain. A
valid shorter prefix still rejects against a selected later checkpoint. This does
not prevent rollback if an application itself supplies an obsolete expectation.

`checkpoint()` reads the fields from an already accepted branch. It cannot establish
external freshness. Parent lookup uses the private block at each real height in
that candidate branch; another branch's operation anchor fails even if the header
parent, work and signature otherwise pass. Two separately accepted branches can
be valid simultaneously. MB-03A can persist and reopen either explicitly selected
immutable branch; MB-03B can append accepted children in its separate versioned
store and reconcile clean selected states. Neither supplies global fork choice,
branch replacement or automatic recovery of pending artifacts. MB-03C separately
handles bounded recognized staging under an explicit retained request; a pending
file never chooses authority. MB-03D next specifies canonical development fork
choice, before durable membership reorganization and peer reconstruction. The later
human-finality profile remains MB-06.

## 7. Explicit finite resource profile

All paths retain configured per-block/state caps. This chain adds independent caps:

| Resource | Inclusive maximum | Accounting |
|---|---:|---|
| Blocks | 4,096 | Includes genesis and empty blocks |
| Encoded history | 16,777,216 bytes (16 MiB) | Sum of full canonical MRB2 sizes |
| State/configuration work | 1,048,576 items | Checked cumulative charge below |

Let `S` be configured seed-descriptor count and `A` allocation-entry count.
The genesis charge is `2 * (S + A)`. Each child charges:

```text
S + A
+ current account count (including zero balances)
+ current membership record count
+ current retained approval count
+ one item for each incoming Transfer, Enroll or Approve
```

An incoming Withdraw adds no prospective item. Charges deliberately remain
conservative when a transfer creates no account or an operation removes activity.
They are an abstract bounded-work proxy, not exact allocation counts or measured
CPU. Arithmetic overflow and cap exhaustion reject; charges commit only after
successful acceptance. Each transition checks its charge before expensive state
staging, including empty-block configuration/account/record copying.

There is no eviction, identity-history deletion, automatic cap increase or partial
replay success. A chain reaching a cap stops growing in this development component.
These are not human population limits, monetary ceilings or a billion-user design.
Changing them or introducing checkpoints/pruning requires a reviewed profile/version
contract. Existing B4 diagnostic replay caps remain separate and unchanged.

## 8. Independent evidence and remaining work

`tools/reference_chain_v2.py` independently constructs exact public seed proofs,
signed actual-ancestor operations, manifest bytes, mined blocks and executed states.
Its fixture was frozen before the Rust vector tests were first run. Normal `--check`
uses only the Python standard library; optional `--verify-signatures` independently
regenerates all 20 signatures with Python cryptography. Prior fixtures are untouched.

The fixed history contains five main-branch blocks, an empty expiry block, an
alternate valid genesis, valid work over a wrong ledger root, and a genuine fork
whose reviewer withdrawal changes membership while accounts match. Rust compares
full bytes, component/combined roots, ancestry, replay and unchanged state on error.
Separate manifest, work and chain tests exercise malformed input and exact/+1 caps.
Resource arithmetic tests are not a maximum-sized replay stress benchmark.

`chain_v2_demo` runs two separate in-memory validators against the same actual
mixed blocks, demonstrates invalid-root rollback and complete MRB2 replay. It does
not run a multi-process network or write a durable chain. Acceptance evidence and
exact tested source/CI are tracked in CURRENT_STATUS.md and MASTER_PLAN.md.

The implemented [MB-03A contract](STORAGE_V2.md) adds separate explicit storage
initialization, exclusive reopening and shared read-only inspection. Its
generation-zero MSS2 record is selected by independent C/genesis/height/tip/root
expectations; exact MNM2 and complete MRB2 bytes are retained and fully replayed.
It does not install account or membership caches as authority or change the
finite resource profile above. Current source verification and publication
evidence remain in the repository checkpoint files.

The separate [MB-03B contract](STORAGE_APPEND_V2.md) adds MSU2 profile 2 with an
independently selected local generation. Each append stages through `ChainV2`,
checks the complete current disk history, and advances memory only after block
and selector publication have been synchronized. Failed writes fence the handle;
writer reopen or consuming reconciliation fully replays and resynchronizes a clean
selected state before allowing another update. Ordinary entry points preserve and
reject partial staging. The [MB-03C recovery contract](STORAGE_RECOVERY_V2.md) adds
explicit Complete/RetainParent actions for one independently authorized child;
both fully replay the parent and validate the child before mutation. Final strict
replay reproduces the complete selected ledger and counters. Unknown data remains
rejected, and no storage operation changes these chain rules.

**Next is MB-03D:** canonical development fork-choice specification and implementation.
Durable membership-aware branch replacement and peer reconstruction follow; the
V2 node/API remains later work.
No founder or marketing reserve, grant, mint, fee/reward formula, real biometric
enrollment or device integration is enabled by these contracts. D-01/D-04 remain
open and Gate A/B remain unmet.
