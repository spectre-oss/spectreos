import importlib.util
from contextlib import closing
import json
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]

def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / file)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value

supervisor = module('supervisor', 'build-supervisor.py')
notifier = module('notifier', 'deltachat-notify.py')
finish = module('finish', 'finish-build.py')

class Supervision(unittest.TestCase):
    def test_release_checksum_failure_prevents_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = ['boot.img', 'system.img', 'install.py', 'INSTALL.md', 'kernel/build.config', 'kernel/Image']
            files = {}
            for name in names:
                path = root / name; path.parent.mkdir(exist_ok=True)
                path.write_bytes(b'content')
                files[name] = {'bytes': 7, 'sha256': hashlib.sha256(b'content').hexdigest()}
            (root / 'manifest.json').write_text(json.dumps({'kind':'system','panel':'tianma','files':files}))
            finish.verify_release(root)
            (root / 'system.img').write_bytes(b'corrupt')
            with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
                finish.verify_release(root)
            self.assertTrue((root / 'boot.img').exists())

    def test_partial_release_cannot_complete(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'manifest.json').write_text(json.dumps({'kind':'kernel','panel':'tianma','files':{}}))
            with self.assertRaisesRegex(ValueError, 'complete Tianma'):
                finish.verify_release(root)

    def test_repair_context_preserves_findings_after_failed_turn(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / '.build').mkdir()
            (root / '.build/grok-progress.md').write_text('Confirmed: bindings force GIR')
            with patch.object(supervisor, 'ROOT', root):
                context = supervisor.repair_context({'agentFindings': 'Next: change bindings flag'})
            self.assertIn('bindings force GIR', context)
            self.assertIn('change bindings flag', context)

    def test_requested_model_profile(self):
        self.assertEqual(supervisor.GROK_MODEL, 'grok-4.7')
        self.assertEqual(supervisor.GROK_EFFORT, 'low')

    def test_duplicate_notification_survives_reopen(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / '.data').mkdir()
            with closing(notifier.database(root)) as db:
                notifier.enqueue(db, 'event', 'First notice')
            with closing(notifier.database(root)) as db:
                notifier.enqueue(db, 'event', 'Duplicate notice')
                self.assertEqual(db.execute('SELECT text FROM notices').fetchall(), [('First notice',)])

    def test_long_message_rejected(self):
        with self.assertRaises(ValueError):
            notifier.enqueue(None, 'event', 'x' * 3501)

    def test_no_duplicate_build(self):
        with patch.object(supervisor, 'active_builds', return_value=[123]):
            with self.assertRaisesRegex(RuntimeError, 'already running'):
                supervisor.start_build({})

    def test_stale_camera_log_not_supplied_as_current_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / '.build').mkdir()
            (root / '.build/image.log').write_text('Current build is running')
            (root / '.build/retry-camera.log').write_text('Historical failure')
            with patch.object(supervisor, 'ROOT', root), patch.object(supervisor, 'active_builds', return_value=[1]):
                result = supervisor.evidence()
            self.assertNotIn('retry-camera.log', result['logs'])
            self.assertEqual(result['builds'], [1])
            self.assertIsNone(result['exit'])

    def test_state_write_is_readable_and_no_temporary_left(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'state.json'
            supervisor.save(path, {'phase': 'building', 'round': 2})
            self.assertEqual(json.loads(path.read_text())['round'], 2)
            self.assertFalse(path.with_suffix('.tmp').exists())

if __name__ == '__main__':
    unittest.main()
