# MB-04C2 — finite local pool intake and inspection

## 1. Design boundary

This contract was selected before implementation on 2026-11-28 against accepted
checkpoint `b488287ab47dba327d6b6d1efbde86213fb3fbfc`. It adds one finite Linux
operator command, `mirror-node-v2 pool-inspect`, around the unchanged
[MB-04C1 pending pool](PENDING_POOL_V2.md) and
[MB-04A1 controller](NODE_CONTROL_V2.md). Acceptance evidence is recorded later
in the project checkpoint; this design is not itself passing-test evidence.

The operator supplies a bounded regular file of already signed V2 operations,
independent configuration/genesis pins, two input limits, all five existing pool
limits and an explicit evaluation count. One invocation captures that file,
opens one clean accepted chain, creates one ephemeral pool, admits the complete
input in order and optionally evaluates one exact deduplicated FIFO prefix.

Contextual admission, execution of a requested batch and canonical inclusion
remain distinct. No accepted-history inclusion scan or reconciliation is implied.
The command never signs, mines, initializes, recovers, prepares publication,
appends, adopts, persists a pool, connects to peers or starts a service loop.
The existing B4 `run` command does not create or consume this pool.

No consensus encoding, signed-operation encoding, storage format, public A1/C1
API, crate/version, lockfile, CI command or CI timeout changes. The existing monetary,
personhood, privacy and accessibility policies remain unchanged.

Design refinement before coordinator implementation: `mirror-node` promotes its
existing local `mirror-personhood` dependency from development-only to normal
dependency access solely to classify typed nested membership failures. That
crate is already in the production transitive graph and locked dependency set.
This avoids treating allocation/internal-state failures as ordinary blocked
operations without adding a public reexport or formatting nested error text.

## 2. Complete command surface

All thirteen option/value pairs are required; order is immaterial. There are no
defaults, environment/configuration fallback, stdin input or repeated options.

| Option | Meaning and admitted range |
| --- | --- |
| `--control-dir PATH` | Existing explicit A1 authority root. |
| `--store-dir PATH` | Existing explicit replacement-store root. |
| `--config-hash HASH` | Independently selected existing network configuration hash. |
| `--genesis-hash HASH` | Independently selected existing genesis hash. |
| `--input-file FILE` | One local regular file; `-` is refused, not stdin. |
| `--max-input-bytes UINT` | 1..149248 complete file bytes, including LF framing. |
| `--max-input-count UINT` | 1..256 framed records; an empty file contains zero. |
| `--max-entries UINT` | Existing C1 retained-entry limit, 1..256. |
| `--max-encoded-bytes UINT` | Existing C1 retained encoding-byte limit, 1..74496. |
| `--max-batch-operations UINT` | Existing C1 batch-count limit, 1..64 and no greater than `max-entries`. |
| `--max-batch-bytes UINT` | Existing C1 complete framed MRB2 byte limit, 168..19048. |
| `--max-work-units UINT` | Existing cumulative C1 abstract-work limit, 1..67108864. |
| `--evaluate-count UINT` | 0..`max-batch-operations`; zero skips evaluation explicitly. |

The 27 argv tokens fit the retained 32-argument ceiling. The existing 4096-byte
token and 16384-byte total limits, valid UTF-8/control-character rejection,
canonical unsigned decimal syntax and hash rules remain. All options and
cross-field relations are validated before signal setup, file access or A1
opening. New options are exclusive to `pool-inspect`; old command requirements
and output remain intact. The help record remains within 4096 bytes.

Input caps are independent of retention, batch and work caps. Every framed
record, including each exact duplicate, consumes input count and file bytes.
The hard file ceiling is `256 * (2 * 291 + 1) = 149248`. C1's unchanged charge
formulas govern its owner; framing and file checks have separate finite input
bounds and do not pretend to be part of the C1 meter.

## 3. CLI-only file framing

An empty file is valid. Every nonempty record contains 2..582 lowercase ASCII
hexadecimal characters, an even character count, and exactly one terminating LF.
Hex pairs decode to the unchanged existing `SignedOperationV2` bytes, at most
291 bytes per record. Complete framing is checked before opening A1.

Blank lines, CRLF, missing final LF, uppercase hex, BOM, whitespace, comments,
nonhex bytes and odd hex lengths are refused. There is no header, version field,
network selector, parent selector or signer selector. A short even hex record
can be valid framing and still fail the existing operation decoder; the CLI
does not duplicate or weaken that decoder.

The implementation keeps bounded raw bytes and bounded record ranges, then
decodes each record into a fixed 291-byte stack buffer for its single admission
call. It does not allocate an unbounded per-record payload collection. The file
is never written, renamed, removed or used as a pool persistence store.

## 4. File snapshot and filesystem trust

The command rejects lexical `..` and the stdin-like path `-`. Ordinary absolute
or relative paths and `.` components are admitted. It walks existing parent
components with no symlink following, requiring directories, then checks the
leaf with `symlink_metadata`. The leaf must be a regular file, have link count
one and fit the explicit byte cap. Missing and nonregular paths fail closed.

Open is read-only with `O_NOFOLLOW | O_NONBLOCK`; the existing Rust descriptor
close-on-exec behavior is retained. Compare descriptor and pre-open metadata,
including device, inode, regular type, link count, length, mtime and ctime with
their nanoseconds. Read at most the observed length plus one growth/EOF sentinel
byte into a fallibly reserved initialized buffer. An explicit read loop checks
shutdown before reads and interrupted retries. Early EOF, extra bytes or a
changed descriptor/path identity fails. Require exact expected length and EOF,
then recheck descriptor and leaf metadata before accepting the byte snapshot.
Drop the input descriptor before A1 opening.

The parent namespace and local filesystem remain trusted operator inputs.
Checks detect ordinary replacement, truncation, growth and metadata changes;
they do not prove an atomic snapshot against a hostile same-user writer or
namespace actor. A parent walk is not an `openat2` resolution capability.
`O_NOFOLLOW` covers the final component, and `O_NONBLOCK` avoids a raced FIFO
open waiting for a writer; it does not impose a hard deadline on regular-file
or block-device I/O. These limits follow the
[Linux open(2) contract](https://man7.org/linux/man-pages/man2/open.2.html).
Byte and work caps do not establish hard wall-clock, CPU or RSS bounds.

Placing an input file inside a strict A1/store root can cause the unchanged
unknown-layout refusal. The command does not clean such files automatically.

## 5. Ownership and phase order

1. Parse and validate the entire command without effects.
2. Install the existing sticky SIGINT/SIGTERM guard once.
3. Capture and frame-check the bounded input; close its descriptor.
4. Check shutdown and call `NodeControlV2::open_writer` exactly once using the
   independently supplied identity. Check shutdown again after it returns.
5. Create one `PendingPoolV2` against the actual immutable `ready.chain()` and
   keep the same accepted chain and both roots owned throughout all pool work.
6. Admit every framed record in order, checking shutdown between calls.
7. If requested, evaluate exactly one retained FIFO prefix once.
8. Copy only bounded scalar/checkpoint observations into the report.
9. Drop the pool, controller, input snapshot and signal guard before returning.
10. Validate/format the fixed report and write stdout after all owners drop.

A1 opening is clean-only: pending intent requires explicit existing recovery;
missing, corrupt, wrong-network and busy roots retain their current refusals.
Opening takes exclusive ownership, reconstructs the accepted chain, validates
the confirmed checkpoint and performs existing replay/synchronization. It is
not the read-only `status` path. Equal application-file snapshots and selected
checkpoints prove unchanged canonical contents/selection, not absence of sync.

There is no automatic initialization, recovery, acquisition retry or reopened
owner between records. No C1 reconciliation, staged-state binding, publication
preparation or controller mutation is introduced. Evaluated batch copies are
discarded after their count/size/parent observations have been checked.

## 6. Admission and evaluation outcomes

Intake is all-or-nothing at the command boundary. Any file/framing, operation,
retention or cumulative-work refusal aborts with exit 1, a bounded static
diagnostic and no normal/partial-success stdout record. The ephemeral owner is
dropped. Already consumed work is never reset, refunded or retried.

An exact duplicate is accepted under unchanged C1 rules even at full retention,
keeps FIFO order and incurs the existing identification charge. On a normal
report, `input_count = retained_count + duplicate_count`.

| Requested evaluation | Report | Exit |
| --- | --- | --- |
| Count zero | `evaluation=not-requested`, reason/index `none`, evaluated count/bytes zero. | 0 |
| Exact requested prefix succeeds | `evaluation=evaluated`, exact count/framed size; reason/index `none`. | 0 |
| Expected domain/budget refusal | `evaluation=blocked`, closed reason and optional FIFO operation index; evaluated count/bytes zero. | 1 |
| Operational resource/invariant failure | Static diagnostic only; no normal report. | 1 |

A positive count above the deduplicated retained length is blocked with
`invalid-count`; it is not shortened. The closed blocked reasons are
`invalid-count`, `batch-byte-limit`, `work-limit`, `operation`, `body` and
`ledger`. Body length arithmetic failure is operational. A ledger operation
refusal retains its typed index, zero-based in the deduplicated FIFO prefix,
not the original file line. Ledger account-limit refusal can be `ledger` with
no index. Ledger allocation, length/supply/internal-state failures remain
operational; nested diagnostics are never formatted or echoed. The coordinator
uses a closed explicit mapping and unit tests for its resource/error boundary.

The command never retries a smaller prefix, reorders, drops a refused record,
performs a signature prepass or claims a partially staged ledger was committed.
Context-valid overspending, sequence gaps and conflicts may be retained; only
the one exact ledger evaluation establishes executability at the reported
parent. Admission/evaluation does not establish absence from accepted history.

## 7. Fixed normal report

The existing `mirror-node-v2-1` format and 4096-byte output buffer are retained.
There are exactly 31 newline-terminated `key=value` fields, no variable-length
ID listing, paths, input bytes or nested errors:

| Group | Fields |
| --- | --- |
| Existing state identity | `format`, `operation=pool-inspect`, `status=inspected`, `config_hash`, `genesis_hash`, `store_selection=confirmed` |
| Selected checkpoint | `selected_generation`, `selected_height`, `selected_tip_hash`, `selected_ledger_root` |
| Intake observation | `input_bytes`, `input_count`, `retained_count`, `duplicate_count`, `retained_encoded_bytes`, `work_used` |
| Explicit limits | `max_input_bytes`, `max_input_count`, `max_entries`, `max_encoded_bytes`, `max_batch_operations`, `max_batch_bytes`, `max_work_units` |
| Evaluation | `evaluation_requested_count`, `evaluation`, `evaluation_reason`, `evaluation_operation_index`, `evaluated_count`, `evaluated_block_bytes` |
| Scope | `canonical_inclusion=not-checked`, `pool_disposition=discarded` |

Absent reason/index is the literal `none`. Only successful evaluation has
nonzero evaluated count/bytes. The formatter validates limit relations, input
and retained counts/bytes, count sum, work, evaluation consistency and index
range, and the reported parent identity. `discarded` means aggregate counts
describe a past local observation; no pool remains available after exit.

Operational diagnostics retain the existing bounded static format. Local codes
cover input path/unsafe/missing/I/O/change/byte/count/framing failures,
`pool_operation`, `pool_entry_limit`, `pool_encoded_byte_limit`,
`pool_work_limit`, `pool_invariant`, allocation/arithmetic failure and
`pool_shutdown`; A1 errors retain their existing classifications. No payload,
raw path, operation ID or arbitrary lower-layer string enters diagnostics.

## 8. Shutdown and output loss

The existing signal guard is sticky. Check before reads/retries, before/after
A1 opening, before and after successful admissions, and before/after successful
evaluation. A caught stop yields `pool_shutdown`, exit 1, no normal report and
resource release. An admitted synchronous A1 or C1 call finishes or fails
before a subsequent check; there is no hard preemption guarantee. A real
failure returned by such a call keeps its failure/blocked outcome instead of
being relabeled shutdown. A signal arriving after the final observation check
may arrive after the computation has completed.

All descriptors and authority owners are released before formatting/output.
Broken output uses the existing `output_io`, `inspect-status` and conservative
`operation_may_have_completed=true` context. A1 replay/synchronization and local
inspection may already have completed. This does not claim a new canonical
selection or authorize an automatic retry. Stdout itself has no deadline.

## 9. Implementation and verification plan

Private binary modules separate bounded file capture/framing from coordination.
The existing argument parser, output buffer, failure mapping, signal guard,
A1 and C1 code are reused. New private tests and `pool_commands_v2` process tests
are additive. Frozen fixtures and existing helpers are preserved. One retained
B2 test requires the narrowly recorded observation repair below; its production
path, timing constants and result/state assertions remain unchanged.

The initial full local gate failed before its chosen SIGINT: the retained
receive-deadline test observed that its child had already exited, without
capturing that exit's diagnostic. The test constructed/transmitted/replayed a
candidate inside an existing 1000 ms session, then began another 650 ms wait
after observing terminal receive. That observation did not establish 650 ms of
remaining allowance. The repair precomputes the full candidate transcript before
launch, anchors a 650 ms signal offset to observed request receipt, waits only
the remaining offset after terminal-receive observation and freshly observes the
same one-byte EOF read immediately before SIGINT. The original 1450 ms upper
bound is measured from that same earlier epoch: a renewed 1000 ms deadline after
the signal would take at least 1650 ms. All existing outcome, no-adoption and
state assertions remain. This strengthens the test's clock relationship without
changing runtime behavior or loosening its three timing constants. The initial
failed gate is retained separately from final-source acceptance.

Verification covers effect-free argument refusal; exact byte/count limits;
empty and malformed framing; missing/nonregular/symlink/hardlink/FIFO inputs;
four operation kinds, duplicates and independent retention limits; contextual
refusals versus overspend/sequence/order/late-operation batch blockage;
cumulative work; missing/pending/corrupt/wrong-network/busy A1; fresh invocation
counts; unchanged full canonical state; bounded output; broken stdout; and an
observed full-pipe stdout write while input and both authority owners are gone.

Use deterministic private reader tests for short reads, interruption, growth
and early EOF where feasible. Do not label timing-only races or signals as
observed admission/evaluation interruption. Fallible allocation and metadata
race boundaries must be described according to actual evidence. Process tests
keep bounded child deadlines and kill/reap cleanup; no production test hooks.

Run every retained gate: the full workspace, documentation tests, seven release
commands, twelve independent references and sixteen synthetic examples under
Rust 1.98.0 and locked dependencies. All 38 required commands and 43 CI steps
remain. Freeze and verify the exact published implementation source and its
full CI log before the later nine-document checkpoint; verify that checkpoint
head separately. Passing C2 establishes only this finite local component, not
gossip, persistence, wallet custody, autonomous production or deployment.
