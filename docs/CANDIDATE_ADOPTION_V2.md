# V2 completed-candidate adoption and retained recovery inputs

Contract: **MIRROR-MB03F2-001**, synthetic development profile, 2026-11-26.
Implementation: [`mirror_network::v2::adoption`](../crates/mirror-network/src/v2/adoption.rs).

This orchestration connects a completely accepted
[`ReceivedCandidateV2`](PEER_HISTORY_V2.md) to the existing
[whole-history comparator](FORK_CHOICE_V2.md),
[replacement store](STORAGE_REPLACEMENT_STORE_V2.md), and
[explicit replacement recovery](STORAGE_REPLACEMENT_RECOVERY_V2.md).
Its TCP caller is defined by [the bounded session contract](PEER_SESSION_V2.md).
Receiving a candidate and choosing to store it remain separate operations.

## 1. Independent authority and complete accepted state

`AdoptionAttemptV2::prepare` consumes an owned `ReplacementStoreV2`, the caller's
independently chosen `ManifestIdentityV2`, its independently retained full
`ReplacementCheckpointV2`, and one completed `ReceivedCandidateV2`.

The network identity contains the selected configuration hash and exact genesis.
The current checkpoint includes every chain endpoint field and the local storage
generation. These are local inputs, never values learned from the candidate's
Begin/End claims or selected-file bytes. A received candidate already contains a
complete history accepted through actual genesis/block execution; an unfinished
receiver, raw block, untrusted summary or cache cannot be passed instead.

Preparation requires a writer, rejects a fenced handle, independently checks both
accepted histories against the selected network, and requires the actual current
memory endpoint and generation to equal the supplied checkpoint. It also derives
and compares the complete canonical current selection metadata, including the
manifest and encoded-history accounting. It does not read or write a file.

The unchanged fixed-target development rule selects a candidate only when its
accepted height is strictly higher. Equal-height, identical and shorter histories
keep current. Equal balances do not imply equal membership or equal history;
common ancestry is determined from accepted block hashes. No peer generation,
advertised score, population maximum, matching root or connection completion can
authorize a replacement independently of these checks.

`AdoptionReportV2` records the local choice, relation, actual common ancestor,
previous checkpoint, candidate endpoint and selected checkpoint. Before
publication, `selected` is a **planned** local checkpoint. A copied report is not
a write permit, evidence that files were published, global latestness or finality.

## 2. Preparation retains ownership and exact intent

A keep-current decision retains the owned store and discards the losing received
candidate. It creates no replacement generation and no recovery record. Calling
`publish` on that decision makes no filesystem call and returns the unchanged
writer. Consequently this path does not independently audit disk for changes by
the trusted local owner; a later store operation or strict reopening retains its
ordinary full validation requirements.

For a preferred candidate, preparation calls the existing pure replacement
preparation API to check exact current metadata, strict preference and the next
local generation. It then retains an `AdoptionRecoveryInputsV2` containing:

- The independent network identity and full old checkpoint.
- The next local candidate checkpoint, derived from the actual accepted chain.
- An owned exact clone of the complete old accepted history.
- The completed candidate history, moved into the record without cloning.

All four exist before the first publication operation. The old clone is made
only after strict preference is established. Both retained histories have
immutable accessors; no mutable chain or checkpoint setter is exposed. The store
continues holding its same exclusive root-directory lock while the prepared
attempt exists. There is no unlock/reopen window between preparation and
publication.

The retained old chain and candidate preserve their own manifests, blocks,
accounts, membership records, permanent reservations, approvals, sequences,
population history and resource counters. The code never combines accounts from
one fork with membership from another. `request()` borrows both exact histories
and both checkpoints into the existing `ReplacementRecoveryRequestV2`.

These are **in-memory recovery inputs**, not a persisted intent journal or an
automatic archive. A process crash destroys them unless the caller has separately
preserved or can independently reconstruct the exact inputs. No startup scan or
peer's higher generation may substitute for that missing authority. Persistent
operator intent and restart orchestration remain separate node work.

## 3. Publication and failure ownership

`AdoptionAttemptV2::publish` consumes the attempt; it cannot be called twice.
Preferred publication delegates to `ReplacementStoreV2::replace` using the
retained independent identity, old checkpoint and complete candidate. The store
rechecks the actual selected disk history and exact candidate under its held lock,
stages and synchronizes complete candidate files, publishes and synchronizes the
selection, checks old bytes before cleanup, and fully replays/synchronizes before
returning a confirmed writer. All existing file caps, checked cleanup and fencing
rules remain in force.

Successful `AdoptionOutcomeV2` exposes the confirmed checkpoint, report, immutable
store reference and consuming `into_store`. Advancing several heights still
advances the local generation exactly once. A genuinely diverging membership
history replaces the entire canonical history; orphaned approvals and population
peaks do not remain canonical just because account bytes happen to agree.

Pure preparation rejection returns `AdoptionPreparationFailureV2`; its
`into_inputs` returns the unchanged owned store and completed candidate. No store
mutation or hidden fencing is introduced by preparation. An already fenced input
remains fenced.

Publication rejection returns `AdoptionPublicationFailureV2`. It privately retains
the owning store and both exact histories. It exposes the error, root, fencing
state, prior last-confirmed memory checkpoint and immutable recovery inputs, but
no ordinary writer accessor. The prior memory checkpoint is **not a claim about
current disk selection**: a failed publication may already have selected the
candidate. Neither failure construction nor dropping the failure retries,
cleans, rolls back or deletes any file.

The existing store fences disk failures. A pure storage preflight error remains
subject to its existing behavior; the wrapper does not claim every conceivable
error must have modified disk or set a fence. In either case it exposes no normal
writer from the publication failure.

## 4. Explicit recovery

Calling `AdoptionPublicationFailureV2::recover(action)` is an explicit new caller
action. It creates the existing recovery request from the retained exact inputs
and consumes the owned store into `ReplacementStoreV2::recover`. That operation
retains the **same** root-directory lock throughout validation and recovery.

The unchanged actions are `Complete` and `RetainOld`. Completion can extend only
authorized exact candidate prefixes and publish the fully accepted candidate.
Retention can remove only authorized staging while the old history is still
selected. A selected candidate can never be rolled back by `RetainOld`. Unknown
files, corrupt selectors, noncanonical residues and unsafe artifacts are preserved
and rejected. Ordinary opening, inspection, reconciliation and publication do not
call recovery implicitly.

Successful recovery returns the existing fully replayed, synchronized writer.
Failure returns `AdoptionRecoveryFailureV2`, with **no owning handle**, while
preserving the root and exact identity/checkpoints/histories. Its consuming
`retry(action)` calls the existing `recover_interrupted` API and therefore must
acquire fresh exclusive ownership and revalidate everything. Retry is always
explicit; the wrapper has no retry loop or background work. A competing owner or
unchanged invalid residue rejects again and leaves the inputs available.

The wrapper does not extend recovery to interrupted initialization, damaged
selected markers, arbitrary corruption repair, inferred intent or external
biometric data. It adds no filesystem mutation primitive of its own.

## 5. Fresh initialization stays separate

`initialize_received_v2(root, identity, &received)` independently checks the
completed chain against the caller's selected network and calls only the existing
explicit `ReplacementStoreV2::initialize` operation. Initialization begins at local
generation zero at any accepted height. The completed candidate is borrowed and
remains available on failure.

An existing, legacy, partial or nonempty store is never overwritten or adopted as
a fallback. Opening a selected store requires the separate existing `open_writer`
API and independent current pins. A failed open is not permission to initialize.

## 6. Resource and compatibility boundaries

Each accepted history keeps its existing 4,096-block, 16-MiB complete-block and
1,048,576 cumulative state/configuration-item bounds. F1 transfer limits and F2
session accounting are unchanged by adoption. Storage's distinct encoded-input
and repeated-read caps retain their existing meaning.

Preferred preparation additionally holds an exact old-chain clone while the store
still owns its current chain, plus the received candidate. Manifest encoding and
later store replay can allocate additional bounded structures. The existing
encoded-input ceilings do **not** bound this combined resident memory. Allocation
failure, synchronous validation/replay latency and physical power-loss behavior
are not newly guaranteed by this wrapper.

Adoption starts after successful reception and is outside the transport session's
deadline. Storage operation duration is controlled by its existing bounded work
and I/O contracts, not by a socket timer. No change is made to MPN2, MRB2, MNM2,
MSR2, any prior hash domain, consensus preference, monetary rule or biometric
policy. Linux cooperative directory ownership and trusted local owners/parents
remain assumptions; this adds no Windows, DrvFS or network-filesystem certification.

## 7. Verification scope

[`candidate_adoption_v2.rs`](../crates/mirror-network/tests/candidate_adoption_v2.rs)
uses completed received public synthetic histories. It checks explicit fresh
initialization, independent identities and current pins, keep-current no-write
behavior, complete membership-fork adoption, strict reopen, ownership retention,
failure fencing and explicit completion/retention/retry. It also checks that a
selected candidate cannot roll back and that corrupt or unknown artifacts remain
unchanged. Injected residue is created only inside newly owned test directories;
these tests do not claim a real power-loss event occurred.

The consuming-publication compile-fail example checks that one prepared attempt
cannot be reused for a second publication. Real TCP peer-process demonstrations
and session failure coverage are recorded separately. Exact executed results,
source revisions and CI evidence belong to
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).
