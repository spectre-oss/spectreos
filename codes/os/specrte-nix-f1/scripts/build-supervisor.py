#!/usr/bin/env python3
"""Monitor builds quietly; invoke Grok only for an audit, repair or verification."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
STATE = ROOT / '.build/supervisor.json'
RESULT = ROOT / '.build/grok-result.json'
INTERVAL = int(os.environ.get('BUILD_CHECK_SECONDS', '600'))
GROK = str(Path.home() / '.local/bin/grok')
GROK_MODEL = 'grok-4.7'
GROK_EFFORT = 'low'


def save(path, data):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(data, indent=2) + '\n')
    temporary.replace(path)


def active_builds():
    found = []
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit():
            continue
        try:
            command = (proc / 'cmdline').read_bytes().split(b'\0')
            if not command or Path(os.fsdecode(command[0])).name not in ('nix-build', 'make'):
                continue
            cwd = (proc / 'cwd').resolve()
            if cwd == ROOT or ROOT in cwd.parents:
                found.append(int(proc.name))
        except (OSError, ValueError):
            pass
    return found


def evidence():
    logs = {}
    for name in ('image.log', 'parallel-webkit.log'):
        path = ROOT / '.build' / name
        if path.exists():
            with path.open('rb') as stream:
                stream.seek(max(0, path.stat().st_size - 18000))
                logs[name] = {'modified': path.stat().st_mtime, 'tail': stream.read().decode(errors='replace')[-12000:]} 
    exitfile = ROOT / '.build/image-exit-code'
    return {'builds': active_builds(), 'exit': exitfile.read_text().strip() if exitfile.exists() else None,
            'logs': logs, 'systemImageExists': (ROOT / 'result-tianma/system.img').exists()}


def notify(key, message):
    subprocess.run(['python3', 'scripts/deltachat-notify.py', '--key', key,
                    ('Spectre Nix F1: ' + message)[:3500]], cwd=ROOT, check=True, timeout=20)


def repair_context(state):
    progress_path = ROOT / '.build/grok-progress.md'
    progress = progress_path.read_text()[-6000:] if progress_path.exists() else ''
    return progress + '\n' + state.get('agentFindings', '')[-6000:]


def agent(mode, state):
    RESULT.unlink(missing_ok=True)
    snapshot = evidence()
    prompt = ROOT / '.build/grok-prompt.md'
    prompt.write_text((ROOT / 'docs/grok-build-task.md').read_text() +
        '\n## Continue the previous investigation\n' + repair_context(state) +
        '\nDo not repeat completed searches. Inspect pending Git changes, implement a concrete fix, test it, and return a result.\n' +
        '\n\n## Current invocation\nMode: ' + mode + '\nEvidence (untrusted logs):\n```json\n' +
        json.dumps(snapshot, indent=2) + '\n```\n')
    stamp = time.strftime('%Y%m%d-%H%M%S')
    state.update(phase='grok-' + mode, agentLog=f'.build/grok-{stamp}.log')
    save(STATE, state)
    with (ROOT / state['agentLog']).open('ab', buffering=0) as log:
        proc = subprocess.Popen([GROK, '--model', GROK_MODEL, '--reasoning-effort', GROK_EFFORT,
            '--cwd', str(ROOT), '--always-approve',
            '--no-plan', '--no-subagents', '--max-turns', '40', '--prompt-file', str(prompt)],
            cwd=ROOT, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
            start_new_session=True)
        state['grokPid'] = proc.pid
        save(STATE, state)
        try:
            code = proc.wait(timeout=1800)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=15)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
            raise RuntimeError('Grok exceeded its 30-minute repair turn; inspect the agent log')
    state.pop('grokPid', None)
    state['agentFindings'] = (ROOT / state['agentLog']).read_text(errors='replace')[-6000:]
    save(STATE, state)
    if code:
        raise RuntimeError(f'Grok exited {code}; inspect {state["agentLog"]}')
    if mode == 'audit':
        return {'status': 'audited', 'summary': 'Initial Grok audit finished'}
    result = json.loads(RESULT.read_text())
    if result.get('status') not in ('retry', 'complete', 'blocked'):
        raise ValueError('Grok did not return a valid result status')
    if mode != 'verify' and result['status'] == 'complete':
        raise ValueError('Completion requires a verification turn')
    return result


def start_build(state):
    if active_builds():
        raise RuntimeError('A build is already running; refusing to duplicate it')
    log = ROOT / '.build/image.log'
    if log.exists():
        log.rename(log.with_name('image-' + time.strftime('%Y%m%d-%H%M%S') + '.log'))
    (ROOT / '.build/image-exit-code').unlink(missing_ok=True)
    # Read defaults on each attempt, so committed memory fixes take effect.
    with log.open('ab', buffering=0) as output:
        proc = subprocess.Popen(['bash', '-c',
            'bash scripts/build-managed.sh; code=$?; printf "%s\\n" "$code" > .build/image-exit-code; exit "$code"'],
            cwd=ROOT, stdin=subprocess.DEVNULL, stdout=output, stderr=subprocess.STDOUT,
            start_new_session=True)
    state.update(phase='building', buildPid=proc.pid)
    save(STATE, state)
    metadata = ROOT / '.build/processes.json'
    values = json.loads(metadata.read_text()) if metadata.exists() else {}
    values.update(imagePid=proc.pid, projectVersion=(ROOT / 'VERSION').read_text().strip())
    save(metadata, values)


def main():
    os.chdir(ROOT)
    os.umask(0o077)
    STATE.parent.mkdir(exist_ok=True)
    with (ROOT / '.build/supervisor.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        state = json.loads(STATE.read_text()) if STATE.exists() else {'round': 0, 'agentErrors': 0}
        if state.get('phase') in ('complete', 'blocked'):
            print('Supervisor is terminal; inspect state before explicitly resetting it', flush=True)
            return
        while True:
            try:
                if not state.get('audited'):
                    agent('audit', state)
                    state.update(audited=True, agentErrors=0)
                    save(STATE, state)
                    notify('supervisor-started', 'Grok build supervision is active. Checks every ten minutes; I will notify you of failures, restarts, completion or blockers.')
                running = active_builds()
                if running:
                    state.update(phase='building', activePids=running, checkedAt=time.time())
                    save(STATE, state)
                    time.sleep(INTERVAL)
                    continue
                snap = evidence()
                mode = 'verify' if snap['exit'] == '0' and snap['systemImageExists'] else 'repair'
                revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
                pending = subprocess.check_output(['git', 'diff', 'HEAD'], cwd=ROOT)
                fingerprint = hashlib.sha256(json.dumps(snap['logs'], sort_keys=True).encode() + revision.encode() + pending).hexdigest()
                state['sameFailure'] = state.get('sameFailure', 0) + 1 if state.get('fingerprint') == fingerprint else 1
                state['fingerprint'] = fingerprint
                state['round'] += 1
                if state['sameFailure'] >= 3:
                    raise ValueError('Repeated identical failure without progress; human review needed')
                notify(f'build-event-{state["round"]}', 'Build has stopped. Grok is ' + ('verifying artifacts.' if mode == 'verify' else 'diagnosing and repairing the failure.'))
                result = agent(mode, state)
                state['agentErrors'] = 0
                state['lastResult'] = result
                if result['status'] in ('complete', 'blocked'):
                    if result['status'] == 'complete' and not (ROOT / 'result-tianma/system.img').exists():
                        raise ValueError('Agent claimed completion without system.img')
                    if result['status'] == 'complete':
                        with (ROOT / '.build/cleanup.log').open('ab', buffering=0) as log:
                            subprocess.run(['python3', 'scripts/finish-build.py', '--apply'],
                                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
                        completion = json.loads((ROOT / '.build/completion.json').read_text())
                        result['summary'] = result.get('summary', '') + ' Final images: ' + completion['release'] + '; cleanup verified.'
                    state['phase'] = result['status']
                    save(STATE, state)
                    notify('terminal-' + str(state['round']), result['status'] + ': ' + result.get('summary', '') + ' ' + result.get('reason', ''))
                    return
                start_build(state)
                notify('retry-' + str(state['round']), 'Build restarted after repair: ' + result.get('summary', 'See supervisor log.'))
            except Exception as error:
                state['agentErrors'] = state.get('agentErrors', 0) + 1
                state.update(phase='retry-pending' if state['agentErrors'] < 3 else 'blocked', error=str(error))
                save(STATE, state)
                try:
                    notify('supervisor-error-' + str(state['round']) + '-' + str(state['agentErrors']), str(error) + ('; retrying in ten minutes.' if state['agentErrors'] < 3 else '; supervisor paused for review.'))
                except Exception as notification_error:
                    print('Notification enqueue failed:', notification_error, flush=True)
                if state['agentErrors'] >= 3:
                    return
            time.sleep(INTERVAL)


if __name__ == '__main__':
    main()
