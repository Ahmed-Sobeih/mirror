# Next session — MB-04B3

Roadmap MIRROR-PLAN-001. Updated **2026-11-27**.

## Recover live state first

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

## Current position

**MB-04B2 is VERIFIED_COMPONENT — FINITE REJOIN / SERVE COORDINATION.**
MB-04B1 remains verified for finite serving and draining shutdown, A2 for operator
commands, A1 for durable local control and explicit restart recovery, and MB-03
for its finite storage/recovery/peer scope. MB-04 remains IN_PROGRESS: broader
reconnect and multi-peer scheduling, transaction pool, wallet and wider operator
APIs remain. Complete Ant Kiss remains MB-05 and final human consensus MB-06.
Gate A/B remain UNMET.

## Exact next work

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

## Preserve the accepted B2 rejoin boundary

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

## Preserve the accepted B1 provider boundary

The existing `mirror-node-v2` executable now has an explicit finite `serve`
command. All A2 commands and the V1 default invocation remain available. The
complete contract is [NODE_PROVIDER_V2.md](docs/NODE_PROVIDER_V2.md).

| Boundary | Delivered behavior |
|---|---|
| Startup | Validate all arguments, install scoped Linux signal handling, then strictly open ready A1 control before binding. Missing, pending, corrupt, wrong-network or busy state refuses startup without listening. |
| Fixed selection | Retain one controller and its complete accepted history. Concurrent status/sync commands are busy; no history refresh or controller reacquisition occurs between clients. |
| Admission | One nonblocking listener, one retained F2 owner and at most one active admitted stream. Failed sessions keep their charged attempts and actual read/write bytes. |
| Shutdown | SIGINT/SIGTERM stop new admissions. An already admitted session completes or rejects under its original absolute F2 deadline. Admission expiry follows the same final-drain rule. |
| Final output | Release streams, listener and both directory owners before one bounded result record. Report full selected pins, reason, charged attempts/bytes and completed/rejected counts. |

Require explicit control/store paths, independent config/genesis hashes, numeric
nonzero-port `--listen`, `--max-attempts` 1..8, `--max-io-bytes` 1..157129488,
`--timeout-ms` 1..60000 and `--admission-ms` 1..60000. The existing parser caps remain
32 arguments after argv[0], 4,096 UTF-8 bytes per token and 16,384 total. No defaults,
DNS, environment/stdin/config fallback or side effects before complete validation.

The admission clock starts after A1 opening, immediately before bind. Check stop,
admission time, remaining bytes and attempts before accept, and check stop/time
again after accept. A raced stream closes before F2 and is not an admitted attempt.
A successful TCP handshake or kernel-backlog entry is not application admission.
One owner retains all attempt/byte charges across clients; creation of a fresh owner
cannot replenish this invocation's budget because the loop never creates one.

`PeerSessionsV2::serve_connection` adds an accepted-stream entry with an immediate
shutdown/drop guard, including preflight refusals. It shares the checked request,
EOF and history-output routine with `serve_once`; the latter retains its original
pre-accept deadline and consumed-listener ordering. The 72-byte MPQ2 request,
mandatory request EOF, complete F1 history and all original encodings remain.

The handler only sets a sticky atomic stop flag and prior handlers are restored on
exit. Final drain retains at most one original attempt timeout after the admission
window; configured ceilings total 120,000 ms. This is deadline-aware synchronous
behavior, not a hard process wall-clock guarantee: processing and scheduling are
not preempted, and initial open/replay plus final output lie outside that interval.
A1 opening may synchronize; it is distinct from read-only status. SIGKILL is forced
termination and does not establish graceful drain or physical power-loss safety.

Ordinary stop priority is shutdown, admission timeout, wire limit, then attempt
limit. Completed plus rejected equals charged attempts; the last static session
error persists across later successes. Normal limits/signals exit 0 even with
rejected peers, operational/output failures exit 1, and invalid commands exit 2.
Only status uses exits 3/4. Peer input errors can consume a bounded attempt; local
history/allocation/setup/listener errors terminate rather than masquerading as peer
rejections. Diagnostics echo no raw paths, addresses or peer bytes.

There is no readiness/progress output while resources are held. Final records stay
within 4,096 bytes. Output failure may follow completed serving and reports
`operation_may_have_completed=true` with explicit status inspection, without retry.
Provider completion reports local transfer completion, not remote validation,
durable adoption, global latestness or human finality.

## Preserve the accepted A2 command boundary

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

## Preserve the accepted A1 boundary

A separate `mirror_node::v2` library now preserves trusted local restart authority
outside the strict profile-3 store. The operator supplies separate non-nested
control/store paths and independently chosen C/genesis. Both roots use cooperative
Linux directory-inode locks, always control first and store second. The ready
controller exposes no raw store writer; failures consume it and return no writer.

`NodeControlV2::initialize` explicitly attaches control to an already initialized
store under independent full checkpoint/generation pins. It never initializes or
repairs a store after failed opening. `open_writer` accepts only clean control and
strictly replays the selected store. `status` performs no application writes or
synchronization: absent/empty control is Missing, verified clean storage is Clean,
and fully replayed pending intent is RecoveryRequired. Invalid input rejects.
RecoveryRequired reports local intent and last-confirmed pins; it does not inspect
or assert the store's current selected endpoint.

MNC2 is an exact 464-byte confirmed record with full local checkpoint and unchanged
MSR2 canonical metadata. MNI2 has a 952-byte header, both complete unchanged MPN2
history streams and a checksum; its maximum is 39,283,208 bytes. Loading binds the
external network identity, executes both complete histories, checks both canonical
records and reuses existing strict replacement/generation rules. Checksums detect
corruption; they cannot defeat a hostile operator who replaces all trusted state.

Consuming adoption accepts a completed F1/F2 candidate. Ties, shorter and identical
histories keep the current controller without filesystem calls. Strict preference
retains an old-chain clone and moves the complete candidate into intent. The exact
intent is written, synchronized, read back, fully replayed, renamed and its control
directory synchronized BEFORE invoking existing storage replacement. The ready
writer rechecks confirmed bytes, intent and exact layout immediately before that
mutation. Accounts, memberships, approvals, sequences and population history always
come from one complete accepted branch.

After storage returns clean success, the exact new confirmed record is staged,
synchronized, renamed and the directory synchronized. Only then is the exact intent
removed and that absence synchronized. A retry reestablishes intent durability
before storage recovery and confirmed durability before retiring intent; visible
renames alone are insufficient. Clean writer reopening also resynchronizes the
confirmed record/control directory/parent before another adoption.

Startup never completes or retains pending intent automatically. Explicit recovery
selects Complete or RetainOld through existing storage rules. Once candidate
confirmation has begun, only Complete with a strictly clean candidate store is
permitted. Old-selected storage contradicting candidate confirmation rejects before
mutation. A selected candidate cannot roll back.

`intent.pending`, incomplete control initialization, unknown entries, corrupt data,
overlong files and unsafe links are preserved and rejected. The only recoverable
confirmation staging is an exact 0..464-byte candidate-record prefix beside valid
old confirmation and complete intent; explicit Complete extends it without truncation.
Candidate confirmation plus remaining intent is also explicitly completable. A
failure after intent retirement may leave an already confirmed clean state; errors
do not always leave intent and do not promise arbitrary corruption repair.

Control reads check type, length, inode and hardlink count, allocate only after
checked length-plus-one admission and charge one 314,269,392-byte per-call allowance.
Storage keeps its separate bounds. Encoded sizes are not RSS/time guarantees;
history copies, checksum buffers and replay consume additional memory. Parent paths
and local operators remain trusted. Process termination tests are not physical
power-loss certification or hostile-same-user race protection.


## Acceptance carried forward

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

## User policy carried forward

No founder-specific or marketing reserve. The founder is one of the first 1,000
qualifying humans eligible for the same 10 MRY; ordinary allowance remains 0.2 MRY
per qualifying historical population maximum. The combined formula and fees/rewards
are open D-01/D-04 work. Easy PoW remains a development profile. The combined
ECG/designated-pinky/supervised Ant Kiss and accessibility proposals retain their
future evaluation gates. No personal records, biometric archive or real keys belong
in code, GitHub or on-chain. No user monetary/admission/device policy changed here.
