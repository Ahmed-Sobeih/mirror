//! Deliberately private fault injection. No public arbitrary-state constructor
//! or snapshot-to-ledger bypass is added for these unreachable boundary states.

use super::*;
use mirror_core::v2::TransferBodyV2;
use mirror_crypto::Keypair;
use mirror_personhood::v2::{
    BootstrapDescriptor, ConfigParameters, LimitsV2, OperationHeaderV2, TestAllocation,
};

fn key(number: u8) -> Keypair {
    Keypair::from_secret_bytes([number; 32])
}

fn address(number: u8) -> Address {
    Address::from_public_key(&key(number).public_key())
}

fn ledger() -> LedgerV2 {
    let config = ConfigV2::new(
        ConfigParameters {
            format_version: 2,
            profile_version: 1,
            network: Hash256::from_bytes([1; 32]),
            pow_bits: 0x207f_ffff,
            genesis_timestamp: 0,
            required_approvals: 1,
            evidence_profile: Hash256::from_bytes([2; 32]),
        },
        LimitsV2 {
            max_accounts: 2,
            ..LimitsV2::default()
        },
        &[BootstrapDescriptor {
            public_key: key(1).public_key(),
            nonce: Hash256::from_bytes([3; 32]),
            evidence: Hash256::from_bytes([4; 32]),
        }],
        &[TestAllocation {
            address: *address(1).as_bytes(),
            balance_nusa: 100,
        }],
    )
    .unwrap();
    let proof = config.sign_seed(0, &key(1)).unwrap();
    LedgerV2::bootstrap(&config.verify_bootstrap(&[proof]).unwrap())
        .unwrap()
        .bind_to_validated_block(Hash256::from_bytes([5; 32]))
}

struct View<'a>(&'a LedgerV2);
impl ParentChainViewV2 for View<'_> {
    fn config_hash(&self) -> Hash256 {
        self.0.config().context()
    }
    fn parent_height(&self) -> u64 {
        self.0.height()
    }
    fn canonical_hash_at(&self, height: u64) -> Option<Hash256> {
        (height == self.0.height()).then_some(self.0.tip_hash())
    }
}

fn transfer(ledger: &LedgerV2, sequence: u64) -> SignedTransferV2 {
    SignedTransferV2::sign(
        OperationHeaderV2 {
            config_hash: ledger.config().context(),
            anchor_height: ledger.height(),
            anchor_hash: ledger.tip_hash(),
            valid_from: ledger.height() + 1,
            valid_until: ledger.height() + 1,
            actor_public_key: key(1).public_key(),
            actor_sequence: sequence,
        },
        TransferBodyV2 {
            recipient: address(2),
            amount: 1,
            fee: 0,
        },
        &key(1),
    )
    .unwrap()
}

#[test]
fn exhausted_account_sequence_fails_without_any_mutation() {
    let mut ledger = ledger();
    ledger
        .accounts
        .accounts
        .get_mut(&address(1))
        .unwrap()
        .next_sequence = u64::MAX;
    let before = ledger.clone();
    let operation = transfer(&ledger, u64::MAX);
    assert_eq!(
        ledger.stage_operations(&View(&ledger), &[operation.into()]),
        Err(LedgerErrorV2::Operation {
            index: 0,
            error: LedgerOperationErrorV2::AccountSequenceOverflow
        })
    );
    assert_eq!(ledger, before);
    assert_eq!(ledger.root().unwrap(), before.root().unwrap());
}

#[test]
fn recipient_credit_overflow_is_checked_before_sender_mutation() {
    let mut ledger = ledger();
    ledger.accounts.accounts.insert(
        address(2),
        AccountV2 {
            balance: u64::MAX,
            next_sequence: 0,
        },
    );
    let before = ledger.accounts.clone();
    let config = ledger.config().clone();
    let operation = transfer(&ledger, 0);
    assert_eq!(
        ledger.accounts.apply_transfer(&config, &operation),
        Err(LedgerOperationErrorV2::RecipientBalanceOverflow)
    );
    assert_eq!(ledger.accounts, before);
}

#[test]
fn account_component_rejects_corrupt_balance_sum_and_supply() {
    let mut ledger = ledger();
    ledger.accounts.accounts.insert(
        address(2),
        AccountV2 {
            balance: u64::MAX,
            next_sequence: 0,
        },
    );
    assert_eq!(
        ledger.account_bytes(),
        Err(LedgerErrorV2::BalanceSumOverflow)
    );
    ledger.accounts.accounts.remove(&address(2));
    ledger
        .accounts
        .accounts
        .get_mut(&address(1))
        .unwrap()
        .balance = 99;
    assert_eq!(
        ledger.accounts_root(),
        Err(LedgerErrorV2::SupplyMismatch {
            expected: 100,
            got: 99
        })
    );
}

#[test]
fn excess_historical_accounts_are_rejected_before_encoding() {
    let mut ledger = ledger();
    ledger
        .accounts
        .accounts
        .insert(address(2), AccountV2::default());
    ledger
        .accounts
        .accounts
        .insert(address(3), AccountV2::default());
    assert_eq!(ledger.account_bytes(), Err(LedgerErrorV2::AccountLimit));
}

#[test]
fn final_account_invariant_failure_discards_completed_candidate() {
    let mut ledger = ledger();
    ledger.accounts.initial_test_supply = 101;
    let before = ledger.clone();
    let operation = transfer(&ledger, 0);
    assert_eq!(
        ledger.stage_operations(&View(&ledger), &[operation.into()]),
        Err(LedgerErrorV2::SupplyMismatch {
            expected: 100,
            got: 101
        })
    );
    assert_eq!(ledger, before);
    assert_eq!(ledger.account(address(1)).balance(), 100);
    assert_eq!(ledger.account(address(1)).next_sequence(), 0);
    assert_eq!(ledger.height(), 0);
}
