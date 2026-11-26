# Current status

Roadmap: [MASTER_PLAN.md](MASTER_PLAN.md), MIRROR-PLAN-001.
Checkpoint: **MIRROR-MB04A1-001 — durable node control state and restart authority**.
Updated **2026-11-26**.

**MB-04A1 is VERIFIED_COMPONENT — DURABLE CONTROL / EXPLICIT RESTART.**
MB-03 remains complete within its finite storage, recovery and peer-flow scope.
MB-04 remains IN_PROGRESS: operator commands, persistent serving, reconnect and
multi-peer scheduling, pool and wallet are separate work. Complete Ant Kiss remains
MB-05 and final human consensus MB-06. Gate A and Gate B remain UNMET.

## Exact source and publication

Implementation source: **32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0**.
Tested source tree: **d79080968d822a2267ce5f1b1196efcf16d2e546**.
Parent checkpoint: **37010bc5c5827f5a8dd886af0e79d079ebfe1d4e**.
[Source CI 45861912935](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45861912935)
passed every configured gate. Full job `140054826270` logs were inspected and confirm
1,083 runtime tests and eleven compile-fail doctests. Its actual checkout
`62437dc1c338156fb984349b791efc7a4ad34ebb` is verified against the implementation's exact source tree.
All sixteen published implementation/contract/workflow files were read back
byte-for-byte against the locally tested source. Later documentation-head CI is
checked separately and recorded on PR #1; these results do not assert a future
head is tested. PR #1 remains OPEN and UNMERGED on `work/personhood-register-v0.1`.
Main remains `ad7ffd120fb451fe44cf59cb909176785b33bf8f`.

## Delivered behavior

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

## Acceptance evidence

Local acceptance used Rust **1.98.0** and locked dependencies:

- **1,083 workspace/all-target runtime tests passed**, zero failures: 1,010 retained
  plus **73 new** (24 record/vector, 31 public controller, 12 real-process parent
  tests and six internal failure/mutation tests). Parameterized boundary scenarios
  and child invocations are not extra unique tests.
- **Eleven compile-fail doctests passed**, ten retained plus a consuming-controller
  test preventing a second adoption through the same owner.
- All required release suites passed: personhood 234 runtime + 3 doctests, core 121,
  state 49, consensus 23, chain 85 + 3, storage 350 + 2, protocol 38, network 99 + 2,
  and node **73 runtime + 1 doctest**. Release reruns are not new unique tests.
- All **twelve independent Python references** and **sixteen examples** passed.
  Three subprocess helpers are intentionally ignored by ordinary enumeration and
  exercised by their parent tests.
- Scoped formatting, whitespace and allocation/runtime-data audits passed.
  **140 of 142 retained Rust/dependency/reference/fixture files are byte-identical**
  to the parent. The only retained changes are node's Cargo manifest and lockfile,
  adding one local test dependency on personhood. No external versions change.
  V1, storage/network runtime source, monetary code and earlier fixtures are unchanged.

The independent Python reference derives six confirmed records and four complete
intents from established independent history/selection generators, not Rust output.
It checks both new checksum domains, exact layouts and bounds.

Actual child processes are killed at all sixteen adoption and six initialization
boundaries, then fresh processes perform status/open/explicit recovery. Additional
tests interrupt recovery again, including a twice-extended confirmation prefix.
The error-return suite covers the same adoption boundaries, fifteen recovery-error
scenarios and changed control authority immediately before store writes or intent
retirement. Every child has bounded readiness/exit and kill/reap ownership.

The finite `node_control_v2_demo` uses four independent processes:

| Phase | Height / generation | Accounts | Active / pending / peak | MRB2 bytes / state items |
|---|---|---:|---|---|
| Initialize trusted control | 3 / 0 | 4 | 3 / 0 / 3 | 3,206 / 58 |
| Restart and keep membership-fork tie | 3 / 0 | 4 | 3 / 0 / 3 | 3,206 / 58 |
| Restart and adopt preferred membership fork | 4 / 1 | 4 | 1 / 1 / 2 | 3,605 / 73 |
| Fresh process strict reopen | 4 / 1 | 4 | 1 / 1 / 2 | 3,605 / 73 |

Every phase retains two recorded approvals. The two forks have identical account
bytes and different membership histories; full-state equality is checked. This is
finite component acceptance and implementation review, not an external audit.

## Exact continuation

**Next: MB-04A2 — separate finite V2 operator commands. READY.**

1. Read the accepted node-control, F2 session/adoption and profile-3 recovery
   contracts. Specify and implement an explicit V2 command surface alongside the
   preserved V1 executable behavior: init, status, sync-once and explicit recover.
2. Require deliberate control/store paths and independently selected network
   identity. Define strict bounded argument/config parsing and clear diagnostics;
   do not derive trusted identity from a peer or promote the highest generation.
3. Init must separately initialize accepted synthetic storage and its A1 control
   state, or explicitly attach control using independent full pins. Report partial
   initialization; never turn a failed open into init, repair or overwrite.
4. Status must be read-only and distinguish missing, invalid, clean and
   recovery-required outcomes. It must not mine, contact a peer, create directories,
   synchronize files or repair. Keep pending intent distinct from current-disk truth.
5. Sync-once must use the existing bounded F2 TCP receiver and consuming A1 adoption.
   Recovery takes explicit Complete or RetainOld and preserves every A1 no-rollback,
   durability and unknown-artifact boundary. No automatic retry/recovery loop.
6. Test actual command processes, argument rejection and exit outcomes, read-only
   status, independent identities, successful finite synchronization and restart,
   unresolved intent and explicit recovery. Preserve synthetic fixtures and avoid
   real keys, biometrics or live monetary issuance.

Keep A2 focused on finite operator usability. Persistent serving, reconnect/rejoin
scheduling, global admission, multi-peer policy, transaction pool, wallet and
signer remain later MB-04 work. No monetary, device or human-consensus decision
blocks this increment.

Run every AGENTS.md gate with Rust 1.98.0 and locked dependencies, including the
node release suite, twelve references and sixteen examples. Keep source acceptance
separate from later documentation-head CI and refresh all checkpoint records.

Read [AGENTS.md](AGENTS.md), [CURRENT_STATUS.md](CURRENT_STATUS.md),
[MASTER_PLAN.md](MASTER_PLAN.md) tracker and section 27, [NEXT_SESSION.md](NEXT_SESSION.md),
[decisions](docs/DECISIONS.md), [node control](docs/NODE_CONTROL_V2.md),
[peer sessions](docs/PEER_SESSION_V2.md), [adoption](docs/CANDIDATE_ADOPTION_V2.md),
[history reconstruction](docs/PEER_HISTORY_V2.md),
[replacement storage](docs/STORAGE_REPLACEMENT_STORE_V2.md),
[explicit storage recovery](docs/STORAGE_REPLACEMENT_RECOVERY_V2.md),
[threat model](docs/THREAT_MODEL.md), [acceptance matrix](docs/MEMBERSHIP_TEST_MATRIX.md)
and [the-prompt](the-prompt). Recheck live refs and preserve newer/unpushed work.
