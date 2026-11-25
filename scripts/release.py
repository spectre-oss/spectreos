#!/usr/bin/env python3
"""Per-component Conventional Commit releases for the SpectreOS monorepo.

Python 3.11+, Git and Cargo only.
`codes/shiraz-core/**` is never scanned, versioned or edited.

  release.py [--dry-run | --write | --check] [--push] ...
  release.py detect-pr --base SHA --head SHA --github-output PATH   (read-only)
"""

import argparse
import contextlib
import dataclasses
import datetime
import fcntl
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import components as registry  # noqa: E402
from components import COMPONENTS, REPOSITORY, VERSION  # noqa: E402

HEADER = re.compile(r"^(\w+)(?:\(([^\r\n)]+)\))?(!)?: (.+)$")
BREAKING = re.compile(r"^BREAKING[ -]CHANGE:\s*(.*)$", re.MULTILINE)
SHA_ARG = re.compile(r"[0-9a-fA-F]{7,64}\Z")
RELEASE_SUBJECT = re.compile(r"^chore\(release\)(?:!)?:")
GITHUB_EVENT_BLOCKED = {"pull_request", "pull_request_target"}


class ReleaseError(Exception):
    """An expected validation or release failure safe to display in CI."""


# --------------------------------------------------------------------------- process


def run(root, args, *, env=None, input_text=None):
    """Never execute commit text, versions or credentials through a shell."""
    child_env = os.environ.copy()
    # An inherited alternate index/worktree must not redirect release writes.
    for key in ("GIT_INDEX_FILE", "GIT_WORK_TREE", "GIT_DIR"):
        child_env.pop(key, None)
    child_env.update(env or {})
    result = subprocess.run(
        args, cwd=root, env=child_env, input=input_text, text=True,
        capture_output=True, check=False,
    )
    if result.returncode:
        # Git transport diagnostics can contain credential-bearing remote URLs.
        raise ReleaseError(f"{args[0]} {args[1] if len(args) > 1 else ''} failed (exit {result.returncode})")
    return result.stdout


def git(root, *args, env=None, input_text=None):
    return run(root, ["git", "-c", "core.hooksPath=/dev/null", *args], env=env, input_text=input_text)


def git_ok(root, *args):
    try:
        git(root, *args)
    except ReleaseError:
        return False
    return True


# --------------------------------------------------------------------------- versions


def version_tuple(value):
    if not VERSION.fullmatch(value):
        raise ReleaseError("Expected a stable MAJOR.MINOR.PATCH version")
    return tuple(map(int, value.split(".")))


def bump(version, level, *, allow_major=False):
    major, minor, patch = version_tuple(version)
    if level == 3 and allow_major:
        return f"{major + 1}.0.0"
    if level >= 2:
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


@dataclasses.dataclass(frozen=True)
class Commit:
    sha: str
    message: str


@dataclasses.dataclass(frozen=True)
class Change:
    sha: str
    kind: str
    scope: str
    description: str
    breaking: str
    level: int
    references: tuple = ()


def changes_from_commits(commits):
    """Classify changes; breaking severity is capped to minor unless opted in."""
    cancelled = set()
    # Match real Git revert trailers; do not suppress an arbitrary same subject.
    for commit in reversed(commits):
        if commit.sha in cancelled:
            continue
        match = re.search(r"^This reverts commit ([0-9a-f]{7,40})\.$", commit.message, re.MULTILINE)
        if match:
            original = next((c for c in commits if c.sha.startswith(match[1])), None)
            if original:
                cancelled.update((original.sha, commit.sha))
    changes = []
    for commit in commits:
        if commit.sha in cancelled:
            continue
        subject, _, body = commit.message.partition("\n")
        if RELEASE_SUBJECT.match(subject):
            continue  # generated release notes are never a source of releases
        match = HEADER.match(subject)
        revert = subject.startswith('Revert "') and "This reverts commit " in body
        breaking_match = BREAKING.search(body)
        if not match and not revert and not breaking_match:
            continue
        kind, scope, bang, description = (
            match.groups() if match else ("revert" if revert else "", "", "", subject)
        )
        breaking = breaking_match[1].strip() if breaking_match else ""
        if breaking_match:
            continuation = []
            for line in body[breaking_match.end():].splitlines():
                if not line.strip() and continuation:
                    break
                if re.match(r"^[A-Za-z][A-Za-z-]*(?::| #\d+)", line):
                    break
                if line.strip():
                    continuation.append(line.strip())
            breaking = " ".join([breaking, *continuation]).strip()
        if breaking_match and not breaking:
            breaking = description
        if bang and not breaking:
            breaking = description
        level = 3 if breaking else {"feat": 2, "fix": 1, "perf": 1, "revert": 1}.get(kind, 0)
        if level:
            changes.append(Change(
                commit.sha, kind, scope or "", description, breaking, level,
                tuple(dict.fromkeys(re.findall(r"#([1-9]\d*)", commit.message))),
            ))
    return changes


# --------------------------------------------------------------------------- notes


def markdown_text(value):
    # Release notes are Markdown, never executable HTML or workflow output.
    value = re.sub(r"[\x00-\x1f\x7f]", " ", value)
    return re.sub(r"([\\`*_{}\[\]<>])", r"\\\1", value)


def release_notes(component, version, base_tag, changes, date, *, forced=False):
    tag = component.tag(version)
    compare = (f"{REPOSITORY}/compare/{base_tag}...{tag}" if base_tag
               else f"{REPOSITORY}/releases/tag/{tag}")
    notes = [f"## [{version}]({compare}) ({date})", ""]
    groups = (("feat", "Features"), ("fix", "Bug Fixes"),
              ("perf", "Performance Improvements"), ("revert", "Reverts"))
    for kind, title in groups:
        entries = [c for c in changes if c.kind == kind]
        if not entries:
            continue
        notes.extend((f"### {title}", ""))
        for change in sorted(entries, key=lambda c: (c.scope, c.description, c.sha)):
            scope = f"**{markdown_text(change.scope)}:** " if change.scope else ""
            references = ", ".join(
                f"[#{number}]({REPOSITORY}/issues/{number})" for number in change.references)
            suffix = f", {references}" if references else ""
            notes.append(f"* {scope}{markdown_text(change.description)} "
                         f"([{change.sha[:7]}]({REPOSITORY}/commit/{change.sha})){suffix}")
        notes.append("")
    other = [c for c in changes if c.kind not in {g[0] for g in groups}]
    if other:
        notes.extend(("### Other Changes", ""))
        for change in other:
            notes.append(f"* {markdown_text(change.description)} "
                         f"([{change.sha[:7]}]({REPOSITORY}/commit/{change.sha}))")
        notes.append("")
    if not changes and forced:
        notes.extend(("### Maintenance", "", f"* Manual release of {component.name}.", ""))
    breaking = [c for c in changes if c.breaking]
    if breaking:
        notes.extend(("### BREAKING CHANGES", ""))
        notes.extend(f"* {markdown_text(c.breaking)}" for c in breaking)
        notes.append("")
    return "\n".join(notes).rstrip() + "\n"


def prepend_changelog(text, version, notes):
    # Never append a second entry after an interrupted/manual preparation.
    if re.search(rf"^#{{1,2}} \[?{re.escape(version)}(?:\]|\s|$)", text, re.MULTILINE):
        raise ReleaseError("Changelog already contains the planned release")
    first = re.search(r"(?m)^#{1,2} \[", text)
    if first:
        return text[: first.start()] + notes + "\n" + text[first.start():]
    return text.rstrip("\n") + "\n\n" + notes


# --------------------------------------------------------------------------- files


def replace_toml_version(text, section, version):
    pattern = re.compile(rf"(^\[{re.escape(section)}\]\s*\n)(.*?)(?=^\[|\Z)", re.MULTILINE | re.DOTALL)
    matches = list(pattern.finditer(text))
    if len(matches) != 1:
        raise ReleaseError(f"Expected one [{section}] section")
    match = matches[0]
    body, count = re.subn(r'(?m)^version\s*=\s*"[^"\n]+"', f'version = "{version}"', match[2])
    if count != 1:
        raise ReleaseError(f"Expected one version in [{section}]")
    return text[: match.start(2)] + body + text[match.end(2):]


def update_lock_text(text, versions):
    """Set the version of source-less (path/workspace) packages named in `versions`."""
    def update(match):
        block = match[0]
        parsed = tomllib.loads(block)["package"][0]
        if parsed["name"] in versions and "source" not in parsed:
            return re.sub(r'(?m)^version = "[^"\n]+"', f'version = "{versions[parsed["name"]]}"', block, count=1)
        return block
    return re.sub(r"(?ms)^\[\[package\]\].*?(?=^\[\[package\]\]|\Z)", update, text)


def read_file(root, relative):
    path = root / relative
    # Do not follow symlinks into credentials, another worktree or system files.
    if path.is_symlink() or root not in path.resolve().parents:
        raise ReleaseError(f"Unsafe release path: {relative}")
    if not path.is_file():
        raise ReleaseError(f"Missing release file: {relative}")
    if not git(root, "ls-files", "--", relative).strip():
        raise ReleaseError(f"Release file is not tracked: {relative}")
    return path.read_text(encoding="utf-8")


def atomic_write(path, text):
    mode = path.stat().st_mode & 0o777
    fd, temporary = tempfile.mkstemp(prefix=".release-", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(text)
            handle.flush()
            os.fsync(handle.fileno())
        os.chmod(temporary, mode)
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def tracked_lockfiles(root):
    names = git(root, "ls-files", "-z", "--", "*Cargo.lock").split("\0")
    return sorted(
        n for n in names
        if n and n.rsplit("/", 1)[-1] == "Cargo.lock" and not registry.excluded(n)
        and "target" not in n.split("/")
    )


def cargo_package_versions(root, manifest):
    """`cargo metadata --no-deps --locked`: validates the manifest and lockfile."""
    metadata = json.loads(run(root, [
        "cargo", "metadata", "--no-deps", "--locked", "--format-version", "1",
        "--manifest-path", str(root / manifest),
    ]))
    ids = set(metadata["workspace_members"])
    return {p["name"]: p["version"] for p in metadata["packages"] if p["id"] in ids}


def verify_versions(root, *, cargo=True):
    """Return ({component: version}, {path: text}); raise when anything disagrees."""
    data, current = {}, {}
    for component in COMPONENTS:
        version = read_file(root, component.version_file).strip()
        version_tuple(version)
        data[component.version_file] = read_file(root, component.version_file)
        data[component.changelog] = read_file(root, component.changelog)
        current[component.name] = version
        if not component.cargo:
            continue
        manifest = read_file(root, component.manifest)
        data[component.manifest] = manifest
        if tomllib.loads(manifest)["package"]["name"] != component.package:
            raise ReleaseError(f"{component.manifest} does not declare package {component.package}")
        if tomllib.loads(manifest)["package"]["version"] != version:
            raise ReleaseError(f"{component.name}: version file and Cargo.toml disagree")
    expected = {c.package: current[c.name] for c in COMPONENTS if c.cargo}
    for lock in tracked_lockfiles(root):
        data[lock] = read_file(root, lock)
        packages = tomllib.loads(data[lock]).get("package", [])
        for package in packages:
            if "source" not in package and expected.get(package["name"], package["version"]) != package["version"]:
                raise ReleaseError(f"{lock}: {package['name']} lock version disagrees with its component")
    for component in COMPONENTS:
        if not component.cargo:
            continue
        own = [p for p in tomllib.loads(data[component.lockfile])["package"]
               if p["name"] == component.package and "source" not in p]
        if len(own) != 1:
            raise ReleaseError(f"{component.lockfile}: missing or ambiguous {component.package}")
    if cargo:
        for component in COMPONENTS:
            if component.cargo and cargo_package_versions(root, component.manifest).get(
                    component.package) != current[component.name]:
                raise ReleaseError(f"{component.name}: cargo metadata version disagrees")
    return current, data


@contextlib.contextmanager
def release_lock(root):
    directory = Path(git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").strip())
    path = directory / "spectreos-release.lock"
    with path.open("a") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise ReleaseError("Another release is running in this repository") from exc
        yield


# --------------------------------------------------------------------------- planning


@dataclasses.dataclass
class Plan:
    component: object
    base_tag: str
    version: str
    level: int
    changes: list
    forced: bool = False
    notes: str = ""

    @property
    def tag(self):
        return self.component.tag(self.version)


def latest_tag(root, component, revision="HEAD"):
    tags = git(root, "tag", "--merged", revision, "--list", f"{component.name}-v*").splitlines()
    found = [(version_tuple(m[1]), t) for t in tags if (m := component.tag_pattern.fullmatch(t))]
    return max(found)[1] if found else ""


def component_paths(component):
    return [component.directory,
            *(f":(exclude){p.rstrip('/')}" for p in registry.EXCLUDED_PREFIXES)]


def commits_since(root, tag, paths):
    span = f"{tag}..HEAD" if tag else "HEAD"
    rows = git(root, "log", "--reverse", "--no-merges", "--format=%H%x00%B%x00", span, "--", *paths).split("\0")
    return [Commit(rows[i].strip(), rows[i + 1].strip())
            for i in range(0, len(rows) - 1, 2) if rows[i].strip()]


def plan_component(root, component, current, *, allow_major, forced):
    tag = latest_tag(root, component)
    changes = changes_from_commits(commits_since(root, tag, component_paths(component)))
    level = max((c.level for c in changes), default=0)
    if not level and not forced:
        return None
    level = max(level, 1)
    base = component.tag_pattern.fullmatch(tag)[1] if tag else current
    version = bump(base, level, allow_major=allow_major)
    if version_tuple(version) < version_tuple(current):
        raise ReleaseError(f"{component.name}: planned version would downgrade the manifests")
    if git(root, "tag", "--list", component.tag(version)).strip():
        raise ReleaseError(f"{component.name}: planned release tag already exists")
    return Plan(component, tag, version, level, changes, forced=forced and not changes)


def planned_files(root, data, plans):
    updates = {}
    versions = {p.component.package: p.version for p in plans if p.component.cargo}
    for plan in plans:
        c = plan.component
        updates[c.version_file] = plan.version + "\n"
        updates[c.changelog] = prepend_changelog(data[c.changelog], plan.version, plan.notes)
        if c.cargo:
            updates[c.manifest] = replace_toml_version(data[c.manifest], "package", plan.version)
    if versions:
        for lock in tracked_lockfiles(root):
            text = data.get(lock) or read_file(root, lock)
            updated = update_lock_text(text, versions)
            if updated != text:
                updates[lock] = updated
    return updates


def identity_env(name, email):
    if not name or re.search(r"[\x00-\x1f\x7f<>]", name):
        raise ReleaseError("Invalid release identity name")
    if re.search(r"[\x00-\x1f\x7f]", email) or not re.fullmatch(r"[^\s<>@]+@[^\s<>@]+", email):
        raise ReleaseError("Invalid release identity email")
    return {"GIT_AUTHOR_NAME": name, "GIT_AUTHOR_EMAIL": email,
            "GIT_COMMITTER_NAME": name, "GIT_COMMITTER_EMAIL": email}


def release_subject(entries):
    return "chore(release): " + ", ".join(f"{n} {v}" for n, v in entries) + " [skip ci]"


def push_release(root, tags, head):
    # Atomic and non-forced: neither branch nor tags change on a concurrent push.
    git(root, "push", "--atomic", "origin", f"{head}:refs/heads/main",
        *[f"refs/tags/{t}:refs/tags/{t}" for t in tags])


def prepare(root, updates, originals, plans, head, identity, sign):
    index = Path(git(root, "rev-parse", "--path-format=absolute", "--git-path", "index").strip())
    index_before = index.read_bytes() if index.exists() else None
    committed, created = None, []
    try:
        for name, text in updates.items():
            atomic_write(root / name, text)
        verify_versions(root)
        # Every touched lockfile's manifest must still resolve with --locked.
        for name in updates:
            if name.endswith("Cargo.lock") and (root / Path(name).parent / "Cargo.toml").is_file():
                cargo_package_versions(root, (Path(name).parent / "Cargo.toml").as_posix())
        if git(root, "rev-parse", "HEAD").strip() != head:
            raise ReleaseError("HEAD changed during release preparation")
        git(root, "add", "--", *updates)
        subject = release_subject([(p.component.name, p.version) for p in plans])
        message = subject + "\n\n" + "\n".join(f"### {p.component.name}\n\n{p.notes}" for p in plans)
        tree = git(root, "write-tree").strip()
        commit_args = ["-c", f"commit.gpgsign={'true' if sign else 'false'}", "commit-tree",
                       tree, "-p", head, "-F", "-"]
        if sign:
            commit_args.append("-S")
        candidate = git(root, *commit_args, env=identity, input_text=message).strip()
        # Compare-and-swap the branch: an intervening writer is never overwritten.
        git(root, "update-ref", "refs/heads/main", candidate, head)
        committed = candidate
        for plan in plans:
            git(root, "-c", f"tag.gpgsign={'true' if sign else 'false'}", "tag",
                "-s" if sign else "-a", plan.tag, "-m", f"{plan.component.name} {plan.version}",
                candidate, env=identity)
            created.append(plan.tag)
    except BaseException:
        # Only tags created by this run are removed; competing tags are untouched.
        for tag in created:
            git_ok(root, "tag", "-d", tag)
        if committed:
            git_ok(root, "update-ref", "refs/heads/main", head, committed)
        for name, text in originals.items():
            atomic_write(root / name, text)
        if index_before is not None:
            index.write_bytes(index_before)
        elif index.exists():
            index.unlink()
        raise
    return committed


def recover_prepared(root, head, current):
    """HEAD already is a prepared release commit whose push may have failed."""
    tags = git(root, "tag", "--points-at", head).splitlines()
    found = []
    for component in COMPONENTS:
        for tag in tags:
            match = component.tag_pattern.fullmatch(tag)
            if match:
                found.append((component, match[1], tag))
    if not found:
        return []
    subject = git(root, "log", "-1", "--format=%s", head).strip()
    expected = release_subject([(c.name, v) for c, v, _ in found])
    if subject != expected or any(current[c.name] != v for c, v, _ in found):
        raise ReleaseError("HEAD is not a prepared release")
    return found


# --------------------------------------------------------------------------- landing / outputs


def check_sha(value, what):
    if not SHA_ARG.fullmatch(value):
        raise ReleaseError(f"Invalid {what} revision")
    return value


def diff_paths(root, *revisions):
    out = git(root, "diff", "--name-only", "--no-renames", "-z", *revisions, "--")
    return [p for p in out.split("\0") if p and not registry.excluded(p)]


def landing_changed(root, before, force):
    if force:
        return True
    if not before:
        return False
    check_sha(before, "before")
    if set(before) == {"0"}:
        return True  # new branch / first push: deploy
    if not git_ok(root, "cat-file", "-e", f"{before}^{{commit}}"):
        return True  # force-push or unknown history: deploy rather than skip
    return any(registry.is_landing_path(p) for p in diff_paths(root, before, "HEAD"))


def parse_forced(value):
    names = [n.strip() for n in (value or "").split(",") if n.strip()]
    unknown = [n for n in names if n not in registry.BY_NAME]
    if unknown:
        raise ReleaseError(f"Unknown component for --force-components: {unknown[0][:40]}")
    return set(names)


def release(root, *, write=False, push=False, sign=False, name="SpectreOS-CI",
            email="ci@spectreos.invalid", check=False, allow_major=False,
            before="", force_landing=False, force_components=""):
    identity = identity_env(name, email)
    if push and not write:
        raise ReleaseError("--push requires --write")
    forced = parse_forced(force_components)
    if write and os.environ.get("GITHUB_EVENT_NAME", "") in GITHUB_EVENT_BLOCKED:
        raise ReleaseError("release writes cannot run for a pull request")
    if not check and git(root, "rev-parse", "--is-shallow-repository").strip() != "false":
        raise ReleaseError("Fetch full history and tags before releasing")
    if write and git(root, "status", "--porcelain", "--untracked-files=normal").strip():
        raise ReleaseError("Release requires a clean worktree and index")
    with release_lock(root):
        if write and git(root, "status", "--porcelain", "--untracked-files=normal").strip():
            raise ReleaseError("Release requires a clean worktree and index")
        if write and git(root, "branch", "--show-current").strip() != "main":
            raise ReleaseError("Release writes are allowed only on main")
        head = git(root, "rev-parse", "HEAD").strip()
        current, data = verify_versions(root)
        landing = landing_changed(root, before, force_landing)
        base = {"sha": head, "released": [], "check": check, "dry_run": not write,
                "versions": current, "deploy_landing": landing, "files": [], "notes": {}}
        if check:
            base["deploy_landing"] = False
            return base
        date = datetime.datetime.now(datetime.UTC).date().isoformat()
        plans = []
        for component in COMPONENTS:
            plan = plan_component(root, component, current[component.name],
                                  allow_major=allow_major, forced=component.name in forced)
            if plan:
                plan.notes = release_notes(component, plan.version, plan.base_tag,
                                           plan.changes, date, forced=plan.forced)
                plans.append(plan)
        if not plans:
            # Recover an atomic push failure without creating another commit/tag.
            recovered = recover_prepared(root, head, current) if push else []
            if recovered:
                push_release(root, [t for _, _, t in recovered], head)
                base["released"] = [{"name": c.name, "version": v, "tag": t, "recovered": True}
                                    for c, v, t in recovered]
                base["deploy_landing"] = True
            return base
        updates = planned_files(root, data, plans)
        base["released"] = [{"name": p.component.name, "version": p.version, "tag": p.tag,
                             "level": p.level} for p in plans]
        base["files"] = sorted(updates)
        base["notes"] = {p.component.name: p.notes for p in plans}
        base["deploy_landing"] = True  # landing renders component versions
        if write:
            originals = {n: (data[n] if n in data else read_file(root, n)) for n in updates}
            base["sha"] = prepare(root, updates, originals, plans, head, identity, sign)
            if push:
                push_release(root, [p.tag for p in plans], base["sha"])
        return base


def flag(value):
    return "true" if value else "false"


def compact(value):
    return json.dumps(value, separators=(",", ":"), ensure_ascii=True)


def release_outputs(result):
    by_name = {r["name"]: r for r in result["released"]}

    def entry(name):
        return by_name.get(name)

    def triple(name, key):
        e = entry(name)
        return {f"{key}_version": e["version"] if e else "", f"{key}_tag": e["tag"] if e else ""}

    out = {"sha": result["sha"], "any_released": flag(by_name),
           "released_components": compact([c.name for c in COMPONENTS if c.name in by_name]),
           "publish_libbhoo": flag(entry("libbhoo"))}
    out.update(triple("libbhoo", "libbhoo"))
    apps = []
    for c in COMPONENTS:
        if c.kind == "app" and c.name in by_name:
            e = by_name[c.name]
            apps.append({"name": c.name, "package": c.package, "binary": c.binary,
                         "dir": c.directory, "features": c.features, "version": e["version"],
                         "tag": e["tag"], "changelog": c.changelog})
    out["has_release_apps"] = flag(apps)
    out["release_apps_matrix"] = compact(apps)
    out["release_calendar"] = flag(entry("calendar"))
    out.update(triple("calendar", "calendar"))
    out["release_spectref1"] = flag(entry("spectref1"))
    out.update(triple("spectref1", "spectref1"))
    out["deploy_landing"] = flag(result["deploy_landing"])
    return out


def write_outputs(path, values):
    lines = []
    for key, value in values.items():
        if not re.fullmatch(r"[a-z0-9_]+", key) or re.search(r"[\r\n]", str(value)):
            raise ReleaseError("Unsafe GitHub output")
        lines.append(f"{key}={value}\n")
    with path.open("a", encoding="utf-8") as handle:
        handle.write("".join(lines))


# --------------------------------------------------------------------------- detect-pr


def detect_pr(root, base, head):
    check_sha(base, "base")
    check_sha(head, "head")
    try:
        paths = diff_paths(root, f"{base}...{head}")
    except ReleaseError:
        paths = diff_paths(root, base, head)
    under = registry.under
    root_changed = any(
        p.startswith(registry.ROOT_PREFIXES) or p in registry.ROOT_FILES for p in paths)
    libbhoo = any(under(p, registry.LIBBHOO_DIR) for p in paths) or root_changed
    names = set()
    for app, (directory, _) in registry.CHECK_APPS.items():
        if libbhoo or any(under(p, directory) for p in paths):
            names.add(app)
    for app in list(names):
        names.update(registry.CHECK_DEPENDENTS.get(app, ()))
    apps = [{"name": n, "dir": d, "manifest": f"{d}/Cargo.toml", "package": pkg}
            for n, (d, pkg) in registry.CHECK_APPS.items() if n in names]
    return {
        "check_scripts": flag(any(p.startswith(("scripts/", ".github/")) for p in paths)),
        "check_libbhoo": flag(libbhoo),
        "has_check_apps": flag(apps),
        "check_apps_matrix": compact(apps),
        "check_spectref1": flag(any(under(p, registry.SPECTREF1_DIR) for p in paths) or
                                any(p.startswith(".github/") for p in paths)),
        "check_landing": flag(any(registry.is_landing_path(p) for p in paths)),
    }


# --------------------------------------------------------------------------- CLI


def resolve_root(repo):
    return Path(git(repo, "rev-parse", "--show-toplevel").strip()).resolve()


def detect_main(argv):
    parser = argparse.ArgumentParser(prog="release.py detect-pr",
                                     description="Read-only PR change detection.")
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args(argv)
    try:
        outputs = detect_pr(resolve_root(args.repo), args.base, args.head)
        print(json.dumps(outputs, indent=2))
        if args.github_output:
            write_outputs(args.github_output, outputs)
        return 0
    except (ReleaseError, OSError, ValueError, KeyError) as exc:
        message = str(exc) if isinstance(exc, ReleaseError) else type(exc).__name__
        print(f"release: {message}", file=sys.stderr)
        return 1


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv and argv[0] == "detect-pr":
        return detect_main(argv[1:])
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true", help="default: report without release writes")
    mode.add_argument("--write", action="store_true", help="update, validate, commit and tag on main")
    mode.add_argument("--check", action="store_true", help="check manifest/lockfile consistency")
    parser.add_argument("--push", action="store_true", help="atomically push prepared branch and tags")
    parser.add_argument("--sign", action="store_true", help="sign release commit and annotated tags")
    parser.add_argument("--allow-major", action="store_true",
                        help="explicitly allow breaking changes to increase the major version")
    parser.add_argument("--before", default="", help="github.event.before for landing detection")
    parser.add_argument("--force-landing", action="store_true", help="always deploy the landing page")
    parser.add_argument("--force-components", default="",
                        help="comma-separated components to patch-release even without releasable commits")
    parser.add_argument("--name", default="SpectreOS-CI")
    parser.add_argument("--email", default="ci@spectreos.invalid")
    parser.add_argument("--github-output", type=Path, help="append release job outputs")
    args = parser.parse_args(argv)
    try:
        result = release(
            resolve_root(args.repo), write=args.write, push=args.push, sign=args.sign,
            name=args.name, email=args.email, check=args.check, allow_major=args.allow_major,
            before=args.before, force_landing=args.force_landing,
            force_components=args.force_components)
        print(json.dumps(result, indent=2))
        if args.github_output:
            write_outputs(args.github_output, release_outputs(result))
        return 0
    except (ReleaseError, OSError, ValueError, KeyError) as exc:
        # Do not include arbitrary subprocess stderr, environment values or URLs.
        message = str(exc) if isinstance(exc, ReleaseError) else type(exc).__name__
        print(f"release: {message}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
