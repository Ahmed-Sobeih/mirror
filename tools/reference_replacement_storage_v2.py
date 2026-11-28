#!/usr/bin/env python3
"""Independent MB-03E1 MSR2 replacement-selection vectors.

Pinned, previously frozen MB-02C actual-chain and MB-03A immutable-storage
fixtures are the only data inputs. Python struct/hashlib derive the nested MSS2
snapshots and new profile-3 envelopes; no Rust output is consumed and no previous
fixture is written. Every example is public synthetic development data.

The selected snapshot is complete-branch metadata, not proof of a publication,
authorization, accepted chain, global freshness or successful reorganization.
The local generation is independent of fork score. Generation zero can describe
initialization at any accepted height; the structural maximum is selected height
because this development profile only permits strictly higher preferred branches.
"""

import argparse
import hashlib
from pathlib import Path
import struct


CHAIN_FIXTURE_SHA256 = "c5c443a604022669889f8b05a5244816d6e702e37cd409f79cf3ccebd3bbed2e"
STORAGE_FIXTURE_SHA256 = "ab25051450c08774528fa14c253072a7856824051f526a2b7ea331adc34902ef"
SNAPSHOT_DOMAIN = b"MIRROR_STORAGE_SELECTION_V2"
REPLACEMENT_DOMAIN = b"MIRROR_STORAGE_REPLACEMENT_SELECTION_V2"
SNAPSHOT_PREFIX = struct.Struct("<4sHHQ32s32sQ32s32sIIQ32s")
WRAPPER_PREFIX = struct.Struct("<4sHHQ")
SNAPSHOT_LEN = 232
REPLACEMENT_LEN = 280
MAX_BLOCKS = 4096
MAX_MANIFEST_BYTES = 2_781_636
MAX_BLOCKS_BYTES = 16_777_216


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


def replacement_envelope(snapshot: bytes, generation: int) -> bytes:
    """Independently encode a bounded snapshot under the frozen new contract."""
    if len(snapshot) != SNAPSHOT_LEN:
        raise ValueError("snapshot must be exactly 232 bytes")
    fields = SNAPSHOT_PREFIX.unpack(snapshot[:200])
    magic, version, profile, inner_generation = fields[:4]
    height, count, manifest_len, blocks_bytes = fields[6], fields[9], fields[10], fields[11]
    if (magic, version, profile, inner_generation) != (b"MSS2", 2, 1, 0):
        raise ValueError("snapshot must retain immutable profile 1 and generation zero")
    if snapshot[200:] != sha256d(SNAPSHOT_DOMAIN + snapshot[:200]):
        raise ValueError("nested snapshot checksum mismatch")
    if not (1 <= count <= MAX_BLOCKS and height == count - 1):
        raise ValueError("snapshot height/count is out of bounds")
    if not (46 <= manifest_len <= MAX_MANIFEST_BYTES):
        raise ValueError("snapshot manifest length is out of bounds")
    if not (168 * count <= blocks_bytes <= MAX_BLOCKS_BYTES):
        raise ValueError("snapshot block bytes are out of bounds")
    if not (0 <= generation <= height):
        raise ValueError("generation must be nonnegative and no greater than selected height")
    prefix = WRAPPER_PREFIX.pack(b"MSR2", 2, 3, generation) + snapshot
    assert len(prefix) == 248
    encoded = prefix + sha256d(REPLACEMENT_DOMAIN + prefix)
    assert len(encoded) == REPLACEMENT_LEN
    return encoded


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
    assert SNAPSHOT_PREFIX.size == 200 and WRAPPER_PREFIX.size == 16

    output = {
        "source_chain_fixture_sha256": CHAIN_FIXTURE_SHA256,
        "source_storage_fixture_sha256": STORAGE_FIXTURE_SHA256,
        "replacement_selection_length": str(REPLACEMENT_LEN),
        "replacement_selection_version": "2",
        "replacement_selection_profile": "3",
        "replacement_checksum_domain_hex": REPLACEMENT_DOMAIN.hex(),
        "embedded_snapshot_length": str(SNAPSHOT_LEN),
        "config_hash": context.hex(),
        "genesis_hash": genesis.hex(),
    }
    histories = {
        "genesis": (["block0"], 0, "genesis"),
        **{f"main{i}": ([f"block{j}" for j in range(i + 1)], i, None) for i in range(1, 5)},
        "initialized_main": ([f"block{j}" for j in range(5)], 0, "main"),
        "main4_after_replacement": ([f"block{j}" for j in range(5)], 1, "main"),
        "fork": (["block0", "block1", "fork2", "fork3"], 3, "fork"),
        "initialized_fork": (["block0", "block1", "fork2", "fork3"], 0, "fork"),
    }
    for name, (labels, generation, old_label) in histories.items():
        previous_hash = bytes(32)
        blocks_bytes = 0
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
            blocks_bytes += len(encoded)

        count = len(labels)
        height = count - 1
        ledger_root = header[108:140]
        snapshot_prefix = SNAPSHOT_PREFIX.pack(
            b"MSS2", 2, 1, 0, context, genesis, height, previous_hash,
            ledger_root, count, len(manifest), blocks_bytes, manifest_checksum,
        )
        snapshot = snapshot_prefix + sha256d(SNAPSHOT_DOMAIN + snapshot_prefix)
        assert len(snapshot) == SNAPSHOT_LEN
        if old_label is not None:
            assert snapshot.hex() == old[f"{old_label}_selection_hex"]
        encoded = replacement_envelope(snapshot, generation)
        assert encoded[16:248] == snapshot
        assert struct.unpack_from("<Q", encoded, 8)[0] == generation
        assert struct.unpack_from("<Q", encoded, 24)[0] == 0
        assert encoded[248:] == sha256d(REPLACEMENT_DOMAIN + encoded[:248])
        output.update({
            f"{name}_generation": str(generation),
            f"{name}_height": str(height),
            f"{name}_tip_hash": previous_hash.hex(),
            f"{name}_ledger_root": ledger_root.hex(),
            f"{name}_block_count": str(count),
            f"{name}_blocks_bytes": str(blocks_bytes),
            f"{name}_snapshot_hex": snapshot.hex(),
            f"{name}_record_checksum": encoded[248:].hex(),
            f"{name}_selection_hex": encoded.hex(),
        })
        for invalid_generation in (-1, height + 1, 2**64):
            try:
                replacement_envelope(snapshot, invalid_generation)
            except ValueError:
                pass
            else:
                raise AssertionError("independent encoder admitted an out-of-bounds generation")
        assert len(replacement_envelope(snapshot, 0)) == REPLACEMENT_LEN
        assert len(replacement_envelope(snapshot, height)) == REPLACEMENT_LEN

    # A branch can be initialized or selected at different local generations.
    # Equal snapshots do not mean equal selected-generation records.
    assert output["main4_snapshot_hex"] == output["initialized_main_snapshot_hex"]
    assert output["main4_snapshot_hex"] == output["main4_after_replacement_snapshot_hex"]
    assert len({output[f"{name}_selection_hex"] for name in
                ("main4", "initialized_main", "main4_after_replacement")}) == 3
    assert output["fork_snapshot_hex"] == output["initialized_fork_snapshot_hex"]
    assert output["fork_selection_hex"] != output["initialized_fork_selection_hex"]
    assert output["main3_ledger_root"] != output["fork_ledger_root"]
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    values = reference(root)
    result = "# Independent MSR2 replacement selections; tools/reference_replacement_storage_v2.py.\n"
    result += "# Frozen public synthetic MB-02C/MB-03A inputs; no real identities or money.\n"
    result += "".join(f"{key}={value}\n" for key, value in values.items())
    if args.check:
        path = root / "crates/mirror-storage/tests/vectors/replacement_storage_v2.txt"
        if path.read_text(encoding="utf-8") != result:
            raise SystemExit("FAIL: independent replacement vectors differ; investigate, never bless Rust output.")
        print("Independent MSR2 selections (nine cases, unchanged MSS2, distinct generations): PASS")
    else:
        print(result, end="")


if __name__ == "__main__":
    main()
