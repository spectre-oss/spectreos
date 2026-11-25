#!/usr/bin/env python3
"""Publish one component's GitHub Release. Never signs, never replaces assets.

Dry-run by default; `--write` creates the release (or idempotently adds missing
assets) with `gh`.
"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import components as registry  # noqa: E402

SHA = re.compile(r"[0-9a-f]{40}\Z")
TAG = re.compile(r"([a-z0-9]+)-v((?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*))\Z")
VERSION_HEADING = re.compile(r"^#{1,2} \[([0-9]+\.[0-9]+\.[0-9]+)\].*$", re.MULTILINE)
BLOCKED_EVENTS = {"pull_request", "pull_request_target"}
REPO = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\Z")


class PublishError(Exception):
    """A publish failure safe to print in CI."""


def changelog_section(text, version):
    match = VERSION_HEADING.search(text)
    while match:
        if match.group(1) == version:
            rest = text[match.end():]
            nxt = VERSION_HEADING.search(rest)
            body = rest[: nxt.start()] if nxt else rest
            return (text[match.start(): match.end()] + body).strip() + "\n"
        match = VERSION_HEADING.search(text, match.end())
    return ""


def build_notes(component, version, sha, changelog_text, *, allow_missing=False):
    section = changelog_section(changelog_text, version)
    if not section.strip():
        if not allow_missing:
            raise PublishError("changelog has no section for this version")
        section = f"# {component.name} {version}\n"
    return f"Source revision: {sha}\nComponent: {component.name} {version}\n\n{section}"


def validate(tag, sha, version=None, component=None):
    match = TAG.fullmatch(tag)
    if not match or not SHA.fullmatch(sha):
        raise PublishError("invalid release tag or commit")
    name, number = match.groups()
    if name not in registry.BY_NAME:
        raise PublishError("tag does not name a versioned component")
    if component and component != name:
        raise PublishError("tag does not match component")
    if version and version != number:
        raise PublishError("tag does not match numeric version")
    return registry.BY_NAME[name], number


def assert_local_tag(tag, sha):
    result = subprocess.run(["git", "rev-parse", f"{tag}^{{commit}}"], text=True,
                            capture_output=True, check=False)
    if result.returncode or result.stdout.strip() != sha:
        raise PublishError("tag does not point at the expected commit")


def validate_assets(component, paths):
    names = []
    for path in paths:
        if path.is_symlink() or not path.is_file():
            raise PublishError("asset is missing or not a regular file")
        names.append(path.name)
        # Binary apps ship only their binary (and its sidecars), never sources/build trees.
        if component.kind == "app" and not path.name.startswith(component.binary):
            raise PublishError(f"asset {path.name} is not a {component.binary} binary")
    if len(names) != len(set(names)):
        raise PublishError("duplicate asset names")
    return names


class CommandGh:
    """GitHub CLI adapter. Asset uploads never overwrite existing files."""

    def __init__(self, repo=None):
        self.repo = ["-R", repo] if repo else []

    def _run(self, args):
        result = subprocess.run(["gh", *args, *self.repo], text=True, capture_output=True, check=False)
        if result.returncode:
            if args[:2] == ["release", "view"] and "release not found" in result.stderr.lower():
                return None
            raise PublishError(f"gh {args[0]} {args[1]} failed (exit {result.returncode})")
        return result.stdout

    def view(self, tag):
        stdout = self._run(["release", "view", tag, "--json", "tagName,isDraft,isPrerelease,targetCommitish,assets"])
        return None if stdout is None else json.loads(stdout)

    def create(self, tag, sha, title, notes, files, latest):
        fd, name = tempfile.mkstemp(prefix="release-notes-", suffix=".md")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(notes)
            self._run(["release", "create", tag, "--verify-tag", "--target", sha,
                       "--title", title, "--notes-file", name,
                       f"--latest={'true' if latest else 'false'}",
                       *[str(path) for path in files]])
        finally:
            os.unlink(name)

    def upload(self, tag, files):
        self._run(["release", "upload", tag, *[str(path) for path in files]])

    def download(self, tag, name):
        directory = Path(tempfile.mkdtemp(prefix="release-asset-"))
        self._run(["release", "download", tag, "--pattern", name, "--dir", str(directory)])
        path = directory / name
        if path.is_symlink() or not path.is_file():
            raise PublishError("downloaded asset is missing")
        return path


def publish(tag, sha, title, notes, files, gh, *, write, latest=False):
    names = [path.name for path in files]
    if not write:
        return {"created": False, "uploaded": names, "dry_run": True}
    existing = gh.view(tag)
    if existing is None:
        gh.create(tag, sha, title, notes, files, latest)
        return {"created": True, "uploaded": names, "dry_run": False}
    if existing.get("isDraft") or existing.get("isPrerelease"):
        raise PublishError("refusing to change a draft or prerelease")
    if existing.get("targetCommitish") != sha:
        raise PublishError("existing release points at a different commit")
    have = {asset.get("name") for asset in existing.get("assets", [])}
    for path in files:
        if path.name not in have:
            continue
        downloaded = gh.download(tag, path.name)
        try:
            if downloaded.read_bytes() != path.read_bytes():
                raise PublishError(f"asset {path.name} conflicts with the published file")
        finally:
            downloaded.unlink(missing_ok=True)
    missing = [path for path in files if path.name not in have]
    if missing:
        gh.upload(tag, missing)
    return {"created": False, "uploaded": [path.name for path in missing], "dry_run": False}


def main(argv=None, gh=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--component")
    parser.add_argument("--version")
    parser.add_argument("--changelog", type=Path, required=True)
    parser.add_argument("--title")
    parser.add_argument("--asset", type=Path, action="append", default=[],
                        help="file to upload (repeatable); binary apps upload binaries only")
    parser.add_argument("--allow-missing-section", action="store_true",
                        help="use a clean fallback header when the changelog has no section")
    parser.add_argument("--latest", action="store_true", help="mark as the repository's latest release")
    parser.add_argument("--repo", help="owner/name passed to gh -R")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true", help="default: validate without publishing")
    mode.add_argument("--write", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.write and os.environ.get("GITHUB_EVENT_NAME", "") in BLOCKED_EVENTS:
            raise PublishError("publication cannot run for a pull request")
        if args.repo and not REPO.fullmatch(args.repo):
            raise PublishError("invalid repository")
        component, version = validate(args.tag, args.sha, args.version, args.component)
        assert_local_tag(args.tag, args.sha)
        names = validate_assets(component, args.asset)
        notes = build_notes(component, version, args.sha, args.changelog.read_text(encoding="utf-8"),
                            allow_missing=args.allow_missing_section)
        title = args.title or f"{component.name} {version}"
        result = publish(args.tag, args.sha, title, notes, args.asset,
                         (gh or CommandGh(args.repo)) if args.write else None,
                         write=args.write, latest=args.latest)
        mode_text = "dry-run" if result["dry_run"] else ("created" if result["created"] else "reused")
        print(f"{mode_text} {args.tag}; {len(result['uploaded']) if not result['dry_run'] else len(names)} asset(s)")
        return 0
    except (PublishError, OSError, ValueError) as exc:
        message = str(exc) if isinstance(exc, PublishError) else type(exc).__name__
        print(f"publish: {message}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
