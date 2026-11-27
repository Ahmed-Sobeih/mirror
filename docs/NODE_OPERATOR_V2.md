# V2 finite node operator commands

Initial surface: **MB-04A2**, the separate `mirror-node-v2` executable.
This contract specifies the original commands and their boundaries. The
**MB-04B1 `serve` extension** has its separate
[provider lifecycle and shutdown contract](NODE_PROVIDER_V2.md), summarized in
section 9 below. The **MB-04B2 `rejoin-serve` extension** has its separate
[finite coordination contract](NODE_REJOIN_V2.md), summarized in section 10.
The **MB-04B3 `rejoin-failover-serve` extension** has a separate
[bounded explicit-peer failover contract](NODE_FAILOVER_V2.md). It preserves
the existing commands and uses the same post-receive adoption/service path.
Executed tests and
exact source/CI acceptance belong to [CURRENT_STATUS.md](../CURRENT_STATUS.md)
and the [master plan](../MASTER_PLAN.md).

The original commands compose the accepted [node controller](NODE_CONTROL_V2.md) and
[finite peer sessions](PEER_SESSION_V2.md). They add no consensus, storage or wire
encoding, new admission rule, monetary operation, wallet, signer or daemon.
The existing V1 executable source is unchanged. The package's explicit
`default-run = "mirror-node"` preserves its prior default invocation after adding
the second executable.

## 1. Invocation and explicit inputs

Use the pinned toolchain and locked dependencies:

```sh
cargo +1.98.0 run -p mirror-node --bin mirror-node-v2 --locked -- --help
```

After building, the same arguments can be supplied directly to `mirror-node-v2`.
`help` and `--help` are accepted only as the sole argument. No arguments, unknown
commands and help combined with other arguments are usage errors. The executable
reads no command configuration from environment variables, standard input or a
configuration file. There is no implicit working-directory store selection.

Every operational command requires these options, each with a separate value:

| Option | Meaning |
|---|---|
| `--control-dir PATH` | Trusted operator-selected A1 control directory |
| `--store-dir PATH` | Separate trusted profile-3 replacement-store directory |
| `--config-hash HEX` | Independently selected 32-byte configuration hash C |
| `--genesis-hash HEX` | Independently selected 32-byte genesis hash |

Network hashes are exact 64-character ASCII hexadecimal strings. Uppercase and
lowercase input are accepted; output is lowercase. Identity comes from the
operator's independently retained network selection, never a peer response or a
scan of store directories. A checksum does not independently authenticate local
operator authority. Configuration/genesis hashes select a network, not a peer's
identity or global latestness.

The A1 path rules remain in force: nonempty, distinct, non-nested paths; existing
parents; no unsafe links, `..`, managed filesystem root or same-inode roots.
Local owners and parent directories are trusted. These checks do not defend
against a hostile same-user namespace race. Locking uses the existing cooperative
Linux directory-inode profile, with control ownership before store ownership.

## 2. Strict bounded command parsing

The complete command, including session limits and independent initialization
pins, is parsed and validated before any filesystem or socket operation.

| Input limit | Bound |
|---|---:|
| Arguments after the executable name | 32 |
| One argument | 4,096 UTF-8 bytes |
| Cumulative argument bytes | 16,384 |
| Initialization post-genesis height | 0 through 4,095 |
| Initialization local generation | 0 through the supplied height |
| One requested TCP attempt timeout | 1 through 60,000 milliseconds |
| Requested cumulative application I/O allowance | 1 through 157,129,488 bytes |

Non-UTF-8 arguments, control characters, Unicode line/paragraph separators and
bidirectional embedding/isolate controls, duplicate or unknown flags, missing
values, positional extras and flags belonging to another command reject. Flags
use separate tokens; `--flag=value` is not supported. Unsigned numbers use
canonical decimal: `0`, or a nonzero first digit followed by decimal digits.
Signs, whitespace, leading zeros and overflow reject. Paths with spaces can be
passed as one quoted token. Parser limits bound accepted command input; they are
not a guarantee about operating-system argument allocation or total RSS.

Invalid command input exits before initialization, directory creation, locking,
storage inspection, synchronization or peer contact. Filesystem state errors are
distinct operational failures after successful parsing.

## 3. `init`: explicit attachment to an existing store

This command requires the common options plus:

```text
init --attach-existing --height UINT --tip-hash HEX --ledger-root HEX --generation UINT
```

`--attach-existing` is mandatory and has no value. The other four values form the
independently retained full checkpoint, together with the common C/genesis pins.
Height is the selected post-genesis height; generation is the local replacement
counter, not a score or freshness claim. The ledger root commits the complete
accepted account and membership state. The supplied checkpoint is never inferred
from the highest directory, a stored checksum or a peer's advertised endpoint.

`init` invokes `NodeControlV2::initialize` for an **already initialized profile-3
store**. The store must strictly reopen and replay under all supplied pins. Only
the separate A1 control state is initialized. It never creates an initial chain,
mines a block, issues currency or initializes/repairs storage after a failed open.
Creating fresh accepted storage remains an explicit library operation in the
existing synthetic development workflows; this command does not introduce a
production bootstrap distribution or import policy.

Control initialization requires an absent or empty control directory. Existing
confirmed state is not overwritten. A1 may create an empty control directory
before a later validation/I/O failure; failure can also leave partial control
initialization. Such state is reported as failure and preserved. No command
automatically removes it or retries initialization. Successful output includes
`mode=attach-existing` and the fully verified selected checkpoint.

## 4. `status`: a read-only observation

`status` accepts only the common options and calls `NodeControlV2::status`.
It creates, initializes, mines, synchronizes, contacts peers and repairs nothing.
The filesystem may independently update access time while reading; this is not
an application write or an asserted no-atime guarantee.

| Observation | Output | Exit |
|---|---|---:|
| Absent or empty control, for otherwise valid paths | `status=missing`, `store_selection=uninspected` | 3 |
| Strictly replayed clean store matching complete canonical confirmed metadata | `status=clean`, `store_selection=verified`, selected checkpoint | 0 |
| Fully validated/replayed pending control intent | `status=recovery-required`, `store_selection=uninspected`, confirmed/old/candidate checkpoints | 4 |
| Corrupt, unsupported, unsafe, mismatched or unavailable state | Operational error on standard error | 1 |

Missing describes **control**, not whether a store exists. A valid pending intent
does not assert which history the store currently selects. Pending status does
not inspect that unresolved store layout, choose a recovery action or assert
that both actions are available. In particular, a confirmed-old record can
coexist with candidate-confirmation staging that permits only `Complete`.

## 5. `sync-once`: one finite receive and adoption

This command requires the common options plus:

```text
sync-once --peer IP:PORT --timeout-ms UINT --max-io-bytes UINT
```

The peer must be a numeric socket address with a nonzero port; IPv6 uses bracket
notation, for example `[::1]:9000`. No hostname resolution or peer discovery is
performed. The operator explicitly selects the address, timeout and byte budget.
Positive budgets too small to complete a transfer are valid bounds and can cause
an operational budget-exhaustion error.

After complete argument validation, execution is:

1. Strictly open the A1 ready controller under the independent identity. Missing,
   pending, malformed, mismatched or busy local state fails before TCP contact.
2. Keep that controller, including both directory locks, while a single
   `PeerSessionsV2` owner with `max_attempts = 1` requests the candidate history.
3. Require the existing MPQ2 request, unchanged full MPN2 validation, matching End,
   terminal EOF and consuming receiver completion. No partial candidate is exposed.
4. Consume the controller and accepted candidate through `NodeControlV2::adopt`.
   Whole-branch comparison and A1's durable intent/confirmation ordering are
   unchanged. Equal, identical or shorter candidates keep the selected branch.
5. Report the completed choice, previous and selected checkpoints, actual attempts
   and actual charged application I/O bytes; release the returned controller.

There is no retry, reconnect, second peer, automatic recovery or persistent
listener. A transfer error drops the ready owner and the failed session. An
adoption error returns no ordinary writer and may have changed durable state;
the operator must inspect status and use the permitted explicit action.

The configured timeout covers the F2 attempt, including connect, request, every
read/write, semantic completion checks and terminal EOF. It excludes initial
local reopening/replay and subsequent A1 adoption/synchronization. Synchronous
validation is not preempted; late completion is rejected. This is not a hard
end-to-end command-return deadline.

The I/O allowance is F2's cumulative actual application-byte budget, including
partial operations and rejection sentinels. Kernel buffering, TCP/IP overhead,
other processes, local storage reads and memory are outside it. The sole response
also retains F1's independent 19,641,112-byte framed-history limit and all existing
block/state/replay bounds. Local control and storage retain their separate read
allowances. A successful provider transfer is not an adoption acknowledgement.

## 6. `recover`: a separately requested action

This command requires the common options plus:

```text
recover --action complete
recover --action retain-old
```

Execution directly invokes `NodeControlV2::recover`; it does not first guess an
action from status. A1 reloads both exact accepted histories from durable intent,
checks all authority and recognized layout, reestablishes required durability and
delegates to the existing explicit storage recovery rules.

`retain-old` is available only when the old branch is still selected and the
recognized control/storage state permits retention. Once candidate confirmation
begins, only `complete` against a strictly clean candidate store is permitted.
An already selected candidate cannot roll back. Unsupported partial intent or
initialization, unknown bytes and foreign confirmation prefixes remain rejected
and preserved. A clean node with no pending intent rejects a recovery request.

There is no automatic action switch, repair, cleanup or retry after failure.
Successful output reports the requested action and fully checked selected
checkpoint. Errors after intent retirement can leave an already confirmed clean
state, so neither a failed command nor an error code alone identifies disk state.

## 7. Output and exit contract

Successful observations/operations use bounded `key=value` lines on standard
output, with no human prose mixed into the record:

```text
format=mirror-node-v2-1
operation=status
status=clean
config_hash=<64 lowercase hex characters>
genesis_hash=<64 lowercase hex characters>
store_selection=verified
selected_generation=<canonical decimal>
selected_height=<canonical decimal>
selected_tip_hash=<64 lowercase hex characters>
selected_ledger_root=<64 lowercase hex characters>
```

The angle-bracket fields above are explanatory placeholders, not literal output.
Pending status uses `confirmed_`, `old_` and `candidate_` checkpoint prefixes.
Synchronization adds `choice=keep-current|prefer-candidate`, `previous_` and
`selected_` checkpoints, `attempts` and `wire_bytes`. Init adds its explicit mode;
recovery adds its action. Output never claims latestness, human finality,
biological uniqueness or a verified current store when only intent was inspected.

| Exit | Meaning |
|---:|---|
| 0 | Help, clean status, completed init/sync/recover, or normal bounded provider stop |
| 1 | Operational failure, including invalid state, ownership, transport or output I/O |
| 2 | Invalid command syntax/value/limit; no filesystem or network operation started |
| 3 | `status` observed missing control |
| 4 | `status` observed valid unresolved intent |

Missing/pending during a mutating command is an operational failure (exit 1);
only status uses exits 3/4. Diagnostics use static error categories on standard
error and do not echo untrusted tokens, raw paths or sensitive payloads. The `next`
field is advisory: `help`, `verify-inputs`, `inspect-status` or `explicit-recover`.
It is not permission to repeat a mutation or choose a recovery action automatically.
Usage diagnostics include a static `reason` for the rejected argument rule.
No success record is produced for an operation that fails. The entire response is bounded
to 4,096 bytes and written through fallible output handling. A partial or broken
output channel can still prevent the caller from receiving a complete record.

Output failure after an operation can occur after durable changes. Such an error
reports `operation_may_have_completed=true` for init/sync/recover and directs the
operator to inspect status. A caller must not automatically retry a mutation merely
because standard output was lost. Output-channel writes themselves have no deadline;
the peer timeout does not bound a blocked consumer of standard output.
For `serve`, lost final output can follow completed network service and also reports
`operation_may_have_completed=true`. The provider releases every socket and both
directory owners before final output; output failure never restarts serving.

## 8. Verification and remaining work

Parser tests exercise token/count/byte bounds and semantic rejection before
effects. Integration tests launch the actual executable to check exit codes,
diagnostics, read-only observations, independent selection, successful finite TCP
transfer and complete state, local ownership, malformed/truncated/trailing input,
explicit completion/retention and no candidate rollback. Canonical pending-state
fixtures in these command tests are deliberate constructions, not claims of newly
observed physical interruption. A1's existing actual process-termination evidence
remains separately recorded in [NODE_CONTROL_V2.md](NODE_CONTROL_V2.md).

These commands remain a finite development component. MB-04B1 adds the bounded
fixed-history provider described below; MB-04B2 composes one explicit peer receive,
keep/adopt and subsequent finite serving. Fresh-store distribution, indefinitely
persistent serving, reconnect/rejoin scheduling, broader admission/multi-peer
policy, transaction pool, wallet/signer and final human consensus are separate
work. The unchanged local operator/filesystem assumptions remain material; this
contract is not an external audit, power-loss certification or Gate A/B acceptance.

## 9. MB-04B1 `serve`: bounded provider extension

In addition to the common paths and network identity, require all five options:

```text
serve --listen ADDRESS --max-attempts UINT --max-io-bytes UINT
      --timeout-ms UINT --admission-ms UINT
```

The listen address must be numeric with a nonzero port. Attempts are 1..8;
application bytes are 1..157129488; timeout and admission-window values are
1..60000 ms. Parse all inputs before effects, then strictly open clean A1 control
before binding. One held controller supplies one fixed full accepted history;
one F2 owner retains cumulative attempts and actual I/O across clients and failures.
The provider owns its one nonblocking listener and admits at most one active stream.

Linux SIGINT/SIGTERM requests sticky graceful shutdown. Idle admission stops at
the next bounded check. An already admitted connection completes or rejects using
its original timeout, then no further connection is admitted. Admission expiry has
the same final-drain rule: the serving phase has one admission window plus at most
one original session timeout, subject to non-preemptible work and scheduling.
Initial A1 opening/replay and final output are outside that interval.

No readiness/progress output is written while owners are held. After resource
release, a normal run reports `status=stopped`, the selected full checkpoint,
`reason=shutdown|admission-timeout|attempt-limit|wire-limit`, `attempts`,
`completed_sessions`, `rejected_sessions`, actual `wire_bytes` and the retained
static `last_session_error`. Peer rejection can consume allowance without making
the provider lifecycle itself fail. Exit 0 means a normal stop, not proof of remote
adoption; fatal startup/listener/local/history/output failure exits 1, usage exits 2.
The [complete provider contract](NODE_PROVIDER_V2.md) defines ordering, shutdown
races, exact limits, source/dependency exceptions and process acceptance.

## 10. MB-04B2 `rejoin-serve`: one receive, keep/adopt and finite service

Require the four common path/network options plus all six coordination options:

```text
rejoin-serve --peer ADDRESS --listen ADDRESS --max-attempts UINT
             --max-io-bytes UINT --timeout-ms UINT --admission-ms UINT
```

Both addresses are numeric with nonzero ports. The existing 1..8 total attempts,
1..157129488 actual application bytes and 1..60000-ms timeout/admission bounds
remain, as do the original parser count/byte limits and validation before effects.
The receive consumes the first attempt and shares every byte charge with later
service. A valid `--max-attempts 1` can complete keep/adopt and stop at the attempt
limit without binding or serving; no phase replenishes allowance.

Install one sticky signal guard, strictly open clean A1 control before source
contact, receive at most one fully validated candidate, and use existing A1
keep/adopt. Retain `outcome.into_control()` through service and pass the same F2
owner and signal guard into the private provider loop. Missing, pending, corrupt,
wrong-network or busy state fails before connection or bind. Receive failure
returns operational failure without serving the old branch as a fallback.

A stop observed before the request prevents it. A stop during an admitted receive
does not renew or cancel its original F2 deadline. After successful receipt, check
stop before admitting adoption; an observed stop discards the candidate and
preserves the old selection. Once adoption is admitted, unchanged A1 durable work
finishes or fails before shutdown is honored. Later service retains B1's drain
rules. The service admission window starts after keep/adopt.

After resource release, the normal record identifies previous/selected local
checkpoints, receive status, actual choice, stop reason and separate receive,
service and total counters. Completed plus rejected service sessions equals
`serve_attempts`; adding `receive_attempts` yields total `attempts`. Phase byte
counts sum to total `wire_bytes`. Local generations are not imported from peers.
The complete 26-field record and counter rules are in
[NODE_REJOIN_V2.md](NODE_REJOIN_V2.md).

Adoption errors and later fatal bind, listener, service or output errors carry
conservative `operation_may_have_completed=true` and `next=inspect-status`.
An adoption error returns no writer and makes no selected-state assertion.
Inspection and any permitted recovery remain explicit; a lost final report never
authorizes another receive or repeated adoption. Normal shutdown/limits exit 0,
operational/output failure exits 1 and invalid input exits 2. Existing commands
retain their original diagnostic and exit behavior.
