#!/usr/bin/env python3
"""Independent mixed-ledger V2 vectors; no Mirror Rust implementation is called.

The states below are explicit expected results of a small synthetic history,
encoded with Python struct/hashlib. They are not copied from Rust execution.
All Ed25519 signatures were independently generated and verified with Python
cryptography from PUBLIC development seeds [1;32] through [5;32]. The normal
--check path requires only the standard library; --verify-signatures optionally
regenerates and verifies every fixed signature with Python cryptography.

Never use these public keys/seeds for real people, private evidence or funds.
Repeated-byte block hashes are trusted synthetic provenance for executor tests,
not actual mined block hashes or evidence of complete block validation.
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

# Fixed output of independent Python Ed25519, never a Rust-produced expectation.
SIGNATURES = {'seed0': '6cdcbd8e693b92fc1b5a489e850d99db31ea84eb06303549b52eba61b43f16a2c4aecc7ea76b36b6d4e6a5b15795dbc57ed8501931772c9a3aea145d8f49d00a',
 'seed1': '522489c34bd5213869f967ce9118b0c256fa5918507752046a630601fb50ae594558029d346bd8cee6ef42a43833a1da43608ba1aae3a5d8bba4516a7fdcc70f',
 'h1_a_to_b': '73d49758294e3b05832aeea3c7447207abd911dbb9051e22fe6ead7b0737a034219a93b2f01133ffb22ccb8c9f8a92f445581a6e7c60198053ce74131b587a02',
 'h1_a_self': '3cac16a76819552ed60ec5c118f6d21268499aae3f6c2d10e77bb7d8549a3acd5d48f0ee591047f4235555891b5ec0c4e2d2c14946d803bf2f397dda3d12390e',
 'h1_b_to_c': '5a374215cbaa7fdb0f3e7b9136f1a328c94ad9b3d2598706ea59064d398a6aba72476354e424015ddcc2879dbd39753c0f40e2a0fb8a5051fad4db556320ff00',
 'enroll_c': 'df812035b379adb6d24d964cdb3115090485477f70f0a65bdfc42b0217435aa936f6f1ff55f7f73880002032b22ff80f0eee5caead3378e533301d7b20bd4c0c',
 'h1_c_to_a': 'd68f76ec3ec1b79f7a680459395bd0bb8545096fdc19060877fe1ace36bf4034c1f4199d2be51597e28f37ecab5dc0340c6ce94fe2cb53eeed358fe5399e7500',
 'approve_a_c': '741119e53c3b188fc5c217bc0e972c352ff8963d25b0339d94eeb623d10be2f4203793c8b1ed1f5ea35032b1abe79070910f541732fd66ae983d891f7c80660c',
 'enroll_d': 'ad16d446cc01ca1baca6c559858fe92a2ee07a8fb20761875dad76534989ef2707596bd383106768887f996fdcdcc3d6059becdf60d3c39f98cf17bbacfd3c0e',
 'h3_b_self': 'da7283d1033bb3655d8cd2b5bb5017893647c2edeb2fe427cc1f2435dd411a78099c6250dd15223c5973b7313e80d7623871a702ca114d294b5b298b5fc51001',
 'approve_b_c': 'e34019b2bcbfd8b648a492e311612a623c67d8cefd3ef2f8c53939422ac021aee450425ca2500e3e557dddb60c6235cee8316d2123f9944f91d15cf382497909',
 'h3_a_to_c': 'fed91f8b22c15a7760f8b96940b6a8eca6c531a9e800d186c317456f3312a91118575f532cc0367cdd52bf64ae818ccf76613cc77ea289eaa453fd663e526b04',
 'withdraw_c': '229e37557742e4f6e6f6e3be54eec6ec863341185d8a048edfea36f264d1fff3ea773c1b69b9c95a0ff5c0ce56700271d9b76ec1d70126d7d885efaddaaf790e',
 'h4_c_to_b': '70e68dbd7b03f6752e725338f1bf4462e8a3ff4916a7151a10b526140b463101c4c4dfe4642aac3a8aa042bbf523c9f314eb8e4eb4877321c96727cba8d0ff02',
 'withdraw_a': 'df3dff592c31a698c87ecac84e4e6f67c6984db93f967c4e14c364120d691ffee075054744d8b9440bc0861031c460f0b86db56d34aaba0254fed8b947fb720f',
 'h4_a_to_b': '671ce6cc90c1cfaea27709c8766655bc317df146f84735669f2700628145225b7af71306c4ac1d87478db77d3a82130fe3fc3a9583866eee8a603af48dc2f607'}


GROUPS = [
    [],
    ['h1_a_to_b', 'h1_a_self', 'h1_b_to_c', 'enroll_c', 'h1_c_to_a',
     'approve_a_c', 'enroll_d'],
    [],
    ['h3_b_self', 'approve_b_c', 'h3_a_to_c'],
    ['withdraw_c', 'h4_c_to_b', 'withdraw_a', 'h4_a_to_b'],
]


def inputs():
    """MCF2 and operation preimages specified directly from the byte contracts."""
    addresses = {n: h(b'MIRROR_ADDRESS_V1' + key) for n, key in PUBLIC_KEYS.items()}
    descriptors = [PUBLIC_KEYS[n] + repeated(0x30 + n) + repeated(0x40 + n)
                   for n in (1, 2)]
    allocations = sorted([(addresses[1], 1000), (addresses[2], 0), (addresses[5], 0)])
    limits = [1048576, 8, 3, 2, 1024, 1024, 65536, 8, 1000, 4194304]
    config = b'MCF2' + u16(2) + u16(1) + repeated(0x11)
    config += u32(0x207fffff) + u64(0) + u16(2) + repeated(0x22)
    config += b''.join(u32(n) for n in limits) + u64(32) + u64(256)
    config += u32(2) + b''.join(descriptors) + u32(len(allocations))
    config += b''.join(address + u64(balance) for address, balance in allocations)
    assert len(config) == 462
    context = h(b'MIRROR_CONFIG_V2' + config)
    seed_ids = [h(b'MIRROR_SEED_ID_V2' + context + u32(i) + descriptor)
                for i, descriptor in enumerate(descriptors)]
    seed_hashes = [h(b'MIRROR_SEED_V2' + context + u32(i) + descriptor)
                   for i, descriptor in enumerate(descriptors)]
    unsigned, actors = {}, {}

    def operation(name, kind, actor, anchor, until, sequence, body):
        prefix = u16(2) + bytes([kind]) + context + u64(anchor)
        prefix += repeated(0xA0 + anchor) + u64(anchor + 1) + u64(until)
        prefix += PUBLIC_KEYS[actor] + u64(sequence)
        assert len(prefix) == 131
        unsigned[name], actors[name] = prefix + body, actor

    def transfer(name, actor, recipient, anchor, sequence, amount):
        operation(name, 0, actor, anchor, 8, sequence,
                  addresses[recipient] + u64(amount) + u64(0))

    transfer('h1_a_to_b', 1, 2, 0, 0, 100)
    transfer('h1_a_self', 1, 1, 0, 1, 10)
    transfer('h1_b_to_c', 2, 3, 0, 0, 40)
    operation('enroll_c', 1, 3, 0, 5, 0,
              repeated(0x53) + repeated(0x63) + repeated(0x22))
    c_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_c'])
    transfer('h1_c_to_a', 3, 1, 0, 0, 7)
    operation('approve_a_c', 2, 1, 0, 5, 0, c_id)
    operation('enroll_d', 1, 4, 0, 1, 0,
              repeated(0x54) + repeated(0x64) + repeated(0x22))
    # h2 is empty and expires D before any operations run.
    transfer('h3_b_self', 2, 2, 2, 1, 5)
    operation('approve_b_c', 2, 2, 2, 5, 0, c_id)
    transfer('h3_a_to_c', 1, 3, 2, 2, 17)
    operation('withdraw_c', 3, 3, 3, 8, 1, c_id)
    transfer('h4_c_to_b', 3, 2, 3, 1, 36)
    operation('withdraw_a', 3, 1, 3, 8, 1, seed_ids[0])
    transfer('h4_a_to_b', 1, 2, 3, 3, 499)
    assert list(unsigned) == [name for group in GROUPS for name in group]
    return config, context, descriptors, seed_ids, seed_hashes, addresses, unsigned, actors


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


def account_bytes(addresses, balances):
    # Explicit zero entries must survive; an absent entry is a different state.
    entries = sorted((addresses[n], balance, sequence)
                     for n, (balance, sequence) in balances.items())
    assert len({address for address, _, _ in entries}) == len(entries)
    assert sum(balance for _, balance, _ in entries) == 1000
    return u32(len(entries)) + b''.join(address + u64(balance) + u64(sequence)
                                      for address, balance, sequence in entries)


def signature_inputs():
    _, _, _, _, seed_hashes, _, unsigned, actors = inputs()
    items = {f'seed{i}': (i + 1, digest) for i, digest in enumerate(seed_hashes)}
    items.update({name: (actors[name], h(b'MIRROR_OPERATION_V2' + body))
                  for name, body in unsigned.items()})
    return items


def verify_signatures():
    """Optional, independently regenerate and verify all sixteen fixed proofs."""
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

    items = signature_inputs()
    assert set(SIGNATURES) == set(items)
    for name, (actor, message) in items.items():
        private_key = Ed25519PrivateKey.from_private_bytes(repeated(actor))
        public_key = private_key.public_key()
        assert public_key.public_bytes(Encoding.Raw, PublicFormat.Raw) == PUBLIC_KEYS[actor]
        fixed = bytes.fromhex(SIGNATURES[name])
        assert private_key.sign(message) == fixed, name
        public_key.verify(fixed, message)


def reference() -> str:
    config, context, descriptors, seed_ids, seed_hashes, addresses, unsigned, _ = inputs()
    signed = {name: body + bytes.fromhex(SIGNATURES[name]) for name, body in unsigned.items()}
    assert all(len(body) == {0: 243, 1: 291, 2: 227, 3: 227}[body[2]]
               for body in signed.values())

    # Explicit expected membership states, independently described at each height.
    a = Record(seed_ids[0], 0, u32(0) + descriptors[0], 1, 0, 0, 0, (0, 1))
    b = Record(seed_ids[1], 0, u32(1) + descriptors[1], 1, 0, 1, 0, (0, 2))
    states = [snapshot(context, (0, 2, 0, 2, 2, 0), [a, b])]
    c_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_c'])
    d_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_d'])
    c = Record(c_id, 1, signed['enroll_c'], 0, 1, 3, 1, None,
               ((PUBLIC_KEYS[1], 1, signed['approve_a_c']),))
    d = Record(d_id, 1, signed['enroll_d'], 0, 1, 6, 1, None)
    a = replace(a, next_sequence=1)
    states.append(snapshot(context, (1, 2, 2, 2, 2, 1), [a, b, c, d]))
    alternate = snapshot(context, (1, 2, 2, 2, 2, 1),
                         [a, b, replace(c, inclusion_index=4), d])

    d = replace(d, status=3)
    states.append(snapshot(context, (2, 2, 1, 2, 2, 1), [a, b, c, d]))
    b = replace(b, next_sequence=1)
    c = replace(c, status=1, activation=(3, 3),
                approvals=c.approvals + ((PUBLIC_KEYS[2], 3, signed['approve_b_c']),))
    states.append(snapshot(context, (3, 3, 0, 3, 3, 2), [a, b, c, d]))
    a, c = replace(a, status=2, next_sequence=2), replace(c, status=2, next_sequence=2)
    states.append(snapshot(context, (4, 1, 0, 3, 3, 2), [a, b, c, d]))

    balances = [
        {1: (1000, 0), 2: (0, 0), 5: (0, 0)},
        {1: (907, 2), 2: (60, 1), 3: (33, 1), 5: (0, 0)},
        {1: (907, 2), 2: (60, 1), 3: (33, 1), 5: (0, 0)},
        {1: (890, 3), 2: (60, 2), 3: (50, 1), 5: (0, 0)},
        {1: (391, 4), 2: (595, 2), 3: (14, 2), 5: (0, 0)},
    ]
    economics = u64(1000)
    economics_root = h(b'MIRROR_ECONOMICS_DISABLED_V2' + economics)
    out = ['# Independent synthetic mixed-ledger V2 fixture; tools/reference_ledger_v2.py.',
           '# PUBLIC keys; 1000 Nusa total, explicit zero allocations, no issuance or rewards.',
           '# Five states include full operation indices, an empty expiry block and withdrawals.',
           f'config_len={len(config)}', f'config_hex={config.hex()}',
           f'context={context.hex()}', 'initial_test_supply=1000']
    for actor, address in addresses.items():
        out.append(f'address_{chr(96 + actor)}={address.hex()}')
    for index, member_id in enumerate(seed_ids):
        out.extend([f'seed{index}_id={member_id.hex()}',
                    f'seed{index}_signing_hash={seed_hashes[index].hex()}',
                    f'seed{index}_signature={SIGNATURES[f"seed{index}"]}'])
    for name, body in unsigned.items():
        out.extend([f'{name}_unsigned={body.hex()}',
                    f'{name}_signing_hash={h(b"MIRROR_OPERATION_V2" + body).hex()}',
                    f'{name}_signed={signed[name].hex()}',
                    f'{name}_id={h(b"MIRROR_OPERATION_ID_V2" + signed[name]).hex()}'])
        if name.startswith('enroll'):
            out.append(f'{name}_member_id={h(b"MIRROR_ENROLLMENT_ID_V2" + body).hex()}')
    out.extend([f'economics_hex={economics.hex()}',
                f'economics_root={economics_root.hex()}'])

    def add_state(label, height, account_state, membership):
        accounts = account_bytes(addresses, account_state)
        accounts_root = h(b'MIRROR_ACCOUNTS_V2' + accounts)
        membership_root = h(b'MIRROR_MEMBERSHIP_STATE_V2' + membership)
        combined = context + u64(height) + accounts_root + membership_root + economics_root
        assert len(combined) == 136
        out.extend([f'{label}_height={height}', f'{label}_account_count={len(account_state)}',
                    f'{label}_accounts_hex={accounts.hex()}',
                    f'{label}_accounts_root={accounts_root.hex()}',
                    f'{label}_membership_len={len(membership)}',
                    f'{label}_membership_hex={membership.hex()}',
                    f'{label}_membership_root={membership_root.hex()}',
                    f'{label}_ledger_preimage={combined.hex()}',
                    f'{label}_ledger_root={h(b"MIRROR_LEDGER_STATE_V2" + combined).hex()}'])

    for height, (account_state, membership) in enumerate(zip(balances, states)):
        add_state(f'state{height}', height, account_state, membership)
        out.append(f'block{height}_hash={repeated(0xA0 + height).hex()}')
        out.append(f'block{height}_operations={",".join(GROUPS[height])}')
    add_state('alternate1', 1, balances[1], alternate)
    alt_group = GROUPS[1].copy()
    alt_group[3], alt_group[4] = alt_group[4], alt_group[3]
    out.append(f'alternate1_operations={",".join(alt_group)}')
    return '\n'.join(out) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--verify-signatures', action='store_true')
    arguments = parser.parse_args()
    if arguments.verify_signatures:
        verify_signatures()
    result = reference()
    if arguments.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-state/tests/vectors/ledger_v2.txt'
        if path.read_text(encoding='utf-8') != result:
            raise SystemExit('FAIL: independent ledger vectors differ; investigate, never bless Rust output.')
        print('Independent V2 ledger vectors (5 states, mixed operations, expiry, alternate index): PASS')
        if arguments.verify_signatures:
            print('Independent Python Ed25519 generation and verification (16 signatures): PASS')
    else:
        print(result, end='')
