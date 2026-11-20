//! B3: bounded, immutable candidate-block membership execution.
//!
//! This is synthetic membership authorization, not biometric matching or block
//! consensus. The caller supplies locally validated, stable parent ancestry.
//! Complete block/root/PoW validation must precede binding the resulting state to
//! a block hash. That hash is provenance, never part of canonical state bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use mirror_crypto::{Hash256, PublicKey};

use super::{
    BootstrapDescriptor, ConfigV2, MAX_APPROVAL_THRESHOLD, MembershipActionV2, OperationErrorV2,
    ParentChainViewV2, SignedMembershipOperationV2, VerifiedBootstrap,
};

#[path = "state_codec.rs"]
mod codec;
pub use codec::{StateCodecErrorV2, UntrustedMembershipSnapshotV2};

/// Fixed synthetic-profile budget. Retained approvals also have an aggregate
/// budget of `config.limits().max_records`, independent of the record count.
/// Neither history nor duplicate protection is evicted to reclaim this budget.
pub const MAX_APPROVALS_PER_RECORD_V2: usize = MAX_APPROVAL_THRESHOLD as usize;

/// MRB2 magic/header/count overhead, charged before operation bytes.
pub const MEMBERSHIP_BLOCK_BASE_BYTES_V2: usize = 168;

// Cross-crate V2 format contract: the only non-membership operation in MRB2 is
// Transfer (kind 0), whose complete signed encoding is exactly 243 bytes. This
// crate cannot depend on mirror-core; indexed staging charges every omitted slot
// this exact size plus its four-byte length prefix. Any new operation kind or
// changed encoding requires explicit version/format review in both crates.
const INDEXED_TRANSFER_OPERATION_BYTES_V2: usize = 243;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipStatusV2 {
    Pending,
    Active,
    Revoked,
    Expired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InclusionPositionV2 {
    pub height: u64,
    pub operation_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivationV2 {
    pub height: u64,
    pub ordinal: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipOriginV2 {
    Seed {
        index: u32,
        descriptor: BootstrapDescriptor,
    },
    Enrollment(SignedMembershipOperationV2),
}

impl MembershipOriginV2 {
    fn public_key(&self) -> PublicKey {
        match self {
            Self::Seed { descriptor, .. } => descriptor.public_key,
            Self::Enrollment(operation) => operation.header().actor_public_key,
        }
    }

    fn evidence(&self) -> Result<Hash256, StateErrorV2> {
        match self {
            Self::Seed { descriptor, .. } => Ok(descriptor.evidence),
            Self::Enrollment(operation) => match operation.action() {
                MembershipActionV2::Enroll {
                    evidence_commitment,
                    ..
                } => Ok(*evidence_commitment),
                _ => Err(StateErrorV2::InconsistentState("non-enrollment origin")),
            },
        }
    }

    fn request(&self) -> Result<&SignedMembershipOperationV2, StateErrorV2> {
        match self {
            Self::Enrollment(operation) => Ok(operation),
            Self::Seed { .. } => Err(StateErrorV2::InconsistentState("pending seed")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApprovalRecordV2 {
    operation: SignedMembershipOperationV2,
    accepted_height: u64,
}

impl ApprovalRecordV2 {
    pub const fn operation(&self) -> &SignedMembershipOperationV2 {
        &self.operation
    }

    pub const fn accepted_height(&self) -> u64 {
        self.accepted_height
    }
}

/// Read-only record. Terminal records retain their origin, sequence and reviews.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipRecordV2 {
    id: Hash256,
    origin: MembershipOriginV2,
    status: MembershipStatusV2,
    inclusion: InclusionPositionV2,
    next_sequence: u64,
    activation: Option<ActivationV2>,
    approvals: BTreeMap<[u8; 32], ApprovalRecordV2>,
}

impl MembershipRecordV2 {
    pub const fn id(&self) -> Hash256 {
        self.id
    }
    pub fn public_key(&self) -> PublicKey {
        self.origin.public_key()
    }
    pub const fn origin(&self) -> &MembershipOriginV2 {
        &self.origin
    }
    pub const fn status(&self) -> MembershipStatusV2 {
        self.status
    }
    pub const fn inclusion(&self) -> InclusionPositionV2 {
        self.inclusion
    }
    pub const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }
    pub const fn activation(&self) -> Option<ActivationV2> {
        self.activation
    }
    pub fn approval_count(&self) -> usize {
        self.approvals.len()
    }
    pub fn approvals(&self) -> impl Iterator<Item = &ApprovalRecordV2> {
        self.approvals.values()
    }
}

/// Canonical membership data. No mutable accessor or single-event apply bypass.
/// Indexes are derived from ALL records, including expired/revoked tombstones.
/// The B4 codec/root must use the ordering frozen in MEMBERSHIP_STATE_V2.md;
/// Rust's in-memory layout and this type's Debug output are not a wire format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipStateV2 {
    config: ConfigV2,
    height: u64,
    records: BTreeMap<[u8; 32], MembershipRecordV2>,
    used_keys: BTreeMap<[u8; 32], [u8; 32]>,
    used_evidence: BTreeMap<[u8; 32], [u8; 32]>,
    active_count: u64,
    pending_count: u64,
    active_high_water: u64,
    total_activations: u64,
    retained_approval_count: u64,
}

impl MembershipStateV2 {
    pub const fn config(&self) -> &ConfigV2 {
        &self.config
    }
    pub const fn height(&self) -> u64 {
        self.height
    }
    pub fn record(&self, id: Hash256) -> Option<&MembershipRecordV2> {
        self.records.get(id.as_bytes())
    }
    pub fn record_by_key(&self, key: PublicKey) -> Option<&MembershipRecordV2> {
        self.used_keys
            .get(key.as_bytes())
            .and_then(|id| self.records.get(id))
    }
    pub fn records(&self) -> impl Iterator<Item = &MembershipRecordV2> {
        self.records.values()
    }
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
    pub const fn active_count(&self) -> u64 {
        self.active_count
    }
    pub const fn pending_count(&self) -> u64 {
        self.pending_count
    }
    pub const fn active_high_water(&self) -> u64 {
        self.active_high_water
    }
    pub const fn total_activations(&self) -> u64 {
        self.total_activations
    }
    pub const fn retained_approval_count(&self) -> u64 {
        self.retained_approval_count
    }

    fn active_keys(&self) -> BTreeSet<[u8; 32]> {
        self.records
            .values()
            .filter(|record| record.status == MembershipStatusV2::Active)
            .map(|record| *record.public_key().as_bytes())
            .collect()
    }

    fn expire_pending(&mut self) -> Result<(), StateErrorV2> {
        for record in self.records.values_mut() {
            if record.status == MembershipStatusV2::Pending
                && record.origin.request()?.header().valid_until < self.height
            {
                record.status = MembershipStatusV2::Expired;
                self.pending_count = self
                    .pending_count
                    .checked_sub(1)
                    .ok_or(StateErrorV2::CounterOverflow)?;
            }
        }
        Ok(())
    }

    fn enroll(
        &mut self,
        operation: &SignedMembershipOperationV2,
        index: u32,
    ) -> Result<(), StateErrorV2> {
        let id = operation
            .enrollment_id()
            .ok_or(StateErrorV2::InconsistentState("enrollment kind"))?;
        let origin = MembershipOriginV2::Enrollment(*operation);
        let key = *origin.public_key().as_bytes();
        let evidence = *origin.evidence()?.as_bytes();
        if self.records.contains_key(id.as_bytes()) {
            return Err(StateErrorV2::DuplicateId);
        }
        if self.used_keys.contains_key(&key) {
            return Err(StateErrorV2::DuplicateKey);
        }
        if self.used_evidence.contains_key(&evidence) {
            return Err(StateErrorV2::DuplicateEvidence);
        }
        if self.records.len() >= self.config.limits().max_records as usize {
            return Err(StateErrorV2::RecordLimit);
        }
        if self.pending_count >= u64::from(self.config.limits().max_pending) {
            return Err(StateErrorV2::PendingLimit);
        }
        let pending = self
            .pending_count
            .checked_add(1)
            .ok_or(StateErrorV2::CounterOverflow)?;
        let record = MembershipRecordV2 {
            id,
            origin,
            status: MembershipStatusV2::Pending,
            inclusion: InclusionPositionV2 {
                height: self.height,
                operation_index: index,
            },
            next_sequence: 1,
            activation: None,
            approvals: BTreeMap::new(),
        };
        self.records.insert(*id.as_bytes(), record);
        self.used_keys.insert(key, *id.as_bytes());
        self.used_evidence.insert(evidence, *id.as_bytes());
        self.pending_count = pending;
        Ok(())
    }

    fn next_sequence(&self, actor: &[u8; 32], supplied: u64) -> Result<u64, StateErrorV2> {
        let record = self.records.get(actor).ok_or(StateErrorV2::UnknownActor)?;
        if record.next_sequence != supplied {
            return Err(StateErrorV2::SequenceMismatch {
                expected: record.next_sequence,
                got: supplied,
            });
        }
        supplied
            .checked_add(1)
            .ok_or(StateErrorV2::SequenceOverflow)
    }

    fn approve(
        &mut self,
        operation: &SignedMembershipOperationV2,
        target_id: Hash256,
        parent_active: &BTreeSet<[u8; 32]>,
    ) -> Result<(), StateErrorV2> {
        let header = operation.header();
        let key = *header.actor_public_key.as_bytes();
        let actor_id = *self.used_keys.get(&key).ok_or(StateErrorV2::UnknownActor)?;
        let target = self
            .records
            .get(target_id.as_bytes())
            .ok_or(StateErrorV2::UnknownTarget)?;
        if actor_id == *target_id.as_bytes() {
            return Err(StateErrorV2::SelfApproval);
        }
        if !parent_active.contains(&key)
            || self
                .records
                .get(&actor_id)
                .ok_or(StateErrorV2::UnknownActor)?
                .status
                != MembershipStatusV2::Active
        {
            return Err(StateErrorV2::ReviewerNotActive);
        }
        if target.status != MembershipStatusV2::Pending {
            return Err(StateErrorV2::TargetNotPending);
        }
        let request = target.origin.request()?.header();
        if header.valid_until != request.valid_until || header.valid_from < request.valid_from {
            return Err(StateErrorV2::ApprovalWindowMismatch);
        }
        if target.approvals.contains_key(&key) {
            return Err(StateErrorV2::DuplicateApproval);
        }
        if target.approvals.len() >= MAX_APPROVALS_PER_RECORD_V2 {
            return Err(StateErrorV2::ApprovalRecordLimit);
        }
        if self.retained_approval_count >= u64::from(self.config.limits().max_records) {
            return Err(StateErrorV2::ApprovalTotalLimit);
        }
        let sequence = self.next_sequence(&actor_id, header.actor_sequence)?;
        let count = self
            .retained_approval_count
            .checked_add(1)
            .ok_or(StateErrorV2::CounterOverflow)?;
        self.records
            .get_mut(&actor_id)
            .ok_or(StateErrorV2::UnknownActor)?
            .next_sequence = sequence;
        self.records
            .get_mut(target_id.as_bytes())
            .ok_or(StateErrorV2::UnknownTarget)?
            .approvals
            .insert(
                key,
                ApprovalRecordV2 {
                    operation: *operation,
                    accepted_height: self.height,
                },
            );
        self.retained_approval_count = count;
        Ok(())
    }

    fn withdraw(
        &mut self,
        operation: &SignedMembershipOperationV2,
        own_id: Hash256,
    ) -> Result<(), StateErrorV2> {
        let header = operation.header();
        let actor_id = *self
            .used_keys
            .get(header.actor_public_key.as_bytes())
            .ok_or(StateErrorV2::UnknownActor)?;
        if actor_id != *own_id.as_bytes() {
            return Err(StateErrorV2::NotOwner);
        }
        let status = self
            .records
            .get(&actor_id)
            .ok_or(StateErrorV2::UnknownActor)?
            .status;
        let count = match status {
            MembershipStatusV2::Pending => self.pending_count.checked_sub(1),
            MembershipStatusV2::Active => self.active_count.checked_sub(1),
            _ => return Err(StateErrorV2::TerminalMembership),
        }
        .ok_or(StateErrorV2::CounterOverflow)?;
        let sequence = self.next_sequence(&actor_id, header.actor_sequence)?;
        let actor = self
            .records
            .get_mut(&actor_id)
            .ok_or(StateErrorV2::UnknownActor)?;
        actor.status = MembershipStatusV2::Revoked;
        actor.next_sequence = sequence;
        match status {
            MembershipStatusV2::Pending => self.pending_count = count,
            MembershipStatusV2::Active => self.active_count = count,
            _ => return Err(StateErrorV2::TerminalMembership),
        }
        Ok(())
    }

    fn activate_at_block_end(
        &mut self,
        parent_active: &BTreeSet<[u8; 32]>,
    ) -> Result<(), StateErrorV2> {
        // Capture the post-operation set BEFORE any activations. Prior approvals
        // count only if their reviewers survive in both this and the parent set.
        let surviving = self.active_keys();
        let threshold = usize::from(self.config.parameters().required_approvals);
        let mut ready = Vec::new();
        for (id, record) in &self.records {
            if record.status != MembershipStatusV2::Pending {
                continue;
            }
            let eligible = record
                .approvals
                .keys()
                .filter(|key| parent_active.contains(*key) && surviving.contains(*key))
                .count();
            if eligible >= threshold {
                ready.push((record.inclusion, *id));
            }
        }
        ready.sort_unstable_by_key(|(position, _)| *position);
        for (_, id) in ready {
            let ordinal = self
                .total_activations
                .checked_add(1)
                .ok_or(StateErrorV2::CounterOverflow)?;
            let active = self
                .active_count
                .checked_add(1)
                .ok_or(StateErrorV2::CounterOverflow)?;
            let pending = self
                .pending_count
                .checked_sub(1)
                .ok_or(StateErrorV2::CounterOverflow)?;
            let record = self
                .records
                .get_mut(&id)
                .ok_or(StateErrorV2::UnknownTarget)?;
            if record.activation.is_some() {
                return Err(StateErrorV2::InconsistentState("repeat activation"));
            }
            record.status = MembershipStatusV2::Active;
            record.activation = Some(ActivationV2 {
                height: self.height,
                ordinal,
            });
            self.total_activations = ordinal;
            self.active_count = active;
            self.pending_count = pending;
        }
        self.active_high_water = self.active_high_water.max(self.active_count);
        Ok(())
    }

    /// Structural consistency check for private derived indexes/counters. This
    /// is NOT an untrusted snapshot validator or a substitute for signed replay.
    /// B4 must reconstruct state through verified history, not bless decoded data
    /// solely because these internal accounting checks happen to agree.
    fn check_invariants(&self) -> Result<(), StateErrorV2> {
        let invalid = |reason| StateErrorV2::InconsistentState(reason);
        if self.records.len() > self.config.limits().max_records as usize
            || self.used_keys.len() != self.records.len()
            || self.used_evidence.len() != self.records.len()
        {
            return Err(invalid("record/index counts"));
        }
        let mut active = 0u64;
        let mut pending = 0u64;
        let mut approvals = 0u64;
        let mut ordinals = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for (id, record) in &self.records {
            if id != record.id.as_bytes()
                || self.used_keys.get(record.public_key().as_bytes()) != Some(id)
                || self.used_evidence.get(record.origin.evidence()?.as_bytes()) != Some(id)
                || record.inclusion.height > self.height
                || !positions.insert(record.inclusion)
            {
                return Err(invalid("record identity/index/position"));
            }
            match record.origin {
                MembershipOriginV2::Seed { index, descriptor } => {
                    if self.config.bootstrap().get(index as usize) != Some(&descriptor)
                        || self
                            .config
                            .seed_id(index)
                            .map_err(|_| invalid("seed index"))?
                            != record.id
                        || record.inclusion
                            != (InclusionPositionV2 {
                                height: 0,
                                operation_index: index,
                            })
                        || record.activation
                            != Some(ActivationV2 {
                                height: 0,
                                ordinal: u64::from(index) + 1,
                            })
                        || !matches!(
                            record.status,
                            MembershipStatusV2::Active | MembershipStatusV2::Revoked
                        )
                        || !record.approvals.is_empty()
                    {
                        return Err(invalid("seed record"));
                    }
                }
                MembershipOriginV2::Enrollment(operation) => {
                    let header = operation.header();
                    if operation.enrollment_id() != Some(record.id)
                        || header.config_hash != self.config.context()
                        || header.actor_sequence != 0
                        || record.inclusion.height == 0
                        || record.next_sequence == 0
                        || header.valid_from > record.inclusion.height
                        || header.valid_until < record.inclusion.height
                    {
                        return Err(invalid("enrollment origin"));
                    }
                    if record.status == MembershipStatusV2::Pending
                        && header.valid_until < self.height
                    {
                        return Err(invalid("overdue pending"));
                    }
                    if record.status == MembershipStatusV2::Expired
                        && header.valid_until >= self.height
                    {
                        return Err(invalid("premature expiry"));
                    }
                }
            }
            match record.status {
                MembershipStatusV2::Pending => {
                    pending = pending
                        .checked_add(1)
                        .ok_or(StateErrorV2::CounterOverflow)?;
                    if record.activation.is_some() {
                        return Err(invalid("pending activation"));
                    }
                }
                MembershipStatusV2::Active => {
                    active = active.checked_add(1).ok_or(StateErrorV2::CounterOverflow)?;
                    if record.activation.is_none() {
                        return Err(invalid("missing activation"));
                    }
                }
                MembershipStatusV2::Expired if record.activation.is_some() => {
                    return Err(invalid("expired activation"));
                }
                _ => {}
            }
            if let Some(activation) = record.activation {
                if activation.height < record.inclusion.height
                    || activation.height > self.height
                    || activation.ordinal == 0
                    || activation.ordinal > self.total_activations
                    || !ordinals.insert(activation.ordinal)
                {
                    return Err(invalid("activation height/ordinal"));
                }
            }
            if record.approvals.len() > MAX_APPROVALS_PER_RECORD_V2 {
                return Err(invalid("approval record budget"));
            }
            approvals = approvals
                .checked_add(record.approvals.len() as u64)
                .ok_or(StateErrorV2::CounterOverflow)?;
            for (key, approval) in &record.approvals {
                let operation = &approval.operation;
                let header = operation.header();
                let request = record.origin.request()?.header();
                let reviewer = self
                    .record_by_key(header.actor_public_key)
                    .ok_or(invalid("approval reviewer"))?;
                if header.actor_public_key.as_bytes() != key
                    || reviewer.id == record.id
                    || operation.action()
                        != &(MembershipActionV2::Approve {
                            target_enrollment_id: record.id,
                        })
                    || header.config_hash != self.config.context()
                    || header.actor_sequence >= reviewer.next_sequence
                    || header.valid_until != request.valid_until
                    || header.valid_from < request.valid_from
                    || approval.accepted_height < record.inclusion.height
                    || approval.accepted_height > self.height
                    || approval.accepted_height < header.valid_from
                    || approval.accepted_height > header.valid_until
                {
                    return Err(invalid("approval record"));
                }
            }
        }
        for index in 0..self.config.bootstrap().len() {
            let id = self
                .config
                .seed_id(index as u32)
                .map_err(|_| invalid("missing seed"))?;
            if !self.records.contains_key(id.as_bytes()) {
                return Err(invalid("missing seed"));
            }
        }
        if active != self.active_count
            || pending != self.pending_count
            || pending > u64::from(self.config.limits().max_pending)
            || approvals != self.retained_approval_count
            || approvals > u64::from(self.config.limits().max_records)
            || ordinals.len() as u64 != self.total_activations
            || self.active_high_water < active
            || self.active_high_water > self.total_activations
        {
            return Err(invalid("counters"));
        }
        Ok(())
    }
}

/// A state associated by the TRUSTED integration caller with a validated block.
/// Provenance catches accidental mixing of different parents at the same height;
/// supplying an arbitrary hash does not prove that a block or state is valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipRegistryV2 {
    state: MembershipStateV2,
    tip_hash: Hash256,
}

impl MembershipRegistryV2 {
    /// All seed proofs have already passed ConfigV2::verify_bootstrap. Seeds get
    /// ordinary rights, sequence zero and activation ordinals in descriptor order.
    /// No account balance, bootstrap payout or privileged runtime key is created.
    pub fn bootstrap(
        bootstrap: &VerifiedBootstrap,
    ) -> Result<StagedMembershipStateV2, StateErrorV2> {
        let config = bootstrap.config();
        let mut state = MembershipStateV2 {
            config: config.clone(),
            height: 0,
            records: BTreeMap::new(),
            used_keys: BTreeMap::new(),
            used_evidence: BTreeMap::new(),
            active_count: 0,
            pending_count: 0,
            active_high_water: 0,
            total_activations: 0,
            retained_approval_count: 0,
        };
        for (index, seed) in bootstrap.seeds().iter().enumerate() {
            let descriptor = *config
                .bootstrap()
                .get(index)
                .ok_or(StateErrorV2::InconsistentState("bootstrap descriptor"))?;
            let id = seed.id();
            let record = MembershipRecordV2 {
                id,
                origin: MembershipOriginV2::Seed {
                    index: index as u32,
                    descriptor,
                },
                status: MembershipStatusV2::Active,
                inclusion: InclusionPositionV2 {
                    height: 0,
                    operation_index: index as u32,
                },
                next_sequence: 0,
                activation: Some(ActivationV2 {
                    height: 0,
                    ordinal: seed.ordinal(),
                }),
                approvals: BTreeMap::new(),
            };
            if state.records.insert(*id.as_bytes(), record).is_some() {
                return Err(StateErrorV2::DuplicateId);
            }
            state
                .used_keys
                .insert(*seed.public_key().as_bytes(), *id.as_bytes());
            state
                .used_evidence
                .insert(*descriptor.evidence.as_bytes(), *id.as_bytes());
        }
        state.active_count = bootstrap.seeds().len() as u64;
        state.active_high_water = state.active_count;
        state.total_activations = state.active_count;
        state.check_invariants()?;
        Ok(StagedMembershipStateV2 { state })
    }

    pub const fn state(&self) -> &MembershipStateV2 {
        &self.state
    }
    pub const fn tip_hash(&self) -> Hash256 {
        self.tip_hash
    }

    /// Stage one bounded, ordered membership-only block. The original registry
    /// never changes, even on success. The caller must validate the COMPLETE block
    /// and its claimed root before binding/publishing the returned candidate.
    pub fn stage_block<V: ParentChainViewV2 + ?Sized>(
        &self,
        parent: &V,
        operations: &[SignedMembershipOperationV2],
    ) -> Result<StagedMembershipStateV2, StateErrorV2> {
        let mut stage =
            self.begin_plan(parent, MembershipOperationPlanV2::Contiguous(operations))?;
        for index in 0..operations.len() {
            // begin_plan has already bounded the complete slice by max_operations.
            stage = stage.apply_next(index as u32)?;
        }
        stage.finish()
    }

    /// Begin membership execution within an ordered mixed MRB2 block. Indices
    /// are positions in the complete block, including transfers, not positions
    /// in the filtered membership slice. The plan must be strictly increasing.
    ///
    /// Every omitted position is charged as the frozen V2 Transfer encoding.
    /// The caller must independently validate and execute those transfers in
    /// their original order, stopping at the first failure, before finishing
    /// and accepting the complete accounts/membership transition. This stage
    /// cannot authenticate omitted operations or establish block authority.
    ///
    /// All budgets and parent binding are checked before cloning state. The
    /// returned builder is consumed by each step: a failed operation cannot be
    /// skipped and later published using the same candidate.
    pub fn begin_indexed_block<'a, V: ParentChainViewV2 + ?Sized>(
        &self,
        parent: &'a V,
        total_operation_count: u32,
        operations: &'a [(u32, SignedMembershipOperationV2)],
    ) -> Result<IndexedMembershipStageV2<'a, V>, StateErrorV2> {
        self.begin_plan(
            parent,
            MembershipOperationPlanV2::Indexed {
                total_operation_count,
                operations,
            },
        )
    }

    fn begin_plan<'a, V: ParentChainViewV2 + ?Sized>(
        &self,
        parent: &'a V,
        plan: MembershipOperationPlanV2<'a>,
    ) -> Result<IndexedMembershipStageV2<'a, V>, StateErrorV2> {
        self.check_budget(&plan)?;
        if parent.config_hash() != self.state.config.context() {
            return Err(StateErrorV2::WrongParentContext);
        }
        if parent.parent_height() != self.state.height {
            return Err(StateErrorV2::WrongParentHeight);
        }
        let height = self
            .state
            .height
            .checked_add(1)
            .ok_or(StateErrorV2::HeightOverflow)?;
        match parent.canonical_hash_at(self.state.height) {
            None => return Err(StateErrorV2::UnknownParent),
            Some(hash) if hash != self.tip_hash => return Err(StateErrorV2::WrongParentHash),
            Some(_) => {}
        }
        // No input-sized copies, historical clone, or signature work precedes
        // cheap block budgets and parent binding. Historical state is also bounded.
        let parent_active = self.state.active_keys();
        let mut candidate = self.state.clone();
        candidate.height = height;
        candidate.expire_pending()?;
        Ok(IndexedMembershipStageV2 {
            candidate,
            parent_active,
            parent,
            plan,
            next_operation: 0,
        })
    }

    fn check_budget(&self, plan: &MembershipOperationPlanV2<'_>) -> Result<(), StateErrorV2> {
        let limits = self.state.config.limits();
        let total_operation_count = plan.total_operation_count()?;
        if total_operation_count > limits.max_operations {
            return Err(StateErrorV2::OperationLimit);
        }
        if plan.len() > limits.max_membership_operations as usize {
            return Err(StateErrorV2::MembershipOperationLimit);
        }
        if plan.len() > total_operation_count as usize {
            return Err(StateErrorV2::MembershipCountExceedsTotal);
        }
        if let MembershipOperationPlanV2::Indexed { operations, .. } = plan {
            let mut previous = None;
            for &(index, _) in *operations {
                if index >= total_operation_count {
                    return Err(StateErrorV2::OperationIndexOutOfRange { index });
                }
                if let Some(previous) = previous {
                    if index <= previous {
                        return Err(StateErrorV2::OperationIndicesNotIncreasing {
                            previous,
                            got: index,
                        });
                    }
                }
                previous = Some(index);
            }
        }
        let transfers = total_operation_count as usize - plan.len();
        if transfers > 0
            && INDEXED_TRANSFER_OPERATION_BYTES_V2 > limits.max_operation_bytes as usize
        {
            return Err(StateErrorV2::OperationBytesLimit);
        }
        let mut bytes = transfers
            .checked_mul(4 + INDEXED_TRANSFER_OPERATION_BYTES_V2)
            .and_then(|value| value.checked_add(MEMBERSHIP_BLOCK_BASE_BYTES_V2))
            .ok_or(StateErrorV2::BlockBytesLimit)?;
        if bytes > limits.max_block_bytes as usize {
            return Err(StateErrorV2::BlockBytesLimit);
        }
        let mut enrollments = 0u32;
        for position in 0..plan.len() {
            let (_, operation) = plan
                .get(position)
                .ok_or(StateErrorV2::InconsistentState("membership plan"))?;
            let size = operation.encoded_len();
            if size > limits.max_operation_bytes as usize {
                return Err(StateErrorV2::OperationBytesLimit);
            }
            bytes = bytes
                .checked_add(4)
                .and_then(|value| value.checked_add(size))
                .ok_or(StateErrorV2::BlockBytesLimit)?;
            if bytes > limits.max_block_bytes as usize {
                return Err(StateErrorV2::BlockBytesLimit);
            }
            if matches!(operation.action(), MembershipActionV2::Enroll { .. }) {
                enrollments = enrollments
                    .checked_add(1)
                    .ok_or(StateErrorV2::EnrollmentLimit)?;
                if enrollments > limits.max_new_enrollments {
                    return Err(StateErrorV2::EnrollmentLimit);
                }
            }
        }
        Ok(())
    }
}

/// Borrowed immutable execution plan. Contiguous input preserves the original
/// membership-only path without allocating an indexed copy before preflight.
enum MembershipOperationPlanV2<'a> {
    Contiguous(&'a [SignedMembershipOperationV2]),
    Indexed {
        total_operation_count: u32,
        operations: &'a [(u32, SignedMembershipOperationV2)],
    },
}

impl MembershipOperationPlanV2<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Contiguous(operations) => operations.len(),
            Self::Indexed { operations, .. } => operations.len(),
        }
    }

    fn total_operation_count(&self) -> Result<u32, StateErrorV2> {
        match self {
            Self::Contiguous(operations) => {
                u32::try_from(operations.len()).map_err(|_| StateErrorV2::OperationLimit)
            }
            Self::Indexed {
                total_operation_count,
                ..
            } => Ok(*total_operation_count),
        }
    }

    fn get(&self, position: usize) -> Option<(u32, &SignedMembershipOperationV2)> {
        match self {
            Self::Contiguous(operations) => operations
                .get(position)
                .map(|operation| (position as u32, operation)),
            Self::Indexed { operations, .. } => operations
                .get(position)
                .map(|(index, operation)| (*index, operation)),
        }
    }
}

/// Opaque in-progress membership candidate for an ordered mixed block. There is
/// deliberately no Clone, candidate accessor, registry conversion, or fallible
/// borrowed mutator. Both apply_next and finish consume the candidate, so an
/// execution failure cannot leave a publishable partial state behind.
///
/// ```compile_fail,E0382
/// use mirror_personhood::v2::{IndexedMembershipStageV2, ParentChainViewV2};
/// fn cannot_finish_after_failed_step<V: ParentChainViewV2>(stage: IndexedMembershipStageV2<'_, V>) {
///     let _result = stage.apply_next(0);
///     let _partial = stage.finish();
/// }
/// ```
///
/// ```compile_fail,E0599
/// use mirror_personhood::v2::{IndexedMembershipStageV2, ParentChainViewV2};
/// fn cannot_clone_partial_state<V: ParentChainViewV2>(stage: IndexedMembershipStageV2<'_, V>) {
///     let _copy = stage.clone();
/// }
/// ```
pub struct IndexedMembershipStageV2<'a, V: ParentChainViewV2 + ?Sized> {
    candidate: MembershipStateV2,
    parent_active: BTreeSet<[u8; 32]>,
    parent: &'a V,
    plan: MembershipOperationPlanV2<'a>,
    next_operation: usize,
}

impl<V: ParentChainViewV2 + ?Sized> IndexedMembershipStageV2<'_, V> {
    /// Execute exactly the next planned membership operation at its full-block
    /// position. The integration caller executes any preceding transfers first.
    pub fn apply_next(mut self, expected_full_index: u32) -> Result<Self, StateErrorV2> {
        let (index, operation) = self
            .plan
            .get(self.next_operation)
            .ok_or(StateErrorV2::NoRemainingMembershipOperations)?;
        if expected_full_index != index {
            return Err(StateErrorV2::UnexpectedOperationIndex {
                expected: index,
                got: expected_full_index,
            });
        }
        operation
            .validate_for_parent(&self.candidate.config, self.parent)
            .map_err(|error| StateErrorV2::Operation { index, error })?;
        match *operation.action() {
            MembershipActionV2::Enroll { .. } => self.candidate.enroll(operation, index)?,
            MembershipActionV2::Approve {
                target_enrollment_id,
            } => self
                .candidate
                .approve(operation, target_enrollment_id, &self.parent_active)?,
            MembershipActionV2::Withdraw { own_member_id } => {
                self.candidate.withdraw(operation, own_member_id)?
            }
        }
        self.next_operation += 1;
        Ok(self)
    }

    /// Complete block-end activation only after every planned membership
    /// operation has succeeded. The returned state still needs complete ledger
    /// and block validation before it may be bound as the next parent.
    pub fn finish(mut self) -> Result<StagedMembershipStateV2, StateErrorV2> {
        if self.next_operation != self.plan.len() {
            return Err(StateErrorV2::IncompleteMembershipStage {
                remaining: (self.plan.len() - self.next_operation) as u32,
            });
        }
        self.candidate.activate_at_block_end(&self.parent_active)?;
        self.candidate.check_invariants()?;
        Ok(StagedMembershipStateV2 {
            state: self.candidate,
        })
    }
}

/// Completed B3 transition, not an accepted chain block. No stage can be used
/// directly as the next parent until the trusted integration layer binds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedMembershipStateV2 {
    state: MembershipStateV2,
}

impl StagedMembershipStateV2 {
    pub const fn state(&self) -> &MembershipStateV2 {
        &self.state
    }

    /// TRUSTED CALLER BOUNDARY: this method does not validate a hash, PoW, parent,
    /// genesis manifest, account state, ledger root or finality. MB-02 must do that
    /// before accepting a block. The tag is external provenance and is excluded
    /// from membership root bytes, avoiding a block-hash/state-root cycle.
    pub fn bind_to_validated_block(self, block_hash: Hash256) -> MembershipRegistryV2 {
        MembershipRegistryV2 {
            state: self.state,
            tip_hash: block_hash,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateErrorV2 {
    OperationLimit,
    MembershipOperationLimit,
    EnrollmentLimit,
    OperationBytesLimit,
    BlockBytesLimit,
    MembershipCountExceedsTotal,
    OperationIndexOutOfRange { index: u32 },
    OperationIndicesNotIncreasing { previous: u32, got: u32 },
    UnexpectedOperationIndex { expected: u32, got: u32 },
    NoRemainingMembershipOperations,
    IncompleteMembershipStage { remaining: u32 },
    WrongParentContext,
    WrongParentHeight,
    UnknownParent,
    WrongParentHash,
    HeightOverflow,
    Operation { index: u32, error: OperationErrorV2 },
    DuplicateId,
    DuplicateKey,
    DuplicateEvidence,
    RecordLimit,
    PendingLimit,
    UnknownActor,
    UnknownTarget,
    SelfApproval,
    ReviewerNotActive,
    TargetNotPending,
    ApprovalWindowMismatch,
    DuplicateApproval,
    ApprovalRecordLimit,
    ApprovalTotalLimit,
    SequenceMismatch { expected: u64, got: u64 },
    SequenceOverflow,
    NotOwner,
    TerminalMembership,
    CounterOverflow,
    InconsistentState(&'static str),
}

impl fmt::Display for StateErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2 membership state rejected: {self:?}")
    }
}

impl std::error::Error for StateErrorV2 {}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
