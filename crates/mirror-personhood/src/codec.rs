//! Bounded development event-log encoding. Decoding is NOT authorization.
//! Supply the trusted rules/bootstrap separately and replay before accepting state.

use super::*;

const MAGIC: [u8; 4] = *b"MPH1";
pub const MAX_LOG_BYTES: usize = 4 * 1024 * 1024;
const MIN_EVENT_LEN: usize = 129;

pub fn encode_log(events: &[MembershipEvent]) -> Result<Vec<u8>, LogError> {
    let count = u32::try_from(events.len()).map_err(|_| LogError::TooLarge)?;
    let size = events.iter().try_fold(8usize, |size, event| {
        size.checked_add(event_len(event)).ok_or(LogError::TooLarge)
    })?;
    if size > MAX_LOG_BYTES { return Err(LogError::TooLarge); }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&count.to_le_bytes());
    for event in events {
        match event {
            MembershipEvent::Enroll(request) => {
                bytes.push(1);
                bytes.extend_from_slice(request.context.as_bytes());
                bytes.extend_from_slice(request.public_key.as_bytes());
                bytes.extend_from_slice(request.nonce.as_bytes());
                bytes.extend_from_slice(request.evidence.as_bytes());
                bytes.extend_from_slice(request.signature.as_bytes());
            }
            MembershipEvent::Approve(approval) => {
                bytes.push(2);
                bytes.extend_from_slice(approval.context.as_bytes());
                bytes.extend_from_slice(approval.applicant.as_bytes());
                bytes.extend_from_slice(approval.verifier.as_bytes());
                bytes.extend_from_slice(approval.signature.as_bytes());
            }
            MembershipEvent::Withdraw(withdrawal) => {
                bytes.push(3);
                bytes.extend_from_slice(withdrawal.context.as_bytes());
                bytes.extend_from_slice(withdrawal.member.as_bytes());
                bytes.extend_from_slice(withdrawal.signature.as_bytes());
            }
        }
    }
    Ok(bytes)
}

pub fn decode_log(bytes: &[u8]) -> Result<Vec<MembershipEvent>, LogError> {
    if bytes.len() > MAX_LOG_BYTES { return Err(LogError::TooLarge); }
    let mut input = Input { bytes, offset: 0 };
    if input.array::<4>()? != MAGIC { return Err(LogError::InvalidMagic); }
    let count = usize::try_from(u32::from_le_bytes(input.array::<4>()?))
        .map_err(|_| LogError::TooLarge)?;
    // Reject impossible counts BEFORE reserving any attacker-selected capacity.
    if count > input.remaining() / MIN_EVENT_LEN { return Err(LogError::Truncated); }
    let mut events = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = input.array::<1>()?[0];
        let event = match kind {
            1 => MembershipEvent::Enroll(Enrollment {
                context: Hash256::from_bytes(input.array()?),
                public_key: PublicKey::from_bytes(input.array()?),
                nonce: Hash256::from_bytes(input.array()?),
                evidence: Hash256::from_bytes(input.array()?),
                signature: SignatureBytes::from_bytes(input.array()?),
            }),
            2 => MembershipEvent::Approve(Approval {
                context: Hash256::from_bytes(input.array()?),
                applicant: MemberId::from_bytes(input.array()?),
                verifier: MemberId::from_bytes(input.array()?),
                signature: SignatureBytes::from_bytes(input.array()?),
            }),
            3 => MembershipEvent::Withdraw(Withdrawal {
                context: Hash256::from_bytes(input.array()?),
                member: MemberId::from_bytes(input.array()?),
                signature: SignatureBytes::from_bytes(input.array()?),
            }),
            _ => return Err(LogError::UnknownEvent),
        };
        events.push(event);
    }
    if input.remaining() != 0 { return Err(LogError::TrailingBytes); }
    Ok(events)
}

fn event_len(event: &MembershipEvent) -> usize {
    match event {
        MembershipEvent::Enroll(_) => 193,
        MembershipEvent::Approve(_) => 161,
        MembershipEvent::Withdraw(_) => 129,
    }
}

struct Input<'a> { bytes: &'a [u8], offset: usize }
impl Input<'_> {
    fn remaining(&self) -> usize { self.bytes.len() - self.offset }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], LogError> {
        let end = self.offset.checked_add(N).ok_or(LogError::TooLarge)?;
        let value = self.bytes.get(self.offset..end).ok_or(LogError::Truncated)?;
        let array = value.try_into().map_err(|_| LogError::Truncated)?;
        self.offset = end;
        Ok(array)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogError { InvalidMagic, Truncated, UnknownEvent, TrailingBytes, TooLarge }
impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid membership event log: {self:?}")
    }
}
impl std::error::Error for LogError {}
