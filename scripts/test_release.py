#!/usr/bin/env python3
import contextlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import components  # noqa: E402
import release  # noqa: E402

DEPS = {
    "libbhoo": [],
    "files": ["libbhoo"],
    "settings": ["libbhoo"],
    "calendar": ["libbhoo", "files"],
    "wisp": ["libbhoo", "settings", "files", "calendar"],
    "keyboard": ["libbhoo"],
}
GIT_ENV = {
    "GIT_AUTHOR_NAME": "T", "GIT_AUTHOR_EMAIL": "t@example.invalid",
    "GIT_COMMITTER_NAME": "T", "GIT_COMMITTER_EMAIL": "t@example.invalid",
    "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_CONFIG_SYSTEM": "/dev/null",
}


def commits(*messages):
    return [release.Commit(f"{i:040x}", m) for i, m in enumerate(messages, 1)]


def levels(*messages):
    return [x.level for x in release.changes_from_commits(commits(*messages))]


class CommitParsing(unittest.TestCase):
    def test_levels(self):
        self.assertEqual(levels("feat: a"), [2])
        self.assertEqual(levels("feat(ui): a", "fix: b", "perf: c"), [2, 1, 1])
        self.assertEqual(levels("chore: a", "docs: b", "ci: c", "style: d", "refactor: e",
                                "test: f", "build: g", "not conventional"), [])
        self.assertEqual(levels("feat!: a"), [3])
        self.assertEqual(levels("fix(x)!: a"), [3])

    def test_breaking_footer_multiline(self):
        found = release.changes_from_commits(commits(
            "chore: x\n\nBREAKING CHANGE: wire format\nchanged again\n\nRefs: #4"))
        self.assertEqual(len(found), 1)
        self.assertEqual(found[0].breaking, "wire format changed again")
        self.assertEqual(release.changes_from_commits(commits("docs: a\n\nBREAKING-CHANGE: yes"))[0].level, 3)

    def test_revert_cancellation(self):
        first = release.Commit("a" * 40, "feat: add thing")
        second = release.Commit("b" * 40, 'Revert "feat: add thing"\n\nThis reverts commit ' + "a" * 40 + ".")
        self.assertEqual(release.changes_from_commits([first, second]), [])
        orphan = release.Commit("c" * 40, 'Revert "x"\n\nThis reverts commit ' + "d" * 40 + ".")
        self.assertEqual([x.kind for x in release.changes_from_commits([orphan])], ["revert"])

    def test_release_commits_ignored(self):
        self.assertEqual(release.changes_from_commits(commits(
            "chore(release): wisp 0.2.0 [skip ci]\n\nBREAKING CHANGE: noise")), [])

    def test_bump(self):
        self.assertEqual(release.bump("0.1.0", 2), "0.2.0")
        self.assertEqual(release.bump("0.1.0", 1), "0.1.1")
        self.assertEqual(release.bump("0.12.1", 2), "0.13.0")
        self.assertEqual(release.bump("1.2.3", 3), "1.3.0")
        self.assertEqual(release.bump("1.2.3", 3, allow_major=True), "2.0.0")
        with self.assertRaises(release.ReleaseError):
            release.bump("1.2", 1)

    def test_markdown_escaped(self):
        notes = release.release_notes(components.BY_NAME["wisp"], "0.2.0", "", [
            release.Change("a" * 40, "feat", "x*", "<b>hi</b> [l](u)", "", 2)], "2026-01-01")
        self.assertNotIn("<b>", notes)
        self.assertIn("\\[l\\]", notes)

    def test_changelog_prepend(self):
        base = "# Changelog\n\nIntro.\n"
        one = release.prepend_changelog(base, "0.2.0", "## [0.2.0](u) (d)\n")
        two = release.prepend_changelog(one, "0.3.0", "## [0.3.0](u) (d)\n")
        self.assertLess(two.index("0.3.0"), two.index("0.2.0"))
        self.assertTrue(two.startswith("# Changelog\n\nIntro."))
        with self.assertRaises(release.ReleaseError):
            release.prepend_changelog(two, "0.2.0", "x")


class Repo:
    """A throwaway monorepo with the real component layout."""

    def __init__(self, root):
        self.root = Path(root)
        self.root.mkdir(parents=True)
        self.git("init", "-q", "-b", "main")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "tag.gpgsign", "false")
        for name, deps in DEPS.items():
            component = components.BY_NAME[name]
            self.crate(component.directory, component.package, deps)
            self.write(component.version_file, "0.1.0\n")
            self.write(component.changelog, "# Changelog\n")
        self.write("codes/os/specrte-nix-f1/VERSION", "0.12.1\n")
        self.write("codes/os/specrte-nix-f1/CHANGELOG.md", "# Changelog\n")
        self.crate("codes/apps/bhoo-web", "bhoo-web-demo", ["libbhoo", "wisp"])
        self.crate("codes/utils/wisp_remote_viewer", "wisp_remote_viewer", [])
        self.crate("tools/bhoo-web-debug", "bhoo-web-debug", ["libbhoo", "files", "settings", "calendar", "keyboard", "wisp"])
        # A foreign project that must never be touched.
        self.crate("codes/shiraz-core", "shiraz-core", [])
        lock = self.root / "codes/shiraz-core/Cargo.lock"
        lock.write_text(lock.read_text() + '\n[[package]]\nname = "libbhoo"\nversion = "0.1.0"\n')
        self.write("README.md", "# SpectreOS\n")
        self.write("landing/index.html", "x")
        self.write("docs/a.md", "x")
        self.commit("chore: initial import")

    def git(self, *args, check=True):
        env = {**os.environ, **GIT_ENV}
        result = subprocess.run(["git", *args], cwd=self.root, env=env, text=True,
                                capture_output=True, check=False)
        if check and result.returncode:
            raise AssertionError(result.stderr)
        return result.stdout.strip()

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def crate(self, directory, package, deps):
        lines = [f'[package]\nname = "{package}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]']
        for dep in deps:
            target = os.path.relpath(self.root / components.BY_NAME[dep].directory, self.root / directory)
            lines.append(f'{dep} = {{ package = "{components.BY_NAME[dep].package}", path = "{target}" }}')
        self.write(f"{directory}/Cargo.toml", "\n".join(lines) + "\n")
        self.write(f"{directory}/src/lib.rs", "")
        subprocess.run(["cargo", "generate-lockfile", "--offline", "--manifest-path",
                        str(self.root / directory / "Cargo.toml")],
                       check=True, capture_output=True)

    def commit(self, message, **files):
        for relative, text in files.items():
            self.write(relative.replace("__", "/"), text)
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", message)
        return self.git("rev-parse", "HEAD")

    def touch(self, message, relative):
        path = self.root / relative
        existing = path.read_text() if path.exists() else ""
        return self.commit(message, **{relative.replace("/", "__"): existing + "x\n"})

    def read(self, relative):
        return (self.root / relative).read_text()

    def lock_version(self, directory, package):
        for entry in tomllib.loads(self.read(f"{directory}/Cargo.lock"))["package"]:
            if entry["name"] == package and "source" not in entry:
                return entry["version"]


@unittest.skipUnless(shutil.which("cargo") and shutil.which("git"), "needs git and cargo")
class Engine(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Repo(Path(self.tmp.name) / "repo")
        patcher = mock.patch.dict(os.environ, {**GIT_ENV, "GITHUB_EVENT_NAME": "push"})
        patcher.start()
        self.addCleanup(patcher.stop)

    def run_release(self, **kwargs):
        return release.release(self.repo.root, **kwargs)

    def names(self, result):
        return {r["name"]: r["version"] for r in result["released"]}

    def test_check_and_no_release(self):
        self.assertEqual(self.run_release(check=True)["versions"]["spectref1"], "0.12.1")
        result = self.run_release()
        self.assertEqual(result["released"], [])
        self.assertFalse(release.release_outputs(result)["any_released"] == "true")

    def test_independent_versions(self):
        self.repo.touch("feat(wisp): shell", "codes/apps/wisp/src/lib.rs")
        self.repo.touch("fix: radio", "codes/apps/keyboard/src/lib.rs")
        self.repo.touch("feat(os): kernel", "codes/os/specrte-nix-f1/kernel.cfg")
        self.repo.touch("docs: only", "codes/apps/files/README.md")
        self.repo.touch("feat: foreign", "codes/shiraz-core/src/lib.rs")
        self.repo.touch("feat: unrelated root", "scripts/other.py")
        result = self.run_release()
        self.assertEqual(self.names(result), {"wisp": "0.2.0", "keyboard": "0.1.1", "spectref1": "0.13.0"})
        self.assertFalse(any(f.startswith("codes/shiraz-core") for f in result["files"]))
        self.assertIn("codes/apps/bhoo-web/Cargo.lock", result["files"])
        self.assertIn("tools/bhoo-web-debug/Cargo.lock", result["files"])
        self.assertNotIn("codes/apps/files/Cargo.lock", result["files"])

    def test_breaking_capped_unless_allowed(self):
        self.repo.touch("feat!: rewrite", "codes/apps/files/src/lib.rs")
        self.assertEqual(self.names(self.run_release()), {"files": "0.2.0"})
        self.assertEqual(self.names(self.run_release(allow_major=True)), {"files": "1.0.0"})

    def test_write_single_commit_tags_and_locks(self):
        self.repo.touch("feat(ui): widgets", "codes/libs/ui/libbhoo/src/lib.rs")
        self.repo.touch("fix(files): bug", "codes/apps/files/src/lib.rs")
        before = self.repo.git("rev-parse", "HEAD")
        result = self.run_release(write=True)
        after = self.repo.git("rev-parse", "HEAD")
        self.assertEqual(result["sha"], after)
        self.assertEqual(self.repo.git("rev-parse", "HEAD~1"), before)
        self.assertEqual(self.repo.git("log", "-1", "--format=%s"),
                         "chore(release): libbhoo 0.2.0, files 0.1.1 [skip ci]")
        for tag in ("libbhoo-v0.2.0", "files-v0.1.1"):
            self.assertEqual(self.repo.git("rev-parse", f"{tag}^{{commit}}"), after)
            self.assertEqual(self.repo.git("cat-file", "-t", tag), "tag")
        self.assertEqual(self.repo.read("codes/libs/ui/libbhoo/.version"), "0.2.0\n")
        self.assertEqual(self.repo.read("codes/apps/files/.version"), "0.1.1\n")
        self.assertIn('version = "0.2.0"', self.repo.read("codes/libs/ui/libbhoo/Cargo.toml"))
        for lock in ("wisp", "calendar", "files", "settings", "keyboard", "bhoo-web"):
            self.assertEqual(self.repo.lock_version(f"codes/apps/{lock}", "libbhoo"), "0.2.0")
        self.assertEqual(self.repo.lock_version("tools/bhoo-web-debug", "libbhoo"), "0.2.0")
        self.assertEqual(self.repo.lock_version("tools/bhoo-web-debug", "spectre-files"), "0.1.1")
        self.assertEqual(self.repo.lock_version("codes/apps/calendar", "spectre-files"), "0.1.1")
        self.assertEqual(self.repo.lock_version("codes/shiraz-core", "libbhoo"), "0.1.0")
        self.assertEqual(self.repo.git("status", "--porcelain"), "")
        self.assertIn("[0.2.0]", self.repo.read("codes/libs/ui/libbhoo/CHANGELOG.md"))
        # Everything still resolves with --locked, and the next run has nothing to do.
        self.assertEqual(self.run_release(check=True)["versions"]["libbhoo"], "0.2.0")
        self.assertEqual(self.run_release()["released"], [])
        # Subsequent fix bumps from the tag, not from scratch.
        self.repo.touch("fix: again", "codes/libs/ui/libbhoo/src/lib.rs")
        self.assertEqual(self.names(self.run_release(write=True)), {"libbhoo": "0.2.1"})
        self.assertEqual(self.repo.lock_version("codes/apps/wisp", "libbhoo"), "0.2.1")

    def test_untagged_base_is_current_version(self):
        self.repo.touch("fix: a", "codes/apps/calendar/src/lib.rs")
        self.assertEqual(self.names(self.run_release()), {"calendar": "0.1.1"})

    def test_downgrade_refused_and_existing_tag(self):
        self.repo.git("tag", "-a", "wisp-v0.1.0", "-m", "x")
        for name in ("codes/apps/wisp/.version", ):
            self.repo.write(name, "0.3.0\n")
        manifest = self.repo.read("codes/apps/wisp/Cargo.toml").replace('version = "0.1.0"', 'version = "0.3.0"', 1)
        self.repo.write("codes/apps/wisp/Cargo.toml", manifest)
        for lock in ("codes/apps/wisp/Cargo.lock", "codes/apps/bhoo-web/Cargo.lock", "tools/bhoo-web-debug/Cargo.lock"):
            self.repo.write(lock, release.update_lock_text(self.repo.read(lock), {"wisp": "0.3.0"}))
        self.repo.commit("fix: wisp")
        with self.assertRaisesRegex(release.ReleaseError, "downgrade"):
            self.run_release()

    def test_planned_tag_exists(self):
        self.repo.touch("feat: a", "codes/apps/settings/src/lib.rs")
        # A competing tag on an unmerged branch is not "merged" but still collides.
        self.repo.git("checkout", "-q", "-b", "other")
        self.repo.commit("chore: other")
        self.repo.git("tag", "settings-v0.2.0")
        self.repo.git("checkout", "-q", "main")
        with self.assertRaisesRegex(release.ReleaseError, "tag"):
            self.run_release()

    def test_force_components(self):
        result = self.run_release(force_components="wisp, spectref1")
        self.assertEqual(self.names(result), {"wisp": "0.1.1", "spectref1": "0.12.2"})
        self.assertIn("Manual release", result["notes"]["wisp"])
        self.repo.touch("feat: a", "codes/apps/wisp/src/lib.rs")
        self.assertEqual(self.names(self.run_release(force_components="wisp")), {"wisp": "0.2.0"})
        with self.assertRaises(release.ReleaseError):
            self.run_release(force_components="shiraz-core")

    def test_write_requires_main_clean_and_push(self):
        self.repo.touch("feat: a", "codes/apps/wisp/src/lib.rs")
        with self.assertRaises(release.ReleaseError):
            self.run_release(push=True)
        self.repo.git("checkout", "-q", "-b", "topic")
        with self.assertRaisesRegex(release.ReleaseError, "only on main"):
            self.run_release(write=True)
        self.repo.git("checkout", "-q", "main")
        (self.repo.root / "dirty").write_text("x")
        with self.assertRaisesRegex(release.ReleaseError, "clean"):
            self.run_release(write=True)
        (self.repo.root / "dirty").unlink()
        with mock.patch.dict(os.environ, {"GITHUB_EVENT_NAME": "pull_request"}):
            with self.assertRaisesRegex(release.ReleaseError, "pull request"):
                self.run_release(write=True)

    def test_rollback_restores_everything(self):
        self.repo.touch("feat: a", "codes/apps/wisp/src/lib.rs")
        head = self.repo.git("rev-parse", "HEAD")
        calls = {"n": 0}
        real = release.verify_versions

        def flaky(*args, **kwargs):
            calls["n"] += 1
            if calls["n"] == 2:  # inside prepare() after files were written
                raise release.ReleaseError("boom")
            return real(*args, **kwargs)

        with mock.patch.object(release, "verify_versions", flaky):
            with self.assertRaisesRegex(release.ReleaseError, "boom"):
                self.run_release(write=True)
        self.assertEqual(self.repo.git("rev-parse", "HEAD"), head)
        self.assertEqual(self.repo.git("status", "--porcelain"), "")
        self.assertEqual(self.repo.git("tag", "--list"), "")

    def test_rollback_after_commit_removes_tags(self):
        self.repo.touch("feat: a", "codes/apps/wisp/src/lib.rs")
        self.repo.touch("feat: b", "codes/apps/files/src/lib.rs")
        head = self.repo.git("rev-parse", "HEAD")
        real = release.git
        state = {"tags": 0}

        def failing(root, *args, **kw):
            if "tag" in args and "-a" in args:
                state["tags"] += 1
                if state["tags"] == 2:
                    raise release.ReleaseError("tag failed")
            return real(root, *args, **kw)

        with mock.patch.object(release, "git", failing):
            with self.assertRaises(release.ReleaseError):
                self.run_release(write=True)
        self.assertEqual(self.repo.git("rev-parse", "HEAD"), head)
        self.assertEqual(self.repo.git("tag", "--list"), "")
        self.assertEqual(self.repo.git("status", "--porcelain"), "")
        self.assertEqual(self.repo.read("codes/apps/wisp/.version"), "0.1.0\n")

    def test_push_and_recovery(self):
        remote = Path(self.tmp.name) / "origin.git"
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(remote)], check=True)
        self.repo.git("remote", "add", "origin", str(Path(self.tmp.name) / "missing.git"))
        self.repo.touch("feat: a", "codes/apps/wisp/src/lib.rs")
        with self.assertRaises(release.ReleaseError):
            self.run_release(write=True, push=True)
        # The commit and tag exist locally; the push failed. Re-run recovers.
        self.assertEqual(self.repo.git("log", "-1", "--format=%s"), "chore(release): wisp 0.2.0 [skip ci]")
        self.repo.git("remote", "set-url", "origin", str(remote))
        result = self.run_release(write=True, push=True)
        self.assertEqual(self.names(result), {"wisp": "0.2.0"})
        self.assertTrue(result["released"][0]["recovered"])
        head = self.repo.git("rev-parse", "HEAD")
        self.assertEqual(subprocess.run(["git", "-C", str(remote), "rev-parse", "main"], text=True,
                                        capture_output=True).stdout.strip(), head)
        self.assertEqual(subprocess.run(["git", "-C", str(remote), "tag", "--list"], text=True,
                                        capture_output=True).stdout.split(), ["wisp-v0.2.0"])

    def test_one_atomic_push_for_all_tags(self):
        remote = Path(self.tmp.name) / "origin.git"
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(remote)], check=True)
        self.repo.git("remote", "add", "origin", str(remote))
        self.repo.touch("feat: a", "codes/apps/wisp/src/lib.rs")
        self.repo.touch("fix: a", "codes/apps/files/src/lib.rs")
        calls = []
        real = release.push_release
        with mock.patch.object(release, "push_release", lambda *a: (calls.append(a), real(*a))):
            self.run_release(write=True, push=True)
        self.assertEqual(len(calls), 1)
        self.assertEqual(sorted(calls[0][1]), ["files-v0.1.1", "wisp-v0.2.0"])
        tags = subprocess.run(["git", "-C", str(remote), "tag", "--list"], text=True,
                              capture_output=True).stdout.split()
        self.assertEqual(sorted(tags), ["files-v0.1.1", "wisp-v0.2.0"])

    def test_not_a_prepared_release(self):
        self.repo.git("tag", "-a", "wisp-v0.1.0", "-m", "manual")
        with self.assertRaisesRegex(release.ReleaseError, "not a prepared release"):
            self.run_release(write=True, push=True)

    def test_stale_lock_is_rejected(self):
        lock = self.repo.read("codes/apps/wisp/Cargo.lock")
        stale = release.update_lock_text(lock, {"libbhoo": "0.9.0"})
        self.repo.commit("chore: stale", **{"codes__apps__wisp__Cargo.lock": stale})
        with self.assertRaisesRegex(release.ReleaseError, "disagrees"):
            self.run_release(check=True)

    def test_landing_detection(self):
        base = self.repo.git("rev-parse", "HEAD")
        self.repo.touch("chore: code", "codes/apps/files/src/lib.rs")
        self.repo.touch("chore: shiraz doc", "codes/shiraz-core/README.md")
        self.assertFalse(self.run_release(before=base)["deploy_landing"])
        self.assertTrue(self.run_release(before=base, force_landing=True)["deploy_landing"])
        self.assertTrue(self.run_release(before="0" * 40)["deploy_landing"])
        self.assertTrue(self.run_release(before="f" * 40)["deploy_landing"])
        self.assertFalse(self.run_release(before="")["deploy_landing"])
        mid = self.repo.git("rev-parse", "HEAD")
        self.repo.touch("docs: x", "docs/b.md")
        self.assertTrue(self.run_release(before=mid)["deploy_landing"])
        mid = self.repo.git("rev-parse", "HEAD")
        self.repo.touch("chore: readme", "codes/apps/wisp/README.md")
        self.assertTrue(self.run_release(before=mid)["deploy_landing"])
        with self.assertRaises(release.ReleaseError):
            self.run_release(before="--evil")

    def test_outputs(self):
        self.repo.touch("feat: a", "codes/libs/ui/libbhoo/src/lib.rs")
        self.repo.touch("feat: b", "codes/apps/files/src/lib.rs")
        self.repo.touch("fix: c", "codes/apps/calendar/src/lib.rs")
        self.repo.touch("feat: d", "codes/os/specrte-nix-f1/x")
        out = release.release_outputs(self.run_release(write=True))
        self.assertEqual(out["sha"], self.repo.git("rev-parse", "HEAD"))
        self.assertEqual(out["any_released"], "true")
        self.assertEqual(json.loads(out["released_components"]), ["libbhoo", "files", "calendar", "spectref1"])
        self.assertEqual((out["publish_libbhoo"], out["libbhoo_version"], out["libbhoo_tag"]),
                         ("true", "0.2.0", "libbhoo-v0.2.0"))
        matrix = json.loads(out["release_apps_matrix"])
        self.assertEqual(matrix, [{
            "name": "files", "package": "spectre-files", "binary": "spectre-files",
            "dir": "codes/apps/files", "features": "wayland", "version": "0.2.0",
            "tag": "files-v0.2.0", "changelog": "codes/apps/files/CHANGELOG.md"}])
        self.assertEqual((out["release_calendar"], out["calendar_version"], out["calendar_tag"]),
                         ("true", "0.1.1", "calendar-v0.1.1"))
        self.assertEqual((out["release_spectref1"], out["spectref1_version"], out["spectref1_tag"]),
                         ("true", "0.13.0", "spectref1-v0.13.0"))
        self.assertEqual(out["has_release_apps"], "true")
        empty = release.release_outputs(self.run_release())
        self.assertEqual((empty["any_released"], empty["publish_libbhoo"], empty["has_release_apps"],
                          empty["release_apps_matrix"], empty["libbhoo_tag"]), ("false", "false", "false", "[]", ""))
        path = Path(self.tmp.name) / "gh_out"
        release.write_outputs(path, out)
        lines = path.read_text().splitlines()
        self.assertTrue(all("=" in line for line in lines))
        self.assertEqual(len(lines), len(out))
        with self.assertRaises(release.ReleaseError):
            release.write_outputs(path, {"x": "a\nb"})

    def test_cli(self):
        self.repo.touch("feat: a", "codes/apps/keyboard/src/lib.rs")
        out = Path(self.tmp.name) / "o"
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(release.main(["--repo", str(self.repo.root), "--dry-run",
                                           "--github-output", str(out)]), 0)
            self.assertEqual(release.main(["--repo", str(self.repo.root), "--check"]), 0)
        self.assertIn("released_components=[\"keyboard\"]", out.read_text())
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(release.main(["--repo", str(self.repo.root), "--push"]), 1)
        self.assertEqual(self.repo.git("tag", "--list"), "")

    def test_detect_pr(self):
        base = self.repo.git("rev-parse", "HEAD")

        def detect(*paths):
            for p in paths:
                self.repo.touch("chore: x", p)
            return release.detect_pr(self.repo.root, base, self.repo.git("rev-parse", "HEAD"))

        out = detect("codes/shiraz-core/src/lib.rs")
        self.assertEqual(out, {"check_scripts": "false", "check_libbhoo": "false", "has_check_apps": "false",
                               "check_apps_matrix": "[]", "check_spectref1": "false", "check_landing": "false"})
        out = detect("codes/apps/keyboard/src/lib.rs")
        self.assertEqual([a["name"] for a in json.loads(out["check_apps_matrix"])], ["keyboard"])
        self.assertEqual(out["check_libbhoo"], "false")
        out = detect("codes/apps/files/src/lib.rs")
        self.assertEqual([a["name"] for a in json.loads(out["check_apps_matrix"])],
                         ["wisp", "keyboard", "files", "calendar", "bhoo-web"])
        out = detect("codes/os/specrte-nix-f1/x", "scripts/y.py", "docs/z.md")
        self.assertEqual((out["check_spectref1"], out["check_scripts"], out["check_landing"]),
                         ("true", "true", "true"))
        out = detect("codes/libs/ui/libbhoo/src/lib.rs")
        self.assertEqual(out["check_libbhoo"], "true")
        self.assertEqual(len(json.loads(out["check_apps_matrix"])), 7)
        self.assertEqual(json.loads(out["check_apps_matrix"])[0]["manifest"], "codes/apps/wisp/Cargo.toml")
        # read-only: no refs, tags or files changed
        self.assertEqual(self.repo.git("tag", "--list"), "")
        self.assertEqual(self.repo.git("status", "--porcelain"), "")
        out = detect(".github/workflows/x.yml")
        self.assertEqual(len(json.loads(out["check_apps_matrix"])), 7)
        with self.assertRaises(release.ReleaseError):
            release.detect_pr(self.repo.root, "--bad", "HEAD")

    def test_detect_pr_cli(self):
        base = self.repo.git("rev-parse", "HEAD")
        self.repo.touch("chore: x", "codes/apps/settings/src/lib.rs")
        out = Path(self.tmp.name) / "pr"
        with contextlib.redirect_stdout(io.StringIO()):
            code = release.main(["detect-pr", "--repo", str(self.repo.root), "--base", base,
                                 "--head", "HEAD".replace("HEAD", self.repo.git("rev-parse", "HEAD")),
                                 "--github-output", str(out)])
        self.assertEqual(code, 0)
        values = dict(line.split("=", 1) for line in out.read_text().splitlines())
        self.assertEqual([a["name"] for a in json.loads(values["check_apps_matrix"])], ["wisp", "settings", "bhoo-web"])


class RegistrySanity(unittest.TestCase):
    def test_registry(self):
        self.assertEqual([c.name for c in components.COMPONENTS],
                         ["libbhoo", "wisp", "keyboard", "settings", "files", "calendar", "spectref1"])
        self.assertTrue(all("shiraz-core" not in c.directory for c in components.COMPONENTS))
        self.assertTrue(components.is_landing_path("docs/x.md"))
        self.assertTrue(components.is_landing_path("codes/apps/wisp/README.md"))
        self.assertFalse(components.is_landing_path("codes/shiraz-core/README.md"))
        self.assertFalse(components.is_landing_path("codes/apps/wisp/src/lib.rs"))

    def test_real_repo_is_consistent(self):
        root = Path(__file__).resolve().parents[1]
        if not shutil.which("cargo") or not (root / "codes/libs/ui/libbhoo/.version").exists():
            self.skipTest("not a SpectreOS checkout")
        versions, _ = release.verify_versions(root)
        self.assertEqual(versions["spectref1"], (root / "codes/os/specrte-nix-f1/VERSION").read_text().strip())


if __name__ == "__main__":
    unittest.main()
