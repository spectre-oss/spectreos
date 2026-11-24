#!/usr/bin/env python3
"""Strip Android nine-patch metadata from the expandable-list indicators."""
from pathlib import Path
from android_png import decode_png

root = Path(__file__).resolve().parents[1] / 'codes/libs/ui/libbhoo/assets/android-expandable-list'
for name in ('open', 'close'):
    width, height, data = decode_png(root / f'expander_{name}_mtrl_alpha.9.png')
    if (width, height) != (33, 33):
        raise ValueError(f'Unexpected indicator dimensions: {width}x{height}')
    pixels = b''.join(data[(y * width + 1) * 4:(y * width + 32) * 4] for y in range(1, 32))
    (root / f'{name}.rgba').write_bytes(pixels)
