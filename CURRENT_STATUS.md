# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB04B2-001 — finite V2 rejoin and serve coordination**.
Updated **2026-11-27**.

**MB-04B2 is VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION.**
MB-04B1 remains verified for finite serving and draining shutdown, A2 for operator
commands, A1 for durable local control and explicit restart recovery, and MB-03
for its finite storage/recovery/peer scope. MB-04 remains IN_PROGRESS: broader
reconnect and multi-peer scheduling, transaction pool, wallet and wider operator
APIs remain. Complete Ant Kiss remains MB-05 and final human consensus MB-06.
Gate A/B remain UNMET.

## Exact source and publication

Implementation source: **bbac1622be18c97ab640f6834a158e5378f1d172**.
Tested source tree: **d724df618f6b45429abebaa3c24bb52ad742799b**.
Parent checkpoint: **966286bec62ce422db6bc091c472d74659935344**.
[Source PR CI 45927607784](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45927607784)
passed all **43 steps**. Full job `140272547972` logs were inspected and confirm
**1,201 runtime tests and eleven compile-fail doctests**, with zero failures.
The actual synthetic PR checkout `e00fbb793f5eb417cc0f0cda1ed52443c5cef34d` has no file differences from
the implementation commit. All **13 published implementation, test and
contract files** were read back byte-for-byte against locally tested content.
Source acceptance: **PASSED — all 38 local gates and exact-source CI**.
Later documentation-head CI is checked separately and recorded on PR #1; this
source acceptance does not assert that a future head is tested. PR #1 remains
OPEN and UNMERGED on `work/personhood-register-v0.1`. Main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

The separate `mirror-node-v2` executable adds `rejoin-serve`. It performs
at most one explicit complete receive, existing A1 keep/adopt, then finite serving
of the resulting fixed history. The contract was written before implementation:
[NODE_REJOIN_V2.md](docs/NODE_REJOIN_V2.md). Existing A2 commands, B1 `serve` and
the V1 default invocation remain available.

| Boundary | Delivered behavior |
|---|---|
| Startup | Complete argument validation, one signal guard, strict clean A1 opening before either outgoing contact or bind. Missing, pending, corrupt, wrong-network or busy state refuses. |
| Receive | One unchanged F2 request, actual complete history execution, matching End and mandatory terminal EOF. A failure is fatal even if shutdown was requested; there is no serve-old fallback. |
| Adoption | A stop observed after validated receipt but before adoption admission discards the candidate. Once admitted, synchronous A1 keep/adopt finishes or fails through its unchanged durable ordering. |
| Ownership | Retain `outcome.into_control()` directly. The same controller, signal state and F2 attempt/byte allowance span both phases; no reopen, reset or concurrent receive/serve. |
| Service | Check shutdown and exhausted budgets before binding. Serve the selected whole branch with B1 admission and original-deadline drain. A last-allowed receive can adopt and then stop without binding. |
| Reporting | Release sockets and both directory owners before a bounded 26-field result. Report previous/selected pins, actual choice and separate receive/service usage plus combined totals. |

All ten options are deliberate: separate control/store paths, independent C/genesis,
one numeric nonzero-port `--peer`, numeric nonzero-port `--listen`, `--max-attempts`
1..8, `--max-io-bytes` 1..157129488, `--timeout-ms` 1..60000 and `--admission-ms`
1..60000. Existing 32-argument, 4096-byte-token and 16384-total-byte parser bounds
remain, with no DNS, environment/stdin/config fallback or effects before validation.

Receipt charges one attempt and its actual I/O. Service uses only the remainder.
An allowance equal to the 72-byte request plus complete response bytes is still
insufficient: the terminal EOF probe needs one unspent byte of capacity even though
zero-byte EOF is not charged. The service admission clock starts after adoption,
immediately before bind when permitted. Receive timeout plus admission window plus
at most one final service timeout totals at most 180000 ms at configured ceilings.
This excludes opening, durable adoption and output and is not hard CPU preemption
or a complete wall-clock guarantee. Standalone B1 retains its 120000-ms allowance.

The post-receive stop check admits adoption. Signals after that check do not cancel
A1 or imply rollback. Every adoption error or later fatal bind/listener/service error preserves
its static error class and reports `operation_may_have_completed=true` with
`next=inspect-status`; no error asserts unresolved selected pins. Final output loss
uses the existing conservative rule, including normal pre-adoption shutdown.
No implicit initialization, recovery, retry or rollback is introduced.

Normal results obey receive_attempts + serve_attempts = attempts,
completed_sessions + rejected_sessions = serve_attempts, and receive_wire_bytes +
serve_wire_bytes = wire_bytes. A validated candidate discarded on shutdown reports
`receive_status=validated`, `choice=not-attempted` and the unchanged verified
selection. Normal limits/signals exit 0; operational/output failures exit 1 and
usage errors exit 2. Only status retains exits 3 and 4. No result claims peer
adoption, global latestness, human finality or protection against a later local writer.

## Acceptance evidence

Local acceptance used Rust **1.98.0** and locked dependencies. All **38 required
command gates** passed:

- **1,201 workspace/all-target runtime tests**, zero failures: 1,167 retained
  plus **34 new** (six parser, one diagnostic and
  **27 actual rejoin-process tests**). Parameterized cases,
  subprocesses and release reruns are not extra unique tests.
- **Eleven compile-fail doctests**, unchanged; three retained subprocess helpers
  remain intentionally ignored in ordinary enumeration and exercised by parent tests.
- Required release suites: personhood 234 runtime + 3 doctests, core 121, state 49,
  consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network **108 + 2**,
  node **182 + 1**.
- All **twelve independent Python references** and **sixteen synthetic examples**;
  scoped formatting, whitespace and allocation/runtime-data checks also passed.

The 27 added actual-process tests cover complete seed `serve` -> `rejoin-serve`
-> third `sync-once` transfer with independent directories and an explicit second
restart/rejoin cycle. Fresh strict reopening compares full histories, account
bytes, membership records/bytes, retained approvals, sequences, population and
resource counters. The seed keeps local generation 0 while adopters advance to 1;
the restart keeps those local generations and complete state unchanged. The
preferred fork retains equal account bytes but different membership state.

Further cases cover effect-free parsing; local missing/pending/corrupt/mismatched/
busy refusal before contact or bind; identical/shorter/tied keep-current; shared
attempts and actual bytes; the exact terminal-EOF capacity boundary; rejected and
partial serving; malformed/trailing/incomplete/wrong-network sources; idle/active
service drain; original deadlines and admission timing; post-adoption bind/output
uncertainty; and selected/confirming/confirmed no-rollback fixtures. Constructed
pending layouts are explicitly separate from actual interrupted processes.

Bounded Linux observations identify the actual child's socket endpoints, inode,
descriptor and blocked syscall. SIGINT/SIGTERM during complete or partial receipt
permit original-deadline completion/failure; validated completion under a prior
stop discards the candidate before adoption. Directory descriptors and competing
busy commands are checked across controlled receive/service phases; source review
also verifies direct retained-controller flow. These observations do not claim
to sample every hypothetical microscopic ownership gap.

The adoption test extends a valid synthetic history by 128 mined empty blocks.
An inotify event only prompts SIGSTOP. While the actual child is verified stopped,
the test requires the complete canonical intent, independently validates both
histories and checkpoints, confirms both owners remain held and no listener exists,
then verifies caught, unblocked SIGTERM is pending before SIGCONT. Adoption finishes,
retires intent and reopens the candidate at generation 1, followed by normal shutdown
despite an occupied listen port. This observation passed in debug and release; it
establishes signal generation during live adoption, not a particular fsync boundary.

A full-pipe case observes the child blocked on final stdout while its listener is
gone and fresh status plus strict writer opening already succeed. SIGKILL cases
separately preserve the old branch during reception and the new branch during
adopted service. Existing A1 publication-failure/process tests remain the evidence
for their exact durable substeps. Zero-attempt stop before initial request is not
artificially forced by this new process suite; that branch retains source-level
lifecycle evidence. Process tests use Linux IPv4 loopback; numeric IPv6 parsing
has CLI coverage. No test adds a production hook, permission change, hidden retry,
power-loss guarantee or untested-filesystem claim.

**160 of 166 retained source/dependency/workflow/reference/fixture
paths remain byte-identical**; 6 existing private V2 command
modules change deliberately. New code adds the rejoin composition and its process
suite/helper. Earlier tests, lower-layer/V1/A1/F2/storage/consensus/monetary source,
public APIs, Cargo manifests and lockfile, all twelve references, frozen fixtures,
workflow commands and its 20-minute job budget remain unchanged. No wire/storage/hash
encoding, enrollment/device policy, dependency selection or runtime data changes.

## Exact next action

**Next: MB-04B3 — bounded explicit-peer failover. READY — DESIGN FIRST.**

Specify a separate finite acquisition command before implementation:

1. Accept a small, bounded ordered list of explicitly chosen numeric peers, with
   at most one request per entry. Define list/argv bounds and duplicate-address
   treatment; preserve all existing commands and their behavior.
2. Retain one A1 controller, one sticky signal guard and one cumulative F2
   attempt/actual-byte owner. Define the failure-advance allowlist: only selected
   pre-adoption peer/transport failures may advance. Local authority, allocation,
   internal-limit and adoption failures remain fatal.
3. Stop acquisition at the first fully validated candidate, then perform exactly
   one existing keep/adopt and B2 fixed-history serving. A valid keep-current
   candidate also ends acquisition. Do not imply best-peer selection or global
   latestness. Exhausted peers without a validated candidate fail without serving
   the old branch as a fallback; successful receipt exhausting shared allowance
   retains B2's normal no-bind stop.
4. Specify cumulative network-phase time, unchanged in-flight deadlines, stop
   between and during requests, bounded per-peer diagnostics without raw peer
   disclosure, and truthful possible-completion context after adoption admission.
5. Verify unavailable/malformed first peer -> valid second peer -> third requester,
   no later peer contact after validation, all-peer failure, shared limit exhaustion,
   pending refusal, shutdown and complete-state restart/reopening through actual
   processes and independently owned directories.

This is a planned narrow scheduler step under MB-04, not an implemented capability.
Repeated-peer reconnect/backoff, discovery, parallel sessions, multiple durable
adoptions, an indefinite daemon, pool and wallet remain later work. No monetary,
enrollment, consensus or device policy is selected. Full MB-04 and Gate A/B remain open.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies. Preserve twelve
references, sixteen examples and all required release suites. Record exact-source
acceptance separately from later documentation-head CI and refresh the handoff.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 30, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [rejoin lifecycle](docs/NODE_REJOIN_V2.md),
[provider lifecycle](docs/NODE_PROVIDER_V2.md), [operator commands](docs/NODE_OPERATOR_V2.md),
[node control](docs/NODE_CONTROL_V2.md), [peer sessions](docs/PEER_SESSION_V2.md),
[adoption](docs/CANDIDATE_ADOPTION_V2.md), [history reconstruction](docs/PEER_HISTORY_V2.md),
[replacement storage](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[explicit storage recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Recheck live refs and preserve newer/unpushed work.
