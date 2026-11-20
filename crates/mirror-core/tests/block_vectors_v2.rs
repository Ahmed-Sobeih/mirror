//! Independently calculated MB-02A bytes/signatures/roots. All keys are PUBLIC
//! synthetic fixtures. These values are not accepted ledger/genesis/PoW history.
use std::collections::BTreeMap;

use mirror_core::{
    Address, merkle_root,
    v2::{
        BlockHeaderV2, BlockV2, SignedOperationV2, SignedTransferV2, TransferBodyV2,
        operations_root_v2,
    },
};
use mirror_crypto::{Hash256, Keypair};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    OperationHeaderV2, SignedMembershipOperationV2,
};

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}

fn key(number: u8) -> Keypair {
    Keypair::from_secret_bytes([number; 32])
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

fn hash_from_hex(text: &str) -> Hash256 {
    Hash256::from_bytes(unhex(text).try_into().unwrap())
}

fn vectors() -> BTreeMap<&'static str, &'static str> {
    include_str!("vectors/blocks_v2.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split_once('=').unwrap())
        .collect()
}

fn config() -> ConfigV2 {
    ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: hash(0x11),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 1,
            evidence_profile: hash(0x22),
        },
        LimitsV2::default(),
        &[BootstrapDescriptor {
            public_key: key(1).public_key(),
            nonce: hash(3),
            evidence: hash(4),
        }],
        &[],
    )
    .unwrap()
}

fn operation_header(c: &ConfigV2, sequence: u64) -> OperationHeaderV2 {
    OperationHeaderV2 {
        config_hash: c.context(),
        anchor_height: 7,
        anchor_hash: hash(0x44),
        valid_from: 8,
        valid_until: 24,
        actor_public_key: key(1).public_key(),
        actor_sequence: sequence,
    }
}

fn transfer(c: &ConfigV2, maximum: bool) -> SignedTransferV2 {
    SignedTransferV2::sign(
        operation_header(c, if maximum { u64::MAX } else { 9 }),
        TransferBodyV2 {
            recipient: Address::from_public_key(&key(2).public_key()),
            amount: if maximum { u64::MAX } else { 123_456_789 },
            fee: 0,
        },
        &key(1),
    )
    .unwrap()
}

fn operations(c: &ConfigV2) -> BTreeMap<&'static str, SignedOperationV2> {
    let mut result = BTreeMap::from([
        ("transfer", SignedOperationV2::Transfer(transfer(c, false))),
        (
            "transfer_max",
            SignedOperationV2::Transfer(transfer(c, true)),
        ),
    ]);
    for (name, sequence, action) in [
        (
            "enroll",
            0,
            MembershipActionV2::Enroll {
                applicant_nonce: hash(0x55),
                evidence_commitment: hash(0x66),
                evidence_profile: hash(0x22),
            },
        ),
        (
            "approve",
            9,
            MembershipActionV2::Approve {
                target_enrollment_id: hash(0x77),
            },
        ),
        (
            "withdraw",
            9,
            MembershipActionV2::Withdraw {
                own_member_id: hash(0x88),
            },
        ),
    ] {
        result.insert(
            name,
            SignedOperationV2::Membership(
                SignedMembershipOperationV2::sign(operation_header(c, sequence), action, &key(1))
                    .unwrap(),
            ),
        );
    }
    result
}

fn header(c: &ConfigV2, root: Hash256) -> BlockHeaderV2 {
    BlockHeaderV2::new(
        c.context(),
        8,
        hash(0x44),
        root,
        hash(0x99),
        1_800_000_123,
        0x207f_ffff,
        0x0102_0304_0506_0708,
    )
}

fn assert_block_fixture(name: &str, names: &[&str]) {
    let c = config();
    let v = vectors();
    let available = operations(&c);
    let items: Vec<_> = names.iter().map(|name| available[name]).collect();
    let root = operations_root_v2(&c, &items).unwrap();
    assert_eq!(root.to_hex(), v[format!("{name}_root").as_str()], "{name}");
    let block = BlockV2::new(&c, header(&c, root), &items).unwrap();
    assert_eq!(
        block.operations().len().to_string(),
        v[format!("{name}_count").as_str()]
    );
    assert_eq!(
        hex(&block.header().encode()),
        v[format!("{name}_header").as_str()],
        "{name} header"
    );
    assert_eq!(
        block.hash().to_hex(),
        v[format!("{name}_hash").as_str()],
        "{name} header hash"
    );
    let expected = unhex(v[format!("{name}_block").as_str()]);
    assert_eq!(block.encode(&c).unwrap(), expected, "{name} complete bytes");
    let decoded = BlockV2::decode(&c, &expected).unwrap();
    assert_eq!(decoded, block, "{name} decoded value");
    assert_eq!(decoded.operations(), items.as_slice(), "original positions");
    assert_eq!(decoded.encode(&c).unwrap(), expected, "{name} re-encoding");
    // This checks body commitments/signatures only. No claim of executable
    // balances, member sequences, actual ancestry, genesis, work or ledger root.
    decoded.verify_integrity(&c).unwrap();
}

#[test]
fn configuration_and_address_match_independent_fixture() {
    let c = config();
    let v = vectors();
    assert_eq!(hex(&c.encode()), v["config"]);
    assert_eq!(c.context().to_hex(), v["context"]);
    assert_eq!(key(1).public_key().to_hex(), v["public_key_1"]);
    assert_eq!(key(2).public_key().to_hex(), v["public_key_2"]);
    assert_eq!(
        Address::from_public_key(&key(2).public_key()).to_hex(),
        v["recipient"]
    );
    // The new fixture keeps the existing B2 configuration exactly, including
    // its zero allocations; an encoding example cannot create spendable funds.
    let old: BTreeMap<_, _> =
        include_str!("../../mirror-personhood/tests/vectors/operations_v2.txt")
            .lines()
            .map(|line| line.split_once('=').unwrap())
            .collect();
    assert_eq!(v["config"], old["config"]);
    assert_eq!(v["context"], old["context"]);
    assert_eq!(c.initial_test_supply(), 0);
}

#[test]
fn transfer_preimage_signature_and_id_match_independent_python() {
    let c = config();
    let v = vectors();
    let op = transfer(&c, false);
    assert_eq!(op.encoded_len(), 243);
    assert_eq!(hex(&op.unsigned_bytes()), v["transfer_unsigned"]);
    assert_eq!(op.signing_hash().to_hex(), v["transfer_hash"]);
    assert_eq!(op.signature().to_hex(), v["transfer_signature"]);
    assert_eq!(hex(&op.encode()), v["transfer_signed"]);
    assert_eq!(op.id().to_hex(), v["transfer_id"]);
    op.verify_signature().unwrap();
    assert_eq!(
        SignedTransferV2::decode(&c, &unhex(v["transfer_signed"])).unwrap(),
        op
    );
    let unified = SignedOperationV2::decode(&c, &unhex(v["transfer_signed"])).unwrap();
    assert_eq!(unified, SignedOperationV2::Transfer(op));
    assert_eq!(unified.id().to_hex(), v["transfer_id"]);
    unified.verify_signature().unwrap();
}

#[test]
fn maximum_integer_transfer_matches_independent_bytes_without_funds_claim() {
    let c = config();
    let v = vectors();
    let op = transfer(&c, true);
    assert_eq!(op.header().actor_sequence, u64::MAX);
    assert_eq!(op.body().amount, u64::MAX);
    assert_eq!(hex(&op.unsigned_bytes()), v["transfer_max_unsigned"]);
    assert_eq!(op.signing_hash().to_hex(), v["transfer_max_hash"]);
    assert_eq!(op.signature().to_hex(), v["transfer_max_signature"]);
    assert_eq!(hex(&op.encode()), v["transfer_max_signed"]);
    assert_eq!(op.id().to_hex(), v["transfer_max_id"]);
    op.verify_signature().unwrap();
    assert_eq!(
        SignedTransferV2::decode(&c, &unhex(v["transfer_max_signed"])).unwrap(),
        op
    );
}

#[test]
fn existing_membership_bytes_are_preserved_through_the_unified_envelope() {
    let c = config();
    let v = vectors();
    let available = operations(&c);
    let old: BTreeMap<_, _> =
        include_str!("../../mirror-personhood/tests/vectors/operations_v2.txt")
            .lines()
            .map(|line| line.split_once('=').unwrap())
            .collect();
    for name in ["enroll", "approve", "withdraw"] {
        let op = available[name];
        for field in ["unsigned", "hash", "signature", "signed", "id"] {
            let fixture_name = format!("{name}_{field}");
            assert_eq!(v[fixture_name.as_str()], old[fixture_name.as_str()]);
        }
        let expected = unhex(v[format!("{name}_signed").as_str()]);
        assert_eq!(op.encode(), expected);
        assert_eq!(op.id().to_hex(), v[format!("{name}_id").as_str()]);
        assert_eq!(
            op.signing_hash().to_hex(),
            v[format!("{name}_hash").as_str()]
        );
        assert_eq!(SignedOperationV2::decode(&c, &expected).unwrap(), op);
        op.verify_signature().unwrap();
    }
}

#[test]
fn empty_block_matches_independent_bytes_root_and_hash() {
    assert_block_fixture("empty", &[]);
    assert_eq!(unhex(vectors()["empty_block"]).len(), 168);
}

#[test]
fn single_transfer_block_matches_independent_bytes_root_and_hash() {
    assert_block_fixture("single", &["transfer"]);
    assert_eq!(unhex(vectors()["single_block"]).len(), 168 + 4 + 243);
}

#[test]
fn odd_trees_match_independent_three_and_five_operation_vectors() {
    assert_block_fixture("odd", &["transfer", "enroll", "approve"]);
    assert_block_fixture(
        "odd_five",
        &["transfer", "enroll", "approve", "withdraw", "transfer_max"],
    );
}

#[test]
fn mixed_all_kinds_and_reordered_blocks_match_independent_vectors() {
    assert_block_fixture("mixed", &["transfer", "enroll", "approve", "withdraw"]);
    assert_block_fixture("reordered", &["withdraw", "approve", "enroll", "transfer"]);
    let v = vectors();
    assert_ne!(v["mixed_root"], v["reordered_root"]);
    assert_ne!(v["mixed_hash"], v["reordered_hash"]);
}

#[test]
fn count_commitment_distinguishes_the_duplicate_last_merkle_ambiguity() {
    let v = vectors();
    let c = config();
    let available = operations(&c);
    let three = [
        available["transfer"],
        available["enroll"],
        available["approve"],
    ];
    let four = [three[0], three[1], three[2], three[2]];
    let leaves = [
        hash_from_hex(v["transfer_leaf"]),
        hash_from_hex(v["enroll_leaf"]),
        hash_from_hex(v["approve_leaf"]),
    ];
    let three_merkle = merkle_root(&leaves);
    let four_merkle = merkle_root(&[leaves[0], leaves[1], leaves[2], leaves[2]]);
    assert_eq!(three_merkle, four_merkle);
    assert_eq!(three_merkle.to_hex(), v["odd_merkle"]);
    assert_eq!(four_merkle.to_hex(), v["duplicate_last_merkle"]);
    let three_root = operations_root_v2(&c, &three).unwrap();
    let four_root = operations_root_v2(&c, &four).unwrap();
    assert_eq!(three_root.to_hex(), v["odd_root"]);
    assert_eq!(four_root.to_hex(), v["duplicate_last_root"]);
    assert_ne!(three_root, four_root);
    assert_block_fixture(
        "duplicate_last",
        &["transfer", "enroll", "approve", "approve"],
    );
}

#[test]
fn header_field_variants_match_independent_bytes_and_domain_hashes() {
    let c = config();
    let v = vectors();
    let root = hash_from_hex(v["mixed_root"]);
    let base = header(&c, root);
    let make = |context, height, previous, ops_root, ledger_root, timestamp, bits, nonce| {
        BlockHeaderV2::new(
            context,
            height,
            previous,
            ops_root,
            ledger_root,
            timestamp,
            bits,
            nonce,
        )
    };
    let variants = [
        (
            "context",
            make(
                hash(0x12),
                8,
                hash(0x44),
                root,
                hash(0x99),
                1_800_000_123,
                0x207f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "height",
            make(
                c.context(),
                9,
                hash(0x44),
                root,
                hash(0x99),
                1_800_000_123,
                0x207f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "previous",
            make(
                c.context(),
                8,
                hash(0x45),
                root,
                hash(0x99),
                1_800_000_123,
                0x207f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "operations_root",
            make(
                c.context(),
                8,
                hash(0x44),
                hash(0x46),
                hash(0x99),
                1_800_000_123,
                0x207f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "ledger_root",
            make(
                c.context(),
                8,
                hash(0x44),
                root,
                hash(0x9a),
                1_800_000_123,
                0x207f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "timestamp",
            make(
                c.context(),
                8,
                hash(0x44),
                root,
                hash(0x99),
                1_800_000_124,
                0x207f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "pow_bits",
            make(
                c.context(),
                8,
                hash(0x44),
                root,
                hash(0x99),
                1_800_000_123,
                0x1f0f_ffff,
                0x0102_0304_0506_0708,
            ),
        ),
        (
            "nonce",
            make(
                c.context(),
                8,
                hash(0x44),
                root,
                hash(0x99),
                1_800_000_123,
                0x207f_ffff,
                0x0102_0304_0506_0709,
            ),
        ),
        (
            "max_integers",
            make(
                c.context(),
                u64::MAX,
                hash(0x44),
                root,
                hash(0x99),
                u64::MAX,
                u32::MAX,
                u64::MAX,
            ),
        ),
    ];
    assert_eq!(base.version(), 2);
    for (name, changed) in variants {
        let expected = unhex(v[format!("header_{name}").as_str()]);
        assert_eq!(changed.encode().as_slice(), expected, "{name}");
        assert_eq!(changed.hash().to_hex(), v[format!("hash_{name}").as_str()]);
        assert_ne!(changed.hash(), base.hash());
        assert_eq!(BlockHeaderV2::decode(&expected).unwrap(), changed);
    }
}
