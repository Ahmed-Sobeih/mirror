//! Independent Python MPS2 / MPH2 vectors. All keys are PUBLIC synthetic fixtures.
//! Expected bytes, signatures and roots were computed before the Rust codec.
use std::collections::BTreeMap;

use mirror_crypto::{Hash256, Keypair};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipRegistryV2 as Registry, MembershipStatusV2 as Status, OperationHeaderV2 as Header,
    ParentChainViewV2, SignedMembershipOperationV2 as Op, UntrustedMembershipSnapshotV2,
};

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

fn key(number: u8) -> Keypair {
    Keypair::from_secret_bytes([number; 32])
}

fn block_hash(height: u64) -> Hash256 {
    hash(0xa0 + u8::try_from(height).unwrap())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn vectors() -> BTreeMap<&'static str, &'static str> {
    include_str!("vectors/state_v2.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split_once('=').unwrap())
        .collect()
}

fn config() -> ConfigV2 {
    let descriptors: Vec<_> = (1u8..=2)
        .map(|n| BootstrapDescriptor {
            public_key: key(n).public_key(),
            nonce: hash(0x30 + n),
            evidence: hash(0x40 + n),
        })
        .collect();
    ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(0x11),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile: hash(0x22),
        },
        LimitsV2::default(),
        &descriptors,
        &[],
    )
    .unwrap()
}

struct View {
    context: Hash256,
    height: u64,
}

impl ParentChainViewV2 for View {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        self.height
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height <= self.height).then(|| block_hash(height))
    }
}

fn sign(c: &ConfigV2, actor: u8, anchor: u64, until: u64, sequence: u64, action: Action) -> Op {
    Op::sign(
        Header {
            config_hash: c.context(),
            anchor_height: anchor,
            anchor_hash: block_hash(anchor),
            valid_from: anchor + 1,
            valid_until: until,
            actor_public_key: key(actor).public_key(),
            actor_sequence: sequence,
        },
        action,
        &key(actor),
    )
    .unwrap()
}

fn enroll(c: &ConfigV2, actor: u8, until: u64) -> Op {
    sign(
        c,
        actor,
        0,
        until,
        0,
        Action::Enroll {
            applicant_nonce: hash(0x50 + actor),
            evidence_commitment: hash(0x60 + actor),
            evidence_profile: hash(0x22),
        },
    )
}

struct Fixture {
    config: ConfigV2,
    operations: BTreeMap<&'static str, Op>,
    groups: Vec<Vec<Op>>,
    registries: Vec<Registry>,
}

fn fixture() -> Fixture {
    let config = config();
    let c = enroll(&config, 3, 4);
    let d = enroll(&config, 4, 2);
    let e = enroll(&config, 5, 5);
    let c_id = c.enrollment_id().unwrap();
    let d_id = d.enrollment_id().unwrap();
    let e_id = e.enrollment_id().unwrap();
    let a_c = sign(
        &config,
        1,
        0,
        4,
        0,
        Action::Approve {
            target_enrollment_id: c_id,
        },
    );
    let b_d = sign(
        &config,
        2,
        0,
        2,
        0,
        Action::Approve {
            target_enrollment_id: d_id,
        },
    );
    let b_c = sign(
        &config,
        2,
        1,
        4,
        1,
        Action::Approve {
            target_enrollment_id: c_id,
        },
    );
    let withdraw_e = sign(
        &config,
        5,
        1,
        5,
        1,
        Action::Withdraw {
            own_member_id: e_id,
        },
    );
    let withdraw_c = sign(
        &config,
        3,
        3,
        8,
        1,
        Action::Withdraw {
            own_member_id: c_id,
        },
    );
    let withdraw_a = sign(
        &config,
        1,
        3,
        8,
        1,
        Action::Withdraw {
            own_member_id: config.seed_id(0).unwrap(),
        },
    );
    let groups = vec![
        vec![c, a_c, d, b_d, e],
        vec![b_c, withdraw_e],
        vec![],
        vec![withdraw_c, withdraw_a],
    ];
    let proofs = [
        config.sign_seed(0, &key(1)).unwrap(),
        config.sign_seed(1, &key(2)).unwrap(),
    ];
    let genesis = Registry::bootstrap(&config.verify_bootstrap(&proofs).unwrap())
        .unwrap()
        .bind_to_validated_block(block_hash(0));
    let mut registries = vec![genesis];
    for operations in &groups {
        let parent = registries.last().unwrap();
        let view = View {
            context: config.context(),
            height: parent.state().height(),
        };
        let next = parent
            .stage_block(&view, operations)
            .unwrap()
            .bind_to_validated_block(block_hash(view.height + 1));
        registries.push(next);
    }
    Fixture {
        config,
        operations: [
            ("enroll_c", c),
            ("approve_a_c", a_c),
            ("enroll_d", d),
            ("approve_b_d", b_d),
            ("enroll_e", e),
            ("approve_b_c", b_c),
            ("withdraw_e", withdraw_e),
            ("withdraw_c", withdraw_c),
            ("withdraw_a", withdraw_a),
        ]
        .into(),
        groups,
        registries,
    }
}

fn check_state(height: usize) {
    let f = fixture();
    let v = vectors();
    let state = f.registries[height].state();
    let expected = unhex(v[format!("state{height}_hex").as_str()]);
    assert_eq!(state.encode().unwrap(), expected);
    assert_eq!(
        state.root().unwrap().to_hex(),
        v[format!("state{height}_root").as_str()]
    );
    assert_eq!(
        expected.len(),
        v[format!("state{height}_len").as_str()]
            .parse::<usize>()
            .unwrap()
    );
}

#[test]
fn configuration_and_signed_operations_match_independent_preimages() {
    let f = fixture();
    let v = vectors();
    assert_eq!(f.config.encode(), unhex(v["config_hex"]));
    assert_eq!(f.config.context().to_hex(), v["context"]);
    for index in 0..2 {
        assert_eq!(
            f.config.seed_id(index).unwrap().to_hex(),
            v[format!("seed{index}_id").as_str()]
        );
    }
    for (name, operation) in f.operations {
        assert_eq!(
            hex(&operation.unsigned_bytes()),
            v[format!("{name}_unsigned").as_str()]
        );
        assert_eq!(
            operation.signing_hash().to_hex(),
            v[format!("{name}_signing_hash").as_str()]
        );
        assert_eq!(
            operation.encode(),
            unhex(v[format!("{name}_signed").as_str()])
        );
        assert_eq!(operation.verify_signature(), Ok(()));
        if let Some(id) = operation.enrollment_id() {
            assert_eq!(id.to_hex(), v[format!("{name}_id").as_str()]);
        }
    }
}

#[test]
fn bootstrap_with_height_zero_activation_matches_independent_commitment() {
    check_state(0);
}

#[test]
fn pending_records_with_absent_activation_match_independent_commitment() {
    check_state(1);
}

#[test]
fn activation_and_pending_withdrawal_match_independent_commitment() {
    check_state(2);
}

#[test]
fn empty_block_expiry_with_retained_approval_matches_independent_commitment() {
    check_state(3);
}

#[test]
fn active_withdrawals_and_persistent_high_water_match_independent_commitment() {
    check_state(4);
    let f = fixture();
    let state = f.registries[4].state();
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.active_high_water(), 3);
    assert_eq!(state.total_activations(), 3);
    assert_eq!(state.retained_approval_count(), 3);
    assert_eq!(
        state
            .record_by_key(key(1).public_key())
            .unwrap()
            .activation()
            .unwrap()
            .height,
        0
    );
    assert_eq!(
        state.record_by_key(key(3).public_key()).unwrap().status(),
        Status::Revoked
    );
    assert_eq!(
        state.record_by_key(key(4).public_key()).unwrap().status(),
        Status::Expired
    );
    assert_eq!(
        state
            .record_by_key(key(5).public_key())
            .unwrap()
            .activation(),
        None
    );
}

#[test]
fn independent_snapshots_decode_but_match_only_the_exact_trusted_state() {
    let f = fixture();
    let v = vectors();
    for height in 0..=4 {
        let bytes = unhex(v[format!("state{height}_hex").as_str()]);
        let snapshot = UntrustedMembershipSnapshotV2::decode(&f.config, &bytes).unwrap();
        assert_eq!(snapshot.height(), height as u64);
        assert_eq!(snapshot.encode().unwrap(), bytes);
        assert_eq!(
            snapshot.root().unwrap().to_hex(),
            v[format!("state{height}_root").as_str()]
        );
        assert_eq!(
            snapshot.verify_against(f.registries[height].state()),
            Ok(())
        );
        assert!(
            snapshot
                .verify_against(f.registries[(height + 1) % 5].state())
                .is_err()
        );
    }
}

#[test]
fn commitment_excludes_external_block_hash_provenance() {
    let config = config();
    let proofs = [
        config.sign_seed(0, &key(1)).unwrap(),
        config.sign_seed(1, &key(2)).unwrap(),
    ];
    let candidate = Registry::bootstrap(&config.verify_bootstrap(&proofs).unwrap()).unwrap();
    let first = candidate.clone().bind_to_validated_block(hash(0xa0));
    let second = candidate.bind_to_validated_block(hash(0xb0));
    assert_ne!(first.tip_hash(), second.tip_hash());
    assert_eq!(first.state().encode(), second.state().encode());
    assert_eq!(first.state().root(), second.state().root());
    assert_eq!(
        first.state().root().unwrap().to_hex(),
        vectors()["state0_root"]
    );
}

#[test]
fn replay_encoding_and_group_commitments_match_independent_vectors() {
    use mirror_personhood::v2::{MembershipReplayV2, ReplayGroupV2, replay_group_commitment_v2};
    let f = fixture();
    let v = vectors();
    let genesis_commitment = replay_group_commitment_v2(&f.config, 0, &[]).unwrap();
    assert_eq!(genesis_commitment.to_hex(), v["group0_commitment"]);
    let groups: Vec<_> = f
        .groups
        .iter()
        .enumerate()
        .map(|(index, operations)| {
            let height = index as u64 + 1;
            assert_eq!(
                replay_group_commitment_v2(&f.config, height, operations)
                    .unwrap()
                    .to_hex(),
                v[format!("group{height}_commitment").as_str()]
            );
            ReplayGroupV2::new(&f.config, height, block_hash(height), operations).unwrap()
        })
        .collect();
    let replay = MembershipReplayV2::new(&f.config, block_hash(0), &groups).unwrap();
    let encoded = replay.encode().unwrap();
    assert_eq!(encoded, unhex(v["replay_hex"]));
    assert_eq!(encoded.len(), v["replay_len"].parse::<usize>().unwrap());
}

#[test]
fn replay_reconstructs_state_against_independently_pinned_roots_and_groups() {
    use mirror_personhood::v2::{
        MembershipReplayV2, ReplayChainViewV2, ReplayExpectationV2, TrustedReplayBlockV2,
    };
    struct IndependentView {
        context: Hash256,
        blocks: Vec<TrustedReplayBlockV2>,
    }
    impl ReplayChainViewV2 for IndependentView {
        fn config_hash(&self) -> Hash256 {
            self.context
        }
        fn block_at(&self, height: u64) -> Option<TrustedReplayBlockV2> {
            self.blocks.get(usize::try_from(height).ok()?).copied()
        }
    }
    let f = fixture();
    let v = vectors();
    let hash_from = |field: &str| Hash256::from_bytes(unhex(v[field]).try_into().unwrap());
    // These roots/commitments come from the independent Python fixture, never
    // from the untrusted replay object's claims or a Rust-produced expectation.
    let history = IndependentView {
        context: hash_from("context"),
        blocks: (0..=4)
            .map(|height| TrustedReplayBlockV2 {
                block_hash: hash_from(&format!("block{height}_hash")),
                group_commitment: hash_from(&format!("group{height}_commitment")),
                membership_root: hash_from(&format!("state{height}_root")),
            })
            .collect(),
    };
    let expected = ReplayExpectationV2 {
        config_hash: history.context,
        height: 4,
        tip_hash: hash_from("block4_hash"),
        membership_root: hash_from("state4_root"),
    };
    let bootstrap = f
        .config
        .verify_bootstrap(&[
            f.config.sign_seed(0, &key(1)).unwrap(),
            f.config.sign_seed(1, &key(2)).unwrap(),
        ])
        .unwrap();
    let untrusted = MembershipReplayV2::decode(&f.config, &unhex(v["replay_hex"])).unwrap();
    let recovered = untrusted.replay(&bootstrap, &history, expected).unwrap();
    assert_eq!(recovered, f.registries[4]);
    assert_eq!(recovered.state().encode().unwrap(), unhex(v["state4_hex"]));
    assert_eq!(recovered.state().root().unwrap(), expected.membership_root);
}
