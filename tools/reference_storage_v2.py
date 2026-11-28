#!/usr/bin/env python3
"""Independent MB-03A MSS2 selection-record vectors.

The input history is the frozen, independently generated MB-02C fixture. Its
complete SHA-256 is pinned below; this script never consumes Rust output or
changes that existing fixture. Python struct/hashlib derive the selected hashes,
byte counts and exact new record encoding before Rust comparison. All inputs are
public synthetic development data, with no biometric archive or private key.

A checksum is not authentication or freshness. These records require an external
trusted checkpoint and complete actual-chain replay before authoritative use.
"""

import argparse
import hashlib
from pathlib import Path
import struct


CHAIN_FIXTURE_SHA256 = "c5c443a604022669889f8b05a5244816d6e702e37cd409f79cf3ccebd3bbed2e"
RECORD_DOMAIN = b"MIRROR_STORAGE_SELECTION_V2"
RECORD_PREFIX = struct.Struct("<4sHHQ32s32sQ32s32sIIQ32s")
RECORD_LEN = 232
MAX_MANIFEST_BYTES = 2_781_636
MAX_BLOCKS_BYTES = 16_777_216
MAX_BLOCKS = 4_096


def sha256d(value: bytes) -> bytes:
    return hashlib.sha256(hashlib.sha256(value).digest()).digest()


def source_values(root: Path) -> dict[str, str]:
    path = root / "crates/mirror-chain/tests/vectors/chain_v2.txt"
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    if actual != CHAIN_FIXTURE_SHA256:
        raise SystemExit("FAIL: frozen MB-02C input fixture changed; investigate before updating any pin.")
    result = {}
    for line in raw.decode("utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        key, value = line.split("=", 1)
        if key in result:
            raise SystemExit(f"FAIL: duplicate source fixture key {key}.")
        result[key] = value
    return result


def reference(root: Path) -> dict[str, str]:
    values = source_values(root)
    manifest = bytes.fromhex(values["manifest_hex"])
    assert manifest[:6] == b"MNM2\x02\x00"
    config_len = struct.unpack_from("<I", manifest, 6)[0]
    config = manifest[10:10 + config_len]
    context = sha256d(b"MIRROR_CONFIG_V2" + config)
    assert context.hex() == values["context"]
    genesis_hash = manifest[-32:]
    assert genesis_hash.hex() == values["block0_hash"]
    assert len(manifest) == int(values["manifest_len"])
    manifest_checksum = sha256d(manifest)
    assert RECORD_PREFIX.size == 200

    output = {
        "source_chain_fixture_sha256": CHAIN_FIXTURE_SHA256,
        "selection_length": str(RECORD_LEN),
        "selection_version": "2",
        "selection_profile": "1",
        "generation": "0",
        "max_blocks": str(MAX_BLOCKS),
        "max_manifest_bytes": str(MAX_MANIFEST_BYTES),
        "max_blocks_bytes": str(MAX_BLOCKS_BYTES),
        "max_encoded_read_bytes": str(RECORD_LEN + MAX_MANIFEST_BYTES + MAX_BLOCKS_BYTES),
        "config_hash": context.hex(),
        "genesis_hash": genesis_hash.hex(),
        "manifest_len": str(len(manifest)),
        "manifest_checksum": manifest_checksum.hex(),
    }
    histories = {
        "genesis": ["block0"],
        "main": [f"block{i}" for i in range(5)],
        "fork": ["block0", "block1", "fork2", "fork3"],
    }
    for name, labels in histories.items():
        previous_hash = bytes(32)
        blocks_bytes = 0
        for height, label in enumerate(labels):
            encoded = bytes.fromhex(values[f"{label}_block"])
            assert encoded[:4] == b"MRB2" and len(encoded) >= 168
            header = encoded[4:164]
            assert header[:4] == struct.pack("<I", 2)
            assert header[4:36] == context
            assert struct.unpack_from("<Q", header, 36)[0] == height
            assert header[44:76] == previous_hash
            block_hash = sha256d(b"MIRROR_BLOCK_V2" + header)
            assert block_hash.hex() == values[f"{label}_hash"]
            if height == 0:
                assert block_hash == genesis_hash
            previous_hash = block_hash
            blocks_bytes += len(encoded)

        count = len(labels)
        tip_height = count - 1
        ledger_root = header[108:140]
        assert 1 <= count <= MAX_BLOCKS
        assert 168 * count <= blocks_bytes <= MAX_BLOCKS_BYTES
        prefix = RECORD_PREFIX.pack(
            b"MSS2", 2, 1, 0, context, genesis_hash, tip_height,
            previous_hash, ledger_root, count, len(manifest), blocks_bytes,
            manifest_checksum,
        )
        checksum = sha256d(RECORD_DOMAIN + prefix)
        encoded = prefix + checksum
        assert len(encoded) == RECORD_LEN
        output.update({
            f"{name}_height": str(tip_height),
            f"{name}_tip_hash": previous_hash.hex(),
            f"{name}_ledger_root": ledger_root.hex(),
            f"{name}_block_count": str(count),
            f"{name}_blocks_bytes": str(blocks_bytes),
            f"{name}_record_checksum": checksum.hex(),
            f"{name}_selection_hex": encoded.hex(),
        })
    assert output["main_ledger_root"] != output["fork_ledger_root"]
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    values = reference(root)
    result = "# Independent MSS2 storage selections; tools/reference_storage_v2.py.\n"
    result += "# Frozen public synthetic MB-02C input; no real identities or money.\n"
    result += "".join(f"{key}={value}\n" for key, value in values.items())
    if args.check:
        path = root / "crates/mirror-storage/tests/vectors/storage_v2.txt"
        if path.read_text(encoding="utf-8") != result:
            raise SystemExit("FAIL: independent storage vectors differ; investigate, never bless Rust output.")
        print("Independent MSS2 selection vectors (genesis, mixed main history, membership fork): PASS")
    else:
        print(result, end="")


if __name__ == "__main__":
    main()
