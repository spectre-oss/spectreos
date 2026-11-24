#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('version', ROOT / 'scripts/version.py')
version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version)

class Versions(unittest.TestCase):
    def test_conventional_commits(self):
        self.assertEqual(version.calculate(['initial import']), '0.0.1')
        self.assertEqual(version.calculate(['fix: repair', '\nfeat(ui): screen',
                                            'chore(release): 0.1.0', 'perf: faster']), '0.1.1')
        self.assertEqual(version.calculate(['feat!: incompatible', 'fix: repair']), '1.0.1')
        self.assertEqual(version.calculate(['feat(api): change\n\nBREAKING CHANGE: wire format']), '1.0.0')
        self.assertEqual(version.calculate(['docs: mention feat: example']), '0.0.1')

    def test_real_hook_and_staged_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'scripts').mkdir()
            (root / '.githooks').mkdir()
            shutil.copy(ROOT / 'scripts/version.py', root / 'scripts/version.py')
            shutil.copy(ROOT / '.githooks/post-commit', root / '.githooks/post-commit')
            def git(*args):
                return subprocess.check_output(['git', '-C', tmp, *args], stderr=subprocess.STDOUT).decode().strip()
            git('init', '-q')
            git('config', 'user.name', 'Version Test')
            git('config', 'user.email', 'test@example.invalid')
            git('config', 'commit.gpgsign', 'false')
            git('config', 'core.hooksPath', '.githooks')
            git('add', '.')
            git('commit', '-qm', 'feat: initial feature')
            self.assertEqual((root / 'VERSION').read_text(), '0.1.0\n')
            self.assertEqual(git('rev-list', '--count', 'HEAD'), '2')
            (root / 'pending').write_text('keep staged')
            git('add', 'pending')
            git('commit', '--allow-empty', '--only', '-qm', 'fix: correction')
            self.assertEqual((root / 'VERSION').read_text(), '0.1.1\n')
            self.assertEqual(git('diff', '--cached', '--name-only'), 'pending')
            self.assertEqual(git('rev-list', '--count', 'HEAD'), '4')
            git('commit', '--allow-empty', '--only', '-qm', 'docs: clarify')
            self.assertEqual(git('rev-list', '--count', 'HEAD'), '5')

if __name__ == '__main__':
    unittest.main()
