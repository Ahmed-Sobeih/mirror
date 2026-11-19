//! Complete synthetic V2 blocks accepted by two independent in-memory validators.
//! All keys, evidence labels and balances are public demonstration fixtures.

use mirror_chain::v2::{ChainErrorV2, ChainV2, NetworkManifestV2, genesis_template_v2};
use mirror_consensus::v2::{mine_header_v2, validate_header_pow_v2};
use mirror_core::{
    Address,
    v2::{BlockHeaderV2, BlockV2, SignedOperationV2, SignedTransferV2, TransferBodyV2},
};
use mirror_crypto::{Hash256, Keypair, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    MembershipStatusV2, OperationHeaderV2, SignedMembershipOperationV2, TestAllocation,
};

fn mine(config: &ConfigV2, proposal: &BlockV2) -> Result<BlockV2, Box<dyn std::error::Error>> {
    let result = mine_header_v2(*proposal.header(), config.parameters().pow_bits, 10_000)?;
    Ok(BlockV2::new(config, result.header, proposal.operations())?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let alice = Keypair::from_secret_bytes([1; 32]);
    let bob = Keypair::from_secret_bytes([2; 32]);
    let applicant = Keypair::from_secret_bytes([3; 32]);
    let address = |key: &Keypair| Address::from_public_key(&key.public_key());
    let evidence_profile = sha256d(b"Mirror chain demo: synthetic evidence profile");
    let mut allocations = [
        TestAllocation {
            address: *address(&alice).as_bytes(),
            balance_nusa: 1_000,
        },
        TestAllocation {
            address: *address(&bob).as_bytes(),
            balance_nusa: 0,
        },
    ];
    allocations.sort_by_key(|allocation| allocation.address);
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: sha256d(b"Mirror actual V2 block demonstration"),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile,
        },
        LimitsV2::default(),
        &[
            BootstrapDescriptor {
                public_key: alice.public_key(),
                nonce: sha256d(b"chain demo Alice seed nonce"),
                evidence: sha256d(b"chain demo Alice synthetic evidence"),
            },
            BootstrapDescriptor {
                public_key: bob.public_key(),
                nonce: sha256d(b"chain demo Bob seed nonce"),
                evidence: sha256d(b"chain demo Bob synthetic evidence"),
            },
        ],
        &allocations,
    )?;
    let proofs = [config.sign_seed(0, &alice)?, config.sign_seed(1, &bob)?];
    let bootstrap = config.verify_bootstrap(&proofs)?;
    let genesis = mine(&config, &genesis_template_v2(&bootstrap)?)?;
    let manifest = NetworkManifestV2::new(&config, &proofs, genesis.hash())?;

    // This fixture creator selects the network before serializing the manifest.
    // A receiver must obtain these pins independently of untrusted input bytes.
    let selected_identity = manifest.identity();
    let encoded_manifest = manifest.encode()?;
    let decoded_manifest = NetworkManifestV2::decode_for(&encoded_manifest, selected_identity)?;
    assert_eq!(decoded_manifest, manifest);

    // Each validator executes and validates genesis independently. They share no
    // mutable ledger and are not copies of an already accepted ChainV2 value.
    let mut validator_a = ChainV2::from_genesis(&decoded_manifest, genesis.clone())?;
    let mut validator_b = ChainV2::from_genesis(&decoded_manifest, genesis)?;
    assert_eq!(validator_a, validator_b);

    let operation_header = |key: &Keypair, sequence| OperationHeaderV2 {
        config_hash: config.context(),
        anchor_height: 0,
        anchor_hash: selected_identity.genesis_hash,
        valid_from: 1,
        valid_until: 8,
        actor_public_key: key.public_key(),
        actor_sequence: sequence,
    };
    let transfer = |key: &Keypair, recipient: Address, amount, sequence| {
        SignedTransferV2::sign(
            operation_header(key, sequence),
            TransferBodyV2 {
                recipient,
                amount,
                fee: 0,
            },
            key,
        )
        .map(SignedOperationV2::from)
    };
    let enrollment = SignedMembershipOperationV2::sign(
        operation_header(&applicant, 0),
        MembershipActionV2::Enroll {
            applicant_nonce: sha256d(b"chain demo applicant nonce"),
            evidence_commitment: sha256d(b"chain demo applicant synthetic evidence"),
            evidence_profile,
        },
        &applicant,
    )?;
    let enrollment_id = enrollment.enrollment_id().expect("Enroll has an ID");
    let approve = |key: &Keypair| {
        SignedMembershipOperationV2::sign(
            operation_header(key, 0),
            MembershipActionV2::Approve {
                target_enrollment_id: enrollment_id,
            },
            key,
        )
        .map(SignedOperationV2::from)
    };
    let operations = vec![
        transfer(&alice, address(&bob), 100, 0)?,
        transfer(&bob, address(&applicant), 40, 0)?,
        enrollment.into(),
        approve(&alice)?,
        approve(&bob)?,
    ];
    let proposal = validator_a.prepare_candidate(1, &operations)?;
    let block_one = mine(&config, &proposal)?;
    validator_a.append(block_one.clone())?;
    validator_b.append(block_one)?;
    assert_eq!(validator_a, validator_b);
    let member = validator_a
        .ledger()
        .membership()
        .record(enrollment_id)
        .expect("accepted enrollment");
    assert_eq!(member.status(), MembershipStatusV2::Active);
    assert_eq!(member.inclusion().operation_index, 2);
    assert_eq!(validator_a.ledger().account(address(&alice)).balance(), 900);
    assert_eq!(validator_a.ledger().account(address(&bob)).balance(), 60);
    assert_eq!(
        validator_a.ledger().account(address(&applicant)).balance(),
        40
    );
    assert_eq!(
        validator_a
            .ledger()
            .accounts()
            .map(|(_, account)| account.balance())
            .sum::<u64>(),
        1_000
    );

    let block_two = mine(&config, &validator_a.prepare_candidate(2, &[])?)?;
    validator_a.append(block_two.clone())?;
    validator_b.append(block_two)?;
    assert_eq!(validator_a, validator_b);

    // A claimed root is never accepted merely because its header has valid work.
    // Change exactly that root, then mine the resulting actual header again.
    let proposal = validator_a.prepare_candidate(3, &[])?;
    let header = proposal.header();
    let mut wrong_root = *header.ledger_root().as_bytes();
    wrong_root[0] ^= 1;
    let forged_header = BlockHeaderV2::new(
        header.config_hash(),
        header.height(),
        header.previous_hash(),
        header.operations_root(),
        Hash256::from_bytes(wrong_root),
        header.timestamp(),
        header.pow_bits(),
        0,
    );
    let forged_block = mine(
        &config,
        &BlockV2::new(&config, forged_header, proposal.operations())?,
    )?;
    validate_header_pow_v2(forged_block.header(), config.parameters().pow_bits)?;
    let before_a = validator_a.clone();
    let before_b = validator_b.clone();
    assert_eq!(
        validator_a.append(forged_block.clone()),
        Err(ChainErrorV2::WrongLedgerRoot)
    );
    assert_eq!(
        validator_b.append(forged_block),
        Err(ChainErrorV2::WrongLedgerRoot)
    );
    assert_eq!(validator_a, before_a);
    assert_eq!(validator_b, before_b);

    // Retain the expected checkpoint separately, before constructing replay
    // input. Parsing a log does not grant authority to choose its expected tip.
    let expected_checkpoint = validator_a.checkpoint();
    let encoded_blocks: Vec<_> = validator_a
        .blocks()
        .iter()
        .map(|block| block.encode(&config))
        .collect::<Result<_, _>>()?;
    let replay_input: Vec<_> = encoded_blocks.iter().map(Vec::as_slice).collect();
    let recovered = ChainV2::replay_encoded(&decoded_manifest, &replay_input, expected_checkpoint)?;
    assert_eq!(recovered, validator_a);
    assert_eq!(recovered, validator_b);

    println!("Mirror V2 actual-block acceptance — public synthetic fixtures");
    println!("Two independent in-memory validators; no network connection");
    println!("Manifest configuration: {}", selected_identity.config_hash);
    println!("Pinned mined genesis: {}", selected_identity.genesis_hash);
    println!("Manifest round-trip: exact independent pins and all seed proofs verified");
    println!("Block 1: two transfers, enrollment and two approvals accepted by both validators");
    println!("Original positions: 0 Transfer, 1 Transfer, 2 Enroll, 3 Approve, 4 Approve");
    println!("Applicant: Active after both existing reviewers approve");
    println!("Test Nusa: Alice 900, Bob 60, applicant 40; total 1000 unchanged");
    println!("Transfer and membership sequence spaces: separate");
    println!("Block 2: empty child accepted; complete validator states match");
    println!("Block 3 with wrong ledger root: valid work, rejected by both validators");
    println!("Rejection rollback: both complete chains unchanged");
    println!("Encoded complete-block replay: exact selected checkpoint and state reproduced");
    println!("Accepted height: {}", recovered.height());
    println!("Accepted tip: {}", recovered.tip_hash());
    println!("Accounts root: {}", recovered.ledger().accounts_root()?);
    println!(
        "Membership root: {}",
        recovered.ledger().membership().root()?
    );
    println!("Ledger root: {}", recovered.ledger().root()?);
    println!("No real biometrics, fees, rewards, durable publication or finality.");
    Ok(())
}
