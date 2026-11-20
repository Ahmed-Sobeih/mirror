//! Synthetic atomic state execution. No real blocks, people, biometrics or money.
use mirror_core::{
    Address,
    v2::{SignedOperationV2, SignedTransferV2, TransferBodyV2},
};
use mirror_crypto::{Hash256, Keypair, sha256d};
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, ConfigV2, LimitsV2, MembershipActionV2,
    MembershipStatusV2, OperationHeaderV2, ParentChainViewV2, SignedMembershipOperationV2,
    TestAllocation,
};
use mirror_state::v2::{LedgerErrorV2, LedgerOperationErrorV2, LedgerV2};

struct SyntheticParent {
    context: Hash256,
    hash: Hash256,
}

impl ParentChainViewV2 for SyntheticParent {
    fn config_hash(&self) -> Hash256 {
        self.context
    }
    fn parent_height(&self) -> u64 {
        0
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height == 0).then_some(self.hash)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let alice = Keypair::from_secret_bytes([1; 32]);
    let bob = Keypair::from_secret_bytes([2; 32]);
    let applicant = Keypair::from_secret_bytes([3; 32]);
    let address = |key: &Keypair| Address::from_public_key(&key.public_key());
    let evidence_profile = sha256d(b"Mirror ledger demo: synthetic evidence profile");
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
            network: sha256d(b"Mirror atomic ledger demonstration"),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 2,
            evidence_profile,
        },
        LimitsV2::default(),
        &[
            BootstrapDescriptor {
                public_key: alice.public_key(),
                nonce: sha256d(b"demo Alice seed nonce"),
                evidence: sha256d(b"demo Alice synthetic evidence"),
            },
            BootstrapDescriptor {
                public_key: bob.public_key(),
                nonce: sha256d(b"demo Bob seed nonce"),
                evidence: sha256d(b"demo Bob synthetic evidence"),
            },
        ],
        &allocations,
    )?;
    let bootstrap =
        config.verify_bootstrap(&[config.sign_seed(0, &alice)?, config.sign_seed(1, &bob)?])?;
    let parent = SyntheticParent {
        context: config.context(),
        hash: sha256d(b"synthetic parent tag; not an accepted Mirror block"),
    };
    // This explicit synthetic provenance is solely for this local demonstration.
    let ledger = LedgerV2::bootstrap(&bootstrap)?.bind_to_validated_block(parent.hash);
    let original = ledger.clone();
    let header = |key: &Keypair, sequence| OperationHeaderV2 {
        config_hash: config.context(),
        anchor_height: 0,
        anchor_hash: parent.hash,
        valid_from: 1,
        valid_until: 8,
        actor_public_key: key.public_key(),
        actor_sequence: sequence,
    };
    let transfer = |key: &Keypair, recipient: Address, amount, sequence| {
        SignedTransferV2::sign(
            header(key, sequence),
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
        header(&applicant, 0),
        MembershipActionV2::Enroll {
            applicant_nonce: sha256d(b"demo applicant nonce"),
            evidence_commitment: sha256d(b"demo applicant synthetic evidence"),
            evidence_profile,
        },
        &applicant,
    )?;
    let target = enrollment.enrollment_id().expect("Enroll has an ID");
    let approve = |key: &Keypair, sequence| {
        SignedMembershipOperationV2::sign(
            header(key, sequence),
            MembershipActionV2::Approve {
                target_enrollment_id: target,
            },
            key,
        )
        .map(SignedOperationV2::from)
    };
    let operations = vec![
        transfer(&alice, address(&bob), 100, 0)?,
        transfer(&bob, address(&applicant), 40, 0)?,
        enrollment.into(),
        approve(&alice, 0)?,
        approve(&bob, 0)?,
    ];

    let mut bad_transfer = operations.clone();
    bad_transfer.push(transfer(&alice, address(&bob), 10_000, 1)?);
    assert!(matches!(
        ledger.stage_operations(&parent, &bad_transfer),
        Err(LedgerErrorV2::Operation {
            index: 5,
            error: LedgerOperationErrorV2::InsufficientBalance { .. },
        })
    ));
    assert_eq!(ledger, original);

    let mut bad_membership = operations.clone();
    bad_membership.push(approve(&alice, 1)?); // A second review by the same actor.
    assert!(matches!(
        ledger.stage_operations(&parent, &bad_membership),
        Err(LedgerErrorV2::Operation {
            index: 5,
            error: LedgerOperationErrorV2::Membership(_),
        })
    ));
    assert_eq!(ledger, original);

    let staged = ledger.stage_operations(&parent, &operations)?;
    let member = staged
        .membership()
        .record(target)
        .expect("staged enrollment");
    assert_eq!(member.status(), MembershipStatusV2::Active);
    assert_eq!(member.inclusion().operation_index, 2);
    assert_eq!(staged.account(address(&alice)).balance(), 900);
    assert_eq!(staged.account(address(&bob)).balance(), 60);
    assert_eq!(staged.account(address(&applicant)).balance(), 40);
    assert_eq!(
        staged
            .accounts()
            .map(|(_, account)| account.balance())
            .sum::<u64>(),
        1_000
    );
    assert_eq!(ledger, original);

    println!("Mirror V2 atomic ledger — synthetic execution only");
    println!("Candidate height: {}", staged.height());
    println!("Original positions: 0 Transfer, 1 Transfer, 2 Enroll, 3 Approve, 4 Approve");
    println!("Applicant: Active after both existing reviewers approve");
    println!("Test Nusa: Alice 900, Bob 60, applicant 40; total 1000 unchanged");
    println!("Transfer and membership sequence spaces: separate");
    println!("Late transfer failure: both components unchanged");
    println!("Late membership failure: both components unchanged");
    println!("Accounts root: {}", staged.accounts_root()?);
    println!("Membership root: {}", staged.membership().root()?);
    println!("Economics root: {}", staged.economics_root());
    println!("Combined ledger root: {}", staged.root()?);
    println!("No actual block, canonical-chain authority, reward or real enrollment accepted.");
    Ok(())
}
