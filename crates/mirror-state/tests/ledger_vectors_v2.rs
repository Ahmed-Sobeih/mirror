//! Independent Python mixed-ledger V2 vectors. Every key/seed is PUBLIC synthetic data.
//! Expected signatures, account bytes, MPS2 bytes and roots predate Rust execution.
//! Repeated-byte tip hashes provide trusted test provenance, not block validation.
use std::collections::BTreeMap;

use mirror_core::{
    Address,
    v2::{SignedOperationV2 as Operation, SignedTransferV2, TransferBodyV2},
};
use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipStatusV2 as Status, OperationHeaderV2 as Header, ParentChainViewV2,
    SignedMembershipOperationV2 as MembershipOperation, TestAllocation,
    UntrustedMembershipSnapshotV2,
};
use mirror_state::v2::{LedgerV2, StagedLedgerV2};

const GROUPS: [&[&str]; 4] = [
    &[
        "h1_a_to_b",
        "h1_a_self",
        "h1_b_to_c",
        "enroll_c",
        "h1_c_to_a",
        "approve_a_c",
        "enroll_d",
    ],
    &[],
    &["h3_b_self", "approve_b_c", "h3_a_to_c"],
    &["withdraw_c", "h4_c_to_b", "withdraw_a", "h4_a_to_b"],
];

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

fn key(number: u8) -> Keypair {
    Keypair::from_secret_bytes([number; 32])
}

fn address(number: u8) -> Address {
    Address::from_public_key(&key(number).public_key())
}

fn block_hash(height: u64) -> Hash256 {
    hash(0xa0 + u8::try_from(height).unwrap())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

fn vectors() -> BTreeMap<&'static str, &'static str> {
    include_str!("vectors/ledger_v2.txt")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| line.split_once('=').unwrap())
        .collect()
}

fn config() -> ConfigV2 {
    let descriptors: Vec<_> = (1u8..=2)
        .map(|number| BootstrapDescriptor {
            public_key: key(number).public_key(),
            nonce: hash(0x30 + number),
            evidence: hash(0x40 + number),
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
            network: hash(0x11),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile: hash(0x22),
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

fn header(config: &ConfigV2, actor: u8, anchor: u64, until: u64, sequence: u64) -> Header {
    Header {
        config_hash: config.context(),
        anchor_height: anchor,
        anchor_hash: block_hash(anchor),
        valid_from: anchor + 1,
        valid_until: until,
        actor_public_key: key(actor).public_key(),
        actor_sequence: sequence,
    }
}

fn transfer(
    config: &ConfigV2,
    actor: u8,
    recipient: u8,
    anchor: u64,
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

fn member(
    config: &ConfigV2,
    actor: u8,
    anchor: u64,
    until: u64,
    sequence: u64,
    action: Action,
) -> MembershipOperation {
    MembershipOperation::sign(
        header(config, actor, anchor, until, sequence),
        action,
        &key(actor),
    )
    .unwrap()
}

fn operations(config: &ConfigV2) -> BTreeMap<&'static str, Operation> {
    let enroll_c = member(
        config,
        3,
        0,
        5,
        0,
        Action::Enroll {
            applicant_nonce: hash(0x53),
            evidence_commitment: hash(0x63),
            evidence_profile: hash(0x22),
        },
    );
    let enroll_d = member(
        config,
        4,
        0,
        1,
        0,
        Action::Enroll {
            applicant_nonce: hash(0x54),
            evidence_commitment: hash(0x64),
            evidence_profile: hash(0x22),
        },
    );
    let c_id = enroll_c.enrollment_id().unwrap();
    BTreeMap::from([
        ("h1_a_to_b", transfer(config, 1, 2, 0, 0, 100)),
        ("h1_a_self", transfer(config, 1, 1, 0, 1, 10)),
        ("h1_b_to_c", transfer(config, 2, 3, 0, 0, 40)),
        ("enroll_c", enroll_c.into()),
        ("h1_c_to_a", transfer(config, 3, 1, 0, 0, 7)),
        (
            "approve_a_c",
            member(
                config,
                1,
                0,
                5,
                0,
                Action::Approve {
                    target_enrollment_id: c_id,
                },
            )
            .into(),
        ),
        ("enroll_d", enroll_d.into()),
        ("h3_b_self", transfer(config, 2, 2, 2, 1, 5)),
        (
            "approve_b_c",
            member(
                config,
                2,
                2,
                5,
                0,
                Action::Approve {
                    target_enrollment_id: c_id,
                },
            )
            .into(),
        ),
        ("h3_a_to_c", transfer(config, 1, 3, 2, 2, 17)),
        (
            "withdraw_c",
            member(
                config,
                3,
                3,
                8,
                1,
                Action::Withdraw {
                    own_member_id: c_id,
                },
            )
            .into(),
        ),
        ("h4_c_to_b", transfer(config, 3, 2, 3, 1, 36)),
        (
            "withdraw_a",
            member(
                config,
                1,
                3,
                8,
                1,
                Action::Withdraw {
                    own_member_id: config.seed_id(0).unwrap(),
                },
            )
            .into(),
        ),
        ("h4_a_to_b", transfer(config, 1, 2, 3, 3, 499)),
    ])
}

fn bootstrap(config: &ConfigV2) -> StagedLedgerV2 {
    let proofs = [
        config.sign_seed(0, &key(1)).unwrap(),
        config.sign_seed(1, &key(2)).unwrap(),
    ];
    LedgerV2::bootstrap(&config.verify_bootstrap(&proofs).unwrap()).unwrap()
}

fn history() -> Vec<LedgerV2> {
    let config = config();
    let operations = operations(&config);
    let mut ledgers = vec![bootstrap(&config).bind_to_validated_block(block_hash(0))];
    for names in GROUPS {
        let parent = ledgers.last().unwrap();
        let group: Vec<_> = names.iter().map(|name| operations[name]).collect();
        let view = View {
            context: config.context(),
            height: parent.height(),
        };
        let staged = parent.stage_operations(&view, &group).unwrap();
        ledgers.push(staged.bind_to_validated_block(block_hash(view.height + 1)));
    }
    ledgers
}

fn assert_reference(ledger: &LedgerV2, label: &str, expected: &BTreeMap<&str, &str>) {
    assert_eq!(
        ledger.height().to_string(),
        expected[format!("{label}_height").as_str()]
    );
    assert_eq!(ledger.initial_test_supply(), 1000);
    assert_eq!(
        ledger.account_count().to_string(),
        expected[format!("{label}_account_count").as_str()]
    );
    assert_eq!(
        hex(&ledger.account_bytes().unwrap()),
        expected[format!("{label}_accounts_hex").as_str()]
    );
    assert_eq!(
        ledger.accounts_root().unwrap().to_hex(),
        expected[format!("{label}_accounts_root").as_str()]
    );
    let membership_bytes = ledger.membership().encode().unwrap();
    assert_eq!(
        membership_bytes.len().to_string(),
        expected[format!("{label}_membership_len").as_str()]
    );
    assert_eq!(
        hex(&membership_bytes),
        expected[format!("{label}_membership_hex").as_str()]
    );
    assert_eq!(
        ledger.membership().root().unwrap().to_hex(),
        expected[format!("{label}_membership_root").as_str()]
    );
    assert_eq!(ledger.economics_root().to_hex(), expected["economics_root"]);
    let mut preimage = ledger.config().context().as_bytes().to_vec();
    preimage.extend_from_slice(&ledger.height().to_le_bytes());
    preimage.extend_from_slice(ledger.accounts_root().unwrap().as_bytes());
    preimage.extend_from_slice(ledger.membership().root().unwrap().as_bytes());
    preimage.extend_from_slice(ledger.economics_root().as_bytes());
    assert_eq!(preimage.len(), 136);
    assert_eq!(
        hex(&preimage),
        expected[format!("{label}_ledger_preimage").as_str()]
    );
    assert_eq!(
        ledger.root().unwrap().to_hex(),
        expected[format!("{label}_ledger_root").as_str()]
    );
    let decoded =
        UntrustedMembershipSnapshotV2::decode(ledger.config(), &membership_bytes).unwrap();
    decoded.verify_against(ledger.membership()).unwrap();
}

#[test]
fn independent_config_addresses_and_signed_operation_vectors() {
    let expected = vectors();
    let config = config();
    assert_eq!(config.encode().len().to_string(), expected["config_len"]);
    assert_eq!(hex(&config.encode()), expected["config_hex"]);
    assert_eq!(config.context().to_hex(), expected["context"]);
    for actor in 1u8..=5 {
        assert_eq!(
            hex(address(actor).as_bytes()),
            expected[format!("address_{}", char::from(b'a' + actor - 1)).as_str()]
        );
    }
    for index in 0u32..2 {
        assert_eq!(
            config.seed_id(index).unwrap().to_hex(),
            expected[format!("seed{index}_id").as_str()]
        );
        assert_eq!(
            config.seed_signing_hash(index).unwrap().to_hex(),
            expected[format!("seed{index}_signing_hash").as_str()]
        );
        assert_eq!(
            hex(config
                .sign_seed(index, &key(index as u8 + 1))
                .unwrap()
                .as_bytes()),
            expected[format!("seed{index}_signature").as_str()]
        );
    }
    let operations = operations(&config);
    assert_eq!(operations.len(), 14);
    for (name, operation) in operations {
        let unsigned = match operation {
            Operation::Transfer(transfer) => transfer.unsigned_bytes(),
            Operation::Membership(member) => member.unsigned_bytes(),
        };
        assert_eq!(
            hex(&unsigned),
            expected[format!("{name}_unsigned").as_str()]
        );
        assert_eq!(
            operation.signing_hash().to_hex(),
            expected[format!("{name}_signing_hash").as_str()]
        );
        assert_eq!(
            hex(&operation.encode()),
            expected[format!("{name}_signed").as_str()]
        );
        assert_eq!(
            operation.id().to_hex(),
            expected[format!("{name}_id").as_str()]
        );
        operation.verify_signature().unwrap();
        if let Operation::Membership(member) = operation
            && let Some(member_id) = member.enrollment_id()
        {
            assert_eq!(
                member_id.to_hex(),
                expected[format!("{name}_member_id").as_str()]
            );
        }
    }
}

#[test]
fn independent_accounts_membership_economics_and_combined_roots() {
    let expected = vectors();
    let ledgers = history();
    assert_eq!(ledgers.len(), 5);
    for (height, ledger) in ledgers.iter().enumerate() {
        assert_reference(ledger, &format!("state{height}"), &expected);
        assert_eq!(
            ledger.tip_hash().to_hex(),
            expected[format!("block{height}_hash").as_str()]
        );
    }
}

#[test]
fn signed_python_fixture_decodes_and_reexecutes_from_verified_bootstrap() {
    let expected = vectors();
    let config = ConfigV2::decode(&unhex(expected["config_hex"])).unwrap();
    let proofs: Vec<_> = (0..2)
        .map(|index| {
            SignatureBytes::from_bytes(
                unhex(expected[format!("seed{index}_signature").as_str()])
                    .try_into()
                    .unwrap(),
            )
        })
        .collect();
    let mut ledger = LedgerV2::bootstrap(&config.verify_bootstrap(&proofs).unwrap())
        .unwrap()
        .bind_to_validated_block(block_hash(0));
    assert_reference(&ledger, "state0", &expected);
    for height in 1..=4 {
        let group = expected[format!("block{height}_operations").as_str()];
        let operations: Vec<_> = group
            .split(',')
            .filter(|name| !name.is_empty())
            .map(|name| {
                Operation::decode(&config, &unhex(expected[format!("{name}_signed").as_str()]))
                    .unwrap()
            })
            .collect();
        let staged = ledger
            .stage_operations(
                &View {
                    context: config.context(),
                    height: height - 1,
                },
                &operations,
            )
            .unwrap();
        assert_eq!(
            staged.root().unwrap().to_hex(),
            expected[format!("state{height}_ledger_root").as_str()]
        );
        ledger = staged.bind_to_validated_block(block_hash(height));
        assert_reference(&ledger, &format!("state{height}"), &expected);
    }
}

#[test]
fn full_operation_positions_survive_interspersed_transfers_and_are_committed() {
    let expected = vectors();
    let config = config();
    let operations = operations(&config);
    let parent = bootstrap(&config).bind_to_validated_block(block_hash(0));
    let view = View {
        context: config.context(),
        height: 0,
    };
    let mut group: Vec<_> = GROUPS[0].iter().map(|name| operations[name]).collect();
    let normal = parent
        .stage_operations(&view, &group)
        .unwrap()
        .bind_to_validated_block(block_hash(1));
    // C's transfer is independent of its enrollment; both orders are valid.
    group.swap(3, 4);
    let alternate = parent
        .stage_operations(&view, &group)
        .unwrap()
        .bind_to_validated_block(block_hash(1));
    assert_reference(&normal, "state1", &expected);
    assert_reference(&alternate, "alternate1", &expected);
    assert_eq!(
        normal.account_bytes().unwrap(),
        alternate.account_bytes().unwrap()
    );
    assert_eq!(
        normal.accounts_root().unwrap(),
        alternate.accounts_root().unwrap()
    );
    assert_ne!(
        normal.membership().root().unwrap(),
        alternate.membership().root().unwrap()
    );
    assert_ne!(normal.root().unwrap(), alternate.root().unwrap());
    assert_eq!(
        normal
            .membership()
            .record_by_key(key(3).public_key())
            .unwrap()
            .inclusion()
            .operation_index,
        3
    );
    assert_eq!(
        alternate
            .membership()
            .record_by_key(key(3).public_key())
            .unwrap()
            .inclusion()
            .operation_index,
        4
    );
    for ledger in [&normal, &alternate] {
        assert_eq!(
            ledger
                .membership()
                .record_by_key(key(4).public_key())
                .unwrap()
                .inclusion()
                .operation_index,
            6
        );
        assert_eq!(ledger.config().limits().max_membership_operations, 3);
        assert_eq!(ledger.config().limits().max_operations, 8);
    }
}

#[test]
fn empty_block_expiry_changes_only_membership_and_height_commitments() {
    let ledgers = history();
    let before = &ledgers[1];
    let after = &ledgers[2];
    assert_eq!(
        before.account_bytes().unwrap(),
        after.account_bytes().unwrap()
    );
    assert_eq!(
        before.accounts_root().unwrap(),
        after.accounts_root().unwrap()
    );
    assert_eq!(before.economics_root(), after.economics_root());
    assert_ne!(
        before.membership().root().unwrap(),
        after.membership().root().unwrap()
    );
    assert_ne!(before.root().unwrap(), after.root().unwrap());
    assert_eq!(before.membership().pending_count(), 2);
    assert_eq!(after.membership().pending_count(), 1);
    assert_eq!(
        before
            .membership()
            .record_by_key(key(4).public_key())
            .unwrap()
            .status(),
        Status::Pending
    );
    assert_eq!(
        after
            .membership()
            .record_by_key(key(4).public_key())
            .unwrap()
            .status(),
        Status::Expired
    );
}

#[test]
fn balances_and_account_sequences_are_separate_from_membership_sequences() {
    let ledgers = history();
    let expected_accounts = [
        [(1000, 0), (0, 0), (0, 0)],
        [(907, 2), (60, 1), (33, 1)],
        [(907, 2), (60, 1), (33, 1)],
        [(890, 3), (60, 2), (50, 1)],
        [(391, 4), (595, 2), (14, 2)],
    ];
    for (ledger, accounts) in ledgers.iter().zip(expected_accounts) {
        for (actor, (balance, sequence)) in (1u8..=3).zip(accounts) {
            let account = ledger.account(address(actor));
            assert_eq!(account.balance(), balance);
            assert_eq!(account.next_sequence(), sequence);
        }
    }
    // Transfers and reviews consume distinct counters, including the same block.
    assert_eq!(ledgers[1].account(address(1)).next_sequence(), 2);
    assert_eq!(
        ledgers[1]
            .membership()
            .record_by_key(key(1).public_key())
            .unwrap()
            .next_sequence(),
        1
    );
    assert_eq!(ledgers[3].account(address(2)).next_sequence(), 2);
    assert_eq!(
        ledgers[3]
            .membership()
            .record_by_key(key(2).public_key())
            .unwrap()
            .next_sequence(),
        1
    );
    let c = ledgers[3]
        .membership()
        .record_by_key(key(3).public_key())
        .unwrap();
    assert_eq!(c.activation().unwrap().height, 3);
    assert_eq!(c.activation().unwrap().ordinal, 3);
    assert_eq!(c.approval_count(), 2);
    let final_state = ledgers[4].membership();
    assert_eq!(final_state.active_count(), 1);
    assert_eq!(final_state.active_high_water(), 3);
    assert_eq!(final_state.total_activations(), 3);
    assert_eq!(final_state.retained_approval_count(), 2);
    for actor in [1, 3] {
        let member = final_state.record_by_key(key(actor).public_key()).unwrap();
        assert_eq!(member.status(), Status::Revoked);
        assert_eq!(member.next_sequence(), 2);
    }
}

#[test]
fn explicit_zero_entries_and_supply_survive_all_membership_changes() {
    for ledger in history() {
        assert!(ledger.contains_account(address(5)));
        assert_eq!(ledger.account(address(5)).balance(), 0);
        assert_eq!(ledger.account(address(5)).next_sequence(), 0);
        assert!(!ledger.contains_account(address(4)));
        assert_eq!(ledger.account(address(4)).balance(), 0);
        assert_eq!(
            ledger.account_count(),
            if ledger.height() == 0 { 3 } else { 4 }
        );
        assert_eq!(
            ledger
                .accounts()
                .map(|(_, account)| account.balance())
                .sum::<u64>(),
            1000
        );
        let addresses: Vec<_> = ledger
            .accounts()
            .map(|(address, _)| *address.as_bytes())
            .collect();
        assert!(addresses.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            ledger.account_bytes().unwrap().len(),
            4 + 48 * ledger.account_count()
        );
    }
}

#[test]
fn staged_roots_do_not_depend_on_later_external_block_hash_binding() {
    let expected = vectors();
    let config = config();
    let staged = bootstrap(&config);
    assert_eq!(
        staged.root().unwrap().to_hex(),
        expected["state0_ledger_root"]
    );
    let a = staged.clone().bind_to_validated_block(hash(0xc0));
    let b = staged.bind_to_validated_block(hash(0xc1));
    assert_ne!(a.tip_hash(), b.tip_hash());
    assert_eq!(a.root().unwrap(), b.root().unwrap());
    assert_reference(&a, "state0", &expected);
    assert_reference(&b, "state0", &expected);

    let operations = operations(&config);
    let group: Vec<_> = GROUPS[0].iter().map(|name| operations[name]).collect();
    let parent = bootstrap(&config).bind_to_validated_block(block_hash(0));
    let staged = parent
        .stage_operations(
            &View {
                context: config.context(),
                height: 0,
            },
            &group,
        )
        .unwrap();
    assert_eq!(
        staged.root().unwrap().to_hex(),
        expected["state1_ledger_root"]
    );
    let a = staged.clone().bind_to_validated_block(hash(0xc2));
    let b = staged.bind_to_validated_block(hash(0xc3));
    assert_ne!(a.tip_hash(), b.tip_hash());
    assert_reference(&a, "state1", &expected);
    assert_reference(&b, "state1", &expected);
}
