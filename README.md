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
in-memory preparation** of a preferred complete branch. **MB-03E2a adds actual
whole-branch publication to a separate replacement store**, with stable root
ownership, checked old-copy cleanup, failure fencing and clean reconciliation.
**MB-03E2b adds explicit recovery of recognized interrupted replacement staging
and cleanup**, using independently retained old/candidate histories and an explicit
completion or retention action. **MB-03F1 adds separate V2 history frames and an
in-memory receiver that executes a complete candidate from genesis**, under an
independently selected network. **MB-03F2 adds finite TCP sessions and explicit
durable adoption of a completely validated candidate**, including retained inputs
for explicit recovery. **MB-03F2, MB-03F and MB-03 are VERIFIED_COMPONENT within
their finite storage and peer-session scope.** **MB-04A1 now adds durable node
control state and restart authority** in a separate operator-owned directory:
confirmed local pins and both exact accepted histories survive process restart,
and unfinished intent requires an explicit recovery decision. MB-04A1 is a
VERIFIED_COMPONENT within this finite control/restart scope. Current acceptance
is recorded in [CURRENT_STATUS.md](CURRENT_STATUS.md) and [MASTER_PLAN.md](MASTER_PLAN.md).

Next is **MB-04A2: finite V2 init/status/sync-once/explicit-recover operator
commands**, using the A1 library. Persistent serving, reconnect/rejoin scheduling,
global connection admission, multi-peer policy, transaction pool and wallet remain
unfinished. The existing V1 executable remains separate.

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
tip updates and explicit recovery of recognized one-child staging are implemented.
Separate V2 peer sessions are described below; grants and rewards remain later work.

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

`mirror_storage::v2` supplies three separate storage profiles. Immutable
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
allowance. These immutable/append APIs provide no automatic migration, implicit
repair, branch replacement or live V2 daemon. Whole-branch publication belongs to
the separate replacement store below.

The separate [replacement preparation contract](docs/STORAGE_REPLACEMENT_V2.md)
adds MSR2 profile-3 metadata without changing either existing store. Its 280-byte
record wraps the unchanged MSS2 snapshot and a bounded local generation. Decoding
requires independent network, complete endpoint and generation pins. Preparation
also derives the entire current record from the accepted current history, rejects
stale or altered metadata, and borrows the whole preferred candidate with its own
accounts, membership and counters. This preparation function remains pure: it
prepares no files and publishes no selection.

The [replacement store contract](docs/STORAGE_REPLACEMENT_STORE_V2.md) adds
`ReplacementStoreV2`. It revalidates the independently pinned current disk history
and complete preferred candidate under one stable exclusive root lock. The
candidate's exact manifest and all blocks are synchronized in a separate generation
directory before the MSR2 marker is published. After root synchronization, only the
fully checked old branch copy is removed, one exact file at a time. Memory advances
after final clean replay and synchronization. The entire accounts/membership state,
permanent records, approvals, sequences and resource counters move together.

A clean store contains one selected generation; a replacement temporarily uses at
most two. Every disk discrepancy or failed update fences ordinary access and further
writes. Reopening and consuming reconciliation require independent full endpoint
and generation pins and accept only a clean complete selected layout. Pending
selectors, extra generations and partial cleanup remain preserved and rejected.

The separate [replacement recovery contract](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md)
defines explicit `Complete` and `RetainOld` actions. A request borrows both exact
accepted histories and retains their independently selected full checkpoints and
local generations. Recovery first checks the complete selected history and all
remaining prescribed files. It can finish authorized candidate byte prefixes or
remove that staging while old remains selected. Once the candidate is selected,
only its completion and exact old-copy cleanup are allowed; `RetainOld` cannot
roll it back. Arbitrary surviving subsets are checked file by file, and unknown or
different bytes remain preserved and rejected. The same exclusive root ownership
continues through final clean replay/synchronization. Errors return no writer.
Missing/corrupt selection and interrupted initialization remain unsupported.

The separate [V2 peer-history contract](docs/PEER_HISTORY_V2.md) defines MPN2
Begin, manifest, full-block and End frames. Its consuming receiver checks the
network independently, validates every actual block through `ChainV2`, and exposes
a candidate only after terminal consistency and full accounting pass. A peer's
claimed endpoint identifies its promised transfer; it cannot select the local
canonical chain. Header admission checks phase, length and remaining budgets
before the TCP adapter allocates its payload. Any receiving error consumes
the attempt and its private partial chain. The immutable sender produces one frame
at a time from an accepted branch. This F1 component opens no socket and publishes
no stored selection. The separate F2 session and adoption layers compose it below.

The [V2 session contract](docs/PEER_SESSION_V2.md) adds one fixed 72-byte MPQ2
request and one complete unchanged F1 response per TCP connection. Both endpoints
use independent configuration/genesis pins. The requester must receive a matching
End and response EOF before obtaining a candidate. A non-Clone session owner permits
one active connection, at most eight sequential attempts, a finite cumulative
read/write allowance and one absolute deadline of at most 60 seconds per attempt.
Partial reads/writes and failed attempts remain charged; slow fragments cannot
refresh that deadline. These are per-owner bounds, not global availability or
hard CPU-preemption guarantees.

The [candidate adoption contract](docs/CANDIDATE_ADOPTION_V2.md) checks the complete
received candidate against the locally selected current history and independently
retained current checkpoint. Tied or shorter histories keep current without writes.
A preferred branch is published through the existing owned replacement store.
Both exact histories and local checkpoints are retained before publication; failure
preserves ownership for explicit recovery and exposes no ordinary writer. These
recovery inputs remain in memory in the F2 API. The A1 node layer below persists
restart authority before calling storage. Fresh initialization remains a separate
explicit operation, never a fallback after an existing-store open fails.

The [node-control contract](docs/NODE_CONTROL_V2.md) adds `mirror_node::v2`.
`NodeControlV2` owns a separate control directory before acquiring the existing
store lock. Its exact 464-byte MNC2 record retains the trusted full checkpoint and
local generation; its bounded MNI2 intent contains both complete accepted histories.
Both histories are replayed and compared against canonical metadata before the
intent is published and synchronized. Existing store publication follows; the new
confirmed record is synchronized before intent retirement. All prior storage,
network, ledger and consensus formats remain unchanged.

Clean restart fully replays the selected store against the retained record and
independently selected C/genesis. Read-only status reports Missing, Clean or
RecoveryRequired without creating, synchronizing or repairing files. Published
intent requires explicit `Complete` or `RetainOld`; a selected candidate cannot
roll back. Exact candidate-confirmation prefixes are completed only with the full
intent and a clean candidate store. Incomplete intent, partial initialization,
unknown/corrupt input and unsafe paths remain preserved and rejected. Errors
consume the ready node owner and return no ordinary storage writer. The operator
and parent directories are trusted; checksums and cooperative Linux locks do not
establish protection against a hostile owner or physical power-loss guarantees.

## Verification checkpoints

Current MB-04A1 source is
[`32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0`](https://github.com/Ahmed-Sobeih/mirror/commit/32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0),
with tested tree `d79080968d822a2267ce5f1b1196efcf16d2e546` and parent
`37010bc5c5827f5a8dd886af0e79d079ebfe1d4e`.
All required local gates passed **1,083 runtime tests, eleven compile-fail doctests,
all required release suites, twelve independent references and sixteen synthetic
examples**. The 73 new cases comprise 24 control-record/intent codec cases, 31 public
control integration cases, twelve actual-process restart cases and six private
failure cases. The new node release gate passed **73 runtime tests and one doctest**.
Three ignored subprocess helpers run through parent tests and are not extra passes.

[Exact-source PR CI 45861912935](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45861912935)
passed all configured gates. Full job `140054826270` was inspected; its synthetic
merge checkout `62437dc1c338156fb984349b791efc7a4ad34ebb` has no file differences from the implementation
source. Later documentation-head CI is recorded separately on PR #1. Preservation
checks found **140 of 142 retained runtime/dependency/reference/fixture files
byte-identical**; only the node manifest and lockfile add a local personhood
**development** dependency. No external package versions change. Exact acceptance
and the test map are in [CURRENT_STATUS.md](CURRENT_STATUS.md) and
[test-matrix section 19](docs/MEMBERSHIP_TEST_MATRIX.md#19-mb-04a1-durable-node-control-and-restart-authority).

The following MB-03F2 source is a **historical accepted checkpoint**:
[`851193ea6428198441e6cdf29361b1885fae157b`](https://github.com/Ahmed-Sobeih/mirror/commit/851193ea6428198441e6cdf29361b1885fae157b).
All required local gates passed **1,010 runtime tests, ten compile-fail doctests,
all required release suites, eleven independent references and fifteen synthetic
examples**. The 59 new runtime cases include actual TCP processes, malformed/stalled
traffic, whole-membership-fork adoption and explicit recovery ownership. Two ignored
subprocess entry points are exercised by their parent tests; they are not additional
passing cases. Exact-source CI and later documentation-head evidence are tracked in
[CURRENT_STATUS.md](CURRENT_STATUS.md) and [MASTER_PLAN.md](MASTER_PLAN.md).
Earlier MB-03E2a/E2b and F1 acceptance remains in master-plan sections 23–25;
F2 is preserved in section 26 and A1 is recorded in section 27. The following
MB-03D record is retained as a **historical accepted checkpoint**;
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

Storage examples create temporary synthetic stores. The append example
adds accepted mixed and empty blocks, then reopens and checks the complete state.
The recovery example constructs known partial-write shapes, completes an append,
repeats it idempotently, retains another parent, and then performs an ordinary next
append. These are controlled software scenarios, not physical power-loss tests.
They do not start a persistent V2 daemon; that integration remains MB-04.
The fork-choice example compares frozen branches with equal account bytes and
different membership, demonstrates sticky ties, and borrows the complete longer
selected branch without changing either input or a store.
The replacement-preparation example checks a genuine divergent candidate and
round-trips MSR2 metadata without opening or changing a store.
The new `replacement_store_v2_demo` initializes the accepted fork at height 3,
publishes the preferred main branch at height 4 as local generation 1, retires the
old copy, and independently reopens and reconstructs the complete candidate state.
It also exercises clean reconciliation; it does not repair interrupted replacement
staging or cleanup.
The separate `replacement_recovery_v2_demo` constructs authorized partial candidate
files, explicitly completes replacement, reopens the complete result and repeats
completion idempotently. It rejects rolling back the selected candidate and shows
`RetainOld` in a second owned store. It simulates residual layouts, not a physical
power cut. Its histories and endpoint pins come from frozen independent fixtures.
The new `peer_session_v2_demo` starts separate real provider processes over localhost
TCP. It explicitly initializes and strictly reopens a fresh peer, keeps current
on an equal-height membership fork, then adopts and reopens a preferred fork whose
accounts match but memberships and population history differ. Each provider and
receiver use their own synthetic store; the example is finite and is not a daemon.
The `node_control_v2_demo` uses four separate processes to initialize trusted control,
restart and keep a tied membership fork, restart and adopt the preferred history,
then strictly reopen its persisted confirmation. The old height-3/generation-0
state has active/pending/peak 3/0/3; the preferred height-4/generation-1 state has
1/1/2, with all four accounts and both recorded approvals checked. Actual process
termination and explicit recovery at each new control-publication boundary are
covered by the separate test harness. The example exposes no interruption hook.
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
cargo test -p mirror-protocol -p mirror-network --release --locked
cargo test -p mirror-node --release --locked
python3 tools/reference_config_v2.py --check
python3 tools/reference_operations_v2.py --check
python3 tools/reference_state_v2.py --check
python3 tools/reference_blocks_v2.py --check
python3 tools/reference_ledger_v2.py --check
python3 tools/reference_chain_v2.py --check
python3 tools/reference_storage_v2.py --check
python3 tools/reference_append_storage_v2.py --check
python3 tools/reference_replacement_storage_v2.py --check
python3 tools/reference_peer_history_v2.py --check
python3 tools/reference_peer_session_v2.py --check
python3 tools/reference_node_control_v2.py --check
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
cargo run -p mirror-storage --example replacement_store_v2_demo --locked
cargo run -p mirror-storage --example replacement_recovery_v2_demo --locked
cargo run -p mirror-network --example history_receiver_v2_demo --locked
cargo run -p mirror-network --example peer_session_v2_demo --locked
cargo run -p mirror-node --example node_control_v2_demo --locked
```

The examples use synthetic attestations, issue no currency and never open the
existing chain database. GitHub Actions verifies the workspace using Rust 1.98.0.

Legacy `mirror-node mine`, `listen` and `connect` remain devnet-only. Do not run
multiple writers against one data directory. Directory replacement has only
limited crash-recovery tests; it is not a proven atomic transaction across all
filesystems or a substitute for locking.
