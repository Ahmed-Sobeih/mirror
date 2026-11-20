//! MB-02B: atomic execution of one ordered V2 accounts/membership transition.
//!
//! This synthetic profile moves only explicitly configured test balances. It
//! does not mint currency, pay rewards, charge fees or award account privileges
//! to members. A staged ledger is not an accepted block: the trusted integration
//! layer still has to validate the actual header, genesis, work and claimed roots.

use std::{collections::BTreeMap, fmt};

use mirror_core::{
    Address,
    v2::{
        BlockErrorV2, SignedOperationV2, SignedTransferV2, TransferErrorV2,
        validate_operation_limits_v2,
    },
};
use mirror_crypto::{Hash256, sha256d};
use mirror_personhood::v2::{
    ConfigV2, MembershipRegistryV2, MembershipStateV2, ParentChainViewV2, StagedMembershipStateV2,
    StateCodecErrorV2, StateErrorV2, VerifiedBootstrap,
};

const ACCOUNTS_DOMAIN: &[u8] = b"MIRROR_ACCOUNTS_V2";
const ECONOMICS_DOMAIN: &[u8] = b"MIRROR_ECONOMICS_DISABLED_V2";
const LEDGER_DOMAIN: &[u8] = b"MIRROR_LEDGER_STATE_V2";
const ENCODED_ACCOUNT_LEN: usize = 32 + 8 + 8;

/// An immutable view of a V2 account. An account sequence is independent of
/// the same key's membership sequence. All amounts are integer Nusa.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccountV2 {
    balance: u64,
    next_sequence: u64,
}

impl AccountV2 {
    pub const fn balance(&self) -> u64 {
        self.balance
    }

    pub const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AccountsStateV2 {
    accounts: BTreeMap<Address, AccountV2>,
    initial_test_supply: u64,
}

impl AccountsStateV2 {
    fn from_config(config: &ConfigV2) -> Result<Self, LedgerErrorV2> {
        if config.allocations().len() > config.limits().max_accounts as usize {
            return Err(LedgerErrorV2::AccountLimit);
        }
        let mut accounts = BTreeMap::new();
        for allocation in config.allocations() {
            // Explicit zero allocations are records too. No seed, founder or
            // operator account receives an implicit balance or account entry.
            accounts.insert(
                Address::from_bytes(allocation.address),
                AccountV2 {
                    balance: allocation.balance_nusa,
                    next_sequence: 0,
                },
            );
        }
        let state = Self {
            accounts,
            initial_test_supply: config.initial_test_supply(),
        };
        state.check_invariants(config)?;
        Ok(state)
    }

    fn account(&self, address: Address) -> AccountV2 {
        self.accounts.get(&address).copied().unwrap_or_default()
    }

    fn check_invariants(&self, config: &ConfigV2) -> Result<(), LedgerErrorV2> {
        if self.accounts.len() > config.limits().max_accounts as usize {
            return Err(LedgerErrorV2::AccountLimit);
        }
        if self.initial_test_supply != config.initial_test_supply() {
            return Err(LedgerErrorV2::SupplyMismatch {
                expected: config.initial_test_supply(),
                got: self.initial_test_supply,
            });
        }
        let total = self.accounts.values().try_fold(0u64, |total, account| {
            total
                .checked_add(account.balance)
                .ok_or(LedgerErrorV2::BalanceSumOverflow)
        })?;
        if total != self.initial_test_supply {
            return Err(LedgerErrorV2::SupplyMismatch {
                expected: self.initial_test_supply,
                got: total,
            });
        }
        Ok(())
    }

    /// Only called after contextual authorization and signature verification.
    /// Check every fallible calculation before either account is changed.
    fn apply_transfer(
        &mut self,
        config: &ConfigV2,
        transfer: &SignedTransferV2,
    ) -> Result<(), LedgerOperationErrorV2> {
        let sender = transfer.sender();
        let body = transfer.body();
        let account = self.account(sender);
        if transfer.header().actor_sequence != account.next_sequence {
            return Err(LedgerOperationErrorV2::AccountSequenceMismatch {
                expected: account.next_sequence,
                got: transfer.header().actor_sequence,
            });
        }
        let next_sequence = account
            .next_sequence
            .checked_add(1)
            .ok_or(LedgerOperationErrorV2::AccountSequenceOverflow)?;
        let remaining = account.balance.checked_sub(body.amount).ok_or(
            LedgerOperationErrorV2::InsufficientBalance {
                available: account.balance,
                required: body.amount,
            },
        )?;

        if sender == body.recipient {
            // A funded positive self-transfer consumes one account sequence.
            // It does not create a second credit or transiently overflow supply.
            self.accounts.insert(
                sender,
                AccountV2 {
                    balance: account.balance,
                    next_sequence,
                },
            );
            return Ok(());
        }

        let recipient = self.account(body.recipient);
        let credited = recipient
            .balance
            .checked_add(body.amount)
            .ok_or(LedgerOperationErrorV2::RecipientBalanceOverflow)?;
        if !self.accounts.contains_key(&body.recipient)
            && self.accounts.len() >= config.limits().max_accounts as usize
        {
            return Err(LedgerOperationErrorV2::AccountLimit);
        }
        self.accounts.insert(
            sender,
            AccountV2 {
                balance: remaining,
                next_sequence,
            },
        );
        self.accounts.insert(
            body.recipient,
            AccountV2 {
                balance: credited,
                next_sequence: recipient.next_sequence,
            },
        );
        // Zero-balance accounts are retained so their consumed sequences cannot
        // disappear when funds later return to the same address.
        Ok(())
    }

    fn account_bytes(&self, config: &ConfigV2) -> Result<Vec<u8>, LedgerErrorV2> {
        self.check_invariants(config)?;
        let count =
            u32::try_from(self.accounts.len()).map_err(|_| LedgerErrorV2::LengthOverflow)?;
        let length = self
            .accounts
            .len()
            .checked_mul(ENCODED_ACCOUNT_LEN)
            .and_then(|length| length.checked_add(4))
            .ok_or(LedgerErrorV2::LengthOverflow)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| LedgerErrorV2::AllocationFailed)?;
        bytes.extend_from_slice(&count.to_le_bytes());
        for (address, account) in &self.accounts {
            bytes.extend_from_slice(address.as_bytes());
            bytes.extend_from_slice(&account.balance.to_le_bytes());
            bytes.extend_from_slice(&account.next_sequence.to_le_bytes());
        }
        Ok(bytes)
    }

    fn accounts_root(&self, config: &ConfigV2) -> Result<Hash256, LedgerErrorV2> {
        let bytes = self.account_bytes(config)?;
        domain_hash(ACCOUNTS_DOMAIN, &bytes)
    }

    fn economics_root(&self) -> Hash256 {
        let mut bytes = [0; ECONOMICS_DOMAIN.len() + 8];
        bytes[..ECONOMICS_DOMAIN.len()].copy_from_slice(ECONOMICS_DOMAIN);
        bytes[ECONOMICS_DOMAIN.len()..].copy_from_slice(&self.initial_test_supply.to_le_bytes());
        sha256d(&bytes)
    }
}

/// A complete ledger associated by the trusted caller with a validated block.
/// Every exposed state view is read-only. The tip is external provenance and is
/// deliberately excluded from state roots to avoid a block-hash/root cycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerV2 {
    accounts: AccountsStateV2,
    membership: MembershipRegistryV2,
}

impl LedgerV2 {
    /// Stage configuration's explicit synthetic allocations and proof-checked
    /// membership seeds. This creates neither mining rewards nor founder funds.
    pub fn bootstrap(bootstrap: &VerifiedBootstrap) -> Result<StagedLedgerV2, LedgerErrorV2> {
        let accounts = AccountsStateV2::from_config(bootstrap.config())?;
        let membership =
            MembershipRegistryV2::bootstrap(bootstrap).map_err(LedgerErrorV2::Membership)?;
        Ok(StagedLedgerV2 {
            accounts,
            membership,
        })
    }

    /// Execute one bounded block body atomically against this immutable parent.
    ///
    /// Budget failures precede input-sized copies and authorization work. Parent
    /// binding precedes the account clone. Expiry runs before the first operation;
    /// transfer and membership checks then run in the original full-body order.
    /// Membership activation and all final invariants run after that loop. Any
    /// failure discards both candidate states, including earlier valid operations.
    pub fn stage_operations<V: ParentChainViewV2 + ?Sized>(
        &self,
        parent: &V,
        operations: &[SignedOperationV2],
    ) -> Result<StagedLedgerV2, LedgerErrorV2> {
        validate_operation_limits_v2(self.config(), operations).map_err(LedgerErrorV2::Limits)?;
        let count = u32::try_from(operations.len())
            .map_err(|_| LedgerErrorV2::Limits(BlockErrorV2::TooManyOperations))?;
        let membership_operations: Vec<_> = operations
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| match operation {
                SignedOperationV2::Transfer(_) => None,
                SignedOperationV2::Membership(operation) => Some((index as u32, *operation)),
            })
            .collect();
        let mut membership = self
            .membership
            .begin_indexed_block(parent, count, &membership_operations)
            .map_err(LedgerErrorV2::Membership)?;
        let mut accounts = self.accounts.clone();

        for (index, operation) in operations.iter().enumerate() {
            let index = index as u32; // Full-body preflight established count <= u32::MAX.
            match operation {
                SignedOperationV2::Transfer(transfer) => {
                    transfer
                        .validate_for_parent(self.config(), parent)
                        .map_err(|error| LedgerErrorV2::Operation {
                            index,
                            error: LedgerOperationErrorV2::Transfer(error),
                        })?;
                    accounts
                        .apply_transfer(self.config(), transfer)
                        .map_err(|error| LedgerErrorV2::Operation { index, error })?;
                }
                SignedOperationV2::Membership(_) => {
                    membership =
                        membership
                            .apply_next(index)
                            .map_err(|error| LedgerErrorV2::Operation {
                                index,
                                error: LedgerOperationErrorV2::Membership(error),
                            })?;
                }
            }
        }
        let membership = membership.finish().map_err(LedgerErrorV2::Membership)?;
        accounts.check_invariants(self.config())?;
        Ok(StagedLedgerV2 {
            accounts,
            membership,
        })
    }

    pub fn account(&self, address: Address) -> AccountV2 {
        self.accounts.account(address)
    }

    pub fn contains_account(&self, address: Address) -> bool {
        self.accounts.accounts.contains_key(&address)
    }

    pub fn account_count(&self) -> usize {
        self.accounts.accounts.len()
    }

    /// Canonical raw-address order, including explicit and retained zero records.
    pub fn accounts(&self) -> impl Iterator<Item = (&Address, &AccountV2)> {
        self.accounts.accounts.iter()
    }

    pub const fn membership(&self) -> &MembershipStateV2 {
        self.membership.state()
    }

    pub const fn config(&self) -> &ConfigV2 {
        self.membership().config()
    }

    pub const fn height(&self) -> u64 {
        self.membership().height()
    }

    pub const fn tip_hash(&self) -> Hash256 {
        self.membership.tip_hash()
    }

    pub const fn initial_test_supply(&self) -> u64 {
        self.accounts.initial_test_supply
    }

    pub fn account_bytes(&self) -> Result<Vec<u8>, LedgerErrorV2> {
        self.accounts.account_bytes(self.config())
    }

    pub fn accounts_root(&self) -> Result<Hash256, LedgerErrorV2> {
        self.accounts.accounts_root(self.config())
    }

    pub fn economics_root(&self) -> Hash256 {
        self.accounts.economics_root()
    }

    pub fn root(&self) -> Result<Hash256, LedgerErrorV2> {
        combined_root(&self.accounts, self.membership())
    }
}

/// A completed atomic transition, with no permission to become the next parent
/// until a trusted integration caller validates and binds the actual block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedLedgerV2 {
    accounts: AccountsStateV2,
    membership: StagedMembershipStateV2,
}

impl StagedLedgerV2 {
    pub fn account(&self, address: Address) -> AccountV2 {
        self.accounts.account(address)
    }

    pub fn contains_account(&self, address: Address) -> bool {
        self.accounts.accounts.contains_key(&address)
    }

    pub fn account_count(&self) -> usize {
        self.accounts.accounts.len()
    }

    pub fn accounts(&self) -> impl Iterator<Item = (&Address, &AccountV2)> {
        self.accounts.accounts.iter()
    }

    pub const fn membership(&self) -> &MembershipStateV2 {
        self.membership.state()
    }

    pub const fn config(&self) -> &ConfigV2 {
        self.membership().config()
    }

    pub const fn height(&self) -> u64 {
        self.membership().height()
    }

    pub const fn initial_test_supply(&self) -> u64 {
        self.accounts.initial_test_supply
    }

    pub fn account_bytes(&self) -> Result<Vec<u8>, LedgerErrorV2> {
        self.accounts.account_bytes(self.config())
    }

    pub fn accounts_root(&self) -> Result<Hash256, LedgerErrorV2> {
        self.accounts.accounts_root(self.config())
    }

    pub fn economics_root(&self) -> Hash256 {
        self.accounts.economics_root()
    }

    pub fn root(&self) -> Result<Hash256, LedgerErrorV2> {
        combined_root(&self.accounts, self.membership())
    }

    /// TRUSTED CALLER BOUNDARY. This does not validate a block hash, work, header,
    /// genesis, operations root, claimed ledger root, chain choice or finality.
    /// The caller must establish those before publishing this ledger as accepted.
    pub fn bind_to_validated_block(self, block_hash: Hash256) -> LedgerV2 {
        LedgerV2 {
            accounts: self.accounts,
            membership: self.membership.bind_to_validated_block(block_hash),
        }
    }
}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> Result<Hash256, LedgerErrorV2> {
    let length = domain
        .len()
        .checked_add(bytes.len())
        .ok_or(LedgerErrorV2::LengthOverflow)?;
    let mut preimage = Vec::new();
    preimage
        .try_reserve_exact(length)
        .map_err(|_| LedgerErrorV2::AllocationFailed)?;
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(bytes);
    Ok(sha256d(&preimage))
}

fn combined_root(
    accounts: &AccountsStateV2,
    membership: &MembershipStateV2,
) -> Result<Hash256, LedgerErrorV2> {
    let accounts_root = accounts.accounts_root(membership.config())?;
    let membership_root = membership.root().map_err(LedgerErrorV2::MembershipCodec)?;
    let economics_root = accounts.economics_root();
    let mut bytes = [0; LEDGER_DOMAIN.len() + 32 + 8 + 32 + 32 + 32];
    let mut position = LEDGER_DOMAIN.len();
    bytes[..position].copy_from_slice(LEDGER_DOMAIN);
    bytes[position..position + 32].copy_from_slice(membership.config().context().as_bytes());
    position += 32;
    bytes[position..position + 8].copy_from_slice(&membership.height().to_le_bytes());
    position += 8;
    for root in [accounts_root, membership_root, economics_root] {
        bytes[position..position + 32].copy_from_slice(root.as_bytes());
        position += 32;
    }
    Ok(sha256d(&bytes))
}

/// Failures inside one operation. The enclosing LedgerErrorV2 always records
/// its original full-block index, including intervening transfer operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerOperationErrorV2 {
    Transfer(TransferErrorV2),
    Membership(StateErrorV2),
    AccountSequenceMismatch { expected: u64, got: u64 },
    AccountSequenceOverflow,
    InsufficientBalance { available: u64, required: u64 },
    RecipientBalanceOverflow,
    AccountLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedgerErrorV2 {
    Limits(BlockErrorV2),
    Membership(StateErrorV2),
    Operation {
        index: u32,
        error: LedgerOperationErrorV2,
    },
    MembershipCodec(StateCodecErrorV2),
    AccountLimit,
    BalanceSumOverflow,
    SupplyMismatch {
        expected: u64,
        got: u64,
    },
    LengthOverflow,
    AllocationFailed,
}

impl fmt::Display for LedgerOperationErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 ledger operation rejected: {self:?}")
    }
}

impl std::error::Error for LedgerOperationErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transfer(error) => Some(error),
            Self::Membership(error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for LedgerErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 ledger transition rejected: {self:?}")
    }
}

impl std::error::Error for LedgerErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Limits(error) => Some(error),
            Self::Membership(error) => Some(error),
            Self::Operation { error, .. } => Some(error),
            Self::MembershipCodec(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
