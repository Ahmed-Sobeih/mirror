//! MNM2 authority and bounded decoding, using public synthetic seed keys only.
use mirror_chain::v2::manifest::{MANIFEST_FIXED_LEN_V2, MAX_MANIFEST_BYTES_V2};
use mirror_chain::v2::{ManifestErrorV2 as Error, ManifestIdentityV2, NetworkManifestV2};
use mirror_crypto::{Hash256, Keypair, SignatureBytes};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigError, ConfigParameters, ConfigV2, LimitsV2, TestAllocation,
};

fn hash(value: u8) -> Hash256 {
    Hash256::from_bytes([value; 32])
}

fn config() -> ConfigV2 {
    let seeds: Vec<_> = (1..=2)
        .map(|n| BootstrapDescriptor {
            public_key: Keypair::from_secret_bytes([n; 32]).public_key(),
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
        &seeds,
        &[
            TestAllocation {
                address: [0x51; 32],
                balance_nusa: 0,
            },
            TestAllocation {
                address: [0x52; 32],
                balance_nusa: 1_000,
            },
        ],
    )
    .unwrap()
}

fn proofs(config: &ConfigV2) -> Vec<SignatureBytes> {
    (0..2)
        .map(|i| {
            config
                .sign_seed(i, &Keypair::from_secret_bytes([i as u8 + 1; 32]))
                .unwrap()
        })
        .collect()
}

fn fixture() -> NetworkManifestV2 {
    let config = config();
    // This is deliberately only a selected pin. These codec tests do not claim
    // this repeated-byte hash belongs to a valid genesis; ChainV2 must prove it.
    NetworkManifestV2::new(&config, &proofs(&config), hash(0xa0)).unwrap()
}

fn count_offset(manifest: &NetworkManifestV2) -> usize {
    10 + manifest.config().encode().len()
}

#[test]
fn manifest_roundtrip_preserves_exact_configuration_proofs_and_selected_pin() {
    let manifest = fixture();
    let bytes = manifest.encode().unwrap();
    assert_eq!(bytes.len(), MANIFEST_FIXED_LEN_V2 + 422 + 2 * 64);
    assert_eq!(&bytes[..4], b"MNM2");
    assert_eq!(&bytes[4..6], &2u16.to_le_bytes());
    assert_eq!(&bytes[6..10], &422u32.to_le_bytes());
    assert_eq!(&bytes[10..432], manifest.config().encode());
    assert_eq!(&bytes[432..436], &2u32.to_le_bytes());
    assert_eq!(&bytes[bytes.len() - 32..], hash(0xa0).as_bytes());
    let decoded = NetworkManifestV2::decode_for(&bytes, manifest.identity()).unwrap();
    assert_eq!(decoded, manifest);
    assert_eq!(decoded.encode().unwrap(), bytes);
    assert_eq!(decoded.bootstrap().config(), manifest.config());
    assert_eq!(decoded.bootstrap().seeds().len(), 2);
}

#[test]
fn caller_selected_configuration_cannot_be_replaced_by_manifest_bytes() {
    let manifest = fixture();
    let expected = ManifestIdentityV2 {
        config_hash: hash(0x99),
        ..manifest.identity()
    };
    assert_eq!(
        NetworkManifestV2::decode_for(&manifest.encode().unwrap(), expected),
        Err(Error::WrongConfiguration)
    );
}

#[test]
fn caller_selected_genesis_cannot_be_replaced_by_manifest_bytes() {
    let manifest = fixture();
    let expected = ManifestIdentityV2 {
        genesis_hash: hash(0x99),
        ..manifest.identity()
    };
    assert_eq!(
        NetworkManifestV2::decode_for(&manifest.encode().unwrap(), expected),
        Err(Error::WrongGenesis)
    );
}

#[test]
fn changed_stored_pin_rejects_before_invalid_proof_can_grant_authority() {
    let manifest = fixture();
    let mut bytes = manifest.encode().unwrap();
    bytes[count_offset(&manifest) + 4] ^= 1;
    *bytes.last_mut().unwrap() ^= 1;
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, manifest.identity()),
        Err(Error::WrongGenesis)
    );
}

#[test]
fn every_seed_proof_is_required_by_constructor_and_decoder() {
    let config = config();
    let valid = proofs(&config);
    let manifest = fixture();
    for index in 0..valid.len() {
        let mut invalid = valid.clone();
        invalid[index] = SignatureBytes::from_bytes([0; 64]);
        assert!(matches!(
            NetworkManifestV2::new(&config, &invalid, hash(0xa0)),
            Err(Error::Config(ConfigError::InvalidSeedProof { index: i, .. })) if i == index as u32
        ));
        let mut bytes = manifest.encode().unwrap();
        let start = count_offset(&manifest) + 4 + index * 64;
        bytes[start..start + 64].fill(0);
        assert!(matches!(
            NetworkManifestV2::decode_for(&bytes, manifest.identity()),
            Err(Error::Config(ConfigError::InvalidSeedProof { index: i, .. })) if i == index as u32
        ));
    }
}

#[test]
fn reordered_or_duplicate_seed_proofs_are_rejected() {
    let config = config();
    let mut p = proofs(&config);
    p.swap(0, 1);
    assert!(matches!(
        NetworkManifestV2::new(&config, &p, hash(0xa0)),
        Err(Error::Config(ConfigError::InvalidSeedProof { .. }))
    ));
    let p = proofs(&config);
    assert!(matches!(
        NetworkManifestV2::new(&config, &[p[0], p[0]], hash(0xa0)),
        Err(Error::Config(ConfigError::InvalidSeedProof {
            index: 1,
            ..
        }))
    ));
}

#[test]
fn missing_or_extra_proofs_reject_without_partial_bootstrap() {
    let config = config();
    let p = proofs(&config);
    for bad in [vec![], vec![p[0]], vec![p[0], p[1], p[0]]] {
        assert_eq!(
            NetworkManifestV2::new(&config, &bad, hash(0xa0)),
            Err(Error::WrongProofCount)
        );
    }
    let manifest = fixture();
    let offset = count_offset(&manifest);
    let mut bytes = manifest.encode().unwrap();
    bytes[offset..offset + 4].copy_from_slice(&1u32.to_le_bytes());
    bytes.drain(offset + 4 + 64..offset + 4 + 128);
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, manifest.identity()),
        Err(Error::WrongProofCount)
    );
}

#[test]
fn changed_configuration_cannot_reuse_old_seed_proofs_or_identity() {
    let manifest = fixture();
    let mut bytes = manifest.encode().unwrap();
    // Change network bytes inside otherwise canonical MCF2.
    bytes[10 + 8] ^= 1;
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, manifest.identity()),
        Err(Error::WrongConfiguration)
    );
    let config_len = manifest.config().encode().len();
    let changed = ConfigV2::decode(&bytes[10..10 + config_len]).unwrap();
    let expected = ManifestIdentityV2 {
        config_hash: changed.context(),
        ..manifest.identity()
    };
    assert!(matches!(
        NetworkManifestV2::decode_for(&bytes, expected),
        Err(Error::Config(ConfigError::InvalidSeedProof { .. }))
    ));
}

#[test]
fn all_truncations_and_trailing_bytes_reject() {
    let manifest = fixture();
    let bytes = manifest.encode().unwrap();
    for len in 0..bytes.len() {
        assert_eq!(
            NetworkManifestV2::decode_for(&bytes[..len], manifest.identity()),
            Err(Error::Truncated),
            "length {len}"
        );
    }
    let mut extra = bytes;
    extra.push(0);
    assert_eq!(
        NetworkManifestV2::decode_for(&extra, manifest.identity()),
        Err(Error::TrailingBytes)
    );
}

#[test]
fn unknown_magic_and_version_do_not_fallback() {
    let manifest = fixture();
    let mut bytes = manifest.encode().unwrap();
    for magic in [*b"MNM1", *b"MCF2", *b"MRB2"] {
        bytes[..4].copy_from_slice(&magic);
        assert_eq!(
            NetworkManifestV2::decode_for(&bytes, manifest.identity()),
            Err(Error::InvalidMagic)
        );
    }
    bytes[..4].copy_from_slice(b"MNM2");
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, manifest.identity()),
        Err(Error::UnsupportedVersion(1))
    );
}

#[test]
fn oversized_declared_configuration_rejects_before_allocation() {
    let manifest = fixture();
    let mut bytes = manifest.encode().unwrap();
    bytes[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, manifest.identity()),
        Err(Error::ConfigTooLarge)
    );
}

#[test]
fn oversized_declared_proof_count_rejects_before_signature_work() {
    let manifest = fixture();
    let offset = count_offset(&manifest);
    let mut bytes = manifest.encode().unwrap();
    for count in [1_001, u32::MAX] {
        bytes[offset..offset + 4].copy_from_slice(&count.to_le_bytes());
        assert_eq!(
            NetworkManifestV2::decode_for(&bytes, manifest.identity()),
            Err(Error::TooManyProofs)
        );
    }
}

#[test]
fn total_manifest_limit_is_checked_before_magic_or_configuration() {
    let manifest = fixture();
    let oversized = vec![0; MAX_MANIFEST_BYTES_V2 + 1];
    assert_eq!(
        NetworkManifestV2::decode_for(&oversized, manifest.identity()),
        Err(Error::TooLarge)
    );
}

#[test]
fn inner_configuration_errors_are_not_hidden_by_outer_framing() {
    let manifest = fixture();
    let mut bytes = manifest.encode().unwrap();
    bytes[10..14].copy_from_slice(b"MCF1");
    assert_eq!(
        NetworkManifestV2::decode_for(&bytes, manifest.identity()),
        Err(Error::Config(ConfigError::InvalidMagic))
    );
}
