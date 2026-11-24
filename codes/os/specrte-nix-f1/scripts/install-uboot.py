#!/usr/bin/env python3
"""Test/install the Spectre U-Boot boot image without touching userdata or ABL."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from fastboot_helpers import Fastboot


def verify(root):
    manifest = json.loads((root / 'manifest.json').read_text())
    if manifest.get('kind') != 'uboot' or manifest.get('panel') != 'tianma':
        raise RuntimeError('Expected the Poco F1 Tianma U-Boot package')
    for required in ('uboot-boot.img', 'rollback-boot.img', 'spectre.itb'):
        if required not in manifest['files']:
            raise RuntimeError('Missing manifest entry: ' + required)
    for name, info in manifest['files'].items():
        path = root / name
        if path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
            raise RuntimeError('Unsafe release path')
        digest = hashlib.sha256()
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
        if path.stat().st_size != info['bytes'] or digest.hexdigest() != info['sha256']:
            raise RuntimeError('Integrity check failed: ' + name)
    for name in ('uboot-boot.img', 'rollback-boot.img'):
        with (root / name).open('rb') as stream:
            if stream.read(8) != b'ANDROID!':
                raise RuntimeError('Invalid boot image: ' + name)
    return manifest


def perform(phone, root, action, tested=False):
    if not re.search(r'product:\s*beryllium(?:\s|$)', phone.device('getvar', 'product')):
        raise RuntimeError('Connected device is not a Poco F1')
    if not re.search(r'Device unlocked:\s*true', phone.device('oem', 'device-info'), re.I):
        raise RuntimeError('An already unlocked bootloader is required')
    output = phone.device('getvar', 'partition-size:boot')
    match = re.search(r'partition-size:boot:\s*(0x[0-9a-fA-F]+|[0-9]+)', output)
    if not match:
        raise RuntimeError('Cannot verify boot capacity')
    capacity = int(match[1], 16 if match[1].startswith('0x') else 10)
    if max((root / n).stat().st_size for n in ('uboot-boot.img', 'rollback-boot.img')) > capacity:
        raise RuntimeError('Image exceeds boot capacity')
    if action == 'check':
        print('Files and device checked; nothing written.')
    elif action == 'test':
        print('Temporarily starting U-Boot. No partition is written.')
        print(phone.device('boot', str(root / 'uboot-boot.img')))
        print('Configure USB Ethernet, then run: python3 uboot-usb.py console')
    else:
        if action == 'install' and not tested:
            raise RuntimeError('First test USB control and Linux boot on this phone; then pass --tested-on-phone')
        phrase = 'INSTALL tested UBOOT beryllium' if action == 'install' else 'RESTORE beryllium'
        print('This replaces only boot. Userdata and Xiaomi ABL/XBL are preserved.')
        if input(f'Type {phrase}: ').strip() != phrase:
            raise RuntimeError('Cancelled; no partitions written')
        filename = 'uboot-boot.img' if action == 'install' else 'rollback-boot.img'
        print(phone.device('flash', 'boot', str(root / filename)))
        print(phone.device('reboot'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['check', 'test', 'install', 'rollback'])
    parser.add_argument('--serial')
    parser.add_argument('--tested-on-phone', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    verify(root)
    phone = Fastboot()
    phone.select(args.serial)
    perform(phone, root, args.action, args.tested_on_phone)


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, OSError, ValueError, KeyError, EOFError, KeyboardInterrupt) as error:
        sys.exit(str(error) or 'Interrupted')
