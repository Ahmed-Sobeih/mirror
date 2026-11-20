//! Checked small-network resource configuration. Not a cap on eventual humanity.
use super::ConfigError;

/// These are ceilings for the synthetic V2 development profile. Changing a
/// configured value changes the signed configuration; raising a hard ceiling
/// requires an explicit profile/version review.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitsV2 {
    pub max_block_bytes: u32,
    pub max_operations: u32,
    pub max_membership_operations: u32,
    pub max_new_enrollments: u32,
    pub max_operation_bytes: u32,
    pub max_pending: u32,
    pub max_records: u32,
    pub max_accounts: u32,
    pub max_bootstrap: u32,
    pub max_log_bytes: u32,
    pub max_anchor_age: u64,
    pub max_validity_blocks: u64,
}

impl Default for LimitsV2 {
    fn default() -> Self {
        Self {
            max_block_bytes: 1_048_576,
            max_operations: 256,
            max_membership_operations: 256,
            max_new_enrollments: 64,
            max_operation_bytes: 1_024,
            max_pending: 1_024,
            max_records: 65_536,
            max_accounts: 65_536,
            max_bootstrap: 1_000,
            max_log_bytes: 4_194_304,
            max_anchor_age: 32,
            max_validity_blocks: 256,
        }
    }
}

impl LimitsV2 {
    pub fn validate(&self) -> Result<(), ConfigError> {
        let ceilings = Self::default();
        for (value, ceiling) in self.u32_fields().into_iter().zip(ceilings.u32_fields()) {
            if value == 0 || value > ceiling {
                return Err(ConfigError::InvalidLimits);
            }
        }
        if self.max_anchor_age == 0 || self.max_anchor_age > ceilings.max_anchor_age
            || self.max_validity_blocks == 0
            || self.max_validity_blocks > ceilings.max_validity_blocks
            || self.max_membership_operations > self.max_operations
            || self.max_new_enrollments > self.max_membership_operations
            || self.max_new_enrollments > self.max_pending
            || self.max_pending > self.max_records
            || self.max_bootstrap > self.max_records
            || self.max_operation_bytes < 291
        {
            return Err(ConfigError::InvalidLimits);
        }
        // At least one largest permitted operation must fit its containers.
        let block_min = 168u32.checked_add(4)
            .and_then(|n| n.checked_add(self.max_operation_bytes))
            .ok_or(ConfigError::InvalidLimits)?;
        let log_min = 8u32.checked_add(self.max_operation_bytes)
            .ok_or(ConfigError::InvalidLimits)?;
        if self.max_block_bytes < block_min || self.max_log_bytes < log_min {
            return Err(ConfigError::InvalidLimits);
        }
        Ok(())
    }

    // Wire ordering is frozen by docs/CONFIG_V2_ENCODING.md and independent vectors.
    pub(super) fn u32_fields(&self) -> [u32; 10] {
        [self.max_block_bytes, self.max_operations, self.max_membership_operations,
            self.max_new_enrollments, self.max_operation_bytes, self.max_pending,
            self.max_records, self.max_accounts, self.max_bootstrap, self.max_log_bytes]
    }
}
