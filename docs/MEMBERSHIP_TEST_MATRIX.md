# Membership integration acceptance matrix

Design checkpoint MIRROR-MB01A-001; reviewed baseline db2861930cc0f01f735ccfba80a4f530f4d1ddd6.
Current implementation checkpoint: **MB-04A2 — finite V2 operator commands**,
with source/test mapping and acceptance fields in section 20. A1 remains a
**VERIFIED_COMPONENT within finite operator-owned control/restart scope**;
section 19 preserves its accepted evidence.
MB-03F2, MB-03F and MB-03 remain verified within finite storage and peer sessions;
persistent serving and the remaining node integration remain MB-04.
These 35 rows are acceptance requirements, **not 35 passing tests**. B1–B4 and
MB-02A/B/C are verified components; MB-02 supplies actual bounded in-memory branch
acceptance. MB-03A adds explicit immutable-store initialization, cooperative
ownership and complete selected-history reopening. MB-03B adds a separate append
profile, one-child publication, failure fencing and clean-store reconciliation.
MB-03C adds explicit recovery of recognized residual staging for one independently
authorized child. MB-03D compares accepted branches with independently selected
network identity and an immutable complete-state result. MB-03E1 pins bounded
replacement metadata and prepares a preferred whole branch without filesystem I/O.
MB-03E2a adds a separate replacement store, synchronized complete-branch publication,
exact old-copy cleanup and failure fencing under stable ownership. MB-03E2b adds
explicit completion/retention of recognized interrupted replacement staging and
cleanup, under independent pins and both accepted histories. MB-03F1 adds separate
V2 framing, header admission and a consuming complete-history receiver. MB-03F2
uses that receiver over bounded TCP sessions, compares complete accepted branches,
and publishes only a preferred candidate through owned storage with both histories
retained for explicit recovery. MB-04A1 persists the trusted local checkpoint and
both exact histories in separate bounded control records before store mutation,
then requires explicit recovery after restart with unfinished intent. MB-04A2
exposes that unchanged authority through a separate finite executable: explicit
attachment, read-only status, one peer receive/adoption, or separately requested
completion/retention. Arbitrary corruption repair, process-wide provider admission
and persistent V2 service remain unfinished.

Historical counts and exact-source CI remain in sections 3–9: B1 added 31 cases,
B2 39, B3 51, B4 69, and MB-02C added 61 to reach its **523-test baseline**. Those
historical green runs do not verify newer storage source. MB-03A added **65 unique
cases: 24 selection codecs/vectors, 34 filesystem, five lock tests and two
initialization-fault tests**. Local Rust 1.98.0 execution passed **588 workspace
tests**, **one ignored subprocess helper**, and the same **three documentation
tests**. The helper runs only through its parent process tests and is not an
extra unique passing test. Release executions and example assertions are reruns,
not additional unique tests. Section 10 preserves that boundary and test mapping.

MB-03B adds **70 unique cases: 28 selector/vector, 31 filesystem integration,
eight private fault and three process ownership cases**. Local Rust 1.98.0 passed
**658 workspace tests**, one ignored subprocess helper, and the same three
documentation tests. Storage release reran 145 cases; all eight independent
references and eight examples passed. Section 11 maps the new cases. Exact-source
push CI 44649111203 and PR CI 44649119266 both passed, with the exact push checkout
and full job logs inspected. Section 11 and [CURRENT_STATUS.md](../CURRENT_STATUS.md)
record that source evidence separately from later documentation-head checks.
Neither local results nor historical green runs establish a newer head's CI.

MB-03C adds **49 test functions: 39 integration, nine private failure and one
additional real-process case**. Local Rust 1.98.0 gates passed **707 workspace
tests**, the same **three documentation tests**, **194 storage release tests**,
all **eight independent references** and **nine examples**, with one ignored
subprocess helper. Source `e8302052ab724f59ff4a212dc64aea9575a6d4b3` is published;
exact-source push CI **44748718086** (job **136330677456**) and PR CI **44748724632**
both passed. The full push log and exact checkout were inspected. Section 12
records this implementation evidence separately from historical MB-03B results
and later documentation-head checks.

MB-03D adds **28 runtime test functions and three compile-fail examples**. Full
local gates passed **735 workspace runtime tests**, **six documentation tests**,
the six required release suites, **eight reference checks** and **ten examples**.
Section 13 records these results separately from exact-source and documentation-head
CI evidence; one subprocess-only helper remains intentionally ignored.

MB-03E1 added **31 codec tests and 21 preparation tests**, with both focused suites
passed and all **nine independent references** checked. Two compile-fail borrow
examples were added. Its historical full-suite and publication/CI evidence remains
in master-plan section 22, separately from the historical MB-03D totals above.

MB-03E2a adds **49 test functions: 32 filesystem integration, 14 private fault/read-
budget and three process ownership cases**. Full local gates and exact-source CI
passed **836 runtime tests, eight doctests, all required release gates, nine
references and twelve examples**. Exact publication/CI evidence is maintained in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and master-plan section
23; an older green checkpoint does not verify the newer source. The retained
subprocess helper and repeated scenarios/release executions are not extra unique
runtime tests.

MB-03E2b adds **55 new test functions: 38 public integration, 15 private
failure/read-budget and two process ownership cases**, mapped in section 16.
Full local gates passed **891 runtime tests, eight retained doctests, all required
release suites, nine independent reference checks and thirteen examples**. No new
doctest count is added. Exact-source publication and CI evidence are recorded in
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and master-plan section 24; local results
alone do not establish a published branch's CI conclusion.

MB-03F1 focused execution passed **24 codec tests and 36 receiver tests**, and its
synthetic `history_receiver_v2_demo` passed. Section 17 maps those **60 runtime
test functions** and the separately gated consuming-API documentation example.
Full-suite, release, reference and exact-source CI results remain separately
recorded in [CURRENT_STATUS.md](../CURRENT_STATUS.md); the earlier 891-test total
belongs to E2b and does not establish this increment's complete acceptance.

MB-03F2 adds **59 runtime cases: 24 session integration, one private blocked-write,
23 adoption, nine real-process and two independent-vector tests**, mapped in
section 18. Full local gates passed **1,010 runtime tests, ten compile-fail
doctests, all required release suites, eleven reference checks and fifteen
examples**. Two intentionally ignored subprocess helpers run through parent
tests. Exact-source CI and later documentation heads are separate evidence in
[CURRENT_STATUS.md](../CURRENT_STATUS.md).

MB-04A1 adds **73 runtime test functions: 24 codec/reference, 31 public control
integration, twelve real-process restart and six private failure cases**, mapped
in section 19. Full local gates passed **1,083 runtime tests, eleven compile-fail
doctests, all required release suites, twelve references and sixteen examples**.
Three subprocess helpers are intentionally ignored and invoked by their parents.
At the A1 checkpoint, the node release gate reran 73 runtime cases and one
documentation case; A2 extends that gate below.

MB-04A2 adds **54 runtime test functions**: 23 parser, four output,
three diagnostic unit cases and **24 actual-command process
cases**. Section 20 maps the finite command surface and records source, required
gates and exact-source CI separately. Full required gates passed **1,137 runtime
tests and eleven compile-fail doctests**, with no failures.

**Next is MB-04B1: bounded V2 provider lifecycle and shutdown**, beginning with
design before implementation. It must retain explicit paths/network/listen
selection, clean A1 before bind, fixed accepted history under ownership,
process-wide attempt/active/aggregate-byte/lifecycle budgets and bounded shutdown.
Repeated F2 owners must not replenish those budgets. Persistent reconnect,
multi-peer policy, transaction pool, wallet, human finality, biological
deduplication and live economy remain outside these component checks. **MB-04
remains IN_PROGRESS; Gate A and Gate B are unmet.**

Normative design: [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md).
Historical source findings: [THREAT_MODEL.md](THREAT_MODEL.md).
Exact byte contracts: [CONFIG_V2_ENCODING.md](CONFIG_V2_ENCODING.md) and
[OPERATIONS_V2_ENCODING.md](OPERATIONS_V2_ENCODING.md); frozen B4 state layout and
B3 execution rules: [MEMBERSHIP_STATE_V2.md](MEMBERSHIP_STATE_V2.md).
Grouped replay: [REPLAY_V2.md](REPLAY_V2.md). Transfer, unified operation and MRB2
byte contracts: [CORE_V2_ENCODING.md](CORE_V2_ENCODING.md). Atomic mixed execution,
root composition and the explicit MPS2 index extension:
[LEDGER_STATE_V2.md](LEDGER_STATE_V2.md). Pinned manifest, actual block acceptance
and complete replay: [CHAIN_V2.md](CHAIN_V2.md). Immutable selected storage,
filesystem assumptions and publication boundaries: [STORAGE_V2.md](STORAGE_V2.md).
Accepted-child publication, generation selection, fencing and clean-store
reconciliation: [STORAGE_APPEND_V2.md](STORAGE_APPEND_V2.md). Explicit one-child
residual recovery: [STORAGE_RECOVERY_V2.md](STORAGE_RECOVERY_V2.md). Accepted-branch
comparison: [FORK_CHOICE_V2.md](FORK_CHOICE_V2.md). Replacement-selection metadata
and preparation: [STORAGE_REPLACEMENT_V2.md](STORAGE_REPLACEMENT_V2.md). Durable
replacement and clean-state reconciliation:
[STORAGE_REPLACEMENT_STORE_V2.md](STORAGE_REPLACEMENT_STORE_V2.md). Explicit whole-branch
residual recovery: [STORAGE_REPLACEMENT_RECOVERY_V2.md](STORAGE_REPLACEMENT_RECOVERY_V2.md).
Bounded peer-history framing and candidate reconstruction:
[PEER_HISTORY_V2.md](PEER_HISTORY_V2.md). Finite TCP sessions and completed-candidate
adoption: [PEER_SESSION_V2.md](PEER_SESSION_V2.md) and
[CANDIDATE_ADOPTION_V2.md](CANDIDATE_ADOPTION_V2.md). Durable local control and
restart authority: [NODE_CONTROL_V2.md](NODE_CONTROL_V2.md). Finite command input,
output and operator boundaries: [NODE_OPERATOR_V2.md](NODE_OPERATOR_V2.md).

## 1. Rule-to-assertion map

| Case | Required assertion / proposed acceptance name | Owner and current coverage |
|---|---|---|
| T01/R01 | `legacy_data_is_not_reinterpreted`: old MRB1/MPH1 only on old paths; V2 rejects; init refuses legacy directories; old bytes unchanged. | Prior format separation retained; E2a rejects implicit storage-profile migration; F1/F2 reject legacy/version fallback; A2 adds a separate V2 executable and attachment-only init while default-run preserves the existing V1 command; persistent provider lifecycle remains MB-04B1 design work |
| T02/R02 | `seed_configuration_changes_signing_context`: vary seed key/order/nonce/evidence, allocation and every setting separately; old signatures fail on otherwise-valid changed configs. | Prior context/bootstrap/manifest checks retained; storage/comparison/F1 require independent network authority; A2 requires explicit external C/genesis and initial full checkpoint flags, with actual wrong-network command rejection before peer contact; no peer terminal claim becomes trusted selection |
| T03/R03 | `bootstrap_has_no_circular_context_or_bypass`: compute C before proofs/roots, all ordered proofs mandatory; reject zero/duplicate descriptors and invalid counts/thresholds; no first-seed runtime bypass or forced revocation. | B1 construction, B3 ordinary seed runtime rights and MB-02C all-proof manifest construction/decoding VERIFIED; production bootstrap/admission policy remains gated |
| T04/R04 | `v2_operations_have_exact_lengths`: prefix131; signed Transfer243/Enroll291/Approve227/Withdraw227; reject all truncations, versions/kinds, oversized/trailing bytes before mutation. | B2 membership plus MB-02A Transfer/unified codecs VERIFIED COMPONENTS; all four lengths and preserved membership bytes covered |
| T05/R05 | `every_signed_field_is_bound`: altered kind/context/anchor/window/key/sequence/evidence/profile/target invalidates signature; malformed keys/signatures do not panic. | B2 membership plus MB-02A Transfer authentication, domain separation and strict signatures VERIFIED COMPONENTS |
| T06/R06 | `approval_targets_complete_request`: changed request/window changes enrollment ID; approval cannot authorize it; exact request/key/evidence reuse rejected after active/revoked/expired outcomes. | B2 complete-request ID/signatures and B3 retained ID/key/evidence reuse rejection VERIFIED; fresh biometric duplicates remain external |
| T07/R07 | `membership_sequences_are_exact_and_atomic`: seeds start0, Enroll establishes next1; accept/increment exact sequence once; reuse/skip/overflow fail unchanged; transfer/membership spaces separate; repeated approval no extra weight. | B3 membership, MB-02B account/member sequence separation and overflow checks, and MB-02C actual-block execution/replay of those rules VERIFIED IN MEMORY |
| T08/R08 | `v2_transfer_conserves_test_balances`: positive amounts, checked balance/self-transfer/nonce; reject nonzero fees/mints/rewards; preserve initial synthetic supply. | MB-02B arithmetic/self-transfer/retained-zero rules and MB-02C actual-block conservation of explicit synthetic supply VERIFIED IN MEMORY; no mint/reward API or final economy |
| T09/R09 | `anchor_must_be_recent_canonical_ancestor`: reject unknown/orphan/future/current anchors; exact max age pass/max+1 fail; validate against candidate branch ancestry. | Accepted ChainV2 ancestry and complete replay retain wrong-fork anchor rejection; F1 reconstructs through actual from_genesis/append and rejects a correctly signed operation anchored in another branch; F2 uses that unchanged execution path for real TCP candidates |
| T10/R10 | `height_window_boundaries`: from/until inclusive; from-1/until+1, inverted/overflow span reject; max/max+1; local clock irrelevant. | B2/MB-02A height-window boundaries and B3/MB-02B execution checks retained through MB-02C acceptance; T11 covers record expiry; strict parent header timestamps are separate from operation lifetimes and use no local clock |
| T11/R11 | `pending_expiry_preserves_tombstones`: empty block expires pending at until+1, frees pending capacity not history; active membership unaffected by old request expiry; approval cannot extend lifetime. | Prior expiry/tombstone rules retained; MB-03B appends/reopens an actual empty expiry block, and MB-03C explicitly recovers that child with complete state equality; unknown residual shapes still reject |
| T12/R12 | `only_self_can_withdraw`: pending/active own signed withdrawal succeeds; unknown/expired/revoked or other signer including founder fails; context/anchor/window failure consumes no sequence. | B3 self-only ownership/status and authorization rules retained by actual MB-02C mixed execution and membership-fork replay; no founder override, recovery or death policy is added |
| T13/R13 | `ordered_block_has_one_transition_result`: reject Approve before unknown Enroll; stage valid ordered pair; order committed, never decoder/executor sorted. | B3/B4 and MB-02A/B ordering retained; MB-02C accepts only the exact full-body commitment and executes original mixed order with first-failure precedence VERIFIED IN MEMORY |
| T14/R14 | `newly_activated_member_cannot_approve_in_same_block`: A->B then B->C in same block rejects; B may approve only after active in committed parent. | B3/MB-02B parent-eligibility rule retained; MB-02C supplies actual accepted parent ancestry to the shared executor; no deployed human-consensus claim |
| T15/R15 | `same_block_withdrawal_excludes_earlier_approval`: withdrawn A's earlier approval cannot cause end activation; reverse order approval rejects; already-active targets not cascade-revoked. | B3/MB-02B reviewer-survival/no-cascade rules retained; MB-02C actual membership-fork vectors show withdrawn earlier reviewers cannot activate a pending target |
| T16/R16 | `activation_order_is_canonical`: multiple requests activate by original canonical inclusion position identically on replicas; ordinal once, no reuse; overflow rejects whole batch. | B3/B4 and MB-02A/B ordering/index rules remain; MB-02C preserves original mixed positions; MB-03E2a/b publishes and explicitly recovers the complete selected branch with its own activation history; human finality remains MB-06 |
| T17/R17 | `record_counters_and_root_agree`: recompute active/pending/activations/indexes; change status/sequence/approval/counter/ordinal changes root or rejects invariant. | Executed membership/combined roots retained; complete selected replay reconstructs ChainV2 accounts, membership and resource counters; MB-03E2a requires full candidate equality before/after publication and final clean reconstruction, without authoritative caches |
| T18/R18 | `high_water_uses_committed_branch_end_state`: decline preserves record; transient cancelled activation cannot fabricate peak; reorg discards orphan counts/peaks. | Prior final-state high-water and genuine membership forks retained; MB-03E2a/b replacement and explicit recovery retain the selected branch's own membership, approvals and population peaks even when account bytes agree; no losing-branch counters are grafted into the winner |
| T19/R19 | `membership_batch_failure_is_atomic`: invalid final event after otherwise valid mutations/expiry leaves all records/indexes/sequences/roots unchanged for every error class. | Prior staged/accepted rollback and MB-03B invalid-child rejection retained; MB-03C validates both parent and candidate before mutation and returns complete selected state or no usable handle after failure |
| T20/R20 | `limits_apply_to_direct_execution`: exact vs +1 bytes/events/enrollments/pending/records/accounts over repeated batches; no apply bypass; bounds before large clones/allocations. | Prior execution/replay count, byte and state-work bounds retained; MB-03C counts the full intended child before payload reads even when only a prefix exists, and never prunes accepted history to restore capacity; MB-03D compares at most 4,096 accepted pairs without new accumulation/allocation |
| T21/R21 | `codec_and_storage_limits_agree`: malformed huge declared lengths/counts/files/frames reject before claimed allocation; encoding also bounded. | Prior codec/storage/F1/F2 bounds retained; A2 parses all flags and bounded arguments before effects, forwards one deliberately bounded F2 attempt and retains actual partial-I/O accounting; parser, history, session, control and store limits are distinct, without a total RSS or process-global availability claim |
| T22/R22 | `capacity_does_not_erase_replay_protection`: expiry frees pending not total tombstones; total cap still allows withdrawal but no Enroll; restart preserves used keys/evidence. | Permanent history and identity reservations survive full selected replay; MB-03E2a removes only the exact unselected old branch copy after new selection, retaining the candidate's complete history and original limits; no pruning or counter/cache bypass |
| T23/R23 | `mrb2_header_and_hash_vectors`: header160/empty168, independent vectors/round-trip/exact genesis; changed header fields/context/height/parent/target/roots reject. | MB-02A bytes/hashes and MB-02B roots now joined by MB-02C exact genesis, configuration/height/parent/time/target/work/body/ledger-root acceptance VERIFIED IN MEMORY |
| T24/R24 | `operation_count_and_order_are_committed`: [A,B,C] vs [A,B,C,C] distinct despite odd-leaf duplication; permutations/signatures change root; fixed empty vector. | MB-02A count/order/full-signed-byte commitment retained; actual acceptance/replay recomputes it; MB-03E2a stages exact complete MRB2 files and replays the entire candidate before and after publication; no operation filtering or reordering |
| T25/R25 | `ledger_root_binds_all_components`: isolated changes in accounts/membership/height/C/test-supply change root; correct accounts cannot hide bad membership. | Canonical combined roots and accepted-header checks retained; F2 and A2 actual-process TCP cases receive equal-account/different-membership histories, publish only the locally preferred whole branch and strictly reopen its complete state, approvals, permanent records, population and resource counters |
| T26/R26 | `canonical_replay_rebuilds_derived_indexes`: independent replay matches roots/IDs/indexes/sequences; old/tampered snapshots not trusted authoritative latest state. | Full MRB2 execution reconstructs candidate state under independent network pins; F2 and A2 require matching End plus EOF before whole-branch comparison/publication, and strict A1 reopen checks retained local authority; pending command status reports endpoints as uninspected, not a current-store or global-freshness claim |
| T27/R27 | `mixed_block_is_failure_atomic`: valid money then bad membership and reverse leave BOTH unchanged; wrong final root after valid operations changes nothing. | Prior mixed execution and false-root rollback retained; MB-03C accepts the exact child before either action, preserves rejected unrelated bytes and does not expose a partial ledger through an error |
| T28/R28 | `restart_pins_config_genesis_and_tip`: different valid genesis nonce/wrong bootstrap/old valid log rejected against trusted manifest/tip; replay requires no matcher/device. | VERIFIED_COMPONENT — DURABLE LOCAL RESTART AUTHORITY: A1 retains the full canonical checkpoint/generation and both exact accepted histories in separate trusted control records; external C/genesis and full selected replay remain required; A2 exposes attachment/status/explicit recovery with those unchanged pins, never greatest-generation selection or peer-derived authority |
| T29/R29 | `crash_boundary_never_mixes_states`: inject every process/I/O publication boundary; reopen complete identifiable old/new state or fail; ambiguous I/O never blindly resumes. | PARTIAL beyond the finite verified scope: E2b and A1 retain bounded recovery, actual publication-boundary SIGKILL and repeated recovery interruption with no candidate rollback; A2 checks canonical pending fixtures through real commands, kills a blocked receive before publication and observes broken stdout after completed init; these are separate evidence and do not establish physical power-loss or arbitrary-corruption repair |
| T30/R30 | `second_writer_and_oversize_file_rejected`: lock concurrent writers; reject corrupt/unexpected metadata/oversized input; status read-only does not initialize/mine. | A1 verifies separate safe roots, control-before-store cooperative Linux ownership and bounded regular records; A2 commands preserve read-only status and rejected layouts, hold both roots through TCP receipt/adoption and fail competing commands before contact; actual process exit releases ownership; hostile same-user races, global admission and untested filesystems remain outside scope |
| T31/R31 | `network_versions_do_not_fallback`: MPN2/version2/C/genesis checked before exchange; reject legacy/wrong context, no downgrade; reject unknown kind/length before body allocation. | VERIFIED_COMPONENT — FINITE SESSION PATH: separate MPQ2/version2 request and unchanged MPN2 admission under independent network identity, no fallback or resynchronization; A2 selects a numeric peer explicitly and requires clean local A1 ownership before TCP; persistent provider lifecycle/global admission remain later MB-04 work |
| T32/R32 | `peer_claims_and_partial_messages_are_bounded`: absurd advertised height/count, wrong response, slow/truncated frames and endless branches hit total byte/time bounds; claims never replace execution. | VERIFIED_COMPONENT — FINITE SESSION PATH: F1 history caps and F2 actual-I/O/deadline rules are retained; A2 fixes one attempt and one peer with explicit allowance at most 157129488 bytes and timeout at most 60s, matching End plus EOF; reception failure preserves selection; local opening/adoption/output, CPU preemption and global availability are outside that attempt deadline |
| T33/R33 | `public_records_exclude_biometric_archives`: canary payloads/keys absent from blocks/logs/artifacts; no archive/URL schema; do not claim fresh-key/fresh-evidence biological deduplication. | B4 and MB-02A use fixed signed fields, no archive/URL schema; real custody/canary/privacy acceptance remains MB-08/09, fixed fields not a privacy proof |
| T34/R34 | `integration_cannot_mint_privileged_funds`: unknown mint/claim rejects, no founder/marketing bypass, old Alice fixture intact, activation cannot change money/reserve. | B3/MB-02A closed kinds and MB-02B explicit allocations/no privileges retained through MB-02C actual-block supply conservation; no mint/reward mechanism or founder/marketing bonus; live economy remains MB-07 |
| T35/R35 | `future_policy_is_not_silently_enabled`: minting/rewards blocked on D-01/D-04; synthetic claims cannot satisfy real-admission gates; old reorg cannot bypass finality. | MB-02C actual acceptance retains the disabled mint/reward/fee profile; D-01/D-04 remain unresolved; real admission, human consensus and finality protection remain later gates including MB-06/07 |


## 2. Boundary and adversarial requirements

Use synthetic keys/commitments only. Rejection tests for stateful execution must
assert equality of COMPLETE pre-state, not just an error. Mutate before/after
serialization; fuzz bounded structural decoding separately from authorization.
Test exact and +1 bounds, zero/u64::MAX and checked arithmetic without constructing
huge attacker-declared buffers merely to test a header. Decoders, local proposals,
replay and candidate validation must share rules. Independent nodes later need
identical roots from genuinely different fork memberships, not timestamp-only forks.
Preserve V1 vectors: do not change expected constants to conceal semantic changes.

Independently calculate new fixed vectors using a separate method, record inputs
and preimages, and compare debug/release. Never bless a failing Rust result as the
reference. Separate a trusted synthetic parent view from an implemented live resolver.

## 3. Historical baseline and B1 coverage

The 133-test baseline included 26 V1 personhood tests. These still pass unchanged
but do not establish V2 timing/ledger behavior. B1 source
`809661b5e6a9338d65c15c45e540289df24fcc99` passed CI 44201393943, job 134456099484.
Tests: `crates/mirror-personhood/tests/config_v2.rs`, independent
`tools/reference_config_v2.py`, fixture `tests/vectors/config_v2.txt`.

T02: `every_scalar_configuration_setting_changes_context`,
`seed_key_nonce_evidence_and_order_are_bound`,
`allocation_address_balance_and_zero_entry_are_bound`,
`each_resource_limit_changes_context_and_invalidates_old_proofs`.
T03 construction: `no_circular_context_or_caller_supplied_context`,
`every_seed_proof_is_required_and_seed_indices_are_not_privileges`,
`forged_last_proof_returns_no_partially_verified_bootstrap`,
`malformed_or_weak_seed_keys_cannot_become_verified_bootstrap`, and duplicate/zero/count tests.
T20 config: limit ceilings/relationships, direct count rejection, maximum counts,
zero/overflow/sorted allocation checks. Structural codec tests cover all truncations,
trailing input, huge declared counts and old signature separation.

B1 totals: 31 new tests, workspace164; personhood release57 reruns. These are not
31 completed acceptance rows. B1 leaves runtime privilege/execution tests to B3.

## 4. Historical B2 exact implementation evidence

Source: `af1dd66c4fb8811d8eab54a1c88a8bddae2ae80e`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44205064789 — SUCCESS.
Job 134468771581 full logs inspected; Rust1.98.0/locked dependencies.
All names below are executable tests in
`crates/mirror-personhood/tests/operations_v2.rs` at that exact commit.

| Case/subcase | Actual tests | Verified scope / remainder |
|---|---|---|
| T04 membership codecs | `membership_operations_have_exact_lengths_and_offsets`, `canonical_round_trip_all_membership_kinds`, `every_truncation_is_rejected`, `trailing_and_wrong_kind_lengths_are_rejected`, `unknown_versions_transfer_and_unknown_kinds_are_rejected`, `oversized_input_rejects_before_field_parsing`, `configured_byte_boundary_accepts_exact_enrollment_size` | Membership bytes verified; Transfer and block containers pending |
| T05 binding/signatures | `every_unsigned_payload_byte_is_covered_by_signature`, `every_signature_byte_is_checked`, `same_length_approval_cannot_be_changed_into_withdrawal`, `signing_rejects_wrong_selected_key`, `weak_or_invalid_keys_and_zero_signatures_do_not_panic`, `seed_and_v1_signatures_do_not_authorize_v2_operations` | All three membership kinds, not future Transfer |
| T02 runtime-context necessary check | `valid_signature_on_wrong_configuration_is_rejected`, `changed_bootstrap_configuration_cannot_reuse_operation`, `mismatched_parent_context_is_rejected_before_lookup` | Reject wrong signed/context view; trusted network manifest still MB-02/03 |
| T06 target binding | `complete_unsigned_request_determines_enrollment_id`, `changed_signature_changes_operation_id_but_not_enrollment_id`, `approval_signature_binds_the_exact_request_id` | Full-request ID/signature binding; used-key/evidence/history state remains B3 |
| T09 synthetic ancestry | `current_and_future_height_anchors_are_rejected`, `anchor_age_maximum_is_inclusive`, `unknown_canonical_ancestor_is_rejected`, `wrong_branch_hash_is_rejected_at_same_height`, `candidate_branch_view_not_old_canonical_branch_controls_anchor` | Trusted synthetic parent interface; actual live chain/reorg resolver pending |
| T10 authorization windows | `both_authorization_endpoints_are_inclusive`, `future_window_is_not_yet_valid`, `anchor_must_precede_window_not_merely_candidate`, `inverted_and_overflowing_windows_reject_without_wrap`, `maximum_window_span_is_inclusive_and_checked`, `lowered_window_and_anchor_limits_are_enforced`, `maximum_height_and_genesis_anchor_boundaries_work`, `candidate_height_is_parent_plus_one_and_overflow_rejects` | Pure height/window checks verified, no clock input; record expiry remains T11/B3 |
| Scope guards | `enrollment_must_sign_sequence_zero`, `contextual_checks_do_not_claim_stateful_permission_or_consume_sequences`, `nonce_evidence_and_required_profile_are_checked`, `parent_lookup_is_bounded_and_validation_has_no_clock_input`, `immutable_signed_value_is_not_changed_by_construction_inputs` | Necessary stateless checks only; no authorization to skip B3 |
| Independent vectors | `independent_python_preimages_signatures_and_ids_match` and `tools/reference_operations_v2.py --check` | Preimages/hashes/IDs plus independently generated Ed25519 signature constants match |

**39 new B2 tests**; **203 workspace tests** =164+39. **96 personhood release
reruns** =26 V1+31 B1+39 B2, not additional tests. Both reference checks, both synthetic
examples, doc command (zero cases), allocation audit and unchanged lockfile passed.
No local Rust execution is claimed; the Python signature reference was generated
locally independently of Rust and checked against Rust in GitHub Actions.

At the B2 checkpoint, B3 still needed to reject wrong/exhausted sequences, unknown/
nonpending targets, unauthorized withdrawals, duplicate approvals and same-block new
reviewers. B2 success alone remains insufficient transition authorization. Section 5
records the later B3 stateful evidence and section 6 records B4. At that historical
B2 checkpoint these tasks, MB-02/03 and Gate A/B were unfinished. Current MB-02 is
a verified in-memory component; MB-03 and the device-readiness gates remain open.

## 5. Historical B3 exact implementation evidence and coverage

Published source: `b6f1233d038901aa0a13c8ddac9afbc34418e846`.
Tree `e98daf435335e31440e6c0bf14f28f5928f556c7` matches the locally tested source.
Exact-head push CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317174000
— SUCCESS; push job 134846238261 full logs and exact checkout inspected. PR CI
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44317184524 also passed;
job 134846284741 logs inspected too. Rust 1.98.0, locked dependencies.

Tests below are in `crates/mirror-personhood/tests/state_v2.rs` unless marked private
(`src/v2/state_tests.rs`). There are **45 public integration tests + six private
arithmetic/invariant tests = 51 new B3 tests**. They test the standalone V2 executor
with trusted synthetic ancestry, not actual accepted blockchain headers.

| Case/subcase | Actual tests | Verified scope / remainder |
|---|---|---|
| T03 seed runtime rights | `bootstrap_has_ordinary_active_members_with_ordered_ordinals_and_no_money`, `first_seed_has_no_threshold_or_forced_withdrawal_bypass` | Ordinary configured seed rights; all-proof construction is B1; production policy remains gated |
| T06 retained identity references | `request_key_and_evidence_reuse_rejects_after_each_lifecycle_outcome`, `bootstrap_keys_and_evidence_are_also_historical_reservations`, `runtime_nonce_alone_is_not_a_global_identity_fingerprint` | Pending/Active/Revoked/Expired keys/evidence remain reserved; exact nonce alone is not global identity |
| T07 exact sequences | `enrollment_creates_pending_record_and_consumes_sequence_zero`, `distinct_approvals_activate_once_and_consume_each_reviewers_sequence`, `skipped_reused_and_maximum_claimed_sequences_are_rejected`, `ordered_operations_require_and_consume_consecutive_sequences`, `repeated_reviewer_cannot_add_weight_even_with_a_new_sequence`; private `sequence_exhaustion_rejects_without_wrapping_or_mutation` | Stale-sequence test uses a different request so duplicate-review rejection cannot mask it; true u64 exhaustion uses private injection; Transfer follows MB-02 |
| T09 parent association | `parent_context_height_and_exact_tip_must_match_even_for_empty_blocks`, `matching_tip_does_not_allow_a_wrong_earlier_operation_anchor`; private `parent_height_exhaustion_rejects_empty_block_without_wrap`, `block_tag_is_external_to_canonical_membership_state` | Context/height/tip and earlier anchors checked; trusted bind does not validate real PoW/header/root/genesis |
| T10/T11 lifetime | `approval_must_share_request_expiry_and_cannot_extend_it`, `approval_window_cannot_start_before_enrollment_window`, `empty_blocks_expire_pending_only_after_inclusive_final_height`, `active_members_do_not_expire_with_their_original_requests`, `expired_pending_frees_quota_without_freeing_key_or_evidence` | Inclusive request lifetime, approval alignment, empty-block expiry and retained history; serialized replay B4 |
| T12 ownership/status | `pending_owner_can_withdraw_but_cannot_repeat_or_reregister`, `active_withdrawal_preserves_original_activation_and_high_water`, `another_signature_cannot_withdraw_an_existing_membership`, `active_target_rejects_later_approvals_without_consuming_sequence`, `context_valid_unknown_target_is_not_state_authorization`, `unknown_pending_and_self_reviewers_are_ineligible`, `every_action_rechecks_context_ancestry_window_and_signature_before_execution` | Self-only terminal withdrawal and authorization rechecks; no forced revocation/recovery/death policy |
| T13 ordered execution | `enrollment_then_approval_in_same_block_is_valid_but_reverse_order_is_not`, `independently_constructed_states_produce_identical_ordered_transitions` | Immutable replicas reach equal state; block order commitment/root remains MB-02/B4 |
| T14 parent eligibility | `newly_activated_member_cannot_approve_until_next_block` | Same-block admission chaining rejects; next-block ordinary approval succeeds |
| T15 surviving approval weight | `same_block_reviewer_withdrawal_removes_earlier_approval_weight`, `approval_after_reviewer_withdrawal_rejects_the_whole_block`, `earlier_block_approval_loses_weight_when_reviewer_withdraws`, `reviewer_withdrawal_never_cascade_revokes_already_active_members` | Parent-Active AND staged-Active required; no retroactive cascade |
| T16 activation order/ordinals | `simultaneous_activations_follow_original_inclusion_not_approval_order`, `same_height_activation_order_is_operation_index_not_sorted_member_id`, `withdrawn_applicant_does_not_activate_or_reuse_an_ordinal`, `declined_population_never_reuses_an_old_activation_ordinal`; private `activation_ordinal_exhaustion_discards_enrollment_and_approval` | Across heights and within a height; deferred activation preserves ordinals and rejects overflow |
| T17 internal consistency | Private `invariant_audit_rejects_damaged_key_and_evidence_indexes`, `invariant_audit_derives_counts_and_activation_ordinals_from_records` | Structural private-state assertions only; not a snapshot-authority validator, no root code yet |
| T18 final population | `high_water_uses_final_population_not_a_transient_early_activation`, `active_withdrawal_preserves_original_activation_and_high_water`, `declined_population_never_reuses_an_old_activation_ordinal` | Final-state counters/ordinary decline; actual competing-branch reorg/canonical recovery MB-03; monetary qualification unresolved |
| T19 candidate failure | `invalid_last_operation_rolls_back_expiry_indexes_sequences_and_withdrawal`, `malformed_signature_cannot_leave_an_earlier_enrollment_or_consume_sequence`, plus whole-parent equality in rejection helpers | Combined late rejection includes expiry/index/sequence/withdrawal changes and successful retry; candidate never published on failure; mixed money/root rejection MB-02 |
| T20 direct budgets | `operation_count_cap_is_checked_before_ancestor_lookup_or_signature_work`, `membership_and_new_enrollment_caps_apply_to_direct_input`, `full_block_byte_budget_includes_header_count_and_each_length_prefix`, `pending_capacity_is_freed_by_ordered_withdrawal_not_early_activation`, `aggregate_approval_budget_retains_history_and_leaves_withdrawal_available`, `per_record_approval_budget_cannot_be_bypassed_across_blocks` | Counts and 168+4/op+message byte accounting, exact/+1 limits, ordered pending capacity and retained approval caps; full mixed block/account budgets MB-02 |
| T22 finite retained history | `record_capacity_includes_seeds_and_tombstones_but_allows_withdrawal`, `expired_pending_frees_quota_without_freeing_key_or_evidence`, `aggregate_approval_budget_retains_history_and_leaves_withdrawal_available` | Capacity does not evict replay protection; expiry/withdrawal work at full history capacity; restart/codec preservation pending |

Observed locally and in CI: **254 workspace tests = 203 retained + 51 B3**;
**147 personhood release reruns = 96 retained + 51 B3**, not extra unique tests.
Documentation command passes with zero cases. Both existing independent reference
checks and all three synthetic examples pass, including new `state_v2_demo`. New
Rust files pass formatting checks; allocation/runtime-data audit and unchanged
Cargo.lock pass. No new dependency, minted funds or biometric archive is introduced.

B3's approval capacity rules are explicit synthetic limits: 64 per record and total
retained approvals at most configured max_records. They never evict history or lower
quorum at capacity; a full archive can stop further admission. This is boundedness
evidence, not a guarantee of fair admission or production availability.

At the historical B3 checkpoint B4 remained next. Section 6 records its later
component coverage. Live ledger/money/device integration and the full 35-row
acceptance remain unfinished.


## 6. Historical B4 commitment, cache and grouped-replay component coverage

The implementation adds `src/v2/state_codec.rs` as a private child of the state
module and `src/v2/replay.rs`. Verified published source is
`21e2272b5ac92c00e840d7aaa26a99570a5ec24f`, tree
`31dfafa463935c640cd5cb31b88588575fc06d4b`.
Exact-source [push CI 44404337870](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404337870)
passed with job 135155424668 full logs and checkout inspected.
[PR CI 44404351436](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44404351436)
also passed. The same source tree passed local Rust 1.98.0 checks. This map
identifies executable cases and their exact component scope.
The new integration suites contain 26 snapshot + 10 vector + 31 replay cases,
plus two private replay-work arithmetic cases. The compile-fail documentation
case protects private snapshot-state access; the reviewed public API exposes no
conversion into a runtime registry. **69 new B4 tests bring the workspace total to 323** (254 + 69).
**216 personhood tests passed again in release**, and the documentation command
passed its one compile-fail case. All three independent references, all three
synthetic examples, changed-file formatting and preservation/allocation/runtime-data
checks passed. Release reruns are not additional unique tests.

| Case/subcase | Actual tests / files | Verified component scope and remainder |
|---|---|---|
| T16/T17 fixed commitments | `state_vectors_v2.rs`: `bootstrap_with_height_zero_activation_matches_independent_commitment`, `pending_records_with_absent_activation_match_independent_commitment`, `activation_and_pending_withdrawal_match_independent_commitment`, `empty_block_expiry_with_retained_approval_matches_independent_commitment`, `active_withdrawals_and_persistent_high_water_match_independent_commitment` | Five independently constructed MPS2 state preimages/roots, including all statuses, approvals, sequences, absent/present activation and high-water; actual block/combined ledger roots MB-02 |
| T17 deterministic layout | `state_codec_v2.rs`: `seed_encoding_matches_fixed_prefix_size_and_domain_contract`, `records_must_be_strictly_sorted_and_distinct`, `approvals_must_be_strictly_sorted_and_distinct`, `activation_none_is_distinct_from_zero_and_positive_seed_activation`, `tip_hash_does_not_enter_state_commitment` | Exact 90-byte prefix, raw-ID/key ordering, option tags and external tip exclusion; no circular block hash inside membership root |
| T17 cache consistency | `all_derived_counters_are_checked`, `seed_origin_id_descriptor_and_inclusion_are_bound`, `pending_and_seed_sequences_cannot_be_invented`, `repeated_outgoing_sequence_across_different_requests_rejects`, `inclusion_index_and_approval_acceptance_height_are_consistent`, `expired_pending_and_active_metadata_cannot_be_interchanged` in `state_codec_v2.rs` | Derived indexes/counters, origins, sequence conservation and lifecycle metadata checked; internally coherent cache still does not prove canonical history |
| T05/T06 cache evidence | `corrupted_enrollment_and_approval_signatures_are_rejected`, `changed_approval_kind_or_target_is_not_a_valid_cache`, `validly_signed_origin_cannot_reuse_a_retained_key_or_evidence`, `validly_signed_origin_with_wrong_profile_or_window_rejects` | Strict signature checks and retained reference consistency, not biological deduplication |
| T21 snapshot input budgets | `snapshot_bound_is_independent_of_diagnostic_log_limit`, both `every_*_truncation_rejects` cases, `trailing_bytes_and_oversized_input_reject`, `unknown_magic_version_and_all_tags_reject`, `declared_record_count_cannot_reserve_unbacked_input`, `declared_approval_counts_are_bounded_before_entries` | Separate MPS2 state-size and configured history/approval caps, actual-byte checks and exact tags before allocating claimed counts; disk/frame reads remain MB-03/04 |
| T26 cache authority | `internally_consistent_cache_never_establishes_current_state`; `state_vectors_v2.rs::independent_snapshots_decode_but_match_only_the_exact_trusted_state`; compile-fail documentation test | Untrusted wrapper has no runtime conversion; cache verification requires equality with independently trusted reconstructed state, not just signatures or internal accounting |
| T11/T16/T17/T22 full replay | `replay_v2.rs`: `replay_recovers_full_lifecycle_records_indexes_sequences_and_counters`, `replay_restores_tombstone_duplicate_protection_and_remains_executable`, `empty_groups_preserve_inclusive_expiry_and_are_not_omitted` | B3/B4 and MB-02B expiry/rollback retained; MB-02C actual empty-block expiry and MB-03A complete immutable-history reopening preserve expiry and retained history; evolving tip recovery remains MB-03B |
| T13 exact diagnostic operations | `exact_group_commitment_binds_context_height_count_order_and_signed_bytes`, `reordered_commuting_approvals_reject_even_when_membership_root_is_identical`, `alternate_valid_withdrawal_bytes_cannot_replace_the_canonical_operation` | State roots alone cannot distinguish all histories; separate trusted group commitment binds exact operation bytes/order; future MRB2 Merkle root remains MB-02 |
| T09/T26/T28 trusted provenance | `genesis_hash_root_and_empty_group_commitment_are_independently_required`, `missing_trusted_metadata_rejects_genesis_intermediate_or_final_height`, `each_trusted_intermediate_hash_root_and_exact_group_are_checked`, `explicit_expected_context_height_tip_and_root_are_all_required`, `an_older_valid_prefix_cannot_satisfy_the_current_trusted_expectation` | B2–B4 and MB-02A/B authorization retained; MB-02C accepted candidate ancestry and MB-03A complete disk replay reject wrong-fork anchors; global canonical selection and durable reorgs remain MB-03B/06 |
| T05/T07/T09/T14 execution during replay | `invalid_signature_fails_even_when_group_metadata_matches_the_bad_bytes`, `wrong_sequence_cannot_be_authorized_by_a_matching_group_commitment`, `signed_wrong_anchor_and_wrong_context_fail_despite_matching_group_metadata`, `merging_blocks_does_not_let_a_new_member_review_in_its_activation_block` | Exact relevant error assertions confirm B2/B3 re-execution cannot be bypassed with metadata matching invalid bytes |
| T19 immutable rejection | `late_failure_publishes_no_registry_and_preserves_inputs_for_retry`, `construction_copies_inputs_and_decoding_never_establishes_replay_authority` | Late mismatch returns no final registry; bootstrap/transcript/existing state unchanged and successful retry possible; combined money/membership and durable publication remain MB-02/03 |
| T20/T21 log/group bounds | `every_log_truncation_and_trailing_byte_is_rejected`, `huge_claimed_counts_and_lengths_fail_without_claimed_allocation`, `incorrect_operation_lengths_versions_and_kinds_are_rejected`, `zero_skipped_duplicate_reordered_and_maximum_heights_are_rejected`, `exact_log_byte_limit_passes_and_one_byte_under_rejects`, `replay_group_budget_is_4096_including_empty_groups`, direct count and 168-byte block-budget cases | MPH2 strict 74/44-byte containers; configured byte/count limits and explicit 4,096 diagnostic groups; no truncation or history eviction at exhaustion |
| T20 work and view limits | `impossible_expected_height_rejects_before_any_trusted_history_lookup`, `replay_caches_each_trusted_height_once_before_execution`, `replay_work_budget_charges_configuration_copies_even_for_empty_groups`; private `work_budget_accepts_exact_limit_and_rejects_next_item_without_mutation`, `work_budget_never_wraps_or_consumes_a_rejected_charge` | Aggregate 1,048,576 record/review/configuration-item proxy, including 65,536 zero-value allocations copied per state; metadata cached once, no loop to arbitrary expected height; not a wall-clock or global availability guarantee |
| Independent grouped reference | `state_vectors_v2.rs::replay_encoding_and_group_commitments_match_independent_vectors`, `replay_reconstructs_state_against_independently_pinned_roots_and_groups`; `tools/reference_state_v2.py --check` | Standard-library Python builds exact MPS2/MPH2/group preimages/hashes independently; Rust reproduces the complete fixture, preserving earlier B1/B2 vectors |

The MPH2 membership-only format must not silently drop transfers from future mixed
blocks: that would change inclusion operation indices and combined state semantics.
MB-02 must explicitly integrate/version full blocks and exact operation positions.
No B4 test establishes actual MRB2 consensus, canonical fork choice, durable latest
state, real-human matching, private credentials, grants, fees or device readiness.
Neither Gate A nor Gate B is completed by this component coverage.

At the B4 checkpoint MB-01B was complete in this scope. The planned next MB-02A
slice covered unified operation and BlockV2 bytes/commitments with independent
vectors; section 7 records its subsequent implementation. Full T23–T28
ledger/chain authority and mixed execution then remained MB-02B/C or MB-03 as
applicable; section 8 records later staged execution. Passing the 69 B4 cases did
not mark all acceptance rows complete.

## 7. Historical MB-02A transfer, unified operation and block component coverage

Published source: `35ad0cf708bd4ceeb4faa306ba4f7bd74912c382`, tree
`930aa22bf9810525372fd31926812c1bde6dcf81`. Local verification passed **403 workspace
tests (323 retained + 80 new)** and one documentation test; **121 core release**
tests and **216 personhood release** tests plus its one documentation test passed.
All four reference checks and four synthetic examples passed. Release reruns are
not additional unique tests. The 80 additions comprise 37 cases in
`crates/mirror-core/tests/operations_v2.rs`, 33 in `blocks_v2.rs` and 10 in
`block_vectors_v2.rs`.
[Exact-source push CI 44428924399](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428924399)
passed; checkout `35ad0cf` and the full 1,392-line log for job `135240927609` were
inspected. The same test, reference, example and audit checks passed without
warnings or errors. [PR CI 44428936866](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44428936866)
also passed. CURRENT_STATUS.md records the acceptance details.

| Case/subcase | Actual tests / files | Verified component scope and remainder |
|---|---|---|
| T01/T04 all operation codecs | `operations_v2.rs`: `transfer_has_exact_frozen_offsets_and_length`, `common_prefix_matches_existing_membership_encoding_except_kind`, `unified_round_trip_preserves_all_kinds_bytes_hashes_and_signatures`, `membership_only_decoder_remains_closed_to_transfer_kind`, truncation/kind/version/legacy cases | B1/B2, MB-02A/C format/no-fallback cases retained; MB-03A explicitly rejects/preserves legacy directories and recovery siblings, with no implicit init or migration; bounded enumeration may classify oversized layouts as InvalidLayout; live V2 node integration remains MB-04 |
| T05 authentication | `every_unsigned_transfer_byte_is_authenticated`, `every_transfer_signature_byte_is_checked`, weak-key/signature/domain-transplant cases | Strict signing and ID domains bind the full Transfer envelope; cryptographic validity does not establish funds or eligibility |
| T08/T09/T10 contextual checks | `zero_amount_is_signed_and_decoded_but_rejected_contextually`, `nonzero_fees_remain_unsupported_by_synthetic_profile`, anchor/window/height boundary cases, `authorization_does_not_claim_balance_current_sequence_or_membership` | Positive amount/zero fee and trusted-parent authorization; actual balance, nonce and membership execution remain MB-02B |
| T20/T21 mixed-body bounds | `blocks_v2.rs`: `all_operation_count_boundaries_apply_to_construct_encode_decode_and_root`, `membership_subcount_is_enforced_across_interspersed_transfers`, enrollment/byte-budget cases, huge declared count/length cases, `encoding_and_integrity_recheck_stricter_membership_and_enrollment_budgets` | Direct construction, encoding, decoding and root calculation enforce limits before input-sized copies; disk reads and session budgets remain MB-03/04 |
| T23 MRB2 structure/hash | `exact_header_layout_and_unsigned_endianness`, header/complete-block truncation cases, `every_nonversion_header_byte_is_bound_to_the_hash`, `empty_block_has_exact_size_magic_count_and_identity` | Header160/empty168 and domain-bound header hashes; no PoW, timestamp, actual ancestry or pinned-genesis validation |
| T13/T24 exact body integrity | `mixed_block_round_trip_preserves_global_positions_and_exact_bytes`, `count_commitment_distinguishes_the_duplicate_last_merkle_ambiguity`, `ordered_mixed_operation_commitment_detects_reordering`, `complete_signed_bytes_including_signatures_are_committed`, corrupt-root/signature/context cases | Count/order/complete signed bytes committed; full mixed positions preserved; no filtering transfers into MPH2 |
| T23/T24 independent reference | All 10 `block_vectors_v2.rs` cases; `tools/reference_blocks_v2.py --check` | Independently computed Transfer preimages/signatures/IDs, seven complete block cases and header-field variations; empty/single/odd/mixed/reordered/duplicate-last cases; B2 membership bytes unchanged |
| T25/T27 explicit boundary | `integrity_does_not_claim_transfer_policy_or_blockchain_acceptance` | `verify_integrity` checks budgets, configuration, operation root and signatures only; it does not execute accounts/membership, validate the supplied ledger root or publish a canonical chain |

The Python reference uses standard-library hashing/encoding and independently
generated Ed25519 signature constants from public test seeds. Expected vectors
were not taken from Rust output. Header fixtures do not establish valid genesis,
PoW, balances or real ancestry.

At the MB-02A checkpoint, **MB-02B was next:** deterministic combined-ledger
execution with original mixed-operation positions and no publication after late
failures. Section 8 records that implementation. Neither Gate A nor Gate B was
completed by MB-02A.

## 8. Historical MB-02B atomic ledger staging and combined commitments

Published source: `420d7114ddf782469a6ace6ba0a2c4076fb93aaf`, tree
`6871ad0fbc178bd76262a26416a7d255d9fcbd66`. Local acceptance passed **462 workspace
tests = 403 retained + 59 new** and **three compile-fail documentation tests**.
Release reruns passed **121 core**, **234 personhood plus three documentation
cases**, and **49 state tests**. All five reference checks and five synthetic
examples passed. Release reruns and example assertions are not additional unique
workspace tests.
[Exact-source push CI 44619155972](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619155972)
passed; the exact `420d711` checkout and full job `135895834032` logs were inspected.
The same workspace, documentation, release and audit gates passed, along with all
five reference checks and five examples.
[PR CI 44619172378](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44619172378)
also passed. [CURRENT_STATUS.md](../CURRENT_STATUS.md) records the final acceptance
checkpoint and continuation.

The 59 additions comprise 18 tests in
[indexed_state_v2.rs](../crates/mirror-personhood/tests/indexed_state_v2.rs),
28 in [ledger_v2.rs](../crates/mirror-state/tests/ledger_v2.rs),
five private fault cases in [v2/tests.rs](../crates/mirror-state/src/v2/tests.rs),
and eight in [ledger_vectors_v2.rs](../crates/mirror-state/tests/ledger_vectors_v2.rs).
Two new compile-fail cases protect consumed-stage reuse and cloning, alongside
the existing snapshot-to-authority case. The contract is
[LEDGER_STATE_V2.md](LEDGER_STATE_V2.md).

In the table below, unqualified ledger test names are in `ledger_v2.rs`; `indexed`
means `indexed_state_v2.rs`, `vectors` means `ledger_vectors_v2.rs`, and `private`
means `mirror-state/src/v2/tests.rs`.

| Case/subcase | Actual tests | Verified component scope and remainder |
|---|---|---|
| T07 separate sequences | `transfer_and_membership_sequences_are_independent`, `replay_or_skipped_account_sequence_cannot_spend`, `zero_balance_sender_history_survives_recredit_and_replay`; private `exhausted_account_sequence_fails_without_any_mutation` | Account and membership sequences remain separate; replay/skips/exhaustion reject without changing the parent |
| T08 conservation and arithmetic | `successful_transfer_creates_recipient_and_preserves_supply`, `self_transfer_requires_funds_and_consumes_exactly_one_sequence`, `maximum_supply_self_transfer_and_full_transfer_do_not_overflow`, `zero_amount_nonzero_fee_and_bad_signature_precede_state_changes`; private `recipient_credit_overflow_is_checked_before_sender_mutation`, `account_component_rejects_corrupt_balance_sum_and_supply` | Positive zero-fee transfers preserve exact initial synthetic supply; checked arithmetic and self-transfer behavior tested; no mint or reward mechanism |
| T09/T10 parent authorization | `wrong_parent_context_height_hash_or_missing_hash_rejects_whole_body`, `operation_lifetime_and_anchor_are_checked_before_execution` | B2–B4 and MB-02A/B authorization retained; MB-02C accepted candidate ancestry and MB-03A complete disk replay reject wrong-fork anchors; global canonical selection and durable reorgs remain MB-03B/06 |
| T11 expiry and rollback | `empty_block_expires_membership_without_changing_account_component`, `failure_after_expiry_does_not_publish_the_expiry`; indexed `transfer_only_membership_stage_still_advances_expiry` | Empty/transfer-only stages still advance membership height and expiry; discarded candidates publish neither expiry nor accounts |
| T13/T14/T15 mixed execution order | `same_signed_transfers_require_receive_before_spend`, `first_failure_is_selected_in_original_mixed_order`, `same_block_activation_does_not_grant_review_rights`, `reviewer_withdrawal_is_reflected_before_end_block_activation` | Executes original order and reports its first per-operation failure after preflight; parent eligibility and end-block reviewer survival retained |
| T16/T17 full-body positions | `full_block_indices_survive_sparse_membership_and_snapshot_roundtrip`; indexed `activation_follows_original_full_enrollment_positions_at_block_end`; vectors `full_operation_positions_survive_interspersed_transfers_and_are_committed` | Enrollment indices include transfers and may exceed the membership-count cap; activation order and committed membership bytes preserve full positions |
| T19/T27 atomic staging | `late_transfer_failure_discards_earlier_membership_and_accounts`, `late_membership_failure_discards_earlier_accounts_and_membership`; private `final_account_invariant_failure_discards_completed_candidate`; indexed `finish_requires_all_preflighted_membership_operations`, `wrong_step_index_cannot_skip_or_reapply_a_planned_operation`, `late_stateful_failure_discards_earlier_candidate_mutation` | B3/B4 and MB-02B staging rollback retained; MB-02C complete-chain equality after late execution/root failure retained; MB-03A initial-publication errors are explicit, while subsequent durable tip-update atomicity remains MB-03B |
| T20 full budgets and account capacity | `full_operation_count_preflight_precedes_parent_access`, `full_membership_and_enrollment_quota_preflight_precedes_parent_access`, `indexed_membership_and_full_block_share_exact_transfer_byte_budget`, `account_limit_rejects_new_recipient_but_keeps_existing_and_zero_records`; indexed plan/count/budget cases | Core and projected membership agree on 243-byte transfers plus four-byte prefixes; exact mixed size 710 passes and 709 rejects; bounded plan/index checks precede state clone/signatures |
| T25 staged commitments | `account_encoding_and_component_roots_follow_canonical_domains`, `provenance_hash_is_excluded_from_committed_state`; vectors `independent_accounts_membership_economics_and_combined_roots`, `empty_block_expiry_changes_only_membership_and_height_commitments`, `staged_roots_do_not_depend_on_later_external_block_hash_binding` | Canonical account, membership, disabled-economics and combined roots reproduced independently; current tip hash excluded; these are staged roots, not accepted header commitments |
| T26 cache and fixture boundaries | indexed `plausible_changed_full_index_remains_untrusted_and_out_of_full_range_rejects`; vectors `signed_python_fixture_decodes_and_reexecutes_from_verified_bootstrap` | Coherent changed-index caches still fail trusted-state comparison; signed synthetic fixtures reexecute from verified bootstrap; actual accepted MRB2 replay/recovery remains MB-02C/03 |
| T34/T35 disabled monetary policy | `bootstrap_uses_only_explicit_allocations_including_zero_records`, `empty_allocations_create_no_supply_or_implicit_member_accounts`, `withdrawing_membership_does_not_destroy_or_restrict_accounts`; vectors `explicit_zero_entries_and_supply_survive_all_membership_changes` | Only explicit synthetic allocations exist; membership changes neither mint funds nor restrict account spending; D-01/D-04 and final economy remain unresolved |

The independent Python reference
[reference_ledger_v2.py](../tools/reference_ledger_v2.py) reconstructs explicit
synthetic configuration, signed operations, account bytes, MPS2 bytes and component
preimages. Rust independently executes the same signed fixture and compares the
committed roots. Frozen older references remain unchanged; neither Python fixture
hashes nor `bind_to_validated_block` establish real block authority.

The MPS2 extension changes only the structural runtime-enrollment index bound from
the membership-count limit to `max_operations`. Its bytes/domains are unchanged;
older decoders may reject the newly possible mixed states. Decoding still returns
an untrusted wrapper. MPH2 remains membership-only and cannot reconstruct a mixed
ledger by omitting transfers.

**At the MB-02B checkpoint, next was MB-02C:** validate actual genesis, headers,
height/parent/configuration, timestamp/target/work, body commitments and the claimed
final ledger root before binding or accepting staged state. T27's wrong-final-root
subcase and T26's actual complete-block replay were not completed by MB-02B; section
9 records their later in-memory implementation. MB-02 remained IN_PROGRESS at
that checkpoint. Durable recovery/reorgs, human consensus, issuance, biometric
admission, privacy and both launch gates were not established by MB-02B.

## 9. Historical MB-02C pinned genesis, actual block acceptance and complete replay

Published source: `db2cb58ba5a9a828ec949debd1b52b5915a66f3b`, tree
`3fa1d3b44abb885898cc256f9525355025649a0a`, identical to the locally tested source.
Local acceptance passed **523 workspace tests = 462 retained + 61 new** and the
same **three compile-fail documentation tests**. Release reruns passed **core
121**, **personhood 234 plus the same three documentation cases**, **state 49**,
**consensus 23** and **chain 57**. All **six independent references**, **six
synthetic examples**, changed-file formatting and preservation/allocation audits
passed. Reruns and example assertions are not additional unique workspace tests.

[Exact-source push CI 44623282448](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623282448)
**passed**. Job `135910328823` full logs and the exact
`db2cb58ba5a9a828ec949debd1b52b5915a66f3b` checkout were inspected; all workspace,
documentation, release, reference, example and audit gates above passed.
[PR CI 44623296178](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44623296178)
also **passed**. This evidence was verified on 2026-11-20. Later documentation-head
CI is separate and is not inferred from the source run.
[CURRENT_STATUS.md](../CURRENT_STATUS.md) records those checkpoints separately.

The 61 additions comprise **15 work**, **14 manifest**, **20 public chain**,
**three private budget** and **nine independent-vector** cases. In the table,
`manifest` means [manifest_v2.rs](../crates/mirror-chain/tests/manifest_v2.rs),
`chain` means [chain_v2.rs](../crates/mirror-chain/tests/chain_v2.rs),
`vectors` means [chain_vectors_v2.rs](../crates/mirror-chain/tests/chain_vectors_v2.rs),
`work` means [pow_v2.rs](../crates/mirror-consensus/tests/pow_v2.rs), and
`private` means [chain v2/tests.rs](../crates/mirror-chain/src/v2/tests.rs).
The acceptance contract is [CHAIN_V2.md](CHAIN_V2.md).

| Case/subcase | Actual tests | Verified component scope and remainder |
|---|---|---|
| T02/T03/T28 manifest authority | manifest: `caller_selected_configuration_cannot_be_replaced_by_manifest_bytes`, `caller_selected_genesis_cannot_be_replaced_by_manifest_bytes`, `every_seed_proof_is_required_by_constructor_and_decoder`, `changed_configuration_cannot_reuse_old_seed_proofs_or_identity`; vectors: `independently_encoded_manifest_and_all_seed_proofs_match` | B1/B2 context and MB-02B/C execution/manifest checks retained; MB-03A selected metadata, manifest and complete disk replay require the independently chosen C and exact genesis |
| T01/T20/T21 manifest framing | manifest: `all_truncations_and_trailing_bytes_reject`, `unknown_magic_and_version_do_not_fallback`, `oversized_declared_configuration_rejects_before_allocation`, `oversized_declared_proof_count_rejects_before_signature_work`, `total_manifest_limit_is_checked_before_magic_or_configuration` | B1/B2, MB-02A/C format/no-fallback cases retained; MB-03A explicitly rejects/preserves legacy directories and recovery siblings, with no implicit init or migration; bounded enumeration may classify oversized layouts as InvalidLayout; live V2 node integration remains MB-04 |
| T23/T28 actual genesis | chain: `genesis_is_executed_pinned_and_exposes_only_actual_ancestry`, `pinned_genesis_still_requires_exact_fields_and_executed_roots`, `genesis_cannot_execute_transactions`; vectors: `alternate_valid_genesis_nonce_cannot_replace_the_selected_identity` | A pinned genesis still requires valid exact fields, empty body, work and executed roots; a different valid nonce cannot replace the selected hash |
| T23 fixed-target work | work: `mining_matches_independent_v2_header_vector`, `configured_bits_reject_a_valid_self_selected_easier_target`, `numerically_equal_compact_aliases_do_not_replace_configured_bits`, `work_comparison_is_big_endian_and_includes_exact_target`; chain: `exact_target_and_actual_work_are_both_required` | Full domain-separated 160-byte header, literal configured bits and raw big-endian target comparison; easy development work is not equal-human consensus |
| T20 local work-search bounds | work: `known_solution_is_not_found_when_budget_ends_one_attempt_early`, `search_starts_at_supplied_nonce_instead_of_resetting_to_zero`, `zero_and_over_cap_budgets_are_rejected_before_work`, `failed_final_nonce_never_wraps`, `valid_final_nonce_is_accepted_without_incrementing` | Explicit local attempt budget, inclusive starting/final nonce and no wrap; helper ceiling does not limit other miners or confer block authority |
| T23 proposal/header boundary | chain: `proposal_is_read_only_and_child_binds_only_after_full_acceptance`, `child_header_failures_are_atomic_and_timestamp_is_strict`, `timestamp_has_no_wall_clock_dependency_or_wrapping_successor` | Preparation never publishes state; accepted next height/parent/C and strict parent timestamp are required; no wall-clock/future-time policy or production liveness claim |
| T13/T24/T27 body and atomic acceptance | chain: `valid_work_cannot_override_wrong_operation_commitment`, `earlier_execution_error_precedes_later_signature_error`, `late_operation_failure_discards_earlier_accounts_and_membership`, `wrong_final_root_with_valid_work_rolls_back_full_mixed_transition`; vectors: `independently_remined_wrong_ledger_root_is_rejected_without_partial_publication` | Full ordered commitment and original per-operation failure order; late execution/root rejection preserves complete accepted blocks, ledger, manifest and resource counters; disk publication remains MB-03 |
| T09 actual branch ancestry | chain: `actual_fork_ancestry_controls_operation_anchors`; vectors: `valid_work_and_signatures_cannot_authorize_a_different_forks_anchor` | Only accepted candidate blocks supply operation ancestry; correct header parent/work/signatures do not authorize another branch's anchor |
| T11/T16/T17/T25 complete state vectors | vectors: `independently_mined_genesis_and_mixed_blocks_match_complete_ledger_states`, `independently_signed_operations_and_actual_anchor_intents_match`; chain: `empty_actual_blocks_process_expiry_and_preserve_accounts` | B3/B4 and MB-02B expiry/rollback retained; MB-02C actual empty-block expiry and MB-03A complete immutable-history reopening preserve expiry and retained history; evolving tip recovery remains MB-03B |
| T18/T25 genuine membership forks | vectors: `real_membership_fork_replays_distinct_membership_with_identical_accounts` | B3 final-state high-water retained; MB-02C genuine branches and MB-03A separate immutable stores reconstruct distinct memberships/counters; durable canonical replacement and orphan recovery remain MB-03B |
| T26/T28 complete replay/checkpoints | chain: `typed_and_encoded_replay_reproduce_complete_branch_and_counters`, `replay_requires_all_five_trusted_checkpoint_fields_and_complete_history`, `replay_rejects_invalid_intermediate_block_and_does_not_change_source`, `encoded_replay_checks_actual_format_not_snapshot_authority`; vectors: `independent_complete_block_bytes_replay_to_the_selected_checkpoint`, `authentic_old_prefix_or_wrong_expected_checkpoint_is_not_latest_tip_authority` | MB-02C actual MRB2 execution and MB-03A complete disk reopening rebuild state against independent C/genesis/height/tip/root expectations; MPS2 remains untrusted; checkpoint freshness is an external responsibility, and live canonical updates remain MB-03B/06 |
| T20/T21 aggregate resource bounds | chain: `replay_count_and_total_bytes_are_preflighted_before_decode`; private: `resource_caps_accept_exact_boundaries_and_reject_one_more`, `resource_arithmetic_never_wraps`, `replay_aggregate_bytes_are_checked_without_overflow` | Inclusive 4,096 blocks, 16 MiB complete encoded history and 1,048,576 state/configuration-item charges with checked arithmetic; local profile limits, not a maximum-sized replay benchmark or network/session guarantee |

[reference_chain_v2.py](../tools/reference_chain_v2.py) independently constructs
the public synthetic configuration, exact manifest, seed proofs, signed actual-
ancestor operations, mined headers/blocks and complete ledger states. Twenty
signatures were independently generated and verified before Rust; all nine vector
tests matched on their first run. Expected values were not copied from Rust or
changed to bless a failure. Normal `--check` uses the Python standard library;
optional `--verify-signatures` regenerates the public test signatures with Python
cryptography. Older fixtures remain unchanged.

The fixture includes five main-branch blocks, an empty expiry block, a genuine
membership fork with identical accounts but different membership roots, another
valid genesis nonce, re-mined wrong ledger root and wrong-branch operation anchors.
The synthetic `chain_v2_demo` additionally runs two separate in-memory validators
against actual mixed blocks and complete replay. It does not run two networked
processes or write durable state. These are implementation tests, not an external
security audit or proof of honest human admission.

`ChainV2` owns and validates its ancestry; it never accepts a caller-bound ledger,
MPH2 transcript or MPS2 cache as an authoritative complete chain. Lower-level
`bind_to_validated_block` remains a trusted staging interface for other callers.
Replay requires a separately selected checkpoint; it cannot detect an application
choosing an obsolete checkpoint or find the latest global tip itself. The bounded
branch stops on cap exhaustion. Timestamp `u64::MAX` admits no successor under
the deliberately finite strict-parent rule. Neither behavior is a production
availability policy.

**At the MB-02C checkpoint, MB-02A/B/C completed MB-02 as a VERIFIED_COMPONENT
for bounded in-memory acceptance, and MB-03A was next.** T28's durable restart
portion and T29–T32 were still open at that checkpoint. Section 10 records the
later immutable-storage increment; global canonical choice, human finality, the
live V2 daemon, grants/fees/rewards, private biological matching, real-device
support and both launch gates remain unfinished.

## 10. Historical MB-03A immutable selected storage and process ownership

Contract [STORAGE_V2.md](STORAGE_V2.md), **MIRROR-MB03A-001**, storage profile 1,
generation zero. Source commit/publication and exact-source CI are tracked in
[CURRENT_STATUS.md](../CURRENT_STATUS.md), separately from local execution and
historical source runs. **Local Rust 1.98.0: 588 workspace cases passed, one
subprocess helper ignored; three existing documentation cases passed.** Storage
release reran 75 cases; all seven Python references passed. Release reruns and
the nine injected boundary scenarios below are not extra unique test cases.

The **65 new cases** are 24 selection codec/vector, 34 filesystem, five substantive
lock and two fault-injection cases. The lock helper is marked ignored in the
normal suite and selected only using `--ignored --exact lock_process_probe` by
parent tests. Its internal subprocess success is not counted as another workspace
case. In the table, `selection` means [selection_v2.rs](../crates/mirror-storage/tests/selection_v2.rs),
`filesystem` means [storage_v2.rs](../crates/mirror-storage/tests/storage_v2.rs),
`locks` means [storage_lock_v2.rs](../crates/mirror-storage/tests/storage_lock_v2.rs),
and `faults` means [v2/tests.rs](../crates/mirror-storage/src/v2/tests.rs).
The listed names identify representative actual cases; those files contain the
complete suite, including each individual external-pin and structural-bound test.

| Case/subcase | Actual tests | Component scope and remainder |
|---|---|---|
| T01 explicit lifecycle and legacy separation | filesystem: `missing_and_empty_directories_never_initialize_during_open_or_inspection`, `explicit_initialization_does_not_create_missing_parent_directories`, `legacy_directory_and_recovery_siblings_are_preserved_without_migration`, `missing_store_does_not_restore_or_remove_legacy_backup_siblings` | Explicit init only; no legacy create/repair calls, recursive parent creation or silent migration; all existing data retained |
| T01/T28 partial and existing stores | filesystem: `partial_initialization_is_not_repaired_deleted_or_reinitialized`, `committed_history_cannot_be_silently_reinitialized`, `stale_pending_marker_beside_committed_selection_is_not_silently_removed` | Pending files do not authorize selection, complete stores are not overwritten, and ambiguous layouts are not automatically repaired |
| T02/T28 external identity and checkpoint | selection: `recomputing_checksum_does_not_authorize_changed_checkpoint_fields`, `external_configuration_pin_is_required`, `external_genesis_pin_is_required`, `external_height_pin_is_required`, `external_tip_pin_is_required`, `external_ledger_root_pin_is_required`; filesystem: `independent_identity_must_agree_with_the_supplied_checkpoint_before_io`, `every_external_checkpoint_field_is_required_and_never_selected_from_disk` | Independently retained C/genesis/height/tip/root required; checksum integrity cannot replace external selection |
| T20/T21 exact selected metadata | selection: `every_truncated_record_and_trailing_bytes_are_rejected`, `wrong_magic_version_and_profile_are_rejected`, `checksum_correct_nonzero_generation_is_rejected`, `checksum_correct_height_count_disagreement_is_rejected_without_overflow`, `inclusive_structural_limits_do_not_require_unbounded_allocations` | Exact 232-byte MSS2, profile1/generation0, checked structural bounds; no format fallback or future update semantics |
| T20/T21 actual disk-read budgets | filesystem: `an_oversized_selection_marker_is_rejected_at_its_exact_size_boundary`, `oversized_manifest_metadata_is_rejected_before_unbounded_read`, `per_block_limit_prevents_oversized_file_reads`, `aggregate_history_limit_rejects_individually_bounded_sparse_files_before_decode` | Manifest, per-block and total selected lengths preflighted before block-payload reads; large sparse files exercise actual disk metadata, not fabricated allocation alone |
| T21/T28 corruption and layout | filesystem: `truncated_and_corrupt_selection_markers_do_not_fall_back_to_a_new_chain`, `truncated_and_tampered_manifest_bytes_are_rejected_without_side_effects`, `missing_selected_block_rejects_without_resetting_or_repairing`, `unselected_tail_and_unexpected_files_are_rejected_and_retained`, `truncated_and_trailing_block_bytes_are_never_accepted_or_rewritten` | Complete selected layout required; corrupt or incomplete input rejects without resetting genesis, accepting a prefix or rewriting files |
| T23/T27 exact actual-chain validity | filesystem: `valid_work_with_a_false_intermediate_ledger_root_is_rejected_by_full_replay`, `another_valid_genesis_nonce_does_not_replace_the_pinned_genesis` | Stored blocks still pass complete actual acceptance; valid work or self-consistent metadata cannot excuse false roots or a different selected genesis |
| T18/T25/T26 selected membership history | filesystem: `complete_selected_history_replays_transfers_membership_and_withdrawals`, `distinct_membership_branches_remain_independently_selected`, `block_files_are_plain_canonical_mrb2_bytes_without_cached_ledger_authority` | Actual full mixed histories reopen, including a real membership-divergent fork with equal accounts; no cache authority, canonical replacement or network synchronization is implied |
| T22/T26 retained history | filesystem: `exact_recovered_ledger_preserves_permanent_records_and_capacity_counters` | Recomputed pending/revoked/expired records, reservations, account/member sequences and resource charges survive reopening; no tombstone deletion or budget reset |
| T28 selected prefixes | selection: `old_valid_selection_is_rejected_against_later_checkpoint`, `valid_membership_fork_is_rejected_against_other_same_height_branch`; filesystem: `a_valid_older_prefix_does_not_satisfy_a_later_selected_checkpoint` | Authentic old/other histories do not satisfy the independently selected checkpoint; an obsolete checkpoint intentionally supplied by the caller is not detected as globally stale |
| T24/T25 independent storage bytes | selection: `genesis_selection_matches_independent_vector`, `mixed_history_selection_matches_independent_vector`, `membership_fork_selection_matches_independent_vector`, `manifest_checksum_is_cross_checked_against_actual_bytes` | Independent exact MSS2 bytes and MNM2 checksum for genesis, full history and actual fork; checksum is not authentication or freshness |
| T30 read-only behavior | filesystem: `read_only_inspection_keeps_all_file_names_and_contents_unchanged`, `unsuccessful_inspection_releases_ownership_for_a_later_checked_open` | Shared inspection does not create, mine, repair or mutate bytes/names/mtime; error returns no lingering ownership; OS atime changes are outside this guarantee |
| T30 independent local ownership | locks: `writer_excludes_independent_handles_and_drop_releases`, `shared_handles_exclude_writer_until_every_reader_drops` | Different descriptors conflict correctly; shared inspectors coexist and must all release before an exclusive owner can enter |
| T30 actual process lifetime | locks: `writer_in_another_process_releases_on_store_drop_while_process_lives`, `killed_writer_releases_without_unlinking_or_repairing_anything`, `shared_locks_in_separate_processes_coexist_and_exclude_a_writer` | Pipe handshakes establish acquired ownership, live-process Drop and process-death release; no sleeps, PID authority or lockfile deletion; killing a holder is not power-loss recovery |
| T30 static aliases and file types | filesystem: `root_symlink_is_rejected_without_changing_its_target`, `selected_files_and_blocks_directory_cannot_be_symlink_aliases`, `selected_regular_files_cannot_share_inodes_with_external_hard_links`, `nonregular_selected_file_is_rejected_without_reading_from_it` | Static unsafe paths and special files reject; trusted-parent assumptions remain, without a hostile same-user race guarantee |
| T29 preparation failures | faults: `prepublication_errors_never_expose_selected_state_or_cleanup_partial_files` | One test iterates seven selected boundaries before marker publication; no owning handle/selected state is exposed, partial files remain, and ownership releases |
| T29 uncertain publication | faults: `postpublication_errors_return_no_handle_and_require_checked_reconciliation` | One test iterates two selected boundaries after marker publication; explicit uncertainty returns no handle and complete visible history can reopen against retained pins; no retroactive power-loss durability proof |

The nine injected scenarios are root preparation; manifest synchronization;
synchronization of genesis and of the final block; block-directory synchronization;
prepared root-directory synchronization; pending-selection synchronization; selection
rename; and final root-directory synchronization. The callback injects an error
after each selected successful operation. It does not replace system calls with
partial writes, inject every possible errno, stop a machine between every block,
or simulate disk firmware, torn writes and power loss. These are **two tests
with nine scenarios**, not nine additional passing cases or universal crash safety.

[reference_storage_v2.py](../tools/reference_storage_v2.py) derives the new fixed
selection vectors independently from the preexisting frozen actual-chain fixture,
checks that fixture's digest, and preserves all prior bytes. Normal `--check` is
read-only. The bounded storage example opens only newly created temporary stores;
it does not migrate legacy node runtime data or start network peers.

Supported ownership scope is cooperative Linux processes with a trusted parent
tree and an unchanged root inode throughout every handle lifetime. Local tmpfs
and overlayfs were exercised; WSL2/ext4 was not directly run here. Native Windows
rejects; DrvFS/NFS/SMB remain unvalidated and there is no mount-type whitelist.
The read-only API uses shared kernel ownership and creates no lockfile. Static
path checks do not prevent a malicious same-user actor changing the namespace.

The selected payload cap is **19,559,084 encoded bytes**: 232 metadata bytes,
at most 2,781,636 manifest bytes and 16,777,216 complete MRB2 history bytes. This
is not a total resident-memory cap. Selected block count is at most 4,096 and
replay retains its 1,048,576 state/configuration-work limit. The four-entry root
enumeration bound can reject large legacy/foreign layouts as `InvalidLayout`
before more specific classification; rejection still preserves them.

**At the MB-03A checkpoint, next was MB-03B:** durable generation/accepted-tip
updates, staging/publication and explicit failure reconciliation. Section 11 records
that later increment. Development canonical fork choice must precede durable
replacement; then membership-aware reorganization and peer reconstruction follow.
T29's complete repair/reorg scope and T31–T32 remain unfinished. Final human
consensus, economics, biometric matching, device recognition and Gate A/B remain
unmet. Local passing tests are neither exact-source CI evidence nor an external
security audit; use CURRENT_STATUS for the separately verified publication state.

## 11. Historical MB-03B accepted-child append, fencing and clean-store reconciliation

Published implementation
[`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c`](https://github.com/Ahmed-Sobeih/mirror/commit/ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c),
tree `06d7c3f4fa24d96c844e2301c76c01f804151192`, matches the locally tested source.
The increment changes 14 files, with 3,568 insertions and 11 deletions.
[Exact-source push CI 44649111203](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649111203)
and [PR CI 44649119266](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44649119266)
both **SUCCEEDED**. Full push job `135999222947` logs confirmed exact checkout
`ecf5e7621bd39f6d2dff6a2f8d89f749a55b0d3c` and all tests, references, examples
and allocation/runtime-data audit gates. Later documentation-head CI is separate.
**MB-03B VERIFIED_COMPONENT — bounded accepted-child append/clean-state
reconciliation; MB-03 IN PROGRESS.**

Contract [STORAGE_APPEND_V2.md](STORAGE_APPEND_V2.md), distinct MSU2 profile 2.
The immutable StoreV2/MSS2 profile and all historical fixtures remain unchanged.
The increment adds **70 unique cases: 28 selector/vector, 31 filesystem integration,
eight private fault tests and three additional process ownership tests**. Local
Rust 1.98.0 verification passed **658 workspace/all-target tests**, one ignored
subprocess-only helper and the same **three documentation tests**. Release reruns
passed core 121, personhood 234 plus the same three doctests, state 49, consensus
23, chain 57 and storage 145; the helper remains ignored in ordinary enumeration.
All **eight independent references and eight examples** passed. The independently
inspected exact-source CI above passed the same gates. CURRENT_STATUS.md records
publication and later checkpoint evidence separately; earlier green runs do not
establish a newer documentation head's result.

Representative actual tests are linked below. `selector` means
[append_selection_v2.rs](../crates/mirror-storage/tests/append_selection_v2.rs),
`filesystem` means [append_v2.rs](../crates/mirror-storage/tests/append_v2.rs),
`faults` means [v2/append_tests.rs](../crates/mirror-storage/src/v2/append_tests.rs),
and `locks` means [storage_lock_v2.rs](../crates/mirror-storage/tests/storage_lock_v2.rs).
The 28 selector tests include seven independent fixture records grouped within
four vector tests; neither fixture count nor scenario iteration adds unique tests.

| Case/subcase | Actual tests | Verified component scope and remaining boundary |
|---|---|---|
| T01/T28 explicit profile boundary | filesystem: `initialization_uses_a_separate_selector_and_preserves_exact_chain_bytes`, `initialization_at_a_nonzero_height_still_starts_generation_zero`, `storage_profiles_cannot_be_silently_reinterpreted_in_either_direction`, `missing_empty_foreign_and_legacy_directories_are_never_initialized_by_open` | Only explicit absent/empty initialization; exact unchanged manifest/block bytes; independent append profile and generation zero; no in-place adoption |
| T23/T24 independent selection bytes | selector: `genesis_generation_zero_matches_independent_vector`, `four_successive_append_generations_match_independent_vectors`, `initialization_at_existing_height_starts_at_generation_zero`, `different_membership_branch_matches_independent_vector`, `exact_wrapper_fields_and_unchanged_snapshot_are_pinned` | Exact 280-byte MSU2 envelope, outer generation/domain and unchanged nested MSS2 matched to independently derived vectors |
| T20/T21 framing and integrity | selector: `every_truncation_and_trailing_bytes_are_rejected`, `wrong_magic_version_and_profile_are_rejected`, `outer_checksum_covers_every_generation_snapshot_and_checksum_byte`, `checksum_with_old_snapshot_domain_does_not_match_append_domain`, `valid_outer_checksum_does_not_hide_corrupted_embedded_checksum` | Fixed-width bounds and separate checksum domains; all truncations and byte flips are iterations within their tests, not extra cases |
| T01/T21 nested frozen semantics | selector: `nested_framing_still_requires_original_mss2_profile_one`, `nonzero_embedded_generation_remains_rejected`, `nested_count_height_and_length_limits_cannot_be_bypassed_by_wrapper`, `old_and_new_selection_decoders_do_not_silently_adopt_other_format` | Outer checks cannot weaken inner profile/count/height/byte limits or reinterpret bare MSS2 |
| T20/T28 generation and selection | selector: `generation_from_zero_through_selected_height_is_structurally_valid`, `constructor_rejects_generation_above_selected_height`, `checksum_correct_excess_generation_is_rejected_without_overflow`, `maximum_chain_height_bounds_generation_without_large_allocation`, `independent_generation_pin_is_required`; filesystem: `an_old_generation_cannot_reopen_a_successfully_appended_store` | Generation is independently selected local metadata, bounded by height; no freshness or global fork authority |
| T02/T28 independent full branch pins | selector: all five `independent_*_pin_is_required` chain-field cases, `correct_checksums_cannot_authorize_changed_selection_fields`, `old_valid_selection_is_rejected_against_later_checkpoint`; filesystem: `independent_identity_disagreement_is_rejected_before_accessing_a_missing_path`, `all_network_branch_and_generation_pins_are_required_on_reopening` | Exact C/genesis/height/tip/root and local generation are external requirements; no self-selected trust from checksums |
| T11/T17/T22/T26 complete durable transitions | filesystem: `every_append_reopens_to_exact_mixed_accounts_membership_and_budget_state`, `empty_appended_block_executes_expiry_and_keeps_accounts_unchanged` | Full original transfer/enrollment/approval/withdrawal and empty-block execution; exact whole-chain, ledger and resource equality on every reopened generation |
| T18/T25 actual membership fork | filesystem: `same_height_account_identical_membership_fork_does_not_satisfy_selection`; selector: `valid_same_height_membership_fork_does_not_match_selected_branch` | A genuine membership-divergent fork with identical account balances cannot satisfy the chosen checkpoint; no durable reorganization is performed |
| T19/T27 rejection before publication | filesystem: `rejected_height_parent_and_time_do_not_change_files_or_fence_the_writer`, `valid_work_with_a_false_root_cannot_publish_partial_mixed_execution`, `repeated_already_selected_block_is_rejected_without_mutation` | Invalid complete candidates preserve bytes/metadata and full memory; valid work cannot excuse a wrong ledger root; writer remains usable for another valid candidate |
| T27/T28 full replay on open | filesystem: `invalid_intermediate_ledger_root_is_detected_by_complete_replay`, `tampered_manifest_and_trailing_block_bytes_are_rejected_without_repair`, `missing_selected_block_is_not_repaired_or_replaced` | Metadata validity is insufficient; required manifest and every actual block reexecute or reject without repair |
| T29 live discrepancy fences | filesystem: `live_disk_mismatch_fences_all_state_access_and_preserves_confirmed_checkpoint`, `changed_selected_bytes_are_not_hidden_by_the_in_memory_chain` | Full disk replay and comparison precede mutation; mismatch fences getters and append, while last-confirmed memory remains explicitly distinguished from selected disk state |
| T29 incomplete artifacts and stale metadata | filesystem: `unselected_tail_pending_marker_and_unknown_files_are_retained_and_rejected`, `unpublished_selection_is_not_promoted_from_a_pending_file`, `corrupt_truncated_and_oversized_selection_never_fall_back_to_a_prior_state`, `stale_checksum_valid_selector_with_new_tail_rejects_both_checkpoints` | Strict clean layout and independent pins; no automatic promotion, longest-generation scan, fallback, truncation or deletion; MB-03C repair remains necessary |
| T20/T21 actual read budgets | filesystem: `oversized_block_file_is_bounded_before_payload_read`, `aggregate_block_metadata_is_bounded_before_any_invalid_payload_is_decoded`; selector: `exact_manifest_length_and_bytes_are_still_required` | Exact marker, manifest, per-block and aggregate limits checked before payload decoding; no budget increase or trusted resource cache |
| T30 static alias checks | filesystem: `symlinked_roots_selected_files_and_blocks_directories_are_rejected`, `hardlinked_selected_files_are_rejected_without_removing_aliases` | Unsafe static aliases reject and remain untouched; this is not hostile same-user namespace isolation |
| T30 read-only/shared ownership | filesystem: `inspection_is_read_only_and_cannot_append`, `writer_excludes_other_handles_before_and_after_append`, `shared_inspectors_exclude_writer_until_all_are_dropped` | Inspectors perform no writes/syncs and cannot append; exclusive ownership covers publication and shared readers exclude writers |
| T29/T30 complete reconciliation | filesystem: `clean_reconciliation_retains_root_ownership_and_allows_the_next_append`, `failed_reconciliation_preserves_files_and_releases_consumed_ownership`; faults: `consuming_reconciliation_retains_lock_and_allows_next_child_only_after_sync`, `stale_pin_cannot_reconcile_an_uncertain_new_selection_and_releases_ownership` | Same directory lock retained across consuming replay/sync; clean recovered writer can append again; stale pins return no usable handle and leave bytes intact |
| T29 append fault matrix | faults: `every_append_boundary_fences_and_reopen_returns_complete_old_new_or_rejection` | Eleven selected boundaries; old memory remains unchanged, getters/further append fence, and externally pinned reopen returns complete old/new clean state or explicit failure |
| T29 recovery fault matrix | faults: `every_reconciliation_sync_boundary_returns_no_handle_on_error` | Eight selected boundaries through both consuming reconciliation and writer reopen; all errors return no usable writer and preserve bytes |
| T29 partial writes and compare-before-publish | faults: `partial_block_and_partial_pending_selector_are_preserved_and_never_promoted`, `selector_changed_during_preparation_is_not_overwritten_or_accepted` | Explicit half-length artifacts remain rejected/retained; changed selected bytes are not overwritten during publication |
| T29/T30 root replacement | faults: `replaced_root_is_detected_before_any_append_write` | Accidental changed root inode rejects before append writes; no claim of hostile race-free descriptor-relative path traversal |
| T29 profile-2 initialization faults | faults: `profile_two_initialization_faults_preserve_partial_data_or_checked_complete_selection` | Nine selected init boundaries preserve partial files or permit independently pinned complete reopen; no silent repair |
| T30 real append-profile processes | locks: `append_writer_child_excludes_handles_until_drop_then_parent_can_commit`, `killed_append_writer_releases_root_for_checked_reopen_and_append`, `append_shared_child_and_local_reader_exclude_writer_until_both_drop` | Real independent child ownership, pipe-ordered Drop/kill/shared cases and successful append afterward; all five prior process cases retained |

### Fault scenarios and what they do not prove

The eleven append injection points are before block creation; after block creation,
write and file synchronization; after block-directory synchronization; after pending
selector creation, write and file synchronization; immediately before attempted
selector rename; after selector rename; and after root synchronization. Errors
fence normal state access and every further append. Before any block exists, the
old clean selection can reopen. After a complete renamed selector, the independently
selected new clean state can reopen. Intermediate residual-tail/pending states reject.
Every path compares retained bytes and complete old/new state, not merely an error.

Reconciliation injection covers validated selection, manifest sync, genesis and
child file sync, block-directory sync, selector sync, root sync and final selector
recheck. The same eight points run through both writer reopening and consuming
reconciliation. A separate successful recovery test checks that no competing
reader or writer acquires the lock at any callback, then appends another valid child.

Initialization injection covers parent sync, manifest sync, two block syncs,
block-directory sync, prepared-root sync, pending-selector sync, selector rename
and final-root sync. The partial-write test explicitly places half of the candidate
block or selector in its newly created file before returning an error. These are
deterministic application boundary/data fault scenarios, not kernel partial-write
simulation, comprehensive errno coverage, power loss, torn sectors or firmware tests.
Boundary loops and repeated release runs are not new unique passing tests.

The subprocess helper remains the same ignored entry used by real parent cases.
All eight parent ownership tests serialize against unrelated parent forks while
retaining their intended concurrent child checks. Pipe acknowledgments establish
ordering; no sleep or production retry weakens exclusion assertions. Killing a
lock holder tests ownership release, not atomic storage recovery under power loss.

### Independent bytes, example and remaining work

[reference_append_storage_v2.py](../tools/reference_append_storage_v2.py) independently
wraps the frozen chain and MSS2 fixtures after checking their prior file digests.
The new fixture SHA-256 is
`d6e597fdac8c5be081a54019149ebbe79651817848139d51bc691681b96df668`.
It covers genesis, four successive append generations, generation-zero initialization
of an already longer chain and a genuine membership fork. Expected old bytes were
not regenerated to match the Rust codec. Normal `--check` is read-only.

The new [append_v2_demo](../crates/mirror-storage/examples/append_v2_demo.rs) uses
new temporary directories and independent public synthetic fixtures. It exercises
accepted-child publication, exact reopened state and selected-generation checks;
it does not run a V2 peer daemon, process user runtime stores or issue currency.

Successful selected encoded payload is at most **19,559,132 bytes**: a 280-byte MSU2
marker, at most 2,781,636 manifest bytes and 16,777,216 complete MRB2 bytes. During
valid preparation an additional 280-byte pending marker can exist. Strict reopening
rejects residual staging before treating it as selected payload. These bounds are
not total resident memory; the unchanged 4,096-block and cumulative state-work caps
also apply. Deployment still assumes cooperative Linux locks, trusted parent/local
owner, stable root and correct atomic rename/synchronization semantics.

**At the MB-03B checkpoint, next was MB-03C:** an explicit reviewed repair procedure for residual staging. This
slice deliberately preserves and rejects those artifacts; it has not solved their
availability or automatic recovery. Canonical fork choice, durable membership-aware
reorganization and peer reconstruction follow under later MB-03 work. T29 remains
partial in that broader scope, T31–T32 remain open, and no public-mainnet, biological
uniqueness, anonymity, real-device or final human-consensus guarantee is inferred.

## 12. Historical MB-03C explicit interrupted-append recovery

**VERIFIED COMPONENT on Rust 1.98.0 with exact-source CI.** The tested implementation
is source commit `e8302052ab724f59ff4a212dc64aea9575a6d4b3`, tree
`dc9eb2c02436f0952ac54593ba8d02317e9d9314`, published with an exact local tree match.
Exact-source [push run 44748718086](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748718086)
(job `136330677456`) and [PR run 44748724632](https://github.com/Ahmed-Sobeih/mirror/actions/runs/44748724632)
both completed successfully at that source. The full 216,857-byte push log was
inspected: exact checkout SHA; workspace **707 passed / one ignored**; **three
doctests**; release core **121**, personhood **237 including the same three
doctests**, state **49**, consensus plus chain **80**, and storage **194 / one
ignored**; all **eight references**, **nine examples** and the audit step passed.
These are reruns of the same assertions, not additional unique tests. These source
runs do not establish a later documentation head's CI result.
[CURRENT_STATUS.md](../CURRENT_STATUS.md) owns current publication evidence; this
matrix maps the new assertions.

The [recovery contract](STORAGE_RECOVERY_V2.md) defines the explicit request and
supported residual states. No format, profile, hash domain or prior fixture changes.
The request owns independent adjacent parent/candidate checkpoints and the exact
child. Its construction is structural; both actions still replay the parent and
validate the child through actual ChainV2 acceptance. Empty/common prefixes do not
prove provenance: the trusted external request authorizes the two named paths.
Unknown, mismatched and unsupported artifacts remain preserved and rejected.

There are **49 new unique test functions: 39 integration, nine private fault and
one additional real-process ownership case**. Local execution passed **707 workspace
tests (658 retained + 49 additions)**, the same **three documentation tests** and
one intentionally ignored subprocess helper. Release gates passed **194 storage**,
**121 core**, **234 personhood** plus the same three documentation tests, **49
state**, **23 consensus** and **57 chain** tests. All **eight independent reference
checks** and **nine examples** passed. Release reruns, doctest reruns and per-test
parameterized scenarios are not additional unique tests. A byte comparison also
confirmed **99 retained source, dependency, reference and fixture files unchanged**.

In the table, `integration` means
[recovery_v2.rs](../crates/mirror-storage/tests/recovery_v2.rs), `faults` means
[append/recovery_tests.rs](../crates/mirror-storage/src/v2/append/recovery_tests.rs),
and `process` means
[storage_lock_v2.rs](../crates/mirror-storage/tests/storage_lock_v2.rs).

| Requirement | Actual named test functions | Assertion and precise boundary |
|---|---|---|
| T01/T28 explicit recovery boundary | integration: `recovery_never_initializes_or_adopts_missing_empty_legacy_or_immutable_stores`, `ordinary_open_and_reconcile_remain_strict_until_explicit_recovery_is_requested` | Recovery is separate from initialization, migration and normal strict opening; immutable profile 1 is never adopted |
| T02/T28 independent request authority | integration: `each_parent_and_candidate_pin_is_checked_before_mutation`, `identity_disagreement_cannot_create_or_initialize_a_missing_root`, `request_relationship_rejects_skipped_height_or_generation_and_wrong_child` | Independent complete endpoints and exact child/header relationships; checked adjacency/bounds; bad selection does not mutate or create a store |
| T19/T27 full candidate acceptance | integration: `full_candidate_validation_rejects_valid_work_with_a_false_ledger_root`, `full_candidate_validation_rejects_incorrect_work_target_before_any_cleanup` | Both actions require actual ChainV2 execution/acceptance before mutation; valid work alone cannot authorize cleanup |
| T29 completing absent/partial/full child | integration: `complete_can_materialize_an_independently_authorized_missing_child`, `complete_finishes_empty_and_partial_child_prefixes_without_changing_the_prefix`, `complete_publishes_a_full_child_without_a_pending_selector`, `complete_finishes_empty_partial_and_full_pending_selector_prefixes` | Only exact canonical missing suffixes are written, with complete child required before pending selector; original accepted files preserved |
| T29 retaining the selected parent | integration: `retaining_a_clean_parent_is_idempotent_and_does_not_advance_generation`, `retain_parent_removes_only_the_named_matching_child_prefix`, `retain_parent_cleans_matching_pending_then_child_and_can_be_retried` | Explicit cleanup removes only independently authorized unselected staging, pending before tail; parent bytes/state and generation remain unchanged |
| T28/T29 already-selected candidate | integration: `repeating_complete_after_new_selection_resynchronizes_without_a_second_advance`, `retain_parent_never_rolls_back_an_already_selected_candidate` | Complete is idempotent after selection; RetainParent cannot silently roll back selected history |
| T11/T17/T22 complete ledger recovery | integration: `recovered_child_can_append_and_reopen_with_all_state_and_resource_counters`, `recovered_empty_block_applies_expiry_without_changing_account_bytes` | Whole ChainV2, account/membership bytes, permanent history and resource counters agree; recovered writer accepts a subsequent real child |
| T18/T25 genuine membership branch | integration: `exact_independent_request_selects_membership_history_even_when_accounts_match`, `a_complete_membership_fork_is_not_overwritten_or_discarded_by_another_request` | Actual distinct membership histories with identical accounts remain distinct; other valid branch bytes are not repair authority |
| T28 generation and stale intent | integration: `generation_tracks_append_count_independently_of_initial_height`, `a_stale_request_cannot_recover_after_a_later_successful_append` | Local publication generation is separate from initial height; a stale request cannot repair a later selection |
| T29 exact-prefix and supported-shape checks | integration: `nonprefix_child_and_pending_files_are_preserved_for_both_actions`, `extra_trailing_bytes_do_not_become_authorized_prefix_data`, `pending_selector_with_missing_or_partial_child_is_unexplained_and_rejected`, `new_selection_with_pending_or_incomplete_child_is_not_silently_repaired` | Mismatched bytes and unsupported orderings reject with files intact; no truncation, best-effort repair or hole filling in selected history |
| T21/T28 unknown/corrupt history | integration: `unknown_root_names_extra_heights_and_noncanonical_block_names_are_preserved`, `missing_or_corrupt_selected_history_cannot_be_hidden_by_good_staging`, `missing_partial_corrupt_or_foreign_selected_marker_is_preserved_and_rejected` | Unknown names, extra history, damaged parent/selection and foreign formats reject; pending metadata never substitutes for selected-history authority |
| T20/T21 bounded intended bytes and metadata | integration: `sparse_oversized_residual_files_are_rejected_without_truncation_or_completion`, `combined_parent_and_candidate_size_is_checked_before_invalid_block_payloads_are_read`, `a_checksum_correct_selected_byte_total_must_match_the_replayed_history` | Limits apply to full intended candidate and aggregate history before block payload access; checksum-correct selected byte totals must equal re-derived metadata |
| T30 safe managed files | integration: `symlinked_roots_directories_selected_files_and_residuals_are_never_followed`, `hardlinked_selected_files_and_residuals_keep_all_aliases_on_rejection`, `directory_in_place_of_a_managed_file_is_not_deleted_or_replaced` | Reject unsafe types/links before completion or deletion; preserve aliases and unexpected directories |
| T29/T30 ownership and consuming API | integration: `a_live_writer_or_shared_inspectors_exclude_static_recovery`, `consuming_recovery_keeps_the_stable_root_and_returns_a_writer`, `consuming_recovery_rejects_read_only_or_unrelated_live_checkpoint_and_releases_lock`, `consuming_recovery_can_retain_parent_and_then_accept_the_original_child` | Same root/held exclusive descriptor through consuming recovery; read-only or unrelated handles reject; error returns no writer; accepted parent can append afterward |
| T29 completion interruptions | faults: `completion_retries_after_each_boundary_from_absent_partial_and_pending_shapes` | 28 boundaries from each of three starting shapes, 84 iterations; parent files retained, uncertainty category correct, same request recovers and accepts another child |
| T29 retention and already-selected interruptions | faults: `retain_parent_retries_after_each_cleanup_and_resynchronization_boundary`, `already_selected_candidate_sync_failures_retry_without_another_generation` | 21 retention boundaries and 16 already-selected synchronization boundaries; repeated independent retry reconstructs exact parent/candidate without a second generation advance |
| T29/T30 original failures and same-lock recovery | faults: `every_original_append_failure_recovers_without_releasing_live_ownership` | Both actions after 11 original append boundaries; readers/writers remain excluded at each recovery callback; RetainParent rejects after candidate publication |
| T29 observed changes before mutation/publication | faults: `changed_root_marker_or_unknown_layout_at_validation_is_not_mutated`, `changed_prefixes_are_never_overwritten_or_deleted_after_validation`, `postwrite_or_prepublication_changes_cannot_be_selected` | Changed root/marker/layout and changed intended bytes return no writer; altered bytes remain intact and the candidate is not selected |
| T21/T30 opened alias and read allowance | faults: `descriptor_alias_created_after_open_blocks_prefix_completion`, `recovery_read_allowance_charges_sentinels_before_touching_payload` | A new hardlink after opening rejects before extension; read meter charges payload allowance plus sentinel before payload access and cannot reset on exhaustion |
| T30 real recovered-writer ownership | process: `recovered_child_process_keeps_exclusive_ownership_until_drop_or_termination` | Independent child holds the recovered writer; Drop and termination release descriptor ownership, followed by checked reopen and another accepted append; existing fork isolation retained |

### Failure scenarios versus test counts

The completion test iterates **28 boundaries × three initial layouts = 84 scenarios**:
no residual child, a partial authorized child, and a complete child with partial
pending selector. Boundaries cover validation and parent prerequisite synchronization;
opening, completing and synchronizing child/selector files; block-directory sync;
publication attempt, selector rename and root sync; and final clean replay plus
all selected-file/directory synchronization. Every iteration checks unchanged parent
files, the error's publication uncertainty, exact recovered state and a subsequent
accepted append.

Retention iterates **21 boundaries**, including pending deletion/root sync before
child deletion/block-directory sync, then final strict replay/resynchronization.
The already-selected path iterates **16 boundaries** and cannot increment the
candidate generation again. Original failed appends are exercised at **11 boundaries
with both actions**; the two postpublication RetainParent cases must reject. These
iterations are scenarios inside four private test functions, not hundreds of unique
passing tests or proof that every possible operating-system failure was simulated.

Other private tests deliberately change the root, selected marker or layout after
validation; alter a prefix before extension/deletion; alter prepared bytes after
writing or immediately before publication; introduce a hardlink after opening; and
exhaust the read allowance before a bounded payload read. No rejected byte mismatch
is overwritten, no unknown file is deleted and no partial ledger handle is returned.
The actual child-process test retains the existing parent-test mutex isolation;
it uses ordered pipes and Drop/termination, not timing sleeps or production retries.

### Bounds, preserved bytes and residual scope

Distinct recognized encoded input is at most **19,559,412 bytes**: manifest plus
complete parent/candidate MRB2 history plus two 280-byte selectors. The full intended
child counts even when only a prefix exists. The separately enforced repeated-read
allowance is **156,508,096 charged bytes**, including sentinel reads and conservative
charges for final strict loading and selector comparisons. These are not total
resident-memory, actual-I/O, write-volume or metadata-operation limits. The unchanged
4,096-block, 16-MiB history and 1,048,576 cumulative state/configuration-item caps
remain enforced; no cache restore, pruning or capacity reset is introduced.

Existing independent MRB2/MSU2 reference bytes supply expected blocks and selectors;
there is no new serialized format to redefine. The
[recovery_v2_demo](../crates/mirror-storage/examples/recovery_v2_demo.rs) uses only
its own synthetic temporary directories and public fixtures. It demonstrates
explicit completion and retention, clean reopening and continued accepted append,
not a V2 daemon, real user database or biometric process.

Normal open/inspection/clean reconciliation remains strict. Recovery refuses
unknown files, nonprefix/corrupt bytes, missing/corrupt selected markers or parent
history, pending data without a complete child, multi-child leftovers and interrupted
initialization. A selected candidate cannot be rolled back by RetainParent. The
caller must retain independent intent or explicitly reestablish trusted authority;
a checksum or matching common prefix cannot supply it.

Injected errors and fixture-created prefixes do not simulate torn kernel writes,
physical power loss, firmware behavior or every syscall failure. Local cooperative
Linux ownership, trusted owners/parents and correct sync/rename behavior remain
assumptions. WSL/DrvFS, native Windows and network filesystems are not certified.

**At the MB-03C checkpoint, MB-03 remained IN PROGRESS and MB-03D was next:**
canonical development fork-choice specification and implementation, before durable
membership-aware branch replacement and peer reconstruction. T29 remains partial
in that broader scope, T31–T32 remain open, and Gate A/B remain unmet. No new
monetary rule, founder/marketing entitlement, real biometric intake, biological
uniqueness, anonymity or human-finality guarantee follows from this component.

## 13. Historical MB-03D canonical development fork choice

**All required local acceptance gates passed.** Source:
`b9a2e870996fcafe1128dcb5cfdfb6b01b158ded`; source tree: `dd0a3fc29f429dcd4999261801c2bc7897d6aace`. The implementation follows
[FORK_CHOICE_V2.md](FORK_CHOICE_V2.md), with no new format, hash domain, dependency
or changed independent fixture. Rust 1.98.0 with locked dependencies passed:

- **735 workspace runtime tests:** 707 retained plus 28 new integration functions.
- **Six documentation tests:** the three retained compile-fail examples and three
  new ownership/type examples passed together.
- **Release suites:** core **121**; personhood **234 plus three doctests**; state
  **49**; consensus **23**; chain **85 plus three doctests**; storage **194**.
- All **eight independent reference checks** and **ten examples**, including the
  new synthetic `fork_choice_v2_demo`.
- Preservation checks: **105 retained source, dependency, reference and fixture
  files are byte-identical**.

One subprocess-only helper remains intentionally ignored in ordinary enumeration;
its parent tests invoke it. Release repeats and individual example assertions are
not additional unique tests. Earlier counts above remain historical. Local results
do not establish remote CI success; exact-source and documentation-head evidence
are recorded separately:
Published-source [push CI 45703714223](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703714223)
and [PR CI 45703726306](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45703726306)
both passed. Full push-job `139520118282` logs confirmed the exact implementation
checkout, **735 workspace tests, six compile-fail doctests, all required release
suites, eight independent references, ten examples and the allocation/runtime-data
audit**. The single intentionally ignored subprocess helper is invoked by parent
tests and is not another pass. Release reruns are not additional unique tests.
MB-03D is **VERIFIED_COMPONENT — ACCEPTED-BRANCH COMPARISON**. Subsequent
documentation-head CI is checked separately and recorded on PR #1.

The integration cases are in
[fork_choice_v2.rs](../crates/mirror-chain/tests/fork_choice_v2.rs).
The common assertion helpers check both inputs remain unchanged, every returned
identity/score/checkpoint/relation, the exact common-ancestor block and root, and
pointer identity of the selected complete branch. Rejection helpers compare both
entire inputs before and after the call. The table names all **28 new runtime
functions exactly once**; loops, release repeats and example assertions are not
additional unique tests.

| Requirement | Actual named integration test functions | Assertion and precise boundary |
|---|---|---|
| T02/T28 identical accepted history | `identical_genesis_has_zero_score_and_keeps_current`, `independently_replayed_identical_history_uses_its_tip_as_common_ancestor`, `same_chain_reference_is_an_identical_comparison` | Genesis scores zero; identical reconstructed and aliased inputs keep current with the shared accepted tip/root as ancestor |
| T17/T26 proper prefixes and extensions | `one_child_extension_is_preferred_without_mutating_current`, `multi_child_extension_preserves_complete_candidate_ledger`, `authentic_shorter_prefix_keeps_current`, `genesis_candidate_cannot_replace_a_nonempty_history` | Exact proper-prefix relationships differ from divergence; longer accepted candidate wins with its complete ledger/counters; shorter or genesis-only candidate cannot displace current |
| T18/T25 genuine divergent preference | `longer_divergent_membership_branch_is_preferred`, `shorter_divergent_membership_branch_keeps_current` | Fully accepted membership forks are compared by normalized fixed-target score, not by agreement with current state |
| T24/T28 ties and hash magnitude | `equal_score_divergence_keeps_current_even_when_candidate_hash_is_lower`, `reversed_equal_score_divergence_also_keeps_its_current_branch` | Equal-score branches keep their ordered current input in both directions; a lower realized hash gives no extra weight or global total order |
| T17/T18/T25 complete membership and counters | `identical_accounts_do_not_make_different_membership_branches_identical`, `preferred_fork_retains_its_own_membership_approvals_and_population_counters` | Equal account bytes do not hide distinct membership; chosen branch retains its own statuses, approvals, activation/population peaks and resource counters rather than grafting orphaned effects |
| T11/T17/T22 empty-block execution | `empty_blocks_count_as_work_and_expiry_stays_with_the_selected_branch` | Accepted empty blocks increase normalized score while carrying the chosen branch's executed expiry, permanent records and complete ledger |
| T09/T24 actual common ancestry | `divergence_at_first_child_has_genesis_as_common_ancestor`, `common_ancestor_is_latest_matching_block_in_a_deeper_history` | Actual accepted hashes locate genesis or a later last shared block; returned checkpoint contains exact C/genesis/height/hash/root |
| T10/T28 timestamp neutrality and terminal tip | `timestamp_magnitude_does_not_break_a_work_tie` | Later and terminal timestamps do not break equal-score ties; valid timestamp magnitude cannot add work or promise a successor |
| T02/T28 independent network selection | `wrong_independent_configuration_rejects_even_identical_inputs`, `wrong_independent_genesis_rejects_even_identical_inputs` | Agreement between both inputs cannot bypass the independently selected configuration or exact genesis |
| T02/T28 candidate identity | `candidate_with_same_config_but_other_valid_genesis_is_rejected`, `candidate_with_different_valid_configuration_is_rejected` | An otherwise accepted candidate under another valid genesis/configuration cannot be ranked against the pinned network |
| T02/T28 current identity and precedence | `current_with_other_valid_genesis_is_rejected_before_candidate`, `current_with_different_configuration_is_rejected_before_candidate`, `configuration_error_precedes_genesis_error_for_the_same_side`, `current_genesis_error_precedes_candidate_configuration_error` | Error order is current C, current genesis, candidate C, candidate genesis; no shorter/longer bypass |
| T02/T28 explicit alternate network | `comparison_can_explicitly_select_another_accepted_network_identity` | A caller can explicitly choose another independent accepted identity; it is not adopted from a peer or guessed by comparison |
| T19/T27 invalid block exclusion | `invalid_peer_block_cannot_increase_the_comparable_accepted_score` | A valid-work block with a false ledger root fails actual acceptance and cannot increase the branch score or enter comparison as an accepted child |
| T20/T21 finite profile boundary | `maximum_accepted_history_compares_without_score_overflow_or_extra_blocks` | Two real 4,096-block branches compare at score 4,095 with the correct ancestor; no extra block can bypass the existing cap and no work-sum arithmetic is introduced |

### Compile-time ownership and type boundary

The [production API documentation](../crates/mirror-chain/src/v2/fork_choice.rs)
adds three compile-fail examples. `cannot_change_current` and
`cannot_change_candidate` demonstrate that neither input can be mutably appended
while the comparison remains in use. `cannot_compare_unaccepted_block` shows that
a raw `BlockV2` is not an accepted `ChainV2` argument. These are three distinct
new doctests, not three runtime integration functions. Existing compile-fail
membership/ledger cases remain required and unchanged.

### Example, scoring proof and remaining scope

The [fork_choice_v2_demo](../crates/mirror-chain/examples/fork_choice_v2_demo.rs)
uses the frozen synthetic block histories to show a real membership fork, keep-
current behavior on an equal score, preference for a longer accepted branch and
complete-state selection. It creates no store, publishes no selection, processes
no user data and enables no currency issuance.

All accepted blocks under the same independent C/genesis use the identical literal
configured target. Constant positive per-block work makes height comparison an
exact ordering normalization, not a measurement of actual attempts. The shared
genesis term cancels. No score input, large-integer multiplication or accumulation
exists; the admitted profile bounds height to `0..=4095`. Common ancestry compares
at most 4,096 pairs/8,192 fixed header hashes with constant additional state and
no branch cloning or new heap collection. Existing per-input budgets remain in force.

The result privately borrows both complete accepted inputs. Public choices and
copied checkpoints are diagnostic values that can become stale; they are not
serialized publication authority. No test here establishes durable branch
replacement, store mutation, global latestness, peer synchronization, partition
liveness or final human consensus. Equal-score divergent peers can retain different
branches, including unresolved ties at a finite/terminal extension boundary.

**At the MB-03D checkpoint, MB-03 remained IN PROGRESS and MB-03E was next:**
durable membership-aware branch replacement with independently rechecked
current/candidate endpoints, stable exclusive ownership
and reviewed publication/interruption/retry rules. Fresh-peer reconstruction
follows. T29 remains partial, T31–T32 remain open, D-01/D-04 remain unresolved,
and Gate A/B remain unmet. No policy decision, founder/marketing allocation,
real biometric processing or anonymity claim was added.

## 14. Historical MB-03E1 replacement-selection metadata and preparation

The [contract](STORAGE_REPLACEMENT_V2.md) covers a new MSR2 profile-3 record and
pure checked preparation only. The focused suites passed **31 codec tests** in
`crates/mirror-storage/tests/replacement_selection_v2.rs` and **21 preparation
tests** in `crates/mirror-storage/tests/replacement_plan_v2.rs`. All **nine independent
reference scripts** passed, including the new stdlib Python implementation and its
nine fixed cases. These are test functions and reference checks, respectively;
vector cases and release reruns are not additional unique runtime tests.

| Boundary | Implemented assertion / representative coverage |
|---|---|
| Frozen encoding and independent expected bytes | `exact_wrapper_fields_and_unchanged_snapshot_are_pinned`; genesis, nonzero generations, existing-height initialization, divergent membership and multi-height replacement match independent vectors. |
| Framing, checksum and nested bounds | Every truncation/trailing byte rejects; both u16 fields, distinct domain, nested profile/generation/checksum and count/height/length limits remain enforced. Checksummed semantic corruption does not bypass these checks. |
| Independent selected endpoint | Every network/height/tip/root/generation pin is checked; old valid selections and valid same-height membership forks reject against another selected checkpoint. |
| Complete current metadata | `checksum_correct_metadata_must_match_complete_accepted_current` and `metadata_mismatch_is_not_hidden_by_nonpreferred_candidate` reject changed manifest or block-byte metadata even when endpoint fields and checksums agree. |
| Preferred whole candidate | `preferred_divergent_branch_keeps_complete_candidate_and_independent_pins` and `equal_account_bytes_cannot_graft_membership_from_current_branch` retain the exact candidate history and membership/counters together. |
| Generation and rejected choices | A multi-block extension advances once; existing-height initialization starts at zero; identical, tied and shorter histories produce no plan. Generation cannot exceed accepted height. |
| Trust-boundary order and immutable inputs | Foreign network identity rejects before current-pin/score checks; the actual current endpoint precedes selector-generation checking. Two compile-fail examples in `replacement_plan.rs` cover mutation of each borrowed chain. |

The synthetic `replacement_plan_v2_demo` prepares a real divergent branch and
round-trips its metadata without opening or changing a store. The plan is not a
durable authorization token; it performs no disk publication, replay, recovery or
global freshness discovery. Existing block/storage formats, budgets and prior
reference fixtures are retained. No production enrollment or monetary policy is
selected by this addition.

This increment's full-suite, release, example and exact-source CI evidence remains
in [MASTER_PLAN.md](../MASTER_PLAN.md), section 22. Historical counts in sections
3–13 remain their own checkpoints. **At the MB-03E1 checkpoint, next was MB-03E2:**
bounded replacement layout, stable ownership, synchronized publication and explicit
interruption/recovery rules. Section 15 records the subsequent E2a implementation;
fresh-peer reconstruction and Gate A/B remain unfinished.

## 15. MB-03E2a durable replacement and strict clean-state reconciliation

**Historical E2a checkpoint.** Section 16 records the later explicit recovery scope.

The [replacement-store contract](STORAGE_REPLACEMENT_STORE_V2.md) defines the new
`ReplacementStoreV2` lifecycle around unchanged MSR2 profile-3 metadata. Source is
[`replacement_store.rs`](../crates/mirror-storage/src/v2/replacement_store.rs).
This section maps **49 new test functions**, not 49 claims of complete fault or
security coverage. Current actual results, exact source/tree and publication/CI
evidence belong to [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
[MASTER_PLAN.md](../MASTER_PLAN.md), section 23. Do not infer newer-source success
from any historical totals above.

The new functions comprise:

- **32 filesystem integration tests** in
  [`replacement_store_v2.rs`](../crates/mirror-storage/tests/replacement_store_v2.rs).
- **14 private fault/read-budget tests** in
  [`replacement_store_tests.rs`](../crates/mirror-storage/src/v2/replacement_store_tests.rs).
- **Three process ownership tests** added to
  [`storage_lock_v2.rs`](../crates/mirror-storage/tests/storage_lock_v2.rs), using the
  existing explicitly invoked subprocess helper. The helper is not another pass.

| Boundary | Representative implemented test functions | Scope of assertion |
|---|---|---|
| Exact new layout and bytes | `initialization_persists_exact_msr2_manifest_and_complete_block_bytes`, `explicit_initialization_accepts_an_existing_empty_directory`, `initialization_never_overwrites_nonempty_or_existing_selected_storage` | One pinned generation directory and unchanged exact MSR2/MNM2/MRB2 bytes; explicit absent/empty initialization only |
| Whole divergent history | `whole_divergent_history_replaces_old_branch_and_reopens_exactly`, `equal_account_bytes_cannot_keep_losing_membership_or_population_history` | Complete chain equality after reopen, including membership records, retained approvals, activations, high-water and resource counters; balances alone do not select state |
| Publication generation and retention | `multi_block_extension_is_one_publication_generation`, `successive_replacements_keep_only_one_complete_selected_generation`, `successful_replacement_preserves_the_locked_root_inode` | One generation advance per preferred publication; exact old unselected copy retired; selected complete ancestry retained and root inode unchanged |
| Preference and independent authority | `identical_chain_is_not_a_new_generation`, `equal_height_divergent_candidate_keeps_current_without_writes`, `shorter_candidates_cannot_roll_back_selected_storage`, `foreign_genesis_and_wrong_network_identity_reject_without_fencing`, `stale_caller_endpoint_and_generation_cannot_authorize_replacement` | Ties, short/identical candidates and wrong caller pins reject before disk mutation; valid current handle remains usable for these preflight errors |
| Pinned reconstruction and canonical metadata | `reopening_requires_every_independent_endpoint_and_generation_pin`, `checksum_correct_but_false_manifest_or_block_metadata_is_rejected`, `full_replay_rejects_changed_block_despite_intact_selected_marker` | All pins and whole canonical metadata checked; full selected history reexecuted; no checksum or endpoint-only authority |
| Read-only and consuming lifecycle | `inspection_is_shared_and_cannot_replace_or_reconcile`, `clean_consuming_reconciliation_reexecutes_whole_selected_state`, `failed_consuming_reconciliation_returns_no_handle_and_keeps_artifacts` | Shared inspection performs no application writes; consuming reconcile keeps ownership and returns a complete synchronized writer or no handle |
| Fencing and retained uncertainty | `selected_disk_discrepancy_fences_access_and_prevents_further_writes`, `every_replacement_boundary_fences_without_exposing_mixed_state`, `failed_after_final_sync_can_reconcile_only_the_complete_new_chain` | Disk-phase failure blocks normal state access and further writes; memory remains the old confirmed whole state while disk may already select new |
| Initialization and reconciliation faults | `every_initialization_boundary_preserves_partial_data_and_returns_no_writer`, `every_clean_reconciliation_boundary_retains_ownership_and_never_returns_failed_handle` | Deterministic hook failures preserve files, do not expose partial state and retain the lock until the consuming operation ends |
| Candidate integrity before cleanup | `candidate_corruption_before_publish_preserves_the_old_selection_and_data`, `selected_candidate_corruption_after_publish_prevents_old_history_cleanup` | Invalid prepared candidate prevents publication; invalid selected candidate prevents any deletion of old history |
| Entire old-copy integrity before deletion | `unexpected_old_file_after_publication_prevents_any_cleanup_deletion`, `all_old_bytes_are_checked_before_any_earlier_old_file_is_deleted` | Foreign old entries, a corrupt late old block or corrupt old manifest stop cleanup before deleting any earlier valid old file |
| Unsupported residue stays untouched | `pending_selector_is_preserved_even_when_empty_or_exact_candidate_bytes`, `higher_generation_directory_does_not_select_or_authorize_cleanup`, `unexpected_generation_or_root_entries_reject_before_any_cleanup`, `partial_candidate_block_is_preserved_and_cannot_be_silently_reconciled`, `leftover_old_history_after_publication_cannot_roll_selection_back` | Ordinary open/reconcile never resumes staging, promotes metadata, rolls back selected new history or deletes unexplained data |
| Exact file layout and safe paths | Missing/corrupt marker, missing/extra/noncanonical block, generation alias, static symlink and hardlink tests | Canonical names, regular-file identities and selected completeness required; all rejection leaves observed bytes intact |
| Input and repeated-read bounds | `oversized_sparse_file_is_rejected_by_length_without_reading_or_truncating_it`, `bounded_root_enumeration_rejects_excess_entries_and_preserves_every_one`, `repeated_payload_reads_share_one_budget_and_charge_growth_sentinels`, `overflowing_or_oversized_read_allowance_rejects_before_payload_open` | Oversized length rejects before payload reads; finite enumeration; shared read allowance charges sentinel bytes and rejects exhaustion/overflow before opening payload |
| Existing profiles | `prior_storage_profiles_are_never_implicitly_migrated_in_either_direction` | Replacement, immutable and append layouts remain distinct; no implicit conversion or old-data rewrite |
| Real process ownership | `replacement_writer_child_excludes_handles_until_drop_then_parent_can_replace`, `replacement_shared_child_and_local_reader_exclude_writer_until_both_drop`, `replaced_and_reconciled_child_retains_same_root_lock_until_drop_or_termination` | Real cooperating child processes enforce exclusive/shared lifetimes; successful replacement/reconciliation keeps the stable root owned until descriptor release or process termination |

The synthetic `replacement_store_v2_demo` initializes a genuine divergent accepted
history, publishes the preferred complete branch, inspects exact selected metadata
and reopens the full state from disk. It uses public synthetic fixtures and newly
owned temporary paths. Its assertions and reruns are not extra unique test functions.
No new serialized format or reference vector is added; all nine independently
frozen reference checks remain required.

The store admits at most two complete generation copies and two selectors during
one replacement: **39,118,264 encoded bytes** at existing per-history caps. Its
single **313,011,680-byte read allowance** charges all payload reads and sentinels.
A successful replacement performs five separately bounded full chain replays and
four further exact payload passes, with a conservative **176,068,508-byte** read
upper bound. Directory enumeration and chain execution/state limits are separate.
These are not RSS limits, physical power-loss trials or measured availability.

**At the E2a checkpoint T29 remained partial, with MB-03E2b next.** E2a's ordinary
operations rejected partial candidate staging and old-copy cleanup residue; no
completion/retention API for those states existed then. The required recovery
needed independent full checkpoints plus exact accepted **old and candidate
histories**: the old endpoint alone cannot authenticate a partly removed old branch.
Section 16 records the subsequent explicit recovery implementation. No selected new generation
may be rolled back by retaining old. No arbitrary cleanup, global freshness,
peer reconstruction, final human consensus or persistent V2 node is inferred.

All ownership claims remain scoped to cooperating Linux processes, trusted local
owners/parents and documented filesystem behavior. Hostile same-user races,
unverified WSL/DrvFS/native Windows/network filesystems and physical power-loss
guarantees remain outside these checks. User policy and old format bytes remain
unchanged; no privileged founder/marketing funds, mint/reward formula, real
biometrics, private matching or anonymity mechanism is added. Gate A/B remain unmet.

## 16. MB-03E2b explicit interrupted whole-branch replacement recovery

The [replacement recovery contract](STORAGE_REPLACEMENT_RECOVERY_V2.md) specifies
the separate `Complete`/`RetainOld` API, independently retained authority, finite
residual shapes and synchronization ordering. Source is
[`replacement_recovery.rs`](../crates/mirror-storage/src/v2/replacement_recovery.rs).
Normal open/inspect/reconcile retain their clean-only behavior. No existing wire
encoding, profile, reference fixture, dependency or monetary rule is repurposed.

This section maps **55 present test functions**, comprising:

- **38 public integration cases** in
  [`replacement_recovery_v2.rs`](../crates/mirror-storage/tests/replacement_recovery_v2.rs).
- **15 private failure/read-budget cases** in
  [`replacement_recovery_tests.rs`](../crates/mirror-storage/src/v2/replacement_recovery_tests.rs).
- **Two independent-process cases** added to
  [`storage_lock_v2.rs`](../crates/mirror-storage/tests/storage_lock_v2.rs).

All 55 functions passed in the full **891-test** local runtime suite; the required
release suites, nine references and thirteen examples also passed. Exact source/tree
and published CI evidence belong to [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
[MASTER_PLAN.md](../MASTER_PLAN.md). The **eight retained compile-fail doctests**
remain unchanged, with no new documentation-test count. The existing subprocess-only
helper remains intentionally ignored during ordinary enumeration and explicitly
invoked by its parent tests; it is not another unique pass. Scenario iterations,
release reruns and executable-example assertions are not extra test functions.

### Rule and source mapping

| Boundary | Implemented test functions | Scope of assertion |
|---|---|---|
| T02/T28 retained request authority | `request_retains_both_accepted_histories_and_external_checkpoints`, `full_accepted_histories_remain_available_after_recovery_failure` | Request borrows both exact accepted histories and their independent complete checkpoints; failure cannot consume caller recovery references |
| T02/T28 invalid authority and preference | `wrong_identity_is_rejected_before_any_staging_changes`, `request_rejects_wrong_history_endpoints_and_generation_transitions`, `request_rejects_shorter_tied_and_foreign_genesis_histories`, `valid_request_with_wrong_local_generation_cannot_adopt_other_residue` | Network, full endpoints, adjacent generations and strict existing preference are checked; a valid request for different local generations cannot authorize observed files |
| T28/T29 clean idempotence | `clean_old_can_complete_and_complete_can_be_repeated`, `clean_old_retention_is_idempotent_and_does_not_publish`, `selected_candidate_can_never_be_rolled_back_to_old` | Complete and RetainOld produce their exact allowed clean selections; repeat completion does not advance generation again and selected candidate never rolls back |
| T29 arbitrary candidate staging | `sparse_staging_completes_missing_and_partial_candidate_files`, `sparse_staging_can_be_removed_while_retaining_the_exact_old_history`, `candidate_directory_only_and_manifest_only_shapes_support_both_actions` | Missing directories, arbitrary surviving block subsets, absent manifests and canonical empty/proper/full prefixes are explicitly supported while old remains selected |
| T29 existing descriptor and pending-prefix handling | `manifest_and_block_prefixes_preserve_existing_file_inodes_during_completion`, `pending_selector_prefixes_require_complete_candidate_and_support_both_actions` | Existing prefix files are extended without replacement/truncation; pending prefixes from empty through complete require an exact complete candidate |
| T29 arbitrary old cleanup residue | `candidate_selected_cleanup_accepts_arbitrary_complete_old_file_subsets`, `candidate_selected_cleanup_accepts_manifest_only_empty_and_absent_old_generation`, `old_cleanup_cannot_skip_manifest_ordering_while_block_directory_survives` | Remaining old blocks may be an arbitrary subset of exact full bytes; manifest-only/empty/absent old generations are recognized, but blocks-present/manifest-missing rejects |
| T17/T18/T25 complete membership state | `same_accounts_do_not_preserve_orphaned_memberships_or_population_history` | A real fork with equal account bytes and different membership state recovers/reopens the exact selected chain, records, approvals, population and resource counters; no orphaned state is grafted |
| T26/T29 ordinary lifecycle stays strict | `recovery_keeps_ordinary_open_inspect_and_reconcile_strict`, `consuming_recovery_accepts_a_fenced_writer_and_retains_the_root_inode` | Normal APIs reject residue; explicit consuming recovery accepts a relevant fenced writer, holds the same root inode and returns the complete selected state |
| T29/T30 inappropriate or failed handles | `consuming_recovery_rejects_shared_inspection_and_unrelated_memory_checkpoint`, `rejected_consuming_recovery_returns_no_handle_and_preserves_unknown_residue`, `consuming_recovery_failure_returns_no_writer_and_can_retry_under_new_ownership` | Read-only/unrelated handles reject; consuming errors expose no writer, preserve the inspected artifacts and release ownership only when the consumed operation ends |
| T28 exact selected authority | `missing_truncated_corrupt_and_unpinned_selection_markers_are_preserved`, `valid_checksum_with_wrong_canonical_metadata_is_not_selection_authority`, `selected_old_requires_its_complete_original_history_before_either_action` | Complete canonical MSR2 and complete selected history are mandatory; public checksums, endpoint-only agreement or good candidate staging cannot repair missing selected authority |
| T29 unsupported candidate/pending combinations | `candidate_selected_with_pending_or_incomplete_selected_history_is_preserved`, `any_pending_selector_with_incomplete_candidate_is_rejected_unchanged` | Even an empty pending marker requires complete candidate prerequisites; selected candidate must be complete with no pending marker |
| T21/T29 unrecognized content | `foreign_or_oversized_candidate_prefixes_are_preserved`, `foreign_or_oversized_pending_selector_is_preserved`, `every_surviving_old_file_is_checked_before_any_cleanup_deletion`, `unknown_root_generation_and_block_entries_are_rejected_before_mutation` | Nonprefix/overlong bytes, corrupt or partial old files and unknown paths reject before cleanup; a corrupt late file prevents deletion of earlier valid files |
| T21/T30 unsafe aliases | `candidate_and_old_generation_symlinks_are_never_followed`, `recovery_rejects_symlink_and_hardlinked_payloads_and_selectors` | Static symlinks/hardlinks and unsafe payload aliases reject with observed files preserved; no hostile same-user traversal guarantee |
| T01/T28 no initialization or migration | `recovery_never_creates_missing_root_or_adopts_interrupted_initialization`, `recovery_does_not_migrate_immutable_append_or_legacy_profiles` | Missing roots and interrupted initialization cannot become accepted stores through recovery; MSS2/MSU2/legacy layouts are not adopted or rewritten |
| T28 local generations | `multi_height_candidate_advances_only_one_local_generation`, `later_generation_recovery_uses_pinned_names_not_highest_directory` | Multi-height replacement advances once; later recovery uses its exact old/new generation pins, without deriving authority from the greatest observed name |
| T29 completion failure/retry paths | `completion_from_clean_old_retries_every_recovery_boundary`, `completion_from_sparse_staging_retries_every_recovery_boundary`, `completion_from_partial_pending_selector_retries_every_recovery_boundary`, `selected_candidate_cleanup_retries_every_recovery_boundary` | Each recorded real recovery boundary is interrupted separately; same-action retries reconstruct the complete selected history and retain root identity and lock ownership |
| T29 already-absent directory durability regression | `already_absent_old_blocks_directory_is_resynchronized_and_every_boundary_retries` | Manifest-only old residue still synchronizes the old generation before manifest unlink; trace asserts sync-before-manifest ordering and retries every boundary |
| T29 retention failure/retry paths | `retention_from_sparse_staging_retries_every_recovery_boundary`, `retention_from_pending_selector_retries_every_recovery_boundary` | Retention retries preserve selected old history; pending removal becomes durable before candidate cleanup can make that generation incomplete |
| T29/T30 original publication failures | `every_original_replacement_failure_can_complete_without_releasing_ownership`, `every_prepublication_replacement_failure_can_retain_exact_old_history` | Actual E2a replacement fault traces feed consuming E2b recovery; ordinary access stays fenced and competing recovery stays excluded throughout the retained lock lifetime |
| T29 explicit action changes on retry | `interrupted_retention_can_explicitly_switch_to_completion_at_every_boundary`, `interrupted_completion_can_retain_old_only_before_candidate_selection` | Completion can restage after interrupted retention; switching completion to retention is allowed only while old remains selected, otherwise files stay unchanged and completion can proceed |
| T29 post-validation discrepancy | `changed_residue_after_selected_sync_stops_before_any_cleanup` | After `SelectedDurable`, a changed late old block, old manifest, selected-candidate block or new unknown old entry rejects before any cleanup mutation; exact post-tamper tree is preserved |
| T21 prefix budget accounting | `residual_prefix_reads_charge_the_growth_sentinel_and_exhaust_without_mutation`, `exhausted_prefix_read_budget_prevents_open_hook_and_suffix_write` | Prefix comparison charges length plus one; exhausted allowance rejects without mutation, descriptor-open callback or suffix write |
| T30 cooperative recovery ownership | `an_existing_writer_or_shared_inspector_excludes_recovery`, `replacement_recovery_obeys_independent_writer_and_reader_ownership` | Same-process and actual independent writer/reader owners exclude a new recovery writer before staging changes |
| T30 recovered writer process lifetime | `replacement_recovery_child_retains_root_ownership_until_drop_or_termination` | A separate process completes or retains, performs consuming idempotent recovery and keeps its returned writer exclusive until Drop or termination; root inode and checked reopen are preserved |

### Failure scenarios and evidence boundaries

Private trace tests first observe a successful path, then inject one error after
each recorded boundary in separate owned directories. They verify continuous root
ownership, unchanged root identity, ordinary-open strictness and recovery with the
retained request. Repeated boundaries and multiple initial shapes remain scenarios
within the named test functions, not additional unique tests.

The action-switch tests cover both directions. An interrupted retention can be
followed by completion at each recorded retention boundary. An interrupted
completion can retain old only before candidate selection; afterward retention
rejects without changing the files and explicit completion remains available.
The absence of an old blocks directory is synchronized on retry before manifest
removal, even if this call did not remove that directory. Observed absence alone
does not prove that a previous removal was durable.

The `SelectedDurable` mutation case verifies another complete comparison before
the first cleanup unlink. It deliberately corrupts a later surviving old block,
the old manifest or a selected candidate block, or adds an unknown old entry.
Recovery returns no handle and preserves that exact damaged layout. This tests the
specified boundary; it does not claim protection from arbitrary hostile same-user
mutation between every pathname operation.

Fault hooks inject errors after real operations, and explicit prefix fixtures
cover partial file contents. These checks do not simulate all torn writes,
physical power loss, firmware faults or filesystem implementations. A successful
retry proves the currently replayed and synchronized result within the documented
filesystem model, not retrospective success of a previously failed synchronization.

### Resource bounds, example and continuation

Recovery retains the **39,118,264-byte** distinct encoded-input ceiling and one
**313,011,680-byte** payload-read allowance, including repeated comparison/replay
and growth sentinels. Root/generation/block enumeration is bounded separately.
Per-chain limits remain 4096 blocks, 16 MiB MRB2 and 1,048,576 cumulative
state/configuration items. No pruning, cap bypass or authoritative state cache is
introduced to finish recovery.

The [contract's derivation](STORAGE_REPLACEMENT_RECOVERY_V2.md#8-separate-finite-resource-bounds)
gives successful path upper bounds: old-selected Complete uses four full replays
and at most **293,447,607 charged bytes**; already-selected Complete uses two and
**176,068,227 bytes**; RetainOld with pending uses two and **215,195,249 bytes**.
These fit the enforced allowance and are separate from resident memory, metadata
calls, write volume and elapsed time. Prefix-descriptor reads use the same meter.

The synthetic
[`replacement_recovery_v2_demo`](../crates/mirror-storage/examples/replacement_recovery_v2_demo.rs)
uses independently accepted public fixture histories and owned temporary files to
exercise explicit recovery and complete reopened state. The example is an execution
gate, not another unique test function. All nine frozen independent references
remain required; no new serialized format or vector is added.

**T29 remains partial beyond the recognized local recovery states. At the E2b
checkpoint, next was MB-03F:** bounded fresh-peer candidate import/reconstruction
under independent network selection. Section 17 records its first framing/receiver
slice. Persistent daemon, wallet and operator APIs remain MB-04. Reference
custody, arbitrary corruption repair, missing/corrupt selected markers, unknown
residue and interrupted initialization are not supplied by this component.
Automatic network discovery, global latestness and final human consensus remain
unproven or unimplemented. Trusted local ownership, cooperative Linux locks and the
documented synchronization model remain assumptions; WSL/DrvFS/native Windows and
network filesystems are not newly verified. Gate A/B remain unmet, and no founder
privilege, monetary-policy choice, real biometric intake or anonymity mechanism
is introduced.

## 17. MB-03F1 bounded peer-history framing and candidate reconstruction

This is the historical F1 checkpoint. Section 18 records the completed F2
transport/adoption component and the current T31/T32 coverage boundary.

The [peer-history contract](PEER_HISTORY_V2.md) specifies separate MPN2/version-2
framing and one finite, consuming in-memory reconstruction attempt. Source is
[`mirror_protocol::v2`](../crates/mirror-protocol/src/v2.rs) and
[`mirror_network::v2`](../crates/mirror-network/src/v2/mod.rs). The receiver checks
the manifest against independently selected C/genesis and executes every complete
original MRB2 through actual `ChainV2::from_genesis`/`append`. Advertised terminal
fields remain transfer-consistency claims; they never become a trusted replay
selection, canonical adoption decision or global freshness claim.

Focused local execution passed **24 protocol tests** in
[`history_v2.rs`](../crates/mirror-protocol/tests/history_v2.rs) and **36 receiver
tests** in [`history_receiver_v2.rs`](../crates/mirror-network/tests/history_receiver_v2.rs).
The new synthetic
[`history_receiver_v2_demo`](../crates/mirror-network/examples/history_receiver_v2_demo.rs)
also passed. These are **60 distinct runtime test functions**, not an inferred
workspace total. Scenario loops and repeated fixtures inside a function, release
reruns and example assertions are not extra unique tests.

Full workspace/documentation/release execution, independent-reference checks,
source/tree publication and CI results are separate acceptance evidence in
[CURRENT_STATUS.md](../CURRENT_STATUS.md) and [MASTER_PLAN.md](../MASTER_PLAN.md).
The new consuming-receiver compile-fail documentation example is a separate gate;
it is not one of these 60 runtime functions. Historical section-16 totals belong
to the accepted E2b source and must not be presented as a new F1 full-suite result.

### Codec and independent framing vectors

| Boundary | Implemented test functions | Scope of assertion |
|---|---|---|
| T21/T31 fixed summaries and exact original payloads | `summaries_and_control_frames_match_independent_vectors`, `manifest_and_complete_block_frames_match_independent_chain_bytes`, `full_transcript_lengths_and_hashes_match_independent_reference` | Begin/End bytes, complete manifest/block bytes, framing order and transcript accounting match independent frozen vectors |
| T21 inclusive finite ceilings | `maximum_structural_shape_has_exact_inclusive_wire_bound`, `header_vectors_cover_every_kind_and_inclusive_size_boundary` | The structural maximum is 4099 frames / 19641112 wire bytes; each kind admits only its documented payload bounds |
| T31 fixed-header rejection | `short_or_extended_header_is_rejected`, `invalid_magic_is_rejected_before_payload`, `unsupported_versions_are_rejected_before_payload`, `unknown_kinds_are_rejected_before_payload` | A header must be exactly 12 bytes and use MPN2/version2 with a known kind before a body is admitted |
| T21 kind-specific payload bounds | `exact_control_payload_sizes_are_enforced_from_header_only`, `variable_payload_bounds_are_enforced_from_header_only` | Begin/End are exactly 152 bytes; manifest and height-prefixed complete block lengths obey inclusive per-kind limits |
| T21/T32 peer-summary arithmetic | `summary_count_and_height_are_bounded_and_agree`, `summary_manifest_length_is_bounded`, `declared_block_bytes_respect_count_and_global_total`, `summary_decoder_requires_exact_length_and_checks_untrusted_fields` | Count, height, manifest length and cumulative block-byte claims are checked with bounded/overflow-safe arithmetic; structural acceptance is not branch validity |
| T31/T32 exact complete-frame boundaries | `all_truncations_of_each_frame_shape_are_rejected`, `trailing_bytes_and_concatenated_frames_are_rejected`, `split_header_payload_decoding_enforces_actual_payload_length` | A complete-frame slice contains exactly one frame; a checked header cannot admit a shorter or longer payload |
| T21 borrowed decoding | `decoded_payloads_borrow_original_bytes_without_copying` | Manifest and MRB2 payload views borrow the supplied input; codec decode performs no input-sized payload copy |
| T21 height and bounded encoding | `block_ordinal_has_inclusive_maximum_and_rejects_overflow_values`, `encoder_rejects_out_of_range_payloads_before_returning_frames`, `maximum_payload_bytes_round_trip_without_weakening_header_limits` | Height prefixes remain in 0..4095; send-side framing also respects the same absolute bounds |
| T31 authority and compatibility separation | `framing_success_is_not_manifest_block_or_network_acceptance`, `v1_and_v2_remain_distinct_protocols` | Well-framed opaque bytes may still be semantically invalid; MIRR/V1 and MPN2/V2 reject cross-decoding without fallback |

The new independent generator
[`reference_peer_history_v2.py`](../tools/reference_peer_history_v2.py) wraps
existing frozen chain/manifest bytes and checks the new
[`peer_history_v2.txt`](../crates/mirror-protocol/tests/vectors/peer_history_v2.txt)
fixture. It introduces no alternate block format or cryptographic hash domain.
Its complete execution result belongs to the reference gate, separately from
Rust tests that compare against its frozen output. Existing reference vectors
remain unchanged and required.

### Complete candidate reconstruction and consuming failure

| Boundary | Implemented test functions | Scope of assertion |
|---|---|---|
| T17/T25/T26 full state and accounting | `complete_history_reproduces_full_state_and_exact_wire_accounting`, `split_header_payload_path_rebuilds_the_same_complete_candidate` | Both full-frame and checked-header/payload paths reproduce the exact accepted ChainV2, resource counters, frame count and wire-byte sum |
| T17/T18/T25 genuine membership forks | `identical_accounts_do_not_collapse_different_membership_histories` | Equal account bytes do not collapse distinct membership state or population peaks; both exact branches reconstruct, and the existing comparator keeps current on the tested tie |
| T26/T32 no latestness inference | `a_complete_genesis_only_candidate_is_not_a_latestness_claim` | A valid complete genesis-only transfer is accepted as a candidate, while the existing comparator keeps a longer current chain |
| T02/T03/T28 independent network and seed proof checks | `independent_network_pins_reject_before_manifest_processing`, `manifest_must_match_independent_genesis_even_when_begin_matches`, `all_manifest_seed_proofs_are_checked` | Matching Begin claims cannot authorize a foreign manifest or missing/invalid configured bootstrap proof |
| T31 header and exact payload admission | `unknown_magic_version_and_kind_reject_in_header_phase`, `truncated_or_trailing_complete_frame_never_yields_a_candidate`, `pending_payload_requires_exact_declared_length` | Header/payload mistakes consume the attempt and expose no partial candidate |
| T31/T32 phase completion | `every_unfinished_phase_refuses_finish`, `phase_order_rejects_missing_repeated_or_restarted_metadata`, `genesis_payload_size_is_rejected_before_payload_is_requested` | Metadata cannot restart or skip a phase; the empty-genesis payload is exactly 176 bytes; every incomplete phase refuses to finish |
| T09/T23/T32 ordered actual blocks | `duplicate_and_reordered_blocks_are_rejected`, `wire_height_and_actual_block_height_must_agree`, `actual_parent_hash_is_checked_for_each_received_block` | Transport ordinal, actual block height, sequence and accepted parent all agree before advancement |
| T32 explicit terminal boundary | `terminal_frame_is_required_only_after_all_promised_blocks`, `terminal_summary_cannot_change_the_captured_offer`, `extra_blocks_and_frames_after_end_are_rejected` | All blocks alone are insufficient; exact matching End is required and completed receivers reject further supplied input |
| T26/T32 prefix and false endpoint rejection | `valid_shorter_prefix_does_not_satisfy_a_longer_promised_history`, `false_advertised_tip_or_root_is_rejected_after_actual_execution` | A valid accepted prefix is not a completed promised history; actual terminal tip/root must match the captured peer claim |
| T21 checked allocation admission | `manifest_frame_length_must_match_offer_before_payload_allocation`, `configured_block_ceiling_is_checked_before_payload_allocation`, `manifest_rejects_claimed_total_impossible_under_its_configuration` | The split API refuses incompatible lengths before exposing a payload length to a future adapter; actual configuration and fixed-size genesis tighten the declared budget |
| T21 future and terminal byte budgets | `remaining_total_reserves_minimum_space_for_every_future_block`, `final_block_length_must_exactly_exhaust_the_claimed_total` | Each admitted payload leaves a feasible finite remainder; the last block must exhaust the claimed MRB2 total exactly |
| T23 invalid actual body/work/roots | `malformed_block_bytes_never_become_an_accepted_candidate`, `invalid_seeded_genesis_work_is_rejected_by_actual_chain_acceptance`, `peer_cannot_choose_another_difficulty_for_a_child`, `valid_work_cannot_override_a_wrong_operation_root`, `independently_frozen_wrong_ledger_root_is_rejected` | Framing, trusted network identity or valid work never replace actual block/body/root acceptance |
| T05/T09 execution authorization and ancestry | `a_remined_block_with_a_bad_signature_is_rejected_during_execution`, `signatures_from_another_valid_branch_do_not_override_actual_ancestry` | Re-mined roots/work reach the signature rejection boundary; otherwise valid signatures cannot supply another candidate's ancestry |
| T01/T21 sender and frozen profile | `sender_emits_exact_original_blocks_and_is_fused_after_end`, `legacy_v1_hello_is_not_a_v2_transfer`, `transport_caps_match_the_frozen_chain_and_block_profile` | Sender preserves complete original blocks and finite termination; V1 stays separate; cross-crate profile bounds agree |

### Public method and failure boundary

`HistoryReceiverV2::begin_frame(self, header)` consumes the receiver and returns
only a pending frame whose payload length has passed phase, per-kind, configured
and remaining aggregate checks. `PendingHistoryFrameV2::accept_payload(self, bytes)`
consumes that pending owner and requires exactly the admitted payload. All errors
destroy continuation; there is no public partial-chain accessor or clone. The
compile-fail example records that a rejected receiver cannot be reused to finish.

`accept_frame(self, bytes)` is convenience for an already supplied complete frame;
it cannot prove that its caller allocated input safely. A future socket adapter
must call `begin_frame` before allocating or reading the declared body, drop a
pending owner on read failure, and close/discard the failed attempt. F1 itself
performs no reads and has no wall-clock or socket-timeout mechanism.

After the final actual block, the peer's terminal claim is checked but the candidate
remains private. A matching End frame and exact total frame/byte accounting precede
successful `finish`. A receiver returning a candidate has executed every original
mixed operation and retains the whole accepted branch; no balances, approvals,
permanent reservations, population values or resource counters are grafted across
histories. The caller may subsequently compare accepted branches under independent
identity, but receiving alone neither selects nor writes a canonical store.

### Budgets, demonstration and remaining acceptance

For N blocks, M manifest bytes and B complete MRB2 bytes, framed-input accounting is
`N + 3` frames and `340 + M + B + 20*N` bytes. Bounds are **4096 blocks**,
**2781636 manifest bytes**, **16777216 MRB2 bytes**, **4099 frames** and
**19641112 total wire bytes**, with each actual block at most its configured cap
and the current hard ceiling of **1048576 bytes**. Actual ChainV2 execution retains
the unchanged **1048576 cumulative state/configuration-item** budget. These are
encoded-input and abstract-work bounds, not a total resident-memory, bandwidth,
multi-session, latency or availability guarantee.

The synthetic receiver example streams separate in-memory frames and reconstructs
the independently known complete main and membership-fork histories. It checks
whole state and existing branch comparison and reports the resource counters.
Malformed/truncated input rejection is covered by the focused receiver tests.
The example opens no socket, waits for no EOF and writes no store. Neither it nor
the focused tests establish network-process behavior.

**At the F1 checkpoint T31/T32 remained partial, with MB-03F2 next: actual TCP-session
integration and durable adoption.** That slice had to use checked header admission before socket payload
allocation, enforce bounded requests/connections and session deadlines, handle
fragmentation and early closure, reconstruct complete candidates across real peers,
and publish only appropriately selected accepted state through existing storage.
It must retain the old and candidate references needed for publication recovery.
An explicit End defines this F1 transfer; a later adapter must specify later-session
or connection behavior without inferring completion from EOF.

At that checkpoint MB-03F and MB-03 remained IN PROGRESS. Persistent node/operator APIs remain MB-04;
Ant Kiss, private matching, device adapters, human consensus, final monetary policy
and both launch gates are not completed by codec/receiver tests. Existing old
encodings, development fixtures and user policy remain preserved.

### F1 full-gate checkpoint evidence

At source `43ef08ca6d3e7878b7dab2d2355a7265d41a0351`, the local required gates passed **951 runtime tests**,
**9 compile-fail doctests**, release totals **237 / 121 / 49 / 111 / 352 / 79**
for personhood / core / state / consensus+chain / storage / protocol+network
(including their doctests), **10 independent references** and **14 examples**.
The latest delta is 24 codec and 36 receiver tests plus one compile-fail ownership
example. Source push/PR CI also passed; exact IDs and verified logs are recorded
in [CURRENT_STATUS.md](../CURRENT_STATUS.md). The one ignored subprocess helper
continues to run through its parent tests. These counts do not complete F2 or
change the historical T31/T32 partial assessment above. F2 evidence follows.

## 18. MB-03F2 finite TCP sessions and durable candidate adoption

Historical F2 acceptance is preserved here. Section 19 adds the separate durable
node-control layer; F2's own API still retains its recovery inputs in memory.

Checkpoint **MIRROR-MB03F2-001**. **MB-03F2, MB-03F and MB-03 are
VERIFIED_COMPONENT within the finite storage/peer-session scope.** T31/T32 are
verified for this path; persistent daemon admission, reconnect/multi-peer policies
and global availability remain MB-04 work.

Published implementation:
[`851193ea6428198441e6cdf29361b1885fae157b`](https://github.com/Ahmed-Sobeih/mirror/commit/851193ea6428198441e6cdf29361b1885fae157b).
Tested tree: `cf2e4bb9d29b250a000bb4680d1571e0355d1126`.
Parent: `f47b4306b13c4d079859472f2fdd919debd83eda`.
The exact-source CI conclusion, inspected job evidence and any later documentation
head are recorded separately in [CURRENT_STATUS.md](../CURRENT_STATUS.md). Local
passing gates below do not independently assert a remote CI result.

### Local gates and unique-test accounting

Rust **1.98.0**, with locked dependencies, passed **1,010 workspace/all-target
runtime tests**: 951 retained and 59 new. The new cases are:

| Location | New runtime test functions | Evidence boundary |
|---|---:|---|
| [session_v2.rs](../crates/mirror-network/tests/session_v2.rs) | 24 | Actual TCP session semantics, limits, failure ownership and deadlines |
| [session.rs private test](../crates/mirror-network/src/v2/session.rs) | 1 | A nonreading peer causes partial writes to expire at the original deadline |
| [candidate_adoption_v2.rs](../crates/mirror-network/tests/candidate_adoption_v2.rs) | 23 | Independent selection, publication, retained recovery input and explicit recovery |
| [peer_process_v2.rs](../crates/mirror-network/tests/peer_process_v2.rs) | 9 | Separate OS providers, complete state, durable reopen and rejected traffic |
| [session_vectors_v2.rs](../crates/mirror-network/tests/session_vectors_v2.rs) | 2 | Independent fixed request/transcript vectors and cross-layer limit arithmetic |

**Ten compile-fail doctests passed**, including the new consuming-adoption example
that forbids publishing the same attempt twice. Required release suites also
passed: personhood **234 runtime + 3 doctests**, core **121**, state **49**,
consensus **23**, chain **85 runtime + 3 doctests**, storage **350 runtime +
2 doctests**, protocol **38**, network **99 runtime + 2 doctests**. Release reruns
are not additional unique tests. Both intentionally ignored subprocess entry points
are invoked by their parent tests and are not counted as ordinary passes.

All **eleven independent Python reference checks** and **fifteen synthetic
examples** passed. The new [session reference](../tools/reference_peer_session_v2.py)
independently encodes MPQ2 and combines it with independently calculated F1
transcripts; it does not invoke Rust or use Rust output as its oracle. Formatting,
whitespace and allocation/runtime-data audits passed. **129 of 132 retained
Rust/dependency/reference/fixture files are byte-identical** to the parent. The
three changed retained files are the V2 module documentation/exports, network
manifest and lockfile; F1 runtime behavior and all prior frozen bytes remain
unchanged. One local `mirror-storage` dependency edge is added; no external
dependency version changes.

### Rule-to-source mapping

| Boundary | Implemented test functions | Scope of assertion |
|---|---|---|
| T01/T02/T31 independent request selection | `provider_checks_its_independent_network_before_accepting`, `provider_rejects_wrong_network_request_without_responding`, `provider_malformed_and_short_requests_never_produce_output`, `false_network_claim_rejects_under_independent_requester_pins` | Both sides use independently selected C/genesis; malformed or foreign MPQ2 and response claims cannot choose the network or trigger downgrade |
| T21/T31 header admission before payload | `oversized_header_rejects_without_waiting_for_or_allocating_payload`, `phase_invalid_header_rejects_before_reading_an_otherwise_bounded_payload`, `admitted_payload_must_fit_remaining_owner_budget_before_allocation` | Fixed header, F1 phase/configuration/remaining limits and remaining owner allowance pass before declared payload allocation/read |
| T31/T32 explicit one-shot termination | `every_early_close_phase_fails_without_a_candidate`, `one_trailing_byte_or_a_second_transcript_rejects_after_end`, `provider_rejects_repeated_requests_before_any_history_output`, `accepted_end_without_eof_expires_instead_of_returning_a_candidate` | Exactly one request/response; early EOF fails, End requires EOF, and a single extra byte rejects without a candidate |
| T32 absolute receive/accept deadline | `partial_payload_stall_obeys_the_original_deadline`, `drip_fed_header_cannot_refresh_the_absolute_deadline`, `provider_requires_request_eof_within_the_same_deadline`, `an_unconnected_listener_has_a_finite_accept_deadline` | Connection/accept, request, payload and EOF phases use one original deadline; short reads cannot refresh it |
| T32 absolute send deadline | `nonreading_peer_cannot_extend_partial_writes_past_absolute_deadline` | The private syscall loop encounters real partial writes and blocked output; sent bytes remain charged and the same deadline terminates the attempt; this does not fabricate a maximum-sized accepted chain |
| T21/T32 cumulative attempts and bytes | `failed_attempts_remain_charged_and_the_owner_cannot_reset_them`, `failed_and_partial_io_spend_one_cumulative_wire_allowance`, `provider_partial_output_never_exceeds_its_remaining_allowance`, `terminal_eof_probe_requires_spare_capacity_but_charges_zero_for_eof` | Failed attempts and partial I/O are cumulative; syscall lengths never exceed remaining capacity; zero-byte EOF is uncharged but requires spare sentinel capacity |
| T23/T25/T32 actual execution and claims | `changed_terminal_claim_rejects_over_real_tcp`, `actual_wrong_ledger_block_rejects_over_real_tcp`, `actual_invalid_block_from_process_is_reexecuted_and_rejected` | Correct framing or mining work never substitutes for complete ledger execution and matching terminal claims |
| T01/T28 explicit initialization | `explicit_fresh_initialization_reopens_complete_received_state`, `fresh_wrong_identity_rejects_before_creating_a_directory`, `fresh_initialization_preserves_unrelated_nonempty_data`, `fresh_initialization_never_falls_back_to_overwriting_an_existing_store` | Initialization is explicitly chosen and cannot reinterpret an existing, foreign or partially initialized store |
| T02/T28 local selection and consecutive publication | `all_current_checkpoint_fields_and_generation_remain_independent_pins`, `even_keep_current_requires_the_actual_local_generation`, `consecutive_adoptions_use_retained_local_generation_without_peer_authority` | Every local endpoint field and generation remains independently retained; complete peer reception cannot supply current-selection authority or invent the next generation |
| T17/T18/T25 complete membership-fork adoption | `equal_account_divergent_tie_preserves_all_current_files_and_membership`, `shorter_divergent_history_cannot_replace_current_or_advance_generation`, `preferred_equal_account_fork_replaces_entire_membership_and_resource_history`, `preferred_remote_fork_replaces_memberships_and_population_with_whole_branch` | Ties/shorter branches keep exact current bytes; preferred equal-account forks replace the whole membership, approval, permanent-record, population and resource history |
| T28/T29/T30 pre-publication input retention | `preparation_is_pure_and_retains_exact_recovery_intent_before_publication`, `changed_disk_after_preparation_fences_and_preserves_exact_inputs_and_lock`, `a_fenced_store_is_not_reactivated_by_receiving_another_candidate` | Preparation writes nothing; both exact histories/pins exist before mutation; disk discrepancy retains the owning failure and cannot revive a fenced writer |
| T29/T30 explicit recovery ownership | `explicit_completion_uses_retained_both_histories_and_same_root_ownership`, `explicit_retention_removes_only_authorized_candidate_prefixes`, `failed_recovery_returns_no_handle_but_keeps_inputs_for_explicit_retry`, `selected_candidate_is_never_rolled_back_by_retention_after_failure` | First recovery consumes the same lock; an error returns no handle; later retry explicitly reacquires ownership; unknown content remains untouched and selected candidate never rolls back |
| T25/T26/T28 actual separate processes | `fresh_process_transfer_reconstructs_and_durably_reopens_complete_state`, `equal_height_remote_membership_fork_keeps_current_exact_store`, `fragmented_process_transcript_reconstructs_the_complete_history` | Actual TCP process boundaries reproduce complete accepted state and strict local reopening, including distinct memberships with equal account bytes |
| T21/T29/T32 process rejection leaves storage unchanged | `early_closed_process_never_exposes_partial_candidate_or_changes_storage`, `stalled_process_is_rejected_by_absolute_deadline_and_keeps_store`, `oversized_header_from_process_rejects_without_reading_declared_body`, `changed_terminal_claim_from_process_does_not_publish_completed_blocks` | Failed reception returns no candidate; the existing selected store remains unfenced and byte-identical and strictly reopens under its original pins |

### Finite-session and durability boundary

The unchanged F1 history caps remain in force. MPQ2 is exactly 72 bytes. One
non-Clone session owner permits **one connected/accepted stream at a time**,
**at most eight attempts**, **157,129,488 cumulative actual read/write bytes**,
and **at most 60 seconds per attempt**. It never retries automatically. Both a
matching F1 End and socket EOF are required before returning a candidate. The
EOF sentinel requires one unused byte of capacity, but EOF itself charges zero.
These are per-owner API bounds, not global connection admission, kernel-buffer,
RSS, bandwidth, real-time scheduling or hard CPU-preemption guarantees.

Adoption follows completed reception and is outside the transport deadline.
It publishes only an independently compared preferred branch under the existing
owned store. Its exact old/candidate recovery inputs are **in memory**, not a
durable node intent journal. Recovery fault cases construct recognized synthetic
residue and use existing verified recovery primitives; they do not establish
physical power-loss behavior or arbitrary-corruption repair.

The fifteenth example,
[`peer_session_v2_demo`](../crates/mirror-network/examples/peer_session_v2_demo.rs),
launches real provider processes with separate synthetic stores. Fresh reception
uses eight F1 frames and 5,406 framed bytes, then reopens height 4/generation 0
with active/pending/peak **1/0/3**. An equal-height fork keeps main height 3 with
**3/0/3**. A preferred equal-account membership fork publishes height 4/generation 1
and reopens with **1/1/2**, **3,605 MRB2 bytes** and **73 state items**. Exact whole
state, including reservations and approvals, is checked throughout. The example
is finite and introduces no daemon, transaction pool or wallet.

**At the F2 checkpoint, next was MB-04A1 — durable node control state and restart
authority**, followed by MB-04A2 operator commands. Section 19 now verifies A1's
separate persisted pins and both-history intent, with explicit restart recovery.
F2 itself cannot scan/promote residue or infer missing intent from a peer. MB-04 persistent networking/global admission, Ant Kiss,
human consensus, policy-gated economics, private matching, actual devices and
Gate A/B remain unfinished. The 35 matrix rows remain requirements, not a count
of completed test functions or a project-readiness percentage.

## 19. MB-04A1 durable node control and restart authority

Checkpoint **MIRROR-MB04A1-001**. **MB-04A1 is VERIFIED_COMPONENT — DURABLE LOCAL
CONTROL / EXPLICIT RESTART RECOVERY**, within the finite library/harness scope.
T28 includes durable local custody. T29 retains its broader physical-failure
boundary; T31/T32 remain verified for the existing finite TCP path. At this A1
checkpoint, the V2 operator CLI and persistent service were separate work. Section
20 records A2's later finite command layer without changing this historical record.

Implementation source:
[`32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0`](https://github.com/Ahmed-Sobeih/mirror/commit/32c9117ab9dab36faa1a47f9ebb4fb0718b57dc0).
Tested tree: `d79080968d822a2267ce5f1b1196efcf16d2e546`.
Parent checkpoint: `37010bc5c5827f5a8dd886af0e79d079ebfe1d4e`.
[Source PR CI 45861912935](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45861912935)
passed all configured gates. Full job `140054826270` was inspected; synthetic merge
checkout `62437dc1c338156fb984349b791efc7a4ad34ebb` has no file differences from the implementation source.
Later documentation-head CI is checked separately and recorded on PR #1.

### Unique tests and required gates

Rust **1.98.0**, with locked dependencies, passed **1,083 workspace/all-target
runtime tests**: 1,010 retained plus **73 new**. These are test functions, not the
number of parameterized paths, spawned subprocesses or boundary stops.

| Location | New runtime functions | Evidence boundary |
|---|---:|---|
| [control_records_v2.rs](../crates/mirror-node/tests/control_records_v2.rs) | 24 | Independently calculated local records/intents, strict metadata and complete two-history replay |
| [control_state_v2.rs](../crates/mirror-node/tests/control_state_v2.rs) | 31 | Public ownership, status, adoption, restart recovery and malformed/unsafe state |
| [control_process_tests.rs](../crates/mirror-node/src/v2/control_process_tests.rs) | 12 | Actual SIGKILL, fresh-process reopening/recovery and competing operators |
| [control_failure_tests.rs](../crates/mirror-node/src/v2/control_failure_tests.rs) | 6 | Returned errors, explicit retry and changed-authority checks before mutation/retirement |

**Eleven compile-fail doctests passed**, ten retained plus a consuming node-adoption
case. The new node release gate passed **73 runtime tests and one doctest**; all
retained required release suites passed. Release reruns add no unique test count.
Three subprocess-only helpers are intentionally ignored by ordinary enumeration
and exercised by parent tests.

All **twelve independent reference checks** and **sixteen synthetic examples**
passed, along with scoped formatting, whitespace, preservation and allocation/
runtime-data audits. **140 of 142 retained runtime/dependency/reference/fixture
files are byte-identical** to the parent. Only the node Cargo manifest and lockfile
add one local personhood development-dependency edge. The existing V1 executable,
storage/network production code, all prior byte/hash domains, monetary behavior
and frozen earlier fixtures remain unchanged; no external package version changes.

The independent [node-control reference](../tools/reference_node_control_v2.py)
constructs MNC2/MNI2 from the established independent chain/history/storage inputs.
It supplies six exact control vectors and four complete two-history intents, without
running Rust or using its output as an oracle. New local checksum domains do not
change any consensus or existing wire format.

### Requirement-to-source mapping

| Boundary | Implemented test functions | Verified assertion |
|---|---|---|
| T02/T28 independent identity and canonical control | `confirmed_record_requires_independent_configuration_and_genesis`, `duplicated_complete_checkpoint_must_match_nested_msr2`, `independent_identity_and_wrong_full_initial_pins_reject_before_control_publication`, `independently_wrong_network_rejects_clean_status_open_and_recovery` | External C/genesis and every initial local checkpoint field are required; the duplicate complete checkpoint and canonical nested MSR2 must agree |
| T21/T28 exact local formats | `confirmed_records_match_six_independently_generated_vectors`, `intents_match_four_independent_complete_two_history_vectors`, `confirmed_record_rejects_every_truncation_and_trailing_byte`, `intent_bounds_both_declared_histories_before_execution`, `candidate_generation_must_be_exactly_old_plus_one` | New version/kind/checksum/length rules and adjacent generations match independent vectors; declared lengths are bounded before execution |
| T23/T25/T26 actual two-history execution | `independently_frozen_wrong_actual_ledger_root_rejects_in_either_history`, `forged_metadata_with_valid_nested_checksums_cannot_replace_actual_history_totals`, `valid_history_cannot_substitute_a_different_membership_fork`, `both_histories_require_complete_matching_end`, `both_histories_reject_input_after_their_end` | Both complete original branches execute under their own ancestry and match canonical metadata; valid checksums cannot replace history acceptance |
| T01/T28 explicit control attachment | `missing_status_does_not_create_control_and_open_never_initializes_it`, `initialization_never_creates_a_missing_store`, `repeated_initialization_preserves_existing_control_and_selected_history`, `explicit_control_initialization_attaches_existing_store_and_clean_restart_replays_it` | Initialization attaches only to an already initialized, independently pinned profile-3 store; status/open cannot initialize or repair missing state |
| T25/T28 whole branch and no-op decisions | `tied_shorter_and_identical_candidates_leave_all_control_and_store_bytes_unchanged`, `preferred_membership_fork_publishes_complete_state_and_retires_durable_intent`, `separate_process_clean_restarts_preserve_tie_and_preferred_membership_history` | Keep-current makes no control/store writes; a preferred equal-account membership fork moves its entire state, generation and confirmed authority together |
| T28/T30 truthful read-only status | `valid_intent_status_is_read_only_and_ordinary_open_requires_explicit_recovery`, `clean_control_cannot_hide_corrupt_or_unexpected_selected_store_contents`, `stale_control_checkpoint_does_not_choose_the_higher_store_generation` | Clean requires full selected replay; valid intent reports recovery-required without choosing or repairing the store; stale pins never scan for a higher generation |
| T28/T29 explicit restart action | `explicit_complete_from_durable_intent_replays_and_publishes_both_membership_histories`, `explicit_retain_old_retires_intent_without_replacing_selected_history`, `killed_published_intent_restarts_require_explicit_completion_or_retention` | Fresh-process recovery obtains both exact histories from the complete persisted intent and applies only the explicitly requested permitted action |
| T29 candidate no-rollback | `already_selected_candidate_cannot_be_retained_as_old_even_with_old_confirmation`, `confirmed_candidate_with_old_selected_store_is_inconsistent_and_preserved`, `confirmation_staging_with_old_selected_store_cannot_authorize_a_new_publication`, `killed_after_store_publication_uses_persisted_histories_and_never_rolls_back` | Selected candidate cannot become old; confirmed/pending candidate confirmation requires a clean matching candidate store, not permission to publish from an old or residual layout |
| T29 intent before store mutation | `killed_partial_intent_writes_reject_and_preserve_store_and_control`, `changed_authority_after_intent_sync_is_rejected_before_store_publication`, `changed_authority_after_recovery_resync_is_rejected_before_store_mutation` | Partial/unpublished intent cannot authorize mutation; exact published layout, confirmation and intent are rechecked after synchronization before storage changes |
| T29 confirmation and retirement | `killed_confirmed_record_writes_allow_only_candidate_completion`, `twice_killed_confirmation_prefix_extension_remains_exact_and_completable`, `killed_intent_retirement_has_durable_candidate_confirmation_for_clean_restart`, `changed_confirmation_is_detected_before_retirement_and_final_ready_returns`, `foreign_confirmation_staging_after_store_success_is_never_overwritten_or_hidden` | Authorized confirmation suffixes append without truncation; confirmed durability precedes intent retirement; foreign or changed data cannot be hidden by a ready return |
| T29 returned errors and repeated restart | `adoption_returned_errors_release_ownership_and_preserve_phase_appropriate_authority`, `recovery_returned_errors_allow_only_explicit_phase_consistent_retry`, `recovery_itself_survives_another_process_termination_without_losing_authority`, `killed_recovery_resynchronization_preserves_both_explicit_action_choices` | Errors return no owner; fresh explicit retries retain the allowed old/candidate choice and cannot silently reverse a completed selection |
| T21/T28/T29 unsupported control | `corrupt_truncated_and_overlong_intent_never_authorize_store_changes`, `oversized_sparse_intent_is_rejected_before_store_mutation`, `intent_staging_is_preserved_even_when_it_contains_a_complete_valid_record`, `orphan_confirmation_staging_and_partial_control_initialization_are_not_repaired`, `killed_partial_control_initialization_rejects_without_touching_the_selected_store` | Corrupt, overlong, unsupported pending intent and partial initialization remain rejected and preserved; oversized length is rejected before payload allocation/store mutation |
| T30 path and file ownership | `symbolic_and_hard_linked_control_files_are_rejected_without_touching_targets`, `unsafe_intent_and_confirmation_staging_files_are_preserved`, `symbolic_control_or_store_root_is_rejected_instead_of_followed`, `identical_or_nested_root_pairs_are_rejected_without_mutation`, `replacing_the_owned_control_root_inode_is_detected_before_adoption` | Separate non-nested stable roots, regular single-link payloads and checked path components are mandatory; no symlink traversal, recursive cleanup or live root swap |
| T30 cooperating processes | `controller_owns_both_roots_and_excludes_competing_controller_and_direct_store_writer`, `competing_operator_process_is_excluded_until_actual_owner_exit` | Nonblocking control-before-store ownership excludes competing controllers and store writers; actual process exit releases descriptors, not a PID-file claim |

### Process-boundary and resource scope

The process harness performs actual SIGKILL at **six initialization boundaries**
and **sixteen adoption boundaries**, including real half-written records, the
interval after store publication but before confirmation, confirmation rename,
and intent retirement. Fresh helper processes then exercise status, ordinary
reopening and explicit recovery; recovery interruption and candidate-prefix
extension are also repeated.
These are software/process termination cases, not a simulated physical power cut.
The private hooks have no production environment-variable or public enable path.
All helper readiness/exit waits are bounded, children are killed/reaped on unwind,
and same-executable file-ownership tests share a gate to avoid unrelated inherited
fork descriptors prolonging a lock. Actual competing-process cases remain enabled.

MNC2 is exactly **464 bytes**. MNI2 contains at most two **19,641,112-byte** complete
F1 histories and **984 fixed bytes**, so its ceiling is **39,283,208 bytes**.
One call shares a **314,269,392-byte control payload-read allowance**, including
length-plus-one sentinels and repeated comparisons. The existing store has its own
separate read/replay budgets. These caps do not establish total memory use,
wall-clock bounds, global admission or protection against a hostile local owner.

The [node-control example](../crates/mirror-node/examples/node_control_v2_demo.rs)
uses four distinct processes. Initial control and a restarted kept tie select
height **3**, generation **0**, four accounts, active/pending/peak **3/0/3**, two
recorded approvals, **3,206 MRB2 bytes** and **58 state items**. Preferred publication
and a fresh process reopen select height **4**, generation **1**, four accounts,
active/pending/peak **1/1/2**, two approvals, **3,605 MRB2 bytes** and **73 state items**.
Full state and canonical metadata match; equal account bytes do not hide changed
membership or population history. The example uses public APIs and synthetic data.

### Next component recorded at the A1 checkpoint

**At this historical checkpoint, next was MB-04A2 — finite V2
init/status/sync-once/explicit-recover operator commands.** Section 20 records
that command layer. The A1 API and accepted-layout contract remain intact.
The 35 original requirements stay intact and are not a project-completion percentage.

## 20. MB-04A2 finite V2 operator commands

Checkpoint **MIRROR-MB04A2-001** adds the separate `mirror-node-v2` executable.
The implemented contract is [NODE_OPERATOR_V2.md](NODE_OPERATOR_V2.md); it composes
the accepted A1 controller and F2 sessions without changing prior production
runtime, storage, consensus or wire semantics. The implementation is finite:
`init` attaches control to an existing accepted profile-3 store, `status` observes,
`sync-once` receives one candidate then adopts it, and `recover` performs only the
explicit requested action. No fresh-store/bootstrap policy is supplied.

### Exact source and acceptance fields

All 38 required local command gates and all 43 exact-source CI steps passed.
These fields record inspected full logs and published readback; full acceptance is
in [CURRENT_STATUS.md](../CURRENT_STATUS.md) and
[master-plan section 28](../MASTER_PLAN.md). Later documentation-head CI remains separate.

| Evidence | Recorded value |
|---|---|
| Implementation source | [eea42a403fef2a1442e553df2a6823b1b25da616](https://github.com/Ahmed-Sobeih/mirror/commit/eea42a403fef2a1442e553df2a6823b1b25da616) |
| Tested source tree | `cda30cd7e8b564a82d09a46b7067012e7112e2dd` |
| Parent checkpoint | `957599df1b8a424ba333d0e0c10247b2807c3a09` |
| Published implementation/contract file count | 9 |
| Exact-source PR CI run | [45882757484](https://github.com/Ahmed-Sobeih/mirror/actions/runs/45882757484) |
| Inspected full job | `140125894368` |
| Inspected synthetic merge checkout | `7dbc9921785f52c4c0136c27dc5890f9b0188e2c` |

Runtime accounting is **1,137 workspace/all-target tests**, comprising
**1,083 retained plus 54 new**. New test functions are:

| Location | New runtime functions | Evidence boundary |
|---|---:|---|
| [args.rs](../crates/mirror-node/src/bin/mirror-node-v2/args.rs) | 23 | Complete bounded parsing, mandatory independent inputs, command separation and pre-effect rejection |
| [output.rs](../crates/mirror-node/src/bin/mirror-node-v2/output.rs) | 4 | Bounded records, truthful uninspected status and fallible write/flush handling |
| [errors.rs](../crates/mirror-node/src/bin/mirror-node-v2/errors.rs) | 3 | Distinct static identity/ownership/transport diagnostics with no raw input echo |
| [operator_commands_v2.rs](../crates/mirror-node/tests/operator_commands_v2.rs) | 24 | Actual executable invocation, sockets, state/metadata preservation, complete adoption, recovery choices, ownership and output failure |

The node gate accounts for **127 runtime tests and one retained
compile-fail documentation case**. The complete required gates passed eleven
workspace compile-fail doctests, all required release suites, twelve independent
references and sixteen synthetic examples. A2 adds no reference program, frozen
format vector or example gate. Existing ignored subprocess entry points remain
parent-invoked helpers; parameterized cases, spawned children, examples and release
reruns are not extra unique runtime tests.

Compatibility comparison found **153 of 154 retained source/reference/fixture
files byte-identical** to the parent. Only `crates/mirror-node/Cargo.toml` adds
`default-run = "mirror-node"`, preserving the prior V1 invocation when the separate
V2 executable is present. Cargo.lock, previous runtime code, all prior fixtures,
byte/hash domains, monetary behavior and dependencies remain unchanged.

### Requirement-to-source mapping

Dispatch and ownership live in
[main.rs](../crates/mirror-node/src/bin/mirror-node-v2/main.rs). The table names
unit cases in the files above and actual-command cases in `operator_commands_v2.rs`.
The [process/socket helper](../crates/mirror-node/tests/support_operator_v2/mod.rs)
reuses the unchanged synthetic A1/F2 history and complete-state assertions.

| Boundary | Implemented test functions | Assertion and evidence scope |
|---|---|---|
| T01/T28 explicit attachment | `init_requires_explicit_attach_and_all_full_checkpoint_flags`, `explicit_init_attaches_existing_store_and_a_fresh_process_verifies_it`, `init_refuses_existing_control_and_never_creates_a_missing_selected_store` | No implicit fresh store, overwritten control or init-after-open-failure; exact successful pins survive a fresh process |
| T02/T28 independently selected authority | `init_rejects_each_wrong_independent_identity_and_full_selection_pin`, `independent_config_and_genesis_mismatches_fail_before_connecting`, `hashes_accept_ascii_case_and_normalize_output` | Both external network pins and every initial selection field are checked; wrong identity fails without peer contact or changed state |
| T21/T30 strict command admission | `malformed_missing_duplicate_and_cross_command_arguments_have_no_effects`, `invalid_hash_numeric_peer_and_limit_arguments_are_rejected_before_effects`, `non_utf8_control_characters_and_argument_budgets_are_bounded_without_effects`, `init_checkpoint_bounds_are_checked_before_any_action` | Usage exit 2 precedes filesystem/socket effects; explicit size/count/range bounds reject malformed or ambiguous command input |
| T28/T30 missing and clean status | `missing_status_reports_uninspected_without_creating_or_reading_a_store`, `clean_status_verifies_full_pins_without_application_writes`, `invalid_clean_control_or_store_returns_failure_and_preserves_every_object` | Actual exits 3/0/1 distinguish missing control, fully replayed clean selection and invalid state; application bytes/layout/inode/mtime stay unchanged |
| T28/T29 unresolved status | `pending_status_reports_authority_endpoints_without_inspecting_or_repairing_store`, `pending_record_labels_intent_without_claiming_current_store_selection` | Exit 4 reports confirmed/old/candidate pins with `store_selection=uninspected`, no selected claim and no guessed action, even with an invalid unresolved store |
| T30/T31 local readiness precedes contact | `sync_rejects_missing_pending_or_malformed_local_state_before_any_tcp_connection`, `sync_requires_all_three_deliberate_session_options`, `sync_peer_is_numeric_ipv4_or_bracketed_ipv6_with_nonzero_port` | Ready A1 ownership under explicit identity is required before numeric TCP contact; missing/pending/malformed state leaves no queued connection |
| T25/T26/T32 whole-branch transfer | `actual_tcp_identical_shorter_and_tied_histories_keep_complete_local_state`, `actual_tcp_preferred_membership_fork_survives_fresh_strict_reopen` | One actual TCP receive keeps current without writes or publishes the complete preferred accounts/membership/approvals/population state; output reports one attempt and actual charged bytes |
| T21/T32 consuming transport failure | `incomplete_trailing_and_invalid_block_transfers_preserve_selected_state`, `valid_end_without_terminal_eof_reaches_the_supplied_deadline_without_adoption`, `positive_tiny_wire_budget_is_enforced_by_the_actual_session` | Partial/invalid/extra data, missing terminal EOF and exhausted actual-byte allowance cannot expose/adopt a candidate; selected bytes and strict reopen remain unchanged |
| T28/T29 explicit recovery | `explicit_complete_command_publishes_the_canonical_pending_candidate`, `explicit_retain_old_command_retires_intent_without_changing_selected_store`, `candidate_selected_confirming_or_confirmed_can_only_complete_never_roll_back` | Command dispatch uses canonical persisted authority and the requested action only; selected/confirming/confirmed candidate cannot roll back |
| T29/T30 preserved unsupported state | `unknown_and_partial_control_artifacts_are_rejected_and_preserved` | Partial control initialization, unpublished intent and unknown files/directories reject across status/init/recovery without cleanup or promotion |
| T30 actual lifetime ownership | `sync_holds_both_roots_during_receive_and_kill_releases_ownership_without_adoption` | A real child owns both roots while response is blocked; another command fails busy without contacting its listener; killing/reaping the owner releases locks and preserves the old state |
| T29/T30 usable failure reporting | `broken_stdout_reports_possible_completed_init_and_fresh_status_resolves_it`, `broken_writes_and_failed_flushes_return_errors_without_panics`, `diagnostics_do_not_copy_untrusted_nested_error_text`, `transport_budget_deadline_and_terminal_failures_remain_distinct` | Broken actual stdout can follow completed initialization; exit 1 carries possible-completed/inspect-status advice, then a fresh command resolves clean state; diagnostics remain static and distinguish useful failure categories |
| T01 finite help | `help_is_finite_and_requires_no_paths_or_network`, `help_is_deliberate_and_standalone` | Help succeeds only as a standalone command and requires no selected data or peer |

### Process evidence, limits and excluded claims

Every new process case launches the actual executable. Listeners bind before the
child starts, so there is no released-port race or arbitrary readiness sleep.
Explicit channels hold a request or terminal End open until the parent has checked
ownership/deadline behavior. Child exit/readiness and synthetic peer waits are
bounded; children are killed and reaped on failure. The executable's tests share
one serialization gate to avoid unrelated inherited ownership descriptors.

The A2 recovery cases **construct canonical pending layouts** from accepted A1
records and storage APIs. They establish real command handling of those layouts,
not new publication-boundary termination evidence. A1's section 19 still supplies
the actual SIGKILL cases at six initialization and sixteen adoption boundaries,
plus interrupted recovery. A2 adds one actual killed child blocked in receive,
before candidate publication. Its deterministic broken-output case closes the
receiving stdout endpoint before child launch, observes the failed response after
successful init, then verifies clean status and released ownership.

Accepted command input is bounded to 32 arguments, 4,096 bytes each and 16,384
cumulative bytes. One sync uses exactly one F2 attempt, timeout 1–60,000 ms and
application I/O allowance 1–157,129,488 bytes. The deadline covers the peer attempt,
including End and EOF; initial local replay, durable adoption and output-channel
writes are outside it. Validation is synchronous, with late completion rejected,
not preempted. Actual-byte limits exclude local storage/RSS/kernel/TCP overhead.
Existing F1, A1 and store bounds remain independent and unchanged.

Status is read-only at the application level; the operating system may update
access time. Output is at most 4,096 bytes, but a blocked consumer has no output
deadline. Operation failure or output loss alone cannot establish what reached
durable storage. These cooperative Linux/process tests establish neither hostile
local-owner protection, global availability nor physical power-loss certification.

### Exact next component

**Next: MB-04B1 — bounded V2 provider lifecycle and shutdown; design first.**
The design must require explicit control/store paths, independent identity and
listen address; strictly clean A1 ownership before bind; and one fixed accepted
history for the provider lifetime under held control/store ownership. Concurrent
A2 invocations must observe busy ownership. Process-wide attempts, active
connections, aggregate actual bytes and lifecycle budgets must survive replacement
of a finite F2 session owner, with explicit bounded shutdown and tested release.
No auto initialization/recovery, reconnect policy, multi-peer scheduling, pool or
wallet is included. MB-04 remains IN_PROGRESS; the original 35 requirements and
Gate A/B remain broader than this finite command component.
