//! Synthetic MB-02A representation tests. These do not accept a ledger or chain.
use mirror_core::{
    Address, merkle_root,
    v2::{
        BLOCK_ENCODING_MAGIC_V2, BLOCK_HEADER_LEN_V2, BLOCK_VERSION_V2, BlockErrorV2 as Error,
        BlockHeaderV2 as Header, BlockV2 as Block, MIN_ENCODED_BLOCK_LEN_V2,
        SignedOperationV2 as Op, SignedTransferV2 as Transfer, TransferBodyV2, operations_root_v2,
    },
};
use mirror_crypto::{Hash256, Keypair, SignatureBytes, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2 as Action,
    OperationHeaderV2, ParentChainViewV2, SignedMembershipOperationV2 as Member,
};

fn hash(byte: u8) -> Hash256 {
    Hash256::from_bytes([byte; 32])
}
fn key() -> Keypair {
    Keypair::from_secret_bytes([1; 32])
}
fn config_with(limits: LimitsV2) -> ConfigV2 {
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
        limits,
        &[BootstrapDescriptor {
            public_key: key().public_key(),
            nonce: hash(3),
            evidence: hash(4),
        }],
        &[],
    )
    .unwrap()
}
fn config() -> ConfigV2 {
    config_with(LimitsV2::default())
}
fn op_header(c: &ConfigV2) -> OperationHeaderV2 {
    OperationHeaderV2 {
        config_hash: c.context(),
        anchor_height: 7,
        anchor_hash: hash(0x44),
        valid_from: 8,
        valid_until: 24,
        actor_public_key: key().public_key(),
        actor_sequence: 0,
    }
}
fn transfer(c: &ConfigV2) -> Op {
    Transfer::sign(
        op_header(c),
        TransferBodyV2 {
            recipient: Address::from_bytes([0x77; 32]),
            amount: 10_000_000,
            fee: 0,
        },
        &key(),
    )
    .unwrap()
    .into()
}
fn member(c: &ConfigV2, action: Action) -> Op {
    Member::sign(op_header(c), action, &key()).unwrap().into()
}
fn enroll(c: &ConfigV2) -> Op {
    member(
        c,
        Action::Enroll {
            applicant_nonce: hash(0x55),
            evidence_commitment: hash(0x66),
            evidence_profile: c.parameters().evidence_profile,
        },
    )
}
fn approve(c: &ConfigV2) -> Op {
    member(
        c,
        Action::Approve {
            target_enrollment_id: hash(0x77),
        },
    )
}
fn mixed(c: &ConfigV2) -> Vec<Op> {
    vec![
        transfer(c),
        enroll(c),
        approve(c),
        member(
            c,
            Action::Withdraw {
                own_member_id: hash(0x88),
            },
        ),
    ]
}
fn header(c: &ConfigV2, root: Hash256) -> Header {
    Header::new(
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
fn block(c: &ConfigV2, ops: &[Op]) -> Block {
    Block::new(c, header(c, operations_root_v2(c, ops).unwrap()), ops).unwrap()
}
fn raw(header: &Header, ops: &[Op]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"MRB2");
    bytes.extend_from_slice(&header.encode());
    bytes.extend_from_slice(&(ops.len() as u32).to_le_bytes());
    for op in ops {
        let b = op.encode();
        bytes.extend_from_slice(&(b.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&b);
    }
    bytes
}
fn leaf(op: &Op) -> Hash256 {
    let bytes = op.encode();
    let mut preimage = b"MIRROR_OP_LEAF_V2".to_vec();
    preimage.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    preimage.extend_from_slice(&bytes);
    sha256d(&preimage)
}

#[test]
fn exact_header_layout_and_unsigned_endianness() {
    let c = config();
    let h = header(&c, hash(0x33));
    let bytes = h.encode();
    assert_eq!(BLOCK_VERSION_V2, 2);
    assert_eq!(BLOCK_HEADER_LEN_V2, 160);
    assert_eq!(bytes.len(), 160);
    assert_eq!(&bytes[0..4], &2u32.to_le_bytes());
    assert_eq!(&bytes[4..36], c.context().as_bytes());
    assert_eq!(&bytes[36..44], &8u64.to_le_bytes());
    assert_eq!(&bytes[44..76], hash(0x44).as_bytes());
    assert_eq!(&bytes[76..108], hash(0x33).as_bytes());
    assert_eq!(&bytes[108..140], hash(0x99).as_bytes());
    assert_eq!(&bytes[140..148], &1_800_000_123u64.to_le_bytes());
    assert_eq!(&bytes[148..152], &0x207f_ffffu32.to_le_bytes());
    assert_eq!(&bytes[152..160], &0x0102_0304_0506_0708u64.to_le_bytes());
    assert_eq!(Header::decode(&bytes), Ok(h));
}

#[test]
fn header_accessors_round_trip_maximum_integer_values() {
    let c = config();
    let h = Header::new(
        c.context(),
        u64::MAX,
        hash(1),
        hash(2),
        hash(3),
        u64::MAX,
        u32::MAX,
        u64::MAX,
    );
    let decoded = Header::decode(&h.encode()).unwrap();
    assert_eq!(decoded.version(), 2);
    assert_eq!(decoded.config_hash(), c.context());
    assert_eq!(decoded.height(), u64::MAX);
    assert_eq!(decoded.previous_hash(), hash(1));
    assert_eq!(decoded.operations_root(), hash(2));
    assert_eq!(decoded.ledger_root(), hash(3));
    assert_eq!(decoded.timestamp(), u64::MAX);
    assert_eq!(decoded.pow_bits(), u32::MAX);
    assert_eq!(decoded.nonce(), u64::MAX);
}

#[test]
fn every_header_truncation_and_trailing_byte_is_rejected() {
    let c = config();
    let bytes = header(&c, hash(3)).encode();
    for end in 0..bytes.len() {
        assert_eq!(
            Header::decode(&bytes[..end]),
            Err(Error::InvalidHeaderLength)
        );
    }
    let mut bytes = bytes.to_vec();
    bytes.push(0);
    assert_eq!(Header::decode(&bytes), Err(Error::InvalidHeaderLength));
}

#[test]
fn unsupported_header_versions_never_decode() {
    let c = config();
    for version in [0u32, 1, 3, u32::MAX] {
        let mut bytes = header(&c, hash(3)).encode();
        bytes[..4].copy_from_slice(&version.to_le_bytes());
        assert_eq!(
            Header::decode(&bytes),
            Err(Error::UnsupportedVersion(version))
        );
        let mut full = raw(&header(&c, hash(3)), &[]);
        full[4..8].copy_from_slice(&version.to_le_bytes());
        assert_eq!(
            Block::decode(&c, &full),
            Err(Error::UnsupportedVersion(version))
        );
    }
}

#[test]
fn every_nonversion_header_byte_is_bound_to_the_hash() {
    let c = config();
    let h = header(&c, hash(3));
    let bytes = h.encode();
    for offset in 4..160 {
        let mut changed = bytes;
        changed[offset] ^= 1;
        let other = Header::decode(&changed).unwrap();
        assert_ne!(other.hash(), h.hash(), "offset {offset}");
    }
    assert_ne!(h.hash(), sha256d(&bytes)); // Header uses its explicit V2 domain.
}

#[test]
fn empty_block_has_exact_size_magic_count_and_identity() {
    let c = config();
    let b = block(&c, &[]);
    let bytes = b.encode(&c).unwrap();
    assert_eq!(BLOCK_ENCODING_MAGIC_V2, *b"MRB2");
    assert_eq!(MIN_ENCODED_BLOCK_LEN_V2, 168);
    assert_eq!(b.encoded_len(), 168);
    assert_eq!(bytes.len(), 168);
    assert_eq!(&bytes[..4], b"MRB2");
    assert_eq!(&bytes[164..168], &[0; 4]);
    assert_eq!(b.hash(), b.header().hash());
    assert!(b.operations().is_empty());
    assert_eq!(Block::decode(&c, &bytes), Ok(b.clone()));
    assert_eq!(b.verify_integrity(&c), Ok(()));
}

#[test]
fn mixed_block_round_trip_preserves_global_positions_and_exact_bytes() {
    let c = config();
    let ops = mixed(&c);
    let b = block(&c, &ops);
    let bytes = b.encode(&c).unwrap();
    assert_eq!(b.encoded_len(), 1172);
    assert_eq!(bytes.len(), 1172);
    assert_eq!(&bytes[164..168], &4u32.to_le_bytes());
    let decoded = Block::decode(&c, &bytes).unwrap();
    assert_eq!(decoded, b);
    assert_eq!(decoded.encode(&c).unwrap(), bytes);
    assert_eq!(decoded.operations(), &ops);
    assert_eq!(
        decoded
            .operations()
            .iter()
            .map(Op::kind)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_eq!(decoded.verify_integrity(&c), Ok(()));
}

#[test]
fn every_complete_block_truncation_is_rejected() {
    let c = config();
    for ops in [vec![], vec![transfer(&c)], mixed(&c)] {
        let bytes = block(&c, &ops).encode(&c).unwrap();
        for end in 0..bytes.len() {
            assert!(Block::decode(&c, &bytes[..end]).is_err(), "length {end}");
        }
    }
}

#[test]
fn bad_magic_and_legacy_bytes_never_fall_back() {
    let c = config();
    let original = block(&c, &[]).encode(&c).unwrap();
    for magic in [*b"MRB1", *b"MPH2", *b"MCF2", [0; 4]] {
        let mut bytes = original.clone();
        bytes[..4].copy_from_slice(&magic);
        assert_eq!(Block::decode(&c, &bytes), Err(Error::InvalidMagic));
    }
}

#[test]
fn complete_trailing_data_is_rejected_for_empty_and_mixed_blocks() {
    let c = config();
    for ops in [vec![], mixed(&c)] {
        let mut bytes = block(&c, &ops).encode(&c).unwrap();
        bytes.push(0);
        assert_eq!(Block::decode(&c, &bytes), Err(Error::TrailingBytes));
    }
}

#[test]
fn oversized_block_is_rejected_before_header_parsing() {
    let c = config();
    let bytes = vec![0; c.limits().max_block_bytes as usize + 1];
    assert_eq!(Block::decode(&c, &bytes), Err(Error::TooLarge));
}

#[test]
fn huge_declared_count_rejects_before_body_allocation() {
    let c = config();
    let mut bytes = block(&c, &[]).encode(&c).unwrap();
    bytes[164..168].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(Block::decode(&c, &bytes), Err(Error::TooManyOperations));
}

#[test]
fn plausible_count_without_actual_minimum_records_is_rejected() {
    let c = config();
    let mut bytes = block(&c, &[]).encode(&c).unwrap();
    bytes[164..168].copy_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(Block::decode(&c, &bytes), Err(Error::Truncated));
    let mut bytes = block(&c, &[transfer(&c)]).encode(&c).unwrap();
    bytes[164..168].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(Block::decode(&c, &bytes), Err(Error::Truncated));
}

#[test]
fn huge_record_length_rejects_before_record_copying() {
    let c = config();
    let mut bytes = block(&c, &[transfer(&c)]).encode(&c).unwrap();
    bytes[168..172].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        Block::decode(&c, &bytes),
        Err(Error::OperationTooLarge { index: 0 })
    );
}

#[test]
fn bounded_but_truncated_record_is_rejected() {
    let c = config();
    let mut bytes = block(&c, &[transfer(&c)]).encode(&c).unwrap();
    bytes[168..172].copy_from_slice(&300u32.to_le_bytes());
    assert_eq!(Block::decode(&c, &bytes), Err(Error::Truncated));
}

#[test]
fn wrong_exact_record_lengths_and_unknown_kinds_are_rejected() {
    let c = config();
    let original = block(&c, &[transfer(&c)]).encode(&c).unwrap();
    for length in [0u32, 1, 226, 227, 242] {
        let mut bytes = original.clone();
        bytes[168..172].copy_from_slice(&length.to_le_bytes());
        assert!(matches!(
            Block::decode(&c, &bytes),
            Err(Error::Operation { index: 0, .. })
        ));
    }
    for kind in [1u8, 2, 3, 4, 255] {
        let mut bytes = original.clone();
        bytes[174] = kind;
        assert!(matches!(
            Block::decode(&c, &bytes),
            Err(Error::Operation { index: 0, .. })
        ));
    }
    let mut bytes = original;
    bytes[172..174].copy_from_slice(&1u16.to_le_bytes());
    assert!(matches!(
        Block::decode(&c, &bytes),
        Err(Error::Operation { index: 0, .. })
    ));
}

#[test]
fn all_operation_count_boundaries_apply_to_construct_encode_decode_and_root() {
    let c = config_with(LimitsV2 {
        max_operations: 2,
        max_membership_operations: 2,
        max_new_enrollments: 2,
        ..LimitsV2::default()
    });
    let two = [transfer(&c); 2];
    let three = [transfer(&c); 3];
    let b = block(&c, &two);
    assert_eq!(b.operations().len(), 2);
    assert!(Block::decode(&c, &b.encode(&c).unwrap()).is_ok());
    assert_eq!(
        Block::new(&c, *b.header(), &three),
        Err(Error::TooManyOperations)
    );
    assert_eq!(
        operations_root_v2(&c, &three),
        Err(Error::TooManyOperations)
    );
    assert_eq!(
        Block::decode(&c, &raw(b.header(), &three)),
        Err(Error::TooManyOperations)
    );
    let generous = config();
    let larger = block(&generous, &[transfer(&generous); 3]);
    assert_eq!(larger.encode(&c), Err(Error::TooManyOperations));
    assert_eq!(larger.verify_integrity(&c), Err(Error::TooManyOperations));
}

#[test]
fn full_default_operation_count_accepts_transfers_without_membership_quota() {
    let c = config_with(LimitsV2 {
        max_membership_operations: 1,
        max_new_enrollments: 1,
        ..LimitsV2::default()
    });
    let ops = vec![transfer(&c); 256];
    let b = block(&c, &ops);
    assert_eq!(Block::decode(&c, &b.encode(&c).unwrap()), Ok(b));
    let too_many = vec![transfer(&c); 257];
    assert_eq!(
        operations_root_v2(&c, &too_many),
        Err(Error::TooManyOperations)
    );
}

#[test]
fn membership_subcount_is_enforced_across_interspersed_transfers() {
    let c = config_with(LimitsV2 {
        max_membership_operations: 2,
        max_new_enrollments: 1,
        ..LimitsV2::default()
    });
    let allowed = [approve(&c), transfer(&c), approve(&c), transfer(&c)];
    let denied = [
        approve(&c),
        transfer(&c),
        approve(&c),
        transfer(&c),
        approve(&c),
    ];
    let b = block(&c, &allowed);
    assert!(Block::decode(&c, &b.encode(&c).unwrap()).is_ok());
    assert_eq!(
        Block::new(&c, *b.header(), &denied),
        Err(Error::TooManyMembershipOperations)
    );
    assert_eq!(
        operations_root_v2(&c, &denied),
        Err(Error::TooManyMembershipOperations)
    );
    assert_eq!(
        Block::decode(&c, &raw(b.header(), &denied)),
        Err(Error::TooManyMembershipOperations)
    );
}

#[test]
fn enrollment_subcount_is_enforced_separately_from_membership_count() {
    let c = config_with(LimitsV2 {
        max_new_enrollments: 1,
        ..LimitsV2::default()
    });
    let allowed = [transfer(&c), enroll(&c), approve(&c)];
    let denied = [enroll(&c), transfer(&c), enroll(&c)];
    let b = block(&c, &allowed);
    assert!(Block::decode(&c, &b.encode(&c).unwrap()).is_ok());
    assert_eq!(
        Block::new(&c, *b.header(), &denied),
        Err(Error::TooManyEnrollments)
    );
    assert_eq!(
        operations_root_v2(&c, &denied),
        Err(Error::TooManyEnrollments)
    );
    assert_eq!(
        Block::decode(&c, &raw(b.header(), &denied)),
        Err(Error::TooManyEnrollments)
    );
}

#[test]
fn default_enrollment_boundary_is_64_not_total_operation_capacity() {
    let c = config();
    let exact = vec![enroll(&c); 64];
    let b = block(&c, &exact);
    assert!(Block::decode(&c, &b.encode(&c).unwrap()).is_ok());
    let over = vec![enroll(&c); 65];
    assert_eq!(
        Block::new(&c, *b.header(), &over),
        Err(Error::TooManyEnrollments)
    );
    assert_eq!(
        Block::decode(&c, &raw(b.header(), &over)),
        Err(Error::TooManyEnrollments)
    );
}

#[test]
fn exact_full_block_byte_budget_and_one_byte_less_are_distinct() {
    // Two transfers require 168 + 2*(4+243) = 662 bytes. The largest permitted
    // operation must still fit the configured block, hence max_operation=291.
    let limits = LimitsV2 {
        max_operation_bytes: 291,
        max_block_bytes: 662,
        ..LimitsV2::default()
    };
    let c = config_with(limits);
    let ops = [transfer(&c); 2];
    let b = block(&c, &ops);
    assert_eq!(b.encoded_len(), 662);
    assert_eq!(Block::decode(&c, &b.encode(&c).unwrap()), Ok(b.clone()));
    let strict = config_with(LimitsV2 {
        max_block_bytes: 661,
        ..limits
    });
    assert_eq!(Block::new(&strict, *b.header(), &ops), Err(Error::TooLarge));
    assert_eq!(operations_root_v2(&strict, &ops), Err(Error::TooLarge));
    assert_eq!(b.encode(&strict), Err(Error::TooLarge));
    assert_eq!(b.verify_integrity(&strict), Err(Error::TooLarge));
    assert_eq!(
        Block::decode(&strict, &raw(b.header(), &ops)),
        Err(Error::TooLarge)
    );
}

#[test]
fn encoding_and_integrity_recheck_stricter_membership_and_enrollment_budgets() {
    let generous = config();
    let ops = [enroll(&generous), approve(&generous), enroll(&generous)];
    let b = block(&generous, &ops);
    let membership = config_with(LimitsV2 {
        max_membership_operations: 1,
        max_new_enrollments: 1,
        ..LimitsV2::default()
    });
    assert_eq!(
        b.encode(&membership),
        Err(Error::TooManyMembershipOperations)
    );
    assert_eq!(
        b.verify_integrity(&membership),
        Err(Error::TooManyMembershipOperations)
    );
    let enrollment = config_with(LimitsV2 {
        max_new_enrollments: 1,
        ..LimitsV2::default()
    });
    assert_eq!(b.encode(&enrollment), Err(Error::TooManyEnrollments));
    assert_eq!(
        b.verify_integrity(&enrollment),
        Err(Error::TooManyEnrollments)
    );
}

#[test]
fn count_commitment_distinguishes_the_duplicate_last_merkle_ambiguity() {
    let c = config();
    let three = [transfer(&c), enroll(&c), approve(&c)];
    let four = [three[0], three[1], three[2], three[2]];
    assert_eq!(
        merkle_root(&three.map(|op| leaf(&op))),
        merkle_root(&four.map(|op| leaf(&op)))
    );
    assert_ne!(
        operations_root_v2(&c, &three).unwrap(),
        operations_root_v2(&c, &four).unwrap()
    );
}

#[test]
fn ordered_mixed_operation_commitment_detects_reordering() {
    let c = config();
    let original = mixed(&c);
    let b = block(&c, &original);
    let mut reordered = original.clone();
    reordered.swap(0, 1);
    assert_ne!(
        operations_root_v2(&c, &original),
        operations_root_v2(&c, &reordered)
    );
    assert_eq!(
        Block::new(&c, *b.header(), &reordered)
            .unwrap()
            .verify_integrity(&c),
        Err(Error::OperationsRootMismatch)
    );
}

#[test]
fn complete_signed_bytes_including_signatures_are_committed() {
    let c = config();
    let original = transfer(&c);
    let mut encoded = original.encode();
    let last = encoded.len() - 1;
    encoded[last] ^= 1;
    let changed = Op::decode(&c, &encoded).unwrap();
    assert_ne!(
        operations_root_v2(&c, &[original]),
        operations_root_v2(&c, &[changed])
    );
}

#[test]
fn corrupted_body_decodes_but_fails_the_original_root() {
    let c = config();
    let mut bytes = block(&c, &[transfer(&c)]).encode(&c).unwrap();
    bytes[172 + 131] ^= 1; // Fixed-size recipient data, leaving framing intact.
    let decoded = Block::decode(&c, &bytes).unwrap();
    assert_eq!(
        decoded.verify_integrity(&c),
        Err(Error::OperationsRootMismatch)
    );
}

#[test]
fn corrupted_signature_with_recomputed_root_still_fails_integrity() {
    let c = config();
    let mut ops = mixed(&c);
    if let Op::Transfer(t) = ops[0] {
        ops[0] = Transfer::from_parts(*t.header(), *t.body(), SignatureBytes::from_bytes([0; 64]))
            .into();
    } else {
        panic!("fixture must start with transfer");
    }
    let b = block(&c, &ops); // The new root deliberately commits the invalid signature.
    let decoded = Block::decode(&c, &b.encode(&c).unwrap()).unwrap();
    assert!(matches!(
        decoded.verify_integrity(&c),
        Err(Error::Operation { index: 0, .. })
    ));
}

#[test]
fn invalid_final_membership_signature_rejects_at_its_original_index() {
    let c = config();
    let mut ops = mixed(&c);
    let mut bytes = ops[3].encode();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    ops[3] = Op::decode(&c, &bytes).unwrap();
    let b = block(&c, &ops);
    assert!(matches!(
        b.verify_integrity(&c),
        Err(Error::Operation { index: 3, .. })
    ));
    assert_eq!(b.operations(), &ops); // Integrity validation is nonmutating.
}

#[test]
fn header_context_mismatch_is_distinct_from_structural_decoding() {
    let c = config();
    let original = block(&c, &mixed(&c));
    let mut bytes = original.encode(&c).unwrap();
    bytes[8] ^= 1;
    let b = Block::decode(&c, &bytes).unwrap();
    assert_eq!(b.verify_integrity(&c), Err(Error::WrongContext));
}

#[test]
fn signed_operation_context_must_match_the_block_configuration() {
    let c = config();
    let mut params = c.parameters();
    params.network = hash(0x12);
    let other = ConfigV2::new(params, *c.limits(), c.bootstrap(), c.allocations()).unwrap();
    let ops = [transfer(&c), enroll(&c), approve(&other)];
    assert_eq!(ops[2].verify_signature(), Ok(()));
    let b = block(&c, &ops);
    assert_eq!(
        b.verify_integrity(&c),
        Err(Error::OperationContext { index: 2 })
    );
}

#[test]
fn construction_copies_input_and_exposes_no_mutating_body_access() {
    let c = config();
    let mut ops = mixed(&c);
    let expected = ops.clone();
    let b = block(&c, &ops);
    ops.swap(0, 1);
    ops.clear();
    assert_eq!(b.operations(), &expected);
    assert_eq!(b.verify_integrity(&c), Ok(()));
}

struct View(Hash256);
impl ParentChainViewV2 for View {
    fn config_hash(&self) -> Hash256 {
        self.0
    }
    fn parent_height(&self) -> u64 {
        7
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height == 7).then(|| hash(0x44))
    }
}

#[test]
fn integrity_does_not_claim_transfer_policy_or_blockchain_acceptance() {
    let c = config();
    let invalid_policy = Transfer::sign(
        op_header(&c),
        TransferBodyV2 {
            recipient: Address::default(),
            amount: 0,
            fee: 7,
        },
        &key(),
    )
    .unwrap();
    assert!(
        invalid_policy
            .validate_for_parent(&c, &View(c.context()))
            .is_err()
    );
    let ops = [invalid_policy.into()];
    let h = Header::new(
        c.context(),
        u64::MAX,
        hash(0xff),
        operations_root_v2(&c, &ops).unwrap(),
        Hash256::default(),
        0,
        0,
        0,
    );
    let b = Block::new(&c, h, &ops).unwrap();
    // Signed body/config commitment is valid. MB-02B/C must still reject invalid
    // execution policy and independently validate height, parent, ledger and PoW.
    assert_eq!(b.verify_integrity(&c), Ok(()));
    assert_eq!(Block::decode(&c, &b.encode(&c).unwrap()), Ok(b));
}
