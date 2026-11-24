#!/usr/bin/env python3
"""Read OV7251 module lane metadata and mode registers without touching a phone."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    'camera_profile', Path(__file__).resolve().parents[2] / 'infra/decode-camera-profile.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def decode(path):
    container = module.Container(path)

    def unique(name):
        matches = [s for s in container.symbols.values() if s['name'] == name]
        if len(matches) != 1:
            raise ValueError(f'Expected one {name}')
        return matches[0]

    lane = unique('laneAssign')
    combo = unique('isComboMode')
    if lane['size'] != 2 or combo['size'] != 1:
        raise ValueError('Unexpected lane metadata size')
    # Follow resolutionInfo -> resolutionData -> regSetting rather than
    # assuming symbol IDs. This layout is specific to the captured module.
    resolution = unique('resolutionInfo')
    if resolution['size'] != 12:
        raise ValueError('Unexpected resolutionInfo size')
    _, count, ref = container.ints(resolution['file_offset'], 3)
    mode = container.symbols[ref]
    if count != 1 or mode['name'] != 'resolutionData' or mode['size'] != 112:
        raise ValueError('Unsupported resolution layout')
    count, ref = container.ints(mode['file_offset'] + 72, 2)
    registers = container.symbols[ref]
    if registers['name'] != 'regSetting' or registers['size'] != count * 40:
        raise ValueError('Unexpected register array')
    rows = []
    for offset in range(registers['file_offset'], registers['file_offset'] + registers['size'], 40):
        words = container.ints(offset, 10)
        value = container.symbols[words[4]]
        if words[3] != 1 or value['name'] != 'registerData' or value['size'] != 4:
            raise ValueError('Unsupported register data')
        rows.append({'address': words[2], 'value': container.ints(value['file_offset'], 1)[0],
                     'file_offset': offset})
    return {'source': path.name, 'sha256': module.hashlib.sha256(container.data).hexdigest(),
            'lane_assign': int.from_bytes(container.block(lane['file_offset'], 2), 'little'),
            'combo_mode': bool(container.block(combo['file_offset'], 1)[0]),
            'mode_registers': rows}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    print(json.dumps(decode(args.binary), indent=2))
