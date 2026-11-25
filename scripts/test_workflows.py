#!/usr/bin/env python3
"""Verify the SpectreOS GitHub Actions workflows without third-party dependencies.

The workflows are parsed with a small YAML-subset parser (block mappings/sequences,
flow lists/maps, quoted and plain scalars, ``|``/``>`` block scalars) so the checks run
on any bare CI runner without PyYAML.  If PyYAML happens to be installed the parsed
documents are additionally cross-checked against it.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"
RELEASE = WORKFLOWS / "release.yml"
PR_CHECKS = WORKFLOWS / "pr-checks.yml"

STAGE2_JOBS = (
    "publish-libbhoo",
    "release-apps",
    "release-calendar",
    "release-spectref1",
    "build-landing",
)


# --------------------------------------------------------------------------------------
# Minimal YAML subset parser
# --------------------------------------------------------------------------------------


class YamlError(ValueError):
    pass


_KEY_RE = re.compile(
    r"""^(?P<key>"(?:[^"\\]|\\.)*"|'(?:[^']|'')*'|[^\s'"\[\]{},#&*!|>%@`-][^:]*?|-[^\s:][^:]*?)"""
    r"""\s*:(?:\s+(?P<value>.*))?$"""
)


def _strip_comment(text: str) -> str:
    """Remove a trailing ``# comment`` that is not inside quotes."""
    quote = None
    for i, ch in enumerate(text):
        if quote:
            if ch == quote:
                quote = None
        elif ch in "\"'":
            if i == 0 or text[i - 1] in " \t:[{,(":
                quote = ch
        elif ch == "#" and (i == 0 or text[i - 1] in " \t"):
            return text[:i].rstrip()
    return text.rstrip()


def _split_flow(text: str) -> list[str]:
    parts, depth, quote, cur = [], 0, None, []
    for ch in text:
        if quote:
            cur.append(ch)
            if ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
        elif ch == "," and depth == 0:
            parts.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    tail = "".join(cur).strip()
    if tail:
        parts.append(tail)
    return parts


def _scalar(text: str):
    text = text.strip()
    if text == "":
        return None
    if text[0] == '"':
        if not text.endswith('"') or len(text) < 2:
            raise YamlError(f"unterminated double-quoted scalar: {text!r}")
        return bytes(text[1:-1], "utf-8").decode("unicode_escape") if "\\" in text else text[1:-1]
    if text[0] == "'":
        if not text.endswith("'") or len(text) < 2:
            raise YamlError(f"unterminated single-quoted scalar: {text!r}")
        return text[1:-1].replace("''", "'")
    if text.startswith("[") and text.endswith("]"):
        return [_scalar(p) for p in _split_flow(text[1:-1])]
    if text.startswith("{") and not text.startswith("{{") and text.endswith("}"):
        result = {}
        for part in _split_flow(text[1:-1]):
            match = _KEY_RE.match(part)
            if not match:
                raise YamlError(f"bad flow mapping entry: {part!r}")
            result[_scalar(match.group("key"))] = _scalar(match.group("value") or "")
        return result
    if text in ("true", "True"):
        return True
    if text in ("false", "False"):
        return False
    if text in ("null", "~"):
        return None
    return text


class _Parser:
    def __init__(self, text: str):
        if "\t" in text:
            raise YamlError("tab characters are not allowed in workflow YAML")
        self.lines = text.splitlines()
        self.pos = 0

    # helpers ------------------------------------------------------------------------
    @staticmethod
    def _indent(line: str) -> int:
        return len(line) - len(line.lstrip(" "))

    def _skip_blank(self) -> None:
        while self.pos < len(self.lines):
            stripped = self.lines[self.pos].strip()
            if stripped == "" or stripped.startswith("#") or stripped == "---":
                self.pos += 1
            else:
                return

    def _peek(self):
        self._skip_blank()
        if self.pos >= len(self.lines):
            return None
        line = self.lines[self.pos]
        return self._indent(line), line.strip()

    def _block_scalar(self, header: str, parent_indent: int) -> str:
        folded = header[0] == ">"
        chomp = "-" if "-" in header else "+" if "+" in header else ""
        body: list[str] = []
        block_indent = None
        while self.pos < len(self.lines):
            line = self.lines[self.pos]
            if line.strip() == "":
                body.append("")
                self.pos += 1
                continue
            indent = self._indent(line)
            if indent <= parent_indent:
                break
            if block_indent is None:
                block_indent = indent
            body.append(line[block_indent:] if indent >= block_indent else line.strip())
            self.pos += 1
        while body and body[-1] == "":
            body.pop()
        if folded:
            text = " ".join(body)
        else:
            text = "\n".join(body)
        return text if chomp == "-" else text + "\n"

    # grammar ------------------------------------------------------------------------
    def parse_document(self):
        peeked = self._peek()
        if peeked is None:
            return None
        value = self.parse_node(peeked[0])
        if self._peek() is not None:
            raise YamlError(f"unparsed content at line {self.pos + 1}: {self.lines[self.pos]!r}")
        return value

    def parse_node(self, indent: int):
        peeked = self._peek()
        if peeked is None:
            return None
        _, content = peeked
        if content == "-" or content.startswith("- "):
            return self.parse_sequence(indent)
        return self.parse_mapping(indent)

    def parse_sequence(self, indent: int):
        items = []
        while True:
            peeked = self._peek()
            if peeked is None or peeked[0] != indent:
                break
            content = peeked[1]
            if not (content == "-" or content.startswith("- ")):
                break
            rest = content[1:].lstrip(" ")
            if rest == "":
                self.pos += 1
                nested = self._peek()
                items.append(self.parse_node(nested[0]) if nested and nested[0] > indent else None)
                continue
            offset = len(self.lines[self.pos]) - len(self.lines[self.pos].lstrip(" ")) + (len(content) - len(rest))
            if _KEY_RE.match(_strip_comment(rest)) and not rest.startswith(("'", '"', "[", "{")) or (
                rest[0] in "'\"" and _KEY_RE.match(_strip_comment(rest))
            ):
                # "- key: value" starts a mapping whose indent is the key's column.
                self.lines[self.pos] = " " * offset + rest
                items.append(self.parse_mapping(offset))
            else:
                self.pos += 1
                items.append(_scalar(_strip_comment(rest)))
        return items

    def parse_mapping(self, indent: int):
        mapping: dict = {}
        while True:
            peeked = self._peek()
            if peeked is None or peeked[0] != indent:
                break
            content = _strip_comment(peeked[1])
            if content == "-" or content.startswith("- "):
                break
            match = _KEY_RE.match(content)
            if not match:
                raise YamlError(f"line {self.pos + 1}: expected 'key: value', got {content!r}")
            key = _scalar(match.group("key"))
            if not isinstance(key, str):
                key = str(key)
            value = (match.group("value") or "").strip()
            if key in mapping:
                raise YamlError(f"line {self.pos + 1}: duplicate key {key!r}")
            self.pos += 1
            if value[:1] in ("|", ">"):
                mapping[key] = self._block_scalar(value, indent)
            elif value == "":
                nested = self._peek()
                if nested is None:
                    mapping[key] = None
                elif nested[0] > indent:
                    mapping[key] = self.parse_node(nested[0])
                elif nested[0] == indent and (nested[1] == "-" or nested[1].startswith("- ")):
                    mapping[key] = self.parse_sequence(indent)
                else:
                    mapping[key] = None
            else:
                mapping[key] = _scalar(value)
        return mapping


def load_yaml(text: str):
    return _Parser(text).parse_document()


def strip_yaml_comments(text: str) -> str:
    """Drop whole-line comments so prose in comments never trips text assertions."""
    return "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))


# --------------------------------------------------------------------------------------
# Helpers
# --------------------------------------------------------------------------------------


def trigger_block(document: dict) -> dict:
    """The ``on:`` mapping (PyYAML would call the key ``True``; we keep the string)."""
    key = "on" if "on" in document else True
    block = document[key]
    if isinstance(block, str):
        return {block: None}
    if isinstance(block, list):
        return {name: None for name in block}
    return block


def iter_steps(job: dict):
    return job.get("steps") or []


def checkout_steps(job: dict):
    return [s for s in iter_steps(job) if str(s.get("uses", "")).startswith("actions/checkout@")]


def run_scripts(document: dict):
    for name, job in document["jobs"].items():
        for step in iter_steps(job):
            if "run" in step:
                yield name, step["run"]


def all_checkouts(document: dict):
    for name, job in document["jobs"].items():
        for step in checkout_steps(job):
            yield name, step


def needs_of(job: dict) -> list:
    needs = job.get("needs", [])
    return [needs] if isinstance(needs, str) else list(needs)


# --------------------------------------------------------------------------------------
# Tests
# --------------------------------------------------------------------------------------


class YamlSubsetParserTests(unittest.TestCase):
    def test_parses_workflow_shapes(self) -> None:
        document = load_yaml(
            """
name: Demo
on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      flag:
        default: false
concurrency: { group: release-main, cancel-in-progress: false }
jobs:
  a:
    needs: [plan, other]
    if: needs.plan.outputs.x == 'true'
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: false # trailing comment
      - name: Run it
        run: |
          echo "a: b"
          # not a comment
          exit 0
      - run: echo hi
"""
        )
        self.assertEqual(document["on"]["push"]["branches"], ["main"])
        self.assertIs(document["on"]["workflow_dispatch"]["inputs"]["flag"]["default"], False)
        self.assertEqual(document["concurrency"], {"group": "release-main", "cancel-in-progress": False})
        job = document["jobs"]["a"]
        self.assertEqual(job["needs"], ["plan", "other"])
        self.assertEqual(job["if"], "needs.plan.outputs.x == 'true'")
        self.assertIs(job["steps"][0]["with"]["submodules"], False)
        self.assertEqual(job["steps"][1]["run"], 'echo "a: b"\n# not a comment\nexit 0\n')
        self.assertEqual(job["steps"][2], {"run": "echo hi"})

    def test_rejects_duplicate_keys_and_tabs(self) -> None:
        with self.assertRaises(YamlError):
            load_yaml("a: 1\na: 2\n")
        with self.assertRaises(YamlError):
            load_yaml("a:\n\tb: 1\n")


class WorkflowFixture(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.release_text = RELEASE.read_text(encoding="utf-8")
        cls.pr_text = PR_CHECKS.read_text(encoding="utf-8")
        cls.release = load_yaml(cls.release_text)
        cls.pr = load_yaml(cls.pr_text)
        try:  # optional cross-check; never required
            import yaml  # type: ignore
        except ImportError:
            cls.pyyaml = None
        else:
            cls.pyyaml = yaml


class WorkflowCrossCheckTests(WorkflowFixture):
    def test_matches_pyyaml_when_available(self) -> None:
        if self.pyyaml is None:
            self.skipTest("PyYAML not installed")
        for text, ours in ((self.release_text, self.release), (self.pr_text, self.pr)):
            theirs = self.pyyaml.safe_load(text)
            self.assertEqual(set(theirs["jobs"]), set(ours["jobs"]))
            for name, job in theirs["jobs"].items():
                self.assertEqual(job.get("needs"), ours["jobs"][name].get("needs"), name)
                self.assertEqual(job.get("if"), ours["jobs"][name].get("if"), name)
                self.assertEqual(len(job["steps"]), len(ours["jobs"][name]["steps"]), name)


class CommonWorkflowTests(WorkflowFixture):
    def test_workflow_files_exist_and_are_ascii_without_tabs(self) -> None:
        for path in (RELEASE, PR_CHECKS):
            data = path.read_bytes()
            self.assertNotIn(b"\t", data, path.name)
            self.assertTrue(data.endswith(b"\n"), path.name)

    def test_shiraz_is_never_checked_out(self) -> None:
        for label, document, text in (
            ("release", self.release, self.release_text),
            ("pr-checks", self.pr, self.pr_text),
        ):
            checkouts = list(all_checkouts(document))
            self.assertTrue(checkouts, label)
            for job, step in checkouts:
                self.assertIs(step.get("with", {}).get("submodules"), False, f"{label}/{job}")
            code = strip_yaml_comments(text)
            self.assertNotIn("submodules: recursive", code)
            self.assertNotIn("submodules: true", code)
            self.assertNotIn("git submodule", code)
            self.assertNotIn("shiraz", code.lower())

    def test_every_action_is_version_pinned(self) -> None:
        for document in (self.release, self.pr):
            for name, job in document["jobs"].items():
                for step in iter_steps(job):
                    if "uses" in step:
                        self.assertRegex(step["uses"], r"^[\w.-]+/[\w./-]+@[\w.-]+$", f"{name}: {step['uses']}")

    def test_untrusted_context_is_never_interpolated_into_shell(self) -> None:
        dangerous = re.compile(
            r"\$\{\{[^}]*(inputs\.|head_ref|github\.event\.(pull_request\.(title|body|head\.ref|head\.label)"
            r"|head_commit|commits|issue|comment|review))"
        )
        for document in (self.release, self.pr):
            for job, script in run_scripts(document):
                self.assertIsNone(dangerous.search(script), f"{job}: {script}")

    def test_no_pull_request_target_anywhere(self) -> None:
        for text in (self.release_text, self.pr_text):
            self.assertNotIn("pull_request_target", strip_yaml_comments(text))


class ReleaseWorkflowTests(WorkflowFixture):
    def test_triggers_only_push_main_and_dispatch(self) -> None:
        triggers = trigger_block(self.release)
        self.assertEqual(set(triggers), {"push", "workflow_dispatch"})
        self.assertEqual(triggers["push"], {"branches": ["main"]})
        inputs = triggers["workflow_dispatch"]["inputs"]
        self.assertEqual(set(inputs), {"force_components", "force_landing"})
        self.assertNotIn("pull_request", strip_yaml_comments(self.release_text))

    def test_top_level_permissions_are_read_only(self) -> None:
        self.assertEqual(self.release["permissions"], {"contents": "read"})

    def test_job_set_and_stage_ordering(self) -> None:
        jobs = self.release["jobs"]
        self.assertEqual(set(jobs), {"plan-and-version", *STAGE2_JOBS, "deploy-landing"})
        self.assertNotIn("needs", jobs["plan-and-version"])
        for name in STAGE2_JOBS:
            self.assertEqual(needs_of(jobs[name]), ["plan-and-version"], name)
            self.assertIn("needs.plan-and-version.outputs.", jobs[name]["if"], name)
        self.assertEqual(needs_of(jobs["deploy-landing"]), ["plan-and-version", "build-landing"])
        self.assertEqual(
            jobs["deploy-landing"]["if"], "needs.plan-and-version.outputs.deploy_landing == 'true'"
        )

    def test_plan_and_version_job(self) -> None:
        job = self.release["jobs"]["plan-and-version"]
        self.assertEqual(job["concurrency"], {"group": "release-main", "cancel-in-progress": False})
        self.assertEqual(job["permissions"], {"contents": "write"})
        self.assertIn("refs/heads/main", job["if"])
        (checkout,) = checkout_steps(job)
        self.assertEqual(checkout["with"]["fetch-depth"], "0")
        self.assertIs(checkout["with"]["submodules"], False)

        steps = iter_steps(job)
        names = [s.get("name", s.get("uses", "")) for s in steps]
        tip = next(s for s in steps if s.get("id") == "tip")
        tests = next(s for s in steps if "unittest discover" in s.get("run", ""))
        release = next(s for s in steps if s.get("id") == "release")
        self.assertLess(steps.index(tip), steps.index(tests), names)
        self.assertLess(steps.index(tests), steps.index(release), names)

        guard = tip["run"]
        self.assertIn("git fetch origin main --tags", guard)
        self.assertIn('[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]', guard)
        self.assertIn("git merge-base --is-ancestor HEAD origin/main", guard)
        self.assertIn('echo "skip=true"', guard)
        self.assertIn("exit 1", guard)
        self.assertEqual(tests["if"], "steps.tip.outputs.skip != 'true'")
        self.assertEqual(release["if"], "steps.tip.outputs.skip != 'true'")
        self.assertIn("python3 -m unittest discover -s scripts -p 'test_*.py' -v", tests["run"])

        script = release["run"]
        for fragment in (
            "python3 scripts/release.py",
            "--write",
            "--push",
            '--before "${{ github.event.before }}"',
            '--name "SpectreOS-CI"',
            '--email "ci@spectreos.dev"',
            '--github-output "$GITHUB_OUTPUT"',
            '--force-components "$FORCE_COMPONENTS"',
            "--force-landing",
        ):
            self.assertIn(fragment, script)
        # concurrent-push grace handling
        self.assertIn("git fetch origin main", script)
        self.assertIn('git merge-base --is-ancestor "$GITHUB_SHA" origin/main', script)
        self.assertIn("main advanced during release", script)
        self.assertEqual(release["env"]["FORCE_COMPONENTS"], "${{ inputs.force_components }}")

    def test_plan_outputs_cover_stage2_needs(self) -> None:
        outputs = self.release["jobs"]["plan-and-version"]["outputs"]
        referenced = set(re.findall(r"needs\.plan-and-version\.outputs\.(\w+)", self.release_text))
        self.assertTrue(referenced)
        self.assertLessEqual(referenced, set(outputs))
        for key in (
            "sha",
            "any_released",
            "released_components",
            "publish_libbhoo",
            "libbhoo_version",
            "libbhoo_tag",
            "has_release_apps",
            "release_apps_matrix",
            "release_calendar",
            "calendar_version",
            "calendar_tag",
            "release_spectref1",
            "spectref1_version",
            "spectref1_tag",
            "deploy_landing",
        ):
            self.assertIn(key, outputs)
            self.assertIn(f"steps.release.outputs.{key}", outputs[key])

    def test_only_the_planner_and_release_publishers_can_write_contents(self) -> None:
        writers = {
            name
            for name, job in self.release["jobs"].items()
            if job.get("permissions", {}).get("contents") == "write"
        }
        self.assertEqual(
            writers,
            {"plan-and-version", "publish-libbhoo", "release-apps", "release-calendar", "release-spectref1"},
        )
        for name in STAGE2_JOBS:
            self.assertIn("permissions", self.release["jobs"][name], name)

    def test_stage2_jobs_check_out_the_release_commit(self) -> None:
        for name in STAGE2_JOBS:
            (checkout,) = checkout_steps(self.release["jobs"][name])
            self.assertEqual(checkout["with"]["ref"], "${{ needs.plan-and-version.outputs.sha }}", name)
            self.assertIs(checkout["with"]["submodules"], False, name)

    def test_publish_libbhoo(self) -> None:
        job = self.release["jobs"]["publish-libbhoo"]
        self.assertEqual(job["if"], "needs.plan-and-version.outputs.publish_libbhoo == 'true'")
        self.assertEqual(job["permissions"], {"contents": "write"})
        steps = iter_steps(job)
        scripts = [s.get("run", "") for s in steps]
        index = {
            key: next(i for i, s in enumerate(scripts) if key in s)
            for key in ("cargo test --locked --lib", "cargo package --locked", "cargo publish --locked", "publish_release.py")
        }
        self.assertEqual(sorted(index.values()), list(index.values()), index)
        publish = steps[index["cargo publish --locked"]]
        self.assertEqual(publish["env"]["CARGO_REGISTRY_TOKEN"], "${{ secrets.CARGO_REGISTRY_TOKEN }}")
        self.assertIn("https://crates.io/api/v1/crates/libbhoo/", publish["run"])  # idempotency check
        self.assertIn("200)", publish["run"])
        self.assertIn("working-directory", publish)
        self.assertEqual(publish["working-directory"], "codes/libs/ui/libbhoo")
        release_script = scripts[index["publish_release.py"]]
        self.assertIn("--write", release_script)
        self.assertIn("--changelog codes/libs/ui/libbhoo/CHANGELOG.md", release_script)
        self.assertNotIn("--asset", release_script)

    def test_cargo_token_is_scoped_to_the_crates_publish_step(self) -> None:
        code = strip_yaml_comments(self.release_text)
        self.assertEqual(code.count("secrets.CARGO_REGISTRY_TOKEN"), 1)
        self.assertEqual(
            [m for m in re.findall(r"secrets\.(\w+)", code) if m != "CARGO_REGISTRY_TOKEN"], []
        )
        for name, job in self.release["jobs"].items():
            for script_job, script in run_scripts({"jobs": {name: job}}):
                if "cargo publish" in script:
                    self.assertEqual(script_job, "publish-libbhoo")

    def test_release_apps_matrix_builds_and_uploads_only_the_binary(self) -> None:
        job = self.release["jobs"]["release-apps"]
        self.assertEqual(job["if"], "needs.plan-and-version.outputs.has_release_apps == 'true'")
        self.assertEqual(job["permissions"], {"contents": "write"})
        strategy = job["strategy"]
        self.assertIs(strategy["fail-fast"], False)
        self.assertIn(
            "fromJSON(needs.plan-and-version.outputs.release_apps_matrix",
            strategy["matrix"]["include"],
        )
        scripts = "\n".join(s.get("run", "") for s in iter_steps(job))
        self.assertIn("cargo build --release --locked", scripts)
        self.assertIn('--features "$APP_FEATURES"', scripts)
        self.assertIn('--bin "$APP_BINARY"', scripts)
        self.assertIn("strip ", scripts)
        self.assertIn("dist/${APP_BINARY}-linux-x86_64", scripts)
        self.assertEqual(scripts.count("--asset"), 1)
        self.assertIn('--asset "dist/${APP_BINARY}-linux-x86_64"', scripts)
        self.assertIn('--tag "$APP_TAG"', scripts)
        self.assertNotIn("--clobber", scripts)
        self.assertNotIn("cargo publish", scripts)
        self.assertNotIn("--workspace", scripts)
        self.assertNotIn("cargo build --release --locked --all", scripts)
        install = next(s for s in iter_steps(job) if "apt-get install" in s.get("run", ""))
        self.assertIn("pkg-config", install["run"])
        self.assertIn("libsystemd-dev", install["run"])

    def test_calendar_and_spectref1_jobs(self) -> None:
        calendar = self.release["jobs"]["release-calendar"]
        self.assertEqual(calendar["if"], "needs.plan-and-version.outputs.release_calendar == 'true'")
        text = "\n".join(s.get("run", "") for s in iter_steps(calendar))
        self.assertIn("cargo test --locked --manifest-path codes/apps/calendar/Cargo.toml", text)
        self.assertIn("publish_release.py --write", text)
        self.assertIn("--changelog codes/apps/calendar/CHANGELOG.md", text)
        self.assertNotIn("--asset", text)

        spectref1 = self.release["jobs"]["release-spectref1"]
        self.assertEqual(spectref1["if"], "needs.plan-and-version.outputs.release_spectref1 == 'true'")
        text = "\n".join(s.get("run", "") for s in iter_steps(spectref1))
        self.assertIn("make -C codes/os/specrte-nix-f1 test", text)
        self.assertIn("publish_release.py --write", text)
        self.assertIn("--changelog codes/os/specrte-nix-f1/CHANGELOG.md", text)

    def test_deploy_landing(self) -> None:
        build_job = self.release["jobs"]["build-landing"]
        self.assertEqual(build_job["if"], "needs.plan-and-version.outputs.deploy_landing == 'true'")
        self.assertEqual(build_job["permissions"], {"contents": "read"})
        build_steps = iter_steps(build_job)
        build_uses = [s.get("uses", "") for s in build_steps]
        self.assertIn("actions/upload-pages-artifact@v3", build_uses)
        upload = next(s for s in build_steps if s.get("uses") == "actions/upload-pages-artifact@v3")
        self.assertEqual(upload["with"]["path"], "landing/dist")
        scripts = [s.get("run", "") for s in build_steps]
        test_i = next(i for i, s in enumerate(scripts) if s == "npm test")
        build_i = next(i for i, s in enumerate(scripts) if s == "npm run build")
        self.assertLess(test_i, build_i)  # tests rebuild dist; the production build must be last
        self.assertLess(build_i, build_steps.index(upload))

        deploy_job = self.release["jobs"]["deploy-landing"]
        self.assertEqual(deploy_job["if"], "needs.plan-and-version.outputs.deploy_landing == 'true'")
        self.assertEqual(deploy_job["permissions"], {"pages": "write", "id-token": "write"})
        self.assertEqual(deploy_job["concurrency"], {"group": "pages", "cancel-in-progress": True})
        self.assertEqual(
            deploy_job["environment"],
            {"name": "github-pages", "url": "${{ steps.deployment.outputs.page_url }}"},
        )
        deploy_steps = iter_steps(deploy_job)
        deploy = next(s for s in deploy_steps if s.get("uses") == "actions/deploy-pages@v4")
        self.assertEqual(deploy["id"], "deployment")

    def test_release_never_reads_or_writes_pr_state(self) -> None:
        code = strip_yaml_comments(self.release_text)
        self.assertNotIn("github.event.pull_request", code)
        self.assertNotIn("detect-pr", code)


class PullRequestWorkflowTests(WorkflowFixture):
    FORBIDDEN = (
        "--write",
        "--push",
        "cargo publish",
        "publish_release.py",
        "gh release",
        "deploy-pages",
        "upload-pages-artifact",
        "git push",
        "git tag",
        "git commit",
        "secrets.",
        "CARGO_REGISTRY_TOKEN",
    )

    def test_triggers_only_pull_request(self) -> None:
        triggers = trigger_block(self.pr)
        self.assertEqual(set(triggers), {"pull_request"})
        code = strip_yaml_comments(self.pr_text)
        self.assertNotRegex(code, r"(?m)^\s*push\s*:")
        self.assertNotIn("workflow_dispatch", code)

    def test_read_only_permissions_everywhere(self) -> None:
        self.assertEqual(self.pr["permissions"], {"contents": "read"})
        for name, job in self.pr["jobs"].items():
            if "permissions" in job:
                self.assertEqual(job["permissions"], {"contents": "read"}, name)
        code = strip_yaml_comments(self.pr_text)
        for forbidden in ("contents: write", "pages: write", "id-token: write", "packages: write"):
            self.assertNotIn(forbidden, code)

    def test_never_releases_versions_or_deploys(self) -> None:
        code = strip_yaml_comments(self.pr_text)
        for forbidden in self.FORBIDDEN:
            self.assertNotIn(forbidden, code, forbidden)
        for _, script in run_scripts(self.pr):
            for line in script.splitlines():
                if "release.py" in line:
                    self.assertRegex(line, r"release\.py (--check|detect-pr)\b")

    def test_concurrency_cancels_superseded_runs(self) -> None:
        self.assertEqual(
            self.pr["concurrency"],
            {"group": "pr-checks-${{ github.event.pull_request.number }}", "cancel-in-progress": True},
        )

    def test_detect_changes_job(self) -> None:
        job = self.pr["jobs"]["detect-changes"]
        script = "\n".join(s.get("run", "") for s in iter_steps(job))
        self.assertIn("python3 scripts/release.py detect-pr", script)
        self.assertIn('--base "${{ github.event.pull_request.base.sha }}"', script)
        self.assertIn('--head "${{ github.event.pull_request.head.sha }}"', script)
        self.assertIn('--github-output "$GITHUB_OUTPUT"', script)
        (checkout,) = checkout_steps(job)
        self.assertEqual(checkout["with"]["fetch-depth"], "0")
        for key in (
            "check_scripts",
            "check_libbhoo",
            "has_check_apps",
            "check_apps_matrix",
            "check_spectref1",
            "check_landing",
        ):
            self.assertIn(key, job["outputs"])

    def test_checks_are_gated_by_detect_changes(self) -> None:
        jobs = self.pr["jobs"]
        gated = {
            "release-tooling": "check_scripts",
            "libbhoo": "check_libbhoo",
            "apps": "has_check_apps",
            "spectref1": "check_spectref1",
            "landing": "check_landing",
        }
        for name, key in gated.items():
            self.assertIn("detect-changes", needs_of(jobs[name]), name)
            self.assertEqual(jobs[name]["if"], f"needs.detect-changes.outputs.{key} == 'true'", name)
        self.assertIn(
            "fromJSON(needs.detect-changes.outputs.check_apps_matrix",
            jobs["apps"]["strategy"]["matrix"]["include"],
        )
        self.assertIs(jobs["apps"]["strategy"]["fail-fast"], False)

    def test_check_commands(self) -> None:
        jobs = self.pr["jobs"]

        def scripts(name: str) -> str:
            return "\n".join(s.get("run", "") for s in iter_steps(jobs[name]))

        tooling = scripts("release-tooling")
        self.assertIn("python3 -m unittest discover -s scripts -p 'test_*.py' -v", tooling)
        self.assertIn("python3 scripts/release.py --check", tooling)
        libbhoo = scripts("libbhoo")
        for command in (
            "cargo check --locked --lib",
            "cargo check --locked --lib --all-features",
            "cargo test --locked --lib",
            "cargo package --locked --no-verify --list",
        ):
            self.assertIn(command, libbhoo)
        self.assertIn("cargo clippy", libbhoo)
        self.assertIn("make -C codes/os/specrte-nix-f1 test", scripts("spectref1"))
        landing = scripts("landing")
        self.assertIn("npm test", landing)
        self.assertIn("npm run build", landing)

    def test_aggregate_gate_fails_on_any_failed_check(self) -> None:
        gate = self.pr["jobs"]["checks-ok"]
        self.assertEqual(gate["if"], "always()")
        self.assertEqual(set(needs_of(gate)), set(self.pr["jobs"]) - {"checks-ok"})
        script = "\n".join(s.get("run", "") for s in iter_steps(gate))
        self.assertIn("failure", script)
        self.assertIn("cancelled", script)


if __name__ == "__main__":
    unittest.main()
