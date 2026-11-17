# Membership integration acceptance matrix

Design checkpoint: MIRROR-MB01A-001, reviewed baseline `db2861930cc0f01f735ccfba80a4f530f4d1ddd6`.
Current implementation checkpoint: **MIRROR-MB01B-B1-001**.

The 35 rows are acceptance specifications, NOT 35 passing tests. Current coverage:
**T02 configuration/bootstrap binding VERIFIED; T03 and T20 PARTIAL; other rows
remain PLANNED.** Narrow supporting checks do not complete a larger acceptance case.
The 31 B1 Rust tests are separate test functions, not 31 completed roadmap rows.

B1 implementation/tested commit: `809661b5e6a9338d65c15c45e540289df24fcc99`.
CI: https://github.com/Ahmed-Sobeih/mirror/actions/runs/44201393943 — success.
Actual tests: [config_v2.rs](../crates/mirror-personhood/tests/config_v2.rs).
Encoding/reference: [CONFIG_V2_ENCODING.md](CONFIG_V2_ENCODING.md).
Normative design: [LEDGER_MEMBERSHIP_V1.md](LEDGER_MEMBERSHIP_V1.md).
Source gaps/residual assumptions: [THREAT_MODEL.md](THREAT_MODEL.md).

## 1. Rule-to-assertion map

| Rule / case | Proposed test(s) and concrete assertion | Owner |
|---|---|---|
| R01 / T01 | `legacy_data_is_not_reinterpreted`: MRB1/MPH1 accepted only on their old paths; V2 rejects them; initialization refuses legacy directories; fixture bytes remain identical. | MB-01B/02 |
| R02 / T02 | `seed_configuration_changes_signing_context`: same network and threshold with changed seed key, order, nonce, evidence or allocation yields a different C; signatures from original config fail. Vary one field at a time, with both configs otherwise valid. | B1 — VERIFIED, section 4 |
| R03 / T03 | `bootstrap_has_no_circular_context_or_bypass`: compute C without proofs/roots, verify ordered seed proofs, reject duplicate/zero descriptors and invalid threshold/count; first seed cannot approve alone under threshold two or revoke another member. | B1 construction VERIFIED; B3 runtime still PLANNED |
| R04 / T04 | `v2_operations_have_exact_lengths`: prefix=131; signed Transfer=243, Enroll=291, Approve/Withdraw=227; reject every truncation, wrong version/kind, oversized/trailing input before mutation. | B2/MB-02 |
| R05 / T05 | `every_signed_field_is_bound`: mutate kind, context, anchor, height window, key, sequence, evidence, profile or target without resigning; strict signature check fails. Invalid public keys/signatures never panic. | B2 |
| R06 / T06 | `approval_targets_complete_request`: different enrollment body/window produces another enrollment ID; an approval cannot authorize the changed request; exact request/key/evidence replay is rejected after active/revoked/expired outcomes. | B2/B3 |
| R07 / T07 | `membership_sequences_are_exact_and_atomic`: seeds start at zero; accepted Enroll establishes next=1; correct sequence increments once; reused/skipped/overflow sequence fails unchanged. Transfer sequence and membership sequence are independent. Repeated same-verifier approval never adds weight. | B2/B3 |
| R08 / T08 | `v2_transfer_conserves_test_balances`: amount>0, checked balances, self-transfer and nonce behavior; nonzero fees and all mint/reward operations reject in the integration profile; sum balances remains initial synthetic supply. | MB-02 |
| R09 / T09 | `anchor_must_be_recent_canonical_ancestor`: unknown/orphan/future/current-height anchors fail; exact max age succeeds and max+1 fails; candidate branch validation uses its own ancestry. | B2/MB-02 |
| R10 / T10 | `height_window_boundaries`: valid_from and valid_until inclusive; from-1/until+1, inverted window and arithmetic overflow reject; span=max succeeds, span=max+1 fails; local clock changes do not change result. | B2 |
| R11 / T11 | `pending_expiry_preserves_tombstones`: empty block at until+1 expires pending entry and frees pending capacity, not key/evidence history; active member does not expire with its old application; approval expiry cannot extend request lifetime. | B3 |
| R12 / T12 | `only_self_can_withdraw`: pending/active self-signed withdrawal succeeds; unknown/expired/revoked target and another signer including first seed fail; wrong context/anchor/window fail without consuming sequence. | B2/B3 |
| R13 / T13 | `ordered_block_has_one_transition_result`: Approve before unknown Enroll rejects, ordered Enroll then valid Approve is evaluated under staging rules; order is committed, never silently sorted by decoder/executor. | B3/MB-02 |
| R14 / T14 | `newly_activated_member_cannot_approve_in_same_block`: threshold-one seed A, Enroll B, approval A->B, Enroll C, approval B->C must reject; after B is active in a committed parent, B may approve a later request under the same dev rules. | B3 |
| R15 / T15 | `same_block_withdrawal_excludes_earlier_approval`: A approves target then withdraws; its signature is valid but does not count at block-end activation. With approval/withdraw ordering reversed the approval is rejected. An older already-active target is not cascaded-revoked. | B3 |
| R16 / T16 | `activation_order_is_canonical`: multiple qualifying applications finalize in canonical enrollment-position order on two replicas; ordinal assigned once, no repeat on later approval/withdrawal; overflow rejects entire batch. | B3 |
| R17 / T17 | `record_counters_and_root_agree`: recompute active/pending/total activations, compare indexed state, mutate a status/sequence/activation/approval entry/counter and require a different root or invariant rejection. | B3 |
| R18 / T18 | `high_water_uses_committed_branch_end_state`: normal active decline does not reduce record; pending/cancelled same-block activation attempts cannot manufacture a peak; reorg drops orphaned counts/peaks and preserves the new branch's own history. | B3/MB-03 |
| R19 / T19 | `membership_batch_failure_is_atomic`: place invalid event last after valid enrollment/approval/withdrawal/expiry work; entire pre-state, indexes, sequences and roots stay byte-equivalent; test each error class. | B3 |
| R20 / T20 | `limits_apply_to_direct_execution`: exact block/event/new-enrollment/pending/total/account bounds versus bound+1, including repeated batches; no unbounded direct apply bypass; reject before large allocation or cloning attacker data. | B1 config VERIFIED; B3/MB-02 execution still PLANNED |
| R21 / T21 | `codec_and_storage_limits_agree`: malformed declared sizes/counts, maximum integers and resource-exhausting files/frames rejected at each ingress before allocating the claimed size; encoding rejects oversize too. | B4/MB-03/04 |
| R22 / T22 | `capacity_does_not_erase_replay_protection`: expiry frees pending quota but not total tombstones; at total cap, existing-member withdrawal remains possible but another Enroll fails; restarting does not clear used keys/evidence. | B3 |
| R23 / T23 | `mrb2_header_and_hash_vectors`: header=160 bytes, empty block=168; independent canonical vectors, byte round-trip and explicit genesis hash; alter version/context/height/parent/target/roots and require rejection. | MB-02 |
| R24 / T24 | `operation_count_and_order_are_committed`: odd-leaf duplication cannot make [A,B,C] and [A,B,C,C] share operations_root; permutations and changed signatures change it; zero operations has a fixed vector. | MB-02 |
| R25 / T25 | `ledger_root_binds_all_components`: change only accounts, membership, height, C or disabled-economics supply and root changes; a correct account root cannot hide incorrect membership or counters. | MB-02 |
| R26 / T26 | `canonical_replay_rebuilds_derived_indexes`: independent node replays all V2 blocks from trusted config and obtains identical roots, member IDs, used-key/evidence indexes and sequences; tampered/older snapshot is not accepted as authoritative latest state. | MB-02/03 |
| R27 / T27 | `mixed_block_is_failure_atomic`: valid transfer followed by invalid membership, and valid membership followed by invalid transfer, leave BOTH components unchanged; wrong final root after otherwise valid operations also changes nothing. | MB-02 |
| R28 / T28 | `restart_pins_config_genesis_and_tip`: different valid genesis nonce, wrong bootstrap config or valid old log fail trusted manifest/tip checks; replay need not contact a matcher or query a device. | MB-02/03 |
| R29 / T29 | `crash_boundary_never_mixes_states`: inject process/I/O failure before/after every publication boundary; reopening gives identifiable complete old or new blocks/accounts/membership/counters, or explicit failure; ambiguous I/O never resumes writes blindly. | MB-03 |
| R30 / T30 | `second_writer_and_oversize_file_rejected`: concurrent open-for-write cannot bypass lock; corrupted/unexpected metadata or oversized block input rejects; read-only status does not initialize/mine or mutate state. | MB-03/04 |
| R31 / T31 | `network_versions_do_not_fallback`: MIR2/version2/config/genesis checked before exchange; legacy frames and wrong-context V2 peers reject rather than downgrade; unknown kind/length fails before body allocation. | MB-03/04 |
| R32 / T32 | `peer_claims_and_partial_messages_are_bounded`: absurd advertised height/count, wrong requested block, truncated/slow frames and endless branch responses hit total byte/time budgets; no reported population/root replaces locally executed state. | MB-04 |
| R33 / T33 | `public_records_exclude_biometric_archives`: synthetic canary recordings/keys never appear in public blocks, diagnostics or CI artifacts; no archive/URL field in public schema. Also assert fresh-key/new-evidence enrollment is NOT claimed as biological deduplication. | B4/MB-08/09 |
| R34 / T34 | `integration_cannot_mint_privileged_funds`: unknown mint/claim kinds reject; no founder/marketing key bypass; legacy Alice fixture is untouched; membership activation changes no balances or spendable reserve. | MB-02/07 |
| R35 / T35 | `future_policy_is_not_silently_enabled`: combined bootstrap ceiling and real reward distribution remain disabled until D-01/D-04; synthetic admissions do not satisfy D-02/D-05/D-06; legacy reorg cannot override future finalized checkpoints (last assertion belongs to MB-06). | MB-02/06/07 |

## 2. Boundary and adversarial execution requirements

Use synthetic keys/commitments only. For every rejection case retain the complete
pre-state and assert equality after failure, not merely an error return. Test
mutations both before and after serialization. Fuzz bounded decoders separately
from authorization. Test exact limits and one beyond; use checked arithmetic
around zero/u64::MAX; never allocate attacker-declared huge vectors in the test
itself to exercise a declared-length error.

Independent fixed vectors must be computed by a separate small reference method,
with inputs/preimage bytes recorded. Do not copy a failing implementation's output
into its expected constant. Debug and release modes must agree. Do not count a
release rerun as a new unique test.

A direct library call, disk replay, block receive, local proposal and candidate
reorg must share validation rules. At least two independent state instances and
later two independent node processes must reproduce the same roots. Membership
fork tests use genuinely different enrollment/withdrawal histories, not only
changed timestamps.

V2 admission and limits may reject operations valid in a V1 demo. This is why
new contexts/encodings are mandatory. Preserve the V1 vectors/tests; do not edit
expected V1 hashes merely to conceal a semantic change.

## 3. Existing evidence and its limits

The baseline suite at the audited source was 133 tests. Its 26 V1 personhood tests
remain unchanged and passing. They cover exact request/key/evidence reuse,
signature tampering, network/threshold separation, self/duplicate/inactive approval
rejection, voluntary self-withdrawal, batch atomicity, roots and log replay.
They are NOT proof of V2 temporal/ledger behavior. All later non-B1 requirements
above remain planned unless a future recorded tested commit advances them.

## 4. B1 implementation coverage ledger

Every name below refers to the executable integration test file
`crates/mirror-personhood/tests/config_v2.rs` at commit
`809661b5e6a9338d65c15c45e540289df24fcc99`, tested in debug/release by
https://github.com/Ahmed-Sobeih/mirror/actions/runs/44201393943 (job 134456099484).

| Case | Actual tests / scope | Status |
|---|---|---|
| T02 | `every_scalar_configuration_setting_changes_context`, `seed_key_nonce_evidence_and_order_are_bound`, `allocation_address_balance_and_zero_entry_are_bound`, `each_resource_limit_changes_context_and_invalidates_old_proofs` | VERIFIED for config/seed proof context; runtime operation integration follows B2 |
| T03 construction | `config_and_seed_hashes_match_independent_python_vectors`, `no_circular_context_or_caller_supplied_context`, `every_seed_proof_is_required_and_seed_indices_are_not_privileges`, `forged_last_proof_returns_no_partially_verified_bootstrap`, `malformed_or_weak_seed_keys_cannot_become_verified_bootstrap`, duplicate/zero/threshold tests | VERIFIED subcase; runtime approve/withdraw privilege tests are B3, so T03 overall PARTIAL |
| T20 configuration | `limit_ceilings_and_zero_values_are_rejected`, `inconsistent_limits_cannot_be_used_directly`, `smallest_consistent_limits_and_largest_timestamp_round_trip`, `direct_constructor_rejects_count_over_limits_before_copying`, `maximum_config_counts_fit_and_one_more_is_rejected` | VERIFIED subcase; runtime budgets not yet enforced, so T20 overall PARTIAL |
| Supporting format/replay checks | `canonical_round_trip_recomputes_context_and_verifies_proofs`, `all_truncations_and_trailing_bytes_are_rejected`, `declared_huge_counts_are_rejected_before_reserving_memory`, `legacy_unknown_magic_and_versions_never_fall_back`, `decoding_rechecks_descriptor_and_allocation_semantics`, `valid_legacy_enrollment_signatures_are_not_v2_seed_proofs` | PASS only for configuration/seed boundary, not full block/storage/network cases |

31 new tests passed, all 133 prior tests retained: 164 workspace total. Release
personhood reruns: 26 V1 + 31 B1 = 57, not extra unique cases. Independent Python
fixture verification passed too. No V2 block or runtime registry is claimed.
Next: **MB-01B/B2**, then B3/B4 and the separate MB-02/03 integration gates.
