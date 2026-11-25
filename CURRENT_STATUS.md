# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB03E1-001 — replacement metadata and preparation**.
Updated **2026-11-26**. Prior MB-01/02 and MB-03A/B/C/D components remain verified.
**MB-03E1: VERIFIED_COMPONENT — METADATA / IN MEMORY.**
**MB-03 remains IN PROGRESS. Next: MB-03E2 — durable replacement and recovery.**

## Exact source and publication

Active branch: `work/personhood-register-v0.1`.
[PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1) is open and unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

Implementation source: **349edf5a6f453f365fa55480f6117c240876b0e1**.
Tested tree: **b88c9cafc8bf417d5609743bacb1ab278070bd0e**.
[Source push CI 45721188614](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45721188614)
and [source PR CI 45721198833](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45721198833)
both passed. Full push-job `139579639094` logs confirmed the exact source checkout,
787 workspace tests, eight compile-fail doctests, every required release suite,
nine independent references, eleven examples and the allocation/runtime-data audit.
Later documentation-head CI is a separate check; verify the live branch/PR rather
than treating this record as evidence for an unknown future head.

The prior implementation MB-03D and its exact-source CI remain recorded in master
section 21. Sections 18–20 retain immutable storage, append and explicit recovery
acceptance. The intervening device/accessibility discussion checkpoint was
`3da75720319094f1b91b4a337b45cfdeb6869e7b`; it changed planning, not runtime evidence.

## Delivered behavior

The new [replacement contract](docs/STORAGE_REPLACEMENT_V2.md) defines exact
280-byte **MSR2**, encoding version 2/profile 3, with a local generation, unchanged
MSS2 snapshot and a distinct checksum domain. Its decoder requires independently
retained configuration/genesis/height/tip/ledger-root and generation pins. A valid
checksum does not make a record authoritative or establish an accepted history.

`prepare_replacement_v2` checks both accepted chains' network identity, the exact
current endpoint/generation and the entire current selector against canonical
metadata derived from the accepted current chain. This catches altered manifest
length/checksum or block-byte totals even when both checksums and endpoint pins pass.
It then requires a strictly preferred candidate under the existing fixed-target
rule, increments local generation once and derives the candidate's own metadata.
Identical, equal-height and shorter candidates reject without a new generation.

The private result borrows the exact whole current and candidate chains. Accounts,
membership, permanent records, approvals, sequences, population history, manifest,
blocks and resource counters remain together. Nothing is copied from the losing
branch into the candidate. Safe Rust prevents either chain changing while the plan
is still used. This is a pure preparation API: no filesystem store, root lock,
selection publication or replacement recovery exists in this increment.

## Verification

Local verification used Rust 1.98.0 with locked dependencies:

- **787 workspace/all-target runtime tests passed:** 735 retained + 31 new codec
  tests + 21 new preparation tests. Zero failures.
- **Eight compile-fail doctests passed:** six retained + two replacement-plan
  borrow checks. Both accepted chains remain immutable while their plan is used.
- Required release suites passed: personhood **234** runtime + **3** doctests;
  core **121**; state **49**; consensus **23**; chain **85** + **3** doctests;
  storage **246** + **2** doctests. Reruns are not additional unique tests.
- **Nine independent Python reference checks** and **eleven synthetic examples**
  passed, including the new MSR2 reference and replacement preparation demo.
- One subprocess-only storage helper remains intentionally ignored in ordinary
  enumeration and is explicitly invoked by parent tests; it is not another pass.
- Scoped formatting, whitespace and allocation/runtime-data audits passed.
  **108 retained source/dependency/reference/fixture files are byte-identical**
  against parent `3da75720319094f1b91b4a337b45cfdeb6869e7b`. The only existing Rust
  file changed is the storage module's additive documentation/exports. Dependencies,
  old format bytes/vectors and Cargo.lock are unchanged. No runtime data is tracked.

Tests include genuine divergent histories with identical account bytes and different
membership, independent network/current pins, stale generations, exact error order,
checksum-correct metadata forgery, generation-zero initialization above genesis,
multi-block replacement counting once, legacy-profile rejection and unchanged inputs.
Two read-only reviews found no blocking implementation/contract discrepancy. These
checks are not an external security audit or a physical power-loss experiment.

The new synthetic demo prepares fork height **3**, generation **0**, for replacement
by accepted main height **4**, generation **1**, with common ancestor height **1**.
Candidate tip: `33d3201d81345860dd5c38629a763b543ad412788962e9fdeafeecfb96c4ce3d`.
Candidate ledger root: `785580b604b65294af265374021fa10cecf08a95bdc51bc044992d3ffbfcfe68`.
Active / pending / historical high-water: **1 / 0 / 3**.
Encoded MRB2 bytes / cumulative state items: **4,330 / 75**.
The exact prepared record matches independent vectors. No disk selection changes.

## Next work and practical limits

MB-03E2 must freeze and implement a bounded storage layout, publication and cleanup
ordering, write fencing and independently authorized recovery under one stable
exclusive root lock. E1 does not choose two reusable slots versus retained-generation
directories. Revalidate exact selected disk history/current pins and the complete
candidate under ownership; an old plan is not a publication permit. Never replace
the locked root, silently reinterpret MSS2/MSU2 or turn one-child append recovery
into generic reorganization. [NEXT_SESSION.md](NEXT_SESSION.md) gives the next actions.

A local generation is neither chain score nor global freshness. Profile 3 currently
requires strictly increasing accepted height; equal-height or future human-weighted
replacement needs a separately reviewed rule. The 4096-block, 16-MiB MRB2 and
1,048,576 cumulative-item caps remain. Preparation encodes bounded manifests and
is not allocation-free. Existing filesystem assumptions remain; no new WSL/DrvFS,
native Windows or network-filesystem support is established.

Fresh-peer reconstruction follows. Persistent V2 node/wallet interfaces, complete
Ant Kiss lifecycle, final human consensus, policy-dependent money rules, private
intake/matching and end-to-end hardening remain unfinished. Gate A and Gate B are UNMET.

## Retained user decisions and design input

No founder privilege or privileged marketing reserve. The equal first 1,000 grants
of 10 MRY include the founder. The ordinary allowance is 0.2 MRY per qualifying
historical population maximum; the combined bootstrap formula and fee/reward rules
remain unresolved. This step introduces no issuance, grant, fee or reward API.

The user's preferred initial standard enrollment combines ECG, one designated pinky
and human-supervised Ant Kiss. A proposed exception combines ECG, observed capture
limitations and reviewed human supervision, with a proposed committee of three
qualified people. The exact finger, committee/quorum, procedures and measured
acceptance remain open. The user considers the combined routes potentially enough
for launch and may consider more fingerprints later; this is a design preference
and sufficiency hypothesis, not measured uniqueness or launch acceptance.

Keep minimum explanations and biometric records protected off-chain. Normal and
exception routes share duplicate-checking scope and the same continuing membership
and entitlements. No public medical label, mandatory all-finger collection or
runtime accommodation mechanism is implemented. See [the proposal](docs/ACCESSIBLE_ENROLLMENT_PROPOSAL.md)
and [device draft v0.3](docs/DEVICE_INPUT_CONTRACT.md).

The last rough estimate remains **20–25% of Gate A** as discussed on 2026-11-26.
It is an engineering scope estimate, not a ratio of tests, a time forecast or a
security score; this bounded increment does not justify an automatic percentage rise.
Gate A uses synthetic readings and complete software integration. Actual device
compatibility, capture and recognition evidence belong to Gate B.
