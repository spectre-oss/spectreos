#!/usr/bin/env python3
"""Local, peer-authenticated hardware service. No arbitrary command execution."""
import argparse
import queue
import json
import os
import re
import pwd
import socket
import signal
import socketserver
import struct
import subprocess
import threading
import time
from pathlib import Path

SOCKET = '/run/spectre-fingerprint/control.sock'
LOCK = threading.Lock()


def result_for(returncode, output, duration):
    geometry = re.search(r'Sensor geometry=(\d+)x(\d+); OTP bytes=(\d+); OTP data=(\w+)', output)
    ok = (returncode == 0 and geometry is not None and geometry[4] == 'present'
          and int(geometry[1]) > 0 and int(geometry[2]) > 0 and int(geometry[3]) == 32)
    result = dict(ok=ok, duration=round(duration, 2), kernel=os.uname().release,
                  details=output[-16000:], enrollment_supported=True)
    if ok:
        result.update(rows=int(geometry[1]), columns=int(geometry[2]),
                      title='Sensor is responding', message='Detection and initialization passed.')
    elif 'No such file' in output or 'TEE open' in output:
        result.update(title='Test kernel needed', message='Boot the fingerprint test kernel, then try again.')
    elif '1062' in output:
        result.update(title='Sensor unavailable', message='The fingerprint sensor did not respond. Try again.')
    else:
        result.update(title='Sensor check failed', message='See test details for the hardware response.')
    return result


def operation_result(action, returncode, output, duration):
    result = result_for(returncode, output, duration)
    if action == 'test':
        return result
    result['ok'] = False
    result['enrolled_count'] = None
    count = re.search(r'Group 1000 template count=(\d+)', output)
    saved = re.search(r'SAVE_VERIFIED count=(\d+) saved_template_listed=yes', output)
    if action == 'enroll' and returncode == 0 and saved:
        result.update(ok=True, enrolled_count=int(saved[1]), title='Fingerprint saved',
                      message='You can now check this fingerprint with the matching button.')
    elif action == 'verify' and returncode == 0 and 'MATCH_RESULT accepted=yes identity=expected' in output:
        result.update(ok=True, matched=True, title='Fingerprint matched', message='The enrolled fingerprint was recognized.')
    elif action == 'verify' and returncode == 0 and 'MATCH_RESULT accepted=no' in output:
        result.update(ok=True, matched=False, title='Fingerprint not recognized', message='This finger did not match the saved fingerprint.')
    elif 'isolated group already has a template' in output:
        result.update(title='Fingerprint already saved', message='This version supports one fingerprint. Your existing enrollment was kept.')
    elif action == 'enroll' and 'Vendor status=1057' in output:
        result.update(title='Enrollment authorization needed',
                      message='The sensor requires a trusted authorization token. No fingerprint was enrolled.')
    elif returncode == 0 and count and action == 'list':
        n = int(count[1])
        result.update(ok=True, enrolled_count=n, group=1000, title=f'{n} saved fingerprints',
                      message='Saved fingerprints for Fingerprint Lab.')
    elif action == 'verify' and count and int(count[1]) == 0:
        result.update(title='No enrolled fingerprints', message='Enroll a fingerprint first.')
    elif action == 'verify':
        result.update(title='No matching result', message='No complete match result was received. Try again.')
    elif action == 'enroll':
        result.update(title='Enrollment is not complete', message='No fingerprint was confirmed saved.')
    return result


def progress_for(line):
    if line.startswith('CAPTURE_READY:'):
        return dict(title='Touch and lift your finger', message='Use one finger and vary its angle slightly until enrollment finishes.')
    if line.startswith('MATCH_READY:'):
        return dict(title='Touch the sensor', message='Place your finger on the rear sensor and hold it naturally.')
    match = re.search(r'ENROLL_PROGRESS remaining=(\d+)', line)
    if match:
        return dict(title='Capturing fingerprint', message=f'{match[1]} samples remaining. Lift and touch again.')
    return None


def run_test(helper, action='test', progress=None):
    start = time.monotonic()
    try:
        # Helper is fixed by the root-owned service unit, never supplied by a client.
        options = {'test': [], 'list': ['--list'], 'enroll': ['--save-probe'], 'verify': ['--match-probe']}
        with subprocess.Popen([helper, *options[action]], stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                              text=True, start_new_session=True) as p:
            try:
                lines = queue.Queue()
                def read_output():
                    try:
                        for line in p.stdout:
                            lines.put(line)
                    finally:
                        lines.put(None)
                threading.Thread(target=read_output, daemon=True).start()
                output = ''
                deadline = time.monotonic() + 125
                while True:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise subprocess.TimeoutExpired(helper, 125)
                    try:
                        line = lines.get(timeout=remaining)
                    except queue.Empty:
                        raise subprocess.TimeoutExpired(helper, 125)
                    if line is None:
                        break
                    output = (output + line)[-16000:]
                    event = progress_for(line)
                    if event and progress:
                        progress(event)
                p.wait(timeout=max(.1, deadline - time.monotonic()))
            except subprocess.TimeoutExpired:
                # Signal the helper and child together so the shell restores the firmware path.
                os.killpg(p.pid, signal.SIGTERM)
                try:
                    p.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(p.pid, signal.SIGKILL)
                    p.wait()
                return dict(ok=False, title='Sensor check timed out',
                            message='The hardware did not finish its request.', details=output[-16000:])
        return operation_result(action, p.returncode, output, time.monotonic() - start)
    except OSError as e:
        return dict(ok=False, title='Hardware service error', message=str(e), details=str(e))


class Handler(socketserver.StreamRequestHandler):
    def handle(self):
        self.connection.settimeout(5)
        _, uid, _ = struct.unpack('3i', self.connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12))
        if uid not in (0, self.server.allowed_uid):
            self.wfile.write(b'{"ok":false,"title":"Access denied"}\n')
            return
        action = self.rfile.readline(65).decode('ascii', errors='replace').strip()
        if action == 'status':
            result = dict(ok=True, ready=Path('/dev/teepriv0').exists(), kernel=os.uname().release,
                          enrollment_supported=True)
        elif action not in ('test', 'enroll', 'list', 'verify'):
            result = dict(ok=False, title='Unknown action', message='Unsupported fingerprint operation.')
        elif not LOCK.acquire(blocking=False):
            result = dict(ok=False, title='Test already running', message='Wait for the current check to finish.')
        else:
            try:
                def progress(event):
                    try:
                        self.wfile.write((json.dumps(dict(event='progress', **event)) + '\n').encode())
                        self.wfile.flush()
                    except OSError:
                        pass  # Complete/cancel the bounded hardware operation even if UI closes.
                result = run_test(self.server.helper, action, progress)
            finally:
                LOCK.release()
        self.wfile.write((json.dumps(result) + '\n').encode())


class Server(socketserver.ThreadingUnixStreamServer):
    daemon_threads = True


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--client', choices=['status', 'test', 'enroll', 'list', 'verify'])
    p.add_argument('--helper')
    p.add_argument('--uid', type=int, default=1000)
    p.add_argument('--user', help='Resolve the allowed local account at service startup')
    args = p.parse_args()
    if args.client:
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                s.settimeout(140)
                s.connect(SOCKET)
                s.sendall((args.client + '\n').encode())
                with s.makefile('rb') as f:
                    while True:
                        data = f.readline(131073)
                        if len(data) > 131072:
                            raise ValueError('Service response too large')
                        result = json.loads(data)
                        print(json.dumps(result), flush=True)
                        if result.get('event') != 'progress':
                            break
        except (OSError, ValueError) as e:
            print(json.dumps(dict(ok=False, title='Hardware service unavailable',
                                  message='The local fingerprint service is not ready.', details=str(e))))
        return
    if os.geteuid() != 0 or not args.helper:
        p.error('The service requires root and a fixed --helper path')
    allowed_uid = pwd.getpwnam(args.user).pw_uid if args.user else args.uid
    Path(SOCKET).parent.mkdir(mode=0o755, parents=True, exist_ok=True)
    Path(SOCKET).unlink(missing_ok=True)
    with Server(SOCKET, Handler) as s:
        s.allowed_uid, s.helper = allowed_uid, args.helper
        os.chown(SOCKET, allowed_uid, -1)
        os.chmod(SOCKET, 0o600)
        s.serve_forever()


if __name__ == '__main__':
    main()
