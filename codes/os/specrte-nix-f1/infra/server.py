#!/usr/bin/env python3
"""Build on the Tailscale server, or download a published release separately."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import subprocess
import sys

from sync import ROOT, config, safe_path, ssh_args

TARGETS = ['eval', 'test', 'kernel', 'boot-image', 'build', 'phone-debug', 'camera', 'camera-eeprom', 'qt-camera', 'qt-camera-test',
           'uboot', 'uboot-package']


def verify_release(directory):
    manifest = json.loads((directory / 'manifest.json').read_text())
    if not manifest.get('files'):
        raise RuntimeError('Release manifest has no files')
    for name, expected in manifest['files'].items():
        path = safe_path(directory, name)
        digest = hashlib.sha256()
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        if path.stat().st_size != expected['bytes'] or digest.hexdigest() != expected['sha256']:
            raise RuntimeError('Release verification failed: ' + name)
    return len(manifest['files'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    build = commands.add_parser('build', help='sync code, then run make ONLY on server')
    build.add_argument('target', choices=TARGETS)
    build.add_argument('--panel', choices=['tianma', 'ebbg'], default='tianma')
    build.add_argument('--jobs', type=int, default=3)
    build.add_argument('--max-jobs', type=int, default=2)
    commands.add_parser('releases', help='list published release manifests on server')
    fetch = commands.add_parser('fetch', help='download one release; never install or flash')
    fetch.add_argument('release', help='relative release directory: VERSION/PANEL/BUILD-ID')
    args = parser.parse_args()
    cfg = config()
    if args.command == 'build':
        if args.jobs < 1 or args.max_jobs < 1:
            parser.error('job counts must be positive')
        subprocess.run([sys.executable, str(ROOT / 'infra/sync.py'), '--apply'], check=True)
        command = ('cd ' + shlex.quote(cfg['project']) + ' && make '
                   + shlex.join([args.target, 'PANEL=' + args.panel,
                                 'JOBS=' + str(args.jobs), 'MAX_JOBS=' + str(args.max_jobs)]))
        # No local make/nix invocation, and no flash/install targets are accepted.
        subprocess.run(ssh_args(cfg) + [command], check=True)
    elif args.command == 'releases':
        directory = str(Path(cfg['project']).parent / 'dist')
        subprocess.run(ssh_args(cfg) + ['find ' + shlex.quote(directory)
                                      + ' -name manifest.json -type f -print'], check=True)
    else:
        parts = args.release.split('/')
        if len(parts) != 3 or any(not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', p) for p in parts):
            parser.error('release must be VERSION/PANEL/BUILD-ID without traversal or shell characters')
        destination = ROOT / 'artifacts' / args.release
        destination.mkdir(parents=True, exist_ok=True)
        source = str(Path(cfg['project']).parent / 'dist' / args.release) + '/'
        ssh = shlex.join(ssh_args(cfg)[:-1])
        subprocess.run(['rsync', '-rtc', '--safe-links', '--partial', '--protect-args',
                        '-e', ssh, cfg['user'] + '@' + cfg['host'] + ':' + source,
                        str(destination) + '/'], check=True)
        count = verify_release(destination)
        print('Downloaded to', destination)
        print(f'Verified {count} files against release SHA-256 manifest. Nothing installed or flashed.')


if __name__ == '__main__':
    main()
