//! Independent Python vectors for actual complete synthetic V2 histories.
//!
//! Expected signatures, mined headers, manifest and complete ledger states were
//! frozen before this Rust test ran. Unlike the earlier ledger-only vectors,
//! every operation anchor here is an actual accepted ancestor header hash.
//! All repeated-byte signing seeds and evidence are PUBLIC synthetic test data.

use std::collections::BTreeMap;

use mirror_chain::v2::{
    ChainErrorV2, ChainReplayErrorV2, ChainReplayExpectationV2, ChainV2, ManifestErrorV2,
    ManifestIdentityV2, NetworkManifestV2, genesis_template_v2,
};
use mirror_consensus::v2::{mine_header_v2, validate_header_pow_v2};
use mirror_core::{
    Address,
    v2::{
        BlockV2, SignedOperationV2 as Operation, SignedTransferV2, TransferBodyV2, TransferErrorV2,
    },
};
use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipStatusV2 as Status, OperationHeaderV2 as Header, SignedMembershipOperationV2,
    TestAllocation,
};
use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2};

fn repeated(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

fn key(number: u8) -> Keypair {
    Keypair::from_secret_bytes([number; 32])
}

fn address(number: u8) -> Address {
    Address::from_public_key(&key(number).public_key())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

fn config() -> ConfigV2 {
    let descriptors: Vec<_> = (1u8..=2)
        .map(|number| BootstrapDescriptor {
            public_key: key(number).public_key(),
            nonce: repeated(0x30 + number),
            evidence: repeated(0x40 + number),
        })
        .collect();
    let mut allocations = vec![
        TestAllocation {
            address: *address(1).as_bytes(),
            balance_nusa: 1000,
        },
        TestAllocation {
            address: *address(2).as_bytes(),
            balance_nusa: 0,
        },
        TestAllocation {
            address: *address(5).as_bytes(),
            balance_nusa: 0,
        },
    ];
    allocations.sort_by_key(|allocation| allocation.address);
    ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: repeated(0x11),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile: repeated(0x22),
        },
        LimitsV2 {
            max_operations: 8,
            max_membership_operations: 3,
            max_new_enrollments: 2,
            max_accounts: 8,
            ..LimitsV2::default()
        },
        &descriptors,
        &allocations,
    )
    .unwrap()
}

fn header(
    config: &ConfigV2,
    actor: u8,
    anchor: (u64, Hash256),
    until: u64,
    sequence: u64,
) -> Header {
    Header {
        config_hash: config.context(),
        anchor_height: anchor.0,
        anchor_hash: anchor.1,
        valid_from: anchor.0 + 1,
        valid_until: until,
        actor_public_key: key(actor).public_key(),
        actor_sequence: sequence,
    }
}

fn transfer(
    config: &ConfigV2,
    actor: u8,
    recipient: u8,
    anchor: (u64, Hash256),
    sequence: u64,
    amount: u64,
) -> Operation {
    SignedTransferV2::sign(
        header(config, actor, anchor, 8, sequence),
        TransferBodyV2 {
            recipient: address(recipient),
            amount,
            fee: 0,
        },
        &key(actor),
    )
    .unwrap()
    .into()
}

fn membership(
    config: &ConfigV2,
    actor: u8,
    anchor: (u64, Hash256),
    until: u64,
    sequence: u64,
    action: Action,
) -> SignedMembershipOperationV2 {
    SignedMembershipOperationV2::sign(
        header(config, actor, anchor, until, sequence),
        action,
        &key(actor),
    )
    .unwrap()
}

struct Fixture {
    values: BTreeMap<&'static str, &'static str>,
    config: ConfigV2,
    proofs: Vec<SignatureBytes>,
    manifest: NetworkManifestV2,
}

impl Fixture {
    fn new() -> Self {
        let values: BTreeMap<_, _> = include_str!("vectors/chain_v2.txt")
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| line.split_once('=').unwrap())
            .collect();
        let config = config();
        let proofs: Vec<_> = (0u32..2)
            .map(|index| config.sign_seed(index, &key(index as u8 + 1)).unwrap())
            .collect();
        let genesis = Hash256::from_bytes(unhex(values["block0_hash"]).try_into().unwrap());
        let manifest = NetworkManifestV2::new(&config, &proofs, genesis).unwrap();
        Self {
            values,
            config,
            proofs,
            manifest,
        }
    }

    fn hash(&self, field: &str) -> Hash256 {
        Hash256::from_bytes(unhex(self.values[field]).try_into().unwrap())
    }

    fn number(&self, field: &str) -> u64 {
        self.values[field].parse().unwrap()
    }

    fn block_bytes(&self, name: &str) -> Vec<u8> {
        unhex(self.values[format!("{name}_block").as_str()])
    }

    fn block(&self, name: &str) -> BlockV2 {
        BlockV2::decode(&self.config, &self.block_bytes(name)).unwrap()
    }

    fn expected(&self, name: &str, state: &str) -> ChainReplayExpectationV2 {
        ChainReplayExpectationV2 {
            config_hash: self.hash("context"),
            genesis_hash: self.hash("block0_hash"),
            height: self.number(&format!("{name}_height")),
            tip_hash: self.hash(&format!("{name}_hash")),
            ledger_root: self.hash(&format!("{state}_ledger_root")),
        }
    }

    fn main_blocks(&self, through: u64) -> Vec<BlockV2> {
        (0..=through)
            .map(|height| self.block(&format!("block{height}")))
            .collect()
    }

    fn main_chain(&self, through: u64) -> ChainV2 {
        ChainV2::replay(
            &self.manifest,
            &self.main_blocks(through),
            self.expected(&format!("block{through}"), &format!("state{through}")),
        )
        .unwrap()
    }

    fn assert_block(&self, name: &str, block: &BlockV2) {
        assert_eq!(block.hash(), self.hash(&format!("{name}_hash")), "{name}");
        assert_eq!(
            hex(&block.header().encode()),
            self.values[format!("{name}_header").as_str()],
            "{name} header"
        );
        assert_eq!(block.encode(&self.config).unwrap(), self.block_bytes(name));
        assert_eq!(
            block.header().nonce(),
            self.number(&format!("{name}_nonce"))
        );
        assert_eq!(
            block.header().operations_root(),
            self.hash(&format!("{name}_operations_root"))
        );
        block.verify_integrity(&self.config).unwrap();
        validate_header_pow_v2(block.header(), self.config.parameters().pow_bits).unwrap();
    }

    fn assert_state(&self, label: &str, chain: &ChainV2) {
        let ledger = chain.ledger();
        assert_eq!(
            hex(&ledger.account_bytes().unwrap()),
            self.values[format!("{label}_accounts_hex").as_str()],
            "{label} accounts"
        );
        assert_eq!(
            hex(&ledger.membership().encode().unwrap()),
            self.values[format!("{label}_membership_hex").as_str()],
            "{label} membership"
        );
        assert_eq!(
            ledger.accounts_root().unwrap(),
            self.hash(&format!("{label}_accounts_root"))
        );
        assert_eq!(
            ledger.membership().root().unwrap(),
            self.hash(&format!("{label}_membership_root"))
        );
        assert_eq!(ledger.economics_root(), self.hash("economics_root"));
        assert_eq!(
            ledger.root().unwrap(),
            self.hash(&format!("{label}_ledger_root"))
        );
        assert_eq!(chain.checkpoint().ledger_root, ledger.root().unwrap());
        assert_eq!(ledger.initial_test_supply(), 1000);
        assert_eq!(
            ledger
                .accounts()
                .map(|(_, account)| account.balance())
                .sum::<u64>(),
            1000
        );
        assert!(
            ledger.contains_account(address(5)),
            "explicit zero account retained"
        );
        assert_eq!(ledger.account(address(5)).balance(), 0);
        assert_eq!(ledger.tip_hash(), chain.blocks().last().unwrap().hash());
    }

    // Re-sign human-readable intents in Rust and compare against independently
    // signed Python bytes. Actual anchor hashes are fixed in the reference and
    // separately regenerated from complete accepted block construction below.
    fn operations(&self) -> BTreeMap<&'static str, Operation> {
        let config = &self.config;
        let a0 = (0, self.hash("block0_hash"));
        let a1 = (1, self.hash("block1_hash"));
        let a2 = (2, self.hash("block2_hash"));
        let a3 = (3, self.hash("block3_hash"));
        let f2 = (2, self.hash("fork2_hash"));
        let enroll_c = membership(
            config,
            3,
            a0,
            5,
            0,
            Action::Enroll {
                applicant_nonce: repeated(0x53),
                evidence_commitment: repeated(0x63),
                evidence_profile: repeated(0x22),
            },
        );
        let enroll_d = membership(
            config,
            4,
            a0,
            1,
            0,
            Action::Enroll {
                applicant_nonce: repeated(0x54),
                evidence_commitment: repeated(0x64),
                evidence_profile: repeated(0x22),
            },
        );
        let c_id = enroll_c.enrollment_id().unwrap();
        assert_eq!(c_id, self.hash("c_member_id"));
        assert_eq!(enroll_d.enrollment_id().unwrap(), self.hash("d_member_id"));
        let approve_c = Action::Approve {
            target_enrollment_id: c_id,
        };
        let withdraw_a = Action::Withdraw {
            own_member_id: config.seed_id(0).unwrap(),
        };
        BTreeMap::from([
            ("h1_a_to_b", transfer(config, 1, 2, a0, 0, 100)),
            ("h1_a_self", transfer(config, 1, 1, a0, 1, 10)),
            ("h1_b_to_c", transfer(config, 2, 3, a0, 0, 40)),
            ("enroll_c", enroll_c.into()),
            ("h1_c_to_a", transfer(config, 3, 1, a0, 0, 7)),
            (
                "approve_a_c",
                membership(config, 1, a0, 5, 0, approve_c).into(),
            ),
            ("enroll_d", enroll_d.into()),
            ("h3_b_self", transfer(config, 2, 2, a2, 1, 5)),
            (
                "approve_b_c",
                membership(config, 2, a2, 5, 0, approve_c).into(),
            ),
            ("h3_a_to_c", transfer(config, 1, 3, a2, 2, 17)),
            (
                "withdraw_c",
                membership(
                    config,
                    3,
                    a3,
                    8,
                    1,
                    Action::Withdraw {
                        own_member_id: c_id,
                    },
                )
                .into(),
            ),
            ("h4_c_to_b", transfer(config, 3, 2, a3, 1, 36)),
            (
                "withdraw_a",
                membership(config, 1, a3, 8, 1, withdraw_a).into(),
            ),
            ("h4_a_to_b", transfer(config, 1, 2, a3, 3, 499)),
            (
                "fork2_withdraw_a",
                membership(config, 1, a1, 8, 1, withdraw_a).into(),
            ),
            ("fork3_b_self", transfer(config, 2, 2, f2, 1, 5)),
            (
                "fork3_approve_b_c",
                membership(config, 2, f2, 5, 0, approve_c).into(),
            ),
            ("fork3_a_to_c", transfer(config, 1, 3, f2, 2, 17)),
        ])
    }
}

fn mine(config: &ConfigV2, proposal: &BlockV2) -> BlockV2 {
    let result =
        mine_header_v2(*proposal.header(), config.parameters().pow_bits, 1_000_000).unwrap();
    assert_eq!(result.hash, result.header.hash());
    assert_eq!(
        result.attempts,
        result.header.nonce() - proposal.header().nonce() + 1
    );
    BlockV2::new(config, result.header, proposal.operations()).unwrap()
}

#[test]
fn independently_encoded_manifest_and_all_seed_proofs_match() {
    let fixture = Fixture::new();
    assert_eq!(hex(&fixture.config.encode()), fixture.values["config_hex"]);
    assert_eq!(fixture.config.context(), fixture.hash("context"));
    for (index, proof) in fixture.proofs.iter().enumerate() {
        assert_eq!(
            hex(proof.as_bytes()),
            fixture.values[format!("seed{index}_signature").as_str()]
        );
        assert_eq!(
            fixture.config.seed_id(index as u32).unwrap(),
            fixture.hash(&format!("seed{index}_id"))
        );
        assert_eq!(
            fixture.config.seed_signing_hash(index as u32).unwrap(),
            fixture.hash(&format!("seed{index}_signing_hash"))
        );
    }
    let bytes = fixture.manifest.encode().unwrap();
    assert_eq!(bytes.len(), 636);
    assert_eq!(hex(&bytes), fixture.values["manifest_hex"]);
    let identity = ManifestIdentityV2 {
        config_hash: fixture.hash("context"),
        genesis_hash: fixture.hash("block0_hash"),
    };
    assert_eq!(fixture.manifest.identity(), identity);
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, identity).unwrap(),
        fixture.manifest
    );
    assert_eq!(
        NetworkManifestV2::decode_for(
            &bytes,
            ManifestIdentityV2 {
                config_hash: repeated(0xab),
                ..identity
            }
        ),
        Err(ManifestErrorV2::WrongConfiguration)
    );
    assert_eq!(
        NetworkManifestV2::decode_for(
            &bytes,
            ManifestIdentityV2 {
                genesis_hash: fixture.hash("alternate_genesis_hash"),
                ..identity
            }
        ),
        Err(ManifestErrorV2::WrongGenesis)
    );
}

#[test]
fn independently_signed_operations_and_actual_anchor_intents_match() {
    let fixture = Fixture::new();
    let operations = fixture.operations();
    assert_eq!(operations.len(), 18);
    for (name, operation) in operations {
        assert_eq!(
            hex(&operation.encode()),
            fixture.values[format!("{name}_signed").as_str()],
            "{name}"
        );
        let unsigned = match operation {
            Operation::Transfer(value) => value.unsigned_bytes(),
            Operation::Membership(value) => value.unsigned_bytes(),
        };
        assert_eq!(
            hex(&unsigned),
            fixture.values[format!("{name}_unsigned").as_str()]
        );
        assert_eq!(
            operation.signing_hash(),
            fixture.hash(&format!("{name}_signing_hash"))
        );
        assert_eq!(operation.id(), fixture.hash(&format!("{name}_id")));
        operation.verify_signature().unwrap();
        assert_eq!(
            Operation::decode(&fixture.config, &operation.encode()).unwrap(),
            operation
        );
    }
}

#[test]
fn independently_mined_genesis_and_mixed_blocks_match_complete_ledger_states() {
    let fixture = Fixture::new();
    let genesis = mine(
        &fixture.config,
        &genesis_template_v2(fixture.manifest.bootstrap()).unwrap(),
    );
    fixture.assert_block("block0", &genesis);
    let mut chain = ChainV2::from_genesis(&fixture.manifest, genesis).unwrap();
    fixture.assert_state("state0", &chain);
    let operations = fixture.operations();
    let c_id = fixture.hash("c_member_id");
    let d_id = fixture.hash("d_member_id");
    for height in 1..=4 {
        let name = format!("block{height}");
        let names = fixture.values[format!("{name}_operations").as_str()];
        let body: Vec<_> = names
            .split(',')
            .filter(|name| !name.is_empty())
            .map(|name| operations[name])
            .collect();
        let before = chain.clone();
        let proposal = chain.prepare_candidate(height, &body).unwrap();
        assert_eq!(
            chain, before,
            "preparing a block must not publish staged state"
        );
        let block = mine(&fixture.config, &proposal);
        fixture.assert_block(&name, &block);
        chain.append(block).unwrap();
        fixture.assert_state(&format!("state{height}"), &chain);
        assert_eq!(
            chain.checkpoint(),
            fixture.expected(&name, &format!("state{height}"))
        );
        let membership = chain.ledger().membership();
        assert_eq!(
            membership.record(c_id).unwrap().inclusion().operation_index,
            3
        );
        assert_eq!(
            membership.record(d_id).unwrap().inclusion().operation_index,
            6
        );
        if height == 2 {
            assert!(chain.blocks().last().unwrap().operations().is_empty());
            assert_eq!(membership.record(d_id).unwrap().status(), Status::Expired);
            assert_eq!(membership.record(c_id).unwrap().status(), Status::Pending);
        }
        if height == 3 {
            assert_eq!(membership.record(c_id).unwrap().status(), Status::Active);
            assert_eq!(
                membership
                    .record(c_id)
                    .unwrap()
                    .activation()
                    .unwrap()
                    .height,
                3
            );
        }
    }
    assert_eq!(chain.ledger().account(address(1)).balance(), 391);
    assert_eq!(chain.ledger().account(address(1)).next_sequence(), 4);
    assert_eq!(chain.ledger().account(address(2)).balance(), 595);
    assert_eq!(chain.ledger().account(address(3)).balance(), 14);
    assert_eq!(
        chain.ledger().membership().record(c_id).unwrap().status(),
        Status::Revoked
    );
    assert_eq!(
        chain
            .ledger()
            .membership()
            .record(c_id)
            .unwrap()
            .next_sequence(),
        2
    );
}

#[test]
fn independent_complete_block_bytes_replay_to_the_selected_checkpoint() {
    let fixture = Fixture::new();
    let expected = fixture.expected("block4", "state4");
    let typed = ChainV2::replay(&fixture.manifest, &fixture.main_blocks(4), expected).unwrap();
    let owned: Vec<_> = (0..=4)
        .map(|height| fixture.block_bytes(&format!("block{height}")))
        .collect();
    let borrowed: Vec<_> = owned.iter().map(Vec::as_slice).collect();
    let encoded = ChainV2::replay_encoded(&fixture.manifest, &borrowed, expected).unwrap();
    assert_eq!(typed, encoded);
    assert_eq!(typed.checkpoint(), expected);
    assert_eq!(
        typed.encoded_bytes(),
        owned.iter().map(Vec::len).sum::<usize>()
    );
    fixture.assert_state("state4", &encoded);
}

#[test]
fn alternate_valid_genesis_nonce_cannot_replace_the_selected_identity() {
    let fixture = Fixture::new();
    let alternate = fixture.block("alternate_genesis");
    fixture.assert_block("alternate_genesis", &alternate);
    assert_ne!(alternate.hash(), fixture.manifest.genesis_hash());
    assert_eq!(
        alternate.header().ledger_root(),
        fixture.hash("state0_ledger_root")
    );
    assert_eq!(
        ChainV2::from_genesis(&fixture.manifest, alternate.clone()),
        Err(ChainErrorV2::WrongGenesisHash)
    );
    // Explicitly selecting this other genesis defines another network identity.
    // The incoming block itself cannot change an already selected manifest.
    let selected_other =
        NetworkManifestV2::new(&fixture.config, &fixture.proofs, alternate.hash()).unwrap();
    let other_chain = ChainV2::from_genesis(&selected_other, alternate).unwrap();
    fixture.assert_state("state0", &other_chain);
    assert_ne!(
        other_chain.manifest().identity(),
        fixture.manifest.identity()
    );
}

#[test]
fn independently_remined_wrong_ledger_root_is_rejected_without_partial_publication() {
    let fixture = Fixture::new();
    let mut chain = fixture.main_chain(0);
    let before = chain.clone();
    let invalid = fixture.block("wrong_root1");
    fixture.assert_block("wrong_root1", &invalid);
    assert_eq!(invalid.operations(), fixture.block("block1").operations());
    assert_eq!(invalid.header().previous_hash(), chain.tip_hash());
    assert_ne!(
        invalid.header().ledger_root(),
        fixture.hash("state1_ledger_root")
    );
    assert_eq!(chain.append(invalid), Err(ChainErrorV2::WrongLedgerRoot));
    assert_eq!(
        chain, before,
        "accounts, membership, prefix and resource counters stay unchanged"
    );
    fixture.assert_state("state0", &chain);
}

#[test]
fn real_membership_fork_replays_distinct_membership_with_identical_accounts() {
    let fixture = Fixture::new();
    let main = fixture.main_chain(3);
    let mut fork = fixture.main_chain(1);
    let operations = fixture.operations();
    for height in 2..=3 {
        let name = format!("fork{height}");
        let body: Vec<_> = fixture.values[format!("{name}_operations").as_str()]
            .split(',')
            .map(|name| operations[name])
            .collect();
        let candidate = mine(
            &fixture.config,
            &fork.prepare_candidate(height, &body).unwrap(),
        );
        fixture.assert_block(&name, &candidate);
        fork.append(candidate).unwrap();
        fixture.assert_state(&format!("fork_state{height}"), &fork);
    }
    assert_eq!(
        main.ledger().account_bytes().unwrap(),
        fork.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        main.ledger().membership().root().unwrap(),
        fork.ledger().membership().root().unwrap()
    );
    assert_ne!(main.tip_hash(), fork.tip_hash());
    assert_eq!(
        main.ledger()
            .membership()
            .record(fixture.hash("c_member_id"))
            .unwrap()
            .status(),
        Status::Active
    );
    assert_eq!(
        fork.ledger()
            .membership()
            .record(fixture.hash("c_member_id"))
            .unwrap()
            .status(),
        Status::Pending
    );
    assert_eq!(
        fork.ledger()
            .membership()
            .record(fixture.hash("seed0_id"))
            .unwrap()
            .status(),
        Status::Revoked
    );
    assert_eq!(
        ChainV2::replay(
            &fixture.manifest,
            fork.blocks(),
            fixture.expected("fork3", "fork_state3")
        )
        .unwrap(),
        fork
    );
}

#[test]
fn valid_work_and_signatures_cannot_authorize_a_different_forks_anchor() {
    let fixture = Fixture::new();
    let mut fork = fixture.main_chain(1);
    fork.append(fixture.block("fork2")).unwrap();
    let before = fork.clone();
    let candidate = fixture.block("cross_anchor3");
    fixture.assert_block("cross_anchor3", &candidate);
    assert_eq!(candidate.header().previous_hash(), fork.tip_hash());
    assert_eq!(candidate.operations(), fixture.block("block3").operations());
    assert_eq!(
        candidate.operations()[0].header().anchor_hash,
        fixture.hash("block2_hash")
    );
    assert_ne!(
        candidate.operations()[0].header().anchor_hash,
        fork.tip_hash()
    );
    assert_eq!(
        fork.append(candidate),
        Err(ChainErrorV2::Ledger(LedgerErrorV2::Operation {
            index: 0,
            error: LedgerOperationErrorV2::Transfer(TransferErrorV2::WrongAnchorHash),
        }))
    );
    assert_eq!(fork, before);
    fixture.assert_state("fork_state2", &fork);
}

#[test]
fn authentic_old_prefix_or_wrong_expected_checkpoint_is_not_latest_tip_authority() {
    let fixture = Fixture::new();
    let blocks = fixture.main_blocks(4);
    let expected = fixture.expected("block4", "state4");
    assert_eq!(
        ChainV2::replay(&fixture.manifest, &blocks[..2], expected),
        Err(ChainReplayErrorV2::WrongExpectedHeight {
            expected: 4,
            got: 1
        })
    );
    for (altered, error) in [
        (
            ChainReplayExpectationV2 {
                config_hash: repeated(0xe0),
                ..expected
            },
            ChainReplayErrorV2::WrongExpectedConfiguration,
        ),
        (
            ChainReplayExpectationV2 {
                genesis_hash: fixture.hash("alternate_genesis_hash"),
                ..expected
            },
            ChainReplayErrorV2::WrongExpectedGenesis,
        ),
        (
            ChainReplayExpectationV2 {
                tip_hash: fixture.hash("fork3_hash"),
                ..expected
            },
            ChainReplayErrorV2::WrongExpectedTip,
        ),
        (
            ChainReplayExpectationV2 {
                ledger_root: fixture.hash("fork_state3_ledger_root"),
                ..expected
            },
            ChainReplayErrorV2::WrongExpectedLedgerRoot,
        ),
    ] {
        assert_eq!(
            ChainV2::replay(&fixture.manifest, &blocks, altered),
            Err(error)
        );
    }
}
