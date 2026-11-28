#!/usr/bin/env python3
"""Independent MPQ2 request and finite-session vectors, using Python stdlib.

The established independent Python reference regenerates the synthetic MPN2
histories. This script packs the request itself and checks exact request/response
accounting. It does not invoke Rust or bless Rust output. Transcript hashes are
test assertions only, not additional wire fields or cryptographic domains.
"""

import argparse
import hashlib
from pathlib import Path
import struct

from reference_peer_history_v2 import reference as history_reference


def sha256d(data):
    return hashlib.sha256(hashlib.sha256(data).digest()).digest()


def reference():
    history = history_reference()
    raw = lambda name: bytes.fromhex(history[name])
    summary = raw('main4_summary')
    request = struct.pack('<4sHH32s32s', b'MPQ2', 2, 1, summary[:32], summary[32:64])
    assert len(request) == 72
    accepted_ceiling = 72 + int(history['max_wire_bytes'])
    conservative_attempt_ceiling = accepted_ceiling + 2
    values = {
        'magic': b'MPQ2'.hex(),
        'version': '2',
        'kind': '1',
        'request_len': str(len(request)),
        'request': request.hex(),
        'config_hash': summary[:32].hex(),
        'genesis_hash': summary[32:64].hex(),
        'max_accepted_session_wire_bytes': str(accepted_ceiling),
        'max_conservative_attempt_wire_bytes': str(conservative_attempt_ceiling),
        'max_attempts': '8',
        'max_cumulative_wire_bytes': str(8 * conservative_attempt_ceiling),
        'max_session_seconds': '60',
    }
    cases = (
        ('genesis', ['block0']),
        ('main3', ['block0', 'block1', 'block2', 'block3']),
        ('main4', ['block0', 'block1', 'block2', 'block3', 'block4']),
        ('fork3', ['block0', 'block1', 'fork2', 'fork3']),
    )
    for label, blocks in cases:
        response = raw(label + '_begin') + raw('manifest_frame')
        response += b''.join(raw(block + '_frame') for block in blocks)
        response += raw(label + '_end')
        session = request + response
        assert len(response) == int(history[label + '_wire_len'])
        values[label + '_response_wire_bytes'] = str(len(response))
        values[label + '_session_wire_bytes'] = str(len(session))
        values[label + '_transcript_sha256d'] = sha256d(session).hex()
    assert accepted_ceiling == 19641184
    assert 8 * conservative_attempt_ceiling == 157129488
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    values = reference()
    text = '# Independent MPQ2 finite session; tools/reference_peer_session_v2.py.\n'
    text += '# PUBLIC synthetic data. EOF has zero encoded bytes; hashes are test-only.\n'
    text += ''.join(f'{name}={value}\n' for name, value in values.items())
    if args.check:
        path = Path(__file__).resolve().parents[1] / 'crates/mirror-network/tests/vectors/peer_session_v2.txt'
        if path.read_text(encoding='utf-8') != text:
            raise SystemExit('FAIL: independent peer-session vectors differ; investigate, never bless Rust output.')
        print('Independent V2 peer-session request, four transcripts and cumulative bounds: PASS')
    else:
        print(text, end='')


if __name__ == '__main__':
    main()
