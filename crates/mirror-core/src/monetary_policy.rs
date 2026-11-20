//! Agreed monetary-policy components, NOT an enabled minting mechanism.
//!
//! The bootstrap allocation's interaction with the population ceiling remains
//! undecided (additive allowance versus temporary floor). Therefore this module
//! deliberately provides NO combined supply ceiling and NO mint/claim API.
//! The historical 100-MRY Alice fixture belongs only to the existing devnet.

use crate::NUSA_PER_MRY;

pub const FOUNDER_EXTRA_ALLOCATION_NUSA: u64 = 0;
pub const MARKETING_PREALLOCATION_NUSA: u64 = 0;
pub const BOOTSTRAP_MEMBER_LIMIT: u64 = 1_000;
pub const BOOTSTRAP_GRANT_NUSA: u64 = 10 * NUSA_PER_MRY;
pub const BOOTSTRAP_TOTAL_ALLOCATION_NUSA: u64 = BOOTSTRAP_MEMBER_LIMIT * BOOTSTRAP_GRANT_NUSA;
pub const POPULATION_ALLOWANCE_NUSA_PER_HUMAN: u64 = NUSA_PER_MRY / 5;

/// Ordinary population-linked allowance, excluding the unresolved treatment of
/// the bootstrap exception. Input must be a validated historical population
/// maximum, not wallet count, online devices, cumulative signups or a peer claim.
pub fn population_allowance_nusa(population_high_water: u64) -> Result<u64, PolicyError> {
    population_high_water.checked_mul(POPULATION_ALLOWANCE_NUSA_PER_HUMAN)
        .ok_or(PolicyError::Overflow)
}

/// Policy amount for an eligible first-activation ordinal. No currency is
/// transferred or minted. Eligibility/replay-resistant claiming must be enforced
/// by a future ledger integration. Founder and other participants share this rule.
pub fn bootstrap_grant_nusa(first_activation_ordinal: u64) -> Result<u64, PolicyError> {
    match first_activation_ordinal {
        0 => Err(PolicyError::InvalidActivationOrdinal),
        1..=BOOTSTRAP_MEMBER_LIMIT => Ok(BOOTSTRAP_GRANT_NUSA),
        _ => Ok(0),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyError { Overflow, InvalidActivationOrdinal }
impl core::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "monetary policy calculation rejected: {self:?}")
    }
}
impl std::error::Error for PolicyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_allowance_is_exactly_one_fifth_mry_per_human() {
        assert_eq!(POPULATION_ALLOWANCE_NUSA_PER_HUMAN, 2_000_000);
        assert_eq!(population_allowance_nusa(0), Ok(0));
        assert_eq!(population_allowance_nusa(1_000_000), Ok(200_000 * NUSA_PER_MRY));
        assert_eq!(population_allowance_nusa(2_000_000), Ok(400_000 * NUSA_PER_MRY));
    }

    #[test]
    fn first_and_thousandth_members_have_identical_ten_mry_grants() {
        assert_eq!(bootstrap_grant_nusa(1), Ok(10 * NUSA_PER_MRY));
        assert_eq!(bootstrap_grant_nusa(1_000), bootstrap_grant_nusa(1));
        assert_eq!(bootstrap_grant_nusa(1_001), Ok(0));
        assert_eq!(bootstrap_grant_nusa(u64::MAX), Ok(0));
    }

    #[test]
    fn bootstrap_total_is_ten_thousand_mry_not_a_founder_premine() {
        let total: u64 = (1..=1_000).map(|i| bootstrap_grant_nusa(i).unwrap()).sum();
        assert_eq!(total, 10_000 * NUSA_PER_MRY);
        assert_eq!(total, BOOTSTRAP_TOTAL_ALLOCATION_NUSA);
        assert_eq!(FOUNDER_EXTRA_ALLOCATION_NUSA, 0);
        assert_eq!(MARKETING_PREALLOCATION_NUSA, 0);
    }

    #[test]
    fn zero_activation_ordinal_is_invalid() {
        assert_eq!(bootstrap_grant_nusa(0), Err(PolicyError::InvalidActivationOrdinal));
    }

    #[test]
    fn population_arithmetic_never_wraps() {
        let maximum = u64::MAX / POPULATION_ALLOWANCE_NUSA_PER_HUMAN;
        assert!(population_allowance_nusa(maximum).is_ok());
        assert_eq!(population_allowance_nusa(maximum + 1), Err(PolicyError::Overflow));
        assert_eq!(population_allowance_nusa(u64::MAX), Err(PolicyError::Overflow));
    }
}
