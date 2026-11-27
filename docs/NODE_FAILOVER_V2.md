# MB-04B3 — bounded explicit-peer failover

This contract specifies the separate `mirror-node-v2 rejoin-failover-serve`
command before its implementation. It extends the finite acquisition phase of
[B2 rejoin](NODE_REJOIN_V2.md), then uses the same adoption admission, A1
keep/adopt and fixed-history [provider](NODE_PROVIDER_V2.md) path. Checkpoint
records distinguish this design, implemented source and executed acceptance.

The operator chooses an ordered list. Acquisition ends at the first complete,
validated candidate, including a candidate that keeps the current branch. This
is not best-peer selection, peer authentication, discovery or global latestness.
There is no repeated-entry reconnect, backoff, parallel acquisition, multiple
adoption, automatic recovery, serve-old fallback, pool or wallet.

## 1. Explicit command and peer-list admission

```text
mirror-node-v2 rejoin-failover-serve
  --control-dir PATH --store-dir PATH
  --config-hash HEX --genesis-hash HEX
  --peers ADDRESS[,ADDRESS...]
  --listen ADDRESS
  --max-attempts UINT --max-io-bytes UINT
  --timeout-ms UINT --admission-ms UINT
```

All ten options are required. `--peers` supplies **one through four** ordered
numeric endpoints in one token. The ordinary invocation remains 21 argument
tokens. Existing limits remain **32 arguments after argv[0], 4096 UTF-8 bytes per
token and 16384 total bytes**. Validate the whole invocation before installing
signals, opening paths, acquiring owners, connecting or binding. Existing
command flags, scalar validation and operational record schemas and semantics
remain unchanged. The help catalogue adds this command and its options.

Reject empty lists/elements, more than four entries, leading/trailing/doubled
commas, whitespace within the list, DNS, malformed addresses and zero ports.
IPv6 uses bracketed numeric notation. The new peer list rejects scoped IPv6
(`%` syntax or nonzero scope); existing single-address commands are unchanged.
Store the list in four fixed slots with a private validated length, without a
separate dynamically growing peer collection. A distinct `--peers` flag retains
ordinary duplicate-option rejection; the new command rejects `--peer`, and older
commands reject `--peers`.

Reject duplicate parsed endpoints, including alternate IPv6 spellings and an
IPv4 endpoint repeated as IPv4-mapped IPv6. Normalize mapped addresses only for
duplicate comparison; retain the supplied parsed address for connection. Distinct
ports remain distinct endpoints. This does not identify physical machines or
prevent different addresses from reaching the same host. No DNS, environment,
stdin, configuration-file fallback or peer inference is introduced.

The supplied list may be longer than the total attempt allowance. Later entries
then remain unattempted. Listen uses the existing numeric nonzero-port syntax;
it need not differ from a source endpoint.

## 2. One retained owner and first-valid selection

Install one sticky Linux signal guard after parsing. Strictly open one clean
`NodeControlV2` before any peer contact or listener bind. Missing, pending,
corrupt, wrong-network or busy state remains fatal and unchanged. Independently
supplied C/genesis remain authority; peer claims never select a network.

Create one non-Clone `PeerSessionsV2` and retain it through acquisition and
service. Visit an ordered prefix of the configured list, calling `request` at
most once per entry. Every call retains the unchanged F2 frame admission,
complete history execution, matching End and terminal EOF requirements. F2
closes its stream and discards partial state before returning an error. No failed
candidate, stream or receiver is resumed; attempts and actual bytes are not
refunded. Keep both directory owners continuously, including between entries.

Classify a returned typed error using section 3 before converting it to the
coarser static operator diagnostic. Only an expressly permitted failure may
advance, and only while another entry, shared capacity and continued operation
remain. No output or readiness/progress event occurs while ownership is held.

The first successful request permanently ends acquisition. Pass that complete
candidate and the retained controller through B2's shared post-receive routine:
check stop for adoption admission, then perform exactly one existing keep/adopt
and retain `outcome.into_control()` for the shared provider. Identical, shorter
and tied candidates keep current and still end acquisition. No later entry is
contacted to search for a preferred branch. Full accounts, memberships, approvals,
sequences, population and resource accounting stay with the selected whole branch.

## 3. Closed typed failure-advance policy

The following is the complete allowlist, not a category-prefix or error-text
rule. Every unlisted variant is fatal. In particular, the static category
`invalid_peer_history` is insufficient to decide advancement: it also represents
allocation and internal failures. Public lower-layer APIs and errors are unchanged.

| Typed returned error | Permitted variants |
|---|---|
| `SessionErrorV2` | `DeadlineExceeded`, `EarlyClose`, `TrailingData` |
| `SessionErrorV2::Io` kind | `ConnectionRefused`, `ConnectionReset`, `ConnectionAborted`, `NetworkUnreachable`, `HostUnreachable`, `BrokenPipe`, `NotConnected`, `TimedOut`, `WouldBlock`, `WriteZero` |
| `HistoryErrorV2` under `SessionErrorV2::History` | `WrongNetwork`, `UnexpectedFrame`, `WrongPayloadLength`, `WrongBlockHeight`, `BlockTooLarge`, `BlockBytesMismatch`, `EndpointMismatch`, `ChangedSummary` |
| Nested `ProtocolErrorV2` | `InvalidMagic`, `UnsupportedVersion`, `UnknownKind`, `InvalidPayloadLength`, `InvalidBlockCount`, `InvalidHeight`, `InvalidManifestLength`, `InvalidBlockBytes`, `BlockHeightOutOfRange` |
| Nested `ManifestErrorV2` | `InvalidMagic`, `UnsupportedVersion`, `Truncated`, `TrailingBytes`, `TooLarge`, `ConfigTooLarge`, `TooManyProofs`, `WrongProofCount`, `WrongConfiguration`, `WrongGenesis` |
| Nested `BlockErrorV2`, directly or through `ChainErrorV2::Body` | `UnsupportedVersion`, `InvalidMagic`, `Truncated`, `TrailingBytes`, `TooLarge`, `TooManyOperations`, `TooManyMembershipOperations`, `TooManyEnrollments`, `OperationTooLarge`, `WrongContext`, `OperationContext`, `OperationsRootMismatch` |
| Nested `ChainErrorV2` | `WrongConfiguration`, `WrongHeight`, `WrongParent`, `InvalidTimestamp`, `WrongGenesisTimestamp`, `GenesisHasOperations`, `WrongGenesisHash`, `WrongOperationsRoot`, `WrongLedgerRoot`; `Body` only through the block row |
| Nested `ChainErrorV2::Work` | `WrongBits`, `HashAboveTarget` |

All allocation failures, arithmetic/accounting failures, session/global resource
limits and unlisted I/O kinds remain fatal. `WireBudgetExhausted` is fatal even
when positive bytes remain: an admitted payload may not fit the remainder.
Top-level `SessionErrorV2::WrongNetwork` and `InvalidRequest` are also fatal;
the current requester does not emit them, and a future local-authority defect
must not silently become a peer refusal. Received mismatch uses the nested history
path listed above.

The first profile deliberately keeps `ManifestErrorV2::Config`,
`BlockErrorV2::Operation`, `ChainErrorV2::Ledger`, history/frame work ceilings,
fixed internal block-header length errors,
`AccountingMismatch`, `UnexpectedPhase`, `AlreadyComplete`, incomplete internal
receiver state and all other unlisted variants fatal. Some excluded nested errors
can be triggered by bad input; B3 does not promise failover for every possible
invalid candidate. A future expansion needs an explicit typed review and tests.

## 4. Shared limits, deadlines and terminal precedence

| Limit | One invocation |
|---|---:|
| Configured peers | 1..4 |
| Total receive plus service attempts | 1..8 |
| Cumulative actual application I/O | 1..157129488 bytes |
| Original deadline per attempt | 1..60000 ms |
| Service admission window after keep/adopt | 1..60000 ms |

At most `min(peer_count, max_attempts)` acquisition requests are admitted. Each
retains its original F2 absolute deadline through connect, request, response,
validation and EOF; partial I/O, signals and advancement never refresh an active
deadline. Each new explicitly permitted entry is a separate charged attempt with
its own original deadline. There is no aggregate-time knob or transport API change.

Let P be peer count, M total attempts, T per-attempt duration and A admission.
A conservative configured network-phase allowance is
`min(P, M) * T + A + T`, at most **360000 ms**. The final T is at most one service
attempt draining beyond admission, not an extra permitted attempt: all attempts
still share M. The independent bound `M * T + A` also applies; a successful final
allowed receive cannot produce a ninth attempt. Service begins only after
keep/adopt and retains B1's admission/original-deadline drain.

These are sums of configured phase allowances, not hard total wall-clock, CPU,
RSS or scheduling guarantees. Initial opening/replay, durable adoption, synchronous
nonpreemptible processing and final output keep their existing qualifications.
Standalone B1 and B2 retain their existing timing contracts.

Before each request check stop and shared remaining capacity without calling F2
merely to manufacture a budget error. After a failed request, keep its original
static error class and all charges. Determine why acquisition ends in this order:

1. A fatal typed error stops immediately (`fatal`, or `wire-limit` / `attempt-limit`
   for those actual typed session-limit errors).
2. For an eligible failure, an observed stop ends acquisition (`shutdown`).
3. Otherwise exhausted actual wire allowance ends it (`wire-limit`).
4. Otherwise exhausted total attempt allowance ends it (`attempt-limit`).
5. Otherwise exhaustion of configured entries ends it (`peers-exhausted`).
6. Otherwise advance once to the next entry, rechecking stop/capacity before contact.

Thus an unavailable first peer under a one-attempt allowance retains
`error=peer_unreachable`, with `acquisition_stop=attempt-limit`; it does not replace
the observed cause with an invented extra request. If stop is observed between
failed requests, preserve the latest failed-receive class and end with shutdown.
No unvalidated exhaustion or failed receive becomes a successful serve-old result.

Successful validation using the last allowed attempt can keep/adopt and stop
normally before bind, even if the listen address is occupied. There is no reserved
service allowance. The terminal EOF probe still needs one unspent byte of capacity;
zero-byte EOF itself is not charged. Failed acquisition, rejected service and
partial I/O all consume the same actual-byte allowance.

## 5. Shutdown and possible completion

- After successful opening, stop before the first request gives normal shutdown,
  zero attempts and unchanged verified selection.
- During a request, drain to successful complete validation or failure under its
  original deadline. A concurrent signal never converts failed receipt to success.
- After one or more failed receives, stop before another request remains an
  operational failure with retained receive evidence; no next peer is contacted.
- After validated receipt but before adoption admission, stop discards the
  candidate and returns normal shutdown with old verified selection. Earlier
  failed attempts and the validated ordinal remain reported.
- Once adoption is admitted, use the unchanged B2/A1 durable sequence to success
  or error. Do not asynchronously cancel or repeat it. Honor stop before binding
  after success; later serving retains B1's drain behavior.

Every adoption error and later fatal setup/service error retains its original
category plus `operation_may_have_completed=true` and `next=inspect-status`.
Do not assert selected pins on unresolved errors, promise rollback or assume an
error always leaves intent. Explicit recovery and selected-candidate no-rollback
rules are unchanged. Output loss retains the existing conservative completion
warning, including a normal pre-adoption shutdown, and never repeats work.

## 6. Bounded reports and per-entry evidence

Release streams, listener, both directory owners and signal guard before final
formatting/writes. Preserve the existing 4096-byte output cap and static fallback.
Normal B3 output contains B2's 26 fields with
`operation=rejoin-failover-serve`, plus these five fields and one result per
configured entry (32..35 fields total):

| Field | Meaning |
|---|---|
| `peer_count` | Configured entry count, 1..4 |
| `validated_peer` | One-based first validated ordinal, or 0 |
| `failed_receives` | Number of charged acquisition attempts that failed |
| `last_receive_error` | Latest static receive failure category, or `none` |
| `acquisition_stop` | `validated` or pre-first-request `shutdown` on normal output |
| `peer_1_result` through `peer_N_result` | `not-attempted`, `validated`, or the existing sanitized failure category |

Use fixed literal keys and static labels only. Never emit raw endpoints, paths,
arguments, nested error text or received bytes. Omit entries beyond peer_count.
The validated ordinal remains present if shutdown discards the candidate; validation
does not imply adoption. `last_session_error` remains service-only and is retained
separately from `last_receive_error`.

On operational failure attach the same acquisition context plus
`receive_attempts` and `receive_wire_bytes`; use `acquisition_stop=startup` for
signal/opening failure before a request, the terminal labels in section 4 for
acquisition failure, and `validated` after completed acquisition. Such diagnostics
contain no selected checkpoint or clean-state assertion. A failure before a charged
attempt leaves that entry `not-attempted`. Existing command diagnostics remain
unchanged through default-absent optional context. A failed final output channel
need not reproduce per-entry evidence; its existing bounded output-loss diagnostic
and exit status remain authoritative.

If internal trace validation or bounded extension fails while reporting an error,
retain the already valid base diagnostic with its original category, next action
and possible-completion flag; omit the invalid optional trace entirely. A partial
trace or generic fallback must not erase known post-adoption uncertainty.

Normal-report identities:

- receive_attempts + serve_attempts = attempts;
- receive_wire_bytes + serve_wire_bytes = wire_bytes;
- completed_sessions + rejected_sessions = serve_attempts;
- attempted entry count = receive_attempts;
- failed_receives + (validated_peer != 0) = receive_attempts;
- a nonzero validated_peer equals receive_attempts, because acquisition stops at
  the first validated member of an ordered prefix.

Reject inconsistent internal reports rather than wrapping counters. Normal
configured stops exit 0, operational/output failures 1 and usage failures 2;
status alone retains exits 3/4. Error evidence describes observed attempts, not
permission to automatically retry, recover or initialize.

## 7. Required acceptance evidence

Preserve every existing gate in AGENTS.md, locked Rust 1.98.0 dependencies, twelve
independent references, sixteen examples and all seven required release commands.
Retain existing B2 tests/helpers and all prior matrix requirements. Add meaningful
typed-policy, parser/report and actual-process tests, counting test functions once.

1. Reject all malformed/missing/mixed options, invalid lists and duplicate aliases
   before effects; verify local missing/pending/corrupt/mismatched/busy refusal
   before any configured peer or listener contact.
2. Exercise unavailable first -> real seed second -> B3 -> third requester with
   independent directories, complete-state agreement and explicit restart.
   Exercise malformed first -> valid second, including the preferred fork whose
   accounts match while membership differs.
3. Show first validated identical/shorter/tied candidates end acquisition; a later
   better peer remains uncontacted. Exercise the supported failed-peer classes,
   all-peer failure, and at most one contact per configured entry.
4. Verify shared attempts, failed bytes and EOF capacity; final allowed successful
   receive adopts then stops without binding. A fixed invalid 12-byte header after
   the 72-byte request gives a measured 84-byte failed exchange; offered malformed
   response length is not an observed consumed-byte count.
5. Preserve acquisition and service rejection evidence after later success, and
   verify cumulative and phase counters without replenishing allowances.
6. Observe actual child socket endpoints/inode/FD and blocked syscall before
   signals; verify original deadlines, failure-plus-stop preventing advancement,
   and successful later receipt discarded after a stop. Keep unused peer listeners
   alive until exit so absence of contact is testable.
7. Compare both held directory descriptors across separate acquisition attempts
   and service; competing actual commands must remain busy and contact no peer.
   Inspect the retained owner path as well; finite observations do not sample
   every possible microscopic gap.
8. Verify post-adoption bind/output failure, blocked final output after resource
   release, termination during acquisition, complete strict reopening and no
   rollback. Reuse B2's actual shared adoption/service routine so its retained
   phase-specific process evidence continues to exercise that common path.
9. Exhaustively exercise the closed typed classifier, including nested allocation,
   accounting/resource failures and unlisted errors, without production hooks,
   inducing OOM or changing privileges. Distinguish unit classification evidence
   from actual transport behavior and process termination from physical power loss.

## 8. Preserved scope and policy

Changes are confined to private node command composition, a behavior-preserving
extraction of B2's post-receive path, new tests and documentation. Existing public
A1/F2 APIs, V1 behavior, dependency selection, workflows, wire/storage/hash formats,
references, fixtures and all previous operational command record contracts remain
unchanged; the help catalogue has the additions noted in section 1.
Cooperative Linux ownership and trusted local operator/parent-path assumptions
remain. Process evidence is Linux IPv4; IPv6 parsing is separate coverage.

Broader reconnect/scheduling, persistent daemon operation, transaction pool and
wallet remain later MB-04 work. Full MB-04 and Gate A/B remain unfinished. Monetary,
combined ECG/designated-pinky/human-supervised Ant Kiss, accessibility, privacy and
identity-continuity decisions/proposals retain their existing gates. No real keys,
personal records, biometric archives, enrollment policy or hardware choice is added.
