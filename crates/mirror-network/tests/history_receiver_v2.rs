//! The peer supplies claims and bytes; only actual execution yields a candidate.
//! All keys, metadata and operations here are public synthetic development data.

mod support_history_v2;

use mirror_chain::v2::{
    ChainErrorV2, ChainV2, ForkChoiceV2, ManifestErrorV2, ManifestIdentityV2, NetworkManifestV2,
    compare_branches_v2, genesis_template_v2,
};
use mirror_consensus::v2::{PowErrorV2, mine_header_v2, validate_header_pow_v2};
use mirror_core::{
    Address,
    v2::{BlockHeaderV2, BlockV2, SignedOperationV2, SignedTransferV2, operations_root_v2},
};
use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_network::v2::{
    HistoryErrorV2 as Error, HistoryFramesV2, HistoryReceiverV2 as Receiver, ReceivedCandidateV2,
};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, TestAllocation,
};
use mirror_protocol::v2::{
    FRAME_HEADER_LEN_V2, FrameHeaderV2, FrameKindV2 as Kind, FrameRefV2 as Frame, HistorySummaryV2,
    decode_frame_v2, encode_frame_v2,
};
use support_history_v2::{
    block, fixture_bytes, fixture_hash, fork_chain, fork_prefix, independent_identity, main_chain,
};

fn frames(chain: &ChainV2) -> Vec<Vec<u8>> {
    HistoryFramesV2::new(chain)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn summary(frames: &[Vec<u8>]) -> HistorySummaryV2 {
    match decode_frame_v2(&frames[0]).unwrap() {
        Frame::Begin(summary) => summary,
        _ => panic!("source must begin with its captured summary"),
    }
}

fn with_begin(frames: &mut [Vec<u8>], change: impl FnOnce(&mut HistorySummaryV2)) {
    let mut claim = summary(frames);
    change(&mut claim);
    frames[0] = encode_frame_v2(Frame::Begin(claim)).unwrap();
}

fn feed(identity: ManifestIdentityV2, frames: &[Vec<u8>]) -> Result<Receiver, Error> {
    frames
        .iter()
        .try_fold(Receiver::new(identity), |receiver, frame| {
            receiver.accept_frame(frame)
        })
}

fn receive(frames: &[Vec<u8>]) -> Result<ReceivedCandidateV2, Error> {
    feed(independent_identity(), frames)?.finish()
}

fn replace_block(frames: &mut [Vec<u8>], height: u64, bytes: &[u8]) {
    frames[height as usize + 2] = encode_frame_v2(Frame::Block { height, bytes }).unwrap();
}

fn reheader(chain: &ChainV2, block: &BlockV2, change: impl FnOnce(&mut [u8; 160])) -> BlockV2 {
    let mut header = block.header().encode();
    change(&mut header);
    BlockV2::new(
        chain.config(),
        BlockHeaderV2::decode(&header).unwrap(),
        block.operations(),
    )
    .unwrap()
}

fn mined(chain: &ChainV2, block: &BlockV2) -> BlockV2 {
    let work = mine_header_v2(
        *block.header(),
        chain.config().parameters().pow_bits,
        10_000,
    )
    .unwrap();
    BlockV2::new(chain.config(), work.header, block.operations()).unwrap()
}

fn rebody(chain: &ChainV2, block: &BlockV2, operations: &[SignedOperationV2]) -> BlockV2 {
    let mut header = block.header().encode();
    header[76..108].copy_from_slice(
        operations_root_v2(chain.config(), operations)
            .unwrap()
            .as_bytes(),
    );
    header[152..160].copy_from_slice(&0u64.to_le_bytes());
    let altered = BlockV2::new(
        chain.config(),
        BlockHeaderV2::decode(&header).unwrap(),
        operations,
    )
    .unwrap();
    mined(chain, &altered)
}

fn assert_chain_rejection(frames: &[Vec<u8>], expected: ChainErrorV2) {
    match receive(frames).unwrap_err() {
        Error::Chain(actual) => assert_eq!(actual, expected),
        other => panic!("expected actual chain validation {expected:?}, got {other:?}"),
    }
}

fn small_block_chain() -> ChainV2 {
    let key = Keypair::from_secret_bytes([77; 32]);
    let seed = BootstrapDescriptor {
        public_key: key.public_key(),
        nonce: Hash256::from_bytes([71; 32]),
        evidence: Hash256::from_bytes([72; 32]),
    };
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: Hash256::from_bytes([73; 32]),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 10,
            required_approvals: 1,
            evidence_profile: Hash256::from_bytes([74; 32]),
        },
        LimitsV2 {
            max_block_bytes: 512,
            max_operation_bytes: 291,
            ..LimitsV2::default()
        },
        &[seed],
        &[TestAllocation {
            address: *Address::from_public_key(&key.public_key()).as_bytes(),
            balance_nusa: 0,
        }],
    )
    .unwrap();
    let proofs = [config.sign_seed(0, &key).unwrap()];
    let bootstrap = config.verify_bootstrap(&proofs).unwrap();
    let proposal = genesis_template_v2(&bootstrap).unwrap();
    let work = mine_header_v2(*proposal.header(), config.parameters().pow_bits, 10_000).unwrap();
    let genesis = BlockV2::new(&config, work.header, &[]).unwrap();
    let manifest = NetworkManifestV2::new(&config, &proofs, genesis.hash()).unwrap();
    let mut chain = ChainV2::from_genesis(&manifest, genesis).unwrap();
    for timestamp in [11, 12] {
        let proposal = chain.prepare_candidate(timestamp, &[]).unwrap();
        chain.append(mined(&chain, &proposal)).unwrap();
    }
    chain
}

#[test]
fn complete_history_reproduces_full_state_and_exact_wire_accounting() {
    let expected = main_chain(4);
    let input = frames(&expected);
    let received = receive(&input).unwrap();
    assert_eq!(received.chain(), &expected);
    assert_eq!(received.chain().state_items_used(), 75);
    assert_eq!(received.chain().encoded_bytes(), 4330);
    assert_eq!(received.frame_count(), 8);
    assert_eq!(received.wire_bytes(), 5406);
    assert_eq!(received.wire_bytes(), input.iter().map(Vec::len).sum());
    assert_eq!(received.summary(), summary(&input));
    assert_eq!(received.into_chain(), expected);
}

#[test]
fn identical_accounts_do_not_collapse_different_membership_histories() {
    let main = main_chain(3);
    let fork = fork_chain();
    assert_eq!(
        main.ledger().account_bytes().unwrap(),
        fork.ledger().account_bytes().unwrap()
    );
    let received_main = receive(&frames(&main)).unwrap().into_chain();
    let received_fork = receive(&frames(&fork)).unwrap().into_chain();
    assert_eq!(received_main, main);
    assert_eq!(received_fork, fork);
    assert_ne!(
        received_main.ledger().membership().root().unwrap(),
        received_fork.ledger().membership().root().unwrap()
    );
    assert_ne!(
        received_main
            .ledger()
            .membership()
            .record(fixture_hash("c_member_id"))
            .unwrap()
            .status(),
        received_fork
            .ledger()
            .membership()
            .record(fixture_hash("c_member_id"))
            .unwrap()
            .status()
    );
    assert_ne!(
        received_main.ledger().membership().active_high_water(),
        received_fork.ledger().membership().active_high_water()
    );
    let comparison =
        compare_branches_v2(independent_identity(), &received_main, &received_fork).unwrap();
    assert_eq!(comparison.choice(), ForkChoiceV2::KeepCurrent);
    assert_eq!(comparison.common_ancestor().height, 1);
}

#[test]
fn a_complete_genesis_only_candidate_is_not_a_latestness_claim() {
    let genesis = main_chain(0);
    let received = receive(&frames(&genesis)).unwrap();
    assert_eq!(received.chain(), &genesis);
    assert_eq!(received.frame_count(), 4);
    assert_eq!(received.wire_bytes(), 1164);
    let current = main_chain(4);
    assert_eq!(
        compare_branches_v2(independent_identity(), &current, received.chain())
            .unwrap()
            .choice(),
        ForkChoiceV2::KeepCurrent
    );
}

#[test]
fn split_header_payload_path_rebuilds_the_same_complete_candidate() {
    let expected = main_chain(4);
    let mut receiver = Receiver::new(independent_identity());
    for frame in frames(&expected) {
        let pending = receiver.begin_frame(&frame[..FRAME_HEADER_LEN_V2]).unwrap();
        assert_eq!(pending.payload_len(), frame.len() - FRAME_HEADER_LEN_V2);
        receiver = pending
            .accept_payload(&frame[FRAME_HEADER_LEN_V2..])
            .unwrap();
    }
    assert_eq!(receiver.finish().unwrap().chain(), &expected);
}

#[test]
fn independent_network_pins_reject_before_manifest_processing() {
    let input = frames(&main_chain(4));
    let identity = independent_identity();
    for wrong in [
        ManifestIdentityV2 {
            config_hash: Hash256::default(),
            ..identity
        },
        ManifestIdentityV2 {
            genesis_hash: Hash256::default(),
            ..identity
        },
    ] {
        assert!(matches!(
            Receiver::new(wrong).accept_frame(&input[0]),
            Err(Error::WrongNetwork)
        ));
    }
}

#[test]
fn manifest_must_match_independent_genesis_even_when_begin_matches() {
    let mut input = frames(&main_chain(4));
    let mut bytes = fixture_bytes("manifest_hex");
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    input[1] = encode_frame_v2(Frame::Manifest(&bytes)).unwrap();
    assert!(matches!(
        receive(&input),
        Err(Error::Manifest(ManifestErrorV2::WrongGenesis))
    ));
}

#[test]
fn all_manifest_seed_proofs_are_checked() {
    let mut input = frames(&main_chain(4));
    let mut bytes = fixture_bytes("manifest_hex");
    let config_length = u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
    let second_signature = 10 + config_length + 4 + 64;
    bytes[second_signature] ^= 1;
    input[1] = encode_frame_v2(Frame::Manifest(&bytes)).unwrap();
    assert!(matches!(
        receive(&input),
        Err(Error::Manifest(ManifestErrorV2::Config(_)))
    ));
}

#[test]
fn unknown_magic_version_and_kind_reject_in_header_phase() {
    let begin = frames(&main_chain(0)).remove(0);
    for (offset, changed) in [(0, 0), (4, 3), (6, 99)] {
        let mut header = begin[..FRAME_HEADER_LEN_V2].to_vec();
        header[offset] = changed;
        assert!(matches!(
            Receiver::new(independent_identity()).begin_frame(&header),
            Err(Error::Protocol(_))
        ));
    }
}

#[test]
fn truncated_or_trailing_complete_frame_never_yields_a_candidate() {
    let input = frames(&main_chain(1));
    for index in 0..input.len() {
        for length in [
            0,
            FRAME_HEADER_LEN_V2 - 1,
            FRAME_HEADER_LEN_V2,
            input[index].len() - 1,
        ] {
            let receiver = feed(independent_identity(), &input[..index]).unwrap();
            assert!(
                receiver.accept_frame(&input[index][..length]).is_err(),
                "frame {index}, length {length}"
            );
        }
        let receiver = feed(independent_identity(), &input[..index]).unwrap();
        let mut extra = input[index].clone();
        extra.push(0);
        assert!(
            receiver.accept_frame(&extra).is_err(),
            "frame {index} with trailing byte"
        );
    }
}

#[test]
fn pending_payload_requires_exact_declared_length() {
    let input = frames(&main_chain(0));
    let payload = &input[0][FRAME_HEADER_LEN_V2..];
    for bytes in [&payload[..payload.len() - 1], &[0u8; 153][..]] {
        let pending = Receiver::new(independent_identity())
            .begin_frame(&input[0][..FRAME_HEADER_LEN_V2])
            .unwrap();
        assert!(matches!(
            pending.accept_payload(bytes),
            Err(Error::Protocol(_))
        ));
    }
}

#[test]
fn every_unfinished_phase_refuses_finish() {
    let input = frames(&main_chain(4));
    for count in 0..input.len() {
        let receiver = feed(independent_identity(), &input[..count]).unwrap();
        assert!(
            matches!(receiver.finish(), Err(Error::IncompleteHistory)),
            "accepted prefix count {count}"
        );
    }
}

#[test]
fn phase_order_rejects_missing_repeated_or_restarted_metadata() {
    let input = frames(&main_chain(1));
    for (prefix, next) in [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)] {
        let receiver = feed(independent_identity(), &input[..prefix]).unwrap();
        assert!(
            matches!(
                receiver.begin_frame(&input[next][..FRAME_HEADER_LEN_V2]),
                Err(Error::UnexpectedFrame { .. })
            ),
            "prefix {prefix}, next {next}"
        );
    }
}

#[test]
fn genesis_payload_size_is_rejected_before_payload_is_requested() {
    let input = frames(&main_chain(1));
    let receiver = feed(independent_identity(), &input[..2]).unwrap();
    let header = FrameHeaderV2::new(Kind::Block, 177).unwrap().encode();
    assert!(matches!(
        receiver.begin_frame(&header),
        Err(Error::WrongPayloadLength {
            expected: 176,
            got: 177
        })
    ));
}

#[test]
fn duplicate_and_reordered_blocks_are_rejected() {
    let source = main_chain(3);
    for incorrect_height in [0, 2] {
        let input = frames(&source);
        let receiver = feed(independent_identity(), &input[..3]).unwrap();
        let wrong = source
            .block_at(incorrect_height)
            .unwrap()
            .encode(source.config())
            .unwrap();
        let frame = encode_frame_v2(Frame::Block {
            height: incorrect_height,
            bytes: &wrong,
        })
        .unwrap();
        assert!(
            matches!(receiver.accept_frame(&frame), Err(Error::WrongBlockHeight { expected: 1, got }) if got == incorrect_height)
        );
    }
}

#[test]
fn wire_height_and_actual_block_height_must_agree() {
    let chain = main_chain(1);
    let mut input = frames(&chain);
    let wrong = reheader(&chain, &block("block1"), |bytes| {
        bytes[36..44].copy_from_slice(&2u64.to_le_bytes())
    });
    replace_block(&mut input, 1, &wrong.encode(chain.config()).unwrap());
    assert!(matches!(
        receive(&input),
        Err(Error::WrongBlockHeight {
            expected: 1,
            got: 2
        })
    ));
}

#[test]
fn terminal_frame_is_required_only_after_all_promised_blocks() {
    let input = frames(&main_chain(4));
    for count in [1, 2, 3, input.len() - 2] {
        let receiver = feed(independent_identity(), &input[..count]).unwrap();
        assert!(matches!(
            receiver.begin_frame(&input.last().unwrap()[..FRAME_HEADER_LEN_V2]),
            Err(Error::UnexpectedFrame { .. })
        ));
    }
    let receiver = feed(independent_identity(), &input[..input.len() - 1]).unwrap();
    assert!(matches!(receiver.finish(), Err(Error::IncompleteHistory)));
}

#[test]
fn terminal_summary_cannot_change_the_captured_offer() {
    let input = frames(&main_chain(4));
    let captured = summary(&input);
    let mut changed = Vec::new();
    let mut value = captured;
    value.config_hash = Hash256::default();
    changed.push(value);
    let mut value = captured;
    value.genesis_hash = Hash256::default();
    changed.push(value);
    let mut value = captured;
    value.tip_hash = Hash256::default();
    changed.push(value);
    let mut value = captured;
    value.ledger_root = Hash256::default();
    changed.push(value);
    let mut value = captured;
    value.height += 1;
    value.block_count += 1;
    changed.push(value);
    let mut value = captured;
    value.block_bytes += 1;
    changed.push(value);
    let mut value = captured;
    value.manifest_bytes += 1;
    changed.push(value);
    for claim in changed {
        let receiver = feed(independent_identity(), &input[..input.len() - 1]).unwrap();
        let end = encode_frame_v2(Frame::End(claim)).unwrap();
        assert!(matches!(
            receiver.accept_frame(&end),
            Err(Error::ChangedSummary)
        ));
    }
}

#[test]
fn extra_blocks_and_frames_after_end_are_rejected() {
    let input = frames(&main_chain(1));
    let before_end = feed(independent_identity(), &input[..input.len() - 1]).unwrap();
    assert!(matches!(
        before_end.begin_frame(&input[2][..FRAME_HEADER_LEN_V2]),
        Err(Error::UnexpectedFrame {
            expected: Kind::End,
            got: Kind::Block
        })
    ));
    let completed = feed(independent_identity(), &input).unwrap();
    assert!(matches!(
        completed.accept_frame(&input[0]),
        Err(Error::AlreadyComplete)
    ));
}

#[test]
fn valid_shorter_prefix_does_not_satisfy_a_longer_promised_history() {
    let mut input = frames(&main_chain(4));
    input.remove(input.len() - 2);
    assert!(matches!(
        receive(&input),
        Err(Error::UnexpectedFrame {
            expected: Kind::Block,
            got: Kind::End
        })
    ));
}

#[test]
fn false_advertised_tip_or_root_is_rejected_after_actual_execution() {
    for root in [false, true] {
        let mut input = frames(&main_chain(4));
        with_begin(&mut input, |claim| {
            if root {
                claim.ledger_root = Hash256::default();
            } else {
                claim.tip_hash = Hash256::default();
            }
        });
        assert!(matches!(receive(&input), Err(Error::EndpointMismatch)));
    }
}

#[test]
fn manifest_frame_length_must_match_offer_before_payload_allocation() {
    let mut input = frames(&main_chain(4));
    with_begin(&mut input, |claim| claim.manifest_bytes += 1);
    let receiver = feed(independent_identity(), &input[..1]).unwrap();
    assert!(matches!(
        receiver.begin_frame(&input[1][..FRAME_HEADER_LEN_V2]),
        Err(Error::WrongPayloadLength {
            expected: 637,
            got: 636
        })
    ));
}

#[test]
fn configured_block_ceiling_is_checked_before_payload_allocation() {
    let chain = small_block_chain();
    let mut input = frames(&chain);
    with_begin(&mut input, |claim| claim.block_bytes = 168 + 513 + 168);
    let receiver = feed(chain.manifest().identity(), &input[..3]).unwrap();
    let header = FrameHeaderV2::new(Kind::Block, 8 + 513).unwrap().encode();
    assert!(matches!(
        receiver.begin_frame(&header),
        Err(Error::BlockTooLarge { len: 513, max: 512 })
    ));
}

#[test]
fn manifest_rejects_claimed_total_impossible_under_its_configuration() {
    let chain = small_block_chain();
    let mut input = frames(&chain);
    with_begin(&mut input, |claim| claim.block_bytes = 168 + 2 * 512 + 1);
    let receiver = feed(chain.manifest().identity(), &input[..1]).unwrap();
    assert!(matches!(
        receiver.accept_frame(&input[1]),
        Err(Error::BlockBytesMismatch)
    ));
}

#[test]
fn remaining_total_reserves_minimum_space_for_every_future_block() {
    let input = frames(&main_chain(2));
    let receiver = feed(independent_identity(), &input[..3]).unwrap();
    // One extra byte in block 1 would leave only 167 for the required last MRB2.
    let header = FrameHeaderV2::new(Kind::Block, 8 + 1978).unwrap().encode();
    assert!(matches!(
        receiver.begin_frame(&header),
        Err(Error::BlockBytesMismatch)
    ));
}

#[test]
fn final_block_length_must_exactly_exhaust_the_claimed_total() {
    for difference in [-1i64, 1] {
        let mut input = frames(&main_chain(4));
        with_begin(&mut input, |claim| {
            claim.block_bytes = (claim.block_bytes as i64 + difference) as u64
        });
        let receiver = feed(independent_identity(), &input[..input.len() - 2]).unwrap();
        assert!(matches!(
            receiver.begin_frame(&input[input.len() - 2][..FRAME_HEADER_LEN_V2]),
            Err(Error::BlockBytesMismatch)
        ));
    }
}

#[test]
fn malformed_block_bytes_never_become_an_accepted_candidate() {
    let mut input = frames(&main_chain(4));
    let mut genesis = fixture_bytes("block0_block");
    genesis[..4].copy_from_slice(b"MPS2");
    replace_block(&mut input, 0, &genesis);
    assert!(matches!(receive(&input), Err(Error::Block(_))));
}

#[test]
fn invalid_seeded_genesis_work_is_rejected_by_actual_chain_acceptance() {
    let chain = main_chain(0);
    let mut input = frames(&chain);
    let genesis = chain.genesis();
    let wrong = (0u64..100)
        .find_map(|nonce| {
            let altered = reheader(&chain, genesis, |bytes| {
                bytes[152..160].copy_from_slice(&nonce.to_le_bytes())
            });
            (validate_header_pow_v2(altered.header(), chain.config().parameters().pow_bits)
                == Err(PowErrorV2::HashAboveTarget))
            .then_some(altered)
        })
        .unwrap();
    replace_block(&mut input, 0, &wrong.encode(chain.config()).unwrap());
    assert_chain_rejection(&input, ChainErrorV2::Work(PowErrorV2::HashAboveTarget));
}

#[test]
fn peer_cannot_choose_another_difficulty_for_a_child() {
    let chain = main_chain(1);
    let mut input = frames(&chain);
    let wrong = reheader(&chain, &block("block1"), |bytes| {
        bytes[148..152].copy_from_slice(&0x207f_fffeu32.to_le_bytes())
    });
    replace_block(&mut input, 1, &wrong.encode(chain.config()).unwrap());
    assert_chain_rejection(
        &input,
        ChainErrorV2::Work(PowErrorV2::WrongBits {
            expected: 0x207f_ffff,
            got: 0x207f_fffe,
        }),
    );
}

#[test]
fn valid_work_cannot_override_a_wrong_operation_root() {
    let chain = main_chain(1);
    let mut input = frames(&chain);
    let wrong = reheader(&chain, &block("block1"), |bytes| {
        bytes[76..108].copy_from_slice(&[88; 32])
    });
    let wrong = mined(&chain, &wrong);
    replace_block(&mut input, 1, &wrong.encode(chain.config()).unwrap());
    assert_chain_rejection(&input, ChainErrorV2::WrongOperationsRoot);
}

#[test]
fn independently_frozen_wrong_ledger_root_is_rejected() {
    let mut input = frames(&main_chain(1));
    replace_block(&mut input, 1, &fixture_bytes("wrong_root1_block"));
    assert_chain_rejection(&input, ChainErrorV2::WrongLedgerRoot);
}

#[test]
fn a_remined_block_with_a_bad_signature_is_rejected_during_execution() {
    let parent = main_chain(0);
    let valid = block("block1");
    let mut operations = valid.operations().to_vec();
    let SignedOperationV2::Transfer(transfer) = &operations[0] else {
        panic!("first fixture operation is a transfer");
    };
    operations[0] = SignedTransferV2::from_parts(
        *transfer.header(),
        *transfer.body(),
        SignatureBytes::from_bytes([0; 64]),
    )
    .into();
    assert!(operations[0].verify_signature().is_err());
    let wrong = rebody(&parent, &valid, &operations);
    let expected = parent.clone().append(wrong.clone()).unwrap_err();
    assert!(matches!(expected, ChainErrorV2::Ledger(_)));
    let mut input = frames(&main_chain(1));
    replace_block(&mut input, 1, &wrong.encode(parent.config()).unwrap());
    assert_chain_rejection(&input, expected);
}

#[test]
fn signatures_from_another_valid_branch_do_not_override_actual_ancestry() {
    let parent = fork_prefix(2);
    let foreign_operations = block("block3").operations().to_vec();
    for operation in &foreign_operations {
        operation.verify_signature().unwrap();
    }
    let wrong = rebody(&parent, &block("fork3"), &foreign_operations);
    let expected = parent.clone().append(wrong.clone()).unwrap_err();
    assert!(matches!(expected, ChainErrorV2::Ledger(_)));
    assert!(format!("{expected:?}").contains("WrongAnchorHash"));
    let mut input = frames(&fork_chain());
    replace_block(&mut input, 3, &wrong.encode(parent.config()).unwrap());
    assert_chain_rejection(&input, expected);
}

#[test]
fn actual_parent_hash_is_checked_for_each_received_block() {
    let chain = main_chain(1);
    let mut input = frames(&chain);
    let wrong = reheader(&chain, &block("block1"), |bytes| {
        bytes[44..76].copy_from_slice(&[91; 32])
    });
    replace_block(&mut input, 1, &wrong.encode(chain.config()).unwrap());
    assert_chain_rejection(&input, ChainErrorV2::WrongParent);
}

#[test]
fn sender_emits_exact_original_blocks_and_is_fused_after_end() {
    let chain = main_chain(4);
    let before = chain.clone();
    let mut sender = HistoryFramesV2::new(&chain).unwrap();
    let captured = sender.summary();
    assert_eq!(
        decode_frame_v2(&sender.next().unwrap().unwrap()).unwrap(),
        Frame::Begin(captured)
    );
    let manifest_frame = sender.next().unwrap().unwrap();
    assert_eq!(
        decode_frame_v2(&manifest_frame).unwrap(),
        Frame::Manifest(&fixture_bytes("manifest_hex"))
    );
    for height in 0..=4 {
        let frame = sender.next().unwrap().unwrap();
        assert_eq!(
            decode_frame_v2(&frame).unwrap(),
            Frame::Block {
                height,
                bytes: &fixture_bytes(&format!("block{height}_block"))
            }
        );
    }
    assert_eq!(
        decode_frame_v2(&sender.next().unwrap().unwrap()).unwrap(),
        Frame::End(captured)
    );
    assert!(sender.next().is_none());
    assert!(sender.next().is_none());
    assert_eq!(chain, before);
}

#[test]
fn legacy_v1_hello_is_not_a_v2_transfer() {
    let hello = mirror_protocol::WireMessage::Hello(mirror_protocol::HelloMessage::new(
        1,
        Hash256::default(),
        0,
        Hash256::default(),
    ));
    let bytes = mirror_protocol::encode_message(&hello).unwrap();
    assert!(matches!(
        Receiver::new(independent_identity()).accept_frame(&bytes),
        Err(Error::Protocol(_))
    ));
}

#[test]
fn transport_caps_match_the_frozen_chain_and_block_profile() {
    use mirror_protocol::v2 as wire;
    assert_eq!(
        wire::MAX_HISTORY_BLOCKS_V2,
        mirror_chain::v2::MAX_CHAIN_BLOCKS_V2
    );
    assert_eq!(
        wire::MAX_HISTORY_BLOCK_BYTES_V2,
        mirror_chain::v2::MAX_CHAIN_BYTES_V2
    );
    assert_eq!(
        wire::MAX_MANIFEST_BYTES_V2,
        mirror_chain::v2::manifest::MAX_MANIFEST_BYTES_V2
    );
    assert_eq!(
        wire::MIN_BLOCK_BYTES_V2,
        mirror_core::v2::MIN_ENCODED_BLOCK_LEN_V2
    );
    assert_eq!(
        wire::MAX_BLOCK_BYTES_V2,
        LimitsV2::default().max_block_bytes as usize
    );
    assert_eq!(wire::MAX_HISTORY_FRAMES_V2, wire::MAX_HISTORY_BLOCKS_V2 + 3);
    assert_eq!(
        wire::MAX_HISTORY_WIRE_BYTES_V2,
        340 + wire::MAX_MANIFEST_BYTES_V2
            + wire::MAX_HISTORY_BLOCK_BYTES_V2
            + 20 * wire::MAX_HISTORY_BLOCKS_V2
    );
}
