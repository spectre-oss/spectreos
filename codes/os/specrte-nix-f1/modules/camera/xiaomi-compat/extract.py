#!/usr/bin/env python3
"""Read-only extraction of the exact camera runtime from already reconstructed ROM images."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ.get('CAMERA_RESEARCH_DIR', str(HERE.parents[3] / 'camera-research')))
APEX = '/system/apex/com.android.runtime.release'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--research', type=Path, default=ROOT)
    parser.add_argument('--vendor-image', type=Path, default=Path('/tmp/spectre-vendor-colour/vendor.img'))
    parser.add_argument('--system-image', type=Path, default=ROOT / 'original/system.img')
    args = parser.parse_args()
    required_host = os.environ.get('SPECTRE_BUILD_HOST', '').strip()
    if required_host and socket.gethostname() != required_host:
        parser.error(f'Extract on the designated build host ({required_host}).')
    pins = json.loads((HERE / 'blobs.json').read_text())
    for relative, expected in pins.items():
        family, name = relative.split('/')
        if family == 'vendor':
            image, source = args.vendor_image, '/lib/' + name
        else:
            image = args.system_image
            source = (APEX + '/bin/linker' if name == 'linker' else
                      APEX + '/lib/bionic/' + name if name in ('libc.so', 'libm.so', 'libdl.so')
                      else '/system/lib/' + name)
        destination = args.research / 'original' / relative
        if destination.exists():
            if hashlib.sha256(destination.read_bytes()).hexdigest() != expected:
                raise RuntimeError(f'Refusing to replace unexpected existing file: {destination}')
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='spectre-rom-') as td:
            output = Path(td) / 'blob'
            subprocess.run(['debugfs', '-R', f'dump {source} {output}', str(image)],
                           check=True, capture_output=True)
            data = output.read_bytes()
            if hashlib.sha256(data).hexdigest() != expected:
                raise RuntimeError(f'ROM version/hash mismatch: {source}')
            with destination.open('xb') as target:
                target.write(data)
    print(f'Verified {len(pins)} original files; images were never mounted or modified.')


if __name__ == '__main__':
    main()
