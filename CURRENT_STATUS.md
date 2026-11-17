# Current status

Roadmap: MIRROR-PLAN-001, [MASTER_PLAN.md](MASTER_PLAN.md).
Latest implementation: **MIRROR-MB01B-B1-001 — configuration/bootstrap VERIFIED**.
MB-01A design is complete. MB-01B is **IN_PROGRESS**: B1 complete; B2–B4 remain.
**Next: MB-01B/B2 — signed V2 membership operations, codecs, anchors and windows.**
Status of next slice: READY, not implemented.

## Publication and exact evidence

Active branch: `work/personhood-register-v0.1`.
PR #1: https://github.com/Ahmed-Sobeih/mirror/pull/1 — open, not merged.
Observed main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.
B1 implementation/tested commit: `809661b5e6a9338d65c15c45e540289df24fcc99`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44201393943 — SUCCESS.
Job 134456099484 summaries and full logs were inspected. These checkpoint documents
follow the tested source; inspect the latest exact-head CI separately. Recheck live
refs before continuing; newer verified work overrides copied prompts.

## Completed in B1

Added separate `crates/mirror-personhood/src/v2/` modules: config, limits and
canonical unsigned MCF2 encoding. The V1 library and its encodings are unchanged.

ConfigV2 validates versions, network/evidence domains, representable PoW target,
approval threshold, all development limits, ordered seed descriptors, strictly
sorted synthetic account allocations and checked initial test-supply sum. Context
binds every field, including seed order/contents and zero-balance allocation entries.
Counts and byte limits are checked before input-sized cloning/allocation. Decoding
rejects malformed/truncated/trailing input and repeats constructor validation.

Context is computed BEFORE seed signatures/IDs, avoiding circular genesis hashing.
All ordered seed proofs are required, using the existing strict Ed25519 verifier.
A structurally valid ConfigV2 is not a VerifiedBootstrap until all proofs pass.
The first seed has no shortcut. Immutable public APIs do not accept a caller's
claimed context hash. A verified bootstrap establishes key possession only, not
biological uniqueness, authority of an untrusted peer's configuration, or live
membership. There is no V2 runtime membership mutation or currency API yet.

Full wire format: [CONFIG_V2_ENCODING.md](docs/CONFIG_V2_ENCODING.md).
Independent reference: `tools/reference_config_v2.py`; it uses Python hashlib and
struct, not Mirror Rust code. Full bytes/context/seed hashes/IDs are committed in
`crates/mirror-personhood/tests/vectors/config_v2.txt` and checked by Rust tests.

## Actual test results

- **164 workspace/all-target tests passed**: all 133 previous tests plus 31 new B1
  tests in `crates/mirror-personhood/tests/config_v2.rs`.
- **57 personhood tests passed in release mode**: 26 V1 + 31 B1. These are reruns,
  not additional unique tests.
- Independent Python configuration/seed reference-vector check passed.
- Legacy synthetic enrollment/replay example executed successfully; no MRY issued.
- Documentation-test command passed, currently zero doctest cases.
- Allocation/runtime-data audit passed; Cargo.lock was not rewritten.

Rust compile/runtime claims come from actual GitHub Actions with pinned Rust 1.98.0
and the lockfile, not local Rust execution. The local environment had no Rust
compiler. The independent Python reference was also executed locally.

## Acceptance scope and remaining gaps

T02 configuration/bootstrap signature binding is VERIFIED. T03 construction/proof
checks and T20 configuration/count bounds are VERIFIED SUBCASES only: their runtime
membership privileges/execution limits remain B3/MB-02. Other matrix rows remain
planned. See [test matrix](docs/MEMBERSHIP_TEST_MATRIX.md), section 4.

F01 from the historical threat review is addressed at the V2 configuration/seed
boundary, not retroactively fixed in V1. Runtime operations must use this context
in B2; full genesis/network pinning is MB-02/03. The historical F01–F11 table records
the earlier source, not a claim that B1 solves all its findings. Remaining expiry,
parent eligibility, block-end activation, resource enforcement, combined ledger,
storage/transport and real-admission issues remain on the roadmap.

## Preserved boundaries

No node source, historical genesis, wallet balance, monetary-policy source,
dependency or lockfile changed. The V1 personhood source only exports the new V2
module; its old tests/vectors remain intact. No local mirror-data was touched.
Alice's known 100-MRY development fixture is NOT a founder entitlement.

No real enrollment, biometric matching, anonymous credentials, live membership
blocks, human-round consensus, grants/rewards/fees or device adapters are enabled.
Gate A and Gate B remain UNMET. Public member/reviewer relationships remain linkable.

No founder bonus or privileged marketing reserve. First 1,000 qualifying humans,
including founder, have the same 10-MRY bootstrap allocation; ordinary allowance
is 0.2 MRY per qualifying historical population maximum. D-01 combined-ceiling
and D-04 reward/fee choices remain unresolved. B1 does not decide them.

Continue NEXT_SESSION.md: B2, then B3/B4, then MB-02 ledger integration.
