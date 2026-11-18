#!/usr/bin/env python3
"""Independent MPS2 preimages and commitments; no Mirror Rust code is called.

Python struct/hashlib encode five explicitly specified synthetic states, not a
second runtime executor. Fixed Ed25519 signatures were generated independently
with Python cryptography from the PUBLIC development seeds [1;32] through [5;32].
The check path uses only the standard library. Rust independently signs, verifies,
executes the complete history and compares every byte/root with this fixture.
Never use these public keys/seeds for real people, private evidence or funds.
"""
import argparse
from dataclasses import dataclass, replace
import hashlib
from pathlib import Path
import struct


def h(data: bytes) -> bytes:
    return hashlib.sha256(hashlib.sha256(data).digest()).digest()


def u16(number: int) -> bytes:
    return struct.pack('<H', number)


def u32(number: int) -> bytes:
    return struct.pack('<I', number)


def u64(number: int) -> bytes:
    return struct.pack('<Q', number)


def repeated(number: int) -> bytes:
    return bytes([number]) * 32


PUBLIC_KEYS = {
    1: bytes.fromhex('8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c'),
    2: bytes.fromhex('8139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b394'),
    3: bytes.fromhex('ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d1'),
    4: bytes.fromhex('ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c'),
    5: bytes.fromhex('6e7a1cdd29b0b78fd13af4c5598feff4ef2a97166e3ca6f2e4fbfccd80505bf1'),
}

# Filled from independent Python Ed25519, not the Rust implementation or fixture.
SIGNATURES = {'enroll_c': 'd39fd67e86009f92a72f9a3b31b6fda405ce68a4ef5f867d3b2c493e929b6c71cd70c7ce96c0e441bf648bdd665e27aa87ed7406c35638b7eb865ca885b67e00',
 'approve_a_c': '783f3a8b6af6f5de37b8b99645ff33f66d216b1d8fcbe3bdd86e4ee7a3de7a2bda64233ec8b7896d1834cefefbdd0d1bb4ba2b163a7ed7a1e3e869e9757b2c08',
 'enroll_d': '6907a2a83bf890d98dcc352a4b3acc21973c7947570a950ff35c01565374c3c6b438d8210e5e420e0d905d8af5bca18f7fc2ede112bf775b90570a355695800e',
 'approve_b_d': '676cc18c9d47cd7bb86236ecfa70e481f125d0793a85866939f1a281c9ee5739828c521e6857abbc12264d71277aa09dedb3a7648f358b99b9c0c5a66144b805',
 'enroll_e': '9240a26725130576f4cdaa32ca70bb7cc7afb5da9ba061768b0e794a3253bbc5e297a9a935ae15dfca07d205cf4c1c2128e70187c568fe20e0b1500d367e190c',
 'approve_b_c': '073391f7dd30abd3dc002d06a828f620990573370593560019cab530426e4c17163de178dc043367a20948a386dee0e59742b70be44c0f7ed34bc1201b905c0b',
 'withdraw_e': '68c99ab5035054a7ee17d8ca39fcd5bac2c628e88f57a027ca02707f1dc73e704c43ca720b2d9c26b46055b03e73c6768b1f246b72d8400609a7d63a54076506',
 'withdraw_c': 'e5bff7d47303b128bc090730990d7e72a82c249f0385634389db4d7643e420af1defd5075a79ff4d1bdfab3c983a12fc20ea1a6a75a46af40a7f181ec57eb606',
 'withdraw_a': '2c1aa75b78065898620a04943c7ad0a9f077e7323cb207d8bd52a3b511f287219c5813bb9dfa1337d52875149e995254860dfae26332a4864315bce9706ac306'}


def inputs():
    """Configuration and signed preimages defined directly from B1/B2 contracts."""
    descriptors = [PUBLIC_KEYS[n] + repeated(0x30 + n) + repeated(0x40 + n)
                   for n in (1, 2)]
    limits = [1048576, 256, 256, 64, 1024, 1024, 65536, 65536, 1000, 4194304]
    config = b'MCF2' + u16(2) + u16(1) + repeated(0x11)
    config += u32(0x207fffff) + u64(0) + u16(2) + repeated(0x22)
    config += b''.join(u32(n) for n in limits) + u64(32) + u64(256)
    config += u32(2) + b''.join(descriptors) + u32(0)
    assert len(config) == 342
    context = h(b'MIRROR_CONFIG_V2' + config)
    seed_ids = [h(b'MIRROR_SEED_ID_V2' + context + u32(i) + descriptor)
                for i, descriptor in enumerate(descriptors)]
    unsigned, actors = {}, {}

    def operation(name, kind, actor, anchor, until, sequence, body):
        prefix = u16(2) + bytes([kind]) + context + u64(anchor)
        prefix += repeated(0xA0 + anchor) + u64(anchor + 1) + u64(until)
        prefix += PUBLIC_KEYS[actor] + u64(sequence)
        assert len(prefix) == 131
        unsigned[name], actors[name] = prefix + body, actor

    # Block 1, in exact inclusion order: C, A->C, D, B->D, E.
    operation('enroll_c', 1, 3, 0, 4, 0,
              repeated(0x53) + repeated(0x63) + repeated(0x22))
    c_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_c'])
    operation('approve_a_c', 2, 1, 0, 4, 0, c_id)
    operation('enroll_d', 1, 4, 0, 2, 0,
              repeated(0x54) + repeated(0x64) + repeated(0x22))
    d_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_d'])
    operation('approve_b_d', 2, 2, 0, 2, 0, d_id)
    operation('enroll_e', 1, 5, 0, 5, 0,
              repeated(0x55) + repeated(0x65) + repeated(0x22))
    e_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_e'])
    # Block 2 activates C at the end and withdraws E while still Pending.
    operation('approve_b_c', 2, 2, 1, 4, 1, c_id)
    operation('withdraw_e', 3, 5, 1, 5, 1, e_id)
    # Block 3 is EMPTY; D expires. Block 4 withdraws C and seed A.
    operation('withdraw_c', 3, 3, 3, 8, 1, c_id)
    operation('withdraw_a', 3, 1, 3, 8, 1, seed_ids[0])
    return config, context, descriptors, seed_ids, unsigned, actors


@dataclass(frozen=True)
class Record:
    member_id: bytes
    origin_tag: int
    origin_body: bytes
    status: int
    inclusion_height: int
    inclusion_index: int
    next_sequence: int
    activation: tuple | None
    approvals: tuple = ()

    def encode(self):
        result = self.member_id + bytes([self.origin_tag]) + self.origin_body
        result += bytes([self.status]) + u64(self.inclusion_height)
        result += u32(self.inclusion_index) + u64(self.next_sequence)
        result += b'\x00' if self.activation is None else b'\x01'
        if self.activation is not None:
            result += u64(self.activation[0]) + u64(self.activation[1])
        result += u32(len(self.approvals))
        # Review insertion order intentionally differs from this canonical order.
        for key, height, operation in sorted(self.approvals, key=lambda item: item[0]):
            assert len(key) == 32 and len(operation) == 227
            result += key + u64(height) + operation
        return result


def snapshot(context, counters, records):
    result = b'MPS2' + u16(2) + context
    result += b''.join(u64(number) for number in counters) + u32(len(records))
    assert len(result) == 90
    return result + b''.join(record.encode() for record in
                             sorted(records, key=lambda record: record.member_id))


def reference() -> str:
    config, context, descriptors, seed_ids, unsigned, _ = inputs()
    signed = {name: body + bytes.fromhex(SIGNATURES[name]) for name, body in unsigned.items()}
    assert all(len(body) == (291 if name.startswith('enroll') else 227)
               for name, body in signed.items())
    a = Record(seed_ids[0], 0, u32(0) + descriptors[0], 1, 0, 0, 0, (0, 1))
    b = Record(seed_ids[1], 0, u32(1) + descriptors[1], 1, 0, 1, 0, (0, 2))
    states = [snapshot(context, (0, 2, 0, 2, 2, 0), [a, b])]

    c_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_c'])
    d_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_d'])
    e_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_e'])
    c = Record(c_id, 1, signed['enroll_c'], 0, 1, 0, 1, None,
               ((PUBLIC_KEYS[1], 1, signed['approve_a_c']),))
    d = Record(d_id, 1, signed['enroll_d'], 0, 1, 2, 1, None,
               ((PUBLIC_KEYS[2], 1, signed['approve_b_d']),))
    e = Record(e_id, 1, signed['enroll_e'], 0, 1, 4, 1, None)
    a, b = replace(a, next_sequence=1), replace(b, next_sequence=1)
    states.append(snapshot(context, (1, 2, 3, 2, 2, 2), [a, b, c, d, e]))

    b = replace(b, next_sequence=2)
    c = replace(c, status=1, activation=(2, 3),
                approvals=c.approvals + ((PUBLIC_KEYS[2], 2, signed['approve_b_c']),))
    e = replace(e, status=2, next_sequence=2)
    states.append(snapshot(context, (2, 3, 1, 3, 3, 3), [a, b, c, d, e]))

    d = replace(d, status=3)
    states.append(snapshot(context, (3, 3, 0, 3, 3, 3), [a, b, c, d, e]))

    a, c = replace(a, status=2, next_sequence=2), replace(c, status=2, next_sequence=2)
    states.append(snapshot(context, (4, 1, 0, 3, 3, 3), [a, b, c, d, e]))

    out = ['# Independent synthetic MPS2 fixture; tools/reference_state_v2.py.',
           '# Five snapshots: bootstrap, pending, activation/pending-withdrawal, expiry, active-withdrawals.',
           f'config_hex={config.hex()}', f'context={context.hex()}']
    for index, member_id in enumerate(seed_ids):
        out.append(f'seed{index}_id={member_id.hex()}')
    for name, body in unsigned.items():
        out.append(f'{name}_unsigned={body.hex()}')
        out.append(f'{name}_signing_hash={h(b"MIRROR_OPERATION_V2" + body).hex()}')
        out.append(f'{name}_signed={signed[name].hex()}')
        if name.startswith('enroll'):
            out.append(f'{name}_id={h(b"MIRROR_ENROLLMENT_ID_V2" + body).hex()}')
    for height, state in enumerate(states):
        out.extend([f'state{height}_len={len(state)}', f'state{height}_hex={state.hex()}',
                    f'state{height}_root={h(b"MIRROR_MEMBERSHIP_STATE_V2" + state).hex()}'])
    # Diagnostic replay retains exact grouping, including the empty third block.
    # These hashes are synthetic provenance, not independently mined block hashes.
    groups = [[], ['enroll_c', 'approve_a_c', 'enroll_d', 'approve_b_d', 'enroll_e'],
              ['approve_b_c', 'withdraw_e'], [], ['withdraw_c', 'withdraw_a']]
    replay = b'MPH2' + u16(2) + context + repeated(0xA0) + u32(4)
    assert len(replay) == 74
    for height, names in enumerate(groups):
        operations = u32(len(names)) + b''.join(u32(len(signed[name])) + signed[name]
                                               for name in names)
        group_preimage = b'MIRROR_MEMBERSHIP_REPLAY_GROUP_V2' + context + u64(height) + operations
        out.append(f'block{height}_hash={repeated(0xA0 + height).hex()}')
        out.append(f'group{height}_preimage={group_preimage.hex()}')
        out.append(f'group{height}_commitment={h(group_preimage).hex()}')
        if height:
            replay += u64(height) + repeated(0xA0 + height) + operations
    out.extend([f'replay_len={len(replay)}', f'replay_hex={replay.hex()}'])
    return '\n'.join(out) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    arguments = parser.parse_args()
    result = reference()
    if arguments.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-personhood/tests/vectors/state_v2.txt'
        if path.read_text(encoding='utf-8') != result:
            raise SystemExit('FAIL: independent MPS2 vectors differ; investigate, never bless Rust output.')
        print('Independent MPS2 state and MPH2 grouping vectors (5 snapshots, 4 blocks): PASS')
    else:
        print(result, end='')
