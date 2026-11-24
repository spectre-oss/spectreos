import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from test_flash import MOCK

ROOT = Path(__file__).resolve().parents[1]

class LaptopInstaller(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copy(ROOT / 'scripts/release-install.py', self.root / 'install.py')
        (self.root / 'boot.img').write_bytes(b'ANDROID!' + bytes(64))
        data = bytearray(4096)
        data[1080:1082] = b'\x53\xef'
        data[1144:1156] = b'NIXOS_SYSTEM'
        (self.root / 'system.img').write_bytes(data)
        files = {}
        for name in ['boot.img', 'system.img']:
            data = (self.root / name).read_bytes()
            files[name] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
        (self.root / 'manifest.json').write_text(json.dumps(
            {'kind': 'system', 'projectVersion': '0.2.0', 'panel': 'tianma', 'files': files}))
        mock = MOCK.replace(" 'flash userdata '*)", " 'getvar partition-size:userdata') echo 'partition-size:userdata: 0x10000000' >&2;;\n 'flash userdata '*)")
        (self.root / 'fastboot').write_text(mock)
        (self.root / 'fastboot').chmod(0o755)

    def run_install(self, check=False, answer='ERASE beryllium', **extra):
        log = self.root / 'calls'
        log.write_text('')
        env = os.environ | {'PATH': str(self.root) + ':' + os.environ['PATH'], 'CALL_LOG': str(log)} | extra
        args = ['python3', str(self.root / 'install.py')] + (['--check'] if check else [])
        result = subprocess.run(args, input='\n' + answer + '\n', text=True, capture_output=True, env=env)
        return result, log.read_text()

    def test_transport_timeout_is_reported_without_retry(self):
        import importlib.util
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location('release_installer', ROOT / 'scripts/release-install.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with patch.object(module.shutil, 'which', return_value='/fake/fastboot'):
            phone = module.Fastboot()
        with patch.object(module.subprocess, 'run', side_effect=subprocess.TimeoutExpired('fastboot', 15)) as run:
            with self.assertRaisesRegex(RuntimeError, 'timed out'):
                phone.run('-s', 'serial', 'getvar', 'product')
            self.assertEqual(run.call_count, 1)
            self.assertEqual(run.call_args.kwargs['timeout'], 15)

    def test_readonly_and_success(self):
        result, calls = self.run_install(check=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('flash ', calls)
        result, calls = self.run_install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertLess(calls.index('flash userdata'), calls.index('flash boot'))
        self.assertLess(calls.index('flash boot'), calls.index('reboot'))

    def test_refusal_and_failed_transfer(self):
        for settings in [{'PRODUCT': 'other'}, {'UNLOCKED': 'false'}, {'NO_DEVICE': '1'},
                         {'BOOT_SIZE': '1'}, {'answer': 'no'}]:
            result, calls = self.run_install(**settings)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn('flash ', calls)
        result, calls = self.run_install(FAIL_ROOTFS='1')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('flash boot', calls)
        self.assertNotIn('reboot', calls)

    def test_boot_update_preserves_userdata(self):
        p = self.root / 'manifest.json'
        data = json.loads(p.read_text())
        data['kind'] = 'boot'
        del data['files']['system.img']
        p.write_text(json.dumps(data))
        (self.root / 'system.img').unlink()
        result, calls = self.run_install(check=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('flash ', calls)
        result, calls = self.run_install(answer='ERASE beryllium')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('flash ', calls)
        result, calls = self.run_install(answer='UPDATE beryllium')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('flash boot', calls)
        self.assertNotIn('userdata', calls)
        self.assertNotIn('erase ', calls)
        self.assertLess(calls.index('flash boot'), calls.index('reboot'))
        for settings in [{'PRODUCT': 'other'}, {'UNLOCKED': 'false'}, {'BOOT_SIZE': '1'}]:
            result, calls = self.run_install(answer='UPDATE beryllium', **settings)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn('flash ', calls)
        mock = self.root / 'fastboot'
        mock.write_text(mock.read_text().replace(' *) exit 0;;', " 'flash boot '*) exit 1;;\n *) exit 0;;"))
        result, calls = self.run_install(answer='UPDATE beryllium')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('reboot', calls)
        self.assertNotIn('userdata', calls)

    def test_corrupt_and_partial_release(self):
        (self.root / 'system.img').write_bytes(b'corrupt')
        result, calls = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, '')
        p = self.root / 'manifest.json'
        data = json.loads(p.read_text())
        data['kind'] = 'kernel'
        p.write_text(json.dumps(data))
        result, calls = self.run_install()
        self.assertIn('partial', result.stderr)
        self.assertEqual(calls, '')

if __name__ == '__main__':
    unittest.main()
