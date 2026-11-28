# V2 bounded peer-history framing and candidate reconstruction

Contract: **MIRROR-MB03F1-001**, synthetic development profile, 2026-11-26.
Implementations: [`mirror_protocol::v2`](../crates/mirror-protocol/src/v2.rs) and
[`mirror_network::v2`](../crates/mirror-network/src/v2/mod.rs).

This contract adds one finite framed transfer and complete in-memory validation of
its candidate branch. It composes the existing [MNM2 manifest and accepted
chain](CHAIN_V2.md), [MCF2 configuration](CONFIG_V2_ENCODING.md), [MRB2 block
encoding](CORE_V2_ENCODING.md), and [combined ledger](LEDGER_STATE_V2.md).
**MB-03F2 TCP sessions and durable adoption are not implemented by this increment.**

## 1. Authority and scope

The caller independently selects `ManifestIdentityV2 { config_hash, genesis_hash }`
before creating a receiver. The configuration hash is called **C** below. Neither
pin may be chosen from the peer's Begin frame, manifest, blocks, or End frame.

Every on-wire summary field is initially an **untrusted transfer claim**. A
reported height, tip, root, byte count, or matching Begin/End pair cannot select
the network, establish freshness, or authorize storage replacement. The receiver
checks C/genesis against the independent pins and reconstructs the actual branch
from its complete genesis and children. It only releases a candidate after every
block and the explicit End have passed validation and exact accounting.

`ReceivedCandidateV2` means that this finite branch has been fully executed and
matches the claims for this transfer. A valid shorter branch, or a valid branch
that would lose a fork comparison, can still be received successfully. The
receiver has no current canonical chain against which to compare it. Separate
[fork choice](FORK_CHOICE_V2.md), independently selected storage state, and
[durable replacement](STORAGE_REPLACEMENT_STORE_V2.md) remain application work.
The peer's claims must never be converted directly into a trusted
`ChainReplayExpectationV2` or local generation selection.

The component opens no sockets, selects no peer, changes no store, issues no
currency, and captures no device readings. Easy fixed work remains the existing
development validation rule; framing does not change the intended later human
consensus design or establish biological uniqueness.

## 2. Exact frame encoding — MPN2

All integers are unsigned little-endian. Hashes retain their exact stored byte
order with no reversal. Byte intervals below include their start and exclude
their end. Every frame has this **12-byte header**, immediately followed by the
declared payload, with no padding, checksum, or optional fields:

| Header interval | Bytes | Field | Required value or meaning |
|---|---:|---|---|
| `[0, 4)` | 4 | Magic | ASCII `MPN2` |
| `[4, 6)` | 2 | Protocol version | `u16`, exactly `2` |
| `[6, 8)` | 2 | Kind | `u16`, one of the values below |
| `[8, 12)` | 4 | Payload length | `u32`, excludes the 12-byte header |

| Kind | Value | Exact payload | Permitted payload size |
|---|---:|---|---:|
| `Begin` | 1 | One `HistorySummaryV2` | 152 bytes |
| `Manifest` | 2 | Entire canonical MNM2 encoding | 46 through 2,781,636 bytes |
| `Block` | 3 | `u64` height followed by one complete MRB2 encoding | 176 through 1,048,584 bytes |
| `End` | 4 | One `HistorySummaryV2` | 152 bytes |

Begin and End each occupy 164 bytes including their header. A Manifest frame
occupies `12 + M` bytes. A Block frame occupies `20 + L` bytes, where `L` is the
complete MRB2 length, including MRB2's own magic/version/header/body. Its maximum
total frame size is therefore 1,048,596 bytes. The outer block height must be less
than 4,096; the receiver additionally requires the exact next height and agreement
with the decoded MRB2 header.

`FrameHeaderV2` has private kind/length fields. Its constructor and exact-header
decoder validate the version, known kind, and per-kind size bounds before any
payload work. A header slice must be exactly 12 bytes. Unknown versions/kinds,
wrong magic, and out-of-range lengths reject; there is no fallback or negotiation
into another encoding.

`decode_frame_v2` accepts exactly one complete frame. Truncation, trailing bytes,
or a second concatenated frame reject. `decode_payload_v2` accepts exactly the
payload length from a previously checked header. `FrameRefV2` borrows Manifest and
Block bytes directly from its input; decoding framing does not copy those payloads
or validate their semantic contents. `encode_frame_v2` validates the frame shape
and uses a checked exact reservation before producing its output buffer.

In particular, structurally sized opaque bytes can pass the framing codec and
still fail manifest or block validation. Protocol decoding is never chain
acceptance.

## 3. Exact summary encoding and claims

Begin and End use the same **152-byte** payload, with fields in this exact order:

| Payload interval | Bytes | Field | Encoding or meaning |
|---|---:|---|---|
| `[0, 32)` | 32 | `config_hash` | Claimed C |
| `[32, 64)` | 32 | `genesis_hash` | Claimed exact genesis hash |
| `[64, 72)` | 8 | `height` | `u64`, claimed terminal height |
| `[72, 104)` | 32 | `tip_hash` | Claimed terminal block hash |
| `[104, 136)` | 32 | `ledger_root` | Claimed executed terminal ledger root |
| `[136, 140)` | 4 | `block_count` | `u32`, includes genesis |
| `[140, 148)` | 8 | `block_bytes` | `u64`, sum of complete MRB2 lengths |
| `[148, 152)` | 4 | `manifest_bytes` | `u32`, complete MNM2 length |

There is no local storage generation, score, proof-of-personhood certificate,
snapshot authority, or trusted checkpoint field. Let `N = block_count`,
`B = block_bytes`, and `M = manifest_bytes`. Structural summary validation requires:

- `1 <= N <= 4,096` and checked `height + 1 == N`.
- `46 <= M <= 2,781,636`.
- `168 * N <= B <= min(1,048,576 * N, 16,777,216)`.

Multiplications use checked `u64` arithmetic before aggregate caps or conversion
to `usize`; intermediate products must not wrap on a 32-bit target. Summary
decoding requires exactly 152 bytes. These bounds do not prove that any manifest
or valid chain realizes the claimed shape.

The End summary must equal the accepted Begin summary field for field. Before
End is permitted, the reconstructed branch must already match the claimed count,
MRB2-byte total, C, genesis hash, height, tip hash, and ledger root. These checks
run again at End, along with exact transfer frame/byte accounting.

## 4. Consuming receiver and validation sequence

`HistoryReceiverV2::new(independent_identity)` creates one attempt. Its private
state advances in this order:

1. **Begin:** validate structural claims and require the independently selected
   C/genesis. The following frame must be Manifest.
2. **Manifest:** require its exact declared length, then call
   `NetworkManifestV2::decode_for(bytes, independent_identity)`. This checks the
   actual canonical configuration, selected network, and ordered seed proofs.
   Apply the tighter configured total-byte bound described below.
3. **Genesis:** require a Block payload containing height zero and exactly the
   168-byte empty canonical genesis block. Decode with `BlockV2::decode` under
   the accepted configuration, require agreement between outer and inner height,
   and call `ChainV2::from_genesis`.
4. **Children:** accept exactly the next height on every subsequent Block. Decode
   the complete MRB2 under that same configuration and pass it to `ChainV2::append`.
   All actual parent/work/body/signature/ancestry/membership/economics/ledger-root
   checks remain mandatory. On the claimed last block, check the computed endpoint
   and exact count/bytes before advancing to End.
5. **End:** require the identical summary, recheck the executed endpoint, and
   require the precise final frame count and wire length. Mark the attempt Complete.
6. **Finish:** consuming `finish()` releases `ReceivedCandidateV2` only from
   Complete. Earlier calls return `IncompleteHistory` and discard the attempt.

The receiver reconstructs through `decode_for`, `from_genesis`, and `append`.
It does **not** manufacture a replay expectation from the peer's summary and
then treat that expectation as independent authority. The accepted chain retains
its own actual ancestry and combined executed ledger throughout validation.

### Header-first ownership

`begin_frame(self, header_bytes)` consumes the current receiver. It checks the
exact header, allowed phase, relevant exact/configured lengths, and remaining
transfer budgets. On success it returns `PendingHistoryFrameV2`, which privately
owns the receiver and checked header. Its `payload_len()` is the approved length
that a future transport may use when allocating/reading this payload.

`PendingHistoryFrameV2::accept_payload(self, bytes)` requires that exact payload
length, decodes it, executes the authorized transition, and returns the next
receiver. Callers must retain the returned owner after each success. Neither
receiver nor pending permit is Clone, and neither exposes a partial-chain getter.
Only `ReceivedCandidateV2` exposes `chain()`, `into_chain()`, the now-checked
summary, and final frame/byte accounting.

Every returned header, payload, or finish error consumes the attempt and returns
no continuation or partial candidate. Dropping a pending permit similarly drops
the partial state; a future I/O adapter can use this on read failure. It must not
skip an invalid frame, resume the same failed attempt, or scan arbitrary input
for another magic value. `accept_frame(self, bytes)` is a convenience for an
already bounded complete frame; future socket adapters must use `begin_frame`
before allocating an attacker-declared body.

**End is the explicit transfer boundary.** After End succeeds, the caller must
invoke `finish` to consume the receiver and obtain its candidate. Any additional
input instead rejects with `AlreadyComplete` and consumes the attempt, leaving
no receiver to finish. After a successful finish no receiver remains either.
Missing End cannot become success merely because all claimed blocks arrived.
This component neither reads nor requires a
socket EOF and does not specify TCP session termination, trailing socket traffic,
timeouts, deadlines, or resumption. Those are MB-03F2 responsibilities.

## 5. Transfer, chain, and future-byte limits

Exactly one successful transfer has `N + 3` frames: Begin, Manifest, all `N` blocks
starting at genesis, and End. Its complete wire length is:

```text
W = 2 * 152 + M + B + N * (12 + 8) + 3 * 12
  = M + B + 20 * N + 340
```

The inclusive absolute limits are:

| Resource | Maximum | Meaning |
|---|---:|---|
| Frames | 4,099 | Includes Begin, Manifest, genesis/children, End |
| Full MRB2 history | 16,777,216 bytes | Includes every encoded block; excludes MPN2 overhead |
| Manifest | 2,781,636 bytes | Entire MNM2 payload |
| Complete transfer | 19,641,112 bytes | `2,781,636 + 16,777,216 + 20 * 4,096 + 340` |
| Chain state/configuration work | 1,048,576 items | Existing cumulative accepted-chain work proxy |

The last limit is maintained by `ChainV2`, using the genesis and per-child charges
defined in [the chain resource profile](CHAIN_V2.md#7-explicit-finite-resource-profile).
Configured per-block, account, membership, and operation caps also remain in force.
The wire maximum does not imply an accepted maximum-sized branch exists, raise
any chain cap, or provide pruning/checkpoint shortcuts.

After decoding the manifest, let `Lmax` be its configured `max_block_bytes`.
Genesis is exactly 168 bytes, so the receiver additionally requires:

```text
B <= 168 + (N - 1) * Lmax
```

Before approving each Block payload, let `K` be the already accepted block count,
`A` its total MRB2 bytes, `L` the next declared MRB2 length, and
`R = N - K - 1` the number of blocks still to follow. The checked header is
accepted only if `L <= Lmax` and:

```text
A + L + R * 168 <= B <= A + L + R * Lmax
```

For genesis its payload is additionally fixed to `8 + 168` bytes. These lower and
upper future-byte checks reject an impossible advertised total before payload
allocation, including totals that would leave too little or too much data for
the remaining blocks. Products and sums are checked in `u64`. Every header also
must fit both the absolute and declared frame/wire budgets. Final counts and
bytes must match exactly; unused declared capacity is not success.

**19,641,112 is an encoded-transfer limit, not an RSS or wall-clock guarantee.**
The bounded manifest parser may allocate a validated configuration and seed
proofs, block decoding creates typed blocks, and accepted-chain execution holds
history and ledger structures under its separate work budget. A caller may also
hold a current branch while receiving a competing candidate. Ordinary allocation
failure/process termination is not made recoverable by this API. Global connection
counts, concurrent transfers, per-peer budgets, timeouts, deadlines, bandwidth,
and retry/resume policy remain outside these per-transfer bounds.

## 6. Sender

`HistoryFramesV2::new(&accepted_chain)` borrows one immutable accepted branch for
the iterator's lifetime. It encodes and retains the bounded canonical manifest,
derives its summary from the actual branch checkpoint/count/encoded bytes, and
yields Begin, Manifest, all complete blocks in order, and End.

Each `next()` returns at most one encoded frame. Encoding a Block temporarily
holds that block's encoded bytes as well as its emitted frame; it does not build
an all-history output buffer. A caller that collects every yielded frame creates
its own additional buffering outside this iterator. Once exhausted or after
yielding an encoding error, the iterator is fused and returns no further frames.
It performs no socket operation, selected-store mutation, or fork comparison.

## 7. Compatibility and dependencies

MPN2/version 2 is distinct from the frozen `MIRR`/version-1 peer protocol. Existing
V1 APIs, encodings, tests, and data directories are not reinterpreted. Existing
MCF2, MNM2, MRB2, operation/state/storage bytes and cryptographic domains are
unchanged. The new framing introduces no hash domain, signed checkpoint, local
generation field, mint, founder allocation, marketing reserve, reward rule, or
biometric/device payload.

The network crate adds local runtime path dependencies on `mirror-chain` and
`mirror-core`, plus local test dependencies on `mirror-consensus` and
`mirror-personhood`. The lockfile adds only those four local dependency edges;
there is no external dependency version update for this increment.

## 8. Independent evidence and acceptance boundary

[`reference_peer_history_v2.py`](../tools/reference_peer_history_v2.py) uses Python's
standard library and the existing independent Python chain generator to rebuild
public synthetic history bytes. It independently packs the new headers, summaries,
height prefixes, complete frames, and complete transcript hashes/lengths into
[`peer_history_v2.txt`](../crates/mirror-protocol/tests/vectors/peer_history_v2.txt).
It does not invoke Rust or accept Rust output as its oracle. Prior chain fixtures
and their byte domains remain unchanged.

The 24 new [codec tests](../crates/mirror-protocol/tests/history_v2.rs) compare exact
independent frames and complete transcript accounting, exercise inclusive and
out-of-range lengths/counts, checked height arithmetic, invalid versions/kinds,
all frame-shape truncations, trailing data, borrowed payloads, semantic-validation
separation, and V1/V2 separation. Focused protocol verification passed **38 tests**
(14 retained V1 plus 24 V2) and the independent reference `--check` passed. These
are focused results, not a statement that the full current workspace or CI passed.

Receiver/sender integration coverage is maintained in
[`history_receiver_v2.rs`](../crates/mirror-network/tests/history_receiver_v2.rs),
with synthetic fixtures in its
[support module](../crates/mirror-network/tests/support_history_v2/mod.rs).
Exact runtime results, full gates, tested source, and publication evidence belong
in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).
This contract does not certify unrun network tests or physical crash behavior.

F1 completion provides a bounded, fully validated in-memory candidate. Actual TCP
exchange, session lifecycle/deadlines, node integration, independent durable
selection/adoption, and their failure handling remain F2 work. Mainnet readiness,
human-finality consensus, private biometric custody/matching, device ingestion,
and Gate A/B completion are not implied.
