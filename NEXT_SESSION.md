# Next session — MB-04D1

Updated **2026-11-28**. Checkpoint **MIRROR-MB04C2-001**.

## Recover live state first

Verify live default/active branches, PRs, recent commits/comments and exact-source
CI. Preserve newer/unpushed changes and runtime data. PR #1 remains OPEN and
UNMERGED on `work/personhood-register-v0.1`; do not merge merely to publish handoff.

Implementation source: **39948d40703ee832d528f662c04f6cd5df7d69e4**.
Tested source tree: **733e483ebaa0ecfba8737fd7fe2e1da12d4cd8f1**.
Parent checkpoint: **b488287ab47dba327d6b6d1efbde86213fb3fbfc**.
[Source PR CI 46097203497](https://github.com/Ahmed-Sobeih/mirror/actions/runs/46097203497)
**attempt 1** passed all **43 steps**. Full job `140831260936` logs confirm
**1,386 runtime tests and eleven compile-fail doctests**, zero failures and
three retained parent-exercised ignored helpers. Actual synthetic checkout
`d699c9be6e52114b2de1481e55d3b50f9a05fd4d` has no file differences from the implementation source.
All **12 published implementation/test/contract files** were read back byte-for-byte.
Source acceptance: **VERIFIED_COMPONENT — all 38 local gates and exact-source CI**.
The later checkpoint changes nine status/handoff documents; all **213 frozen
noncheckpoint files** remain byte-identical. Later checkpoint-head CI is checked
separately and recorded on PR #1. This source record does not assert that a future
checkpoint head has passed, and does not embed its own future commit SHA.
PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`; main remains
`ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Current position

**MB-04C2 is VERIFIED_COMPONENT — FINITE LOCAL POOL INTAKE AND INSPECTION.**
C2 adds the finite `pool-inspect` command around C1's accepted in-memory pool and
one clean A1 controller. C1 retention/evaluation/reconciliation, B4 finite run
rounds, B3 failover, B2 rejoin/serve, B1 provider/draining shutdown, A2 operator
commands and the existing storage/peer components retain their boundaries.
MB-04 remains IN_PROGRESS: wallet/key custody, pool persistence/gossip and retained
network-loop integration, wider APIs and broader scheduling remain unfinished.
Complete Ant Kiss remains MB-05 and final human consensus MB-06. Gate A/B remain
UNMET.

## Exact next work

**Next: MB-04D1 — local wallet and signing boundary. READY — DESIGN FIRST.**

C2 now consumes already signed operations; the next useful MB-04 increment is
an explicit local wallet boundary that can eventually supply those inputs. Begin
with repository inspection and a written threat model/API/file-lifetime design,
then select the smallest reviewable implementation slice. D1 is not implemented
by this checkpoint, and choosing this next slice does not mark broader pool or
node integration complete.

Specify fresh operating-system randomness, key ownership and separate signing,
transport and membership roles; independently selected configuration/genesis;
the exact operation fields and user intent authorized by signing; and a truthful
distinction between constructing/signing bytes, pool admission, ledger execution
and canonical inclusion. Do not let an untrusted input choose its signer, network,
recipient, amount, sequence, anchor or authorization window silently.

Choose deliberate local secret input/output, authenticated protected storage,
file permissions/identity, bounded decoding/KDF work, backup/export/recovery,
cancellation and error semantics before implementation. Review established
cryptographic libraries and official documentation if new dependency support is
needed; pin exact reviewed versions and do not invent encryption or KDF schemes.
Document the limits of memory clearing, filesystem persistence and same-user
trust honestly. Keep real private keys, passwords, recovery seeds and biometric data
out of argv, logs, fixtures, repository files and diagnostics. Tests use fresh temporary
or explicitly synthetic public development material only; known demo keys are
not real wallets.

Keep signing local and explicitly authorized. Do not add automatic mining,
append/adoption, transaction gossip, pool persistence, a remote submission/signer
endpoint, default key selection or B4 loop integration as incidental work.
Existing canonical operation encodings, no-fee/no-mint development behavior,
monetary decisions, enrollment profiles and open privacy/accessibility/consensus
gates remain unchanged. Design separately any later persistence/gossip or retained
pool lifecycle across chain changes; C2 still discards its owner after each call.

Retain all existing gates and test bodies. Verify exact implementation source,
full local/CI evidence and published readbacks before the later nine-document
checkpoint; verify checkpoint-head CI separately and refresh `the-prompt`.

## Preserve the accepted C2 operator boundary

The separate finite `mirror-node-v2 pool-inspect` command requires thirteen
deliberate option/value pairs: the existing roots and independent configuration/
genesis pins, one input path, input byte/count caps, all five C1 pool limits and
an explicit evaluation count. The existing 32-argument, 4096-byte-token and
16384-total-byte ceilings remain. Every argument and scalar relation is checked
before effects; old commands reject C2-only options. No default network or stdin
fallback is introduced.

The file is a bounded local regular file of LF-terminated lowercase hex records
around unchanged signed V2 operation bytes. Empty input is valid; nonempty
records have 2..582 even hexadecimal characters and one LF. Blank/uppercase/CRLF/
odd/nonhex/truncated framing fails before A1 opens. Input caps are independently
1..149248 bytes and 1..256 records; every duplicate counts. Retention, batch and
abstract work retain C1's separate limits. Signed input cannot choose its network.

The input path rejects lexical `..`, stdin-like `-`, symlink parents/leaves,
nonregular objects and multiply linked files. Read-only `O_NOFOLLOW|O_NONBLOCK`
opening, pre/open/post device/inode/type/link/length/mtime/ctime checks and a
fallibly allocated length-plus-one EOF sentinel reject observed changes. The
descriptor closes before A1 opens. Parent namespaces remain trusted: metadata
checks are not hostile-writer snapshot isolation, and nonblocking open prevents
a raced FIFO's writer wait without imposing a regular-filesystem deadline.

One sticky signal guard, one clean `NodeControlV2::open_writer` and one C1 pool
span the invocation. The real immutable accepted chain remains owned throughout
admission/evaluation. A1 still replays and synchronizes; unchanged application
bytes and selected checkpoint do not mean no synchronization. Missing, pending,
corrupt, wrong-network or busy roots retain their existing refusal/recovery rules.

Complete framing precedes sequential FIFO admission. Any file, operation,
capacity or admission-work refusal ends with a static diagnostic, exit 1 and no
normal partial-success result. The ephemeral pool is discarded. Exact duplicates
preserve their retained position and charged identification work, including at
full retention. No failure or retry replenishes the owner allowance.

`--evaluate-count 0` skips evaluation explicitly. A positive count evaluates
exactly that deduplicated FIFO prefix once; a count above retained length is
blocked instead of shortened. Success returns an evaluated count/framed size.
Expected operation/body/ledger/count/byte/work refusal returns `evaluation=blocked`
with exit 1, a closed reason and an available zero-based FIFO ledger index.
Allocation, arithmetic and internal-state failures remain operational diagnostics.
Typed nested membership errors are classified without formatting their text.

One fixed 31-field report names the complete observed checkpoint, input/retention/
duplicate/work counters, every supplied limit and evaluation result. It explicitly
reports `canonical_inclusion=not-checked` and `pool_disposition=discarded`.
The formatter checks scalar, limit, identity, count and evaluation consistency;
normal output and diagnostics remain within 4096 bytes. No raw paths, input
bytes, unbounded operation-ID list or nested error strings are emitted.

Sticky shutdown is checked around bounded reads and successful A1/C1 calls;
already admitted synchronous work finishes or fails before another check. A real
returned error takes precedence over relabeling it as shutdown. The pool,
controller, input bytes and signal guard drop before formatting/stdout. Lost
output retains conservative completion uncertainty and never triggers a retry.

C2 does not reconcile or scan accepted history, retain a pool after exit, sign,
mine, prepare/append/adopt a block, initialize/recover, use a peer or integrate
with B4's run loop. Contextual retention, exact batch executability and canonical
inclusion remain separate. C1/A1 public APIs, lower-layer code/encodings, crate
versions, Cargo.lock, workflow commands and timeout remain unchanged. The sole
dependency access adjustment promotes the existing local mirror-personhood path
dependency for typed nested error classification; no new crate is introduced.

## Preserve the accepted C1 pool boundary

The new `mirror_node::v2::pool` component borrows an actual accepted `ChainV2`.
Its private binding contains configuration/genesis, height, tip and accepted ledger
root. Five deliberate local limits cap retained entries (at most 256), encoded
operation bytes (74,496), one batch's operations (64), complete framed block bytes
(19,048), and cumulative abstract work (67,108,864). Individual input is capped at
291 bytes before decoding/ID/signature work. Canonical membership `max_pending`
remains unrelated, and canonical operation/body limits apply independently.

Admission decodes the exact existing signed operation, deduplicates its complete
signed ID against retained entries and validates it for the selected next block.
Stable FIFO retention permits context-valid overspends, sequence gaps, competing
sequences and unknown membership targets. An exact duplicate preserves its position
even at full capacity; a distinct full-pool input never evicts another entry.
Admission does not scan history, so an already-included but still context-valid
input may be retained until evaluation/reconciliation. Future windows and invalid
expiry/anchor/signature/configuration reject. No fee replacement is invented.

Evaluation chooses exactly one requested nonempty FIFO prefix. Local count/framed-
byte and canonical complete-body preflight precede copying or execution. One full
`LedgerV2::stage_operations` call preserves original mixed order and atomic accounts/
membership semantics, including separate account and member sequence domains.
The staged ledger is discarded even on success; only immutable ordered input and
parent/revision observations are returned. Failures retain original indexed ledger
errors and leave all pool entries and accepted chain state unchanged.

Explicit reconciliation handles the same parent, append or accepted replacement
within the same configuration/genesis. It reserves complete work before history
ID hashing, contextual checks or temporary allocation, scans accepted history
once. Each input's removal reason prefers exact inclusion over contextual
invalidity; removal reports and survivors keep FIFO order. Context-valid stateful
blockers remain. Parent,
entries, exact bytes and revision publish together; budget/allocation/arithmetic
failure preserves the prior binding and semantic entries. Removed inputs are not
automatically resurrected after a replacement.

Work is cumulative for one non-Clone owner. Identification charges failed/duplicate
attempts, each evaluation charges the current parent-state size again, and every
reconciliation charges accepted history plus bounded retained-ID comparisons and
revalidation. No failure, clear, removal or parent change refunds work. The meter
is an explicit abstract budget, not CPU/RSS/wall-clock or a global rate guarantee.
Every successful reconciliation and actual entry mutation invalidates earlier
batch observations. Currentness checks the real chain, full parent, revision and
exact prefix; a different pool's different entries cannot match on revision alone.

The component never returns/binds staged state, appends/mines/prepares blocks,
signs, mutates A1/store, persists, gossips, exposes submission CLI/API ingress or
assumes wallet custody. Existing commands do not create a pool automatically.
Retained input, contextual validity, evaluated execution and canonical inclusion
remain separate. Actual block acceptance still checks complete blocks, timestamps,
work, roots and cumulative chain budgets. Zero-fee/no-mint development semantics
and all monetary, enrollment, privacy, device and accessibility decisions remain.

## Preserve the accepted B4 run boundary

The separate `mirror-node-v2 run` command requires all twelve deliberate
options. It uses B3's ordered one-to-four numeric peer list and existing common,
listen, attempt, byte, timeout and admission options, plus `--max-rounds` in 1..4
and `--round-wait-ms` in 0..60000. The old 32-argument, 4096-byte-token and
16384-total-byte limits remain. All validation precedes effects; existing commands
reject the new flags and `run` rejects the single-source `--peer` flag.

Strict clean A1 opening occurs once. One controller, sticky signal guard and
cumulative F2 owner span every acquisition, admitted keep/adopt, service phase and
wait. Successful private round helpers return the same owner after closing their
listener. Earlier commands remain dropping wrappers. Neither root is reopened or
released between rounds; at most one stream is active and acquisition pauses service.

Each entered round restarts the same ordered list at entry one, with one request
per entry and B3's unchanged closed typed advancement policy. Checked round-local
deltas determine peer ordinals; the actual cumulative owner enforces budgets.
First complete validated history, including terminal EOF, ends acquisition even
when stale or tied. All-peer or fatal failure ends the invocation even after
earlier successful rounds, without another-round retry or serving an old branch.

A round completes only after validated receipt, admitted successful keep/adopt and
normal service return. Service may consume the remaining allowance; no attempt is
reserved for a later round. Overall stop priority is shutdown, wire-limit,
attempt-limit, round-limit. Only a normal service admission expiry with allowance
and rounds remaining permits a bounded signal-aware wait and another acquisition.
Waits hold the roots but no listener or stream, and never occur after a final stop.

The unchanged invocation caps are 1..8 total attempts and 1..157129488 actual I/O
bytes, with each attempt timeout and per-round service admission in 1..60000 ms.
Failed/partial traffic is charged without refund. The terminal EOF probe still
needs one unused byte. For total attempts M, timeout T, round limit R, admission A
and wait W, `M*T + R*A + (R-1)*W <= 900000 ms` bounds configured network,
admission and wait allowances. Opening, synchronous replay/adoption/processing,
scheduling and output are separate; this is not a hard wall-clock, CPU or RSS bound.

Shutdown before adoption discards a validated candidate; admitted A1 completes or
fails. An active service request drains under its original deadline. Once any
round admits keep/adopt, every later fatal error preserves its original class,
possible completion and inspect-status guidance, including errors in later receipt
or bookkeeping. Keep-current admission also establishes that sticky context.

The fixed normal record has `30 + 16*C + 7*I` fields for completed rounds C and
optional incomplete acquisition I, with C+I<=4. It distinguishes initial, each
completed and final selected checkpoints, checked phase totals and retained prior
errors. Error context adds fifteen fixed keys and eight for a current acquisition;
it does not assert a selected checkpoint or unreturned service outcome counts.
Invalid optional evidence falls back to the safe original diagnostic. All sockets,
roots and signal guards are dropped before final output; output loss never repeats work.

## Preserve the accepted B3 failover boundary

The separate `mirror-node-v2 rejoin-failover-serve` command accepts one to four
ordered numeric peers in a single required `--peers ADDRESS[,ADDRESS...]` token.
The other nine deliberate options, 32-argument/4096-token-byte/16384-total-byte
limits and effect-free validation remain. Reject empty/malformed/scoped lists,
duplicate numeric endpoints including IPv4-mapped aliases, DNS and zero ports.
Original parsed connection addresses are retained; no distinct-physical-node
or peer-authentication claim follows from endpoint checks.

Strict clean A1 opening precedes all contact and binding. One controller, sticky
signal guard and cumulative F2 owner span a finite ordered acquisition prefix,
at most one admitted keep/adopt and fixed-history service. Each configured entry
is requested at most once. Classify the typed session/history error before
sanitizing it: only the explicit contract allowlist advances. Allocation,
authority, accounting/global-resource and all unlisted errors remain fatal,
including ambiguous nested configuration/operation/ledger failures. A failed
receive plus shutdown remains an operational error, with no next-peer contact.

The first complete validated candidate permanently ends acquisition, including
identical/shorter/tied keep-current choices. Matching End and terminal EOF remain
mandatory. The shared B2 post-receive routine handles adoption admission, existing
A1 durability, retained-controller service and checked phase deltas. Stop before
admission discards a validated candidate; admitted adoption finishes or fails.
All-peer failure never serves the old branch as a fallback.

Configured caps remain 1..8 total receive/service attempts and 1..157129488
actual I/O bytes;
each original attempt timeout and post-adoption service admission remain
1..60000 ms. Failed/partial traffic is charged without refund or replenishment.
The EOF probe needs one unused byte even though zero-byte EOF is not charged.
A final allowed successful receive can adopt then stop normally without binding.
For P peers, M attempts, timeout T and admission A, conservative configured phase
allowances are min(P,M)*T+A+T (at most 360000 ms) and M*T+A. All service attempts
remain within M; these are not hard wall-clock/CPU/RSS guarantees. Local opening,
adoption, nonpreemptible processing, scheduling and output retain their qualifications.

Normal output has 32..35 fields: B2's 26 fields plus bounded acquisition totals,
first validated ordinal, latest receive error, terminal acquisition reason and
one static result per configured entry. Service rejection evidence stays separate.
Errors attach sanitized acquisition evidence without selected-pin assertions.
Every adoption error and later fatal setup/service failure retains its category,
possible-completion warning and inspect-status guidance. Invalid optional trace
formatting cannot erase that core diagnostic. All sockets/owners/signal guards
are released before final output; output loss never repeats the operation.

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

Local acceptance used pinned **Rust 1.98.0** and locked dependencies. All
**38 required gates** passed, with all **222 tested source/checkpoint inputs
unchanged during execution**:

- **1,386 workspace/all-target runtime tests**, zero failures: 1,333 retained
  plus **53 new**. Parameterized cases, subprocesses and release reruns are
  not additional tests.
- **Eleven compile-fail doctests**, unchanged; three retained ignored subprocess
  helpers remain exercised by parent tests.
- All seven required release commands: personhood 234+3 doctests, core 121,
  state 49, consensus 23, chain 85+3, storage 350+2, protocol 38, network 108+2,
  node **367+1**. All **twelve references** and **sixteen examples** passed.

New executable test inventory:

- `crates/mirror-node/src/bin/mirror-node-v2/args.rs`: **7** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/errors.rs`: **1** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/main.rs`: **1** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/output.rs`: **7** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/pool_input.rs`: **8** new tests.
- `crates/mirror-node/src/bin/mirror-node-v2/pool_inspect.rs`: **9** new tests.
- `crates/mirror-node/tests/pool_commands_v2.rs`: **20** new tests.

Focused binary verification first passed all 114 then-registered tests. Source
review then tightened impossible typed-body representation/context/root errors
to operational invariants and added one classifier test, which passed separately;
the final full gates exercise all 115 binary tests. These are 33 new private
tests plus the retained 82-test binary suite.

The first process-suite run passed 19 of 20 functions. Its sole failure was the
test harness's optional Unix-domain socket creation returning EPERM on this host,
before the operator command ran. That complete failed log is retained. Only the
optional socket fixture and unused import were removed; missing-file, directory,
writerless FIFO, symlink-parent/leaf and hardlink checks remain. The final focused
suite passed all 20 functions in debug and release, with no ignored cases. Socket
path rejection is not claimed as observed process evidence. No production
behavior or retained test was weakened to accommodate the host limitation.

The new process suite covers full framing admission before a busy controller;
empty input; malformed operation encodings; exact maximum 149248-byte/256-record
input and distinct count/byte refusals; all four operation kinds; duplicates at
full retention and exact FIFO order; independent retention caps; signature/context
refusal; overspending, gaps/conflicts, funding order and a late mixed-operation
failure; literal cumulative work and body quotas; missing/wrong/busy/corrupt/
unknown/pending A1 state; restart; bounded records; broken stdout; and actual
blocked stdout with input and both authority owners released. Observed SIGKILL
there occurs after computation/resource release, not during admission or staging.

The initial publication `e2c1c68d72380bb4c111d6839119377febed52f2` was not accepted.
Its first full local workspace gate passed all 53 new tests but failed a retained
B2 test before its chosen SIGINT: the helper found that the child had already
exited. That failed path did not retain the child's diagnostic, so the exact
exit class is unknown. The test constructed/transmitted/replayed its candidate
inside a 1000 ms session, then started another 650 ms sleep after observing EOF
receive, without establishing that 650 ms remained. The complete failed gate
and initial publication evidence are preserved separately.

The final source repairs only that test's observation timing. It precomputes
the transcript before launch, anchors the 650 ms offset to completed-request
observation before transmission/replay, sleeps only the remaining offset and
freshly observes the same one-byte EOF receive immediately before SIGINT. The
unchanged 1450 ms elapsed bound uses that earlier epoch; renewing 1000 ms after
the signal would require at least 1650 ms. All three timing constants, the full
post-signal assertions, the other 26 test bodies and all helpers are unchanged.
One exact targeted debug execution passed after the repair. Production B2 and
C2 behavior were not changed. The full final gates and exact-source CI reported
above apply to this repaired source; the earlier publication's CI alone cannot
accept the later repair.

The initial publication's PR CI run `46095907470`, job `140827064527`, ended
cancelled during the workspace gate. Its complete archived log contains all 53
new passing test names and no reported test-failure marker, but not a complete
workflow. That partial run is not source acceptance. The exact final-source run
identified above is verified independently from its full log and actual checkout.

New process tests use actual accepted synthetic chains, real signed operations,
separate roots and bounded child cleanup. Full canonical state, selected
checkpoint, control/store/input bytes and strict reopening are checked. The
observed full-pipe stdout write establishes release of both authority owners and
the input descriptor before output. This evidence is local and finite; it does
not establish production network operation or a wall-clock bound on filesystem I/O.

Private deterministic checks cover bounded framing/read behavior and the closed
error/report boundary. Static unsafe-file cases are actual process evidence.
Concurrent hostile namespace mutation, induced allocator exhaustion and signals
at every internal synchronous validation/synchronization phase are not claimed
as observed process evidence. A1 replay/synchronization retains its accepted
contract; equal file contents do not establish absence of synchronization.

Preservation audit: **71 of 72 existing test/helper files** and
**67 pre-existing private test functions** in changed binary
modules are byte-identical. The exception is the single retained B2 deadline
observer described in the evidence; all **26 other test bodies**, the rest of that
file and its complete post-signal assertions are unchanged. All lower-layer
production source, C1/A1 public APIs,
frozen vectors, reference tools, monetary policy and runtime-data boundaries
remain unchanged. The existing mirror-personhood path dependency is promoted to
normal dependency access for typed nested error classification; no new crate or
version is added, and **Cargo.lock is unchanged**. The existing workflow commands,
pinned toolchain and 20-minute job limit are unchanged. The new contract was
written before implementation, with the dependency-access refinement recorded
before coordinator implementation. No monetary, enrollment, privacy, device or
accessibility choice was silently changed.

## User policy carried forward

No founder-specific or marketing reserve. The founder is one of the first 1,000
qualifying humans eligible for the same 10 MRY; ordinary allowance remains 0.2 MRY
per qualifying historical population maximum. The combined formula and fees/rewards
are open D-01/D-04 work. Easy PoW remains a development profile. The combined
ECG/designated-pinky/supervised Ant Kiss and accessibility proposals retain their
future evaluation gates. No personal records, biometric archive or real keys belong
in code, GitHub or on-chain. No user monetary/admission/device policy changed here.
