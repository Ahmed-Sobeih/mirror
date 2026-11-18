# Membership integration acceptance matrix

Design checkpoint MIRROR-MB01A-001; reviewed baseline db2861930cc0f01f735ccfba80a4f530f4d1ddd6.
Current implementation checkpoint: **MIRROR-MB01B-B4-001**.
These 35 rows are acceptance requirements, NOT 35 passing tests. Runtime ledger
integration is still pending. B1 added 31 Rust tests, B2 added 39 and B3 added 51.
B4 adds canonical MPS2 commitments, untrusted cache decoding and bounded MPH2
replay through the B3 executor. Its suites have 26 snapshot cases, 10 independent
vector cases, 31 replay cases and two private work-budget cases; a compile-fail
documentation test separately protects the snapshot-to-authority boundary. Counts do not
represent completed roadmap rows. Exact source/test/CI acceptance is recorded in
CURRENT_STATUS.md. Release executions are reruns, not new unique tests.

**B4 and the MB-01B standalone component are VERIFIED.** Next is MB-02A:
TransferV2/unified operations and BlockV2 encoding, commitments and independent
vectors. Combined-ledger execution, actual block authority and durable integration
remain MB-02B/C and MB-03. Section 6 records exact B4 source and acceptance evidence.

Normative design: [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md).
Historical source findings: [THREAT_MODEL.md](THREAT_MODEL.md).
Exact byte contracts: [CONFIG_V2_ENCODING.md](CONFIG_V2_ENCODING.md) and
[OPERATIONS_V2_ENCODING.md](OPERATIONS_V2_ENCODING.md); frozen B4 state layout and
B3 execution rules: [MEMBERSHIP_STATE_V2.md](MEMBERSHIP_STATE_V2.md).
Grouped replay: [REPLAY_V2.md](REPLAY_V2.md).

## 1. Rule-to-assertion map

| Case | Required assertion / proposed acceptance name | Owner and current coverage |
|---|---|---|
| T01/R01 | `legacy_data_is_not_reinterpreted`: old MRB1/MPH1 only on old paths; V2 rejects; init refuses legacy directories; old bytes unchanged. | B1/B2 decoder subcases pass; full MB-02 acceptance PLANNED |
| T02/R02 | `seed_configuration_changes_signing_context`: vary seed key/order/nonce/evidence, allocation and every setting separately; old signatures fail on otherwise-valid changed configs. | B1 config/seed VERIFIED; B2 context-bound messages tested; live use MB-02 |
| T03/R03 | `bootstrap_has_no_circular_context_or_bypass`: compute C before proofs/roots, all ordered proofs mandatory; reject zero/duplicate descriptors and invalid counts/thresholds; no first-seed runtime bypass or forced revocation. | B1 construction and B3 ordinary seed runtime rights VERIFIED; production bootstrap policy remains gated |
| T04/R04 | `v2_operations_have_exact_lengths`: prefix131; signed Transfer243/Enroll291/Approve227/Withdraw227; reject all truncations, versions/kinds, oversized/trailing bytes before mutation. | B2 membership VERIFIED SUBCASES; Transfer/MB-02 pending |
| T05/R05 | `every_signed_field_is_bound`: altered kind/context/anchor/window/key/sequence/evidence/profile/target invalidates signature; malformed keys/signatures do not panic. | B2 membership VERIFIED; Transfer follows MB-02 |
| T06/R06 | `approval_targets_complete_request`: changed request/window changes enrollment ID; approval cannot authorize it; exact request/key/evidence reuse rejected after active/revoked/expired outcomes. | B2 complete-request ID/signatures and B3 retained ID/key/evidence reuse rejection VERIFIED; fresh biometric duplicates remain external |
| T07/R07 | `membership_sequences_are_exact_and_atomic`: seeds start0, Enroll establishes next1; accept/increment exact sequence once; reuse/skip/overflow fail unchanged; transfer/membership spaces separate; repeated approval no extra weight. | B3 exact membership sequences, rollback, duplicate-review and checked overflow VERIFIED; separate Transfer sequence space MB-02 |
| T08/R08 | `v2_transfer_conserves_test_balances`: positive amounts, checked balance/self-transfer/nonce; reject nonzero fees/mints/rewards; preserve initial synthetic supply. | MB-02 PLANNED |
| T09/R09 | `anchor_must_be_recent_canonical_ancestor`: reject unknown/orphan/future/current anchors; exact max age pass/max+1 fail; validate against candidate branch ancestry. | B2/B3 synthetic ancestry plus B4 cached trusted hash/group/root and replay anchor checks VERIFIED COMPONENTS; actual validated blocks/branch ancestry MB-02/03 |
| T10/R10 | `height_window_boundaries`: from/until inclusive; from-1/until+1, inverted/overflow span reject; max/max+1; local clock irrelevant. | B2 message windows and B3 per-operation rechecks VERIFIED; deterministic record expiry covered in T11 |
| T11/R11 | `pending_expiry_preserves_tombstones`: empty block expires pending at until+1, frees pending capacity not history; active membership unaffected by old request expiry; approval cannot extend lifetime. | B3 execution and B4 empty-group expiry/tombstone reconstruction VERIFIED COMPONENTS; durable restart MB-03 |
| T12/R12 | `only_self_can_withdraw`: pending/active own signed withdrawal succeeds; unknown/expired/revoked or other signer including founder fails; context/anchor/window failure consumes no sequence. | B3 ownership/state and per-operation authorization VERIFIED IN MEMORY |
| T13/R13 | `ordered_block_has_one_transition_result`: reject Approve before unknown Enroll; stage valid ordered pair; order committed, never decoder/executor sorted. | B3 ordered transition and B4 exact diagnostic group commitment VERIFIED COMPONENTS, including commuting-operation changes; actual block operation root MB-02 |
| T14/R14 | `newly_activated_member_cannot_approve_in_same_block`: A->B then B->C in same block rejects; B may approve only after active in committed parent. | B3 VERIFIED IN MEMORY; trusted synthetic parent snapshots, not deployed membership consensus |
| T15/R15 | `same_block_withdrawal_excludes_earlier_approval`: withdrawn A's earlier approval cannot cause end activation; reverse order approval rejects; already-active targets not cascade-revoked. | B3 VERIFIED IN MEMORY, including retained earlier-block approvals and no cascade revocation |
| T16/R16 | `activation_order_is_canonical`: multiple requests activate by original canonical inclusion position identically on replicas; ordinal once, no reuse; overflow rejects whole batch. | B3 inclusion-order/ordinal preservation/overflow and B4 independent roots/replay VERIFIED COMPONENTS |
| T17/R17 | `record_counters_and_root_agree`: recompute active/pending/activations/indexes; change status/sequence/approval/counter/ordinal changes root or rejects invariant. | B3 derived consistency plus B4 canonical roots, snapshot metadata/sequence/counter checks and independent vectors VERIFIED COMPONENTS; snapshot remains untrusted |
| T18/R18 | `high_water_uses_committed_branch_end_state`: decline preserves record; transient cancelled activation cannot fabricate peak; reorg discards orphan counts/peaks. | B3 final-state high-water and decline preservation VERIFIED; actual reorg/orphan counter recovery MB-03 |
| T19/R19 | `membership_batch_failure_is_atomic`: invalid final event after otherwise valid mutations/expiry leaves all records/indexes/sequences/roots unchanged for every error class. | B3 immutable staging plus B4 late replay rejection/retry and serialized-state equality VERIFIED COMPONENTS; mixed ledger MB-02 |
| T20/R20 | `limits_apply_to_direct_execution`: exact vs +1 bytes/events/enrollments/pending/records/accounts over repeated batches; no apply bypass; bounds before large clones/allocations. | B1/B2/B3 bounds and B4 direct/decoded group, snapshot/log and cumulative work budgets VERIFIED COMPONENTS; account/mixed budgets MB-02 |
| T21/R21 | `codec_and_storage_limits_agree`: malformed huge declared lengths/counts/files/frames reject before claimed allocation; encoding also bounded. | B1/B2 narrow codecs and B4 MPS2/MPH2 huge-count/length/byte bounds VERIFIED COMPONENTS; disk/frame/streaming cases MB-03/04 |
| T22/R22 | `capacity_does_not_erase_replay_protection`: expiry frees pending not total tombstones; total cap still allows withdrawal but no Enroll; restart preserves used keys/evidence. | B3 tombstone/capacity behavior and B4 exact encoded replay reservations/indexes VERIFIED COMPONENTS; durable restart MB-03 |
| T23/R23 | `mrb2_header_and_hash_vectors`: header160/empty168, independent vectors/round-trip/exact genesis; changed header fields/context/height/parent/target/roots reject. | MB-02 PLANNED |
| T24/R24 | `operation_count_and_order_are_committed`: [A,B,C] vs [A,B,C,C] distinct despite odd-leaf duplication; permutations/signatures change root; fixed empty vector. | MB-02 PLANNED |
| T25/R25 | `ledger_root_binds_all_components`: isolated changes in accounts/membership/height/C/test-supply change root; correct accounts cannot hide bad membership. | MB-02 PLANNED |
| T26/R26 | `canonical_replay_rebuilds_derived_indexes`: independent replay matches roots/IDs/indexes/sequences; old/tampered snapshots not trusted authoritative latest state. | B4 membership-only replay matches full state and trusted expectations; actual combined-ledger/canonical recovery MB-02/03 |
| T27/R27 | `mixed_block_is_failure_atomic`: valid money then bad membership and reverse leave BOTH unchanged; wrong final root after valid operations changes nothing. | MB-02 PLANNED |
| T28/R28 | `restart_pins_config_genesis_and_tip`: different valid genesis nonce/wrong bootstrap/old valid log rejected against trusted manifest/tip; replay requires no matcher/device. | B4 trusted synthetic config/genesis/height/tip/root expectations reject old logs; real MRB2 pinning and durable latestness MB-02/03 |
| T29/R29 | `crash_boundary_never_mixes_states`: inject every process/I/O publication boundary; reopen complete identifiable old/new state or fail; ambiguous I/O never blindly resumes. | MB-03 PLANNED |
| T30/R30 | `second_writer_and_oversize_file_rejected`: lock concurrent writers; reject corrupt/unexpected metadata/oversized input; status read-only does not initialize/mine. | MB-03/04 PLANNED |
| T31/R31 | `network_versions_do_not_fallback`: MIR2/version2/C/genesis checked before exchange; reject legacy/wrong context, no downgrade; reject unknown kind/length before body allocation. | MB-03/04 PLANNED |
| T32/R32 | `peer_claims_and_partial_messages_are_bounded`: absurd advertised height/count, wrong response, slow/truncated frames and endless branches hit total byte/time bounds; claims never replace execution. | MB-04 PLANNED |
| T33/R33 | `public_records_exclude_biometric_archives`: canary payloads/keys absent from blocks/logs/artifacts; no archive/URL schema; do not claim fresh-key/fresh-evidence biological deduplication. | B4 has only fixed signed references, no archive/URL schema; real custody/canary/privacy acceptance MB-08/09, fixed fields not a privacy proof |
| T34/R34 | `integration_cannot_mint_privileged_funds`: unknown mint/claim rejects, no founder/marketing bypass, old Alice fixture intact, activation cannot change money/reserve. | B3 no money API/no seed monetary privilege; actual integrated mint/grant rejection MB-02/07 |
| T35/R35 | `future_policy_is_not_silently_enabled`: minting/rewards blocked on D-01/D-04; synthetic claims cannot satisfy real-admission gates; old reorg cannot bypass finality. | MB-02/06/07 PLANNED |

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
B2 checkpoint these tasks remained open; MB-02/03 and Gate A/B are still unfinished.

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


## 6. B4 commitment, cache and grouped-replay component coverage

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
| T11/T16/T17/T22 full replay | `replay_v2.rs`: `replay_recovers_full_lifecycle_records_indexes_sequences_and_counters`, `replay_restores_tombstone_duplicate_protection_and_remains_executable`, `empty_groups_preserve_inclusive_expiry_and_are_not_omitted` | Full state/index/approval/ordinal/high-water equality and executable duplicate protection after replay, preserving empty block expiry; no durable restart or real-chain reorg claim |
| T13 exact diagnostic operations | `exact_group_commitment_binds_context_height_count_order_and_signed_bytes`, `reordered_commuting_approvals_reject_even_when_membership_root_is_identical`, `alternate_valid_withdrawal_bytes_cannot_replace_the_canonical_operation` | State roots alone cannot distinguish all histories; separate trusted group commitment binds exact operation bytes/order; future MRB2 Merkle root remains MB-02 |
| T09/T26/T28 trusted provenance | `genesis_hash_root_and_empty_group_commitment_are_independently_required`, `missing_trusted_metadata_rejects_genesis_intermediate_or_final_height`, `each_trusted_intermediate_hash_root_and_exact_group_are_checked`, `explicit_expected_context_height_tip_and_root_are_all_required`, `an_older_valid_prefix_cannot_satisfy_the_current_trusted_expectation` | Synthetic independently executed per-height metadata and externally selected final C/height/tip/root mandatory; provider is a trusted caller boundary, not implemented live consensus or durable latestness discovery |
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

MB-01B is complete in this scope. Next MB-02A covers the unified operation and
BlockV2 bytes/commitments, with independently calculated vectors. Full T23–T28
ledger/chain authority and mixed execution remain MB-02B/C or MB-03 as applicable;
passing the 69 B4 cases does not mark all acceptance rows complete.
