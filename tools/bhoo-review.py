#!/usr/bin/env python3
"""Package, send and supervise Bhoo's human-review loop through delta 0.1.1."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import signal
import subprocess
import sys
import time
import zipfile

from bhoo_review.store import ROOT, DEFAULT_LEDGER, atomic_json, now, transaction
from bhoo_review.demo_viewport import add_scroll_viewport

CONFIG = ROOT / 'context/bhoo-review/config.json'
HOOK = ROOT / 'tools/bhoo-review-hook.py'


def config():
    return json.loads(CONFIG.read_text())


def delta(conf, *args, timeout=60):
    command = [conf['binary'], *map(str, args), '--data-path', conf['data_path'], '--json']
    result = subprocess.run(command, capture_output=True, text=True, timeout=timeout)
    if result.returncode:
        raise RuntimeError(f'delta {args[0]} failed: {result.stderr[-1500:]}')
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]


def package(args):
    spec = json.loads(args.spec.read_text())
    for key in ['title', 'component', 'purpose', 'changes', 'steps', 'expected', 'limits', 'documentation', 'references', 'validation']:
        if key not in spec or not spec[key]:
            raise ValueError('Missing review field: ' + key)
    if not isinstance(spec['steps'], list) or not all(isinstance(s, str) and s.strip() for s in spec['steps']):
        raise ValueError('Review steps must be nonempty strings')
    if not re.fullmatch('[a-z0-9-]{1,36}', args.test_id):
        raise ValueError('Use a short lowercase test ID')
    output = args.output.resolve()
    if output == args.source.resolve():
        raise ValueError('Review output must differ from source')
    token = secrets.token_hex(24)
    app = 'bhoo-review-' + args.test_id
    public = {**spec, 'test_id': args.test_id, 'token': token}
    with transaction(args.ledger) as state:
        if args.test_id in state['tests']:
            raise ValueError('Test ID already registered; use a new revision ID')
        for issue_id in args.repairs:
            if issue_id not in state['issues'] or state['issues'][issue_id]['status'] == 'resolved':
                raise ValueError('Repair must refer to an unresolved issue: ' + issue_id)
        output.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(args.source) as source, zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as out:
            for entry in source.infolist():
                name = entry.filename
                if name in ('review.json', 'review.js', 'review.css', 'demo.html'):
                    raise ValueError('Source is already a review package')
                if name == 'manifest.toml':
                    content = f'name = {json.dumps("Review · " + spec["title"], ensure_ascii=False)}\ndescription = {json.dumps(spec["purpose"], ensure_ascii=False)}\n'.encode()
                else:
                    content = source.read(name)
                if name == 'index.html' and 'demo_height' in spec:
                    content = add_scroll_viewport(content, spec['demo_height'])
                out.writestr('demo.html' if name == 'index.html' else name, content)
            for name in ('review.html', 'review.js', 'review.css'):
                out.writestr('index.html' if name == 'review.html' else name, (ROOT / 'tools/bhoo_review' / name).read_bytes())
            out.writestr('review.json', json.dumps(public, ensure_ascii=False))
        with zipfile.ZipFile(output) as archive:
            assert archive.testzip() is None
            assert 'webxdc.js' not in archive.namelist()
        state['tests'][args.test_id] = {**public, 'app': app, 'state': 'ready', 'created_at': now(),
            'file': str(output.relative_to(ROOT)), 'sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
            'repairs': args.repairs, 'source_spec': str(args.spec.resolve().relative_to(ROOT))}
        for issue_id in args.repairs:
            state['issues'][issue_id]['retest_ids'].append(args.test_id)
            state['issues'][issue_id]['status'] = 'awaiting_retest'
    print('Queued review test:', args.test_id, output)


def status(args):
    with transaction(args.ledger) as state:
        print(json.dumps({'tests': [{k: t.get(k) for k in ('test_id', 'component', 'state', 'decision', 'deletion')} for t in state['tests'].values()],
                          'unresolved': [i for i in state['issues'].values() if i['status'] != 'resolved']}, indent=2, ensure_ascii=False))


def successful_hook(conf, test, ledger=DEFAULT_LEDGER):
    records = delta(conf, 'app-events', test['app'])
    results = [r for r in records if r.get('type') == 'hook_finished' and r.get('hook_id') == 'review-feedback'
               and r.get('event_seq') in test.get('review_ack_event_seqs', [test.get('review_event_seq')])]
    if any(r.get('success') is True for r in results):
        return True
    if results:
        # The record was saved, but the hook acknowledgment failed. Recover through IPC.
        with transaction(ledger) as state:
            review = next(r for r in state['feedback'] if r['test_id'] == test['test_id'])
        delta(conf, 'app-update', test['app'], json.dumps({'payload': {
            'protocol': 'bhoo-test-review-ack/v1', 'test_id': test['test_id'],
            'review_id': review['review_id'], 'decision': review['decision'], 'saved': True}}))
        return True
    return False


def supervise(args):
    conf = config()
    os.environ['BHOO_REVIEW_LEDGER'] = str(args.ledger.resolve())
    private = Path(conf['data_path']).parent
    private.mkdir(parents=True, exist_ok=True)
    stopping = False
    idle = None
    current_app = None
    scan_index = 0
    def stop(*_):
        nonlocal stopping
        stopping = True
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    with (private / 'supervisor.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        atomic_json(private / 'supervisor.json', {'pid': os.getpid(), 'started_at': now()})
        def stop_idle():
            nonlocal idle
            if idle and idle.poll() is None:
                idle.terminate()
                try: idle.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    idle.kill(); idle.wait(timeout=5)
            idle = None
        def stop_current():
            nonlocal current_app
            if current_app:
                try:
                    delta(conf, 'app-stop', current_app)
                except RuntimeError:
                    # A worker can exit independently. Only suppress the stop
                    # error when the registry confirms it is no longer running.
                    info = delta(conf, 'app-info', current_app)
                    if not info or any(record.get('running') for record in info):
                        raise
                current_app = None
        def start_app(app):
            nonlocal current_app
            if current_app != app:
                stop_idle()
                stop_current()
                delta(conf, 'app-start', app)
                current_app = app
        try:
            # A previous supervisor can leave its independently running app behind.
            with transaction(args.ledger) as state:
                existing = [t['app'] for t in state['tests'].values()
                            if t['state'] == 'awaiting_review' or
                            (t['state'] == 'reviewed' and t.get('deletion') == 'pending')]
            for app in existing:
                info = delta(conf, 'app-info', app)
                if any(record.get('running') for record in info):
                    delta(conf, 'app-stop', app)
            while not stopping:
                try:
                    with transaction(args.ledger) as state:
                        tests = list(state['tests'].values())
                    reviewed = None
                    for candidate in (t for t in tests if t['state'] == 'reviewed' and t.get('deletion') == 'pending'):
                        try:
                            if successful_hook(conf, candidate, args.ledger):
                                reviewed = candidate
                                break
                        except RuntimeError as error:
                            # An acknowledgment may need its worker to be running.
                            # Rotation will retry it; other deliveries can proceed.
                            print('Review acknowledgment pending:', candidate['test_id'], error, file=sys.stderr, flush=True)
                    ready = next((t for t in tests if t['state'] in ('ready', 'sending')), None)
                    # Send all ready tests without waiting for an earlier decision.
                    # Each app is stopped before a direct account command because
                    # delta holds an exclusive account lock for its worker.
                    if reviewed:
                        stop_idle()
                        stop_current()
                        delta(conf, 'delete-message', reviewed['message_id'], '--for-all')
                        with transaction(args.ledger) as state:
                            state['tests'][reviewed['test_id']].update(deletion='requested', deletion_requested_at=now())
                        print('Recorded review and requested deletion:', reviewed['test_id'], flush=True)
                    elif ready:
                        stop_idle()
                        stop_current()
                        app = ready['app']
                        if ready['state'] == 'ready':
                            package_file = ROOT / ready['file']
                            if hashlib.sha256(package_file.read_bytes()).hexdigest() != ready['sha256']:
                                raise RuntimeError('Package changed after registration')
                            with transaction(args.ledger) as state:
                                state['tests'][ready['test_id']]['state'] = 'sending'
                            records = delta(conf, 'send-webxdc', conf['chat_id'], package_file, '--chat', '--app', app,
                                            '--caption', ready['purpose'], timeout=180)
                        else:
                            # Never blindly retry an uncertain send: inspect the saved app binding.
                            records = delta(conf, 'app-list')
                        identity = next((r for r in records if r.get('message_id') and (r.get('app', r.get('name', app)) == app)), None)
                        if not identity:
                            identity_records = delta(conf, 'app-info', app)
                            identity = next((r for r in identity_records if r.get('message_id')), None)
                        if not identity:
                            raise RuntimeError('Send outcome uncertain; inspect app registry before retrying')
                        hooks = delta(conf, 'app-hook-list', app)
                        if not any(h.get('id') == 'review-feedback' for r in hooks for h in r.get('hooks', [])):
                            delta(conf, 'app-hook-add', app, 'review-feedback', HOOK, '--type', 'update',
                                  '--match', '{"protocol":"bhoo-test-review/v1"}')
                        with transaction(args.ledger) as state:
                            state['tests'][ready['test_id']].update(state='awaiting_review', message_id=identity['message_id'], sent_at=now())
                        start_app(app)
                        print('Sent review and started worker:', ready['test_id'], flush=True)
                    else:
                        # One worker owns the account. Rotate through all reviews;
                        # workers replay durable updates and unfinished hooks on start.
                        monitored = [t for t in tests if t['state'] == 'awaiting_review' or
                                     (t['state'] == 'reviewed' and t.get('deletion') == 'pending')]
                        if monitored:
                            test = monitored[scan_index % len(monitored)]
                            scan_index += 1
                            start_app(test['app'])
                        elif idle is None or idle.poll() is not None:
                            stop_current()
                            idle = subprocess.Popen([conf['binary'], 'listen', '--data-path', conf['data_path'], '--json'],
                                                    stdout=subprocess.DEVNULL, stderr=sys.stderr)
                    if args.once:
                        break
                except Exception as error:
                    print('Review supervisor:', error, file=sys.stderr, flush=True)
                    if args.once:
                        raise
                for _ in range(3):
                    if stopping: break
                    time.sleep(1)
        finally:
            stop_idle()
            stop_current()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ledger', type=Path, default=DEFAULT_LEDGER, help='Path to the ledger file (default: DEFAULT_LEDGER)')
    sub=parser.add_subparsers(dest='command', required=True)
    pack=sub.add_parser('package')
    pack.add_argument('--source', type=Path, required=True)
    pack.add_argument('--spec', type=Path, required=True)
    pack.add_argument('--test-id', required=True)
    pack.add_argument('--output', type=Path, required=True)
    pack.add_argument('--repairs', nargs='*', default=[])
    sub.add_parser('status')
    serve=sub.add_parser('serve')
    serve.add_argument('--once', action='store_true')
    args=parser.parse_args()
    {'package': package, 'status': status, 'serve': supervise}[args.command](args)


if __name__=='__main__': main()
