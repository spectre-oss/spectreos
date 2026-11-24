#!/usr/bin/env python3
"""Exercise publication with tiny fixtures rather than copying real images."""
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class Publication(unittest.TestCase):
    def test_complete_partial_and_failed_publications(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'project'
            (root / 'scripts').mkdir(parents=True)
            for name in ['dist.py', 'version.py', 'release-install.py', 'INSTALL.md']:
                shutil.copy(ROOT / 'scripts' / name, root / 'scripts' / name)
            (root / 'docs').mkdir()
            shutil.copy(ROOT / 'docs/usb-ssh.md', root / 'docs/usb-ssh.md')
            kernel = root / 'result-kernel'
            (kernel / 'dtbs/qcom').mkdir(parents=True)
            (kernel / 'vmlinuz-test').write_bytes(b'kernel fixture')
            (kernel / 'build.config').write_text('test config')
            (kernel / 'dtbs/qcom/sdm845-xiaomi-beryllium-tianma.dtb').write_bytes(b'dtb')
            system = root / 'result-tianma'
            system.mkdir()
            (system / 'boot.img').write_bytes(b'boot fixture')
            (system / 'system.img').write_bytes(b'root fixture')
            command = ['python3', str(root / 'scripts/dist.py'), 'system',
                       '--version', '0.2.0', '--revision', 'a' * 40]
            first = Path(subprocess.check_output(command).decode().strip())
            second = Path(subprocess.check_output(command).decode().strip())
            self.assertNotEqual(first, second)
            manifest = json.loads((first / 'manifest.json').read_text())
            self.assertEqual(manifest['projectVersion'], '0.2.0')
            self.assertEqual((first / 'system.img').read_bytes(), b'root fixture')
            self.assertFalse((first / 'system.img').is_symlink())
            self.assertTrue((first / 'install.py').is_file())
            self.assertTrue((first / 'INSTALL.md').is_file())
            self.assertEqual(manifest['files']['system.img']['bytes'], 12)
            (system / 'system.img').unlink()
            failed = subprocess.run(command, capture_output=True)
            self.assertNotEqual(failed.returncode, 0)
            self.assertEqual(sorted(first.parent.iterdir()), sorted([first, second]))

if __name__ == '__main__':
    unittest.main()
