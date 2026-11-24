#!/usr/bin/env python3
"""Watch a Linux build PID and play a remote desktop sound once it exits."""
import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import time

REMOTE = r'''
import math, os, shutil, struct, subprocess, tempfile, wave
from pathlib import Path
runtime = Path('/run/user') / str(os.getuid())
if runtime.is_dir():
    os.environ.setdefault('XDG_RUNTIME_DIR', str(runtime))
    os.environ.setdefault('PULSE_SERVER', 'unix:' + str(runtime / 'pulse/native'))
with tempfile.TemporaryDirectory(prefix='build-alert-') as directory:
    sound = Path(directory) / 'alert.wav'
    with wave.open(str(sound), 'wb') as output:
        output.setparams((1, 2, 22050, 0, 'NONE', 'not compressed'))
        samples = [int(10000 * math.sin(2 * math.pi * 880 * i / 22050))
                   if i % 11025 < 6615 else 0 for i in range(33075)]
        output.writeframes(struct.pack('<' + 'h' * len(samples), *samples))
    for player in ('paplay', 'pw-play', 'aplay'):
        if shutil.which(player):
            if subprocess.run([player, str(sound)], timeout=15).returncode == 0:
                break
    else:
        raise SystemExit('No working desktop audio player/session found')
'''


def identity(pid):
    """Linux start time avoids mistaking a reused PID for the original build."""
    try:
        fields = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
        return None if fields[0] == 'Z' else fields[19]
    except FileNotFoundError:
        return None


def beep(host):
    return subprocess.run(
        ['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=8',
         '--', host, 'python3 -c ' + shlex.quote(REMOTE)], timeout=60
    ).returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', required=True, help='SSH address of the desktop PC')
    parser.add_argument('--pid', type=int, help='Build process to watch')
    parser.add_argument('--test', action='store_true', help='Play the sound immediately')
    parser.add_argument('--interval', type=float, default=10)
    parser.add_argument('--state', type=Path, default=Path('.build/alert-state.json'))
    args = parser.parse_args()
    if args.test:
        return beep(args.host)
    if not args.pid or args.pid < 1 or args.interval <= 0:
        parser.error('provide a positive --pid and --interval')
    original = identity(args.pid)
    args.state.parent.mkdir(parents=True, exist_ok=True)
    def record(status, **extra):
        args.state.write_text(json.dumps(dict(pid=args.pid, host=args.host,
            status=status, time=time.time(), **extra), indent=2) + '\n')
    record('watching')
    while original is not None and identity(args.pid) == original:
        time.sleep(args.interval)
    record('stopped; notifying')
    # Preserve the event through a temporarily unreachable/offline desktop.
    while True:
        try:
            result = beep(args.host)
        except (OSError, subprocess.TimeoutExpired) as error:
            record('notification pending', error=str(error))
        else:
            if result == 0:
                record('notified')
                return 0
            record('notification pending', exit_code=result)
        time.sleep(60)


if __name__ == '__main__':
    raise SystemExit(main())
