#!/usr/bin/env python3
"""Explicit, baseline-based two-way text-source sync. No builds or flashing."""
import argparse
import base64
import fcntl
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1] if '__file__' in globals() else Path.cwd()
IGNORED_DIRS = {'.git', '.agents', '.codex', '.aws', '.code-sync', '.build',
                'nix-store', 'nix', 'dist', 'artifacts', 'node_modules',
                '__pycache__', '.pytest_cache', '.venv', 'venv'}
UPSTREAM = {'mobile-nixos', 'u-boot', 'phosh'}
IGNORED_FILES = ['result*', '*.log', '*.tar', '*.tar.*', '*.tgz', '*.tbz2',
                 '*.txz', '*.zip', '*.7z', '*.gz', '*.xz', '*.zst', '*.img',
                 '*.img.*', '*.bin', '*.mbn', '*.fw', '*.elf', '*.dtb', '*.dtbo',
                 '*.ko', '*.ko.*', 'vmlinux', 'vmlinuz*', 'Image', 'Image.*',
                 'System.map', '*.pyc', '*.pyo', '*.swp', '*.swo', '*~',
                 '.env', '.env.*', '*.pem', '*.key', 'id_rsa*', 'id_ed25519*']


def safe_path(root, name):
    p = Path(name)
    if p.is_absolute() or not p.parts or any(x in ('.', '..') for x in p.parts):
        raise RuntimeError('Unsafe path: ' + name)
    target = root / p
    for parent in [target, *target.parents]:
        if parent == root:
            break
        if parent.is_symlink():
            raise RuntimeError('Refusing symlink path: ' + name)
    return target


def record(path):
    if not path.exists():
        return None
    if path.is_symlink() or not path.is_file():
        raise RuntimeError('Expected regular file: ' + str(path))
    data = path.read_bytes()
    return {'hash': hashlib.sha256(data).hexdigest(),
            'executable': bool(path.stat().st_mode & stat.S_IXUSR),
            'data': base64.b64encode(data).decode()}


def signature(item):
    return None if item is None else [item['hash'], item['executable']]


def scan(root, excluded_top=()):
    files, skipped = {}, []
    for directory, dirs, names in os.walk(root, followlinks=False):
        rel = Path(directory).relative_to(root)
        for name in dirs:
            if (Path(directory) / name).is_symlink() and name not in IGNORED_DIRS and not (rel == Path('.') and name in excluded_top):
                skipped.append(((rel / name).as_posix()) + '/')
        dirs[:] = sorted(d for d in dirs if d not in IGNORED_DIRS
                         and not (rel == Path('.') and d in excluded_top)
                         and not (rel == Path('.') and d in UPSTREAM)
                         and not (Path(directory) / d).is_symlink())
        for name in sorted(names):
            p = Path(directory) / name
            key = p.relative_to(root).as_posix()
            if any(fnmatch.fnmatchcase(name, pat) for pat in IGNORED_FILES):
                continue
            if p.is_symlink():
                skipped.append(key)
                continue
            if p.stat().st_size > 10 * 1024 * 1024:
                skipped.append(key)
                continue
            item = record(p)
            data = base64.b64decode(item['data'])
            try:
                data.decode('utf-8')
                if b'\0' in data:
                    raise ValueError('binary')
            except (UnicodeDecodeError, ValueError):
                skipped.append(key)
                continue
            files[key] = item
    return {'files': files, 'skipped': skipped}


def local_scan(root, docs):
    """Present the split local tree in the server's project-relative namespace."""
    if not docs.is_dir():
        raise RuntimeError('Mapped documentation directory is missing: ' + str(docs))
    result = scan(root, excluded_top={'docs'})
    documents = scan(docs)
    result['files'].update({'docs/' + name: item for name, item in documents['files'].items()})
    result['skipped'].extend('docs/' + name for name in documents['skipped'])
    return result


def local_apply(root, docs, changes, backup):
    batches = {'code': [], 'docs': []}
    # Preflight both mapped destinations before changing either tree.
    for change in changes:
        target = docs if change['name'].startswith('docs/') else root
        mapped = dict(change)
        if target == docs:
            mapped['name'] = change['name'][5:]
        if signature(record(safe_path(target, mapped['name']))) != change['expected']:
            raise RuntimeError('Changed during sync: ' + change['name'])
        batches['docs' if target == docs else 'code'].append(mapped)
    apply(root, batches['code'], backup)
    apply(docs, batches['docs'], backup / 'docs')


def plan(base, local, remote):
    actions, conflicts = [], []
    for name in sorted(set(base) | set(local) | set(remote)):
        left, right = signature(local.get(name)), signature(remote.get(name))
        before = base.get(name)
        if left == right:
            continue
        if left != before and right != before:
            conflicts.append(name)
        elif left != before:
            actions.append(('remote', name, local.get(name), right))
        else:
            actions.append(('local', name, remote.get(name), left))
    return actions, conflicts


def apply(root, changes, backup):
    # Check the whole batch first; check again immediately before each write.
    for change in changes:
        p = safe_path(root, change['name'])
        if signature(record(p)) != change['expected']:
            raise RuntimeError('Changed during sync: ' + change['name'])
    for change in changes:
        name, item = change['name'], change['item']
        p = safe_path(root, name)
        if signature(record(p)) != change['expected']:
            raise RuntimeError('Changed during sync: ' + name)
        if p.exists():
            saved = safe_path(backup, name)
            saved.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(p, saved)
        if item is None:
            p.unlink()
            continue
        data = base64.b64decode(item['data'], validate=True)
        if hashlib.sha256(data).hexdigest() != item['hash']:
            raise RuntimeError('Content hash mismatch: ' + name)
        p.parent.mkdir(parents=True, exist_ok=True)
        fd, temporary = tempfile.mkstemp(prefix='.code-sync-', dir=p.parent)
        try:
            with os.fdopen(fd, 'wb') as stream:
                stream.write(data)
            os.chmod(temporary, 0o755 if item['executable'] else 0o644)
            os.replace(temporary, p)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)


def worker(request):
    root = Path(request['root']).resolve(strict=True)
    state = root / '.code-sync'
    state.mkdir(exist_ok=True, mode=0o700)
    with (state / 'worker.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if request['operation'] == 'apply':
            apply(root, request['changes'], state / 'backups' / request['id'])
            return {'ok': True}
        if request['operation'] == 'scan':
            return scan(root)
        raise RuntimeError('Unknown worker operation')


def config():
    return json.loads((ROOT / 'infra/remote.json').read_text())


def ssh_args(cfg):
    return ['ssh', '-F', '/dev/null', '-i', str(Path(cfg['identity']).expanduser()),
            '-o', 'IdentitiesOnly=yes', '-o', 'BatchMode=yes',
            '-o', 'ConnectTimeout=15', cfg['user'] + '@' + cfg['host']]


def remote(cfg, request):
    # Ship this exact worker implementation; never depend on a stale remote copy.
    source = Path(__file__).read_text()
    request['root'] = cfg['project']
    result = subprocess.run(ssh_args(cfg) + ['python3 -c ' + shlex.quote(source)
                            + ' --worker'], input=json.dumps(request),
                            text=True, stdout=subprocess.PIPE, check=True)
    return json.loads(result.stdout)


def sync(commit):
    cfg = config()
    docs = (ROOT / cfg['local_docs']).resolve()
    state = ROOT / '.code-sync'
    state.mkdir(exist_ok=True, mode=0o700)
    with (state / 'sync.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        baseline = state / 'baseline.json'
        base = json.loads(baseline.read_text()) if baseline.exists() else {}
        left, right = local_scan(ROOT, docs), remote(cfg, {'operation': 'scan'})
        for side, snapshot in [('local', left), ('remote', right)]:
            for name in snapshot['skipped']:
                print('Excluded binary/oversized/symlink file:', side, name)
                if name in base or (name.endswith('/') and any(n.startswith(name) for n in base)):
                    raise RuntimeError('Previously synchronized file is now excluded: '
                                       + side + ' ' + name + '; resolve it before syncing.')
        actions, conflicts = plan(base, left['files'], right['files'])
        for side, name, item, _ in actions:
            print(('DELETE' if item is None else 'WRITE'), side, name)
        if conflicts:
            raise RuntimeError('Conflicts; no files changed. Resolve both copies to identical '
                               'content (or save/rename one copy), then retry:\n'
                               + '\n'.join(conflicts))
        print(f'{len(actions)} changes' + ('' if commit else ' (preview; use --apply)'))
        if not commit:
            return
        run_id = time.strftime('%Y%m%dT%H%M%S') + '-' + str(os.getpid())
        batches = {'local': [], 'remote': []}
        for side, name, item, expected in actions:
            batches[side].append({'name': name, 'item': item, 'expected': expected})
        if batches['remote']:
            remote(cfg, {'operation': 'apply', 'changes': batches['remote'], 'id': run_id})
        local_apply(ROOT, docs, batches['local'], state / 'backups' / run_id)
        final_local = {n: signature(v) for n, v in local_scan(ROOT, docs)['files'].items()}
        final_remote = {n: signature(v) for n, v in remote(cfg, {'operation': 'scan'})['files'].items()}
        if final_local != final_remote:
            raise RuntimeError('Trees changed during sync; baseline unchanged. Rerun sync.')
        temporary = state / 'baseline.tmp'
        temporary.write_text(json.dumps(final_local, indent=2, sort_keys=True) + '\n')
        os.replace(temporary, baseline)
        print(f'Synchronized {len(final_local)} source files; baseline saved.')


def main():
    if '--worker' in sys.argv:
        print(json.dumps(worker(json.load(sys.stdin))))
        return
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true', help='apply previewed two-way changes')
    args = parser.parse_args()
    sync(args.apply)


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
