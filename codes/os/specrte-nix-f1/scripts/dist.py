#!/usr/bin/env python3
"""Publish standalone artifacts without copying Nix closures into dist."""
import argparse
import datetime
import hashlib
import json
import re
from pathlib import Path
import shutil
import subprocess
import tempfile
import version

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('kind', choices=['kernel', 'boot', 'system'])
p.add_argument('--panel', choices=['tianma', 'ebbg'], default='tianma')
p.add_argument('--version', default=None)
p.add_argument('--revision', default=None)
p.add_argument('--dist', type=Path, default=ROOT.parent / 'dist')
a = p.parse_args()
release = a.version or version.current()
revision = a.revision or version.git('rev-parse', 'HEAD')
if not re.fullmatch(r'\d+\.\d+\.\d+', release):
    p.error('version must be MAJOR.MINOR.PATCH')
if not re.fullmatch(r'[0-9a-f]{40,64}', revision):
    p.error('revision must be a full Git commit hash')
base = a.dist.resolve() / release / a.panel
base.mkdir(parents=True, exist_ok=True)
stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
target = base / (stamp + '-' + revision[:12] + '-' + a.kind)
staging = Path(tempfile.mkdtemp(prefix='.publishing-', dir=base))
try:
    kernel = ROOT / 'result-kernel'
    files = {}
    if a.kind in ('kernel', 'system'):
        candidates = list(kernel.glob('vmlinuz*')) + list(kernel.glob('Image*'))
        if len(candidates) != 1:
            raise RuntimeError('Build kernel first; expected one kernel image')
        files['kernel/' + candidates[0].name] = candidates[0]
        files['kernel/build.config'] = kernel / 'build.config'
        if not (kernel / ('dtbs/qcom/sdm845-xiaomi-beryllium-' + a.panel + '.dtb')).is_file():
            raise RuntimeError('Missing panel device tree')
        for dtb in (kernel / 'dtbs/qcom').glob('sdm845-xiaomi-beryllium-*.dtb'):
            files['kernel/' + dtb.name] = dtb
    if a.kind == 'system':
        source = ROOT / ('result-' + a.panel)
        files.update({'boot.img': source / 'boot.img', 'system.img': source / 'system.img'})
    elif a.kind == 'boot':
        files['boot.img'] = ROOT / ('result-boot-' + a.panel)
    if a.kind in ('boot', 'system'):
        files['USB-SSH.md'] = ROOT / 'docs/usb-ssh.md'
    files['install.py'] = ROOT / 'scripts/release-install.py'
    files['INSTALL.md'] = ROOT / ('scripts/INSTALL-BOOT.md' if a.kind == 'boot' else 'scripts/INSTALL.md')
    manifest = {'projectVersion': release, 'revision': revision, 'panel': a.panel,
                'kind': a.kind, 'createdAt': stamp, 'files': {}}
    for name, source in files.items():
        dest = staging / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        # Reflink when supported, sparse copy otherwise. Never retain store symlinks.
        subprocess.run(['cp', '--reflink=auto', '--sparse=always', '--', str(source.resolve()), str(dest)], check=True)
        digest = hashlib.sha256()
        with dest.open('rb') as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(chunk)
        manifest['files'][name] = {'bytes': dest.stat().st_size, 'sha256': digest.hexdigest()}
    (staging / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    staging.rename(target)
    print(target)
except BaseException:
    shutil.rmtree(staging)
    raise
