#!/usr/bin/env python3
"""Export a versioned U-Boot test package, rollback image and host tools."""
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile
import version

ROOT = Path(__file__).resolve().parents[1]
release, revision = version.current(), version.git('rev-parse', 'HEAD')
base = ROOT.parent / 'dist' / release / 'tianma'
base.mkdir(parents=True, exist_ok=True)
stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
target = base / f'{stamp}-{revision[:12]}-uboot'
staging = Path(tempfile.mkdtemp(prefix='.publishing-', dir=base))
try:
    source = ROOT / 'result-uboot-package'
    files = {name: source / name for name in ['uboot-boot.img', 'rollback-boot.img', 'spectre.itb', 'build-info.json']}
    files.update({
        'install-uboot.py': ROOT / 'scripts/install-uboot.py',
        'uboot-usb.py': ROOT / 'scripts/uboot-usb.py',
        'fastboot_helpers.py': ROOT / 'scripts/release-install.py',
        'README.md': ROOT / 'docs/uboot.md',
        'USB-SSH.md': ROOT / 'docs/usb-ssh.md',
    })
    manifest = {'kind': 'uboot', 'panel': 'tianma', 'projectVersion': release, 'revision': revision,
                'upstream': json.loads((ROOT / 'sources.json').read_text())['u-boot'],
                'hardwareTested': False, 'files': {}}
    for name, path in files.items():
        dest = staging / name
        subprocess.run(['cp', '--reflink=auto', '--sparse=always', str(path.resolve()), str(dest)], check=True)
        digest = hashlib.sha256()
        with dest.open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        manifest['files'][name] = {'bytes': dest.stat().st_size, 'sha256': digest.hexdigest()}
    (staging / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    staging.rename(target)
except BaseException:
    shutil.rmtree(staging)
    raise
archive = target.with_suffix('.zip')
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=3) as z:
    for path in sorted(target.iterdir()):
        z.write(path, Path(target.name) / path.name)
with zipfile.ZipFile(archive) as z:
    if z.testzip() is not None:
        raise RuntimeError('ZIP integrity check failed')
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix('.zip.sha256').write_text(digest + '  ' + archive.name + '\n')
print(target)
print(archive)
