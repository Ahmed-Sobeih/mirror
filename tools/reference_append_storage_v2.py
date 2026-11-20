#!/usr/bin/env python3
"""Independent MB-03B MSU2 append-selection vectors.

The complete MB-02C chain and MB-03A storage fixtures are pinned inputs. This
stdlib-only script derives canonical MSS2 snapshots directly from the complete
MRB2 history, checks the existing MSS2 vectors, then wraps them in MSU2. It never
consumes Rust output or alters an existing fixture. All history is synthetic and
public; checksums do not supply authenticity, trusted selection or freshness.
"""

import argparse
import hashlib
from pathlib import Path
import struct


CHAIN_FIXTURE_SHA256 = "c5c443a604022669889f8b05a5244816d6e702e37cd409f79cf3ccebd3bbed2e"
STORAGE_FIXTURE_SHA256 = "ab25051450c08774528fa14c253072a7856824051f526a2b7ea331adc34902ef"
SNAPSHOT_DOMAIN = b"MIRROR_STORAGE_SELECTION_V2"
APPEND_DOMAIN = b"MIRROR_STORAGE_APPEND_SELECTION_V2"
SNAPSHOT_PREFIX = struct.Struct("<4sHHQ32s32sQ32s32sIIQ32s")
WRAPPER_PREFIX = struct.Struct("<4sHHQ")
APPEND_LEN = 280


def sha256d(value: bytes) -> bytes:
    return hashlib.sha256(hashlib.sha256(value).digest()).digest()


def pinned_values(path: Path, expected_sha256: str) -> dict[str, str]:
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected_sha256:
        raise SystemExit(f"FAIL: frozen input {path.name} changed; investigate before updating its pin.")
    result = {}
    for line in raw.decode("utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        key, value = line.split("=", 1)
        if key in result:
            raise SystemExit(f"FAIL: duplicate fixture key {key}.")
        result[key] = value
    return result


def reference(root: Path) -> dict[str, str]:
    chain = pinned_values(
        root / "crates/mirror-chain/tests/vectors/chain_v2.txt",
        CHAIN_FIXTURE_SHA256,
    )
    old = pinned_values(
        root / "crates/mirror-storage/tests/vectors/storage_v2.txt",
        STORAGE_FIXTURE_SHA256,
    )
    manifest = bytes.fromhex(chain["manifest_hex"])
    assert manifest[:6] == b"MNM2\x02\x00"
    config_len = struct.unpack_from("<I", manifest, 6)[0]
    context = sha256d(b"MIRROR_CONFIG_V2" + manifest[10:10 + config_len])
    genesis = manifest[-32:]
    assert context.hex() == chain["context"] == old["config_hash"]
    assert genesis.hex() == chain["block0_hash"] == old["genesis_hash"]
    assert len(manifest) == int(chain["manifest_len"]) == int(old["manifest_len"])
    manifest_checksum = sha256d(manifest)
    assert manifest_checksum.hex() == old["manifest_checksum"]
    assert SNAPSHOT_PREFIX.size == 200
    assert WRAPPER_PREFIX.size == 16

    output = {
        "source_chain_fixture_sha256": CHAIN_FIXTURE_SHA256,
        "source_storage_fixture_sha256": STORAGE_FIXTURE_SHA256,
        "append_selection_length": str(APPEND_LEN),
        "append_selection_version": "2",
        "append_selection_profile": "2",
        "embedded_snapshot_length": "232",
        "config_hash": context.hex(),
        "genesis_hash": genesis.hex(),
    }
    histories = {
        "genesis": (["block0"], 0, "genesis"),
        **{f"main{i}": ([f"block{j}" for j in range(i + 1)], i, None) for i in range(1, 5)},
        "initialized_main": ([f"block{j}" for j in range(5)], 0, "main"),
        "fork": (["block0", "block1", "fork2", "fork3"], 3, "fork"),
    }
    for name, (labels, generation, old_label) in histories.items():
        previous_hash = bytes(32)
        block_bytes = 0
        for height, label in enumerate(labels):
            encoded = bytes.fromhex(chain[f"{label}_block"])
            assert encoded[:4] == b"MRB2" and len(encoded) >= 168
            header = encoded[4:164]
            assert header[:4] == struct.pack("<I", 2)
            assert header[4:36] == context
            assert struct.unpack_from("<Q", header, 36)[0] == height
            assert header[44:76] == previous_hash
            block_hash = sha256d(b"MIRROR_BLOCK_V2" + header)
            assert block_hash.hex() == chain[f"{label}_hash"]
            if height == 0:
                assert block_hash == genesis
            previous_hash = block_hash
            block_bytes += len(encoded)
        count = len(labels)
        height = count - 1
        ledger_root = header[108:140]
        assert 0 <= generation <= height
        assert 1 <= count <= 4096
        assert 168 * count <= block_bytes <= 16_777_216
        snapshot_prefix = SNAPSHOT_PREFIX.pack(
            b"MSS2", 2, 1, 0, context, genesis, height, previous_hash,
            ledger_root, count, len(manifest), block_bytes, manifest_checksum,
        )
        snapshot = snapshot_prefix + sha256d(SNAPSHOT_DOMAIN + snapshot_prefix)
        assert len(snapshot) == 232
        if old_label is not None:
            assert snapshot.hex() == old[f"{old_label}_selection_hex"]
        prefix = WRAPPER_PREFIX.pack(b"MSU2", 2, 2, generation) + snapshot
        checksum = sha256d(APPEND_DOMAIN + prefix)
        encoded = prefix + checksum
        assert len(prefix) == 248 and len(encoded) == APPEND_LEN
        output.update({
            f"{name}_generation": str(generation),
            f"{name}_height": str(height),
            f"{name}_tip_hash": previous_hash.hex(),
            f"{name}_ledger_root": ledger_root.hex(),
            f"{name}_block_count": str(count),
            f"{name}_blocks_bytes": str(block_bytes),
            f"{name}_snapshot_hex": snapshot.hex(),
            f"{name}_record_checksum": checksum.hex(),
            f"{name}_selection_hex": encoded.hex(),
        })
    assert output["main4_snapshot_hex"] == output["initialized_main_snapshot_hex"]
    assert output["main4_selection_hex"] != output["initialized_main_selection_hex"]
    assert output["main3_ledger_root"] != output["fork_ledger_root"]
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    values = reference(root)
    result = "# Independent MSU2 append selections; tools/reference_append_storage_v2.py.\n"
    result += "# Frozen public synthetic MB-02C/MB-03A inputs; no real identities or money.\n"
    result += "".join(f"{key}={value}\n" for key, value in values.items())
    if args.check:
        path = root / "crates/mirror-storage/tests/vectors/append_storage_v2.txt"
        if path.read_text(encoding="utf-8") != result:
            raise SystemExit("FAIL: independent append vectors differ; investigate, never bless Rust output.")
        print("Independent MSU2 selections (genesis, four append generations, selected initialization, fork): PASS")
    else:
        print(result, end="")


if __name__ == "__main__":
    main()
