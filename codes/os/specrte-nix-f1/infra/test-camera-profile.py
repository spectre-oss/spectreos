#!/usr/bin/env python3
"""Validate profile provenance and reject damaged containers; run on build host.

Usage: python3 infra/test-camera-profile.py /path/to/original/vendor
Uses the owner's original files, without copying them into the source tree.
"""
import importlib.util
import json
from pathlib import Path
import struct
import sys
import tempfile

spec = importlib.util.spec_from_file_location('decoder', Path(__file__).with_name('decode-camera-profile.py'))
decoder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(decoder)
vendor = Path(sys.argv[1])
profile = json.loads((Path(__file__).resolve().parents[1] / 'modules/camera/imx363-sunny-profile.json').read_text())
original = decoder.Container(vendor / profile['source_name'])
report = original.report()
assert report['sha256'] == profile['source_sha256']
root = next(r for r in report['cc13_roots'] if r['id'] == profile['root_symbol'])
assert root['mode_path'] == profile['mode_path']
node = root['tree']
for ident in profile['child_symbols'][:-1]:
    assert node['id'] == ident
    node = node['records'][0]['child']
assert node['id'] == profile['matrix_symbol']
for row in node['records']:
    assert [list(r) for r in row['matrix']] == profile['matrix']
    assert list(row['offsets']) == profile['offsets']
assert node['records'][0]['file_offset'] + 8 == profile['matrix_file_offset']
for name, expected in [('sunny', 1), ('qtech', 6)]:
    module = decoder.Container(vendor / f'com.qti.sensormodule.{name}_imx363.bin').report()
    values = {row['name']: row['value'] for row in module['module_custom_info']}
    assert values['vendorId'] == expected
    assert values['vendorIdOffset'] == values['vendorIdAddr'] == 1

with tempfile.TemporaryDirectory() as temporary:
    path = Path(temporary) / 'damaged.bin'
    def rejected(data):
        path.write_bytes(data)
        try:
            decoder.Container(path).report()
        except (ValueError, KeyError, UnicodeError):
            return
        raise AssertionError('Damaged container was accepted')
    rejected(original.data[:-1])
    damaged = bytearray(original.data)
    struct.pack_into('<f', damaged, profile['matrix_file_offset'], float('nan'))
    rejected(damaged)
    damaged = bytearray(original.data)
    struct.pack_into('<I', damaged, root['file_offset'] + 56, 99999)
    rejected(damaged)
    damaged = bytearray(original.data)
    mode_offset, mode_size = original.sections[2]
    for pos in range(mode_offset, mode_offset + mode_size, 16):
        if original.ints(pos, 1)[0] == root['mode_id']:
            struct.pack_into('<I', damaged, pos + 12, 0xffffffff)
    rejected(damaged)
print('PASS: original Sunny matrix/offset/hash, two module IDs, truncation, NaN, child count, invalid mode parent')
