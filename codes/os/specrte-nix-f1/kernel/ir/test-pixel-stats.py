#!/usr/bin/env python3
"""Validate RAW10 packing and partial-frame rejection with synthetic data."""
import random
import struct
import subprocess
import sys

exe = sys.argv[1]
rng = random.Random(7251)
values = [0, 1, 2, 1023] + [rng.randrange(1024) for _ in range(124)]
packed = bytearray()
for i in range(0, len(values), 4):
    group = values[i:i + 4]
    packed.extend(v >> 2 for v in group)
    packed.append(sum((v & 3) << (2 * j) for j, v in enumerate(group)))
unpacked = struct.pack('<' + 'H' * len(values), *values)
a = subprocess.check_output([exe, '--packed', '--frame-pixels', '64'], input=packed)
b = subprocess.check_output([exe, '--frame-pixels', '64'], input=unpacked)
assert a == b
lines = a.decode().splitlines()
assert len(lines) == 2
for i, line in enumerate(lines):
    group = values[i * 64:(i + 1) * 64]
    assert f'min={min(group)} max={max(group)} mean={sum(group) / len(group):.3f}' in line
for args, data in [(['--packed'], b'x'), ([], b'x'), ([], b'\x00\x04'),
                   (['--packed', '--frame-pixels', '64'], packed[:-5]),
                   ([], b'')]:
    assert subprocess.run([exe, *args], input=data, capture_output=True).returncode != 0
print('Packed/unpacked agreement, two frame aggregates, and malformed inputs: PASS')
