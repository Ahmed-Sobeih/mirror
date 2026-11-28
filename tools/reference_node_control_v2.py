#!/usr/bin/env python3
"""Independent MNC2/MNI2 local node-control records, using Python stdlib.

The established independent references supply frozen complete synthetic histories
and unchanged MSR2 metadata. This script independently packs the new envelopes,
their lengths and checksums. It does not invoke Rust or consume Rust output.
Checksums are corruption checks, not network or local-operator authority.
"""

import argparse
from pathlib import Path
import struct

from reference_peer_history_v2 import reference as history_reference
from reference_replacement_storage_v2 import (
    reference as replacement_reference,
    replacement_envelope,
    sha256d,
)


CONTROL_DOMAIN = b'MIRROR_NODE_CONTROL_RECORD_V2'
INTENT_DOMAIN = b'MIRROR_NODE_REPLACEMENT_INTENT_V2'
CONTROL_LEN = 464
INTENT_HEADER_LEN = 952
MAX_HISTORY_BYTES = 19_641_112
MAX_INTENT_BYTES = 984 + 2 * MAX_HISTORY_BYTES


def control_record(selection):
    assert len(selection) == 280
    generation = struct.unpack_from('<Q', selection, 8)[0]
    snapshot = selection[16:248]
    context, genesis, height, tip, root = struct.unpack_from('<32s32sQ32s32s', snapshot, 16)
    assert generation <= height < 4096
    pin = struct.pack('<Q32s32sQ32s32s', generation, context, genesis, height, tip, root)
    assert len(pin) == 144
    prefix = struct.pack('<4sHH', b'MNC2', 2, 1) + pin + selection
    assert len(prefix) == 432
    encoded = prefix + sha256d(CONTROL_DOMAIN + prefix)
    assert len(encoded) == CONTROL_LEN
    return encoded


def intent_record(old, candidate, old_history, candidate_history):
    assert len(old) == len(candidate) == CONTROL_LEN
    assert 0 < len(old_history) <= MAX_HISTORY_BYTES
    assert 0 < len(candidate_history) <= MAX_HISTORY_BYTES
    old_generation, candidate_generation = [struct.unpack_from('<Q', x, 8)[0] for x in (old, candidate)]
    old_height, candidate_height = [struct.unpack_from('<Q', x, 80)[0] for x in (old, candidate)]
    assert candidate_generation == old_generation + 1
    assert candidate_height > old_height
    assert old[16:80] == candidate[16:80]
    header = struct.pack('<4sHH', b'MNI2', 2, 1) + old + candidate
    header += struct.pack('<QQ', len(old_history), len(candidate_history))
    assert len(header) == INTENT_HEADER_LEN
    prefix = header + old_history + candidate_history
    encoded = prefix + sha256d(INTENT_DOMAIN + prefix)
    assert len(encoded) <= MAX_INTENT_BYTES
    return encoded


def reference():
    root = Path(__file__).resolve().parents[1]
    selections = replacement_reference(root)
    history = history_reference()
    raw_history = lambda name: bytes.fromhex(history[name])
    streams = {}
    for label, blocks in (
        ('genesis', ['block0']),
        ('main3', ['block0', 'block1', 'block2', 'block3']),
        ('main4', ['block0', 'block1', 'block2', 'block3', 'block4']),
        ('fork3', ['block0', 'block1', 'fork2', 'fork3']),
    ):
        streams[label] = raw_history(label + '_begin') + raw_history('manifest_frame')
        streams[label] += b''.join(raw_history(block + '_frame') for block in blocks)
        streams[label] += raw_history(label + '_end')
        assert sha256d(streams[label]).hex() == history[label + '_stream_sha256d']

    values = {
        'control_magic': b'MNC2'.hex(),
        'intent_magic': b'MNI2'.hex(),
        'version': '2',
        'kind': '1',
        'control_record_len': str(CONTROL_LEN),
        'intent_header_len': str(INTENT_HEADER_LEN),
        'intent_fixed_len': '984',
        'max_intent_bytes': str(MAX_INTENT_BYTES),
        'control_checksum_domain': CONTROL_DOMAIN.hex(),
        'intent_checksum_domain': INTENT_DOMAIN.hex(),
        'config_hash': selections['config_hash'],
        'genesis_hash': selections['genesis_hash'],
    }
    controls = {}
    for label, source, generation in (
        ('genesis_g0', 'genesis', 0),
        ('main3_g0', 'main3', 0),
        ('main3_g3', 'main3', 3),
        ('main4_g1', 'main4', 1),
        ('main4_g4', 'main4', 4),
        ('fork3_g0', 'fork', 0),
    ):
        snapshot = bytes.fromhex(selections[source + '_snapshot_hex'])
        controls[label] = control_record(replacement_envelope(snapshot, generation))
        values[label + '_control'] = controls[label].hex()

    for label, old, candidate, old_history, candidate_history in (
        ('genesis_main4', 'genesis_g0', 'main4_g1', 'genesis', 'main4'),
        ('main3_main4', 'main3_g0', 'main4_g1', 'main3', 'main4'),
        ('fork3_main4', 'fork3_g0', 'main4_g1', 'fork3', 'main4'),
        ('main3_g3_main4', 'main3_g3', 'main4_g4', 'main3', 'main4'),
    ):
        encoded = intent_record(controls[old], controls[candidate], streams[old_history], streams[candidate_history])
        values[label + '_intent_len'] = str(len(encoded))
        values[label + '_intent_sha256d'] = sha256d(encoded).hex()
        values[label + '_intent'] = encoded.hex()
    assert MAX_INTENT_BYTES == 39_283_208
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    values = reference()
    text = '# Independent local MNC2/MNI2 control records; tools/reference_node_control_v2.py.\n'
    text += '# PUBLIC synthetic histories, unchanged MPN2/MSR2; no secrets or real identities.\n'
    text += ''.join(f'{name}={value}\n' for name, value in values.items())
    if args.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-node/tests/vectors/node_control_v2.txt'
        if path.read_text(encoding='utf-8') != text:
            raise SystemExit('FAIL: independent node-control vectors differ; investigate, never bless Rust output.')
        print('Independent V2 node control: six confirmed records, four complete two-history intents and bounds: PASS')
    else:
        print(text, end='')


if __name__ == '__main__':
    main()
