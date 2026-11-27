//! Public synthetic keys and accepted chains for local pending-pool evidence.
#![allow(dead_code)]

use mirror_chain::v2::{ChainV2, NetworkManifestV2, genesis_template_v2};
use mirror_consensus::v2::mine_header_v2;
use mirror_core::{
    Address,
    v2::{BlockV2, SignedOperationV2, SignedTransferV2, TransferBodyV2},
};
use mirror_crypto::{Hash256, Keypair};
use mirror_node::v2::pool::PendingPoolLimitsV2;
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    OperationHeaderV2, SignedMembershipOperationV2, TestAllocation,
};

pub type Operation = SignedOperationV2;

pub fn tag(number: u8) -> Hash256 {
    Hash256::from_bytes([number; 32])
}

pub fn key(actor: u8) -> Keypair {
    Keypair::from_secret_bytes([actor; 32])
}

pub fn address(actor: u8) -> Address {
    Address::from_public_key(&key(actor).public_key())
}

pub fn pool_limits() -> PendingPoolLimitsV2 {
    PendingPoolLimitsV2 {
        max_entries: 256,
        max_encoded_bytes: 74_496,
        max_batch_operations: 64,
        max_batch_bytes: 19_048,
        max_work_units: 67_108_864,
    }
}

pub fn chain_with(
    limits: LimitsV2,
    allocations: &[(u8, u64)],
    required_approvals: u16,
    network: u8,
) -> ChainV2 {
    let seeds: Vec<_> = (1..=2)
        .map(|actor| BootstrapDescriptor {
            public_key: key(actor).public_key(),
            nonce: tag(10 + actor),
            evidence: tag(20 + actor),
        })
        .collect();
    let mut allocations: Vec<_> = allocations
        .iter()
        .map(|&(actor, balance)| TestAllocation {
            address: *address(actor).as_bytes(),
            balance_nusa: balance,
        })
        .collect();
    allocations.sort_by_key(|allocation| allocation.address);
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: tag(network),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 10,
            required_approvals,
            evidence_profile: tag(31),
        },
        limits,
        &seeds,
        &allocations,
    )
    .unwrap();
    let proofs: Vec<_> = (1..=2)
        .map(|actor| config.sign_seed(u32::from(actor - 1), &key(actor)).unwrap())
        .collect();
    let bootstrap = config.verify_bootstrap(&proofs).unwrap();
    let template = genesis_template_v2(&bootstrap).unwrap();
    let genesis = mined(&config, &template);
    let manifest = NetworkManifestV2::new(&config, &proofs, genesis.hash()).unwrap();
    ChainV2::from_genesis(&manifest, genesis).unwrap()
}

pub fn fresh() -> ChainV2 {
    chain_with(LimitsV2::default(), &[(1, 100), (2, 0)], 1, 30)
}

pub fn header(chain: &ChainV2, actor: u8, sequence: u64, until: u64) -> OperationHeaderV2 {
    OperationHeaderV2 {
        config_hash: chain.config().context(),
        anchor_height: chain.height(),
        anchor_hash: chain.tip_hash(),
        valid_from: chain.height() + 1,
        valid_until: until,
        actor_public_key: key(actor).public_key(),
        actor_sequence: sequence,
    }
}

pub fn transfer_header(
    actor: u8,
    header: OperationHeaderV2,
    recipient: u8,
    amount: u64,
    fee: u64,
) -> Operation {
    SignedTransferV2::sign(
        header,
        TransferBodyV2 {
            recipient: address(recipient),
            amount,
            fee,
        },
        &key(actor),
    )
    .unwrap()
    .into()
}

pub fn transfer(
    chain: &ChainV2,
    actor: u8,
    recipient: u8,
    sequence: u64,
    amount: u64,
) -> Operation {
    transfer_header(
        actor,
        header(chain, actor, sequence, chain.height() + 8),
        recipient,
        amount,
        0,
    )
}

pub fn enroll(chain: &ChainV2, actor: u8, until: u64) -> Operation {
    SignedMembershipOperationV2::sign(
        header(chain, actor, 0, until),
        MembershipActionV2::Enroll {
            applicant_nonce: tag(actor.wrapping_add(40)),
            evidence_commitment: tag(actor.wrapping_add(50)),
            evidence_profile: chain.config().parameters().evidence_profile,
        },
        &key(actor),
    )
    .unwrap()
    .into()
}

pub fn approve(
    chain: &ChainV2,
    actor: u8,
    sequence: u64,
    target: Hash256,
    until: u64,
) -> Operation {
    SignedMembershipOperationV2::sign(
        header(chain, actor, sequence, until),
        MembershipActionV2::Approve {
            target_enrollment_id: target,
        },
        &key(actor),
    )
    .unwrap()
    .into()
}

pub fn withdraw(chain: &ChainV2, actor: u8, sequence: u64) -> Operation {
    let own_member_id = chain
        .ledger()
        .membership()
        .record_by_key(key(actor).public_key())
        .unwrap()
        .id();
    SignedMembershipOperationV2::sign(
        header(chain, actor, sequence, chain.height() + 8),
        MembershipActionV2::Withdraw { own_member_id },
        &key(actor),
    )
    .unwrap()
    .into()
}

fn mined(config: &ConfigV2, block: &BlockV2) -> BlockV2 {
    let result = mine_header_v2(*block.header(), config.parameters().pow_bits, 10_000).unwrap();
    BlockV2::new(config, result.header, block.operations()).unwrap()
}

pub fn next(chain: &ChainV2, timestamp: u64, operations: &[Operation]) -> BlockV2 {
    mined(
        chain.config(),
        &chain.prepare_candidate(timestamp, operations).unwrap(),
    )
}

pub fn append(chain: &mut ChainV2, operations: &[Operation]) {
    let timestamp = chain.blocks().last().unwrap().header().timestamp() + 1;
    let block = next(chain, timestamp, operations);
    chain.append(block).unwrap();
}
