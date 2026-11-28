#!/usr/bin/env python3
"""Independent MPN2 candidate-history framing vectors, using only Python stdlib.

The existing independent chain reference regenerates public synthetic MNM2/MRB2
inputs. This module packs the new transfer fields itself, without loading Rust,
calling Rust or accepting Rust output as an expected vector. Framing correctness
does not validate a chain or establish a peer's endpoint as canonical or current.
"""

import argparse
import hashlib
from pathlib import Path
import struct

from reference_chain_v2 import reference as chain_reference


def sha256d(data):
    return hashlib.sha256(hashlib.sha256(data).digest()).digest()


def frame(kind, payload):
    return struct.pack('<4sHHI', b'MPN2', 2, kind, len(payload)) + payload


def summary(context, genesis, height, tip, root, count, block_bytes, manifest_bytes):
    assert 1 <= count <= 4096 and height + 1 == count
    assert 46 <= manifest_bytes <= 2781636
    assert count * 168 <= block_bytes <= min(count * 1048576, 16777216)
    encoded = struct.pack('<32s32sQ32s32sIQI', context, genesis, height, tip,
                          root, count, block_bytes, manifest_bytes)
    assert len(encoded) == 152
    return encoded


def reference():
    chain, _ = chain_reference()
    raw = lambda key: bytes.fromhex(chain[key])
    context, genesis, manifest = raw('context'), raw('block0_hash'), raw('manifest_hex')
    value = {
        'magic': b'MPN2'.hex(),
        'version': '2',
        'header_len': str(struct.calcsize('<4sHHI')),
        'summary_len': str(struct.calcsize('<32s32sQ32s32sIQI')),
        'max_blocks': '4096',
        'max_frames': '4099',
        'max_block_bytes': '1048576',
        'max_history_block_bytes': '16777216',
        'max_manifest_bytes': '2781636',
        'manifest_frame': frame(2, manifest).hex(),
    }
    for name in ('block0', 'block1', 'block2', 'block3', 'block4', 'fork2', 'fork3'):
        payload = struct.pack('<Q', int(chain[name + '_height'])) + raw(name + '_block')
        value[name + '_frame'] = frame(3, payload).hex()

    cases = (
        ('genesis', ['block0'], 'state0'),
        ('main3', ['block0', 'block1', 'block2', 'block3'], 'state3'),
        ('main4', ['block0', 'block1', 'block2', 'block3', 'block4'], 'state4'),
        ('fork3', ['block0', 'block1', 'fork2', 'fork3'], 'fork_state3'),
    )
    for label, names, state in cases:
        block_bytes = sum(len(raw(name + '_block')) for name in names)
        body = summary(context, genesis, len(names) - 1, raw(names[-1] + '_hash'),
                       raw(state + '_ledger_root'), len(names), block_bytes, len(manifest))
        begin, end = frame(1, body), frame(4, body)
        frames = [begin, frame(2, manifest)]
        frames.extend(bytes.fromhex(value[name + '_frame']) for name in names)
        frames.append(end)
        stream = b''.join(frames)
        exact_len = 2 * 152 + len(manifest) + block_bytes + len(names) * (12 + 8) + 3 * 12
        assert exact_len == len(stream)
        value[label + '_summary'] = body.hex()
        value[label + '_begin'] = begin.hex()
        value[label + '_end'] = end.hex()
        value[label + '_block_count'] = str(len(names))
        value[label + '_block_bytes'] = str(block_bytes)
        value[label + '_wire_len'] = str(exact_len)
        value[label + '_frame_count'] = str(len(frames))
        value[label + '_stream_sha256d'] = sha256d(stream).hex()

    maximum = summary(bytes([0x11]) * 32, bytes([0x22]) * 32, 4095,
                      bytes([0x33]) * 32, bytes([0x44]) * 32,
                      4096, 16777216, 2781636)
    value['maximum_shape_summary'] = maximum.hex()
    maximum_wire = 2 * 152 + 2781636 + 16777216 + 4096 * (12 + 8) + 3 * 12
    assert maximum_wire == 19641112
    value['max_wire_bytes'] = str(maximum_wire)
    for kind, length, label in (
        (1, 152, 'begin'), (2, 46, 'manifest_min'), (2, 2781636, 'manifest_max'),
        (3, 8 + 168, 'block_min'), (3, 8 + 1048576, 'block_max'), (4, 152, 'end'),
    ):
        value[label + '_header'] = struct.pack('<4sHHI', b'MPN2', 2, kind, length).hex()
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    values = reference()
    text = '# Independent MPN2 framing; tools/reference_peer_history_v2.py.\n'
    text += '# PUBLIC synthetic chain bytes; endpoints are untrusted transfer claims.\n'
    text += ''.join(f'{key}={value}\n' for key, value in values.items())
    if args.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-protocol/tests/vectors/peer_history_v2.txt'
        if path.read_text(encoding='utf-8') != text:
            raise SystemExit('FAIL: independent peer-history vectors differ; investigate, never bless Rust output.')
        print('Independent V2 peer-history framing (four histories, exact lengths, hashes and bounds): PASS')
    else:
        print(text, end='')


if __name__ == '__main__':
    main()
