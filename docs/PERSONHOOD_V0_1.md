# Ant Kiss register v0.1 — experimental library milestone

## Scope

A deterministic authorization/state-machine foundation, not biological
identification or launched consensus. No device, ECG analysis, fingerprint
matching, biometric file or production enrollment service is included.

## Trusted context

All nodes must begin with the SAME externally agreed network identifier, approval
threshold and ordered bootstrap enrollments. `RegistryRules` commits network and
threshold into a domain-separated context. Each bootstrap request is self-signed.
The seed list is trusted initial enrollment evidence, not a decentralized proof.
A threshold-one, one-seed demo models founder-led startup only; it is unsafe to
interpret that configuration as final public admission policy.

Every seed is an ordinary active member after construction. There is no owner key,
forced-revocation key, reward privilege or ongoing bootstrap bypass.

## Events

- **Enroll:** applicant-signed context, public key, nonce and opaque evidence
  commitment. Creates Pending membership. Zero commitments and reused request,
  key or exact evidence commitment are rejected.
- **Approve:** an active member signs one precise applicant enrollment ID in the
  same rules context. Self-approval, inactive/unknown signers, forged signatures
  and repeated approvals cannot count. Distinct currently active approvals reaching
  the configured threshold activate the membership exactly once.
- **Withdraw:** the applicant/member signs terminal voluntary self-revocation.
  This can cancel Pending membership or end Active membership. It cannot alter
  anyone else's membership, transfer balances, establish death or execute an estate.

Unused/revoked records remain tombstones. Re-enrolling with a used key or evidence
commitment fails. Fresh keys plus new evidence from the SAME HUMAN are not detected
here: an external, properly specified uniqueness check is essential.

## Counters and commitment

The registry tracks current active memberships, population high-water, and total
first activations. Pending requests do not increase active population. Withdrawal
reduces current active count but does not reduce the high-water record or reuse an
activation ordinal. These are development accounting inputs, not monetary rights.

The state root commits rules, counters and the sorted complete membership map,
including status, request data, first-activation ordinal and approval identities.
All fields have fixed-size or explicit-count encoding. Derived used-key/evidence
indexes are rebuilt by validated replay, never loaded as trusted snapshots.

`apply` is failure-atomic; `apply_batch` stages all events and commits only after
every event passes. Bootstrap and replay return no partially accepted registry.

## Event log MPH1

4-byte magic `MPH1`, u32 little-endian event count, then fixed-size tagged events:
Enroll = 193 bytes, Approve = 161 bytes, Withdraw = 129 bytes. The development log
reader/writer imposes a 4 MiB resource limit, rejects truncation, invalid tags,
impossible counts and trailing bytes. Decoding only reconstructs untrusted events.
`MembershipRegistry::replay` must recheck all signatures and transition rules using
trusted configuration/bootstrap before accepting the state.

The log is not yet an on-chain transaction or `mirror-storage` crash-safe journal.
It does not authenticate an authoritative latest tip, prevent rollback to an older
valid log, or replace an external expected state root/block commitment.

## Privacy boundary

Only synthetic commitments are used in tests/demo. Real evidence must remain in a
separately designed protected off-chain system. A commitment must be appropriately
randomized; an unsalted digest of a fingerprint or name is NOT a privacy solution.
This code cannot verify how an input commitment was produced. Public enrollment
keys, timings and approval relationships are linkable; anonymity is NOT implemented.

## Pending before live integration

Define production membership operations and trusted bootstrap commitments; bind
membership root and population counters to block state, canonical decoding,
storage/replay and peer validation; add fork/reorg tests over DIFFERENT membership
histories. Specify signature authorizations, reviewer assignment, resource bounds,
challenge/recovery/third-party-revocation rules and economic integration first.
Do not gate the legacy PoW chain or issue money from this library yet.
