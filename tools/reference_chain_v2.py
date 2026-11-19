#!/usr/bin/env python3
"""Independent MB-02C actual MRB2 history, manifest and state vectors.

Every block hash is mined from its complete domain-separated header. Runtime
signatures use those actual ancestor hashes, never synthetic replacement tags.
Expected states are explicitly described below, independently of Mirror Rust.
Only the standard library is required for normal --check. Public development
seeds [1;32] through [5;32] are deliberately non-secret and never real identities.
--verify-signatures independently regenerates and verifies all fixed Ed25519
proofs with Python cryptography. --generate-signatures is fixture-author tooling;
it prints independently generated proofs and must never consume Rust output.

These vectors exercise actual synthetic development block validation, not a
public mainnet, durable canonical-tip authority, biological uniqueness or actual
currency issuance.
"""
import argparse
from dataclasses import dataclass, replace
import hashlib
import json
from pathlib import Path
import struct


def h(value: bytes) -> bytes:
    return hashlib.sha256(hashlib.sha256(value).digest()).digest()


def u16(value: int) -> bytes:
    return struct.pack('<H', value)


def u32(value: int) -> bytes:
    return struct.pack('<I', value)


def u64(value: int) -> bytes:
    return struct.pack('<Q', value)


def repeated(value: int) -> bytes:
    return bytes([value]) * 32


PUBLIC_KEYS = {
    1: bytes.fromhex('8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c'),
    2: bytes.fromhex('8139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b394'),
    3: bytes.fromhex('ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d1'),
    4: bytes.fromhex('ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c'),
    5: bytes.fromhex('6e7a1cdd29b0b78fd13af4c5598feff4ef2a97166e3ca6f2e4fbfccd80505bf1'),
}
BITS = 0x207fffff
TARGET = 0x7fffff << (8 * (0x20 - 3))
SIGNATURES = {
    'seed0': '6cdcbd8e693b92fc1b5a489e850d99db31ea84eb06303549b52eba61b43f16a2c4aecc7ea76b36b6d4e6a5b15795dbc57ed8501931772c9a3aea145d8f49d00a',
    'seed1': '522489c34bd5213869f967ce9118b0c256fa5918507752046a630601fb50ae594558029d346bd8cee6ef42a43833a1da43608ba1aae3a5d8bba4516a7fdcc70f',
    'h1_a_to_b': '4861b5c6bcee92b63b11fc47f6a28ee4d6deae82adfb56697a00fd3976bd264e4d7ab51891c5177372449cdf1edae024100496402d6f5c4fe6072ab0618df307',
    'h1_a_self': 'e4674475a815487a97e21ea761c83a30fc3c6300c9e1d6c11a25c39ebfbd12815065dbfed049efb4dee1cab7806aea86b18c7c4c437faa071aa71ddcbc541300',
    'h1_b_to_c': '34f15462f8d0b10f1fbf52edb7d6a1689bfcbe76f1e38d8f0567d2551ba783f8e2656e933c15bd045273f3b0901dc8f0455625f67ca3494e712900bbcce6e600',
    'enroll_c': '7a1169f170852a14d709928f22730d5b57ad65f7f0c73072c2cb0d140e5f1ad5d9f9b721c9c603064baa3ac81aefd6b58bb87bd17e6b025474839aa1507d910f',
    'h1_c_to_a': '2305db94852c2a4a7820f633bb0d77c99c4621941e98727b325c13076116ff4355acad5f7513cda65537ce6bc58b57b8469a2e8bba164f36a4904dfca8de2f09',
    'approve_a_c': '86e935aea47d9d5f0bb37c404270beb6a69028ac8f2b45ba0400db33720b6ef21343e71ad416186ba36e174c668ec6e420629555fe60e157501afa5ee868f10f',
    'enroll_d': '493a1bd2507c2b0384e22dd0449644b35a5202c6a92b1f9c7f024e9d7597ff7769793fe17599b7a369fd44e25df158e95f56cb82067304d963643a9f7c30c505',
    'h3_b_self': '4671d70d107eacf013f316b1b25420f350fbd3f121ad0b060cb6c6b30458a97068b57eb33a30e26359cc281e47a33bfa219e11d003d8eaf59658ac81ca162109',
    'approve_b_c': '89d3a8eb2b2739d4eab5fdab6c686fb68070f29eef80810894858715a12f487078038c379fb635ce86493b2077e3140147009306c13296ba66e87f76999e5903',
    'h3_a_to_c': 'ab04f2ed25aea3510e7764590b4802109376393be5302063d7815c73e82d80fd3e41deb64c93cd444bccff8afb1ef2d2daa309a65e33c58b62a0168c72955f0b',
    'withdraw_c': '57ba0546d629a3b5abd5a68a66409408578beed5feee29cdaacd6bb5aa830833fbf79eec79ed84c1171a4f5c28a495b63c65c0917b07115f45bfeb82153b9805',
    'h4_c_to_b': '5c49a0f60105cabe1c3612d1cbe6e5f11f6d73e513cd3caa694a93c95c4938dfdbbe6caad2ab97a19482548361996ee2a363d6b76a6619b8fb110fc67087f004',
    'withdraw_a': 'a63bc65db54ab5700ec72b0dfbd1be0cffb06f085f443d7b18868df3b07c5f2c25318198b1cc63329575e0fd17c861b1fde4206b9addd87712a13bdb9f60f80d',
    'h4_a_to_b': '27a333fde6218b3243bd29c156f79a601b64ce2f079f637a90cbc2d76102465af559f4e38c6c5ef02bc06bd3986002a8d48b1c9c95d54cc5e09d2c1ad0621001',
    'fork2_withdraw_a': 'ea7245bff71a36e86039dedb749325003a121faa8a5c09022512ac2b89bf25e15e305a53f8ec6274be8ea6a4615f4fc9928ae5d1746a03555ae795233216d20e',
    'fork3_b_self': '11f551da0003f9f7b636079d3ef05e42bca9f7ba39646d192803bde6048d8a027ace7260f43b3ced81109e4f878c9f3b1226f7ffa8355f4160403f752597900e',
    'fork3_approve_b_c': 'dfb8cb928b94e61b639b772f3e2cdcf12ecab3e19d5a97f849cd608d3c2a9837bc6ec08225f0452ba389446b8e2f2c72ca666b0194ace549bdf36b5d60d9d40c',
    'fork3_a_to_c': '59d87f495b60d6e4c378add0c7e90adcdf3e8f145ea4a43977da299fdb4c798ca01b71b1824837f9a58647f68048a7b97f57dea9a437087e94c6751e3f94e30e',
}


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

    def encode(self) -> bytes:
        result = self.member_id + bytes([self.origin_tag]) + self.origin_body
        result += bytes([self.status]) + u64(self.inclusion_height)
        result += u32(self.inclusion_index) + u64(self.next_sequence)
        result += b'\x00' if self.activation is None else b'\x01'
        if self.activation is not None:
            result += u64(self.activation[0]) + u64(self.activation[1])
        result += u32(len(self.approvals))
        for key, height, operation in sorted(self.approvals, key=lambda item: item[0]):
            assert len(operation) == 227
            result += key + u64(height) + operation
        return result


def operation_root(operations: list[bytes]) -> bytes:
    level = [h(b'MIRROR_OP_LEAF_V2' + u32(len(op)) + op) for op in operations]
    if not level:
        inner = h(b'')
    else:
        while len(level) > 1:
            if len(level) % 2:
                level.append(level[-1])
            level = [h(level[i] + level[i + 1]) for i in range(0, len(level), 2)]
        inner = level[0]
    return h(b'MIRROR_OP_ROOT_V2' + u32(len(operations)) + inner)


def reference(generate=False, verify=False):
    output, generated, signed, unsigned = {}, {}, {}, {}
    addresses = {actor: h(b'MIRROR_ADDRESS_V1' + key) for actor, key in PUBLIC_KEYS.items()}
    descriptors = [PUBLIC_KEYS[n] + repeated(0x30 + n) + repeated(0x40 + n)
                   for n in (1, 2)]
    allocations = sorted([(addresses[1], 1000), (addresses[2], 0), (addresses[5], 0)])
    limits = [1048576, 8, 3, 2, 1024, 1024, 65536, 8, 1000, 4194304]
    config = b'MCF2' + u16(2) + u16(1) + repeated(0x11)
    config += u32(BITS) + u64(0) + u16(2) + repeated(0x22)
    config += b''.join(u32(n) for n in limits) + u64(32) + u64(256)
    config += u32(2) + b''.join(descriptors) + u32(len(allocations))
    config += b''.join(address + u64(balance) for address, balance in allocations)
    assert len(config) == 462
    context = h(b'MIRROR_CONFIG_V2' + config)
    seed_ids = [h(b'MIRROR_SEED_ID_V2' + context + u32(i) + descriptor)
                for i, descriptor in enumerate(descriptors)]
    seed_hashes = [h(b'MIRROR_SEED_V2' + context + u32(i) + descriptor)
                   for i, descriptor in enumerate(descriptors)]
    output.update(config_hex=config.hex(), config_len=str(len(config)),
                  context=context.hex(), target=TARGET.to_bytes(32, 'big').hex(),
                  pow_bits=f'{BITS:08x}', initial_test_supply='1000')
    for actor, address in addresses.items():
        output[f'address_{chr(96 + actor)}'] = address.hex()

    def proof(name, actor, message):
        if generate or verify:
            from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
            from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
            key = Ed25519PrivateKey.from_private_bytes(repeated(actor))
            assert key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw) == PUBLIC_KEYS[actor]
            value = key.sign(message)
            key.public_key().verify(value, message)
            if not generate:
                assert value.hex() == SIGNATURES[name], name
        else:
            value = bytes.fromhex(SIGNATURES[name])
        assert len(value) == 64
        generated[name] = value.hex()
        return value

    seed_proofs = [proof(f'seed{i}', i + 1, digest) for i, digest in enumerate(seed_hashes)]
    for i in range(2):
        output[f'seed{i}_id'] = seed_ids[i].hex()
        output[f'seed{i}_signing_hash'] = seed_hashes[i].hex()
        output[f'seed{i}_signature'] = seed_proofs[i].hex()

    def operation(name, kind, actor, anchor_height, anchor_hash, until, sequence, body):
        prefix = u16(2) + bytes([kind]) + context + u64(anchor_height) + anchor_hash
        prefix += u64(anchor_height + 1) + u64(until) + PUBLIC_KEYS[actor] + u64(sequence)
        assert len(prefix) == 131
        unsigned[name] = prefix + body
        digest = h(b'MIRROR_OPERATION_V2' + unsigned[name])
        signed[name] = unsigned[name] + proof(name, actor, digest)
        assert len(signed[name]) == {0: 243, 1: 291, 2: 227, 3: 227}[kind]
        output[f'{name}_unsigned'] = unsigned[name].hex()
        output[f'{name}_signing_hash'] = digest.hex()
        output[f'{name}_signed'] = signed[name].hex()
        output[f'{name}_id'] = h(b'MIRROR_OPERATION_ID_V2' + signed[name]).hex()
        return signed[name]

    def transfer(name, actor, recipient, anchor_height, anchor_hash, sequence, amount):
        return operation(name, 0, actor, anchor_height, anchor_hash, 8, sequence,
                         addresses[recipient] + u64(amount) + u64(0))

    economics_root = h(b'MIRROR_ECONOMICS_DISABLED_V2' + u64(1000))
    output['economics_root'] = economics_root.hex()

    def state(label, counters, records, balances):
        membership = b'MPS2' + u16(2) + context
        membership += b''.join(u64(n) for n in counters) + u32(len(records))
        assert len(membership) == 90
        membership += b''.join(record.encode() for record in sorted(records, key=lambda r: r.member_id))
        entries = sorted((addresses[n], balance, sequence) for n, (balance, sequence) in balances.items())
        assert sum(balance for _, balance, _ in entries) == 1000
        accounts = u32(len(entries)) + b''.join(address + u64(balance) + u64(sequence)
                                              for address, balance, sequence in entries)
        accounts_root = h(b'MIRROR_ACCOUNTS_V2' + accounts)
        membership_root = h(b'MIRROR_MEMBERSHIP_STATE_V2' + membership)
        combined = context + u64(counters[0]) + accounts_root + membership_root + economics_root
        assert len(combined) == 136
        ledger_root = h(b'MIRROR_LEDGER_STATE_V2' + combined)
        for name, value in [('accounts_hex', accounts), ('membership_hex', membership),
                            ('accounts_root', accounts_root), ('membership_root', membership_root),
                            ('ledger_preimage', combined), ('ledger_root', ledger_root)]:
            output[f'{label}_{name}'] = value.hex()
        return ledger_root

    def block(label, height, previous, operations, ledger_root, timestamp, start_nonce=0):
        root = operation_root(operations)
        fixed_header = u32(2) + context + u64(height) + previous + root + ledger_root
        fixed_header += u64(timestamp) + u32(BITS)
        assert len(fixed_header) == 152
        for nonce in range(start_nonce, start_nonce + 1_000_000):
            header = fixed_header + u64(nonce)
            block_hash = h(b'MIRROR_BLOCK_V2' + header)
            if int.from_bytes(block_hash, 'big') <= TARGET:
                break
        else:
            raise AssertionError('independent fixture mining budget exhausted')
        encoded = b'MRB2' + header + u32(len(operations))
        encoded += b''.join(u32(len(op)) + op for op in operations)
        assert len(encoded) == 168 + sum(4 + len(op) for op in operations)
        output[f'{label}_height'] = str(height)
        output[f'{label}_timestamp'] = str(timestamp)
        output[f'{label}_nonce'] = str(nonce)
        output[f'{label}_operations_root'] = root.hex()
        output[f'{label}_header'] = header.hex()
        output[f'{label}_hash'] = block_hash.hex()
        output[f'{label}_block'] = encoded.hex()
        return block_hash, nonce

    a = Record(seed_ids[0], 0, u32(0) + descriptors[0], 1, 0, 0, 0, (0, 1))
    b = Record(seed_ids[1], 0, u32(1) + descriptors[1], 1, 0, 1, 0, (0, 2))
    balance0 = {1: (1000, 0), 2: (0, 0), 5: (0, 0)}
    ledger0 = state('state0', (0, 2, 0, 2, 2, 0), [a, b], balance0)
    genesis, nonce0 = block('block0', 0, bytes(32), [], ledger0, 0)
    alternate, _ = block('alternate_genesis', 0, bytes(32), [], ledger0, 0, nonce0 + 1)
    assert alternate != genesis
    manifest = b'MNM2' + u16(2) + u32(len(config)) + config + u32(len(seed_proofs))
    manifest += b''.join(seed_proofs) + genesis
    assert len(manifest) == 46 + len(config) + 64 * len(seed_proofs) == 636
    output['manifest_hex'] = manifest.hex()
    output['manifest_len'] = str(len(manifest))
    output['manifest_config_hash'] = context.hex()
    output['manifest_genesis_hash'] = genesis.hex()

    transfer('h1_a_to_b', 1, 2, 0, genesis, 0, 100)
    transfer('h1_a_self', 1, 1, 0, genesis, 1, 10)
    transfer('h1_b_to_c', 2, 3, 0, genesis, 0, 40)
    operation('enroll_c', 1, 3, 0, genesis, 5, 0,
              repeated(0x53) + repeated(0x63) + repeated(0x22))
    c_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_c'])
    transfer('h1_c_to_a', 3, 1, 0, genesis, 0, 7)
    operation('approve_a_c', 2, 1, 0, genesis, 5, 0, c_id)
    operation('enroll_d', 1, 4, 0, genesis, 1, 0,
              repeated(0x54) + repeated(0x64) + repeated(0x22))
    d_id = h(b'MIRROR_ENROLLMENT_ID_V2' + unsigned['enroll_d'])
    output['c_member_id'], output['d_member_id'] = c_id.hex(), d_id.hex()
    names1 = ['h1_a_to_b', 'h1_a_self', 'h1_b_to_c', 'enroll_c', 'h1_c_to_a', 'approve_a_c', 'enroll_d']
    ops1 = [signed[name] for name in names1]
    a = replace(a, next_sequence=1)
    c = Record(c_id, 1, signed['enroll_c'], 0, 1, 3, 1, None,
               ((PUBLIC_KEYS[1], 1, signed['approve_a_c']),))
    d = Record(d_id, 1, signed['enroll_d'], 0, 1, 6, 1, None)
    balance1 = {1: (907, 2), 2: (60, 1), 3: (33, 1), 5: (0, 0)}
    ledger1 = state('state1', (1, 2, 2, 2, 2, 1), [a, b, c, d], balance1)
    hash1, _ = block('block1', 1, genesis, ops1, ledger1, 1)
    wrong_root = bytes([ledger1[0] ^ 1]) + ledger1[1:]
    block('wrong_root1', 1, genesis, ops1, wrong_root, 1)

    d = replace(d, status=3)
    ledger2 = state('state2', (2, 2, 1, 2, 2, 1), [a, b, c, d], balance1)
    hash2, _ = block('block2', 2, hash1, [], ledger2, 2)
    transfer('h3_b_self', 2, 2, 2, hash2, 1, 5)
    operation('approve_b_c', 2, 2, 2, hash2, 5, 0, c_id)
    transfer('h3_a_to_c', 1, 3, 2, hash2, 2, 17)
    b3 = replace(b, next_sequence=1)
    c3 = replace(c, status=1, activation=(3, 3),
                 approvals=c.approvals + ((PUBLIC_KEYS[2], 3, signed['approve_b_c']),))
    balance3 = {1: (890, 3), 2: (60, 2), 3: (50, 1), 5: (0, 0)}
    ledger3 = state('state3', (3, 3, 0, 3, 3, 2), [a, b3, c3, d], balance3)
    names3 = ['h3_b_self', 'approve_b_c', 'h3_a_to_c']
    ops3 = [signed[name] for name in names3]
    hash3, _ = block('block3', 3, hash2, ops3, ledger3, 3)

    operation('withdraw_c', 3, 3, 3, hash3, 8, 1, c_id)
    transfer('h4_c_to_b', 3, 2, 3, hash3, 1, 36)
    operation('withdraw_a', 3, 1, 3, hash3, 8, 1, seed_ids[0])
    transfer('h4_a_to_b', 1, 2, 3, hash3, 3, 499)
    a4, c4 = replace(a, status=2, next_sequence=2), replace(c3, status=2, next_sequence=2)
    balance4 = {1: (391, 4), 2: (595, 2), 3: (14, 2), 5: (0, 0)}
    ledger4 = state('state4', (4, 1, 0, 3, 3, 2), [a4, b3, c4, d], balance4)
    names4 = ['withdraw_c', 'h4_c_to_b', 'withdraw_a', 'h4_a_to_b']
    block('block4', 4, hash3, [signed[name] for name in names4], ledger4, 4)

    # A genuine membership fork shares the entire accepted prefix through h1.
    operation('fork2_withdraw_a', 3, 1, 1, hash1, 8, 1, seed_ids[0])
    af = replace(a, status=2, next_sequence=2)
    ledger_f2 = state('fork_state2', (2, 1, 1, 2, 2, 1), [af, b, c, d], balance1)
    hash_f2, _ = block('fork2', 2, hash1, [signed['fork2_withdraw_a']], ledger_f2, 2)
    transfer('fork3_b_self', 2, 2, 2, hash_f2, 1, 5)
    operation('fork3_approve_b_c', 2, 2, 2, hash_f2, 5, 0, c_id)
    transfer('fork3_a_to_c', 1, 3, 2, hash_f2, 2, 17)
    cf = replace(c, approvals=c.approvals + ((PUBLIC_KEYS[2], 3, signed['fork3_approve_b_c']),))
    ledger_f3 = state('fork_state3', (3, 1, 1, 2, 2, 2), [af, b3, cf, d], balance3)
    names_f3 = ['fork3_b_self', 'fork3_approve_b_c', 'fork3_a_to_c']
    block('fork3', 3, hash_f2, [signed[name] for name in names_f3], ledger_f3, 3)
    # Correct parent/work/body, but signed anchors belong to the other branch.
    block('cross_anchor3', 3, hash_f2, ops3, ledger_f3, 3)
    assert ledger_f3 != ledger3
    assert output['fork_state3_accounts_hex'] == output['state3_accounts_hex']
    output.update(block0_operations='', block1_operations=','.join(names1), block2_operations='',
                  block3_operations=','.join(names3), block4_operations=','.join(names4),
                  fork2_operations='fork2_withdraw_a', fork3_operations=','.join(names_f3))
    if not generate:
        assert generated == SIGNATURES, 'missing or unused fixed signatures'
    return output, generated


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--verify-signatures', action='store_true')
    parser.add_argument('--generate-signatures', action='store_true')
    args = parser.parse_args()
    values, signatures = reference(args.generate_signatures, args.verify_signatures)
    if args.generate_signatures:
        print(json.dumps(signatures, indent=2))
        return
    result = '# Independent actual synthetic V2 chain; tools/reference_chain_v2.py.\n'
    result += '# PUBLIC development keys; no biometric data or real money.\n'
    result += ''.join(f'{key}={value}\n' for key, value in values.items())
    if args.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-chain/tests/vectors/chain_v2.txt'
        if path.read_text(encoding='utf-8') != result:
            raise SystemExit('FAIL: independent chain vectors differ; investigate, never bless Rust output.')
        print('Independent V2 actual-chain vectors (manifest, five main blocks, membership fork, authority failures): PASS')
        if args.verify_signatures:
            print(f'Independent Python Ed25519 generation and verification ({len(signatures)} signatures): PASS')
    else:
        print(result, end='')


if __name__ == '__main__':
    main()
