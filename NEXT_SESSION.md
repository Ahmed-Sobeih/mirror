# Next session — MB-03A

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-20**.
Read AGENTS.md, CURRENT_STATUS.md, docs/DECISIONS.md, docs/CHAIN_V2.md,
docs/LEDGER_STATE_V2.md, docs/CORE_V2_ENCODING.md, docs/CONFIG_V2_ENCODING.md,
docs/OPERATIONS_V2_ENCODING.md, docs/MEMBERSHIP_STATE_V2.md, docs/REPLAY_V2.md,
docs/LEDGER_MEMBERSHIP_V1.md, docs/THREAT_MODEL.md,
docs/MEMBERSHIP_TEST_MATRIX.md and the latest the-prompt.

## Recover live state

Verify default/active branches, PR #1, recent commits/comments and exact CI.
Active branch `work/personhood-register-v0.1`; PR #1 remains open/unmerged.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`. Use feature-branch records,
not stale main. Preserve concurrent/unpushed work and all runtime data. Never reset
to a copied checkpoint or merge merely to publish handoff documents.

Latest implementation `db2cb58ba5a9a828ec949debd1b52b5915a66f3b`, tree
`3fa1d3b44abb885898cc256f9525355025649a0a`. Exact-source push CI:
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623282448
Source push CI and PR CI 44623296178 passed. Full job 135910328823 logs confirmed
the exact source checkout and all gates. Check subsequent documentation-head CI
separately. Local and CI Rust 1.98.0 gates passed 523 workspace tests, 3 doctests,
release core 121 / personhood 234 plus the same 3 doctests / state 49 / consensus 23 /
chain 57, all 6 references/examples and audit. Focused formatting and preservation
checks passed locally. Release reruns do not add unique tests.
MB-01A, MB-01B/B1–B4 and MB-02A/B/C are complete in their component scope.
**MB-02 VERIFIED_COMPONENT — IN MEMORY. MB-03 READY. Gate A/B UNMET.**

## Reuse the completed actual chain

`NetworkManifestV2` carries proof-checked configuration and exact genesis pin.
MNM2 decode requires an independently selected `(C, genesis_hash)`. Every seed proof
is mandatory. `genesis_template_v2` makes an unaccepted proposal; bounded work search
then gives a candidate, not acceptance. `ChainV2::from_genesis` independently checks
all genesis fields, work, body, bootstrap ledger root and exact selected hash.

`ChainV2::append` validates full budgets, context, height/parent/time, configured
bits/work, ordered operation root, original full-body ledger execution and claimed
combined root. Only then does it bind the actual hash and install state. It uses
its private immutable accepted ancestry; do not add an external mutable oracle or
state setter. A failed block leaves the entire branch and counters unchanged.
`prepare_candidate` cannot publish state. Lower-level `bind_to_validated_block`
remains only a trusted provenance operation, not a substitute for acceptance.

Typed and encoded complete MRB2 replay require external C/genesis/height/tip/root
expectations. Replay checks every actual block through the same executor and returns
no partial branch on failure. The expectation chooses a branch, not global freshness.
MPH2 remains membership-only; MPS2 remains an untrusted cache with no runtime
conversion. Never filter full mixed histories or restore authority from a snapshot.

Preserve strict child timestamp > parent, exact configured bits, empty blocks,
full-operation indices, separate account/member sequences and initial test-supply
conservation. There is no fee, mint, reward or added membership spending restriction.
Chain caps include genesis: 4,096 blocks, 16 MiB complete MRB2, 1,048,576 cumulative
state/config items. Exact charge formula is in CHAIN_V2.md. Do not bypass these
limits through recovery, silently raise them or discard identity/account history.

## Exact next slice — MB-03A storage authority and one writer

Start by inspecting existing `mirror-storage` and legacy node recovery, then freeze
an explicit V2 storage contract before adding file formats. Reuse ChainV2 for all
validity; preserve all legacy files, APIs, genesis and existing runtime directories.
This slice must be reviewable independently of future networking/economic work.

1. **Specify storage identity and lifecycle.** Define separate explicit V2 init,
   open and read-only inspection paths. Pin configuration/genesis to trusted caller
   selection, retain all public seed proofs, and define which metadata identifies
   the selected accepted tip. Empty, missing, partial, foreign and legacy directories
   need distinct behavior; reading status must never initialize, mine or repair.
2. **Specify coherent publication and recovery.** Define exact bounded encodings,
   staging/publication boundaries, file and directory synchronization, metadata
   generations, restart decisions and error semantics before implementing them.
   State which filesystem guarantees are assumed. Old/new complete selected state
   or explicit failure must be identifiable; an ambiguous I/O result must force
   reconciliation before further writes. No silent best-effort fallback.
3. **Implement single-writer ownership and explicit initialization/opening.** Reject
   a concurrent writer; bind ownership to store lifetime. Define safe reopening
   after interruption without deleting another process's state or trusting stale
   lock-file contents. Use explicit synthetic configuration/genesis; never create
   a production founder key or initialize from an untrusted peer's manifest alone.
4. **Bound all disk inputs and reconstruct through ChainV2.** Preflight lengths,
   counts and selected metadata before allocation. Replay complete actual blocks
   against the trusted chosen checkpoint. Reject truncation, gaps, wrong genesis,
   tampered roots and obsolete/contradictory metadata. Cached accounts/membership
   cannot become a second authoritative database. No matcher/device runs in replay.
5. **Test the slice at its real boundary.** Use separate temporary directories and
   actual V2 blocks with different membership histories. Cover wrong/legacy identity,
   missing/partial init, a second writer, oversized/corrupt input and read-only open
   without side effects. Document each still-unimplemented publication/fault case;
   do not infer universal crash safety from normal round trips.
6. **Publish with precise evidence.** Add tests and bounded examples for the chosen
   slice, run required gates and exact-source CI, then update/read back handoffs.
   If MB-03A splits further, record the exact implemented substep and leave remaining
   storage acceptance criteria visibly incomplete.

Next MB-03 slices complete durable block/ledger-tip publication with failure
injection, peer reconstruction, then canonical reorganization with different
admissions/withdrawals and no orphan counters retained. Global development fork
choice must be specified before publication; human finality still belongs to MB-06.
A network advertised height/root cannot choose trust. Do not label successful
in-memory fork replay as completed durable reorg or multiple live nodes.

## Constraints unchanged

No founder bonus, privileged marketing reserve, grant/mint API, nonzero development
fees or reward allocation. Retain explicit zero/spent accounts, original indices,
parent-eligible reviewers, end-block activation and permanent identity reservations.
D-01/D-04 remain open and do not block this no-mint storage work.

Keys/evidence equality is not biological uniqueness; public review metadata is
linkable. Raw or encrypted biometric archives, real keys, device secrets and personal
records never belong on-chain or in GitHub. Real capture and device purchasing are
outside this task. Existing easy PoW is not the final human-participation consensus.

Before every Mirror final response refresh the-prompt and relevant checkpoints
with actual results and next work, preserve newer source, publish with head guards
and verify readback. Record implementation evidence separately from documentation
heads; never chase a self-referential SHA or merge merely to publish a checkpoint.
