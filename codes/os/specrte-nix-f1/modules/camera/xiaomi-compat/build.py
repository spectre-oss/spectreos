#!/usr/bin/env python3
"""Cross-build Xiaomi camera compat bundle; original ROM blobs stay out of source."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
RESEARCH = Path(os.environ.get('CAMERA_RESEARCH_DIR', str(HERE.parents[3] / 'camera-research')))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--research', type=Path, default=RESEARCH)
    parser.add_argument('--ndk', type=Path, default=RESEARCH / 'tools/android-ndk-r30')
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    required_host = os.environ.get('SPECTRE_BUILD_HOST', '').strip()
    if required_host and socket.gethostname() != required_host:
        parser.error(f'Builds must run on configured build host ({required_host}).')
    output = args.out.resolve()
    if output.exists():
        parser.error('Output already exists; choose a new bundle directory.')
    output.parent.mkdir(parents=True, exist_ok=True)
    compiler = args.ndk / 'toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi29-clang'
    pins = json.loads((HERE / 'blobs.json').read_text())
    with tempfile.TemporaryDirectory(dir=output.parent, prefix='.xiaomi-build-') as td:
        stage = Path(td) / 'bundle'
        (stage / 'lib').mkdir(parents=True)
        for relative, expected in pins.items():
            source = args.research / 'original' / relative
            if digest(source) != expected:
                raise RuntimeError(f'Original blob hash mismatch: {source}')
            dest = stage / ('linker' if source.name == 'linker' else 'lib/' + source.name)
            shutil.copyfile(source, dest)
        (stage / 'linker').chmod(0o755)
        command = [str(compiler), '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror',
                   '-fPIE', '-pie', '-fstack-protector-strong', '-D_FORTIFY_SOURCE=2',
                   '-Wl,-z,relro,-z,now', str(HERE / 'helper.c'), '-ldl', '-lm',
                   '-o', str(stage / 'xiaomi-camera-helper')]
        subprocess.run(command, check=True)
        shutil.copyfile(HERE / 'client.py', stage / 'client.py')
        shutil.copyfile(HERE / 'run.sh', stage / 'run')
        (stage / 'run').chmod(0o755)
        manifest = {
            'schema': 1, 'camera': 'beryllium-imx363-sunny', 'backend': 'arcsoft-hdr',
            'compiler': subprocess.check_output([str(compiler), '--version'], text=True),
            'source_sha256': {p.name: digest(p) for p in [HERE / name for name in ['helper.c', 'arcsoft-abi.h', 'client.py', 'run.sh', 'build.py', 'blobs.json']]},
            'files': {str(p.relative_to(stage)): digest(p) for p in sorted(stage.rglob('*')) if p.is_file()},
            'original_blobs': pins,
        }
        (stage / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        checksums = {**manifest['files'], 'manifest.json': digest(stage / 'manifest.json')}
        (stage / 'SHA256SUMS').write_text(''.join(f'{value}  {name}\n' for name, value in sorted(checksums.items())))
        os.rename(stage, output)
    print(output)


if __name__ == '__main__':
    main()
