# Membership integration acceptance matrix

Design checkpoint MIRROR-MB01A-001; reviewed baseline db2861930cc0f01f735ccfba80a4f530f4d1ddd6.
Current implementation checkpoint: **MIRROR-MB01B-B3-001**.
These 35 rows are acceptance requirements, NOT 35 passing tests. Runtime integration
is still pending. B1 added 31 Rust tests, B2 added 39 and B3 added 51. Those counts
do not represent completed roadmap rows. B3 verifies in-memory state transitions;
V2 codecs/roots/replay and live ledger integration remain pending. Release executions
are reruns, not new unique tests.

Normative design: [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md).
Historical source findings: [THREAT_MODEL.md](THREAT_MODEL.md).
Exact byte contracts: [CONFIG_V2_ENCODING.md](CONFIG_V2_ENCODING.md) and
[OPERATIONS_V2_ENCODING.md](OPERATIONS_V2_ENCODING.md); frozen B4 state layout and
B3 execution rules: [MEMBERSHIP_STATE_V2.md](MEMBERSHIP_STATE_V2.md).

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
| T09/R09 | `anchor_must_be_recent_canonical_ancestor`: reject unknown/orphan/future/current anchors; exact max age pass/max+1 fail; validate against candidate branch ancestry. | B2 synthetic ancestry and B3 parent C/height/tip association VERIFIED SUBCASES; actual validated blocks/branch ancestry MB-02/03 |
| T10/R10 | `height_window_boundaries`: from/until inclusive; from-1/until+1, inverted/overflow span reject; max/max+1; local clock irrelevant. | B2 message windows and B3 per-operation rechecks VERIFIED; deterministic record expiry covered in T11 |
| T11/R11 | `pending_expiry_preserves_tombstones`: empty block expires pending at until+1, frees pending capacity not history; active membership unaffected by old request expiry; approval cannot extend lifetime. | B3 VERIFIED IN MEMORY; restart/replay preservation B4/MB-03 |
| T12/R12 | `only_self_can_withdraw`: pending/active own signed withdrawal succeeds; unknown/expired/revoked or other signer including founder fails; context/anchor/window failure consumes no sequence. | B3 ownership/state and per-operation authorization VERIFIED IN MEMORY |
| T13/R13 | `ordered_block_has_one_transition_result`: reject Approve before unknown Enroll; stage valid ordered pair; order committed, never decoder/executor sorted. | B3 ordered transition VERIFIED; committed operation order/root MB-02 |
| T14/R14 | `newly_activated_member_cannot_approve_in_same_block`: A->B then B->C in same block rejects; B may approve only after active in committed parent. | B3 VERIFIED IN MEMORY; trusted synthetic parent snapshots, not deployed membership consensus |
| T15/R15 | `same_block_withdrawal_excludes_earlier_approval`: withdrawn A's earlier approval cannot cause end activation; reverse order approval rejects; already-active targets not cascade-revoked. | B3 VERIFIED IN MEMORY, including retained earlier-block approvals and no cascade revocation |
| T16/R16 | `activation_order_is_canonical`: multiple requests activate by original canonical inclusion position identically on replicas; ordinal once, no reuse; overflow rejects whole batch. | B3 inclusion-order, ordinal preservation/overflow and identical replica state VERIFIED; committed roots B4 |
| T17/R17 | `record_counters_and_root_agree`: recompute active/pending/activations/indexes; change status/sequence/approval/counter/ordinal changes root or rejects invariant. | B3 derived counts/index/activation consistency VERIFIED SUBCASES; root tampering/codec checks B4 |
| T18/R18 | `high_water_uses_committed_branch_end_state`: decline preserves record; transient cancelled activation cannot fabricate peak; reorg discards orphan counts/peaks. | B3 final-state high-water and decline preservation VERIFIED; actual reorg/orphan counter recovery MB-03 |
| T19/R19 | `membership_batch_failure_is_atomic`: invalid final event after otherwise valid mutations/expiry leaves all records/indexes/sequences/roots unchanged for every error class. | B3 immutable staging/rejection/retry and representative late-error rollback VERIFIED; serialized roots B4, mixed ledger MB-02 |
| T20/R20 | `limits_apply_to_direct_execution`: exact vs +1 bytes/events/enrollments/pending/records/accounts over repeated batches; no apply bypass; bounds before large clones/allocations. | B1/B2 bounds plus B3 direct operation/block/pending/records/retained-approval limits VERIFIED; account/mixed budgets MB-02 |
| T21/R21 | `codec_and_storage_limits_agree`: malformed huge declared lengths/counts/files/frames reject before claimed allocation; encoding also bounded. | B1/B2 narrow codecs pass; B4/MB-03/04 full cases pending |
| T22/R22 | `capacity_does_not_erase_replay_protection`: expiry frees pending not total tombstones; total cap still allows withdrawal but no Enroll; restart preserves used keys/evidence. | B3 in-memory tombstone/capacity/withdrawal behavior VERIFIED; restart and encoded replay B4/MB-03 |
| T23/R23 | `mrb2_header_and_hash_vectors`: header160/empty168, independent vectors/round-trip/exact genesis; changed header fields/context/height/parent/target/roots reject. | MB-02 PLANNED |
| T24/R24 | `operation_count_and_order_are_committed`: [A,B,C] vs [A,B,C,C] distinct despite odd-leaf duplication; permutations/signatures change root; fixed empty vector. | MB-02 PLANNED |
| T25/R25 | `ledger_root_binds_all_components`: isolated changes in accounts/membership/height/C/test-supply change root; correct accounts cannot hide bad membership. | MB-02 PLANNED |
| T26/R26 | `canonical_replay_rebuilds_derived_indexes`: independent replay matches roots/IDs/indexes/sequences; old/tampered snapshots not trusted authoritative latest state. | MB-02/03 PLANNED |
| T27/R27 | `mixed_block_is_failure_atomic`: valid money then bad membership and reverse leave BOTH unchanged; wrong final root after valid operations changes nothing. | MB-02 PLANNED |
| T28/R28 | `restart_pins_config_genesis_and_tip`: different valid genesis nonce/wrong bootstrap/old valid log rejected against trusted manifest/tip; replay requires no matcher/device. | MB-02/03 PLANNED |
| T29/R29 | `crash_boundary_never_mixes_states`: inject every process/I/O publication boundary; reopen complete identifiable old/new state or fail; ambiguous I/O never blindly resumes. | MB-03 PLANNED |
| T30/R30 | `second_writer_and_oversize_file_rejected`: lock concurrent writers; reject corrupt/unexpected metadata/oversized input; status read-only does not initialize/mine. | MB-03/04 PLANNED |
| T31/R31 | `network_versions_do_not_fallback`: MIR2/version2/C/genesis checked before exchange; reject legacy/wrong context, no downgrade; reject unknown kind/length before body allocation. | MB-03/04 PLANNED |
| T32/R32 | `peer_claims_and_partial_messages_are_bounded`: absurd advertised height/count, wrong response, slow/truncated frames and endless branches hit total byte/time bounds; claims never replace execution. | MB-04 PLANNED |
| T33/R33 | `public_records_exclude_biometric_archives`: canary payloads/keys absent from blocks/logs/artifacts; no archive/URL schema; do not claim fresh-key/fresh-evidence biological deduplication. | B4/MB-08/09 PLANNED; fixed B2 fields not a privacy proof |
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
records the later B3 stateful evidence; B4, MB-02/03 and Gate A/B remain open.

## 5. B3 exact implementation evidence and remaining coverage

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

**Next B4:** implement the frozen MPS2 canonical codec/root and grouped-block MPH2
replay with independent preimages/vectors and adversarial decoding/replay cases.
Empty blocks, expiry, canonical inclusion positions and original operation order
must survive replay. Snapshot validity needs trusted replay/root association, not
only internal accounting or individual signatures. No V2 root/replay, live ledger,
monetary integration, device readiness or complete 35-row acceptance is claimed.
