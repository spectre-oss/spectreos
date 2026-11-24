#!/usr/bin/env python3
"""Fetch the server-built camera and only runtime paths absent from this phone."""
import argparse
import getpass
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess

from sync import ROOT, config, ssh_args


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', choices=['megapixels', 'qt'], default='megapixels')
    parser.add_argument('--full', action='store_true', help='export complete closure without contacting phone')
    parser.add_argument('--host', default='spectre@172.16.42.1')
    parser.add_argument('--known-hosts', type=Path)
    parser.add_argument('--password-env')
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    if not args.full and not args.known_hosts:
        parser.error('--known-hosts is required unless --full is used')
    args.out = args.out or ROOT / ('artifacts/qt-camera' if args.app == 'qt' else 'artifacts/camera-live')
    if not args.full:
        password = os.environ[args.password_env] if args.password_env else getpass.getpass('Phone password: ')
        env = dict(os.environ, SSHPASS=password)
        phone = ['sshpass', '-e', 'ssh', '-F', '/dev/null', '-o', 'ConnectTimeout=15',
                 '-o', 'UserKnownHostsFile=' + str(args.known_hosts.resolve()), args.host]
    cfg = config()
    server = ssh_args(cfg)
    server.insert(1, '-C')  # Compress large NAR streams in transit, not on disk.
    store = str(Path(cfg['project']).parent / 'nix-store')
    app = subprocess.check_output(server + ['readlink ' + shlex.quote(
        cfg['project'] + ('/.build/roots/result-qt-camera' if args.app == 'qt' else '/.build/roots/result-camera'))], text=True).strip()
    if not app.startswith('/nix/store/') or '\n' in app:
        raise RuntimeError('Unexpected server camera output')
    closure = subprocess.check_output(server + ['nix-store --store ' + shlex.quote(store)
                                              + ' -qR ' + shlex.quote(app)], text=True).splitlines()
    existing = set() if args.full else {'/nix/store/' + name for name in subprocess.check_output(
        phone + ['ls -1 /nix/store'], env=env, text=True).splitlines()}
    missing = [p for p in closure if p not in existing]
    args.out.mkdir(parents=True, exist_ok=True)
    nar = args.out / 'camera.nar'
    temporary = args.out / 'camera.nar.part'
    print(f'Closure: {len(closure)} paths; transferring {len(missing)} missing paths.', flush=True)
    if not missing:
        raise RuntimeError('Camera closure already installed; no new artifact needed')
    with temporary.open('wb') as stream:
        subprocess.run(server + ['nix-store --store ' + shlex.quote(store)
                                 + ' --export ' + shlex.join(missing)], stdout=stream, check=True)
    temporary.replace(nar)
    with nar.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    data = {'app': app, 'paths': closure, 'missing': missing, 'server': cfg['host'], 'kind': args.app, 'full': args.full,
            'camera.nar': {'bytes': nar.stat().st_size, 'sha256': digest}}
    (args.out / 'closure.json').write_text(json.dumps(data, indent=2) + '\n')
    print('Exported:', nar, nar.stat().st_size, 'bytes')


if __name__ == '__main__':
    main()
