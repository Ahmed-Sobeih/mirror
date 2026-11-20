use super::*;

// Synthetic fixtures only. These bytes are NOT biometric observations.
fn key(number: u8) -> Keypair { Keypair::from_secret_bytes([number; 32]) }
fn rules(threshold: u16) -> RegistryRules {
    RegistryRules::new(sha256d(b"mirror-personhood-development-network"), threshold).unwrap()
}
fn enrollment(rules: &RegistryRules, number: u8) -> Enrollment {
    Enrollment::sign(rules, &key(number), sha256d(&[number, 1]), sha256d(&[number, 2])).unwrap()
}
fn setup(threshold: u16) -> (RegistryRules, Vec<Enrollment>, MembershipRegistry) {
    let rules = rules(threshold);
    let seeds: Vec<_> = (1..=3).map(|number| enrollment(&rules, number)).collect();
    let registry = MembershipRegistry::new(rules.clone(), &seeds).unwrap();
    (rules, seeds, registry)
}
fn approval(rules: &RegistryRules, applicant: MemberId, verifier: &Enrollment, number: u8) -> MembershipEvent {
    MembershipEvent::Approve(Approval::sign(rules, applicant, verifier.member_id(), &key(number)))
}
fn assert_unchanged(registry: &mut MembershipRegistry, event: MembershipEvent, error: RegistryError) {
    let before = registry.clone();
    assert_eq!(registry.apply(&event), Err(error));
    assert_eq!(*registry, before);
    assert_eq!(registry.state_root(), before.state_root());
}

#[test]
fn bootstrap_has_ordinary_members_and_no_currency_state() {
    let (rules, seeds, mut registry) = setup(2);
    assert_eq!(registry.active_count(), 3);
    assert_eq!(registry.population_high_water(), 3);
    assert_eq!(registry.member(seeds[0].member_id()).unwrap().activation_order(), Some(1));
    let applicant = enrollment(&rules, 4);
    registry.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    registry.apply(&approval(&rules, applicant.member_id(), &seeds[0], 1)).unwrap();
    // Being the first seed does not bypass the configured threshold.
    assert_eq!(registry.member(applicant.member_id()).unwrap().status(), MembershipStatus::Pending);
}

#[test]
fn distinct_active_approvals_activate_exactly_once() {
    let (rules, seeds, mut registry) = setup(2);
    let applicant = enrollment(&rules, 4);
    let id = applicant.member_id();
    registry.apply(&MembershipEvent::Enroll(applicant)).unwrap();
    assert_eq!(registry.active_count(), 3);
    registry.apply(&approval(&rules, id, &seeds[0], 1)).unwrap();
    registry.apply(&approval(&rules, id, &seeds[1], 2)).unwrap();
    assert_eq!(registry.member(id).unwrap().status(), MembershipStatus::Active);
    assert_eq!(registry.member(id).unwrap().activation_order(), Some(4));
    assert_eq!(registry.active_count(), 4);
    assert_unchanged(&mut registry, approval(&rules, id, &seeds[2], 3), RegistryError::NotPending);
}

#[test]
fn replayed_enrollment_is_rejected() {
    let (rules, _, mut registry) = setup(2);
    let event = MembershipEvent::Enroll(enrollment(&rules, 4));
    registry.apply(&event).unwrap();
    assert_unchanged(&mut registry, event, RegistryError::DuplicateEnrollment);
}

#[test]
fn repeated_public_key_cannot_create_another_membership() {
    let (rules, _, mut registry) = setup(2);
    let duplicate = Enrollment::sign(&rules, &key(1), sha256d(b"new nonce"), sha256d(b"new evidence")).unwrap();
    assert_unchanged(&mut registry, MembershipEvent::Enroll(duplicate), RegistryError::PublicKeyAlreadyRegistered);
}

#[test]
fn identical_evidence_commitment_cannot_be_reused_with_another_key() {
    let (rules, seeds, mut registry) = setup(2);
    let duplicate = Enrollment::sign(&rules, &key(4), sha256d(b"new nonce"), seeds[0].evidence).unwrap();
    assert_unchanged(&mut registry, MembershipEvent::Enroll(duplicate), RegistryError::EvidenceAlreadyUsed);
}

#[test]
fn tampered_enrollment_fails_signature_verification() {
    let (rules, _, mut registry) = setup(2);
    let mut forged = enrollment(&rules, 4);
    forged.evidence = sha256d(b"changed evidence");
    let before = registry.clone();
    assert!(matches!(registry.apply(&MembershipEvent::Enroll(forged)), Err(RegistryError::Crypto(_))));
    assert_eq!(registry, before);
}

#[test]
fn signature_cannot_be_replayed_on_another_network_or_threshold() {
    let (rules, seeds, _) = setup(2);
    let other = RegistryRules::new(sha256d(b"other-network"), 2).unwrap();
    assert_eq!(MembershipRegistry::new(other, &seeds).unwrap_err(), RegistryError::WrongContext);
    let changed_threshold = RegistryRules::new(sha256d(b"mirror-personhood-development-network"), 1).unwrap();
    assert_ne!(changed_threshold.context(), rules.context());
    assert_eq!(MembershipRegistry::new(changed_threshold, &seeds).unwrap_err(), RegistryError::WrongContext);
}

#[test]
fn repeated_approval_does_not_add_weight() {
    let (rules, seeds, mut registry) = setup(2);
    let applicant = enrollment(&rules, 4);
    let vote = approval(&rules, applicant.member_id(), &seeds[0], 1);
    registry.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    registry.apply(&vote).unwrap();
    assert_unchanged(&mut registry, vote, RegistryError::DuplicateApproval);
    assert_eq!(registry.member(applicant.member_id()).unwrap().approval_count(), 1);
}

#[test]
fn applicant_cannot_approve_self() {
    let (rules, _, mut registry) = setup(1);
    let applicant = enrollment(&rules, 4);
    registry.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    assert_unchanged(&mut registry, approval(&rules, applicant.member_id(), &applicant, 4), RegistryError::SelfApproval);
}

#[test]
fn unknown_or_pending_verifier_cannot_approve() {
    let (rules, _, mut registry) = setup(1);
    let applicant = enrollment(&rules, 4);
    let verifier = enrollment(&rules, 5);
    registry.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    let vote = approval(&rules, applicant.member_id(), &verifier, 5);
    assert_unchanged(&mut registry, vote.clone(), RegistryError::UnknownMember);
    registry.apply(&MembershipEvent::Enroll(verifier)).unwrap();
    assert_unchanged(&mut registry, vote, RegistryError::InactiveVerifier);
}

#[test]
fn forged_approval_cannot_impersonate_an_active_member() {
    let (rules, seeds, mut registry) = setup(1);
    let applicant = enrollment(&rules, 4);
    registry.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    let before = registry.clone();
    let forged = approval(&rules, applicant.member_id(), &seeds[0], 9);
    assert!(matches!(registry.apply(&forged), Err(RegistryError::Crypto(_))));
    assert_eq!(registry, before);
}

#[test]
fn approval_is_bound_to_the_exact_applicant_request() {
    let (rules, seeds, mut registry) = setup(1);
    let first = enrollment(&rules, 4);
    let second = enrollment(&rules, 5);
    registry.apply(&MembershipEvent::Enroll(first.clone())).unwrap();
    registry.apply(&MembershipEvent::Enroll(second.clone())).unwrap();
    let mut forged = Approval::sign(&rules, first.member_id(), seeds[0].member_id(), &key(1));
    forged.applicant = second.member_id();
    let before = registry.clone();
    assert!(matches!(registry.apply(&MembershipEvent::Approve(forged)), Err(RegistryError::Crypto(_))));
    assert_eq!(registry, before);
}

#[test]
fn withdrawn_members_cannot_approve_and_old_approvals_stop_counting() {
    let (rules, seeds, mut registry) = setup(2);
    let applicant = enrollment(&rules, 4);
    let id = applicant.member_id();
    registry.apply(&MembershipEvent::Enroll(applicant)).unwrap();
    registry.apply(&approval(&rules, id, &seeds[0], 1)).unwrap();
    registry.apply(&MembershipEvent::Withdraw(Withdrawal::sign(&rules, seeds[0].member_id(), &key(1)))).unwrap();
    assert_unchanged(&mut registry, approval(&rules, id, &seeds[0], 1), RegistryError::InactiveVerifier);
    registry.apply(&approval(&rules, id, &seeds[1], 2)).unwrap();
    assert_eq!(registry.member(id).unwrap().status(), MembershipStatus::Pending);
    registry.apply(&approval(&rules, id, &seeds[2], 3)).unwrap();
    assert_eq!(registry.member(id).unwrap().status(), MembershipStatus::Active);
}

#[test]
fn withdrawal_cannot_be_signed_by_someone_else_including_first_seed() {
    let (rules, seeds, mut registry) = setup(2);
    let before = registry.clone();
    let forged = Withdrawal::sign(&rules, seeds[1].member_id(), &key(1));
    assert!(matches!(registry.apply(&MembershipEvent::Withdraw(forged)), Err(RegistryError::Crypto(_))));
    assert_eq!(registry, before);
}

#[test]
fn withdrawal_retains_tombstones_population_record_and_activation_order() {
    let (rules, seeds, mut registry) = setup(2);
    let event = MembershipEvent::Withdraw(Withdrawal::sign(&rules, seeds[0].member_id(), &key(1)));
    registry.apply(&event).unwrap();
    assert_eq!(registry.active_count(), 2);
    assert_eq!(registry.population_high_water(), 3);
    assert_eq!(registry.total_activations(), 3);
    assert_eq!(registry.member(seeds[0].member_id()).unwrap().activation_order(), Some(1));
    assert_unchanged(&mut registry, event, RegistryError::AlreadyRevoked);
    let another = Enrollment::sign(&rules, &key(1), sha256d(b"again"), sha256d(b"again-evidence")).unwrap();
    assert_unchanged(&mut registry, MembershipEvent::Enroll(another), RegistryError::PublicKeyAlreadyRegistered);
}

#[test]
fn pending_applicant_can_cancel_without_changing_active_population() {
    let (rules, _, mut registry) = setup(2);
    let applicant = enrollment(&rules, 4);
    registry.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    registry.apply(&MembershipEvent::Withdraw(Withdrawal::sign(&rules, applicant.member_id(), &key(4)))).unwrap();
    assert_eq!(registry.active_count(), 3);
    assert_eq!(registry.member(applicant.member_id()).unwrap().status(), MembershipStatus::Revoked);
}

#[test]
fn rejected_batch_leaves_state_unchanged() {
    let (rules, _, mut registry) = setup(2);
    let event = MembershipEvent::Enroll(enrollment(&rules, 4));
    let before = registry.clone();
    assert_eq!(registry.apply_batch(&[event.clone(), event]), Err(RegistryError::DuplicateEnrollment));
    assert_eq!(registry, before);
}

#[test]
fn deterministic_state_root_is_independent_of_approval_arrival_order() {
    let (rules, seeds, mut first) = setup(2);
    let applicant = enrollment(&rules, 4);
    first.apply(&MembershipEvent::Enroll(applicant.clone())).unwrap();
    let mut second = first.clone();
    let a = approval(&rules, applicant.member_id(), &seeds[0], 1);
    let b = approval(&rules, applicant.member_id(), &seeds[1], 2);
    first.apply_batch(&[a.clone(), b.clone()]).unwrap();
    second.apply_batch(&[b, a]).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.state_root(), second.state_root());
}

fn example_events(rules: &RegistryRules, seeds: &[Enrollment]) -> Vec<MembershipEvent> {
    let applicant = enrollment(rules, 4);
    vec![MembershipEvent::Enroll(applicant.clone()),
        approval(rules, applicant.member_id(), &seeds[0], 1),
        approval(rules, applicant.member_id(), &seeds[1], 2),
        MembershipEvent::Withdraw(Withdrawal::sign(rules, applicant.member_id(), &key(4)))]
}

#[test]
fn encoded_event_log_replays_to_identical_state_and_root() {
    let (rules, seeds, mut original) = setup(2);
    let events = example_events(&rules, &seeds);
    original.apply_batch(&events).unwrap();
    let bytes = encode_log(&events).unwrap();
    let decoded = decode_log(&bytes).unwrap();
    assert_eq!(encode_log(&decoded).unwrap(), bytes);
    let restored = MembershipRegistry::replay(rules, &seeds, &decoded).unwrap();
    assert_eq!(restored, original);
    assert_eq!(restored.state_root(), original.state_root());
}

#[test]
fn saved_log_survives_file_round_trip_and_revalidation() {
    let (rules, seeds, _) = setup(2);
    let events = example_events(&rules, &seeds);
    let expected = MembershipRegistry::replay(rules.clone(), &seeds, &events).unwrap();
    let path = std::env::temp_dir().join(format!("mirror-personhood-{}-{}.mph",
        std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let bytes = encode_log(&events).unwrap();
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let restored_bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let restored = MembershipRegistry::replay(rules, &seeds, &decode_log(&restored_bytes).unwrap()).unwrap();
    assert_eq!(restored.state_root(), expected.state_root());
}

#[test]
fn malformed_logs_are_rejected_without_accepting_partial_records() {
    let (rules, seeds, _) = setup(2);
    let bytes = encode_log(&example_events(&rules, &seeds)).unwrap();
    for end in 0..bytes.len() { assert!(decode_log(&bytes[..end]).is_err()); }
    let mut trailing = bytes.clone(); trailing.push(0);
    assert_eq!(decode_log(&trailing), Err(LogError::TrailingBytes));
    let mut bad_magic = bytes.clone(); bad_magic[0] ^= 1;
    assert_eq!(decode_log(&bad_magic), Err(LogError::InvalidMagic));
    let mut unknown = bytes; unknown[8] = 99;
    assert_eq!(decode_log(&unknown), Err(LogError::UnknownEvent));
}

#[test]
fn oversized_logs_and_impossible_counts_are_rejected() {
    assert_eq!(decode_log(&vec![0; MAX_LOG_BYTES + 1]), Err(LogError::TooLarge));
    let mut header = b"MPH1".to_vec(); header.extend_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(decode_log(&header), Err(LogError::Truncated));
}

#[test]
fn decoded_tampered_signature_is_rejected_during_replay() {
    let (rules, seeds, _) = setup(2);
    let mut bytes = encode_log(&example_events(&rules, &seeds)).unwrap();
    let last = bytes.len() - 1; bytes[last] ^= 1;
    let parsed = decode_log(&bytes).unwrap();
    assert!(matches!(MembershipRegistry::replay(rules, &seeds, &parsed), Err(RegistryError::Crypto(_))));
}

#[test]
fn invalid_rules_bootstrap_and_empty_commitments_are_rejected() {
    assert_eq!(RegistryRules::new(Hash256::default(), 1), Err(RegistryError::InvalidRules));
    assert_eq!(RegistryRules::new(sha256d(b"network"), 0), Err(RegistryError::InvalidRules));
    assert_eq!(RegistryRules::new(sha256d(b"network"), 65), Err(RegistryError::InvalidRules));
    let rules = rules(2);
    let seed = enrollment(&rules, 1);
    assert_eq!(MembershipRegistry::new(rules.clone(), &[seed.clone()]).unwrap_err(), RegistryError::InvalidBootstrap);
    assert_eq!(MembershipRegistry::new(rules.clone(), &[seed.clone(), seed]).unwrap_err(), RegistryError::DuplicateEnrollment);
    assert_eq!(Enrollment::sign(&rules, &key(1), Hash256::default(), sha256d(b"evidence")), Err(RegistryError::MissingCommitment));
}

#[test]
fn one_founder_seed_can_bootstrap_only_under_explicit_one_approval_dev_rules() {
    let rules = rules(1);
    let founder = enrollment(&rules, 1);
    let mut registry = MembershipRegistry::new(rules.clone(), &[founder.clone()]).unwrap();
    let next = enrollment(&rules, 2);
    registry.apply(&MembershipEvent::Enroll(next.clone())).unwrap();
    registry.apply(&approval(&rules, next.member_id(), &founder, 1)).unwrap();
    // The second member has the same approval ability as the first.
    let third = enrollment(&rules, 3);
    registry.apply(&MembershipEvent::Enroll(third.clone())).unwrap();
    registry.apply(&approval(&rules, third.member_id(), &next, 2)).unwrap();
    assert_eq!(registry.active_count(), 3);
}

#[test]
fn bootstrap_root_matches_independent_fixed_vector() {
    let (_, _, registry) = setup(2);
    assert_eq!(registry.state_root().to_hex(), "6115143bc7066248f7560128e0fa3444e161628de20b3d7b3199e21f83a41725");
}
