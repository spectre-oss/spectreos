#!/usr/bin/env python3
"""Run on build host: boundary tests, or original ARM blob integration via qemu-arm.

--fixtures prepares artificial exposure brackets, not a camera-quality test.
"""
import argparse
import copy
import json
from pathlib import Path
import tempfile
import unittest
import shutil
import subprocess
import client


def fixtures(directory, width=640, height=480, fmt='nv12'):
    directory.mkdir(parents=True, exist_ok=True)
    for i, scale in enumerate((1, .25, 2)):
        row = bytes(min(235, 16 + round(219 * (x / (width - 1)) ** 2 * scale)) for x in range(width))
        # Fixed colourful tiles also catch swapped chroma and interleaving.
        uv = bytes(c for x in range(width // 2)
                   for c in ((96, 160) if (x // 32) % 2 else (160, 96)))
        if fmt == 'nv21':
            uv = bytes(c for x in range(0, len(uv), 2) for c in (uv[x + 1], uv[x]))
        (directory / f'frame-{i}.yuv').write_bytes(row * height + uv * (height // 2))
    job = {'schema': 1, 'camera': 'beryllium-imx363-sunny', 'format': fmt,
           'width': width, 'height': height,
           'frames': [f'frame-{i}.yuv' for i in range(3)],
           'ae_info': [50, 1, 1, 1, 8, 4], 'output': 'output.yuv'}
    (directory / 'job.json').write_text(json.dumps(job, indent=2) + '\n')
    return job


class BoundaryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.job = fixtures(self.root, 64, 64)

    def tearDown(self):
        self.temp.cleanup()

    def test_valid(self):
        inputs, output = client.validate_job(self.job, self.root)
        self.assertEqual(len(inputs), 3)
        self.assertEqual(output.name, 'output.yuv')

    def test_invalid_contract(self):
        for field, value in [('schema', 2), ('camera', 'other'), ('format', 'raw10'),
                             ('width', 65), ('height', True), ('width', 100000),
                             ('frames', ['frame-0.yuv']), ('ae_info', [1] * 5),
                             ('ae_info', [float('nan')] * 6), ('ae_info', [True] * 6)]:
            with self.subTest(field=field, value=value):
                job = copy.deepcopy(self.job)
                job[field] = value
                with self.assertRaises(ValueError):
                    client.validate_job(job, self.root)

    def test_truncated_input(self):
        (self.root / 'frame-0.yuv').write_bytes(b'bad')
        with self.assertRaises(ValueError):
            client.validate_job(self.job, self.root)

    def test_preserve_existing_output(self):
        out = self.root / 'output.yuv'
        out.write_bytes(b'keep')
        with self.assertRaises(ValueError):
            client.validate_job(self.job, self.root)
        self.assertEqual(out.read_bytes(), b'keep')


def integration(bundle, emulator):
    client.verify(bundle)
    report = {'probe': client.invoke(bundle, ['probe', str(bundle / 'lib/libarcsoft_high_dynamic_range.so')], 60, emulator),
              'cases': [], 'rejections': []}
    for fmt, width in [('nv12', 640), ('nv21', 640), ('nv12', 642)]:
        with tempfile.TemporaryDirectory() as td:
            directory = Path(td)
            job = fixtures(directory, width, 480, fmt)
            before = [client.sha256(directory / f) for f in job['frames']]
            result = client.process_job(bundle, job, directory, emulator=emulator)
            assert before == [client.sha256(directory / f) for f in job['frames']]
            assert result['sha256'] not in before, 'No observable processing of artificial bracket'
            report['cases'].append({'format': fmt, **result})
    with tempfile.TemporaryDirectory() as td:
        directory = Path(td)
        job = fixtures(directory)
        args = ['hdr', str(bundle / 'lib/libarcsoft_high_dynamic_range.so'), 'raw10', '640', '480',
                *[str(directory / f) for f in job['frames']], str(directory / 'output.yuv'),
                *[str(v) for v in job['ae_info']]]
        try:
            client.invoke(bundle, args, 60, emulator)
        except RuntimeError as error:
            assert 'pixel-format' in str(error)
        else:
            raise AssertionError('Native helper accepted RAW10 as YUV')
        assert not (directory / 'output.yuv').exists()
        report['rejections'].append('native pixel format')
        copy_bundle = directory / 'bundle'
        shutil.copytree(bundle, copy_bundle)
        with (copy_bundle / 'lib/libarcsoft_high_dynamic_range.so').open('ab') as f:
            f.write(b'corrupt')
        try:
            client.verify(copy_bundle)
        except ValueError as error:
            assert 'integrity' in str(error)
        else:
            raise AssertionError('Modified blob accepted')
        report['rejections'].append('modified original library')
    if emulator:
        with tempfile.TemporaryDirectory() as td:
            directory = Path(td)
            job = fixtures(directory, 4032, 3024)
            try:
                client.process_job(bundle, job, directory, timeout=0.01, emulator=emulator)
            except subprocess.TimeoutExpired:
                assert not (directory / job['output']).exists()
                assert not list(directory.glob('.xiaomi-hdr-*'))
                report['rejections'].append('timed-out job leaves no output or staging directory')
            else:
                raise AssertionError('Timeout fixture unexpectedly finished in 10 ms; adjust this test')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixtures', type=Path)
    parser.add_argument('--width', type=int, default=640)
    parser.add_argument('--height', type=int, default=480)
    parser.add_argument('--bundle', type=Path)
    parser.add_argument('--emulator')
    args = parser.parse_args()
    if args.fixtures:
        fixtures(args.fixtures, args.width, args.height)
    elif args.bundle:
        print(json.dumps(integration(args.bundle.resolve(), args.emulator), indent=2))
    else:
        unittest.main(argv=[__file__])
