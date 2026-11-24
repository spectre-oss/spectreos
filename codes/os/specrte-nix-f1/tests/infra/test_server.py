import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'infra'))
from server import TARGETS, verify_release


class ReleaseTests(unittest.TestCase):
    def test_no_install_or_flash_target(self):
        self.assertNotIn('install', TARGETS)
        self.assertNotIn('boot', TARGETS)

    def test_manifest_integrity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data = b'test artifact'
            (root / 'boot.img').write_bytes(data)
            manifest = {'files': {'boot.img': {'bytes': len(data),
                          'sha256': hashlib.sha256(data).hexdigest()}}}
            (root / 'manifest.json').write_text(json.dumps(manifest))
            self.assertEqual(verify_release(root), 1)
            (root / 'boot.img').write_bytes(b'tampered')
            with self.assertRaises(RuntimeError):
                verify_release(root)

    def test_manifest_path_traversal(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'manifest.json').write_text(json.dumps({'files': {'../outside': {}}}))
            with self.assertRaises(RuntimeError):
                verify_release(root)


if __name__ == '__main__':
    unittest.main()
