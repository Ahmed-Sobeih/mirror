//! B4 adversarial MPS2 cache tests. All keys and evidence are synthetic.
use mirror_crypto::{Hash256, Keypair, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipRegistryV2 as Registry, MembershipStateV2, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as Op,
    state::{StateCodecErrorV2 as Error, UntrustedMembershipSnapshotV2 as Snapshot},
};

fn hash(n: u8) -> Hash256 {
    Hash256::from_bytes([n; 32])
}
fn key(n: u8) -> Keypair {
    Keypair::from_secret_bytes([n; 32])
}
fn block_hash(height: u64) -> Hash256 {
    let mut bytes = [90; 32];
    bytes[..8].copy_from_slice(&height.to_le_bytes());
    Hash256::from_bytes(bytes)
}
fn fixture_with(limits: LimitsV2) -> Registry {
    let descriptors: Vec<_> = (1..=2)
        .map(|n| BootstrapDescriptor {
            public_key: key(n).public_key(),
            nonce: hash(20 + n),
            evidence: hash(30 + n),
        })
        .collect();
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(10),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile: hash(11),
        },
        limits,
        &descriptors,
        &[],
    )
    .unwrap();
    let proofs: Vec<_> = (1..=2)
        .map(|n| config.sign_seed(u32::from(n - 1), &key(n)).unwrap())
        .collect();
    Registry::bootstrap(&config.verify_bootstrap(&proofs).unwrap())
        .unwrap()
        .bind_to_validated_block(block_hash(0))
}
fn fixture() -> Registry {
    fixture_with(LimitsV2::default())
}
struct View<'a>(&'a Registry);
impl ParentChainViewV2 for View<'_> {
    fn config_hash(&self) -> Hash256 {
        self.0.state().config().context()
    }
    fn parent_height(&self) -> u64 {
        self.0.state().height()
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height <= self.parent_height()).then(|| block_hash(height))
    }
}
fn op(r: &Registry, actor: u8, sequence: u64, until: u64, action: Action) -> Op {
    Op::sign(
        Header {
            config_hash: r.state().config().context(),
            anchor_height: r.state().height(),
            anchor_hash: r.tip_hash(),
            valid_from: r.state().height() + 1,
            valid_until: until,
            actor_public_key: key(actor).public_key(),
            actor_sequence: sequence,
        },
        action,
        &key(actor),
    )
    .unwrap()
}
fn enroll(r: &Registry, actor: u8, until: u64) -> Op {
    op(
        r,
        actor,
        0,
        until,
        Action::Enroll {
            applicant_nonce: hash(40 + actor),
            evidence_commitment: hash(50 + actor),
            evidence_profile: r.state().config().parameters().evidence_profile,
        },
    )
}
fn approve(r: &Registry, actor: u8, seq: u64, target: Hash256, until: u64) -> Op {
    op(
        r,
        actor,
        seq,
        until,
        Action::Approve {
            target_enrollment_id: target,
        },
    )
}
fn withdraw(r: &Registry, actor: u8, seq: u64, target: Hash256) -> Op {
    op(
        r,
        actor,
        seq,
        r.state().height() + 8,
        Action::Withdraw {
            own_member_id: target,
        },
    )
}
fn next(r: &Registry, ops: &[Op]) -> Registry {
    r.stage_block(&View(r), ops)
        .unwrap()
        .bind_to_validated_block(block_hash(r.state().height() + 1))
}
fn id(r: &Registry, actor: u8) -> Hash256 {
    r.state()
        .record_by_key(key(actor).public_key())
        .unwrap()
        .id()
}
fn active() -> Registry {
    let r = fixture();
    let e = enroll(&r, 3, 8);
    let target = e.enrollment_id().unwrap();
    next(
        &r,
        &[
            e,
            approve(&r, 1, 0, target, 8),
            approve(&r, 2, 0, target, 8),
        ],
    )
}
fn pending() -> Registry {
    let r = fixture();
    let e = enroll(&r, 3, 8);
    let target = e.enrollment_id().unwrap();
    next(&r, &[e, approve(&r, 1, 0, target, 8)])
}
fn reject(r: &Registry, bytes: &[u8]) {
    assert!(Snapshot::decode(r.state().config(), bytes).is_err());
}
fn put64(bytes: &mut [u8], offset: usize, n: u64) {
    bytes[offset..offset + 8].copy_from_slice(&n.to_le_bytes());
}
fn put32(bytes: &mut [u8], offset: usize, n: u32) {
    bytes[offset..offset + 4].copy_from_slice(&n.to_le_bytes());
}
fn get32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[derive(Clone)]
struct Offsets {
    start: usize,
    status: usize,
    inclusion: usize,
    sequence: usize,
    activation: usize,
    approval_count: usize,
    approvals: usize,
    end: usize,
}
fn offsets(bytes: &[u8]) -> Vec<Offsets> {
    let mut result = Vec::new();
    let mut pos = 90;
    for _ in 0..get32(bytes, 86) {
        let start = pos;
        let origin = bytes[pos + 32];
        let status = pos + 33 + if origin == 0 { 100 } else { 291 };
        let inclusion = status + 1;
        let sequence = inclusion + 12;
        let activation = sequence + 8;
        let approval_count = activation + 1 + if bytes[activation] == 1 { 16 } else { 0 };
        let approvals = approval_count + 4;
        let end = approvals + get32(bytes, approval_count) as usize * 267;
        result.push(Offsets {
            start,
            status,
            inclusion,
            sequence,
            activation,
            approval_count,
            approvals,
            end,
        });
        pos = end;
    }
    assert_eq!(pos, bytes.len());
    result
}
fn record_offset(bytes: &[u8], target: Hash256) -> Offsets {
    offsets(bytes)
        .into_iter()
        .find(|o| &bytes[o.start..o.start + 32] == target.as_bytes())
        .unwrap()
}
fn replace_origin(r: &Registry, bytes: &[u8], target: Hash256, replacement: Op) -> Vec<u8> {
    let mut records: Vec<_> = offsets(bytes)
        .into_iter()
        .map(|o| {
            let mut record = bytes[o.start..o.end].to_vec();
            if &record[..32] == target.as_bytes() {
                record[..32].copy_from_slice(replacement.enrollment_id().unwrap().as_bytes());
                record[33..324].copy_from_slice(&replacement.encode());
            }
            record
        })
        .collect();
    records.sort_by(|a, b| a[..32].cmp(&b[..32]));
    let mut out = bytes[..90].to_vec();
    for record in records {
        out.extend_from_slice(&record);
    }
    assert_eq!(r.state().record_count(), get32(&out, 86) as usize);
    out
}

#[test]
fn seed_encoding_matches_fixed_prefix_size_and_domain_contract() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    assert_eq!(bytes.len(), 440);
    assert_eq!(&bytes[..6], b"MPS2\x02\x00");
    assert_eq!(&bytes[6..38], r.state().config().context().as_bytes());
    assert_eq!(&bytes[38..46], &0u64.to_le_bytes());
    assert_eq!(&bytes[46..54], &2u64.to_le_bytes());
    assert_eq!(&bytes[54..62], &0u64.to_le_bytes());
    assert_eq!(&bytes[62..70], &2u64.to_le_bytes());
    assert_eq!(&bytes[70..78], &2u64.to_le_bytes());
    assert_eq!(&bytes[78..86], &0u64.to_le_bytes());
    assert_eq!(&bytes[86..90], &2u32.to_le_bytes());
    let mut preimage = b"MIRROR_MEMBERSHIP_STATE_V2".to_vec();
    preimage.extend_from_slice(&bytes);
    assert_eq!(r.state().root().unwrap(), sha256d(&preimage));
}

#[test]
fn snapshot_bound_is_independent_of_diagnostic_log_limit() {
    let r = fixture();
    assert_eq!(
        MembershipStateV2::max_encoded_len(r.state().config()).unwrap(),
        41_484_378
    );
    let limits = LimitsV2 {
        max_records: 10,
        max_pending: 4,
        max_bootstrap: 2,
        max_new_enrollments: 4,
        max_log_bytes: 1032,
        ..LimitsV2::default()
    };
    let r = fixture_with(limits);
    let e = enroll(&r, 3, 8);
    let target = e.enrollment_id().unwrap();
    let r = next(&r, &[e, approve(&r, 1, 0, target, 8)]);
    let bytes = r.state().encode().unwrap();
    assert!(bytes.len() > r.state().config().limits().max_log_bytes as usize);
    Snapshot::decode(r.state().config(), &bytes)
        .unwrap()
        .verify_against(r.state())
        .unwrap();
}

#[test]
fn every_status_and_terminal_approval_history_round_trips_exactly() {
    let r = fixture();
    let e = enroll(&r, 3, 3);
    let target = e.enrollment_id().unwrap();
    let pending = next(&r, &[e, approve(&r, 1, 0, target, 3)]);
    let active = next(&pending, &[approve(&pending, 2, 0, target, 3)]);
    let revoked = next(&active, &[withdraw(&active, 3, 1, target)]);
    let expired = next(&next(&next(&pending, &[]), &[]), &[]);
    let revoked_pending = next(&pending, &[withdraw(&pending, 3, 1, target)]);
    for r in [&r, &pending, &active, &revoked, &expired, &revoked_pending] {
        let bytes = r.state().encode().unwrap();
        let decoded = Snapshot::decode(r.state().config(), &bytes).unwrap();
        assert_eq!(decoded.encode().unwrap(), bytes);
        assert_eq!(decoded.root().unwrap(), r.state().root().unwrap());
        assert_eq!(decoded.height(), r.state().height());
        assert_eq!(decoded.record_count(), r.state().record_count());
        decoded.verify_against(r.state()).unwrap();
    }
}

#[test]
fn tip_hash_does_not_enter_state_commitment() {
    let r = fixture();
    let staged = r.stage_block(&View(&r), &[]).unwrap();
    let a = staged.clone().bind_to_validated_block(hash(201));
    let b = staged.bind_to_validated_block(hash(202));
    assert_ne!(a.tip_hash(), b.tip_hash());
    assert_eq!(a.state().encode().unwrap(), b.state().encode().unwrap());
    assert_eq!(a.state().root().unwrap(), b.state().root().unwrap());
}

#[test]
fn every_seed_snapshot_truncation_rejects() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    for len in 0..bytes.len() {
        reject(&r, &bytes[..len]);
    }
}

#[test]
fn every_snapshot_with_signed_records_truncation_rejects() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    for len in 0..bytes.len() {
        reject(&r, &bytes[..len]);
    }
}

#[test]
fn trailing_bytes_and_oversized_input_reject() {
    let r = fixture();
    let mut bytes = r.state().encode().unwrap();
    bytes.push(0);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bytes),
        Err(Error::TrailingBytes)
    );
    let r = fixture_with(LimitsV2 {
        max_records: 2,
        max_pending: 2,
        max_bootstrap: 2,
        max_new_enrollments: 2,
        ..LimitsV2::default()
    });
    let bytes = vec![0; MembershipStateV2::max_encoded_len(r.state().config()).unwrap() + 1];
    assert_eq!(
        Snapshot::decode(r.state().config(), &bytes),
        Err(Error::TooLarge)
    );
}

#[test]
fn unknown_magic_version_and_all_tags_reject() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    for (offset, value) in [(0, 0), (4, 3), (122, 2), (223, 4), (244, 2)] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        reject(&r, &bad);
    }
}

#[test]
fn configuration_hash_must_match_pinned_configuration() {
    let r = fixture();
    let mut bytes = r.state().encode().unwrap();
    bytes[6] ^= 1;
    assert_eq!(
        Snapshot::decode(r.state().config(), &bytes),
        Err(Error::WrongContext)
    );
    let other = fixture_with(LimitsV2 {
        max_log_bytes: 2000,
        ..LimitsV2::default()
    });
    reject(&other, &r.state().encode().unwrap());
}

#[test]
fn declared_record_count_cannot_reserve_unbacked_input() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    let mut bad = bytes.clone();
    put32(&mut bad, 86, u32::MAX);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::RecordLimit)
    );
    let mut bad = bytes.clone();
    put32(&mut bad, 86, 65536);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::Truncated)
    );
    let mut bad = bytes;
    put32(&mut bad, 86, 1);
    reject(&r, &bad);
}

#[test]
fn declared_approval_counts_are_bounded_before_entries() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    let mut bad = bytes.clone();
    put64(&mut bad, 78, u64::MAX);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::ApprovalLimit)
    );
    let o = record_offset(&bytes, id(&r, 3));
    let mut bad = bytes;
    put64(&mut bad, 78, 65);
    put32(&mut bad, o.approval_count, 65);
    bad.resize(20_000, 0);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::ApprovalLimit)
    );
}

#[test]
fn records_must_be_strictly_sorted_and_distinct() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    let o = offsets(&bytes);
    let mut bad = bytes[..90].to_vec();
    bad.extend_from_slice(&bytes[o[1].start..o[1].end]);
    bad.extend_from_slice(&bytes[o[0].start..o[0].end]);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::RecordOrder)
    );
    let mut bad = bytes.clone();
    bad[o[1].start..o[1].end].copy_from_slice(&bytes[o[0].start..o[0].end]);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::RecordOrder)
    );
}

#[test]
fn approvals_must_be_strictly_sorted_and_distinct() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, id(&r, 3));
    let first = o.approvals;
    let second = first + 267;
    let mut bad = bytes.clone();
    bad[first..first + 267].copy_from_slice(&bytes[second..second + 267]);
    bad[second..second + 267].copy_from_slice(&bytes[first..first + 267]);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::ApprovalOrder)
    );
    let mut bad = bytes.clone();
    bad[second..second + 267].copy_from_slice(&bytes[first..first + 267]);
    assert_eq!(
        Snapshot::decode(r.state().config(), &bad),
        Err(Error::ApprovalOrder)
    );
}

#[test]
fn all_derived_counters_are_checked() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    for offset in [46, 54, 62, 70, 78] {
        let mut bad = bytes.clone();
        put64(&mut bad, offset, 99);
        reject(&r, &bad);
    }
    let mut bad = bytes.clone();
    put64(&mut bad, 62, 2);
    reject(&r, &bad);
    let mut bad = bytes;
    put64(&mut bad, 70, 2);
    reject(&r, &bad);
}

#[test]
fn seed_origin_id_descriptor_and_inclusion_are_bound() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    let o = &offsets(&bytes)[0];
    for offset in [
        o.start,
        o.start + 33,
        o.start + 37,
        o.start + 69,
        o.start + 101,
        o.inclusion,
        o.inclusion + 8,
    ] {
        let mut bad = bytes.clone();
        bad[offset] ^= 1;
        reject(&r, &bad);
    }
}

#[test]
fn activation_none_is_distinct_from_zero_and_positive_seed_activation() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    let o = &offsets(&bytes)[0];
    assert_eq!(bytes[o.activation], 1);
    assert_eq!(&bytes[o.activation + 1..o.activation + 9], &[0; 8]);
    let mut bad = bytes.clone();
    put64(&mut bad, o.activation + 9, 0);
    reject(&r, &bad);
    let mut bad = bytes.clone();
    bad[o.activation] = 0;
    bad.drain(o.activation + 1..o.activation + 17);
    reject(&r, &bad);
}

#[test]
fn corrupted_enrollment_and_approval_signatures_are_rejected() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, id(&r, 3));
    for offset in [o.start + 33 + 290, o.approvals + 266] {
        let mut bad = bytes.clone();
        bad[offset] ^= 1;
        assert!(matches!(
            Snapshot::decode(r.state().config(), &bad),
            Err(Error::Operation(_))
        ));
    }
}

#[test]
fn pending_and_seed_sequences_cannot_be_invented() {
    let r = pending();
    let bytes = r.state().encode().unwrap();
    for actor in [1, 2, 3] {
        let o = record_offset(&bytes, id(&r, actor));
        let mut bad = bytes.clone();
        put64(&mut bad, o.sequence, 9);
        reject(&r, &bad);
    }
    let r = fixture();
    let mut bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, id(&r, 1));
    bytes[o.status] = 2;
    put64(&mut bytes, 46, 1);
    reject(&r, &bytes);
}

#[test]
fn seed_withdrawal_retains_origin_activation_and_correct_sequence() {
    let r = fixture();
    let r = next(&r, &[withdraw(&r, 1, 0, id(&r, 1))]);
    let bytes = r.state().encode().unwrap();
    Snapshot::decode(r.state().config(), &bytes)
        .unwrap()
        .verify_against(r.state())
        .unwrap();
    let o = record_offset(&bytes, id(&r, 1));
    assert_eq!(bytes[o.status], 2);
    assert_eq!(bytes[o.activation], 1);
}

#[test]
fn repeated_outgoing_sequence_across_different_requests_rejects() {
    let r = fixture();
    let a = enroll(&r, 3, 8);
    let b = enroll(&r, 4, 8);
    let aid = a.enrollment_id().unwrap();
    let bid = b.enrollment_id().unwrap();
    let r = next(
        &r,
        &[a, b, approve(&r, 1, 0, aid, 8), approve(&r, 1, 1, bid, 8)],
    );
    let mut bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, bid);
    let original = Op::decode(
        r.state().config(),
        &bytes[o.approvals + 40..o.approvals + 267],
    )
    .unwrap();
    let mut header = *original.header();
    header.actor_sequence = 0;
    let replacement = Op::sign(header, *original.action(), &key(1)).unwrap();
    bytes[o.approvals + 40..o.approvals + 267].copy_from_slice(&replacement.encode());
    assert_eq!(
        Snapshot::decode(r.state().config(), &bytes),
        Err(Error::Inconsistent("review sequence"))
    );
}

#[test]
fn changed_approval_kind_or_target_is_not_a_valid_cache() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, id(&r, 3));
    let original = Op::decode(
        r.state().config(),
        &bytes[o.approvals + 40..o.approvals + 267],
    )
    .unwrap();
    let actor = if original.header().actor_public_key == key(1).public_key() {
        1
    } else {
        2
    };
    for action in [
        Action::Withdraw {
            own_member_id: id(&r, actor),
        },
        Action::Approve {
            target_enrollment_id: id(&r, actor),
        },
    ] {
        let replacement = Op::sign(*original.header(), action, &key(actor)).unwrap();
        let mut bad = bytes.clone();
        bad[o.approvals + 40..o.approvals + 267].copy_from_slice(&replacement.encode());
        reject(&r, &bad);
    }
}

#[test]
fn validly_signed_origin_cannot_reuse_a_retained_key_or_evidence() {
    let r = fixture();
    let request = enroll(&r, 3, 8);
    let target = request.enrollment_id().unwrap();
    let r = next(&r, &[request]);
    let bytes = r.state().encode().unwrap();
    let mut header = *request.header();
    header.actor_public_key = key(1).public_key();
    let duplicate_key = Op::sign(header, *request.action(), &key(1)).unwrap();
    let bad = replace_origin(&r, &bytes, target, duplicate_key);
    reject(&r, &bad);
    let Action::Enroll {
        applicant_nonce,
        evidence_profile,
        ..
    } = *request.action()
    else {
        unreachable!()
    };
    let duplicate_evidence = Op::sign(
        *request.header(),
        Action::Enroll {
            applicant_nonce,
            evidence_commitment: hash(31),
            evidence_profile,
        },
        &key(3),
    )
    .unwrap();
    let bad = replace_origin(&r, &bytes, target, duplicate_evidence);
    reject(&r, &bad);
}

#[test]
fn validly_signed_origin_with_wrong_profile_or_window_rejects() {
    let r = fixture();
    let request = enroll(&r, 3, 8);
    let target = request.enrollment_id().unwrap();
    let r = next(&r, &[request]);
    let bytes = r.state().encode().unwrap();
    let Action::Enroll {
        applicant_nonce,
        evidence_commitment,
        ..
    } = *request.action()
    else {
        unreachable!()
    };
    let changed = Op::sign(
        *request.header(),
        Action::Enroll {
            applicant_nonce,
            evidence_commitment,
            evidence_profile: hash(255),
        },
        &key(3),
    )
    .unwrap();
    reject(&r, &replace_origin(&r, &bytes, target, changed));
    let mut header = *request.header();
    header.anchor_height = header.valid_from;
    let changed = Op::sign(header, *request.action(), &key(3)).unwrap();
    reject(&r, &replace_origin(&r, &bytes, target, changed));
}

#[test]
fn inclusion_index_and_approval_acceptance_height_are_consistent() {
    let r = active();
    let bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, id(&r, 3));
    let mut bad = bytes.clone();
    put32(&mut bad, o.inclusion + 8, u32::MAX);
    reject(&r, &bad);
    let mut bad = bytes.clone();
    put64(&mut bad, o.approvals + 32, 0);
    reject(&r, &bad);
    let mut bad = bytes;
    put64(&mut bad, o.approvals + 32, 2);
    reject(&r, &bad);
}

#[test]
fn internally_consistent_cache_never_establishes_current_state() {
    let r = fixture();
    let bytes = r.state().encode().unwrap();
    let mut forged_height = bytes.clone();
    put64(&mut forged_height, 38, 1);
    // A seed-only state after an empty block is structurally coherent. That does
    // not make it the caller's expected state or grant a registry conversion.
    let cache = Snapshot::decode(r.state().config(), &forged_height).unwrap();
    assert_eq!(
        cache.verify_against(r.state()),
        Err(Error::SnapshotMismatch)
    );
    assert_ne!(cache.root().unwrap(), r.state().root().unwrap());
    let later = next(&r, &[]);
    let old = Snapshot::decode(r.state().config(), &bytes).unwrap();
    assert_eq!(
        old.verify_against(later.state()),
        Err(Error::SnapshotMismatch)
    );
    // Even plausible same-height withdrawal metadata is not proof of a signed
    // historical withdrawal. No withdrawal message is retained in MPS2.
    let mut forged_withdrawal = later.state().encode().unwrap();
    let seed = later.state().config().seed_id(0).unwrap();
    let o = record_offset(&forged_withdrawal, seed);
    forged_withdrawal[o.status] = 2;
    put64(&mut forged_withdrawal, o.sequence, 1);
    put64(&mut forged_withdrawal, 46, 1);
    let forged = Snapshot::decode(later.state().config(), &forged_withdrawal).unwrap();
    assert_eq!(
        forged.verify_against(later.state()),
        Err(Error::SnapshotMismatch)
    );
}

#[test]
fn expired_pending_and_active_metadata_cannot_be_interchanged() {
    let r = pending();
    let bytes = r.state().encode().unwrap();
    let o = record_offset(&bytes, id(&r, 3));
    let mut bad = bytes.clone();
    bad[o.status] = 3;
    put64(&mut bad, 54, 0);
    reject(&r, &bad);
    let mut bad = bytes.clone();
    put64(&mut bad, 38, 9);
    reject(&r, &bad);
    let mut bad = bytes;
    bad[o.status] = 1;
    put64(&mut bad, 46, 3);
    put64(&mut bad, 54, 0);
    reject(&r, &bad);
}
