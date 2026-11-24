#!/usr/bin/env python3
"""Verify a complete distribution before pruning obsolete partial artifacts."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def verify_release(directory):
    manifest = json.loads((directory / 'manifest.json').read_text())
    required = {'boot.img', 'system.img', 'install.py', 'INSTALL.md', 'kernel/build.config'}
    if manifest.get('kind') != 'system' or manifest.get('panel') != 'tianma':
        raise ValueError('A complete Tianma system release is required')
    files = manifest.get('files', {})
    if not required <= files.keys() or not any(x.startswith('kernel/') and ('vmlinuz' in x or '/Image' in x) for x in files):
        raise ValueError('Release is missing required images, kernel or installer')
    for name, info in files.items():
        relative = Path(name)
        path = directory / relative
        if relative.is_absolute() or '..' in relative.parts or path.is_symlink() or not path.resolve().is_relative_to(directory.resolve()):
            raise ValueError('Invalid release file path')
        if path.stat().st_size != info['bytes']:
            raise ValueError('Release size mismatch: ' + name)
        digest = hashlib.sha256()
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        if digest.hexdigest() != info['sha256']:
            raise ValueError('Release checksum mismatch: ' + name)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('supervisor', ROOT / 'scripts/build-supervisor.py')
    supervisor = importlib.util.module_from_spec(spec); spec.loader.exec_module(supervisor)
    if supervisor.active_builds():
        raise RuntimeError('Refusing cleanup while project builds are active')
    dist = ROOT.parent / 'dist'
    candidates = []
    for path in dist.glob('*/tianma/*/manifest.json'):
        if path.is_symlink() or not path.resolve().is_relative_to(dist.resolve()):
            continue
        manifest = json.loads(path.read_text())
        if manifest.get('kind') == 'system':
            candidates.append((manifest.get('createdAt', ''), path.parent))
    if not candidates:
        raise RuntimeError('No complete distribution exists; nothing will be deleted')
    release = max(candidates)[1]
    manifest = verify_release(release)
    subprocess.run(['python3', 'scripts/inspect-images.py', str(release), '--panel', 'tianma'], cwd=ROOT, check=True)
    # Keep complete releases and source archives. Only obsolete partial exports
    # older than the verified release and archived logs are eligible for pruning.
    obsolete = []
    for path in dist.glob('*/tianma/*/manifest.json'):
        if path.is_symlink() or not path.resolve().is_relative_to(dist.resolve()):
            continue
        data = json.loads(path.read_text())
        if (data.get('kind') in ('kernel', 'boot') and data.get('createdAt', '') < manifest['createdAt']
                and re.fullmatch(r'\d+\.\d+\.\d+', path.parents[2].name)):
            obsolete.append(path.parent)
    logs = sorted((ROOT / '.build').glob('image-*.log'), key=lambda p: p.stat().st_mtime, reverse=True)
    obsolete.extend(p for p in logs[5:] if p.is_file() and not p.is_symlink())
    if args.apply:
        # Ensure output roots still exist before allowing Nix to collect only
        # unreachable paths. Do not manually remove anything inside the store.
        for name in ('result-tianma', 'result-kernel'):
            root = ROOT / '.build/roots' / name
            if not root.is_symlink():
                raise RuntimeError('Missing GC root: ' + str(root))
            logical = Path(root.readlink())
            physical = ROOT.parent / 'nix-store' / str(logical).lstrip('/')
            if not physical.exists() and not logical.exists():
                raise RuntimeError('Broken GC root: ' + str(root))
        for path in obsolete:
            if path.is_dir():
                shutil.rmtree(path)
            else:
                path.unlink()
        subprocess.run(['nix-store', '--store', str(ROOT.parent / 'nix-store'), '--option', 'build-users-group', '', '--gc'], check=True)
        # Verify the portable release again after cleanup, independently of store links.
        verify_release(release)
        supervisor.save(ROOT / '.build/completion.json', {'release': str(release), 'manifest': manifest, 'cleanup': 'complete'})
    print(json.dumps({'release': str(release), 'cleanupApplied': args.apply, 'obsoleteArtifacts': len(obsolete)}))


if __name__ == '__main__':
    main()
