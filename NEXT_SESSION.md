# Next session — MB-04A1

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md). Updated **2026-11-26**.

## Recover live state first

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[NEXT_SESSION.md](NEXT_SESSION.md), [decisions](docs/DECISIONS.md), and
[the-prompt](the-prompt). Active branch `work/personhood-register-v0.1`; PR #1 stays
open and unmerged. Recheck live refs and preserve newer/unpushed work.

Current contracts: [peer sessions](docs/PEER_SESSION_V2.md),
[candidate adoption](docs/CANDIDATE_ADOPTION_V2.md),
[F1 history reconstruction](docs/PEER_HISTORY_V2.md),
[accepted chain](docs/CHAIN_V2.md), [fork choice](docs/FORK_CHOICE_V2.md),
[replacement store](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[explicit recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[threat model](docs/THREAT_MODEL.md) and [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md).

Read master tracker and section 26, verify refs/PR/recent work and exact-source CI.
Never reset newer work to this snapshot or infer that main contains this open PR.

Implementation source: **851193ea6428198441e6cdf29361b1885fae157b**.
Tested source tree: **cf2e4bb9d29b250a000bb4680d1571e0355d1126**.
Parent checkpoint: **f47b4306b13c4d079859472f2fdd919debd83eda**.
[Source PR CI 45750810878](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45750810878)
passed all configured gates. Full job `139677872753` logs were inspected. Its
synthetic PR merge checkout `e74753737b608408a0f424d5b9c290805fa08752` has no file
differences from the implementation source; this is verified source-tree evidence,
not an assumption based on an older green run. All sixteen published source/contract/
workflow files were read back byte-for-byte against the locally tested contents.
Later documentation-head CI is checked separately and recorded on PR #1; these
source results do not assert a future head is tested.
PR #1 remains open and unmerged on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Current position

**MB-03F2, MB-03F and MB-03 are VERIFIED_COMPONENT within finite V2 storage,
explicit recovery and peer-synchronization scope.** This closes the planned MB-03
component sequence, not the persistent V2 node. F2 retains recovery authority only
in memory; persisted operator intent and restart orchestration remain MB-04A1.
Session limits are per owner, not a persistent node's global service policy.
Daemon/reconnect/multi-peer/pool/wallet operations remain MB-04; complete Ant Kiss
MB-05 and final human consensus MB-06. Money, protected intake/matching and actual
devices retain their later decision/acceptance gates. Gate A and Gate B remain UNMET.

## Exact next work

**Next: MB-04A1 — durable V2 node control state and restart authority. READY.**

1. Read the accepted F2 session/adoption contracts and profile-3 store/recovery
   contracts. Specify a separate bounded operator-owned control directory outside
   the strict store root, with deterministic ownership/lock order. Preserve V1 paths.
2. Retain independently selected C/genesis and the complete confirmed local
   checkpoint/generation. Loading a record must not derive authority from a peer,
   selected-file checksum alone, or the highest observed generation.
3. Specify at most one pending replacement intent containing BOTH complete accepted
   old/candidate histories, exact local endpoint/generation pins and network identity.
   Fully validate bounded input and actual replay when loading it; no authoritative
   cache, unknown-artifact cleanup or path-derived authorization.
4. Persist, validate and synchronize that exact intent BEFORE any store publication.
   After confirmed publication/recovery, persist/synchronize the confirmed checkpoint
   before retiring the intent. Test the window in which disk selected candidate but
   the separate confirmed checkpoint still names old.
5. Startup/status must report an unresolved intent as recovery-required. Completion
   or retention is explicit; RetainOld cannot roll back a selected candidate. Unknown,
   corrupt, overlong, unsafe or unsupported partial control artifacts reject and stay
   preserved. Do not silently expand storage recovery or its accepted layouts.
6. Exercise independent process restart, read-only status, wrong/stale local pins,
   failed intent writes before store mutation, interruption at defined publication/
   checkpoint/retirement boundaries, exact same membership-fork state, and exclusion
   of a competing operator. Termination tests are not physical power-loss evidence.

Keep this first increment focused on the durable control-state component and a
finite restart harness. **MB-04A2** then exposes separate V2 init/status/sync-once/
explicit-recover operator commands. A persistent server, reconnect/rejoin scheduler,
global connection admission, multi-peer policy, pool and wallet remain later MB-04.
No monetary, device, biometric or final human-consensus choice blocks A1.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies: workspace tests,
doctests, all required release suites, eleven references, fifteen examples, scoped
formatting/whitespace and preservation/allocation/runtime-data audits. Keep exact
source CI separate from later documentation-head CI and refresh the handoff.

## Preserve the completed F2 boundary

The new one-shot V2 TCP path requests and reconstructs one complete accepted
history under independently selected configuration/genesis. MPQ2 is an exact
72-byte request: magic, version 2, kind 1, C and genesis. The requester half-closes;
the provider validates the request and its EOF before responding. Existing F1
Begin/Manifest/Block/End bytes and every earlier wire/hash domain remain unchanged.

The receiver calls consuming F1 header admission before payload allocation, then
checks the whole admitted payload against the session owner's remaining budget.
Every original block executes against its own accepted branch ancestry. A matching
End, terminal EOF, consuming finish and final deadline check are all required before
a candidate becomes public. Incomplete, malformed, oversized or trailing input
closes the attempt and returns no partial candidate. Reception writes no store.

One non-Clone synchronous owner permits at most eight sequential attempts, one
live stream at a time, and 157,129,488 cumulative actual read/write bytes. Failed
attempts and partial I/O remain charged. Each attempt has one positive absolute
deadline, at most 60 seconds, including connect/accept, every individual read/write,
request/response EOF and checks after semantic work. Slow fragments cannot reset
that deadline. EOF probing requires one unused byte of allowance; EOF itself
charges zero bytes. Limits are per owner, not global peer admission or bandwidth.
CPU validation is bounded but not preempted; late results are rejected afterward.

Explicit adoption consumes a completed candidate and an independently pinned owned
replacement store. It checks both network identities, the full current checkpoint/
generation and canonical metadata, then compares actual accepted whole histories.
Identical, shorter and tied candidates keep current without filesystem calls.
A strictly preferred candidate uses the existing owned replacement publication.
Before any publication, the attempt retains an exact old-chain clone, the moved
candidate, independent identity and both local checkpoints. Complete balances,
memberships, permanent records, approvals, sequences, population history and resource
counters always follow one history; no losing-branch state is grafted into it.

Publication failure retains the same store lock and both exact histories but exposes
no ordinary writer. Recovery is explicit: Complete or RetainOld invokes the existing
consuming recovery under that same lock. Recovery failure returns no handle while
retaining the exact inputs/root for a separately requested retry. A selected candidate
cannot roll back. Fresh initialization remains a separate explicit operation;
failed existing-store opening never falls back to initialization or repair.

These recovery inputs are in memory, not a crash-persistent intent journal.
Adoption runs outside the socket deadline. Extra accepted histories and replay
allocations are not covered by encoded-input ceilings. Storage's cooperative Linux
locking, trusted local owner/parents, fencing, full replay and unsupported-layout
rules remain unchanged. Persistent node control/restart authority is the next task.

## Acceptance carried forward

Local verification used Rust **1.98.0** and locked dependencies:

- **1,010 workspace/all-target runtime tests passed**, zero failures: 951 retained
  plus **59 new** (24 session integration, one private blocked-write deadline,
  23 adoption, nine real-process integration and two independent-vector tests).
  Parameterized scenarios and subprocess invocations are not extra unique tests.
- **Ten compile-fail doctests passed**, nine retained plus one proving that a
  prepared adoption cannot be published twice.
- Required release gates passed: personhood **234 runtime + 3 doctests**, core
  **121**, state **49**, consensus **23**, chain **85 runtime + 3 doctests**,
  storage **350 runtime + 2 doctests**, protocol **38**, and network
  **99 runtime + 2 doctests**. Release reruns are not new unique tests.
- All **eleven independent Python reference checks** and **fifteen synthetic
  examples** passed. Two subprocess helpers are intentionally ignored by ordinary
  enumeration and exercised by their parent tests.
- Scoped formatting, whitespace and allocation/runtime-data audits passed.
  **129 of 132 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent. The existing F1 runtime body is unchanged; its module only gains
  exports/documentation. The network manifest and lockfile add one local storage
  dependency edge, with no external package/version changes. Monetary code, V1,
  previous encodings and frozen fixtures remain byte-identical.

The new independent Python reference packs MPQ2 itself and derives four complete
request/response transcripts from the established independent Python history
generator. It does not invoke Rust or accept Rust output as its oracle. Transcript
hashes are test assertions, not new wire fields or cryptographic domains.

The finite `peer_session_v2_demo` launches three separate provider processes over
TCP and verifies explicit initialization, a kept tie, preferred whole-branch
publication and strict reopen with independently retained local pins:

| Selected result | Height / generation | Accounts | Active / pending / historical peak | MRB2 bytes / state items |
|---|---|---:|---|---|
| Fresh main history, strict reopen | 4 / 0 | 4 | 1 / 0 / 3 | 4,330 / 75 |
| Equal-height membership fork kept current | 3 / 0 | 4 | 3 / 0 / 3 | 3,206 / 58 |
| Preferred membership fork, strict reopen | 4 / 1 | 4 | 1 / 1 / 2 | 3,605 / 73 |

Each result retains two recorded approvals. Fresh main reception uses eight F1
frames / 5,406 response bytes, or 5,478 actual application bytes per endpoint with
the request. The genuine membership forks have identical account bytes; equality
checks cover the full selected state, not just balances or a reported root.

Successful fragmented process transfer reconstructs the complete history.
Failed separate-process transfers include early close/stall, an oversized fixed
header without a body, changed End and an independently frozen
actual wrong-ledger-root block. Failure preserves every selected-store byte.
Drip reads, blocked writes, missing EOF, repeated requests and cumulative quotas
are covered by the transport tests. This is tested component behavior and an
implementation review, not an external security audit or physical power-loss proof.

## User policy carried forward

No founder-specific or marketing reserve. The founder is one of the first 1,000
qualifying humans eligible for the same 10 MRY; ordinary allowance remains 0.2 MRY
per qualifying historical population maximum. The combined formula and fees/rewards
are open D-01/D-04 work, not a reason to block node persistence or to enable issuance.
Easy PoW remains a development profile; final participation must not scale with
hash rate. ECG + designated pinky + supervised Ant Kiss and the accessible route
remain combined-profile preferences awaiting their later evaluation. No real
biometric archive, personal records or real keys belong in code, GitHub or on-chain.
