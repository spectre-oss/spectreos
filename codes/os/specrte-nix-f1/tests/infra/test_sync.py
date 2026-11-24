import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('code_sync', Path(__file__).resolve().parents[2] / 'infra/sync.py')
sync = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sync)


class SyncTests(unittest.TestCase):
    def item(self, value):
        return {'hash': value, 'executable': False}

    def test_two_sided_conflict_stops_plan(self):
        actions, conflicts = sync.plan({'a': ['old', False]},
                                       {'a': self.item('local')}, {'a': self.item('remote')})
        self.assertEqual(actions, [])
        self.assertEqual(conflicts, ['a'])

    def test_add_edit_delete_both_directions(self):
        actions, conflicts = sync.plan({'deleted': ['old', False], 'edited': ['old', False]},
            {'local': self.item('l'), 'edited': self.item('new')},
            {'remote': self.item('r'), 'deleted': self.item('old'), 'edited': self.item('old')})
        self.assertEqual(conflicts, [])
        self.assertEqual({(a[0], a[1], a[2] is None) for a in actions},
                         {('remote', 'deleted', True), ('remote', 'edited', False),
                          ('remote', 'local', False), ('local', 'remote', False)})

    def test_delete_versus_edit_conflict(self):
        self.assertEqual(sync.plan({'a': ['old', False]}, {}, {'a': self.item('new')})[1], ['a'])

    def test_filters_and_worker(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'code.nix').write_text('{}')
            (root / 'boot.img').write_bytes(b'image')
            (root / 'binary').write_bytes(b'\0')
            (root / '.git').mkdir()
            (root / '.git/config').write_text('secret')
            (root / 'mobile-nixos').mkdir()
            (root / 'mobile-nixos/source.nix').write_text('{}')
            (root / 'link').symlink_to('/etc/passwd')
            result = sync.worker({'root': directory, 'operation': 'scan'})
            self.assertEqual(set(result['files']), {'code.nix'})
            self.assertEqual(result['skipped'], ['binary', 'link'])

    def test_race_check_and_backups(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            p = root / 'code'
            p.write_text('original')
            before = sync.signature(sync.record(p))
            p.write_text('new edit')
            change = {'name': 'code', 'expected': before, 'item': None}
            with self.assertRaises(RuntimeError):
                sync.apply(root, [change], root / 'backups')
            self.assertEqual(p.read_text(), 'new edit')
            change['expected'] = sync.signature(sync.record(p))
            sync.apply(root, [change], root / 'backups')
            self.assertFalse(p.exists())
            self.assertEqual((root / 'backups/code').read_text(), 'new edit')

    def test_refuses_symlink_parent(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'escape').symlink_to('/tmp')
            with self.assertRaises(RuntimeError):
                sync.safe_path(root, 'escape/file')

    def test_split_documentation_mapping(self):
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            code = workspace / 'codes/spectre-nix'
            docs = workspace / 'docs/spectre-nix'
            general = workspace / 'docs/general'
            for folder in [code, docs, general]:
                folder.mkdir(parents=True)
            (code / 'module.nix').write_text('{}')
            (code / 'docs').symlink_to('../../docs/spectre-nix', target_is_directory=True)
            (docs / 'guide.md').write_text('guide')
            (general / 'history.md').write_text('local general knowledge')
            result = sync.local_scan(code, docs)
            self.assertEqual(set(result['files']), {'module.nix', 'docs/guide.md'})
            self.assertEqual(result['skipped'], [])
            old = result['files']['docs/guide.md']
            (code / 'incoming').write_text('updated guide')
            new = sync.record(code / 'incoming')
            (code / 'incoming').unlink()
            sync.local_apply(code, docs, [{'name': 'docs/guide.md',
                'item': new, 'expected': sync.signature(old)}], code / '.code-sync/backups/test')
            self.assertEqual((docs / 'guide.md').read_text(), 'updated guide')
            self.assertEqual((code / '.code-sync/backups/test/docs/guide.md').read_text(), 'guide')
            self.assertEqual((general / 'history.md').read_text(), 'local general knowledge')

    def test_missing_mapped_docs_aborts(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaises(RuntimeError):
                sync.local_scan(root, root / 'missing')


if __name__ == '__main__':
    unittest.main()
