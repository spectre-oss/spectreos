#!/usr/bin/env python3
import contextlib
import io
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import components  # noqa: E402
import publish_release as pub  # noqa: E402

CHANGELOG = """# Changelog

## [0.3.0](https://x/compare/a...b) (2026-01-02)

### Features

* three

## [0.2.0](https://x/compare/a...b) (2026-01-01)

* two
"""
SHA = "a" * 40


class FakeGh:
    def __init__(self, existing=None, files=None):
        self.existing, self.files, self.calls = existing, files or {}, []

    def view(self, tag):
        self.calls.append(("view", tag))
        return self.existing

    def create(self, tag, sha, title, notes, files, latest):
        self.calls.append(("create", tag, sha, title, notes, [f.name for f in files], latest))

    def upload(self, tag, files):
        self.calls.append(("upload", tag, [f.name for f in files]))

    def download(self, tag, name):
        self.calls.append(("download", name))
        path = Path(tempfile.mkdtemp()) / name
        path.write_bytes(self.files[name])
        return path


class Sections(unittest.TestCase):
    def test_section(self):
        self.assertIn("three", pub.changelog_section(CHANGELOG, "0.3.0"))
        self.assertNotIn("two", pub.changelog_section(CHANGELOG, "0.3.0"))
        self.assertIn("two", pub.changelog_section(CHANGELOG, "0.2.0"))
        self.assertEqual(pub.changelog_section(CHANGELOG, "0.1.0"), "")

    def test_notes_and_fallback(self):
        wisp = components.BY_NAME["wisp"]
        self.assertIn("three", pub.build_notes(wisp, "0.3.0", SHA, CHANGELOG))
        with self.assertRaises(pub.PublishError):
            pub.build_notes(wisp, "0.9.0", SHA, CHANGELOG)
        self.assertIn("# wisp 0.9.0", pub.build_notes(wisp, "0.9.0", SHA, CHANGELOG, allow_missing=True))

    def test_validate(self):
        self.assertEqual(pub.validate("wisp-v0.3.0", SHA)[1], "0.3.0")
        for tag, sha, kw in [("wisp-v0.3", SHA, {}), ("wisp-v0.3.0", "abc", {}), ("nope-v0.3.0", SHA, {}),
                             ("shiraz-core-v1.0.0", SHA, {}), ("wisp-v0.3.0", SHA, {"version": "0.3.1"}),
                             ("wisp-v0.3.0", SHA, {"component": "files"}), ("wisp-v01.0.0", SHA, {}),
                             ("wisp-v0.3.0", SHA.upper(), {})]:
            with self.assertRaises(pub.PublishError, msg=tag):
                pub.validate(tag, sha, **kw)

    def test_asset_rules(self):
        with tempfile.TemporaryDirectory() as tmp:
            good = Path(tmp) / "spectre-files-x86_64"
            good.write_bytes(b"x")
            other = Path(tmp) / "source.tar.gz"
            other.write_bytes(b"x")
            files = components.BY_NAME["files"]
            self.assertEqual(pub.validate_assets(files, [good]), ["spectre-files-x86_64"])
            with self.assertRaises(pub.PublishError):
                pub.validate_assets(files, [other])
            with self.assertRaises(pub.PublishError):
                pub.validate_assets(files, [Path(tmp) / "missing"])
            link = Path(tmp) / "spectre-files-link"
            link.symlink_to(good)
            with self.assertRaises(pub.PublishError):
                pub.validate_assets(files, [link])
            self.assertEqual(pub.validate_assets(components.BY_NAME["libbhoo"], [other]), ["source.tar.gz"])


class Publishing(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.asset = Path(self.tmp.name) / "spectre-files-x86_64"
        self.asset.write_bytes(b"binary")

    def test_dry_run_never_calls_gh(self):
        gh = FakeGh()
        result = pub.publish("files-v0.2.0", SHA, "t", "n", [self.asset], gh, write=False)
        self.assertTrue(result["dry_run"])
        self.assertEqual(gh.calls, [])

    def test_create_when_missing(self):
        gh = FakeGh()
        result = pub.publish("files-v0.2.0", SHA, "t", "n", [self.asset], gh, write=True)
        self.assertTrue(result["created"])
        self.assertEqual(gh.calls[1][0], "create")
        self.assertFalse(gh.calls[1][-1])

    def test_idempotent_and_never_clobbers(self):
        release = {"isDraft": False, "isPrerelease": False, "targetCommitish": SHA,
                   "assets": [{"name": self.asset.name}]}
        gh = FakeGh(release, {self.asset.name: b"binary"})
        result = pub.publish("files-v0.2.0", SHA, "t", "n", [self.asset], gh, write=True)
        self.assertEqual(result["uploaded"], [])
        self.assertNotIn("upload", [c[0] for c in gh.calls])
        gh = FakeGh(release, {self.asset.name: b"different"})
        with self.assertRaisesRegex(pub.PublishError, "conflicts"):
            pub.publish("files-v0.2.0", SHA, "t", "n", [self.asset], gh, write=True)
        release["assets"] = []
        gh = FakeGh(release)
        self.assertEqual(pub.publish("files-v0.2.0", SHA, "t", "n", [self.asset], gh, write=True)["uploaded"],
                         [self.asset.name])

    def test_refuses_bad_existing(self):
        for override in ({"isDraft": True}, {"isPrerelease": True}, {"targetCommitish": "b" * 40}):
            release = {"isDraft": False, "isPrerelease": False, "targetCommitish": SHA, "assets": [], **override}
            with self.assertRaises(pub.PublishError):
                pub.publish("files-v0.2.0", SHA, "t", "n", [], FakeGh(release), write=True)

    def test_gh_adapter_never_clobbers(self):
        source = Path(pub.__file__).read_text()
        self.assertNotIn("--clobber", source)
        self.assertNotIn("shell=True", source)


class Cli(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name)
        env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@e.invalid",
               "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@e.invalid"}

        def git(*a):
            return subprocess.run(["git", *a], cwd=root, env=env, text=True, capture_output=True, check=True).stdout.strip()
        git("init", "-q")
        git("config", "commit.gpgsign", "false")
        git("commit", "-q", "--allow-empty", "-m", "x")
        self.sha = git("rev-parse", "HEAD")
        git("tag", "-a", "files-v0.3.0", "-m", "x")
        self.root = root
        (root / "CHANGELOG.md").write_text(CHANGELOG)
        (root / "spectre-files-x86_64").write_bytes(b"b")
        self.cwd = os.getcwd()
        os.chdir(root)
        self.addCleanup(os.chdir, self.cwd)
        patcher = mock.patch.dict(os.environ, {"GITHUB_EVENT_NAME": "push"})
        patcher.start()
        self.addCleanup(patcher.stop)

    def main(self, *args, gh=None):
        err = io.StringIO()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(err):
            code = pub.main(list(args), gh=gh)
        return code, err.getvalue()

    def test_dry_run_default_and_write(self):
        base = ["--tag", "files-v0.3.0", "--sha", self.sha, "--changelog", "CHANGELOG.md",
                "--asset", "spectre-files-x86_64"]
        self.assertEqual(self.main(*base)[0], 0)
        gh = FakeGh()
        self.assertEqual(self.main(*base, "--write", gh=gh)[0], 0)
        self.assertEqual([c[0] for c in gh.calls], ["view", "create"])
        notes = gh.calls[1][4]
        self.assertIn(f"Source revision: {self.sha}", notes)
        self.assertIn("three", notes)

    def test_failures(self):
        base = ["--tag", "files-v0.3.0", "--sha", self.sha, "--changelog", "CHANGELOG.md"]
        with mock.patch.dict(os.environ, {"GITHUB_EVENT_NAME": "pull_request"}):
            code, err = self.main(*base, "--write", gh=FakeGh())
        self.assertEqual(code, 1)
        self.assertIn("pull request", err)
        with mock.patch.dict(os.environ, {"GITHUB_EVENT_NAME": "pull_request_target"}):
            self.assertEqual(self.main(*base, "--write", gh=FakeGh())[0], 1)
        self.assertEqual(self.main("--tag", "files-v0.3.0", "--sha", "b" * 40, "--changelog", "CHANGELOG.md")[0], 1)
        self.assertEqual(self.main("--tag", "files-v0.4.0", "--sha", self.sha, "--changelog", "CHANGELOG.md")[0], 1)
        self.assertEqual(self.main(*base, "--asset", "CHANGELOG.md")[0], 1)
        self.assertEqual(self.main(*base[:4], "--changelog", "CHANGELOG.md", "--version", "0.2.0")[0], 1)
        with mock.patch.dict(os.environ, {"GITHUB_EVENT_NAME": "pull_request"}):
            self.assertEqual(self.main(*base)[0], 0)  # dry-run is allowed


if __name__ == "__main__":
    unittest.main()
