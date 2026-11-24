#!/usr/bin/env python3
"""Install a Poco F1 release with Python 3.9+ and Android SDK fastboot."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


# -----------------------------------------------------------------------------
# Terminal presentation
# -----------------------------------------------------------------------------

class Console:
    """Readable plain output, with restrained colour on interactive terminals."""

    def __init__(self):
        self.colour = sys.stdout.isatty() and 'NO_COLOR' not in os.environ

    def style(self, text, code):
        return f'\033[{code}m{text}\033[0m' if self.colour else text

    def banner(self, version, panel, check_only):
        print('\n' + self.style('SPECTRE NIX  /  POCO F1', '1;36'))
        print(f'  Release {version}  |  Panel: {panel}')
        print('  Mode: ' + ('Check only' if check_only else 'Install to phone'))
        print('  ' + '-' * 52, flush=True)

    def step(self, number, title):
        print('\n' + self.style(f'[{number}/5] {title}', '1;36'), flush=True)

    def info(self, text):
        print(f'  {text}', flush=True)

    def success(self, text):
        self.info(self.style('OK  ', '32') + text)

    def warning(self, text):
        self.info(self.style('!   ', '1;33') + text)

    def detail(self, output):
        for line in output.strip().splitlines():
            self.info('    ' + line)


def format_size(size):
    for unit in ('B', 'KiB', 'MiB', 'GiB', 'TiB'):
        if size < 1024 or unit == 'TiB':
            return f'{size:.1f} {unit}'
        size /= 1024


# -----------------------------------------------------------------------------
# Release integrity: validate every artifact before contacting the phone
# -----------------------------------------------------------------------------

def verify_release(root, manifest, console):
    if manifest.get('kind') not in ('system', 'boot'):
        raise RuntimeError(
            'This is a partial kernel/boot release. Copy a complete system release '
            'containing boot.img and system.img.'
        )
    for required in (('boot.img',) if manifest['kind'] == 'boot' else ('boot.img', 'system.img')):
        if required not in manifest['files']:
            raise RuntimeError('Manifest is missing ' + required)

    for name, entry in manifest['files'].items():
        path = root / name
        if not path.resolve().is_relative_to(root) or path.is_symlink():
            raise RuntimeError('Unsafe artifact path: ' + name)
        console.info(f'Checking {name} ({format_size(entry["bytes"])}) ...')
        digest = hashlib.sha256()
        with path.open('rb') as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(chunk)
        if path.stat().st_size != entry['bytes'] or digest.hexdigest() != entry['sha256']:
            raise RuntimeError('Checksum/size mismatch: ' + name)
        console.success(name)

    with (root / 'boot.img').open('rb') as stream:
        if stream.read(8) != b'ANDROID!':
            raise RuntimeError('Invalid Android boot image')
    if manifest['kind'] == 'boot':
        console.success('Recovery boot image verified; existing filesystem will be preserved.')
        return
    with (root / 'system.img').open('rb') as stream:
        stream.seek(1024 + 56)
        if stream.read(2) != b'\x53\xef':
            raise RuntimeError('Expected raw ext4 root filesystem')
        stream.seek(1024 + 120)
        if stream.read(16).rstrip(b'\0') != b'NIXOS_SYSTEM':
            raise RuntimeError('Unexpected root filesystem label')
    console.success('Image formats and root filesystem label verified.')


# -----------------------------------------------------------------------------
# Fastboot connection and device checks
# -----------------------------------------------------------------------------

class Fastboot:
    """Keep every device operation bound to the explicitly selected serial."""

    def __init__(self):
        self.binary = shutil.which('fastboot')
        self.serial = None
        if not self.binary:
            raise RuntimeError(
                'Install Android SDK Platform-Tools on this laptop and put fastboot on PATH.'
            )

    def run(self, *arguments):
        command = arguments[2] if arguments[:1] == ('-s',) else arguments[0]
        timeout = 600 if command == 'flash' else 180 if command == 'boot' else 15
        try:
            result = subprocess.run(
                [self.binary, *arguments], text=True, capture_output=True, timeout=timeout
            )
        except subprocess.TimeoutExpired as error:
            raise RuntimeError(
                f'fastboot {command} timed out after {timeout}s. '
                'Check the USB connection and restart fastboot on the phone. '
                'The operation was not confirmed; it will not be retried automatically.'
            ) from error
        output = result.stdout + result.stderr
        if result.returncode:
            raise RuntimeError('fastboot ' + ' '.join(arguments[:3]) + ' failed:\n' + output)
        return output

    def select(self, requested_serial):
        devices = [
            fields[0] for line in self.run('devices').splitlines()
            if len(fields := line.split()) >= 2 and fields[1] == 'fastboot'
        ]
        if requested_serial:
            if requested_serial not in devices:
                raise RuntimeError('Selected phone is not connected')
            self.serial = requested_serial
        elif len(devices) == 1:
            self.serial = devices[0]
        else:
            raise RuntimeError('Connect exactly one phone, or pass --serial SERIAL')

    def device(self, *arguments):
        if self.serial is None:
            raise RuntimeError('Select a phone before running a device command')
        return self.run('-s', self.serial, *arguments)


def verify_phone(phone, root, console, boot_only=False):
    if not re.search(r'product:\s*beryllium(?:\s|$)', phone.device('getvar', 'product')):
        raise RuntimeError('Refusing: this is not a Poco F1 (beryllium)')
    console.success(f'Poco F1 detected: {phone.serial}')
    if not re.search(r'Device unlocked:\s*true', phone.device('oem', 'device-info'), re.I):
        raise RuntimeError('Bootloader must already be unlocked; this script does not unlock it')
    console.success('Bootloader is unlocked.')

    partitions = [('boot', 'boot.img')]
    if not boot_only:
        partitions.append(('userdata', 'system.img'))
    for partition, filename in partitions:
        output = phone.device('getvar', 'partition-size:' + partition)
        match = re.search(
            r'partition-size:' + partition + r':\s*(0x[0-9a-fA-F]+|[0-9]+)', output
        )
        if not match:
            raise RuntimeError('Cannot verify ' + partition + ' partition capacity')
        capacity = int(match[1], 16 if match[1].startswith('0x') else 10)
        size = (root / filename).stat().st_size
        if size > capacity:
            raise RuntimeError(filename + ' exceeds ' + partition + ' capacity')
        console.success(f'{partition}: {format_size(size)} image / {format_size(capacity)} available')


# -----------------------------------------------------------------------------
# Installation: explicit consent, userdata first, boot second, then reboot
# -----------------------------------------------------------------------------

def flash_release(phone, root, manifest, console):
    console.step(4, 'Confirm and install')
    console.info(f'Release: {manifest["projectVersion"]}  |  Phone: {phone.serial}')
    boot_only = manifest['kind'] == 'boot'
    console.warning('This replaces only boot and preserves userdata.' if boot_only else
                    'This ERASES ALL Android userdata and replaces boot.')
    console.warning(f'Your display panel must match this release: {manifest["panel"]}.')
    if boot_only:
        console.warning('Requires the existing Spectre Nix 0.5.5 filesystem. Sensors are temporarily disabled.')
    else:
        console.info('Make sure your Android data is backed up before continuing.')
    confirmation = 'UPDATE beryllium' if boot_only else 'ERASE beryllium'
    if input(f'\n  Type {confirmation} to install: ').strip() != confirmation:
        raise RuntimeError('Cancelled; no partitions written')

    # Do not replace boot unless the root filesystem transfer succeeds.
    partitions = [('boot', 'boot.img')] if boot_only else [('userdata', 'system.img'), ('boot', 'boot.img')]
    for partition, filename in partitions:
        console.info(f'Writing {filename} to {partition} ... Keep USB connected.')
        console.detail(phone.device('flash', partition, str(root / filename)))
        console.success(f'{partition} written successfully.')

    console.step(5, 'Reboot into Spectre Nix')
    console.detail(phone.device('reboot'))
    console.success('Images written and reboot command accepted.')
    console.info('First boot may take several minutes.')
    console.info('Initial local login: spectre / 123456')
    console.info('Change the password after first boot.')


# -----------------------------------------------------------------------------
# Command-line entry point
# -----------------------------------------------------------------------------

def install():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--serial', help='select one connected fastboot serial')
    parser.add_argument(
        '--check', action='store_true',
        help='check images and phone without writing partitions',
    )
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    manifest = json.loads((root / 'manifest.json').read_text())
    console = Console()
    console.banner(manifest['projectVersion'], manifest['panel'], args.check)

    console.step(1, 'Verify release files')
    verify_release(root, manifest, console)
    phone = Fastboot()

    console.step(2, 'Connect your phone')
    console.info('Power off, hold Volume Down + Power, then connect USB.')
    if not args.check:
        input('\n  Press Enter when the phone shows FASTBOOT: ')
    phone.select(args.serial)

    console.step(3, 'Check phone and partition sizes')
    verify_phone(phone, root, console, boot_only=manifest['kind'] == 'boot')
    if args.check:
        print()
        console.success('Checks passed. No partitions written.')
        console.info(f'Confirm your display panel is {manifest["panel"]} before installing.')
        return

    flash_release(phone, root, manifest, console)


if __name__ == '__main__':
    try:
        install()
    except (RuntimeError, OSError, ValueError, KeyError, EOFError, KeyboardInterrupt) as error:
        message = str(error).strip() or 'Interrupted by user.'
        print('\nStopped: ' + message, file=sys.stderr)
        print('Resolve the issue above before running the installer again.', file=sys.stderr)
        sys.exit(1)
