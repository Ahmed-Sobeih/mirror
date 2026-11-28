//! Frozen, independently packed MPN2 frames and untrusted-input boundaries.

use std::{collections::BTreeMap, sync::OnceLock};

use mirror_crypto::{Hash256, sha256d};
use mirror_protocol::v2::*;
use mirror_protocol::{HelloMessage, ProtocolError, WireMessage, decode_message, encode_message};

fn vectors() -> &'static BTreeMap<&'static str, &'static str> {
    static VALUES: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    VALUES.get_or_init(|| {
        include_str!("vectors/peer_history_v2.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_once('=').unwrap())
            .collect()
    })
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn bytes(key: &str) -> Vec<u8> {
    unhex(vectors()[key])
}

fn summary(label: &str) -> HistorySummaryV2 {
    HistorySummaryV2::decode(&bytes(&format!("{label}_summary"))).unwrap()
}

fn raw_header(kind: u16, len: u32) -> [u8; 12] {
    let mut bytes = [0; 12];
    bytes[..4].copy_from_slice(b"MPN2");
    bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
    bytes[6..8].copy_from_slice(&kind.to_le_bytes());
    bytes[8..12].copy_from_slice(&len.to_le_bytes());
    bytes
}

#[test]
fn summaries_and_control_frames_match_independent_vectors() {
    for label in ["genesis", "main3", "main4", "fork3"] {
        let body = bytes(&format!("{label}_summary"));
        let claim = summary(label);
        assert_eq!(claim.encode().unwrap().as_slice(), body);
        assert_eq!(
            claim.block_count.to_string(),
            vectors()[format!("{label}_block_count").as_str()]
        );
        assert_eq!(
            claim.block_bytes.to_string(),
            vectors()[format!("{label}_block_bytes").as_str()]
        );
        assert_eq!(claim.height + 1, u64::from(claim.block_count));
        for (name, frame) in [
            ("begin", FrameRefV2::Begin(claim)),
            ("end", FrameRefV2::End(claim)),
        ] {
            let expected = bytes(&format!("{label}_{name}"));
            assert_eq!(encode_frame_v2(frame).unwrap(), expected);
            assert_eq!(decode_frame_v2(&expected).unwrap(), frame);
        }
    }
}

#[test]
fn manifest_and_complete_block_frames_match_independent_chain_bytes() {
    let chain: BTreeMap<_, _> = include_str!("../../mirror-chain/tests/vectors/chain_v2.txt")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.split_once('=').unwrap())
        .collect();
    let manifest = unhex(chain["manifest_hex"]);
    let frame = bytes("manifest_frame");
    assert_eq!(
        encode_frame_v2(FrameRefV2::Manifest(&manifest)).unwrap(),
        frame
    );
    assert_eq!(
        decode_frame_v2(&frame).unwrap(),
        FrameRefV2::Manifest(&manifest)
    );
    for name in [
        "block0", "block1", "block2", "block3", "block4", "fork2", "fork3",
    ] {
        let block = unhex(chain[format!("{name}_block").as_str()]);
        let height = chain[format!("{name}_height").as_str()].parse().unwrap();
        let expected = bytes(&format!("{name}_frame"));
        let value = FrameRefV2::Block {
            height,
            bytes: &block,
        };
        assert_eq!(encode_frame_v2(value).unwrap(), expected);
        assert_eq!(decode_frame_v2(&expected).unwrap(), value);
    }
}

#[test]
fn full_transcript_lengths_and_hashes_match_independent_reference() {
    for (label, names) in [
        ("genesis", vec!["block0"]),
        ("main3", vec!["block0", "block1", "block2", "block3"]),
        (
            "main4",
            vec!["block0", "block1", "block2", "block3", "block4"],
        ),
        ("fork3", vec!["block0", "block1", "fork2", "fork3"]),
    ] {
        let mut stream = bytes(&format!("{label}_begin"));
        stream.extend_from_slice(&bytes("manifest_frame"));
        for name in &names {
            stream.extend_from_slice(&bytes(&format!("{name}_frame")));
        }
        stream.extend_from_slice(&bytes(&format!("{label}_end")));
        assert_eq!(stream.len(), summary(label).wire_len().unwrap());
        assert_eq!(
            stream.len().to_string(),
            vectors()[format!("{label}_wire_len").as_str()]
        );
        assert_eq!(
            (names.len() + 3).to_string(),
            vectors()[format!("{label}_frame_count").as_str()]
        );
        assert_eq!(
            sha256d(&stream).as_bytes().as_slice(),
            bytes(&format!("{label}_stream_sha256d"))
        );
    }
}

#[test]
fn maximum_structural_shape_has_exact_inclusive_wire_bound() {
    let claim = summary("maximum_shape");
    assert_eq!(claim.block_count, 4096);
    assert_eq!(claim.height, 4095);
    assert_eq!(claim.wire_len().unwrap(), MAX_HISTORY_WIRE_BYTES_V2);
    assert_eq!(
        MAX_HISTORY_WIRE_BYTES_V2.to_string(),
        vectors()["max_wire_bytes"]
    );
    assert_eq!(MAX_HISTORY_FRAMES_V2, MAX_HISTORY_BLOCKS_V2 + 3);
}

#[test]
fn header_vectors_cover_every_kind_and_inclusive_size_boundary() {
    for (name, kind, len) in [
        ("begin", FrameKindV2::Begin, 152),
        ("end", FrameKindV2::End, 152),
        ("manifest_min", FrameKindV2::Manifest, 46),
        ("manifest_max", FrameKindV2::Manifest, 2_781_636),
        ("block_min", FrameKindV2::Block, 176),
        ("block_max", FrameKindV2::Block, 1_048_584),
    ] {
        let header = FrameHeaderV2::new(kind, len).unwrap();
        assert_eq!(header.encode().as_slice(), bytes(&format!("{name}_header")));
        assert_eq!(header, FrameHeaderV2::decode(&header.encode()).unwrap());
        assert_eq!(header.kind(), kind);
        assert_eq!(header.payload_len(), len);
    }
}

#[test]
fn short_or_extended_header_is_rejected() {
    let header = raw_header(1, 152);
    for size in 0..12 {
        assert_eq!(
            FrameHeaderV2::decode(&header[..size]),
            Err(ProtocolErrorV2::InvalidHeaderLength(size))
        );
    }
    let mut extended = header.to_vec();
    extended.push(0);
    assert_eq!(
        FrameHeaderV2::decode(&extended),
        Err(ProtocolErrorV2::InvalidHeaderLength(13))
    );
}

#[test]
fn invalid_magic_is_rejected_before_payload() {
    let mut header = raw_header(1, 152);
    header[0] ^= 1;
    assert_eq!(
        FrameHeaderV2::decode(&header),
        Err(ProtocolErrorV2::InvalidMagic)
    );
}

#[test]
fn unsupported_versions_are_rejected_before_payload() {
    for version in [0u16, 1, 3, u16::MAX] {
        let mut header = raw_header(1, 152);
        header[4..6].copy_from_slice(&version.to_le_bytes());
        assert_eq!(
            FrameHeaderV2::decode(&header),
            Err(ProtocolErrorV2::UnsupportedVersion(version))
        );
    }
}

#[test]
fn unknown_kinds_are_rejected_before_payload() {
    for kind in [0u16, 5, 255, u16::MAX] {
        assert_eq!(
            FrameHeaderV2::decode(&raw_header(kind, 152)),
            Err(ProtocolErrorV2::UnknownKind(kind))
        );
    }
}

#[test]
fn exact_control_payload_sizes_are_enforced_from_header_only() {
    for kind in [FrameKindV2::Begin, FrameKindV2::End] {
        for len in [0usize, 151, 153, u32::MAX as usize] {
            assert!(matches!(
                FrameHeaderV2::new(kind, len),
                Err(ProtocolErrorV2::InvalidPayloadLength { .. })
            ));
            assert!(matches!(
                FrameHeaderV2::decode(&raw_header(kind as u16, len as u32)),
                Err(ProtocolErrorV2::InvalidPayloadLength { .. })
            ));
        }
    }
}

#[test]
fn variable_payload_bounds_are_enforced_from_header_only() {
    for (kind, min, max) in [
        (FrameKindV2::Manifest, 46usize, 2_781_636usize),
        (FrameKindV2::Block, 176, 1_048_584),
    ] {
        for len in [0, min - 1, max + 1, u32::MAX as usize] {
            assert!(matches!(
                FrameHeaderV2::decode(&raw_header(kind as u16, len as u32)),
                Err(ProtocolErrorV2::InvalidPayloadLength { .. })
            ));
        }
        assert!(FrameHeaderV2::new(kind, usize::MAX).is_err());
    }
}

#[test]
fn summary_count_and_height_are_bounded_and_agree() {
    for count in [0u32, 4097, u32::MAX] {
        let claim = HistorySummaryV2 {
            block_count: count,
            ..summary("genesis")
        };
        assert_eq!(
            claim.validate(),
            Err(ProtocolErrorV2::InvalidBlockCount(count))
        );
        assert!(claim.encode().is_err());
        assert!(claim.wire_len().is_err());
    }
    for height in [1u64, 4095, u64::MAX] {
        let claim = HistorySummaryV2 {
            height,
            ..summary("genesis")
        };
        assert_eq!(
            claim.validate(),
            Err(ProtocolErrorV2::InvalidHeight {
                height,
                block_count: 1
            })
        );
    }
}

#[test]
fn summary_manifest_length_is_bounded() {
    for len in [0u32, 45, 2_781_637, u32::MAX] {
        let claim = HistorySummaryV2 {
            manifest_bytes: len,
            ..summary("genesis")
        };
        assert_eq!(
            claim.validate(),
            Err(ProtocolErrorV2::InvalidManifestLength(len))
        );
    }
    for len in [46, 2_781_636] {
        assert!(
            HistorySummaryV2 {
                manifest_bytes: len,
                ..summary("genesis")
            }
            .validate()
            .is_ok()
        );
    }
}

#[test]
fn declared_block_bytes_respect_count_and_global_total() {
    for len in [0u64, 167, 1_048_577, u64::MAX] {
        let claim = HistorySummaryV2 {
            block_bytes: len,
            ..summary("genesis")
        };
        assert_eq!(
            claim.validate(),
            Err(ProtocolErrorV2::InvalidBlockBytes(len))
        );
    }
    let maximum = summary("maximum_shape");
    for len in [4096 * 168 - 1, 16_777_217, u64::MAX] {
        assert_eq!(
            HistorySummaryV2 {
                block_bytes: len,
                ..maximum
            }
            .validate(),
            Err(ProtocolErrorV2::InvalidBlockBytes(len))
        );
    }
    assert!(
        HistorySummaryV2 {
            block_bytes: 4096 * 168,
            ..maximum
        }
        .validate()
        .is_ok()
    );
}

#[test]
fn summary_decoder_requires_exact_length_and_checks_untrusted_fields() {
    let data = bytes("main4_summary");
    for size in 0..152 {
        assert_eq!(
            HistorySummaryV2::decode(&data[..size]),
            Err(ProtocolErrorV2::InvalidSummaryLength(size))
        );
    }
    let mut extended = data.clone();
    extended.push(0);
    assert_eq!(
        HistorySummaryV2::decode(&extended),
        Err(ProtocolErrorV2::InvalidSummaryLength(153))
    );
    let mut huge_count = data.clone();
    huge_count[136..140].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        HistorySummaryV2::decode(&huge_count),
        Err(ProtocolErrorV2::InvalidBlockCount(u32::MAX))
    );
    let mut huge_height = data;
    huge_height[64..72].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        HistorySummaryV2::decode(&huge_height),
        Err(ProtocolErrorV2::InvalidHeight { .. })
    ));
}

#[test]
fn all_truncations_of_each_frame_shape_are_rejected() {
    for name in [
        "main4_begin",
        "manifest_frame",
        "block0_frame",
        "block1_frame",
        "main4_end",
    ] {
        let frame = bytes(name);
        for size in 0..frame.len() {
            assert!(
                decode_frame_v2(&frame[..size]).is_err(),
                "{name} prefix {size}"
            );
        }
    }
}

#[test]
fn trailing_bytes_and_concatenated_frames_are_rejected() {
    for name in ["main4_begin", "manifest_frame", "block0_frame", "main4_end"] {
        let mut frame = bytes(name);
        frame.push(0);
        assert_eq!(decode_frame_v2(&frame), Err(ProtocolErrorV2::TrailingBytes));
    }
    let mut frames = bytes("main4_begin");
    frames.extend_from_slice(&bytes("manifest_frame"));
    assert_eq!(
        decode_frame_v2(&frames),
        Err(ProtocolErrorV2::TrailingBytes)
    );
}

#[test]
fn split_header_payload_decoding_enforces_actual_payload_length() {
    let frame = bytes("block1_frame");
    let header = FrameHeaderV2::decode(&frame[..12]).unwrap();
    assert_eq!(
        decode_payload_v2(header, &frame[12..]).unwrap(),
        decode_frame_v2(&frame).unwrap()
    );
    assert_eq!(
        decode_payload_v2(header, &frame[12..frame.len() - 1]),
        Err(ProtocolErrorV2::TruncatedFrame)
    );
    let mut payload = frame[12..].to_vec();
    payload.push(0);
    assert_eq!(
        decode_payload_v2(header, &payload),
        Err(ProtocolErrorV2::TrailingBytes)
    );
}

#[test]
fn decoded_payloads_borrow_original_bytes_without_copying() {
    let manifest = bytes("manifest_frame");
    match decode_frame_v2(&manifest).unwrap() {
        FrameRefV2::Manifest(payload) => assert_eq!(payload.as_ptr(), manifest[12..].as_ptr()),
        _ => panic!("manifest variant"),
    }
    let block = bytes("block1_frame");
    match decode_frame_v2(&block).unwrap() {
        FrameRefV2::Block { bytes: payload, .. } => {
            assert_eq!(payload.as_ptr(), block[20..].as_ptr())
        }
        _ => panic!("block variant"),
    }
}

#[test]
fn block_ordinal_has_inclusive_maximum_and_rejects_overflow_values() {
    let block = [0u8; 168];
    let frame = encode_frame_v2(FrameRefV2::Block {
        height: 4095,
        bytes: &block,
    })
    .unwrap();
    assert_eq!(
        decode_frame_v2(&frame).unwrap(),
        FrameRefV2::Block {
            height: 4095,
            bytes: &block
        }
    );
    for height in [4096u64, u64::MAX] {
        assert_eq!(
            encode_frame_v2(FrameRefV2::Block {
                height,
                bytes: &block
            }),
            Err(ProtocolErrorV2::BlockHeightOutOfRange(height))
        );
        let mut invalid = frame.clone();
        invalid[12..20].copy_from_slice(&height.to_le_bytes());
        assert_eq!(
            decode_frame_v2(&invalid),
            Err(ProtocolErrorV2::BlockHeightOutOfRange(height))
        );
    }
}

#[test]
fn encoder_rejects_out_of_range_payloads_before_returning_frames() {
    let short_manifest = [0; 45];
    let short_block = [0; 167];
    assert!(encode_frame_v2(FrameRefV2::Manifest(&short_manifest)).is_err());
    assert!(
        encode_frame_v2(FrameRefV2::Block {
            height: 0,
            bytes: &short_block
        })
        .is_err()
    );
    let manifest = vec![0; MAX_MANIFEST_BYTES_V2 + 1];
    let block = vec![0; MAX_BLOCK_BYTES_V2 + 1];
    assert!(encode_frame_v2(FrameRefV2::Manifest(&manifest)).is_err());
    assert!(
        encode_frame_v2(FrameRefV2::Block {
            height: 0,
            bytes: &block
        })
        .is_err()
    );
}

#[test]
fn maximum_payload_bytes_round_trip_without_weakening_header_limits() {
    let manifest = vec![0x5a; MAX_MANIFEST_BYTES_V2];
    let frame = encode_frame_v2(FrameRefV2::Manifest(&manifest)).unwrap();
    assert_eq!(frame.len(), MAX_MANIFEST_BYTES_V2 + 12);
    assert_eq!(
        decode_frame_v2(&frame).unwrap(),
        FrameRefV2::Manifest(&manifest)
    );
    let block = vec![0x6b; MAX_BLOCK_BYTES_V2];
    let frame = encode_frame_v2(FrameRefV2::Block {
        height: 1,
        bytes: &block,
    })
    .unwrap();
    assert_eq!(frame.len(), MAX_BLOCK_BYTES_V2 + 20);
    assert_eq!(
        decode_frame_v2(&frame).unwrap(),
        FrameRefV2::Block {
            height: 1,
            bytes: &block
        }
    );
}

#[test]
fn framing_success_is_not_manifest_block_or_network_acceptance() {
    // Deliberately not MNM2/MRB2, and not any independently selected identity.
    let manifest = [0; 46];
    let block = [0; 168];
    for frame in [
        FrameRefV2::Manifest(&manifest),
        FrameRefV2::Block {
            height: 0,
            bytes: &block,
        },
    ] {
        let encoded = encode_frame_v2(frame).unwrap();
        assert_eq!(decode_frame_v2(&encoded).unwrap(), frame);
    }
    let claim = HistorySummaryV2 {
        config_hash: Hash256::default(),
        genesis_hash: Hash256::default(),
        ..summary("genesis")
    };
    assert!(claim.encode().is_ok());
}

#[test]
fn v1_and_v2_remain_distinct_protocols() {
    let old = WireMessage::Hello(HelloMessage::new(
        1,
        Hash256::from_bytes([1; 32]),
        0,
        Hash256::from_bytes([2; 32]),
    ));
    let old_bytes = encode_message(&old).unwrap();
    assert_eq!(decode_message(&old_bytes).unwrap(), old);
    assert_eq!(&old_bytes[..6], b"MIRR\x01\x00");
    assert_eq!(
        decode_frame_v2(&old_bytes),
        Err(ProtocolErrorV2::InvalidMagic)
    );
    let new = bytes("genesis_begin");
    assert!(matches!(
        decode_message(&new),
        Err(ProtocolError::InvalidMagic)
    ));
    let mut old_with_new_version = old_bytes;
    old_with_new_version[4..6].copy_from_slice(&2u16.to_le_bytes());
    assert!(matches!(
        decode_message(&old_with_new_version),
        Err(ProtocolError::UnsupportedProtocolVersion {
            expected: 1,
            got: 2
        })
    ));
}
