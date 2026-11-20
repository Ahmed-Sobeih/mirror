#!/usr/bin/env python3
"""Independent B2 byte/hash reference: no Mirror Rust code is called.

Known public fixture key is Ed25519 seed [1;32] (never use for real assets).
Signature constants were generated independently with Python cryptography's
Ed25519 implementation. Standard-library CI recomputes ALL preimages/hashes/IDs;
Rust tests separately sign and strictly verify these independent signature vectors.
This is a synthetic message test, not a biometric or live-chain experiment.
"""
import argparse
import hashlib
import struct
from pathlib import Path


def sha256d(data: bytes) -> bytes:
    return hashlib.sha256(hashlib.sha256(data).digest()).digest()


def reference() -> str:
    u16 = lambda n: struct.pack('<H', n)
    u32 = lambda n: struct.pack('<I', n)
    u64 = lambda n: struct.pack('<Q', n)
    pk = bytes.fromhex('8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c')
    config = b'MCF2' + u16(2) + u16(1) + bytes([0x11])*32
    config += u32(0x207fffff) + u64(0) + u16(1) + bytes([0x22])*32
    config += b''.join(u32(n) for n in [1048576,256,256,64,1024,1024,65536,65536,1000,4194304])
    config += u64(32) + u64(256) + u32(1) + pk + bytes([3])*32 + bytes([4])*32 + u32(0)
    context = sha256d(b'MIRROR_CONFIG_V2' + config)
    out = {'config': config.hex(), 'context': context.hex(), 'public_key': pk.hex()}
    signatures = [
        '9170fdaa8cd392dbb43029c5ccb690258b695c98c4d5724199f5cb06aadafa342ff2646022efc082112cb595610afd11ac1d69e860478d730592378da5e61407',
        '56cc027d366064b6c9ec428948b7155140633b9db251aaf81c46961942fa3d68d8448b7814bce0c8cd75eb9c4642b9cf6d99c5189b0c9dd5766b37dbfe9b440f',
        'b0f95ce74b225c0670cb0d87157b7721590371f18033c8e964272588bb62538ff5839aa2e06ef01957eb1056d979f92c1a24fb9f4c53af99d3f9d43265b95007',
    ]
    bodies = [bytes([0x55])*32 + bytes([0x66])*32 + bytes([0x22])*32, bytes([0x77])*32, bytes([0x88])*32]
    for kind, (name, body, sighex) in enumerate(zip(['enroll','approve','withdraw'], bodies, signatures), 1):
        prefix = u16(2) + bytes([kind]) + context + u64(7) + bytes([0x44])*32
        prefix += u64(8) + u64(24) + pk + u64(0 if kind == 1 else 9)
        assert len(prefix) == 131
        unsigned = prefix + body
        digest = sha256d(b'MIRROR_OPERATION_V2' + unsigned)
        signature = bytes.fromhex(sighex)
        signed = unsigned + signature
        assert len(signed) == (291 if kind == 1 else 227)
        for suffix, value in [('unsigned', unsigned), ('hash', digest), ('signature', signature),
                              ('signed', signed), ('id', sha256d(b'MIRROR_OPERATION_ID_V2' + signed))]:
            out[f'{name}_{suffix}'] = value.hex()
        if kind == 1:
            out['enrollment_id'] = sha256d(b'MIRROR_ENROLLMENT_ID_V2' + unsigned).hex()
    return ''.join(f'{name}={value}\n' for name, value in out.items())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    result = reference()
    if args.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-personhood/tests/vectors/operations_v2.txt'
        if path.read_text(encoding='utf-8') != result:
            raise SystemExit('FAIL: independent V2 operation vectors differ; do not bless Rust output.')
        print('Independent V2 operation preimages/hash/ID vectors: PASS')
    else:
        print(result, end='')
