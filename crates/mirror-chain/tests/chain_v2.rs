//! Synthetic actual-block authority tests. All keys/evidence are public fixtures.
use mirror_chain::v2::{
    ChainErrorV2 as Error, ChainReplayErrorV2 as ReplayError, ChainV2 as Chain,
    MAX_CHAIN_BLOCKS_V2, MAX_CHAIN_BYTES_V2, NetworkManifestV2, genesis_template_v2,
};
use mirror_consensus::v2::{PowErrorV2, mine_header_v2, validate_header_pow_v2};
use mirror_core::{
    Address,
    v2::{
        BlockErrorV2, BlockHeaderV2, BlockV2, SignedOperationV2 as Operation, SignedTransferV2,
        TransferBodyV2, TransferErrorV2, operations_root_v2,
    },
};
use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    MembershipStatusV2 as Status, OperationHeaderV2, ParentChainViewV2,
    SignedMembershipOperationV2, TestAllocation,
};
use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2, LedgerV2};

fn tag(number: u8) -> Hash256 {
    Hash256::from_bytes([number; 32])
}

fn key(actor: u8) -> Keypair {
    Keypair::from_secret_bytes([actor; 32])
}

fn address(actor: u8) -> Address {
    Address::from_public_key(&key(actor).public_key())
}

fn config(timestamp: u64) -> ConfigV2 {
    let seeds: Vec<_> = (1..=2)
        .map(|actor| BootstrapDescriptor {
            public_key: key(actor).public_key(),
            nonce: tag(10 + actor),
            evidence: tag(20 + actor),
        })
        .collect();
    let mut allocations = [
        TestAllocation {
            address: *address(1).as_bytes(),
            balance_nusa: 100,
        },
        TestAllocation {
            address: *address(2).as_bytes(),
            balance_nusa: 0,
        },
    ];
    allocations.sort_by_key(|allocation| allocation.address);
    ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: tag(30),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: timestamp,
            required_approvals: 2,
            evidence_profile: tag(31),
        },
        LimitsV2::default(),
        &seeds,
        &allocations,
    )
    .unwrap()
}

fn proofs(config: &ConfigV2) -> Vec<SignatureBytes> {
    (1..=2)
        .map(|actor| config.sign_seed(u32::from(actor - 1), &key(actor)).unwrap())
        .collect()
}

fn mined(config: &ConfigV2, block: &BlockV2) -> BlockV2 {
    let result = mine_header_v2(*block.header(), config.parameters().pow_bits, 10_000).unwrap();
    BlockV2::new(config, result.header, block.operations()).unwrap()
}

fn fresh_at(timestamp: u64) -> Chain {
    let config = config(timestamp);
    let proofs = proofs(&config);
    let bootstrap = config.verify_bootstrap(&proofs).unwrap();
    let genesis = mined(&config, &genesis_template_v2(&bootstrap).unwrap());
    let manifest = NetworkManifestV2::new(&config, &proofs, genesis.hash()).unwrap();
    Chain::from_genesis(&manifest, genesis).unwrap()
}

fn fresh() -> Chain {
    fresh_at(10)
}

fn reheader(config: &ConfigV2, block: &BlockV2, change: impl FnOnce(&mut [u8; 160])) -> BlockV2 {
    let mut bytes = block.header().encode();
    change(&mut bytes);
    BlockV2::new(
        config,
        BlockHeaderV2::decode(&bytes).unwrap(),
        block.operations(),
    )
    .unwrap()
}

fn header(chain: &Chain, actor: u8, sequence: u64, until: u64) -> OperationHeaderV2 {
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

fn transfer(chain: &Chain, actor: u8, recipient: u8, sequence: u64, amount: u64) -> Operation {
    Operation::Transfer(
        SignedTransferV2::sign(
            header(chain, actor, sequence, chain.height() + 8),
            TransferBodyV2 {
                recipient: address(recipient),
                amount,
                fee: 0,
            },
            &key(actor),
        )
        .unwrap(),
    )
}

fn enroll(chain: &Chain, actor: u8, until: u64) -> Operation {
    Operation::Membership(
        SignedMembershipOperationV2::sign(
            header(chain, actor, 0, until),
            Action::Enroll {
                applicant_nonce: tag(40 + actor),
                evidence_commitment: tag(50 + actor),
                evidence_profile: chain.config().parameters().evidence_profile,
            },
            &key(actor),
        )
        .unwrap(),
    )
}

fn next(chain: &Chain, timestamp: u64, operations: &[Operation]) -> BlockV2 {
    mined(
        chain.config(),
        &chain.prepare_candidate(timestamp, operations).unwrap(),
    )
}

fn arbitrary_child(chain: &Chain, operations: &[Operation]) -> BlockV2 {
    // Deliberately do not stage untrusted operations. The acceptance test must
    // discover their first execution failure before examining this claimed root.
    let header = BlockHeaderV2::new(
        chain.config().context(),
        chain.height() + 1,
        chain.tip_hash(),
        operations_root_v2(chain.config(), operations).unwrap(),
        tag(200),
        chain.blocks().last().unwrap().header().timestamp() + 1,
        chain.config().parameters().pow_bits,
        0,
    );
    mined(
        chain.config(),
        &BlockV2::new(chain.config(), header, operations).unwrap(),
    )
}

fn assert_rejected_unchanged(chain: &mut Chain, block: BlockV2, error: Error) {
    let before = chain.clone();
    assert_eq!(chain.append(block), Err(error));
    assert_eq!(*chain, before);
}

#[test]
fn genesis_is_executed_pinned_and_exposes_only_actual_ancestry() {
    let chain = fresh();
    let header = chain.genesis().header();
    assert_eq!(header.height(), 0);
    assert_eq!(header.previous_hash(), Hash256::default());
    assert_eq!(header.timestamp(), 10);
    assert!(chain.genesis().operations().is_empty());
    assert_eq!(header.ledger_root(), chain.ledger().root().unwrap());
    assert_eq!(chain.tip_hash(), chain.genesis().hash());
    assert_eq!(chain.manifest().genesis_hash(), chain.genesis().hash());
    assert_eq!(chain.ledger().account(address(1)).balance(), 100);
    assert_eq!(chain.ledger().account_count(), 2);
    assert_eq!(chain.ledger().membership().active_count(), 2);
    assert_eq!(chain.canonical_hash_at(0), Some(chain.tip_hash()));
    assert_eq!(chain.canonical_hash_at(1), None);
    assert_eq!(chain.block_at(u64::MAX), None);
    assert_eq!(chain.encoded_bytes(), chain.genesis().encoded_len());
    assert_eq!(chain.state_items_used(), 2 * (2 + 2));
}

#[test]
fn valid_alternate_genesis_nonce_is_rejected_by_exact_pin() {
    let chain = fresh();
    let alternate = reheader(chain.config(), chain.genesis(), |bytes| {
        bytes[152..].copy_from_slice(&(chain.genesis().header().nonce() + 1).to_le_bytes());
    });
    let alternate = mined(chain.config(), &alternate);
    assert_ne!(alternate.hash(), chain.genesis().hash());
    assert!(
        validate_header_pow_v2(alternate.header(), chain.config().parameters().pow_bits).is_ok()
    );
    assert_eq!(
        Chain::from_genesis(chain.manifest(), alternate),
        Err(Error::WrongGenesisHash)
    );
}

#[test]
fn pinned_genesis_still_requires_exact_fields_and_executed_roots() {
    let chain = fresh();
    let alterations: Vec<(BlockV2, Error)> = vec![
        (
            reheader(chain.config(), chain.genesis(), |b| {
                b[4..36].copy_from_slice(tag(98).as_bytes())
            }),
            Error::WrongConfiguration,
        ),
        (
            reheader(chain.config(), chain.genesis(), |b| {
                b[36..44].copy_from_slice(&1u64.to_le_bytes())
            }),
            Error::WrongHeight {
                expected: 0,
                got: 1,
            },
        ),
        (
            reheader(chain.config(), chain.genesis(), |b| {
                b[44..76].copy_from_slice(tag(98).as_bytes())
            }),
            Error::WrongParent,
        ),
        (
            reheader(chain.config(), chain.genesis(), |b| {
                b[140..148].copy_from_slice(&11u64.to_le_bytes())
            }),
            Error::WrongGenesisTimestamp {
                expected: 10,
                got: 11,
            },
        ),
        (
            reheader(chain.config(), chain.genesis(), |b| {
                b[76..108].copy_from_slice(tag(98).as_bytes())
            }),
            Error::WrongOperationsRoot,
        ),
        (
            reheader(chain.config(), chain.genesis(), |b| {
                b[108..140].copy_from_slice(tag(98).as_bytes())
            }),
            Error::WrongLedgerRoot,
        ),
    ];
    for (candidate, error) in alterations {
        let candidate = mined(chain.config(), &candidate);
        let manifest =
            NetworkManifestV2::new(chain.config(), &proofs(chain.config()), candidate.hash())
                .unwrap();
        assert_eq!(Chain::from_genesis(&manifest, candidate), Err(error));
    }
}

#[test]
fn genesis_cannot_execute_transactions() {
    let chain = fresh();
    let genesis = BlockV2::new(
        chain.config(),
        *chain.genesis().header(),
        &[transfer(&chain, 1, 2, 0, 1)],
    )
    .unwrap();
    assert_eq!(
        Chain::from_genesis(chain.manifest(), genesis),
        Err(Error::GenesisHasOperations)
    );
}

#[test]
fn genesis_template_has_exact_bootstrap_roots_but_is_not_a_chain() {
    let chain = fresh();
    let template = genesis_template_v2(chain.manifest().bootstrap()).unwrap();
    assert_eq!(template.header().nonce(), 0);
    assert_eq!(
        template.header().ledger_root(),
        LedgerV2::bootstrap(chain.manifest().bootstrap())
            .unwrap()
            .root()
            .unwrap()
    );
    assert_eq!(
        template.header().operations_root(),
        operations_root_v2(chain.config(), &[]).unwrap()
    );
    assert_eq!(template.operations(), &[]);
}

#[test]
fn proposal_is_read_only_and_child_binds_only_after_full_acceptance() {
    let mut chain = fresh();
    let before = chain.clone();
    let operations = [transfer(&chain, 1, 2, 0, 10), enroll(&chain, 3, 4)];
    let proposal = chain.prepare_candidate(11, &operations).unwrap();
    assert_eq!(proposal.header().nonce(), 0);
    assert_eq!(chain, before);
    let accepted = mined(chain.config(), &proposal);
    let hash = accepted.hash();
    chain.append(accepted).unwrap();
    assert_eq!(chain.height(), 1);
    assert_eq!(chain.tip_hash(), hash);
    assert_eq!(chain.ledger().tip_hash(), hash);
    assert_eq!(chain.ledger().account(address(1)).balance(), 90);
    assert_eq!(chain.ledger().account(address(1)).next_sequence(), 1);
    assert_eq!(chain.ledger().account(address(2)).balance(), 10);
    assert_eq!(
        chain
            .ledger()
            .membership()
            .record_by_key(key(3).public_key())
            .unwrap()
            .status(),
        Status::Pending
    );
    assert_eq!(
        chain.checkpoint().ledger_root,
        chain.ledger().root().unwrap()
    );
    // 8 genesis + 4 config + 2 accounts + 2 records + 0 approvals + 2 incoming.
    assert_eq!(chain.state_items_used(), 18);
}

#[test]
fn wrong_final_root_with_valid_work_rolls_back_full_mixed_transition() {
    let mut chain = fresh();
    let valid = next(
        &chain,
        11,
        &[transfer(&chain, 1, 2, 0, 10), enroll(&chain, 3, 4)],
    );
    let wrong = reheader(chain.config(), &valid, |b| {
        b[108..140].copy_from_slice(tag(99).as_bytes())
    });
    let wrong = mined(chain.config(), &wrong);
    assert_rejected_unchanged(&mut chain, wrong, Error::WrongLedgerRoot);
    chain.append(valid).unwrap();
    assert_eq!(chain.ledger().account(address(2)).balance(), 10);
    assert!(
        chain
            .ledger()
            .membership()
            .record_by_key(key(3).public_key())
            .is_some()
    );
}

#[test]
fn child_header_failures_are_atomic_and_timestamp_is_strict() {
    let mut chain = fresh();
    let valid = next(&chain, 11, &[]);
    let candidates = [
        (
            reheader(chain.config(), &valid, |b| {
                b[4..36].copy_from_slice(tag(99).as_bytes())
            }),
            Error::WrongConfiguration,
        ),
        (
            reheader(chain.config(), &valid, |b| {
                b[36..44].copy_from_slice(&2u64.to_le_bytes())
            }),
            Error::WrongHeight {
                expected: 1,
                got: 2,
            },
        ),
        (
            reheader(chain.config(), &valid, |b| {
                b[44..76].copy_from_slice(tag(99).as_bytes())
            }),
            Error::WrongParent,
        ),
        (
            reheader(chain.config(), &valid, |b| {
                b[140..148].copy_from_slice(&10u64.to_le_bytes())
            }),
            Error::InvalidTimestamp {
                parent: 10,
                got: 10,
            },
        ),
        (
            reheader(chain.config(), &valid, |b| {
                b[140..148].copy_from_slice(&9u64.to_le_bytes())
            }),
            Error::InvalidTimestamp { parent: 10, got: 9 },
        ),
    ];
    for (candidate, error) in candidates {
        assert_rejected_unchanged(&mut chain, candidate, error);
    }
    assert_eq!(
        chain.prepare_candidate(10, &[]),
        Err(Error::InvalidTimestamp {
            parent: 10,
            got: 10
        })
    );
}

#[test]
fn exact_target_and_actual_work_are_both_required() {
    let mut chain = fresh();
    let valid = next(&chain, 11, &[]);
    let wrong_bits = reheader(chain.config(), &valid, |b| {
        b[148..152].copy_from_slice(&0x207f_fffeu32.to_le_bytes())
    });
    assert_rejected_unchanged(
        &mut chain,
        wrong_bits,
        Error::Work(PowErrorV2::WrongBits {
            expected: 0x207f_ffff,
            got: 0x207f_fffe,
        }),
    );
    let invalid_work = (0u64..100)
        .find_map(|nonce| {
            let block = reheader(chain.config(), &valid, |b| {
                b[152..160].copy_from_slice(&nonce.to_le_bytes())
            });
            (validate_header_pow_v2(block.header(), chain.config().parameters().pow_bits)
                == Err(PowErrorV2::HashAboveTarget))
            .then_some(block)
        })
        .unwrap();
    assert_rejected_unchanged(
        &mut chain,
        invalid_work,
        Error::Work(PowErrorV2::HashAboveTarget),
    );
}

#[test]
fn valid_work_cannot_override_wrong_operation_commitment() {
    let mut chain = fresh();
    let valid = next(&chain, 11, &[transfer(&chain, 1, 2, 0, 10)]);
    let wrong = reheader(chain.config(), &valid, |b| {
        b[76..108].copy_from_slice(tag(99).as_bytes())
    });
    let wrong = mined(chain.config(), &wrong);
    assert_rejected_unchanged(&mut chain, wrong, Error::WrongOperationsRoot);
}

#[test]
fn earlier_execution_error_precedes_later_signature_error() {
    let mut chain = fresh();
    let invalid_funds = transfer(&chain, 2, 1, 0, 1);
    let Operation::Transfer(valid) = transfer(&chain, 1, 2, 0, 1) else {
        unreachable!()
    };
    let invalid_signature = Operation::Transfer(SignedTransferV2::from_parts(
        *valid.header(),
        *valid.body(),
        SignatureBytes::from_bytes([0; 64]),
    ));
    let block = arbitrary_child(&chain, &[invalid_funds, invalid_signature]);
    assert_rejected_unchanged(
        &mut chain,
        block,
        Error::Ledger(LedgerErrorV2::Operation {
            index: 0,
            error: LedgerOperationErrorV2::InsufficientBalance {
                available: 0,
                required: 1,
            },
        }),
    );
}

#[test]
fn late_operation_failure_discards_earlier_accounts_and_membership() {
    let mut chain = fresh();
    let operations = [
        transfer(&chain, 1, 2, 0, 5),
        enroll(&chain, 3, 5),
        transfer(&chain, 1, 2, 0, 1),
    ];
    let block = arbitrary_child(&chain, &operations);
    assert_rejected_unchanged(
        &mut chain,
        block,
        Error::Ledger(LedgerErrorV2::Operation {
            index: 2,
            error: LedgerOperationErrorV2::AccountSequenceMismatch {
                expected: 1,
                got: 0,
            },
        }),
    );
}

#[test]
fn actual_fork_ancestry_controls_operation_anchors() {
    let mut left = fresh();
    let mut right = left.clone();
    left.append(next(&left, 11, &[])).unwrap();
    right.append(next(&right, 12, &[])).unwrap();
    assert_ne!(left.tip_hash(), right.tip_hash());
    let anchored_left = transfer(&left, 1, 2, 0, 10);
    let wrong_branch_child = arbitrary_child(&right, &[anchored_left]);
    assert_rejected_unchanged(
        &mut right,
        wrong_branch_child,
        Error::Ledger(LedgerErrorV2::Operation {
            index: 0,
            error: LedgerOperationErrorV2::Transfer(TransferErrorV2::WrongAnchorHash),
        }),
    );
    left.append(next(&left, 13, &[anchored_left])).unwrap();
    assert_eq!(left.ledger().account(address(2)).balance(), 10);
    assert_eq!(right.ledger().account(address(2)).balance(), 0);
}

#[test]
fn empty_actual_blocks_process_expiry_and_preserve_accounts() {
    let mut chain = fresh();
    chain
        .append(next(&chain, 11, &[enroll(&chain, 3, 1)]))
        .unwrap();
    assert_eq!(
        chain
            .ledger()
            .membership()
            .record_by_key(key(3).public_key())
            .unwrap()
            .status(),
        Status::Pending
    );
    chain.append(next(&chain, 12, &[])).unwrap();
    assert_eq!(
        chain
            .ledger()
            .membership()
            .record_by_key(key(3).public_key())
            .unwrap()
            .status(),
        Status::Expired
    );
    assert_eq!(chain.ledger().account(address(1)).balance(), 100);
    assert_eq!(chain.ledger().account(address(1)).next_sequence(), 0);
}

#[test]
fn timestamp_has_no_wall_clock_dependency_or_wrapping_successor() {
    let mut chain = fresh();
    chain.append(next(&chain, u64::MAX, &[])).unwrap();
    let before = chain.clone();
    assert_eq!(
        chain.prepare_candidate(u64::MAX, &[]),
        Err(Error::InvalidTimestamp {
            parent: u64::MAX,
            got: u64::MAX
        })
    );
    assert_eq!(
        chain.prepare_candidate(0, &[]),
        Err(Error::InvalidTimestamp {
            parent: u64::MAX,
            got: 0
        })
    );
    assert_eq!(chain, before);
    assert_eq!(fresh_at(u64::MAX).height(), 0);
}

#[test]
fn typed_and_encoded_replay_reproduce_complete_branch_and_counters() {
    let mut chain = fresh();
    chain
        .append(next(
            &chain,
            11,
            &[transfer(&chain, 1, 2, 0, 5), enroll(&chain, 3, 2)],
        ))
        .unwrap();
    chain.append(next(&chain, 12, &[])).unwrap();
    chain.append(next(&chain, 13, &[])).unwrap();
    let expected = chain.checkpoint();
    assert_eq!(
        Chain::replay(chain.manifest(), chain.blocks(), expected).unwrap(),
        chain
    );
    let encoded: Vec<_> = chain
        .blocks()
        .iter()
        .map(|b| b.encode(chain.config()).unwrap())
        .collect();
    let slices: Vec<_> = encoded.iter().map(Vec::as_slice).collect();
    assert_eq!(
        Chain::replay_encoded(chain.manifest(), &slices, expected).unwrap(),
        chain
    );
}

#[test]
fn replay_requires_all_five_trusted_checkpoint_fields_and_complete_history() {
    let mut chain = fresh();
    chain.append(next(&chain, 11, &[])).unwrap();
    let expected = chain.checkpoint();
    let variants = [
        (
            mirror_chain::v2::ChainReplayExpectationV2 {
                config_hash: tag(90),
                ..expected
            },
            ReplayError::WrongExpectedConfiguration,
        ),
        (
            mirror_chain::v2::ChainReplayExpectationV2 {
                genesis_hash: tag(90),
                ..expected
            },
            ReplayError::WrongExpectedGenesis,
        ),
        (
            mirror_chain::v2::ChainReplayExpectationV2 {
                height: 2,
                ..expected
            },
            ReplayError::WrongExpectedHeight {
                expected: 2,
                got: 1,
            },
        ),
        (
            mirror_chain::v2::ChainReplayExpectationV2 {
                tip_hash: tag(90),
                ..expected
            },
            ReplayError::WrongExpectedTip,
        ),
        (
            mirror_chain::v2::ChainReplayExpectationV2 {
                ledger_root: tag(90),
                ..expected
            },
            ReplayError::WrongExpectedLedgerRoot,
        ),
    ];
    for (expected, error) in variants {
        assert_eq!(
            Chain::replay(chain.manifest(), chain.blocks(), expected),
            Err(error)
        );
    }
    assert_eq!(
        Chain::replay(chain.manifest(), &[], expected),
        Err(ReplayError::Input(Error::EmptyHistory))
    );
    assert_eq!(
        Chain::replay(chain.manifest(), &chain.blocks()[..1], expected),
        Err(ReplayError::WrongExpectedHeight {
            expected: 1,
            got: 0
        })
    );
}

#[test]
fn replay_rejects_invalid_intermediate_block_and_does_not_change_source() {
    let mut chain = fresh();
    chain.append(next(&chain, 11, &[])).unwrap();
    chain.append(next(&chain, 12, &[])).unwrap();
    let before = chain.clone();
    let mut blocks = chain.blocks().to_vec();
    blocks[1] = mined(
        chain.config(),
        &reheader(chain.config(), &blocks[1], |b| {
            b[108..140].copy_from_slice(tag(99).as_bytes())
        }),
    );
    assert_eq!(
        Chain::replay(chain.manifest(), &blocks, chain.checkpoint()),
        Err(ReplayError::Block {
            index: 1,
            error: Error::WrongLedgerRoot
        })
    );
    assert_eq!(chain, before);
}

#[test]
fn encoded_replay_checks_actual_format_not_snapshot_authority() {
    let chain = fresh();
    let bytes = chain.genesis().encode(chain.config()).unwrap();
    let mut wrong_magic = bytes.clone();
    wrong_magic[..4].copy_from_slice(b"MPS2");
    let mut wrong_version = bytes.clone();
    wrong_version[4..8].copy_from_slice(&3u32.to_le_bytes());
    let mut trailing = bytes.clone();
    trailing.push(0);
    for (data, error) in [
        (wrong_magic, BlockErrorV2::InvalidMagic),
        (wrong_version, BlockErrorV2::UnsupportedVersion(3)),
        (bytes[..bytes.len() - 1].to_vec(), BlockErrorV2::Truncated),
        (trailing, BlockErrorV2::TrailingBytes),
    ] {
        assert_eq!(
            Chain::replay_encoded(chain.manifest(), &[&data], chain.checkpoint()),
            Err(ReplayError::Block {
                index: 0,
                error: Error::Body(error)
            })
        );
    }
}

#[test]
fn replay_count_and_total_bytes_are_preflighted_before_decode() {
    let chain = fresh();
    let malformed = [0u8; 1];
    let too_many = vec![malformed.as_slice(); MAX_CHAIN_BLOCKS_V2 + 1];
    assert_eq!(
        Chain::replay_encoded(chain.manifest(), &too_many, chain.checkpoint()),
        Err(ReplayError::Input(Error::TooManyBlocks))
    );
    let exact_count = vec![malformed.as_slice(); MAX_CHAIN_BLOCKS_V2];
    let mut expected = chain.checkpoint();
    expected.height = (MAX_CHAIN_BLOCKS_V2 - 1) as u64;
    assert_eq!(
        Chain::replay_encoded(chain.manifest(), &exact_count, expected),
        Err(ReplayError::Block {
            index: 0,
            error: Error::Body(BlockErrorV2::Truncated)
        })
    );
    let megabyte = vec![0u8; 1_048_576];
    let exact_bytes = vec![megabyte.as_slice(); MAX_CHAIN_BYTES_V2 / megabyte.len()];
    expected.height = (exact_bytes.len() - 1) as u64;
    assert_eq!(
        Chain::replay_encoded(chain.manifest(), &exact_bytes, expected),
        Err(ReplayError::Block {
            index: 0,
            error: Error::Body(BlockErrorV2::InvalidMagic)
        })
    );
    let mut overflow = exact_bytes;
    overflow.push(&malformed);
    assert_eq!(
        Chain::replay_encoded(chain.manifest(), &overflow, expected),
        Err(ReplayError::Input(Error::TooManyBytes))
    );
}
