# Mirror

Mirror is a Rust development blockchain exploring human-based participation.
**It is not a public mainnet, a deployed proof-of-humanity system, or real money.**

## Roadmap and continuation

[MASTER_PLAN.md](MASTER_PLAN.md) tracks the route from the Rust development chain
to an integrated blockchain/enrollment stack ready for
supported fingerprint and ECG inputs. Each task has dependencies and acceptance
criteria. **MB-01B and MB-02A/B/C are verified components:** configuration, membership,
atomic ledger execution and actual V2 block acceptance/replay. **MB-03A/B are
verified storage components:** immutable selected-chain initialization, lifetime
ownership, checked reopening and bounded one-block append publication with failure fencing.
Next is **MB-03C: explicit recovery of interrupted append staging**, followed by
canonical branch selection, durable membership reorganization and peer reconstruction.
MB-03 as a whole and the V2 node/API remain unfinished.

[Device input contract](docs/DEVICE_INPUT_CONTRACT.md) is a draft compatibility
checklist; no physical device is supported merely because the document exists.
[CURRENT_STATUS.md](CURRENT_STATUS.md) records verified progress;
[NEXT_SESSION.md](NEXT_SESSION.md) gives the exact next action;
[the-prompt](the-prompt) is the maintained copy-and-paste handoff for new chats.
Gate A is tested software readiness; Gate B adds controlled actual-device tests.
Both are future milestones, not a claim of completed mainnet security.

## Current direction

- One eligible human, not hardware speed or wealth, is the intended unit of
  consensus participation. The existing easy-PoW/longest-chain devnet is a
  development foundation, NOT that final consensus design.
- Ant Kiss is human-witnessed enrollment and review. Fingerprint and ECG readings
  are proposed off-chain inputs. Devices are interchangeable instruments.
- No raw or encrypted biometric archive belongs on the public blockchain.
- No special founder premine or privileged marketing/airdrop reserve.
- The founder is eligible for the same 10-MRY grant as each of the first 1,000
  qualifying humans: an explicit 10,000-MRY bootstrap exception, not an owner fund.
- Ordinary supply allowance: 0.2 MRY per qualifying historical population maximum.
  A fall in population does not confiscate currency or lower that record.
- The combined treatment of the bootstrap exception and population allowance is
  still OPEN. No minting rule silently chooses an additive allowance or a floor.
- One MRY = 10,000,000 Nusa. Participation rewards and fees remain to be specified.

See [decisions](docs/DECISIONS.md), [current status](CURRENT_STATUS.md), and
[personhood milestone](docs/PERSONHOOD_V0_1.md).

## What is implemented

The existing development chain implements signed transfers, account balances and
nonces, block/state commitments, fixed easy PoW, file persistence, TCP block sync,
and validated competing-chain replacement. It is not security-audited.

The experimental `mirror-personhood` crate preserves its V1 registry, state root
and bounded event-log replay. Its separate V2 modules now implement configuration-
bound signed messages and staged membership execution: exact sequences, expiry,
permanent identity-reference reservations, parent eligibility, end-of-block
activation and bounded approval history. B4 adds canonical MPS2 state commitments,
bounded decoding into an untrusted cache, and MPH2 replay preserving block groups
and empty blocks. Replay reuses the same executor and checks independently trusted
configuration, hashes, exact operation groups, intermediate roots and the requested
final checkpoint. See [state rules and cache boundary](docs/MEMBERSHIP_STATE_V2.md)
and [replay contract](docs/REPLAY_V2.md). A decoded cache cannot become runtime
authority; a caller-supplied hash alone is not proof of actual block validity.
Actual V2 validation and storage now exist in the separate layers below. Bounded
tip updates are implemented; explicit staging recovery, peer messages and rewards
remain later work.

`mirror_core::v2` adds signed transfers, a unified operation envelope and bounded
MRB2 blocks with domain-separated header hashes and count-bound ordered-operation
commitments. Decode is structural; body integrity checks C, roots and signatures.
Neither grants ledger/chain authority. See the [encoding contract](docs/CORE_V2_ENCODING.md).

`mirror_state::v2` now stages accounts and membership atomically, preserves original
mixed-operation positions, executes checked zero-fee transfers and commits the
combined state. Late transfer or membership failure leaves both parent components
unchanged. Explicit zero accounts and independent sequences are retained. The
[ledger contract](docs/LEDGER_STATE_V2.md) specifies canonical bytes and boundaries.

`mirror_chain::v2` now accepts actual complete blocks from an exact selected genesis.
Its MNM2 manifest requires all seed proofs and independently selected configuration/
genesis pins. Block acceptance checks work, real ancestry, original signed bodies
and the executed combined root before installing state. Complete MRB2 replay uses
the same validator and an external expected checkpoint. A false final root with
valid mining work is rejected without changing the chain. The
[chain contract](docs/CHAIN_V2.md) records exact rules, resource caps and trust.

`mirror_storage::v2` supplies two explicitly selected storage profiles. Immutable
`StoreV2` retains its frozen MSS2 generation-zero contract. The separate
`AppendStoreV2` uses MSU2 profile 2 to publish one completely accepted child at a
time. It synchronizes the new block and selector before advancing the in-memory
chain. Both profiles retain directory ownership and reconstruct complete MNM2/MRB2
history under independently chosen network and full checkpoint pins. Append-store
reopening also requires the separately retained local generation; it does not
discover global freshness. No decoded state cache becomes authority.

An append I/O failure fences further writes and ordinary state access. Consuming
reconciliation fully replays and resynchronizes a clean selected history before
returning a writer. Pending selectors, extra tail blocks and unexplained artifacts
are preserved and rejected; their explicit recovery is the next slice. See the
[immutable storage contract](docs/STORAGE_V2.md) and
[append storage contract](docs/STORAGE_APPEND_V2.md) for exact publication boundaries,
Linux/cooperating-process assumptions and bounded-read rules. No automatic migration,
repair, branch replacement or live V2 daemon is provided.

Latest implementation source:
[`ecf5e76`](https://github.com/Ahmed-Sobeih/mirror/commit/ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c).
Local verification
passed **658 workspace tests** (**70 new**), with **one ignored subprocess helper**,
and **3 compile-fail doctests**. Release reruns passed core **121**, personhood
**234** plus the same **3** doctests, state **49**, consensus **23**, chain **57** and
storage **145** tests. All **eight independent references** and **eight examples**
passed locally. Exact-source [push CI 44649111203](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203)
and [PR CI 44649119266](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
both passed. Full push-job logs confirmed the exact published checkout and every
gate. [CURRENT_STATUS.md](CURRENT_STATUS.md) separates source verification from
later documentation-head CI. Prior MB-03A acceptance remains in the master plan;
reruns are not additional unique tests.

Both storage examples create temporary synthetic stores. The append demonstration
adds accepted mixed and empty blocks, then reopens and checks the complete state.
They do not start a persistent V2 network; that integration remains MB-03/04.
[PR #1](https://github.com/Ahmed-Sobeih/mirror/pull/1) remains open and unmerged.

The register prevents exact request/key/evidence reuse. It cannot identify the
same human presenting a different key and new evidence. Human review and private
biometric deduplication remain external, unfinished mechanisms. Public keys and
approval relationships in this prototype are linkable: anonymity is not claimed.

## Important: the development accounts are not founder allocations

The node starts its historical devnet with 100 MRY assigned to the public test
key called Alice (`[1; 32]`); Bob uses `[2; 32]`. Those known keys are insecure for
real funds. The fixture is retained to avoid invalidating existing local devnet
files and reorg checkpoints. It is NOT a founder entitlement, a mainnet genesis,
or an implementation of the new bootstrap grants. No production founder wallet
or billion-MRY founder allocation is established by this repository.

Never use these keys for assets with value. Never upload biometric records,
secrets or `mirror-data` to GitHub.

## Development checks

```sh
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo test -p mirror-personhood --release --locked
cargo test -p mirror-core --release --locked
cargo test -p mirror-state --release --locked
cargo test -p mirror-consensus -p mirror-chain --release --locked
cargo test -p mirror-storage --release --locked
python3 tools/reference_config_v2.py --check
python3 tools/reference_operations_v2.py --check
python3 tools/reference_state_v2.py --check
python3 tools/reference_blocks_v2.py --check
python3 tools/reference_ledger_v2.py --check
python3 tools/reference_chain_v2.py --check
python3 tools/reference_storage_v2.py --check
python3 tools/reference_append_storage_v2.py --check
cargo run -p mirror-personhood --example registry_demo --locked
cargo run -p mirror-personhood --example operation_v2_demo --locked
cargo run -p mirror-personhood --example state_v2_demo --locked
cargo run -p mirror-core --example block_v2_demo --locked
cargo run -p mirror-state --example ledger_v2_demo --locked
cargo run -p mirror-chain --example chain_v2_demo --locked
cargo run -p mirror-storage --example storage_v2_demo --locked
cargo run -p mirror-storage --example append_v2_demo --locked
```

The examples use synthetic attestations, issue no currency and never open the
existing chain database. GitHub Actions verifies the workspace using Rust 1.98.0.

Legacy `mirror-node mine`, `listen` and `connect` remain devnet-only. Do not run
multiple writers against one data directory. Directory replacement has only
limited crash-recovery tests; it is not a proven atomic transaction across all
filesystems or a substitute for locking.
