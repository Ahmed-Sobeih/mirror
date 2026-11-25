# Mirror

Mirror is a Rust development blockchain exploring human-based participation.
**It is not a public mainnet, a deployed proof-of-humanity system, or real money.**

## Roadmap and continuation

[MASTER_PLAN.md](MASTER_PLAN.md) tracks the route from the Rust development chain
to an integrated blockchain/enrollment stack ready for
supported fingerprint and ECG inputs. Each task has dependencies and acceptance
criteria. **MB-01B and MB-02A/B/C are verified components:** configuration, membership,
atomic ledger execution and actual V2 block acceptance/replay. **MB-03A/B supply
verified storage components:** immutable selected-chain initialization, lifetime
ownership, checked reopening and bounded one-block append publication with failure
fencing. MB-03C adds explicit recovery of one independently authorized interrupted
append. MB-03D adds comparison of complete accepted branches under an independent
network identity. **MB-03E1 adds bounded replacement-selection metadata and checked
in-memory preparation** of a preferred complete branch. Current acceptance evidence
is in [CURRENT_STATUS.md](CURRENT_STATUS.md) and master-plan section 22.
Next is **MB-03E2: durable replacement layout, publication and recovery**, followed
by fresh-peer reconstruction.
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
tip updates and explicit recovery of recognized one-child staging are implemented;
V2 peer messages and rewards remain later work.

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

The separate [development fork comparator](docs/FORK_CHOICE_V2.md) compares only
already accepted `ChainV2` values under the same independently selected configuration
and genesis. Because every block uses that configuration's fixed target, accepted
post-genesis block count is an exact normalized work ordering. A strictly longer
candidate is preferred; ties retain the current input. The result identifies the
actual common ancestor and borrows the entire selected chain, including its complete
membership and counters. It performs no durable replacement or network operation.
Equal competing tips can remain different across peers; this is not human finality.

`mirror_storage::v2` supplies two implemented storage profiles. Immutable
`StoreV2` retains its frozen MSS2 generation-zero contract. The separate
`AppendStoreV2` uses MSU2 profile 2 to publish one completely accepted child at a
time. It synchronizes the new block and selector before advancing the in-memory
chain. Both profiles retain directory ownership and reconstruct complete MNM2/MRB2
history under independently chosen network and full checkpoint pins. Append-store
reopening also requires the separately retained local generation; it does not
discover global freshness. No decoded state cache becomes authority.

An append I/O failure fences further writes and ordinary state access. Consuming
reconciliation fully replays and resynchronizes a clean selected history before
returning a writer. Normal opening and reconciliation preserve and reject residual
staging. The new explicit recovery API accepts independently retained parent and
candidate checkpoints, the exact child and an action. It can complete recognized
candidate prefixes or retain the still-selected parent by removing only authorized
staging. It never rolls back an already selected candidate or repairs unexplained
bytes. Full parent/child execution and final replay/synchronization remain required.
See the
[immutable storage contract](docs/STORAGE_V2.md) and
[append storage contract](docs/STORAGE_APPEND_V2.md) for exact publication boundaries,
Linux/cooperating-process assumptions and bounded-read rules. The separate
[recovery contract](docs/STORAGE_RECOVERY_V2.md) specifies the supported failure
states, caller authority, suffix-only completion, parent retention and finite read
allowance. No automatic migration, implicit repair, branch replacement or live V2
daemon is provided.

The separate [replacement preparation contract](docs/STORAGE_REPLACEMENT_V2.md)
adds MSR2 profile-3 metadata without changing either existing store. Its 280-byte
record wraps the unchanged MSS2 snapshot and a bounded local generation. Decoding
requires independent network, complete endpoint and generation pins. Preparation
also derives the entire current record from the accepted current history, rejects
stale or altered metadata, and borrows the whole preferred candidate with its own
accounts, membership and counters. It prepares no files and publishes no selection;
durable ownership, interruption and recovery are MB-03E2 work.

## Verification checkpoints

Current MB-03E1 verification and publication evidence is tracked in
[CURRENT_STATUS.md](CURRENT_STATUS.md) and [MASTER_PLAN.md](MASTER_PLAN.md), section
22. The following MB-03D record is retained as a **historical accepted checkpoint**;
its results do not establish a newer source's full-suite or CI status.

MB-03D source: `b9a2e870996fcafe1128dcb5cfdfb6b01b158ded`.
Source tree: `dd0a3fc29f429dcd4999261801c2bc7897d6aace`.
Focused checks passed 28 new branch-comparison integration tests, three new
compile-fail ownership/type cases and the new `fork_choice_v2_demo` example.
Full checkpoint verification:

Published-source [push CI 45703714223](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703714223)
and [PR CI 45703726306](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703726306)
both passed. Full push-job `139520118282` logs confirmed the exact implementation
checkout, **735 workspace tests, six compile-fail doctests, all required release
suites, eight independent references, ten examples and the allocation/runtime-data
audit**. The single intentionally ignored subprocess helper is invoked by parent
tests and is not another pass. Release reruns are not additional unique tests.
MB-03D is **VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON**. Subsequent
documentation-head CI is checked separately and recorded on PR #1.

[CURRENT_STATUS.md](CURRENT_STATUS.md) records exact source acceptance separately
from later documentation-head CI. Historical MB-03C acceptance remains in
master-plan section 20 and the decision record: 707 workspace tests, three
doctests, eight references and nine examples at that earlier source. Release
reruns are not additional unique tests. Fork comparison adds no encoded score,
wire format, hash domain or dependency.

All three storage examples create temporary synthetic stores. The append example
adds accepted mixed and empty blocks, then reopens and checks the complete state.
The recovery example constructs known partial-write shapes, completes an append,
repeats it idempotently, retains another parent, and then performs an ordinary next
append. These are controlled software scenarios, not physical power-loss tests.
They do not start a persistent V2 network; that integration remains MB-03/04.
The fork-choice example compares frozen branches with equal account bytes and
different membership, demonstrates sticky ties, and borrows the complete longer
selected branch without changing either input or a store.
The replacement-preparation example checks a genuine divergent candidate and
round-trips MSR2 metadata without opening or changing a store.
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
python3 tools/reference_replacement_storage_v2.py --check
cargo run -p mirror-personhood --example registry_demo --locked
cargo run -p mirror-personhood --example operation_v2_demo --locked
cargo run -p mirror-personhood --example state_v2_demo --locked
cargo run -p mirror-core --example block_v2_demo --locked
cargo run -p mirror-state --example ledger_v2_demo --locked
cargo run -p mirror-chain --example chain_v2_demo --locked
cargo run -p mirror-chain --example fork_choice_v2_demo --locked
cargo run -p mirror-storage --example storage_v2_demo --locked
cargo run -p mirror-storage --example append_v2_demo --locked
cargo run -p mirror-storage --example recovery_v2_demo --locked
cargo run -p mirror-storage --example replacement_plan_v2_demo --locked
```

The examples use synthetic attestations, issue no currency and never open the
existing chain database. GitHub Actions verifies the workspace using Rust 1.98.0.

Legacy `mirror-node mine`, `listen` and `connect` remain devnet-only. Do not run
multiple writers against one data directory. Directory replacement has only
limited crash-recovery tests; it is not a proven atomic transaction across all
filesystems or a substitute for locking.
