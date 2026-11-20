//! Independent B4 grouped replay boundary tests. Trusted metadata is built from
//! separate B3 execution, never accepted from the log being checked. Hashes are
//! synthetic tags; these tests do not validate real block headers or finality.
use std::{cell::Cell, collections::BTreeMap};

use mirror_crypto::{Hash256, Keypair, SignatureBytes, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipRegistryV2 as Registry, MembershipReplayV2 as Replay, MembershipStatusV2 as Status,
    OperationErrorV2, OperationHeaderV2 as Header, ParentChainViewV2, ReplayChainViewV2,
    ReplayErrorV2 as ReplayError, ReplayExpectationV2, ReplayGroupV2 as Group,
    SignedMembershipOperationV2 as Op, StateErrorV2 as StateError, TestAllocation,
    TrustedReplayBlockV2, VerifiedBootstrap, replay_group_commitment_v2,
};

fn hash(domain: u8, number: u64) -> Hash256 {
    let mut bytes = [0; 32];
    bytes[0] = domain;
    bytes[1..9].copy_from_slice(&number.to_le_bytes());
    Hash256::from_bytes(bytes)
}

fn key(number: u8) -> Keypair {
    Keypair::from_secret_bytes([number; 32])
}

fn block_hash(height: u64) -> Hash256 {
    hash(200, height)
}

#[derive(Clone)]
struct TrustedHistory {
    context: Hash256,
    blocks: BTreeMap<u64, TrustedReplayBlockV2>,
    lookups: Cell<usize>,
}

impl ReplayChainViewV2 for TrustedHistory {
    fn config_hash(&self) -> Hash256 {
        self.context
    }

    fn block_at(&self, height: u64) -> Option<TrustedReplayBlockV2> {
        self.lookups.set(self.lookups.get() + 1);
        self.blocks.get(&height).copied()
    }
}

struct Parent<'a> {
    height: u64,
    history: &'a TrustedHistory,
}

impl ParentChainViewV2 for Parent<'_> {
    fn config_hash(&self) -> Hash256 {
        self.history.context
    }

    fn parent_height(&self) -> u64 {
        self.height
    }

    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        if height > self.height {
            return None;
        }
        self.history
            .blocks
            .get(&height)
            .map(|block| block.block_hash)
    }
}

struct Fixture {
    bootstrap: VerifiedBootstrap,
    registry: Registry,
    history: TrustedHistory,
    groups: Vec<Group>,
}

impl Fixture {
    fn with_limits(limits: LimitsV2) -> Self {
        Self::with_allocations(limits, &[])
    }

    fn with_allocations(limits: LimitsV2, allocations: &[TestAllocation]) -> Self {
        let seeds: Vec<_> = (1..=2)
            .map(|number| BootstrapDescriptor {
                public_key: key(number).public_key(),
                nonce: hash(1, u64::from(number)),
                evidence: hash(2, u64::from(number)),
            })
            .collect();
        let config = ConfigV2::new(
            ConfigParameters {
                format_version: 2,
                profile_version: 1,
                network: hash(3, 1),
                pow_bits: 0x207f_ffff,
                genesis_timestamp: 0,
                required_approvals: 2,
                evidence_profile: hash(4, 1),
            },
            limits,
            &seeds,
            allocations,
        )
        .unwrap();
        let proofs: Vec<_> = (1..=2)
            .map(|number| {
                config
                    .sign_seed(u32::from(number - 1), &key(number))
                    .unwrap()
            })
            .collect();
        let bootstrap = config.verify_bootstrap(&proofs).unwrap();
        let registry = Registry::bootstrap(&bootstrap)
            .unwrap()
            .bind_to_validated_block(block_hash(0));
        let genesis = TrustedReplayBlockV2 {
            block_hash: block_hash(0),
            group_commitment: replay_group_commitment_v2(&config, 0, &[]).unwrap(),
            membership_root: registry.state().root().unwrap(),
        };
        Self {
            bootstrap,
            registry,
            history: TrustedHistory {
                context: config.context(),
                blocks: BTreeMap::from([(0, genesis)]),
                lookups: Cell::new(0),
            },
            groups: Vec::new(),
        }
    }

    fn new() -> Self {
        Self::with_limits(LimitsV2::default())
    }

    fn config(&self) -> &ConfigV2 {
        self.bootstrap.config()
    }

    fn append(&mut self, operations: &[Op]) {
        let height = self.registry.state().height() + 1;
        let staged = self
            .registry
            .stage_block(
                &Parent {
                    height: height - 1,
                    history: &self.history,
                },
                operations,
            )
            .unwrap();
        let metadata = TrustedReplayBlockV2 {
            block_hash: block_hash(height),
            group_commitment: replay_group_commitment_v2(self.config(), height, operations)
                .unwrap(),
            membership_root: staged.state().root().unwrap(),
        };
        let group = Group::new(self.config(), height, metadata.block_hash, operations).unwrap();
        self.registry = staged.bind_to_validated_block(metadata.block_hash);
        self.history.blocks.insert(height, metadata);
        self.groups.push(group);
    }

    fn expected(&self) -> ReplayExpectationV2 {
        ReplayExpectationV2 {
            config_hash: self.config().context(),
            height: self.registry.state().height(),
            tip_hash: self.registry.tip_hash(),
            membership_root: self.registry.state().root().unwrap(),
        }
    }

    fn log(&self) -> Replay {
        Replay::new(self.config(), block_hash(0), &self.groups).unwrap()
    }

    fn check(&self, log: &Replay) -> Result<Registry, mirror_personhood::v2::ReplayErrorV2> {
        log.replay(&self.bootstrap, &self.history, self.expected())
    }
}

fn signed(fixture: &Fixture, actor: u8, sequence: u64, until: u64, action: Action) -> Op {
    Op::sign(
        Header {
            config_hash: fixture.config().context(),
            anchor_height: 0,
            anchor_hash: block_hash(0),
            valid_from: 1,
            valid_until: until,
            actor_public_key: key(actor).public_key(),
            actor_sequence: sequence,
        },
        action,
        &key(actor),
    )
    .unwrap()
}

fn enroll(fixture: &Fixture, actor: u8, until: u64) -> Op {
    signed(
        fixture,
        actor,
        0,
        until,
        Action::Enroll {
            applicant_nonce: hash(5, u64::from(actor)),
            evidence_commitment: hash(6, u64::from(actor)),
            evidence_profile: fixture.config().parameters().evidence_profile,
        },
    )
}

fn approve(fixture: &Fixture, actor: u8, sequence: u64, id: Hash256, until: u64) -> Op {
    signed(
        fixture,
        actor,
        sequence,
        until,
        Action::Approve {
            target_enrollment_id: id,
        },
    )
}

fn withdraw(fixture: &Fixture, actor: u8, sequence: u64, id: Hash256, until: u64) -> Op {
    signed(
        fixture,
        actor,
        sequence,
        until,
        Action::Withdraw { own_member_id: id },
    )
}

fn active_fixture() -> Fixture {
    let mut f = Fixture::new();
    let request = enroll(&f, 3, 8);
    let id = request.enrollment_id().unwrap();
    f.append(&[request, approve(&f, 1, 0, id, 8), approve(&f, 2, 0, id, 8)]);
    f
}

fn lifecycle_fixture() -> Fixture {
    let mut f = Fixture::new();
    let a = enroll(&f, 3, 4);
    let b = enroll(&f, 4, 2);
    let c = enroll(&f, 5, 8);
    let aid = a.enrollment_id().unwrap();
    let bid = b.enrollment_id().unwrap();
    let cid = c.enrollment_id().unwrap();
    f.append(&[
        a,
        approve(&f, 1, 0, aid, 4),
        approve(&f, 2, 0, aid, 4),
        b,
        approve(&f, 1, 1, bid, 2),
        c,
    ]);
    let d = enroll(&f, 6, 3);
    let did = d.enrollment_id().unwrap();
    f.append(&[
        withdraw(&f, 3, 1, aid, 8),
        withdraw(&f, 5, 1, cid, 8),
        d,
        approve(&f, 2, 1, did, 3),
    ]);
    f.append(&[]);
    f.append(&[]);
    f
}

fn replace_group(fixture: &Fixture, index: usize, operations: &[Op]) -> Replay {
    let mut groups = fixture.groups.clone();
    let old = &groups[index];
    groups[index] =
        Group::new(fixture.config(), old.height(), old.block_hash(), operations).unwrap();
    Replay::new(fixture.config(), block_hash(0), &groups).unwrap()
}

fn raw_log(config: &ConfigV2, groups: &[(u64, Hash256, Vec<Vec<u8>>)]) -> Vec<u8> {
    let mut bytes = b"MPH2".to_vec();
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(config.context().as_bytes());
    bytes.extend_from_slice(block_hash(0).as_bytes());
    bytes.extend_from_slice(&(groups.len() as u32).to_le_bytes());
    for (height, hash, operations) in groups {
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(hash.as_bytes());
        bytes.extend_from_slice(&(operations.len() as u32).to_le_bytes());
        for operation in operations {
            bytes.extend_from_slice(&(operation.len() as u32).to_le_bytes());
            bytes.extend_from_slice(operation);
        }
    }
    bytes
}

#[test]
fn genesis_only_log_replays_verified_seeds_and_exact_trusted_genesis() {
    let f = Fixture::new();
    let log = f.log();
    assert_eq!(log.encode().unwrap().len(), 74);
    assert_eq!(log.config_hash(), f.config().context());
    assert_eq!(log.genesis_hash(), block_hash(0));
    assert!(log.groups().is_empty());
    assert_eq!(f.check(&log).unwrap(), f.registry);
}

#[test]
fn replay_recovers_full_lifecycle_records_indexes_sequences_and_counters() {
    let f = lifecycle_fixture();
    let log = Replay::decode(f.config(), &f.log().encode().unwrap()).unwrap();
    let recovered = f.check(&log).unwrap();
    assert_eq!(recovered, f.registry);
    assert_eq!(
        recovered.state().encode().unwrap(),
        f.registry.state().encode().unwrap()
    );
    assert_eq!(
        recovered.state().root().unwrap(),
        f.expected().membership_root
    );
    assert_eq!(recovered.state().record_count(), 6);
    assert_eq!(recovered.state().active_count(), 2);
    assert_eq!(recovered.state().pending_count(), 0);
    assert_eq!(recovered.state().active_high_water(), 3);
    assert_eq!(recovered.state().total_activations(), 3);
    assert_eq!(recovered.state().retained_approval_count(), 4);
    for (actor, status, sequence, reviews, activated) in [
        (1, Status::Active, 2, 0, true),
        (2, Status::Active, 2, 0, true),
        (3, Status::Revoked, 2, 2, true),
        (4, Status::Expired, 1, 1, false),
        (5, Status::Revoked, 2, 0, false),
        (6, Status::Expired, 1, 1, false),
    ] {
        let record = recovered
            .state()
            .record_by_key(key(actor).public_key())
            .unwrap();
        assert_eq!(record.status(), status);
        assert_eq!(record.next_sequence(), sequence);
        assert_eq!(record.approval_count(), reviews);
        assert_eq!(record.activation().is_some(), activated);
    }
}

#[test]
fn replay_restores_tombstone_duplicate_protection_and_remains_executable() {
    let f = lifecycle_fixture();
    let recovered = f.check(&f.log()).unwrap();
    let before = recovered.clone();
    let view = Parent {
        height: 4,
        history: &f.history,
    };
    for actor in 3..=6 {
        assert!(
            recovered
                .stage_block(&view, &[enroll(&f, actor, 8)])
                .is_err()
        );
        assert_eq!(recovered, before);
    }
    let fresh = enroll(&f, 7, 8);
    assert!(recovered.stage_block(&view, &[fresh]).is_ok());
}

#[test]
fn empty_groups_preserve_inclusive_expiry_and_are_not_omitted() {
    let mut f = Fixture::new();
    let request = enroll(&f, 3, 1);
    let id = request.enrollment_id().unwrap();
    f.append(&[request]);
    assert_eq!(
        f.check(&f.log())
            .unwrap()
            .state()
            .record(id)
            .unwrap()
            .status(),
        Status::Pending
    );
    f.append(&[]);
    let decoded = Replay::decode(f.config(), &f.log().encode().unwrap()).unwrap();
    assert_eq!(decoded.groups().len(), 2);
    assert!(decoded.groups()[1].operations().is_empty());
    assert_eq!(
        f.check(&decoded)
            .unwrap()
            .state()
            .record(id)
            .unwrap()
            .status(),
        Status::Expired
    );
}

#[test]
fn log_bytes_have_exact_prefix_group_and_operation_lengths() {
    let mut f = Fixture::new();
    f.append(&[]);
    let bytes = f.log().encode().unwrap();
    assert_eq!(bytes.len(), 74 + 44);
    assert_eq!(&bytes[0..4], b"MPH2");
    assert_eq!(&bytes[4..6], &2u16.to_le_bytes());
    assert_eq!(&bytes[6..38], f.config().context().as_bytes());
    assert_eq!(&bytes[38..70], block_hash(0).as_bytes());
    assert_eq!(&bytes[70..74], &1u32.to_le_bytes());
    assert_eq!(&bytes[74..82], &1u64.to_le_bytes());
    assert_eq!(&bytes[82..114], block_hash(1).as_bytes());
    assert_eq!(&bytes[114..118], &0u32.to_le_bytes());
    let request = enroll(&f, 3, 8);
    f.append(&[request]);
    let bytes = f.log().encode().unwrap();
    assert_eq!(bytes.len(), 74 + 44 + 44 + 4 + 291);
    assert_eq!(&bytes[162..166], &291u32.to_le_bytes());
    assert_eq!(&bytes[166..], request.encode().as_slice());
}

#[test]
fn exact_group_commitment_binds_context_height_count_order_and_signed_bytes() {
    let f = active_fixture();
    let group = &f.groups[0];
    let ops = group.operations();
    let mut preimage = b"MIRROR_MEMBERSHIP_REPLAY_GROUP_V2".to_vec();
    preimage.extend_from_slice(f.config().context().as_bytes());
    preimage.extend_from_slice(&1u64.to_le_bytes());
    preimage.extend_from_slice(&3u32.to_le_bytes());
    for op in ops {
        preimage.extend_from_slice(&(op.encoded_len() as u32).to_le_bytes());
        preimage.extend_from_slice(&op.encode());
    }
    let expected = sha256d(&preimage);
    assert_eq!(group.commitment(f.config().context()).unwrap(), expected);
    assert_eq!(
        replay_group_commitment_v2(f.config(), 1, ops).unwrap(),
        expected
    );
    assert_ne!(group.commitment(hash(90, 0)).unwrap(), expected);
    assert_ne!(
        replay_group_commitment_v2(f.config(), 2, ops).unwrap(),
        expected
    );
    assert_ne!(
        replay_group_commitment_v2(f.config(), 1, &ops[..2]).unwrap(),
        expected
    );
    let mut reordered = ops.to_vec();
    reordered.swap(1, 2);
    assert_ne!(
        replay_group_commitment_v2(f.config(), 1, &reordered).unwrap(),
        expected
    );
    let mut changed = ops.to_vec();
    changed[1] = Op::from_parts(
        *ops[1].header(),
        *ops[1].action(),
        SignatureBytes::from_bytes([0; 64]),
    );
    assert_ne!(
        replay_group_commitment_v2(f.config(), 1, &changed).unwrap(),
        expected
    );
    assert_ne!(
        replay_group_commitment_v2(f.config(), 0, &[]).unwrap(),
        replay_group_commitment_v2(f.config(), 1, &[]).unwrap()
    );
}

#[test]
fn every_log_truncation_and_trailing_byte_is_rejected() {
    let f = active_fixture();
    let bytes = f.log().encode().unwrap();
    for end in 0..bytes.len() {
        assert!(
            Replay::decode(f.config(), &bytes[..end]).is_err(),
            "prefix {end}"
        );
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Replay::decode(f.config(), &trailing).is_err());
}

#[test]
fn wrong_magic_version_and_configuration_are_rejected() {
    let f = active_fixture();
    let good = f.log().encode().unwrap();
    for (offset, byte) in [(0, b'X'), (3, b'1'), (4, 1), (5, 1), (6, 255)] {
        let mut bytes = good.clone();
        bytes[offset] ^= byte;
        assert!(Replay::decode(f.config(), &bytes).is_err());
    }
}

#[test]
fn huge_claimed_counts_and_lengths_fail_without_claimed_allocation() {
    let f = active_fixture();
    let good = f.log().encode().unwrap();
    for offset in [70, 114, 118] {
        let mut bytes = good.clone();
        bytes[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Replay::decode(f.config(), &bytes).is_err());
    }
    let mut no_payload = good[..118].to_vec();
    no_payload[114..118].copy_from_slice(&2u32.to_le_bytes());
    assert!(Replay::decode(f.config(), &no_payload).is_err());
}

#[test]
fn incorrect_operation_lengths_versions_and_kinds_are_rejected() {
    let f = Fixture::new();
    let op = enroll(&f, 3, 8).encode();
    for size in [0, 1, 226, 228, 290, 292, 1_025] {
        let bytes = raw_log(f.config(), &[(1, block_hash(1), vec![vec![0; size]])]);
        assert!(Replay::decode(f.config(), &bytes).is_err());
    }
    for (offset, value) in [(0, 1), (2, 0), (2, 255)] {
        let mut altered = op.clone();
        altered[offset] = value;
        let bytes = raw_log(f.config(), &[(1, block_hash(1), vec![altered])]);
        assert!(Replay::decode(f.config(), &bytes).is_err());
    }
}

#[test]
fn zero_skipped_duplicate_reordered_and_maximum_heights_are_rejected() {
    let f = Fixture::new();
    for heights in [
        vec![0],
        vec![2],
        vec![u64::MAX],
        vec![1, 1],
        vec![2, 1],
        vec![1, 3],
    ] {
        let raw: Vec<_> = heights
            .iter()
            .map(|h| (*h, block_hash(*h), Vec::new()))
            .collect();
        assert!(Replay::decode(f.config(), &raw_log(f.config(), &raw)).is_err());
        let groups: Result<Vec<_>, _> = heights
            .iter()
            .map(|h| Group::new(f.config(), *h, block_hash(*h), &[]))
            .collect();
        if let Ok(groups) = groups {
            assert!(Replay::new(f.config(), block_hash(0), &groups).is_err());
        }
    }
}

#[test]
fn exact_log_byte_limit_passes_and_one_byte_under_rejects() {
    for (max_log_bytes, permitted) in [(413, true), (412, false)] {
        let f = Fixture::with_limits(LimitsV2 {
            max_operation_bytes: 291,
            max_log_bytes,
            ..LimitsV2::default()
        });
        let op = enroll(&f, 3, 8);
        let group = Group::new(f.config(), 1, block_hash(1), &[op]).unwrap();
        assert_eq!(
            Replay::new(f.config(), block_hash(0), &[group]).is_ok(),
            permitted
        );
        let bytes = raw_log(f.config(), &[(1, block_hash(1), vec![op.encode()])]);
        assert_eq!(bytes.len(), 413);
        assert_eq!(Replay::decode(f.config(), &bytes).is_ok(), permitted);
    }
}

#[test]
fn replay_group_budget_is_4096_including_empty_groups() {
    let f = Fixture::new();
    let groups: Vec<_> = (1..=4096)
        .map(|height| Group::new(f.config(), height, block_hash(height), &[]).unwrap())
        .collect();
    let log = Replay::new(f.config(), block_hash(0), &groups).unwrap();
    let encoded = log.encode().unwrap();
    assert_eq!(encoded.len(), 74 + 4096 * 44);
    assert_eq!(
        Replay::decode(f.config(), &encoded).unwrap().groups().len(),
        4096
    );
    let mut too_many = groups;
    too_many.push(Group::new(f.config(), 4097, block_hash(4097), &[]).unwrap());
    assert!(Replay::new(f.config(), block_hash(0), &too_many).is_err());
    let mut bytes = encoded;
    bytes[70..74].copy_from_slice(&4097u32.to_le_bytes());
    bytes.extend_from_slice(&4097u64.to_le_bytes());
    bytes.extend_from_slice(block_hash(4097).as_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    assert!(Replay::decode(f.config(), &bytes).is_err());
}

#[test]
fn direct_group_and_decoder_enforce_operation_membership_and_enrollment_counts() {
    for limits in [
        LimitsV2 {
            max_operations: 2,
            max_membership_operations: 2,
            max_new_enrollments: 2,
            ..LimitsV2::default()
        },
        LimitsV2 {
            max_membership_operations: 2,
            max_new_enrollments: 2,
            ..LimitsV2::default()
        },
        LimitsV2 {
            max_new_enrollments: 2,
            ..LimitsV2::default()
        },
    ] {
        let f = Fixture::with_limits(limits);
        let ops = [enroll(&f, 3, 8), enroll(&f, 4, 8), enroll(&f, 5, 8)];
        assert!(Group::new(f.config(), 1, block_hash(1), &ops[..2]).is_ok());
        assert!(Group::new(f.config(), 1, block_hash(1), &ops).is_err());
        let raw = raw_log(
            f.config(),
            &[(1, block_hash(1), ops.iter().map(Op::encode).collect())],
        );
        assert!(Replay::decode(f.config(), &raw).is_err());
    }
}

#[test]
fn group_byte_budget_includes_168_byte_block_overhead_and_operation_lengths() {
    for (max_block_bytes, permitted) in [(758, true), (757, false)] {
        let f = Fixture::with_limits(LimitsV2 {
            max_operation_bytes: 291,
            max_block_bytes,
            ..LimitsV2::default()
        });
        let ops = [enroll(&f, 3, 8), enroll(&f, 4, 8)];
        assert_eq!(
            Group::new(f.config(), 1, block_hash(1), &ops).is_ok(),
            permitted
        );
        let raw = raw_log(
            f.config(),
            &[(1, block_hash(1), ops.iter().map(Op::encode).collect())],
        );
        assert_eq!(Replay::decode(f.config(), &raw).is_ok(), permitted);
    }
}

#[test]
fn genesis_hash_root_and_empty_group_commitment_are_independently_required() {
    let f = active_fixture();
    let log = f.log();
    let different_genesis = Replay::new(f.config(), hash(99, 0), &f.groups).unwrap();
    assert!(f.check(&different_genesis).is_err());
    for field in 0..3 {
        let mut history = f.history.clone();
        let block = history.blocks.get_mut(&0).unwrap();
        match field {
            0 => block.block_hash = hash(99, 1),
            1 => block.membership_root = hash(99, 2),
            _ => block.group_commitment = hash(99, 3),
        }
        assert!(log.replay(&f.bootstrap, &history, f.expected()).is_err());
    }
}

#[test]
fn missing_trusted_metadata_rejects_genesis_intermediate_or_final_height() {
    let f = lifecycle_fixture();
    for height in [0, 1, 2, 4] {
        let mut history = f.history.clone();
        history.blocks.remove(&height);
        assert!(
            f.log()
                .replay(&f.bootstrap, &history, f.expected())
                .is_err()
        );
    }
}

#[test]
fn each_trusted_intermediate_hash_root_and_exact_group_are_checked() {
    let f = lifecycle_fixture();
    for field in 0..3 {
        let mut history = f.history.clone();
        let block = history.blocks.get_mut(&2).unwrap();
        match field {
            0 => block.block_hash = hash(99, 1),
            1 => block.membership_root = hash(99, 2),
            _ => block.group_commitment = hash(99, 3),
        }
        assert_eq!(
            f.log().replay(&f.bootstrap, &history, f.expected()),
            Err(match field {
                0 => ReplayError::BlockHashMismatch { height: 2 },
                1 => ReplayError::StateRootMismatch { height: 2 },
                _ => ReplayError::GroupCommitmentMismatch { height: 2 },
            })
        );
    }
    let mut groups = f.groups.clone();
    groups[1] = Group::new(f.config(), 2, hash(98, 2), groups[1].operations()).unwrap();
    assert!(
        f.check(&Replay::new(f.config(), block_hash(0), &groups).unwrap())
            .is_err()
    );
}

#[test]
fn explicit_expected_context_height_tip_and_root_are_all_required() {
    let f = active_fixture();
    let log = f.log();
    for field in 0..4 {
        let mut expected = f.expected();
        match field {
            0 => expected.config_hash = hash(99, 1),
            1 => expected.height = u64::MAX,
            2 => expected.tip_hash = hash(99, 2),
            _ => expected.membership_root = hash(99, 3),
        }
        assert!(log.replay(&f.bootstrap, &f.history, expected).is_err());
    }
    let mut history = f.history.clone();
    history.context = hash(99, 4);
    assert!(log.replay(&f.bootstrap, &history, f.expected()).is_err());
    let other = Fixture::with_limits(LimitsV2 {
        max_records: 65_535,
        ..LimitsV2::default()
    });
    assert!(
        log.replay(&other.bootstrap, &f.history, f.expected())
            .is_err()
    );
}

#[test]
fn an_older_valid_prefix_cannot_satisfy_the_current_trusted_expectation() {
    let mut f = active_fixture();
    let old_log = f.log();
    let old_expected = f.expected();
    let old_registry = f.registry.clone();
    f.append(&[]);
    assert!(
        old_log
            .replay(&f.bootstrap, &f.history, f.expected())
            .is_err()
    );
    // Deliberately requesting the historical checkpoint remains possible, but
    // the caller must explicitly select its older expected tuple.
    assert_eq!(
        old_log
            .replay(&f.bootstrap, &f.history, old_expected)
            .unwrap(),
        old_registry
    );
}

#[test]
fn invalid_signature_fails_even_when_group_metadata_matches_the_bad_bytes() {
    let f = active_fixture();
    let mut ops = f.groups[0].operations().to_vec();
    let op = ops[2];
    ops[2] = Op::from_parts(
        *op.header(),
        *op.action(),
        SignatureBytes::from_bytes([0; 64]),
    );
    let bad = replace_group(&f, 0, &ops);
    let mut history = f.history.clone();
    history.blocks.get_mut(&1).unwrap().group_commitment =
        replay_group_commitment_v2(f.config(), 1, &ops).unwrap();
    assert!(matches!(
        bad.replay(&f.bootstrap, &history, f.expected()),
        Err(ReplayError::State {
            height: 1,
            error: StateError::Operation {
                index: 2,
                error: OperationErrorV2::Crypto(_)
            }
        })
    ));
    assert_eq!(f.check(&f.log()).unwrap(), f.registry);
}

#[test]
fn wrong_sequence_cannot_be_authorized_by_a_matching_group_commitment() {
    let mut f = active_fixture();
    let id = f
        .registry
        .state()
        .record_by_key(key(3).public_key())
        .unwrap()
        .id();
    f.append(&[withdraw(&f, 3, 1, id, 8)]);
    let bad_op = withdraw(&f, 3, 9, id, 8);
    let bad = replace_group(&f, 1, &[bad_op]);
    let mut history = f.history.clone();
    history.blocks.get_mut(&2).unwrap().group_commitment =
        replay_group_commitment_v2(f.config(), 2, &[bad_op]).unwrap();
    assert_eq!(
        bad.replay(&f.bootstrap, &history, f.expected()),
        Err(ReplayError::State {
            height: 2,
            error: StateError::SequenceMismatch {
                expected: 1,
                got: 9
            }
        })
    );
}

#[test]
fn signed_wrong_anchor_and_wrong_context_fail_despite_matching_group_metadata() {
    let f = active_fixture();
    let original = f.groups[0].operations()[0];
    for context_change in [false, true] {
        let mut header = *original.header();
        if context_change {
            header.config_hash = hash(99, 4);
        } else {
            header.anchor_hash = hash(99, 5);
        }
        let modified = Op::sign(header, *original.action(), &key(3)).unwrap();
        assert!(modified.verify_signature().is_ok());
        let mut ops = f.groups[0].operations().to_vec();
        ops[0] = modified;
        let bad = replace_group(&f, 0, &ops);
        let mut history = f.history.clone();
        history.blocks.get_mut(&1).unwrap().group_commitment =
            replay_group_commitment_v2(f.config(), 1, &ops).unwrap();
        assert_eq!(
            bad.replay(&f.bootstrap, &history, f.expected()),
            Err(ReplayError::State {
                height: 1,
                error: StateError::Operation {
                    index: 0,
                    error: if context_change {
                        OperationErrorV2::WrongContext
                    } else {
                        OperationErrorV2::WrongAnchorHash
                    }
                }
            })
        );
    }
}

#[test]
fn reordered_commuting_approvals_reject_even_when_membership_root_is_identical() {
    let f = active_fixture();
    let mut ops = f.groups[0].operations().to_vec();
    ops.swap(1, 2);
    let genesis = Registry::bootstrap(&f.bootstrap)
        .unwrap()
        .bind_to_validated_block(block_hash(0));
    let reordered_state = genesis
        .stage_block(
            &Parent {
                height: 0,
                history: &f.history,
            },
            &ops,
        )
        .unwrap();
    assert_eq!(reordered_state.state(), f.registry.state());
    assert_eq!(
        reordered_state.state().root().unwrap(),
        f.expected().membership_root
    );
    assert_eq!(
        f.check(&replace_group(&f, 0, &ops)),
        Err(ReplayError::GroupCommitmentMismatch { height: 1 })
    );
}

#[test]
fn alternate_valid_withdrawal_bytes_cannot_replace_the_canonical_operation() {
    let mut f = active_fixture();
    let parent = f.registry.clone();
    let id = parent
        .state()
        .record_by_key(key(3).public_key())
        .unwrap()
        .id();
    f.append(&[withdraw(&f, 3, 1, id, 8)]);
    let alternative = withdraw(&f, 3, 1, id, 9);
    let alternate_state = parent
        .stage_block(
            &Parent {
                height: 1,
                history: &f.history,
            },
            &[alternative],
        )
        .unwrap();
    assert_eq!(
        alternate_state.state().root().unwrap(),
        f.expected().membership_root
    );
    assert_eq!(
        f.check(&replace_group(&f, 1, &[alternative])),
        Err(ReplayError::GroupCommitmentMismatch { height: 2 })
    );
}

#[test]
fn merging_blocks_does_not_let_a_new_member_review_in_its_activation_block() {
    let mut f = active_fixture();
    let first_root = f.registry.state().root().unwrap();
    let request = enroll(&f, 4, 8);
    let id = request.enrollment_id().unwrap();
    f.append(&[request, approve(&f, 3, 1, id, 8), approve(&f, 2, 1, id, 8)]);
    assert_eq!(f.check(&f.log()).unwrap().state().active_count(), 4);
    let combined: Vec<_> = f
        .groups
        .iter()
        .flat_map(|group| group.operations().iter().copied())
        .collect();
    let group = Group::new(f.config(), 1, block_hash(1), &combined).unwrap();
    let bad = Replay::new(f.config(), block_hash(0), &[group]).unwrap();
    let mut history = f.history.clone();
    history.blocks.get_mut(&1).unwrap().group_commitment =
        replay_group_commitment_v2(f.config(), 1, &combined).unwrap();
    let expected = ReplayExpectationV2 {
        config_hash: f.config().context(),
        height: 1,
        tip_hash: block_hash(1),
        membership_root: first_root,
    };
    assert_eq!(
        bad.replay(&f.bootstrap, &history, expected),
        Err(ReplayError::State {
            height: 1,
            error: StateError::ReviewerNotActive
        })
    );
}

#[test]
fn impossible_expected_height_rejects_before_any_trusted_history_lookup() {
    let f = active_fixture();
    let mut expected = f.expected();
    expected.height = u64::MAX;
    f.history.lookups.set(0);
    assert_eq!(
        f.log().replay(&f.bootstrap, &f.history, expected),
        Err(ReplayError::UnexpectedHeight)
    );
    assert_eq!(f.history.lookups.get(), 0);
}

#[test]
fn replay_caches_each_trusted_height_once_before_execution() {
    let f = lifecycle_fixture();
    f.history.lookups.set(0);
    assert_eq!(f.check(&f.log()).unwrap(), f.registry);
    assert_eq!(f.history.lookups.get(), f.groups.len() + 1);
}

#[test]
fn replay_work_budget_charges_configuration_copies_even_for_empty_groups() {
    let allocations: Vec<_> = (0..65_536u32)
        .map(|number| {
            let mut address = [0; 32];
            address[..4].copy_from_slice(&number.to_be_bytes());
            TestAllocation {
                address,
                balance_nusa: 0,
            }
        })
        .collect();
    let mut f = Fixture::with_allocations(LimitsV2::default(), &allocations);
    for _ in 0..14 {
        f.append(&[]);
    }
    // Each state costs 65,536 allocations + 2 config seeds + 2 records.
    // Genesis plus 14 groups: 15 * 65,540 = 983,100, within the 1,048,576 budget.
    assert_eq!(f.check(&f.log()).unwrap(), f.registry);
    f.append(&[]);
    // One more tiny empty group would cost 1,048,640 items, so reject before
    // cloning yet another large configuration. No membership or money changes.
    assert_eq!(f.check(&f.log()), Err(ReplayError::WorkLimit));
}

#[test]
fn late_failure_publishes_no_registry_and_preserves_inputs_for_retry() {
    let f = lifecycle_fixture();
    let bootstrap_before = f.bootstrap.clone();
    let registry_before = f.registry.clone();
    let good = f.log();
    let good_bytes = good.encode().unwrap();
    let mut history = f.history.clone();
    history.blocks.get_mut(&4).unwrap().membership_root = hash(99, 9);
    // The externally selected expectation agrees with the changed metadata,
    // so preflight succeeds and replay must discover the mismatch only after
    // executing the final group. This must not pass via UnexpectedRoot.
    let mut expected = f.expected();
    expected.membership_root = hash(99, 9);
    assert_eq!(
        good.replay(&f.bootstrap, &history, expected),
        Err(ReplayError::StateRootMismatch { height: 4 })
    );
    assert_eq!(f.bootstrap, bootstrap_before);
    assert_eq!(f.registry, registry_before);
    assert_eq!(good.encode().unwrap(), good_bytes);
    assert_eq!(f.check(&good).unwrap(), registry_before);
}

#[test]
fn construction_copies_inputs_and_decoding_never_establishes_replay_authority() {
    let f = active_fixture();
    let mut input = f.groups.clone();
    let log = Replay::new(f.config(), block_hash(0), &input).unwrap();
    input.clear();
    assert_eq!(log.groups().len(), 1);
    let mut bytes = log.encode().unwrap();
    let decoded = Replay::decode(f.config(), &bytes).unwrap();
    bytes.fill(0);
    assert_eq!(decoded.encode().unwrap(), log.encode().unwrap());
    let mut unknown = f.history.clone();
    unknown.blocks.clear();
    assert!(
        decoded
            .replay(&f.bootstrap, &unknown, f.expected())
            .is_err()
    );
}
