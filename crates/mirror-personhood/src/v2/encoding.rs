//! MCF2 unsigned configuration encoding. Decoding is NOT seed authorization.
use mirror_crypto::{Hash256, PublicKey};
use super::{BootstrapDescriptor, ConfigError, ConfigParameters, ConfigV2, LimitsV2,
    TestAllocation, CONFIG_FIXED_LEN, MAX_CONFIG_BYTES};

const MAGIC: [u8; 4] = *b"MCF2";

pub(super) fn encode(config: &ConfigV2) -> Vec<u8> {
    // Validated counts are <=1000 seeds and <=65536 allocation entries.
    let mut bytes = Vec::with_capacity(CONFIG_FIXED_LEN
        + config.bootstrap.len() * 96 + config.allocations.len() * 40);
    let p = config.parameters;
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&p.format_version.to_le_bytes());
    bytes.extend_from_slice(&p.profile_version.to_le_bytes());
    bytes.extend_from_slice(p.network.as_bytes());
    bytes.extend_from_slice(&p.pow_bits.to_le_bytes());
    bytes.extend_from_slice(&p.genesis_timestamp.to_le_bytes());
    bytes.extend_from_slice(&p.required_approvals.to_le_bytes());
    bytes.extend_from_slice(p.evidence_profile.as_bytes());
    for value in config.limits.u32_fields() { bytes.extend_from_slice(&value.to_le_bytes()); }
    bytes.extend_from_slice(&config.limits.max_anchor_age.to_le_bytes());
    bytes.extend_from_slice(&config.limits.max_validity_blocks.to_le_bytes());
    bytes.extend_from_slice(&(config.bootstrap.len() as u32).to_le_bytes());
    for seed in &config.bootstrap { bytes.extend_from_slice(&seed.encode()); }
    bytes.extend_from_slice(&(config.allocations.len() as u32).to_le_bytes());
    for allocation in &config.allocations {
        bytes.extend_from_slice(&allocation.address);
        bytes.extend_from_slice(&allocation.balance_nusa.to_le_bytes());
    }
    bytes
}

pub(super) fn decode(bytes: &[u8]) -> Result<ConfigV2, ConfigError> {
    if bytes.len() > MAX_CONFIG_BYTES { return Err(ConfigError::TooLarge); }
    let mut input = Input { bytes, offset: 0 };
    if input.array::<4>()? != MAGIC { return Err(ConfigError::InvalidMagic); }
    let parameters = ConfigParameters {
        format_version: input.u16()?, profile_version: input.u16()?,
        network: input.hash()?, pow_bits: input.u32()?, genesis_timestamp: input.u64()?,
        required_approvals: input.u16()?, evidence_profile: input.hash()?,
    };
    parameters.validate()?;
    let limits = LimitsV2 {
        max_block_bytes: input.u32()?, max_operations: input.u32()?,
        max_membership_operations: input.u32()?, max_new_enrollments: input.u32()?,
        max_operation_bytes: input.u32()?, max_pending: input.u32()?,
        max_records: input.u32()?, max_accounts: input.u32()?,
        max_bootstrap: input.u32()?, max_log_bytes: input.u32()?,
        max_anchor_age: input.u64()?, max_validity_blocks: input.u64()?,
    };
    limits.validate()?;
    let seed_count = input.u32()?;
    if seed_count == 0 || seed_count > limits.max_bootstrap
        || seed_count < u32::from(parameters.required_approvals) {
        return Err(ConfigError::InvalidBootstrapCount);
    }
    // Check actual bytes before reserving attacker-declared capacity.
    if input.remaining() < seed_count as usize * 96 + 4 { return Err(ConfigError::Truncated); }
    let mut bootstrap = Vec::with_capacity(seed_count as usize);
    for _ in 0..seed_count {
        bootstrap.push(BootstrapDescriptor {
            public_key: PublicKey::from_bytes(input.array()?),
            nonce: input.hash()?, evidence: input.hash()?,
        });
    }
    let account_count = input.u32()?;
    if account_count > limits.max_accounts { return Err(ConfigError::TooManyAllocations); }
    let account_bytes = account_count as usize * 40;
    if input.remaining() < account_bytes { return Err(ConfigError::Truncated); }
    if input.remaining() > account_bytes { return Err(ConfigError::TrailingBytes); }
    let mut allocations = Vec::with_capacity(account_count as usize);
    for _ in 0..account_count {
        allocations.push(TestAllocation { address: input.array()?, balance_nusa: input.u64()? });
    }
    ConfigV2::new(parameters, limits, &bootstrap, &allocations)
}

struct Input<'a> { bytes: &'a [u8], offset: usize }
impl Input<'_> {
    fn remaining(&self) -> usize { self.bytes.len() - self.offset }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], ConfigError> {
        let end = self.offset.checked_add(N).ok_or(ConfigError::TooLarge)?;
        let slice = self.bytes.get(self.offset..end).ok_or(ConfigError::Truncated)?;
        let value = slice.try_into().map_err(|_| ConfigError::Truncated)?;
        self.offset = end;
        Ok(value)
    }
    fn u16(&mut self) -> Result<u16, ConfigError> { Ok(u16::from_le_bytes(self.array()?)) }
    fn u32(&mut self) -> Result<u32, ConfigError> { Ok(u32::from_le_bytes(self.array()?)) }
    fn u64(&mut self) -> Result<u64, ConfigError> { Ok(u64::from_le_bytes(self.array()?)) }
    fn hash(&mut self) -> Result<Hash256, ConfigError> { Ok(Hash256::from_bytes(self.array()?)) }
}
