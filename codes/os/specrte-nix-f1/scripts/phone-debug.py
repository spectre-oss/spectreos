#!/usr/bin/env python3
"""Control the phone over SSH. JSON actions in, JSON results / PNG files out."""
import argparse
import base64
import json
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', default='spectre@172.16.42.1')
    parser.add_argument('--control-path', help='Optional existing SSH ControlMaster socket')
    parser.add_argument('--remote-command', default='if command -v spectre-phone-debug-agent >/dev/null 2>&1; then exec spectre-phone-debug-agent; else exec ~/.local/bin/spectre-phone-debug-agent; fi')
    parser.add_argument('--output', type=Path, help='Save screenshot PNG here')
    parser.add_argument('request', help='JSON action, or @file containing a list of actions')
    args = parser.parse_args()
    raw = Path(args.request[1:]).read_text() if args.request.startswith('@') else args.request
    requests = json.loads(raw)
    if not isinstance(requests, list):
        requests = [requests]
    if not requests or len(requests) > 100:
        parser.error('Supply 1..100 actions')
    if args.output and sum(r.get('action') == 'screenshot' for r in requests) != 1:
        parser.error('--output requires exactly one screenshot action')
    ssh = ['ssh', '-T', '-o', 'ConnectTimeout=5']
    if args.control_path:
        ssh += ['-S', args.control_path]
    ssh += ['--', args.host, args.remote_command]
    process = subprocess.run(ssh, input=''.join(json.dumps(r)+'\n' for r in requests),
                             text=True, stdout=subprocess.PIPE,
                             timeout=60 + 45 * len(requests))
    if process.returncode:
        raise RuntimeError(f'SSH command failed ({process.returncode})')
    replies = process.stdout.splitlines()
    if len(replies) != len(requests):
        raise RuntimeError('Incomplete response from phone')
    failed = False
    for line in replies:
        response = json.loads(line)
        failed |= not response.get('ok', False)
        result = response.get('result', {})
        if 'png_base64' in result and args.output:
            data = base64.b64decode(result.pop('png_base64'), validate=True)
            if not data.startswith(b'\x89PNG\r\n\x1a\n'):
                raise RuntimeError('Invalid PNG response')
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_bytes(data)
            result['file'] = str(args.output.resolve())
        print(json.dumps(response))
    return 1 if failed else 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        sys.exit(str(error))
