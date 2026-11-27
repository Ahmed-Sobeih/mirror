# MB-04B2 — finite V2 rejoin and serve coordination

This contract defines `mirror-node-v2 rejoin-serve`: at most one complete receive
from an explicitly selected peer, existing A1 keep/adopt, then finite serving of
the resulting fixed complete history. It composes the accepted
[operator](NODE_OPERATOR_V2.md), [provider](NODE_PROVIDER_V2.md),
[control](NODE_CONTROL_V2.md) and [F2 session](PEER_SESSION_V2.md) boundaries.
The design is specified before implementation; checkpoint records distinguish
implemented code, executed acceptance and exact published-source CI.

[B3 explicit-peer failover](NODE_FAILOVER_V2.md) specifies a separate command
that may advance through a bounded ordered list before its first validated
candidate. It shares this command's post-receive adoption and service routine;
`rejoin-serve` retains exactly one source and the failure behavior below.

The command retains one ready controller, one sticky signal guard and one
cumulative transport owner across its phases. It introduces no initialization,
automatic recovery, retry, discovery, concurrent receive/serve, scheduler, pool or
wallet. It does not establish global latestness or complete MB-04 or Gate A/B.

## 1. Explicit command and unchanged bounds

```text
mirror-node-v2 rejoin-serve
  --control-dir PATH --store-dir PATH
  --config-hash HEX --genesis-hash HEX
  --peer ADDRESS --listen ADDRESS
  --max-attempts UINT --max-io-bytes UINT
  --timeout-ms UINT --admission-ms UINT
```

All ten options are required. Both addresses are numeric IP addresses with
nonzero ports; IPv6 uses the existing bracketed numeric form. `--peer` is the
single deliberately chosen history source. `--listen` is the interface/address
for subsequent service, if that phase is permitted. No DNS, environment, stdin,
configuration-file default, ephemeral-port discovery or inferred peer is added.
The two addresses need not differ; a later bind failure has the explicit
post-adoption semantics in section 5.

Preserve the complete effect-free parser: at most 32 arguments after argv[0],
4,096 UTF-8 bytes per token and 16,384 bytes total; exact 64-character ASCII hex;
canonical unsigned decimal; and rejection of missing, duplicate, unknown,
cross-command or extra arguments, overflow and control/bidirectional characters.
The ordinary invocation has 21 argument tokens. Parsing creates no local state,
acquires no directory owner, installs no signal handler and contacts no socket.

| Limit | One invocation |
|---|---:|
| Selected source peers | Exactly one supplied address; at most one request |
| Cumulative transport attempts | Explicit 1 through 8, including the receive |
| Cumulative actual application bytes | Explicit 1 through 157,129,488 across both phases |
| Original timeout per request or admitted service attempt | Explicit 1 through 60,000 ms |
| Service admission window | Explicit 1 through 60,000 ms |
| Active connected streams | At most one; receiving and serving never overlap |
| Provider listeners | At most one, only after successful keep/adopt when permitted |
| Final result/diagnostic record | At most 4,096 bytes |

Every unchanged F1 history/frame/manifest/block/state-work bound still applies.
The aggregate allowance does not enlarge any one history or erase rejected and
partial I/O. Kernel queues, TCP/IP overhead, unread bytes and other processes
remain outside these application counters. The transport remains unauthenticated
and unencrypted; an independently selected network is not authenticated peer identity.

## 2. One ownership and authority path

After full parsing, install the existing Linux SIGINT/SIGTERM handling exactly
once, then call strict `NodeControlV2::open_writer` with independent C/genesis and
the selected paths. Missing, corrupt, pending, mismatched or busy local state
fails before either outbound connection or listener binding. A shutdown signal
does not turn a failed opening into successful readiness or choose recovery.
Successful opening retains A1's replay, exact selection and synchronization;
it is not the synchronization-free status operation.

Retain this controller and capture its full local checkpoint as `previous`.
Construct one `PeerSessionsV2` with the supplied total limits. Check the sticky
stop state before beginning the one request. If reception starts, call the
existing `request(peer, identity)` exactly once. It keeps all original request,
header-before-payload, actual execution, matching End and terminal EOF checks.
Any failure closes/discards the attempt and returns operational failure; do not
serve the old branch as a fallback, reconnect or publish a partial candidate.

Capture the owner's receive counters after successful completion. Check stop
again immediately before admitting adoption. If no stop is observed, consume the
ready controller and completed candidate through unchanged `NodeControlV2::adopt`.
Ties, identical and shorter histories keep the current branch. A strictly preferred
candidate follows the existing intent, replacement, confirmation and retirement
ordering. Copy `outcome.report()` and retain `outcome.into_control()` directly.
Never release/reacquire control or store ownership between phases.

The subsequent serving loop consumes that returned ready controller, borrows its
one fixed accepted chain and borrows the same session owner and signal guard.
The original `serve` command remains its own setup wrapper around the shared
private serving loop. Do not call that setup wrapper from `rejoin-serve`: doing so
would reopen storage, reset the sticky flag and replenish transport allowance.
No A1 or F2 public API or wire/storage/hash format change is needed.

Control-before-store acquisition and store-before-control release remain A1's
rules. Competing status/sync/recovery operations remain busy while ownership is
held. The caller obtains no raw writer or partially accepted ledger. Every
selected result contains one complete branch's accounts, memberships, approvals,
sequences, permanent records and population state.

## 3. Shared accounting and service admission

The initial receive charges the first attempt. At most seven service attempts
remain at the maximum total allowance. `--max-attempts 1` is valid: a successful
receive and keep/adopt can end normally with `reason=attempt-limit`, zero service
admissions and no listener binding. An occupied listen port must not override that
normal budget stop. There is no minimum guaranteed service count or reserved
service byte allowance; the operator chooses a sufficient total budget.

Before binding, check shutdown and the remaining aggregate bytes/attempts. If
service remains permitted, start its admission clock immediately before binding.
This window excludes initial opening, receive and adoption. Preserve B1's
nonblocking listener, at most one active service attempt, before-accept checks,
post-accept stop/time check and original-deadline final drain. An accepted stream
discarded by the post-accept race check is not an F2 admission.

Normal service stop priority remains shutdown, admission timeout, wire limit,
then attempt limit. Before a service window has begun, only shutdown and already
exhausted budgets can apply. No listener or transport owner is recreated to make
service possible after a normal limit.

F2 requires one remaining byte of capacity for its terminal EOF probe even when
EOF itself returns zero bytes and is not charged. An allowance equal to the
request bytes plus complete response bytes is therefore insufficient for a
successful receive. Preserve and test this boundary rather than assuming a
successful receive can consume the last allowed byte.

`--timeout-ms` supplies a fresh original deadline for each independently admitted
attempt, never a renewed deadline within one attempt. The configured network-phase
allowances are one receive timeout plus the service admission window plus at most
one final service timeout, totaling at most 180,000 ms at the ceilings. This is
not a hard total command deadline: opening, durable adoption, nonpreemptible
processing, scheduling and final output retain their separate qualifications.
The standalone B1 provider retains its 120,000-ms admission-plus-drain allowance.

## 4. Explicit shutdown at each phase

The one signal guard spans opening, receive, adoption and service. Its handler
only sets the existing sticky atomic flag; it performs no I/O, allocation,
formatting, locking, cleanup, recovery or process exit. Never clear the flag
between phases. Restore prior dispositions when the invocation returns.

| Observed phase | Required result |
|---|---|
| Successful opening, before the request | Normal shutdown, zero attempts, old verified selection, no adoption or listener |
| Active connection/request/reception | Let the unchanged F2 call complete or fail under its original deadline; no asynchronous cancellation, reconnection or timeout renewal |
| Failed reception with a simultaneous stop | Preserve the operational receive error and exit 1; stop must not hide invalid history, EOF, timeout or I/O failure |
| Validated receipt, stop observed before adoption admission | Drop the completed candidate; report normal shutdown with its charged receive and unchanged local selection; never bind |
| Signal after the adoption admission check | Adoption is already admitted; run unchanged A1 to success or failure and preserve its durable ordering |
| Successful keep/adopt with stop now observed | Report the actual selected result and normal shutdown; release ownership without binding |
| Failed adoption, with or without stop | Report operational failure and possible completion, with no selected-state assertion and no automatic recovery |
| Active or idle service | Preserve B1's stop-admission and original-deadline drain behavior |

The post-receive stop check is the adoption admission boundary. A signal can arrive
after that check but before an individual instruction inside `adopt`; this is an
already admitted adoption, not a promise of instantaneous cancellation. No stop
check is added inside A1's synchronous durable transaction.

Uncatchable termination remains separate from graceful shutdown. If forced death
occurs during reception, it cannot publish that incomplete candidate. If it occurs
after adoption admission, the existing A1 clean/pending/invalid restart authority
and explicit recovery rules decide the next action. Never infer rollback or choose
recovery from a missing final command response.

## 5. Failures and truthful possible-completion context

Before adoption, retain the existing static error class and next-action mapping.
Startup or receive failure produces no success record and does not enter serving.
Invalid input exits 2 before effects; operational errors exit 1. Only the separate
`status` command uses exits 3 and 4.

Once adoption is admitted, annotate any adoption error and every later bind,
listener, service or final-output error with
`operation_may_have_completed=true` and `next=inspect-status`. This is conservative
possible completion, not an assertion of successful mutation. In particular, a
post-adoption `listen_bind` error must not imply that local state is unchanged.
Preserve its sanitized class while overriding the ordinary verify-inputs advice
with inspection-first guidance. Existing commands keep their previous diagnostic
behavior; private optional failure context defaults to absent.

An adoption error returns no ordinary controller. Its record must not assert old
or candidate pins as the actual current disk selection. Fresh read-only status
and explicit open/recovery resolve the supported state under A1's rules. Errors
after intent retirement can already leave a clean confirmed candidate; do not
claim every error necessarily leaves pending intent. Selected/confirmed candidates
can never roll back to old through RetainOld.

Subsequent peer service rejections remain charged normal sessions under B1's rules.
Local accepted-history, allocation, listener/setup and invalid internal-limit
failures remain fatal. Do not downgrade a local error to a harmless peer rejection.
No diagnostic echoes raw arguments, paths, socket addresses, peer bytes or nested
error text. Final-output loss never authorizes a peer retry or repeat adoption.

## 6. Final normal record and phase-specific counters

Release every connected stream, any listener and the controller before formatting
or writing the final record. There are no readiness or progress writes while
ownership is held. A blocked output consumer may delay process exit, but cannot
prolong provider socket or directory ownership. The record remains at most
4,096 bytes and uses existing fallible writes/flushes.

Final-output loss uses the existing conservative possible-completion reporting
for every normal B2 result, including a shutdown before adoption was admitted.

```text
format=mirror-node-v2-1
operation=rejoin-serve
status=stopped
config_hash=<64 lowercase hex characters>
genesis_hash=<64 lowercase hex characters>
store_selection=verified
previous_generation=<local generation>
previous_height=<height>
previous_tip_hash=<hash>
previous_ledger_root=<hash>
selected_generation=<local generation>
selected_height=<height>
selected_tip_hash=<hash>
selected_ledger_root=<hash>
receive_status=<not-started|validated>
choice=<not-attempted|keep-current|prefer-candidate>
reason=<shutdown|admission-timeout|wire-limit|attempt-limit>
attempts=<total charged attempts>
receive_attempts=<0|1>
serve_attempts=<service-phase charged attempts>
completed_sessions=<successful service sessions>
rejected_sessions=<rejected service sessions>
wire_bytes=<total charged actual application bytes>
receive_wire_bytes=<receive-phase charged bytes>
serve_wire_bytes=<service-phase charged bytes>
last_session_error=<latest static service rejection category|none>
```

These sample values are explanatory placeholders. Every real numeric field uses
canonical unsigned decimal. `previous` always identifies this invocation's ready
local state before reception. `selected` identifies the state verified under the
retained controller; it does not prevent a later writer from changing storage.

`receive_status=validated` with `choice=not-attempted` means shutdown discarded a
complete candidate before adoption. Both checkpoints then identify the unchanged
local selection. Before any request, status is `not-started`, choice is
`not-attempted` and every transport count is zero.

For every normal B2 result:

```text
receive_attempts + serve_attempts = attempts
completed_sessions + rejected_sessions = serve_attempts
receive_wire_bytes + serve_wire_bytes = wire_bytes
```

All normal results with an initial request have one successful validated receive;
receive errors are operational failures without a normal result record. The latest
service error is not cleared by later service success. Standalone `serve` retains
its existing fields and completed-plus-rejected-equals-attempts meaning.

Normal shutdown or a configured service limit exits 0, including runs containing
rejected service peers. It does not promise a minimum number of served clients,
remote validation/adoption or globally latest state. A successful keep/adopt
reports the actual complete local branch choice, not human finality.

## 7. Required acceptance and evidence boundaries

Retain every existing runtime, compile-fail, reference, fixture, example and release
gate. Add targeted parser/output/diagnostic checks and actual command-process
acceptance for the new composition:

1. Require all explicit flags and scalar bounds before effects; preserve old
   command separation, help and byte/count limits.
2. Refuse missing, pending, corrupt, wrong-network and busy local state before
   source contact or listener bind. Preserve unsupported data and recovery rules.
3. Use three independent directories and actual executable processes: a seed
   `serve`, a `rejoin-serve`, and a third `sync-once` requester. Compare complete
   accepted accounts, memberships, approvals, sequences and population state.
   Repeat an explicit clean restart/rejoin and verify strict fresh reopening.
4. Preserve keep-current ties/shorter/identical choices and whole preferred-branch
   adoption, including the retained equal-account/different-membership fork.
   Local replacement generations need not match across the three nodes and must
   not be imported from a peer as network state or score.
5. Verify receive-plus-service accounting, last-attempt stop before an occupied
   bind address, the exact EOF-capacity boundary, and remaining bytes consumed by
   a partial/rejected service. Never replenish allowance between phases.
6. Observe a real child in its outbound receive using the exact socket endpoints,
   child descriptor and blocked syscall. Deliver SIGINT/SIGTERM there and release
   valid EOF to prove validated-candidate discard before adoption. Also exercise
   receive failure under a stop and preservation of the original deadline.
7. Verify ownership while receiving and serving; competing sync must contact no
   peer. Preserve B1 idle/active drain and actual forced-termination observations.
8. Prove post-adoption bind/output failures carry possible-completion context,
   preserve the actual selected branch and allow fresh inspection. Verify blocked
   output cannot hold the listener or directory owners.
9. Preserve pending-state refusal and selected-candidate no-rollback. Distinguish
   deliberately constructed recovery layouts from actual process interruption.
10. Describe phase evidence precisely. Polling a file alone does not prove a signal
    arrived during a particular durable substep. Exact retained A1 publication
    failure/process tests remain separate from new command-level observations.
    Use only bounded owned-child observation and cleanup, without production
    readiness/test hooks, permission changes or hidden retries.

A pre-adoption stop is tested through an actual active receive and terminal-EOF
boundary. The synchronous adoption path must not introduce cancellation into A1.
Any additional claimed signal-during-adoption evidence must establish the owned
child's actual phase and state; otherwise label it as design/composition evidence
backed by the unchanged A1 durable ordering, not an observed substep.

## 8. Preserved scope and policy

The V1 default command, A2 commands, standalone B1 provider, A1/F2 public APIs,
all consensus/storage/wire/hash bytes, reference calculations and frozen fixtures
remain. The intended source exception is private command composition and extraction
of B1's unchanged serving loop. No dependency/package selection or verification
command change is required.

The trusted local operator/parent paths and cooperative Linux ownership assumptions
remain. Complete Ant Kiss, final human consensus, issuance/rewards, protected
biometric matching and actual device integration remain their later roadmap work.
Monetary, combined ECG/designated-pinky/supervised Ant Kiss and accessibility
decisions/proposals retain their existing gates. No real biometric archive,
personal record, secret key or hardware purchase is introduced.
