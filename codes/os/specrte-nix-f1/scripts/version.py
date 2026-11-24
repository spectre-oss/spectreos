#!/usr/bin/env python3
"""Derive project SemVer from first-parent Conventional Commits."""
import argparse
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def calculate(messages):
    version = [0, 0, 1]
    for message in messages:
        message = message.strip()
        lines = message.splitlines()
        if not lines:
            continue
        match = re.match(r'([a-z]+)(?:\([^\n)]+\))?(!)?: .+', lines[0])
        if not match:
            continue
        kind, breaking = match.groups()
        if breaking or re.search(r'^BREAKING[ -]CHANGE: ', message, re.M):
            version = [version[0] + 1, 0, 0]
        elif kind == 'feat':
            version = [version[0], version[1] + 1, 0]
        elif kind in ('fix', 'perf', 'revert'):
            version[2] += 1
    return '.'.join(map(str, version))

def git(*args, **kwargs):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], **kwargs).decode().strip()

def current():
    return calculate(git('log', '--first-parent', '--reverse', '--format=%B%x00').split('\0'))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--commit', action='store_true', help='write and commit VERSION when changed')
    args = parser.parse_args()
    version = current()
    if args.commit:
        target = ROOT / 'VERSION'
        if target.exists() and target.read_text().strip() == version:
            return
        target.write_text(version + '\n')
        env = os.environ.copy()
        env['SPECTRE_VERSION_HOOK'] = '1'
        # Match the triggering commit's identity; never change global Git settings.
        for role in ('AUTHOR', 'COMMITTER'):
            env['GIT_' + role + '_NAME'] = git('log', '-1', '--format=%an')
            env['GIT_' + role + '_EMAIL'] = git('log', '-1', '--format=%ae')
        subprocess.run(['git', '-C', str(ROOT), '-c', 'commit.gpgsign=false',
                        'add', '--', 'VERSION'], check=True)
        subprocess.run(['git', '-C', str(ROOT), '-c', 'commit.gpgsign=false',
                        'commit', '--only', '-m', 'chore(release): ' + version,
                        '--', 'VERSION'], check=True, env=env)
    else:
        print(version)

if __name__ == '__main__':
    main()
