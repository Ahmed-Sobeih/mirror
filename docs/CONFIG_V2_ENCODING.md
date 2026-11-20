# B1: MCF2 configuration and bootstrap proof boundary

Original checkpoint: MIRROR-MB01B-B1-001. This document preserves B1's unsigned
bytes and configuration/bootstrap boundary. CURRENT_STATUS.md and the PR checkpoint
record exact source/CI evidence. The separate [MB-02C chain layer](CHAIN_V2.md) now
checks manifest selection and actual genesis; this does not launch a network or
establish a biometric procedure.

## Wire bytes (all integers unsigned little-endian)

| Offset | Field | Size |
|---|---|---:|
| 0 | magic MCF2 | 4 |
| 4 | format_version = 2 | 2 |
| 6 | synthetic profile_version = 1 | 2 |
| 8 | nonzero network domain | 32 |
| 40 | fixed development pow_bits | 4 |
| 44 | genesis timestamp | 8 |
| 52 | required approvals | 2 |
| 54 | nonzero synthetic evidence-profile identifier | 32 |
| 86 | ten limit fields below | 40 |
| 126 | max_anchor_age | 8 |
| 134 | max_validity_blocks | 8 |
| 142 | bootstrap count N | 4 |
| 146 | N ordered descriptors: key/nonce/evidence | N * 96 |
| 146 + N*96 | allocation count A | 4 |
| 150 + N*96 | A strictly sorted address/u64-balance pairs | A * 40 |

Ten u32 limit fields, exact order: max_block_bytes, max_operations,
max_membership_operations, max_new_enrollments, max_operation_bytes, max_pending,
max_records, max_accounts, max_bootstrap, max_log_bytes. Total size is
150 + 96*N + 40*A. Maximum input is 2,717,590 bytes; counts are validated before
reserving input-sized vectors. Trailing bytes/unknown versions/magic are rejected.
No context hash, signatures, IDs, roots or genesis hash is serialized here.

C = SHA256d(ASCII("MIRROR_CONFIG_V2") || complete MCF2 bytes).
For zero-based index i and 96-byte descriptor d:
seed_hash = SHA256d(ASCII("MIRROR_SEED_V2") || C || u32(i) || d).
seed_id = SHA256d(ASCII("MIRROR_SEED_ID_V2") || C || u32(i) || d).
No domain includes a terminating NUL. Every seed signs its seed_hash. Proofs are
passed separately in descriptor order; ALL proofs are required, independent of
runtime approval threshold. Index zero has no authorization shortcut.

A structurally valid ConfigV2 is not a verified bootstrap: PublicKey::from_bytes
only stores bytes. ConfigV2::verify_bootstrap invokes the existing strict
Ed25519 checker, rejects malformed/weak keys and invalid signatures, and returns
an immutable VerifiedBootstrap only after every proof passes. It exposes no
currency mutation or runtime membership API. Config/VerifiedBootstrap fields are
private; neither accepts a caller-supplied context hash. Seed proofs/IDs derive
from unsigned config BEFORE any roots/genesis, avoiding circular hashing.

## Checked construction

Threshold is 1..64 and <= seed count. Seed count is 1..max_bootstrap. Seeds have
unique keys, nonces and evidence; nonce/evidence must be nonzero. These are exact
record checks, not biological matching. Allocations are strictly increasing by
raw address bytes, including zero-balance entries. Their sum must fit u64; the
empty allocation list has zero initial test supply. No balance is implied for the
first seed, founder or marketing. PoW targets have the same positive/nonzero,
exponent<=32 range as legacy PowTarget::from_compact; B1 does not mine.

Limits default to the MB-01A table and cannot exceed those development ceilings.
Each is nonzero. Membership ops <= all ops, new enrollments <= membership ops and
pending quota, pending/bootstrap <= total records. Single-operation capacity is
at least 291 bytes. Block capacity >= 168+4+max_operation_bytes; log capacity >=
8+max_operation_bytes. Limits may be lowered consistently. These checks validate
configuration ONLY. B2/B3 and MB-02A/B now enforce their respective message,
membership, body and mixed-execution bounds. MB-02C additionally enforces actual
accepted-block and cumulative finite-branch bounds. Durable storage and peer
enforcement remain MB-03/04; configuration construction alone supplies neither.

## Independent evidence

`tools/reference_config_v2.py` uses only Python hashlib/struct, not Mirror code.
`--check` compares its result to the committed Rust fixture. Public test keys are
those derived from known [1;32] and [2;32] seeds. Inputs, full bytes, context and
both seed signing hashes/IDs are committed in
`crates/mirror-personhood/tests/vectors/config_v2.txt` and checked by
`crates/mirror-personhood/tests/config_v2.rs`. Do not change expected values to
match an unexplained Rust failure. CI runs reference, debug and release tests.

## Scope and remaining matrix coverage

T02 configuration/seed-context binding is covered. T03's acyclic construction,
ordered all-seed proof verification and invalid-config parts are covered; its
runtime approval/withdrawal privilege tests were subsequently supplied by B3.
T20 configuration/count bounds are covered here; B3 and MB-02A/B add direct
membership/body/mixed-execution bounds. No other full matrix case is claimed
complete by B1 alone. Legacy V1 APIs, signatures, MPH1/MRB1 bytes,
node source, ledger history and monetary policy are unchanged.

MB-02C's immutable `NetworkManifestV2` verifies every seed proof and carries an
exact selected genesis hash. Its MNM2 decoder requires independently selected C
and genesis expectations; `ChainV2` separately validates the actual pinned genesis
and executed bootstrap root. Valid proofs of possession still do not make a
peer-supplied configuration authoritative. Durable identity/checkpoint storage and
explicit directory initialization remain MB-03; no existing directory is migrated.
No V2 membership activation, genesis initialization, real admission, biometric
matching, new grants, fees or human consensus is enabled by B1.
