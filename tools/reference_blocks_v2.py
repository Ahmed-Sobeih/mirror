#!/usr/bin/env python3
"""Independent MB-02A V2 transfer/header/block byte and hash vectors.

Only Python's standard library is required by this committed checker. The five
signature constants were independently generated AND verified using Python
cryptography Ed25519 and public test seed [1;32], before any Rust codec comparison.
The three membership signatures deliberately preserve the existing B2 vectors.
No Mirror Rust code or Rust-produced expected value is used here.

All fields are synthetic. These are structural/signature fixtures, not executable
ledger history, pinned genesis, proof of work, real money or real membership.
"""
import argparse
import hashlib
import struct
from pathlib import Path

PUBLIC_KEY_1 = bytes.fromhex(
    "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c"
)
PUBLIC_KEY_2 = bytes.fromhex(
    "8139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b394"
)
SIGNATURES = {
    "transfer": "b839706761007fe700910df7957c063c3985b3a21f95b40f5330ed7c70c11249c97f2aaad95bfd1199a9072e27c2061d71757e4d7fd32240ef871d374687a304",
    "transfer_max": "b3e3c08c982885b9e6c3350058aeeac100f39106e37112b8c97baec6df0e868bbdd39cf565e40d23381aee858660a9b21892ccadd9db727ede735e8753d86b0a",
    "enroll": "9170fdaa8cd392dbb43029c5ccb690258b695c98c4d5724199f5cb06aadafa342ff2646022efc082112cb595610afd11ac1d69e860478d730592378da5e61407",
    "approve": "56cc027d366064b6c9ec428948b7155140633b9db251aaf81c46961942fa3d68d8448b7814bce0c8cd75eb9c4642b9cf6d99c5189b0c9dd5766b37dbfe9b440f",
    "withdraw": "b0f95ce74b225c0670cb0d87157b7721590371f18033c8e964272588bb62538ff5839aa2e06ef01957eb1056d979f92c1a24fb9f4c53af99d3f9d43265b95007"
}


def u16(value: int) -> bytes:
    return struct.pack("<H", value)


def u32(value: int) -> bytes:
    return struct.pack("<I", value)


def u64(value: int) -> bytes:
    return struct.pack("<Q", value)


def sha256d(value: bytes) -> bytes:
    return hashlib.sha256(hashlib.sha256(value).digest()).digest()


def configuration() -> bytes:
    # Deliberately the exact independently specified B2 one-seed configuration.
    limits = [1048576, 256, 256, 64, 1024, 1024, 65536, 65536, 1000, 4194304]
    encoded = b"MCF2" + u16(2) + u16(1) + bytes([0x11]) * 32
    encoded += u32(0x207fffff) + u64(0) + u16(1) + bytes([0x22]) * 32
    encoded += b"".join(u32(value) for value in limits) + u64(32) + u64(256)
    encoded += u32(1) + PUBLIC_KEY_1 + bytes([3]) * 32 + bytes([4]) * 32
    encoded += u32(0)  # No allocation: these transfers do not execute balances.
    assert len(encoded) == 246
    return encoded


def unsigned_operation(context: bytes, kind: int, sequence: int, body: bytes) -> bytes:
    prefix = u16(2) + bytes([kind]) + context + u64(7) + bytes([0x44]) * 32
    prefix += u64(8) + u64(24) + PUBLIC_KEY_1 + u64(sequence)
    assert len(prefix) == 131
    return prefix + body


def operation_leaf(signed: bytes) -> bytes:
    return sha256d(b"MIRROR_OP_LEAF_V2" + u32(len(signed)) + signed)


def merkle(leaves: list[bytes]) -> bytes:
    if not leaves:
        return sha256d(b"")
    level = list(leaves)
    while len(level) > 1:
        if len(level) % 2:
            level.append(level[-1])
        level = [sha256d(level[i] + level[i + 1]) for i in range(0, len(level), 2)]
    return level[0]


def operation_root(operations: list[bytes]) -> tuple[bytes, bytes]:
    bare_root = merkle([operation_leaf(op) for op in operations])
    return bare_root, sha256d(b"MIRROR_OP_ROOT_V2" + u32(len(operations)) + bare_root)


def header(config_context: bytes, root: bytes, **changed) -> bytes:
    fields = {
        "version": 2,
        "context": config_context,
        "height": 8,
        "previous": bytes([0x44]) * 32,
        "operations_root": root,
        "ledger_root": bytes([0x99]) * 32,
        "timestamp": 1800000123,
        "pow_bits": 0x207fffff,
        "nonce": 0x0102030405060708,
    }
    assert set(changed) <= set(fields)
    fields.update(changed)
    encoded = u32(fields["version"]) + fields["context"] + u64(fields["height"])
    encoded += fields["previous"] + fields["operations_root"] + fields["ledger_root"]
    encoded += u64(fields["timestamp"]) + u32(fields["pow_bits"]) + u64(fields["nonce"])
    assert len(encoded) == 160
    return encoded


def reference() -> str:
    config = configuration()
    context = sha256d(b"MIRROR_CONFIG_V2" + config)
    recipient = sha256d(b"MIRROR_ADDRESS_V1" + PUBLIC_KEY_2)
    output = {
        "config": config.hex(),
        "context": context.hex(),
        "public_key_1": PUBLIC_KEY_1.hex(),
        "public_key_2": PUBLIC_KEY_2.hex(),
        "recipient": recipient.hex(),
    }
    bodies = {
        "transfer": (0, 9, recipient + u64(123456789) + u64(0)),
        "transfer_max": (0, 2**64 - 1, recipient + u64(2**64 - 1) + u64(0)),
        "enroll": (1, 0, bytes([0x55]) * 32 + bytes([0x66]) * 32 + bytes([0x22]) * 32),
        "approve": (2, 9, bytes([0x77]) * 32),
        "withdraw": (3, 9, bytes([0x88]) * 32),
    }
    operations = {}
    for name, (kind, sequence, body) in bodies.items():
        unsigned = unsigned_operation(context, kind, sequence, body)
        signing_hash = sha256d(b"MIRROR_OPERATION_V2" + unsigned)
        signature = bytes.fromhex(SIGNATURES[name])
        assert len(signature) == 64
        signed = unsigned + signature
        assert len(signed) == {0: 243, 1: 291, 2: 227, 3: 227}[kind]
        operations[name] = signed
        for field, value in [
            ("unsigned", unsigned), ("hash", signing_hash), ("signature", signature),
            ("signed", signed), ("id", sha256d(b"MIRROR_OPERATION_ID_V2" + signed)),
            ("leaf", operation_leaf(signed)),
        ]:
            output[f"{name}_{field}"] = value.hex()

    cases = {
        "empty": [],
        "single": ["transfer"],
        "odd": ["transfer", "enroll", "approve"],
        "duplicate_last": ["transfer", "enroll", "approve", "approve"],
        "mixed": ["transfer", "enroll", "approve", "withdraw"],
        "reordered": ["withdraw", "approve", "enroll", "transfer"],
        "odd_five": ["transfer", "enroll", "approve", "withdraw", "transfer_max"],
    }
    for name, names in cases.items():
        items = [operations[item] for item in names]
        bare_root, root = operation_root(items)
        encoded_header = header(context, root)
        encoded_block = b"MRB2" + encoded_header + u32(len(items))
        encoded_block += b"".join(u32(len(item)) + item for item in items)
        assert len(encoded_block) == 168 + sum(4 + len(item) for item in items)
        output[f"{name}_count"] = str(len(items))
        output[f"{name}_merkle"] = bare_root.hex()
        output[f"{name}_root"] = root.hex()
        output[f"{name}_header"] = encoded_header.hex()
        output[f"{name}_hash"] = sha256d(b"MIRROR_BLOCK_V2" + encoded_header).hex()
        output[f"{name}_block"] = encoded_block.hex()

    assert output["odd_merkle"] == output["duplicate_last_merkle"]
    assert output["odd_root"] != output["duplicate_last_root"]
    assert output["mixed_root"] != output["reordered_root"]
    mixed_root = bytes.fromhex(output["mixed_root"])
    variants = {
        "context": {"context": bytes([0x12]) * 32},
        "height": {"height": 9},
        "previous": {"previous": bytes([0x45]) * 32},
        "operations_root": {"operations_root": bytes([0x46]) * 32},
        "ledger_root": {"ledger_root": bytes([0x9a]) * 32},
        "timestamp": {"timestamp": 1800000124},
        "pow_bits": {"pow_bits": 0x1f0fffff},
        "nonce": {"nonce": 0x0102030405060709},
        "max_integers": {"height": 2**64 - 1, "timestamp": 2**64 - 1,
                         "nonce": 2**64 - 1, "pow_bits": 2**32 - 1},
    }
    for name, changes in variants.items():
        encoded = header(context, mixed_root, **changes)
        output[f"header_{name}"] = encoded.hex()
        output[f"hash_{name}"] = sha256d(b"MIRROR_BLOCK_V2" + encoded).hex()
        assert output[f"hash_{name}"] != output["mixed_hash"]
    return "# Independent synthetic MB-02A vectors; no ledger acceptance claim.\n" + "".join(
        f"{name}={value}\n" for name, value in output.items()
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result = reference()
    if args.check:
        fixture = Path(__file__).resolve().parents[1] / "crates/mirror-core/tests/vectors/blocks_v2.txt"
        if fixture.read_text(encoding="utf-8") != result:
            raise SystemExit("FAIL: independent V2 transfer/block vectors differ; investigate rather than bless Rust output.")
        print("Independent V2 transfer/header/block vectors (5 operations, 7 blocks, 9 header variants): PASS")
    else:
        print(result, end="")


if __name__ == "__main__":
    main()
