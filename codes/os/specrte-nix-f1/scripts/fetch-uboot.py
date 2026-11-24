#!/usr/bin/env python3
"""Fetch the pinned U-Boot checkout and create a deterministic Nix source archive."""
import gzip
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
pin = json.loads((root / 'sources.json').read_text())['u-boot']
checkout = root / 'u-boot'
if not checkout.exists():
    subprocess.run(['git', 'init', str(checkout)], check=True)
    subprocess.run(['git', '-C', str(checkout), 'remote', 'add', 'origin', pin['url']], check=True)
try:
    subprocess.run(['git', '-C', str(checkout), 'cat-file', '-e', pin['revision'] + '^{commit}'], check=True, stderr=subprocess.DEVNULL)
except subprocess.CalledProcessError:
    subprocess.run(['git', '-C', str(checkout), 'fetch', '--depth', '1', pin['url'], pin['revision']], check=True)
if subprocess.check_output(['git', '-C', str(checkout), 'status', '--porcelain'], text=True).strip():
    raise SystemExit('Refusing to change a dirty U-Boot checkout')
subprocess.run(['git', '-C', str(checkout), 'checkout', '--detach', pin['revision']], check=True)
archive = subprocess.check_output(['git', '-C', str(checkout), 'archive', '--format=tar', '--prefix=u-boot/', pin['revision']])
(root / 'bootloader/u-boot.tar.gz').write_bytes(gzip.compress(archive, mtime=0))
