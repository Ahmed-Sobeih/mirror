# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB04A2-001 — finite V2 operator commands**.
Updated **2026-11-26**.

**MB-04A2 is VERIFIED_COMPONENT — FINITE V2 OPERATOR COMMANDS.**
MB-04A1 remains verified for durable local control and explicit restart recovery;
MB-03 remains complete within its finite storage/recovery/peer scope. MB-04 remains
IN_PROGRESS: persistent serving, reconnect/rejoin and multi-peer scheduling,
transaction pool, wallet and wider operator APIs are separate work. Complete Ant
Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain UNMET.

## Exact source and publication

Implementation source: **eea42a403fef2a1442e553df2a6823b1b25da616**.
Tested source tree: **cda30cd7e8b564a82d09a46b7067012e7112e2dd**.
Parent checkpoint: **957599df1b8a424ba333d0e0c10247b2807c3a09**.
[Source PR CI 45882757484](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45882757484)
passed all **43 steps**. Full job `140125894368` logs were inspected and confirm
**1,137 runtime tests and eleven compile-fail doctests**, with zero failures.
Its actual synthetic PR checkout `7dbc9921785f52c4c0136c27dc5890f9b0188e2c` has no file differences
from the implementation commit. All **9 published implementation/contract
files** were read back byte-for-byte against locally tested content.
Later documentation-head CI is checked separately and recorded on PR #1; source
acceptance does not assert that a future head is tested. PR #1 remains OPEN and
UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

A separate `mirror-node-v2` executable composes the accepted A1 controller
and F2 peer receiver. The existing V1 executable source is unchanged; the package
sets `default-run = "mirror-node"` to preserve its prior default invocation.

| Command | Delivered behavior |
|---|---|
| `init --attach-existing` | Initialize only the separate control state for an **already initialized** profile-3 store, under independent C/genesis and all height/tip/ledger-root/generation pins. No store creation, mining, overwrite or failed-open fallback. |
| `status` | Call only A1 read-only status. Missing control exits 3, clean verified selection exits 0, validated pending intent exits 4; invalid/unsafe/busy state returns operational failure. |
| `sync-once` | Open and retain a clean ready controller before connecting, request exactly one candidate with one bounded F2 owner, then consume it through A1 adoption. No retry or automatic recovery. |
| `recover --action complete\|retain-old` | Forward the explicit action to A1's existing fully checked recovery. Selected/confirmed candidates cannot roll back. |

Every operational command requires explicit `--control-dir`, `--store-dir`,
`--config-hash` and `--genesis-hash`. The complete argument list is validated before
filesystem/socket operations: at most 32 arguments after argv[0], 4,096 UTF-8 bytes
per token and 16,384 total bytes. Reject duplicate/unknown/cross-command options,
non-UTF-8/control input, malformed exact-64 hexadecimal hashes, noncanonical
unsigned decimal, overflow and invalid init height/generation. Init height is
0..4095 and generation cannot exceed height. No env/stdin/file-config fallback.

Sync requires a numeric peer address with nonzero port, an explicit 1..60000 ms
timeout and positive I/O allowance up to 157,129,488 bytes. Its one F2 attempt
retains all prior header admission, replay, matching End and terminal EOF checks.
The timeout covers the session, not local open/replay, adoption or stdout; encoded
I/O bounds are not RSS, CPU preemption or global admission guarantees.

Output uses at most 4,096 bytes of deterministic `key=value` records. It reports
network and full checkpoints, successful sync choice/attempts/actual wire bytes,
or static non-echoing diagnostic categories. Pending fields use confirmed/old/
candidate labels with `store_selection=uninspected`; they do not assert current
store selection or infer which recovery actions are permitted. Usage exits 2
before effects; operational/output failures exit 1. Help exits 0 without paths.
Only status uses missing/pending exits 3/4.

Output failure after a mutating command can follow durable completion. The process
reports `operation_may_have_completed=true` and directs inspection instead of
retrying. Owners are released before stdout writing. Partial initialization and
unsupported/corrupt/unknown artifacts remain preserved under A1's rules.

The complete command, output and failure contract is
[NODE_OPERATOR_V2.md](docs/NODE_OPERATOR_V2.md). Existing A1/F2 persistence,
replay/ownership and recovery rules remain unchanged. Fresh-store distribution,
persistent serving and broader operator functions are not supplied by A2.

## Acceptance evidence

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,137 workspace/all-target runtime tests**, zero failures: 1,083 retained plus
  **54 new** (23 parser, four bounded-output, three diagnostic and **24 actual-command
  process tests**). Parameterized cases, child invocations and reruns are not new tests.
- **Eleven compile-fail doctests**, unchanged from the accepted A1 baseline.
- All required release suites: personhood 234 runtime + 3 doctests, core 121,
  state 49, consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network 99 + 2,
  and node **127 runtime + 1 doctest**. Release reruns are not extra unique tests.
- All **twelve independent Python references** and **sixteen synthetic examples**.
  Three existing subprocess helpers remain intentionally ignored in ordinary
  enumeration and are exercised by their parent tests.
- Scoped formatting, whitespace, allocation/runtime-data and preservation checks.
  **153 of 154 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent checkpoint. Only node's Cargo.toml changes, to preserve V1 default
  invocation. Cargo.lock, every earlier runtime source, wire/hash/storage format,
  monetary implementation, reference and frozen fixture remain unchanged.

Actual executable processes verify all full init pins and refused overwrite,
read-only missing/clean/invalid/pending snapshots, rejection before effects,
zero TCP connections when local state is unready, and complete accepted state after
real TCP keep/adopt. The preferred fork has identical account bytes but different
membership history; accounts, memberships, approvals and population state are
checked through strict fresh reopening.

Invalid/truncated/missing-End/trailing histories, missing terminal EOF and an
actual one-byte session allowance reject. Explicit completion/retention, candidate
no-rollback and unknown/partial artifact preservation are tested. Canonical pending
recovery layouts are deliberate record/storage fixtures, distinct from A1's
retained actual publication-boundary process-termination evidence.

One real sync child is killed while blocked in reception: it holds both directory
owners until killed/reaped, competing commands fail before connecting, ownership
then releases and selected state remains unchanged. A deterministically closed
stdout after successful init returns an output error; a fresh status process proves
the clean completed initialization. These are finite component observations, not
physical power-loss certification, an external audit or Gate A/B acceptance.

## Exact continuation

**Next: MB-04B1 — bounded V2 provider lifecycle and shutdown. READY.**

Begin with an explicit design contract, then implement only the accepted first
provider slice through existing A1/F2 components:

1. Require deliberate control/store paths, independently selected C/genesis and
   one numeric listen address. Strictly open clean A1 state before binding. Missing,
   invalid, pending or busy local state must refuse startup without listening.
2. Specify one held controller and a fixed complete accepted history for that
   lifecycle. The current exclusive owner makes concurrent A2 status/sync return
   busy; document that behavior instead of weakening A1 locking or pretending the
   provider sees live updates. Define listener ownership around F2's consuming
   `serve_once` API before implementing a repeated accept path.
3. Define process-level admitted-attempt, active-connection, aggregate application
   byte and lifecycle-duration bounds. Recreating session owners/listeners must
   not replenish them. Failed attempts stay charged. Kernel backlog, TCP/IP overhead
   and other processes remain outside application-admission claims.
4. Define explicit idle and in-flight shutdown: stop acceptance, keep the existing
   attempt deadline, complete or reject bounded work, then release sockets and
   directory ownership. Synchronous validation is not instantly preemptible; do
   not promise immediate shutdown without a separately designed cancellation path.
5. Use real command processes to verify startup refusal before bind, malformed and
   slow peers, admission exhaustion, both shutdown states, lock release and strict
   fresh-process reopening. Preserve whole-history validation and finite resource
   assumptions.

This is **next work, not behavior supplied by A2**. Do not bundle automatic
initialization/recovery, reconnect/rejoin or multi-peer scheduling, a transaction
pool, wallet/signer or unresolved monetary/admission/device choices into the first
provider slice. No policy decision blocks specifying these dependency-ready limits.

Run all AGENTS.md gates with Rust 1.98.0 and locked dependencies; preserve twelve
references, sixteen examples and every required release suite. Record exact source
acceptance separately from later documentation-head CI and refresh the handoff.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 28, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [operator commands](docs/NODE_OPERATOR_V2.md),
[node control](docs/NODE_CONTROL_V2.md), [peer sessions](docs/PEER_SESSION_V2.md),
[adoption](docs/CANDIDATE_ADOPTION_V2.md), [history reconstruction](docs/PEER_HISTORY_V2.md),
[replacement storage](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[explicit storage recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Recheck live refs and preserve newer/unpushed work.
