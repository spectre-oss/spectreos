#!/usr/bin/env python3
"""File-based, versioned boundary for the Poco F1's isolated ARM32 HDR worker."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def sha256(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def verify(bundle):
    manifest = json.loads((bundle / 'manifest.json').read_text())
    if (manifest.get('schema') != 1 or manifest.get('camera') != 'beryllium-imx363-sunny'
            or manifest.get('backend') != 'arcsoft-hdr'):
        raise ValueError('Wrong camera bundle or manifest version')
    required = {'linker', 'xiaomi-camera-helper', 'client.py', 'run', 'lib/libc.so',
                'lib/libm.so', 'lib/libdl.so', 'lib/liblog.so', 'lib/libc++.so',
                'lib/libmpbase.so', 'lib/libarcsoft_high_dynamic_range.so'}
    if set(manifest['files']) != required:
        raise ValueError('Unexpected or incomplete bundle file list')
    for name, expected in manifest['files'].items():
        path = bundle / name
        if path.is_symlink() or not path.is_file() or sha256(path) != expected:
            raise ValueError(f'Bundle integrity failure: {name}')


def validate_job(job, directory):
    if job.get('schema') != 1 or job.get('camera') != 'beryllium-imx363-sunny':
        raise ValueError('Expected job schema 1 for beryllium-imx363-sunny')
    if job.get('format') not in ('nv12', 'nv21'):
        raise ValueError('Only tightly packed 8-bit NV12/NV21 is supported')
    w, h = job.get('width'), job.get('height')
    if (type(w) is not int or type(h) is not int or not 64 <= w <= 4032
            or not 64 <= h <= 4032 or w % 2 or h % 2 or w * h > 4032 * 3024):
        raise ValueError('Invalid dimensions or more than 12.2 megapixels')
    frames = job.get('frames')
    if not isinstance(frames, list) or len(frames) != 3 or not all(isinstance(p, str) for p in frames):
        raise ValueError('HDR requires exactly three input files')
    paths = [(directory / p).resolve() for p in frames]
    size = w * h * 3 // 2
    for path in paths:
        if not path.is_file() or path.stat().st_size != size:
            raise ValueError(f'Input must contain exactly {size} bytes: {path}')
    ae = job.get('ae_info')
    if (not isinstance(ae, list) or len(ae) != 6
            or any(type(x) not in (int, float) or not math.isfinite(x) or not 0 <= x <= 100000 for x in ae)):
        raise ValueError('ae_info must explicitly supply the six finite nonnegative ABI values')
    if not isinstance(job.get('output'), str) or not job['output']:
        raise ValueError('Missing output path')
    output = (directory / job['output']).resolve()
    if output.exists() or output in paths or not output.parent.is_dir():
        raise ValueError('Output must be new, with an existing parent directory')
    return paths, output


def invoke(bundle, args, timeout, emulator=None):
    if not 0 < timeout <= 90:
        raise ValueError('Timeout must be greater than zero and at most 90 seconds')
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(('LD_', 'DYLD_'))}
    loader = [str(bundle / 'linker'), str(bundle / 'xiaomi-camera-helper')]
    if emulator:
        loader = [emulator, '-E', f'LD_LIBRARY_PATH={bundle / "lib"}', *loader]
    else:
        loader = ['env', '-i', f'LD_LIBRARY_PATH={bundle / "lib"}', *loader]
    # Old ARM32 Bionic mutex owner fields cannot represent PID/TID > 65535.
    # --kill-child also prevents a timed-out supervisor leaving a worker behind.
    loader = ['unshare', '--user', '--map-root-user', '--pid', '--fork', '--kill-child', *loader]
    completed = subprocess.run(loader + args, env=env, capture_output=True,
                               text=True, timeout=timeout, check=False)
    try:
        report = json.loads(completed.stdout)
    except json.JSONDecodeError:
        raise RuntimeError(f'Worker returned {completed.returncode}: {completed.stderr[-4000:]}') from None
    if completed.returncode != 0 or report.get('ok') is not True:
        raise RuntimeError(f'Worker failed: {report}; {completed.stderr[-4000:]}')
    return report


def process_job(bundle, job, directory, timeout=60, emulator=None):
    paths, output = validate_job(job, directory)
    verify(bundle)
    with tempfile.TemporaryDirectory(prefix='.xiaomi-hdr-', dir=output.parent) as work:
        work = Path(work)
        inputs = []
        # Copy-on-boundary: the vendor code never receives the user's originals.
        for i, path in enumerate(paths):
            dest = work / f'input-{i}.yuv'
            shutil.copyfile(path, dest)
            inputs.append(str(dest))
        result = work / 'result.yuv'
        report = invoke(bundle, ['hdr', str(bundle / 'lib/libarcsoft_high_dynamic_range.so'),
                                job['format'], str(job['width']), str(job['height']),
                                *inputs, str(result), *[str(x) for x in job['ae_info']]],
                        timeout, emulator)
        if result.stat().st_size != job['width'] * job['height'] * 3 // 2:
            raise RuntimeError('Worker output has wrong size')
        # Atomic publication, refuses an output created concurrently by another job.
        os.link(result, output)
    return {**report, 'output': str(output), 'sha256': sha256(output)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['probe', 'hdr'])
    parser.add_argument('--bundle', type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument('--job', type=Path)
    parser.add_argument('--timeout', type=float, default=60)
    parser.add_argument('--emulator', help='Server tests only: path to qemu-arm')
    args = parser.parse_args()
    try:
        bundle = args.bundle.resolve()
        if args.operation == 'probe':
            verify(bundle)
            report = invoke(bundle, ['probe', str(bundle / 'lib/libarcsoft_high_dynamic_range.so')],
                            args.timeout, args.emulator)
        else:
            if not args.job:
                raise ValueError('--job is required for hdr')
            report = process_job(bundle, json.loads(args.job.read_text()),
                                 args.job.resolve().parent, args.timeout, args.emulator)
        print(json.dumps(report, indent=2))
    except (ValueError, OSError, RuntimeError, KeyError, TypeError, subprocess.TimeoutExpired) as error:
        parser.exit(1, json.dumps({'ok': False, 'error': str(error)}) + '\n')


if __name__ == '__main__':
    main()
