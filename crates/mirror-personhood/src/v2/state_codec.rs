//! MPS2 canonical state bytes and an explicitly untrusted snapshot cache.
//!
//! A coherent cache, even with valid retained signatures, is not canonical
//! history. Decoding never yields a state/registry usable for execution. Compare
//! it against a state reconstructed by trusted replay before treating it as a
//! matching cache. Actual ledger/root provenance remains an outer-chain duty.
use super::*;
use mirror_crypto::sha256d;

const MAGIC: [u8; 4] = *b"MPS2";
const VERSION: u16 = 2;
const PREFIX_LEN: usize = 90;
const MIN_RECORD_LEN: usize = 159;
const MAX_RECORD_BASE_LEN: usize = 366;
const APPROVAL_LEN: usize = 267;
const ROOT_DOMAIN: &[u8] = b"MIRROR_MEMBERSHIP_STATE_V2";

impl MembershipStateV2 {
    /// Conservative checked MPS2 byte bound, distinct from the MPH2 log bound.
    pub fn max_encoded_len(config: &ConfigV2) -> Result<usize, StateCodecErrorV2> {
        (config.limits().max_records as usize)
            .checked_mul(MAX_RECORD_BASE_LEN + APPROVAL_LEN)
            .and_then(|n| n.checked_add(PREFIX_LEN))
            .ok_or(StateCodecErrorV2::TooLarge)
    }

    /// Canonical membership bytes. Derived indexes and tip provenance are omitted.
    pub fn encode(&self) -> Result<Vec<u8>, StateCodecErrorV2> {
        let len = encoded_len(self)?;
        let mut bytes = allocated_bytes(len)?;
        append_state(self, &mut bytes);
        debug_assert_eq!(bytes.len(), len);
        Ok(bytes)
    }

    /// Double SHA-256 over the exact domain (no NUL) followed by MPS2 bytes.
    pub fn root(&self) -> Result<Hash256, StateCodecErrorV2> {
        let len = encoded_len(self)?;
        let capacity = ROOT_DOMAIN
            .len()
            .checked_add(len)
            .ok_or(StateCodecErrorV2::TooLarge)?;
        let mut bytes = allocated_bytes(capacity)?;
        bytes.extend_from_slice(ROOT_DOMAIN);
        append_state(self, &mut bytes);
        Ok(sha256d(&bytes))
    }
}

/// A structurally checked, signature-checked cache that has NO runtime authority.
/// It deliberately exposes no state/registry conversion or mutable accessor.
///
/// ```compile_fail,E0616
/// use mirror_personhood::v2::state::UntrustedMembershipSnapshotV2;
/// fn cannot_extract_runtime_state(snapshot: UntrustedMembershipSnapshotV2) {
///     let _state = snapshot.state;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UntrustedMembershipSnapshotV2 {
    state: MembershipStateV2,
}

impl UntrustedMembershipSnapshotV2 {
    /// Bounds and exact canonical ordering are checked before signature work.
    /// The caller supplies pinned configuration, never a config from this cache.
    /// This cannot establish old withdrawal signatures, history, tip or freshness.
    pub fn decode(config: &ConfigV2, bytes: &[u8]) -> Result<Self, StateCodecErrorV2> {
        if bytes.len() > MembershipStateV2::max_encoded_len(config)? {
            return Err(StateCodecErrorV2::TooLarge);
        }
        let mut input = Reader { bytes, position: 0 };
        if input.array::<4>()? != MAGIC {
            return Err(StateCodecErrorV2::InvalidMagic);
        }
        if input.u16()? != VERSION {
            return Err(StateCodecErrorV2::UnsupportedVersion);
        }
        if input.hash()? != config.context() {
            return Err(StateCodecErrorV2::WrongContext);
        }
        let height = input.u64()?;
        let active_count = input.u64()?;
        let pending_count = input.u64()?;
        let active_high_water = input.u64()?;
        let total_activations = input.u64()?;
        let retained_approval_count = input.u64()?;
        let record_count = input.u32()? as usize;
        let limit = config.limits().max_records as usize;
        if record_count > limit || record_count < config.bootstrap().len() {
            return Err(StateCodecErrorV2::RecordLimit);
        }
        if retained_approval_count > limit as u64 {
            return Err(StateCodecErrorV2::ApprovalLimit);
        }
        if active_count > record_count as u64
            || pending_count > u64::from(config.limits().max_pending)
            || active_high_water > record_count as u64
            || total_activations > record_count as u64
        {
            return Err(StateCodecErrorV2::Inconsistent("counter bounds"));
        }
        // Counts must fit actual bytes before any record/index allocation.
        let minimum = record_count
            .checked_mul(MIN_RECORD_LEN)
            .and_then(|n| {
                (retained_approval_count as usize)
                    .checked_mul(APPROVAL_LEN)
                    .and_then(|a| n.checked_add(a))
            })
            .ok_or(StateCodecErrorV2::TooLarge)?;
        input.require(minimum)?;
        let mut state = MembershipStateV2 {
            config: config.clone(),
            height,
            records: BTreeMap::new(),
            used_keys: BTreeMap::new(),
            used_evidence: BTreeMap::new(),
            active_count,
            pending_count,
            active_high_water,
            total_activations,
            retained_approval_count,
        };
        let mut last_id = None;
        let mut parsed_approvals = 0usize;
        for record_index in 0..record_count {
            let id = input.hash()?;
            let raw_id = *id.as_bytes();
            if last_id.is_some_and(|previous| previous >= raw_id) {
                return Err(StateCodecErrorV2::RecordOrder);
            }
            last_id = Some(raw_id);
            let origin = match input.u8()? {
                0 => MembershipOriginV2::Seed {
                    index: input.u32()?,
                    descriptor: BootstrapDescriptor {
                        public_key: PublicKey::from_bytes(input.array()?),
                        nonce: input.hash()?,
                        evidence: input.hash()?,
                    },
                },
                1 => {
                    let operation = SignedMembershipOperationV2::decode(config, input.slice(291)?)
                        .map_err(StateCodecErrorV2::Operation)?;
                    if !matches!(operation.action(), MembershipActionV2::Enroll { .. }) {
                        return Err(StateCodecErrorV2::Inconsistent("origin kind"));
                    }
                    MembershipOriginV2::Enrollment(operation)
                }
                tag => {
                    return Err(StateCodecErrorV2::InvalidTag {
                        field: "origin",
                        tag,
                    });
                }
            };
            let status = match input.u8()? {
                0 => MembershipStatusV2::Pending,
                1 => MembershipStatusV2::Active,
                2 => MembershipStatusV2::Revoked,
                3 => MembershipStatusV2::Expired,
                tag => {
                    return Err(StateCodecErrorV2::InvalidTag {
                        field: "status",
                        tag,
                    });
                }
            };
            let inclusion = InclusionPositionV2 {
                height: input.u64()?,
                operation_index: input.u32()?,
            };
            let next_sequence = input.u64()?;
            let activation = match input.u8()? {
                0 => None,
                1 => Some(ActivationV2 {
                    height: input.u64()?,
                    ordinal: input.u64()?,
                }),
                tag => {
                    return Err(StateCodecErrorV2::InvalidTag {
                        field: "activation",
                        tag,
                    });
                }
            };
            let approval_count = input.u32()? as usize;
            parsed_approvals = parsed_approvals
                .checked_add(approval_count)
                .ok_or(StateCodecErrorV2::TooLarge)?;
            if approval_count > MAX_APPROVALS_PER_RECORD_V2
                || parsed_approvals > limit
                || parsed_approvals as u64 > retained_approval_count
            {
                return Err(StateCodecErrorV2::ApprovalLimit);
            }
            let minimum_remaining = approval_count
                .checked_mul(APPROVAL_LEN)
                .and_then(|n| {
                    (record_count - record_index - 1)
                        .checked_mul(MIN_RECORD_LEN)
                        .and_then(|r| n.checked_add(r))
                })
                .ok_or(StateCodecErrorV2::TooLarge)?;
            input.require(minimum_remaining)?;
            let mut approvals = BTreeMap::new();
            let mut last_key = None;
            for _ in 0..approval_count {
                let key: [u8; 32] = input.array()?;
                if last_key.is_some_and(|previous| previous >= key) {
                    return Err(StateCodecErrorV2::ApprovalOrder);
                }
                last_key = Some(key);
                let accepted_height = input.u64()?;
                let operation = SignedMembershipOperationV2::decode(config, input.slice(227)?)
                    .map_err(StateCodecErrorV2::Operation)?;
                if !matches!(operation.action(), MembershipActionV2::Approve { .. }) {
                    return Err(StateCodecErrorV2::Inconsistent("approval kind"));
                }
                approvals.insert(
                    key,
                    ApprovalRecordV2 {
                        operation,
                        accepted_height,
                    },
                );
            }
            let record = MembershipRecordV2 {
                id,
                origin,
                status,
                inclusion,
                next_sequence,
                activation,
                approvals,
            };
            if state
                .used_keys
                .insert(*record.public_key().as_bytes(), raw_id)
                .is_some()
                || state
                    .used_evidence
                    .insert(
                        *record
                            .origin
                            .evidence()
                            .map_err(StateCodecErrorV2::State)?
                            .as_bytes(),
                        raw_id,
                    )
                    .is_some()
            {
                return Err(StateCodecErrorV2::Inconsistent("duplicate key/evidence"));
            }
            state.records.insert(raw_id, record);
        }
        if input.remaining() != 0 {
            return Err(StateCodecErrorV2::TrailingBytes);
        }
        state.check_invariants().map_err(StateCodecErrorV2::State)?;
        check_retained_consistency(&state)?;
        // Only after all input-size/count/order/accounting checks succeed.
        for record in state.records.values() {
            if let MembershipOriginV2::Enrollment(operation) = record.origin {
                operation
                    .verify_signature()
                    .map_err(StateCodecErrorV2::Operation)?;
            }
            for approval in record.approvals.values() {
                approval
                    .operation
                    .verify_signature()
                    .map_err(StateCodecErrorV2::Operation)?;
            }
        }
        Ok(Self { state })
    }

    pub fn encode(&self) -> Result<Vec<u8>, StateCodecErrorV2> {
        self.state.encode()
    }
    pub fn root(&self) -> Result<Hash256, StateCodecErrorV2> {
        self.state.root()
    }
    pub const fn height(&self) -> u64 {
        self.state.height
    }
    pub fn record_count(&self) -> usize {
        self.state.records.len()
    }

    /// Compare all decoded fields and reconstructed indexes with independently
    /// trusted execution state. Success conveys no additional authority and does
    /// not return a runtime object constructed from the untrusted cache.
    pub fn verify_against(&self, trusted: &MembershipStateV2) -> Result<(), StateCodecErrorV2> {
        if &self.state != trusted {
            return Err(StateCodecErrorV2::SnapshotMismatch);
        }
        Ok(())
    }
}

fn encoded_len(state: &MembershipStateV2) -> Result<usize, StateCodecErrorV2> {
    state.check_invariants().map_err(StateCodecErrorV2::State)?;
    let mut len = PREFIX_LEN;
    for record in state.records.values() {
        let base = match record.origin {
            MembershipOriginV2::Seed { .. } => 159usize,
            _ => 350usize,
        };
        let activation = if record.activation.is_some() { 16 } else { 0 };
        len = record
            .approvals
            .len()
            .checked_mul(APPROVAL_LEN)
            .and_then(|n| n.checked_add(base))
            .and_then(|n| n.checked_add(activation))
            .and_then(|n| n.checked_add(len))
            .ok_or(StateCodecErrorV2::TooLarge)?;
    }
    if len > MembershipStateV2::max_encoded_len(&state.config)? {
        return Err(StateCodecErrorV2::TooLarge);
    }
    Ok(len)
}

fn allocated_bytes(capacity: usize) -> Result<Vec<u8>, StateCodecErrorV2> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| StateCodecErrorV2::AllocationFailed)?;
    Ok(bytes)
}

fn append_state(state: &MembershipStateV2, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(state.config.context().as_bytes());
    for value in [
        state.height,
        state.active_count,
        state.pending_count,
        state.active_high_water,
        state.total_activations,
        state.retained_approval_count,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&(state.records.len() as u32).to_le_bytes());
    for record in state.records.values() {
        bytes.extend_from_slice(record.id.as_bytes());
        match record.origin {
            MembershipOriginV2::Seed { index, descriptor } => {
                bytes.push(0);
                bytes.extend_from_slice(&index.to_le_bytes());
                bytes.extend_from_slice(&descriptor.encode());
            }
            MembershipOriginV2::Enrollment(operation) => {
                bytes.push(1);
                bytes.extend_from_slice(&operation.encode());
            }
        }
        bytes.push(match record.status {
            MembershipStatusV2::Pending => 0,
            MembershipStatusV2::Active => 1,
            MembershipStatusV2::Revoked => 2,
            MembershipStatusV2::Expired => 3,
        });
        bytes.extend_from_slice(&record.inclusion.height.to_le_bytes());
        bytes.extend_from_slice(&record.inclusion.operation_index.to_le_bytes());
        bytes.extend_from_slice(&record.next_sequence.to_le_bytes());
        match record.activation {
            None => bytes.push(0),
            Some(activation) => {
                bytes.push(1);
                bytes.extend_from_slice(&activation.height.to_le_bytes());
                bytes.extend_from_slice(&activation.ordinal.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&(record.approvals.len() as u32).to_le_bytes());
        for (key, approval) in &record.approvals {
            bytes.extend_from_slice(key);
            bytes.extend_from_slice(&approval.accepted_height.to_le_bytes());
            bytes.extend_from_slice(&approval.operation.encode());
        }
    }
}

/// Additional cache-only consistency checks infer what can be inferred from
/// retained fields. They do not infer missing historical eligibility/withdrawals.
fn check_retained_consistency(state: &MembershipStateV2) -> Result<(), StateCodecErrorV2> {
    let mut outgoing = BTreeMap::<[u8; 32], u64>::new();
    let mut sequences = BTreeSet::new();
    for record in state.records.values() {
        if let MembershipOriginV2::Enrollment(operation) = record.origin {
            check_operation_metadata(state, &operation, record.inclusion.height)?;
            if record.inclusion.operation_index >= state.config.limits().max_operations
                || record.inclusion.operation_index
                    >= state.config.limits().max_membership_operations
            {
                return Err(StateCodecErrorV2::Inconsistent("inclusion operation index"));
            }
            if let Some(activation) = record.activation {
                if activation.height > operation.header().valid_until
                    || record.approvals.len()
                        < usize::from(state.config.parameters().required_approvals)
                {
                    return Err(StateCodecErrorV2::Inconsistent(
                        "activation evidence/window",
                    ));
                }
            }
        }
        for approval in record.approvals.values() {
            check_operation_metadata(state, &approval.operation, approval.accepted_height)?;
            let header = approval.operation.header();
            let reviewer = state
                .record_by_key(header.actor_public_key)
                .ok_or(StateCodecErrorV2::Inconsistent("unknown reviewer"))?;
            if reviewer
                .activation
                .is_none_or(|activation| activation.height >= approval.accepted_height)
                || record
                    .activation
                    .is_some_and(|activation| approval.accepted_height > activation.height)
            {
                return Err(StateCodecErrorV2::Inconsistent(
                    "approval activation timing",
                ));
            }
            let base = if matches!(reviewer.origin, MembershipOriginV2::Seed { .. }) {
                0
            } else {
                1
            };
            let terminal = u64::from(reviewer.status == MembershipStatusV2::Revoked);
            let end = reviewer
                .next_sequence
                .checked_sub(terminal)
                .ok_or(StateCodecErrorV2::Inconsistent("review sequence"))?;
            if header.actor_sequence < base
                || header.actor_sequence >= end
                || !sequences.insert((*header.actor_public_key.as_bytes(), header.actor_sequence))
            {
                return Err(StateCodecErrorV2::Inconsistent("review sequence"));
            }
            let count = outgoing
                .entry(*header.actor_public_key.as_bytes())
                .or_default();
            *count = count.checked_add(1).ok_or(StateCodecErrorV2::TooLarge)?;
        }
    }
    for record in state.records.values() {
        let base = if matches!(record.origin, MembershipOriginV2::Seed { .. }) {
            0u64
        } else {
            1
        };
        let terminal = u64::from(record.status == MembershipStatusV2::Revoked);
        let expected = base
            .checked_add(terminal)
            .and_then(|n| {
                n.checked_add(
                    outgoing
                        .get(record.public_key().as_bytes())
                        .copied()
                        .unwrap_or(0),
                )
            })
            .ok_or(StateCodecErrorV2::TooLarge)?;
        if record.next_sequence != expected {
            return Err(StateCodecErrorV2::Inconsistent("next sequence"));
        }
    }
    Ok(())
}

fn check_operation_metadata(
    state: &MembershipStateV2,
    operation: &SignedMembershipOperationV2,
    accepted_height: u64,
) -> Result<(), StateCodecErrorV2> {
    let h = operation.header();
    let age = accepted_height
        .checked_sub(h.anchor_height)
        .filter(|n| *n > 0)
        .ok_or(StateCodecErrorV2::Inconsistent("anchor height"))?;
    let span = h
        .valid_until
        .checked_sub(h.valid_from)
        .and_then(|n| n.checked_add(1))
        .ok_or(StateCodecErrorV2::Inconsistent("validity window"))?;
    if h.config_hash != state.config.context()
        || age > state.config.limits().max_anchor_age
        || span > state.config.limits().max_validity_blocks
        || h.anchor_height >= h.valid_from
        || accepted_height < h.valid_from
        || accepted_height > h.valid_until
    {
        return Err(StateCodecErrorV2::Inconsistent("operation context/window"));
    }
    if let MembershipActionV2::Enroll {
        applicant_nonce,
        evidence_commitment,
        evidence_profile,
    } = operation.action()
    {
        if *applicant_nonce == Hash256::default()
            || *evidence_commitment == Hash256::default()
            || *evidence_profile != state.config.parameters().evidence_profile
        {
            return Err(StateCodecErrorV2::Inconsistent("enrollment evidence"));
        }
    }
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
    fn require(&self, len: usize) -> Result<(), StateCodecErrorV2> {
        if self.remaining() < len {
            return Err(StateCodecErrorV2::Truncated);
        }
        Ok(())
    }
    fn slice(&mut self, len: usize) -> Result<&'a [u8], StateCodecErrorV2> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(StateCodecErrorV2::TooLarge)?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or(StateCodecErrorV2::Truncated)?;
        self.position = end;
        Ok(slice)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], StateCodecErrorV2> {
        self.slice(N)?
            .try_into()
            .map_err(|_| StateCodecErrorV2::Truncated)
    }
    fn u8(&mut self) -> Result<u8, StateCodecErrorV2> {
        Ok(self.array::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, StateCodecErrorV2> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32, StateCodecErrorV2> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64, StateCodecErrorV2> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    fn hash(&mut self) -> Result<Hash256, StateCodecErrorV2> {
        Ok(Hash256::from_bytes(self.array()?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateCodecErrorV2 {
    TooLarge,
    AllocationFailed,
    Truncated,
    TrailingBytes,
    InvalidMagic,
    UnsupportedVersion,
    WrongContext,
    RecordLimit,
    ApprovalLimit,
    InvalidTag { field: &'static str, tag: u8 },
    RecordOrder,
    ApprovalOrder,
    Inconsistent(&'static str),
    State(StateErrorV2),
    Operation(OperationErrorV2),
    SnapshotMismatch,
}

impl fmt::Display for StateCodecErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 membership snapshot rejected: {self:?}")
    }
}
impl std::error::Error for StateCodecErrorV2 {}
