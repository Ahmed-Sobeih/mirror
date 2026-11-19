# Mirror

Mirror is a Rust development blockchain exploring human-based participation.
**It is not a public mainnet, a deployed proof-of-humanity system, or real money.**

## Roadmap and continuation

[MASTER_PLAN.md](MASTER_PLAN.md) tracks the route from the current standalone
membership register to an integrated blockchain/enrollment stack ready for
supported fingerprint and ECG inputs. Each task has dependencies and acceptance
criteria. **MB-01B and the MB-02A transfer/block format layer are verified components.**
Next is **MB-02B: atomic accounts and membership execution with combined ledger roots**.
Actual V2 block authority follows in MB-02C; durable integration follows in MB-03.

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
authority; a caller-supplied hash is not proof of actual block validity. Live V2
block validation, peer messages, unified balances and rewards remain later work.

`mirror_core::v2` adds signed transfers, a unified operation envelope and bounded
MRB2 blocks with domain-separated header hashes and count-bound ordered-operation
commitments. Decode is structural; body integrity checks C, roots and signatures.
Neither grants ledger/chain authority. See the [encoding contract](docs/CORE_V2_ENCODING.md).

Latest source: [`35ad0cf`](https://github.com/Ahmed-Sobeih/mirror/commit/35ad0cf708bd4ceeb4faa306ba4f7bd74912c382).
[Exact-source CI](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399)
passed **403 workspace tests**, core release 121, personhood release 216, the retained
compile-fail doctest, all four independent references and all four examples.
The same doctest also passed in personhood release. Release reruns are not new tests.
PR #1 remains open and unmerged. Actual V2 balances/membership execution is next.

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
python3 tools/reference_config_v2.py --check
python3 tools/reference_operations_v2.py --check
python3 tools/reference_state_v2.py --check
python3 tools/reference_blocks_v2.py --check
cargo run -p mirror-personhood --example registry_demo --locked
cargo run -p mirror-personhood --example operation_v2_demo --locked
cargo run -p mirror-personhood --example state_v2_demo --locked
cargo run -p mirror-core --example block_v2_demo --locked
```

The examples use synthetic attestations, issue no currency and never open the
existing chain database. GitHub Actions verifies the workspace using Rust 1.98.0.

Legacy `mirror-node mine`, `listen` and `connect` remain devnet-only. Do not run
multiple writers against one data directory. Directory replacement has only
limited crash-recovery tests; it is not a proven atomic transaction across all
filesystems or a substitute for locking.
