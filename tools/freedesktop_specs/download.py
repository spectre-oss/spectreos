#!/usr/bin/env python3
"""Download Freedesktop.org specifications into docs/freedesktop/ (Python stdlib only)."""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import posixpath
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, field
from html.parser import HTMLParser
from pathlib import Path

from html2md import html_to_markdown

BASE_URL = "https://specifications.freedesktop.org/"
USER_AGENT = "spectreos-freedesktop-spec-downloader/1.0 (+https://specifications.freedesktop.org/)"
DEFAULT_REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_OUTPUT_DIR = DEFAULT_REPO_ROOT / "docs" / "freedesktop"


@dataclass
class SpecEntry:
    name: str
    links: list[tuple[str, str]] = field(default_factory=list)  # (href, text)


class IndexParser(HTMLParser):
    """Parse https://specifications.freedesktop.org/ into SpecEntry items."""

    def __init__(self) -> None:
        super().__init__()
        self.specs: list[SpecEntry] = []
        self._current_spec: SpecEntry | None = None
        self._in_h4 = False
        self._h4_buf: list[str] = []
        self._in_a = False
        self._a_href: str | None = None
        self._a_buf: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        if tag == "h4":
            self._in_h4 = True
            self._h4_buf.clear()
        elif tag == "a" and self._current_spec is not None:
            attr_map = dict(attrs)
            href = attr_map.get("href")
            if href:
                self._in_a = True
                self._a_href = href.strip()
                self._a_buf.clear()

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag == "h4" and self._in_h4:
            self._in_h4 = False
            name = "".join(self._h4_buf).strip()
            if name:
                self._current_spec = SpecEntry(name=name)
                self.specs.append(self._current_spec)
        elif tag == "a" and self._in_a:
            self._in_a = False
            if self._current_spec is not None and self._a_href:
                label = " ".join("".join(self._a_buf).split())
                self._current_spec.links.append((self._a_href, label))
            self._a_href = None

    def handle_data(self, data: str) -> None:
        if self._in_h4:
            self._h4_buf.append(data)
        elif self._in_a:
            self._a_buf.append(data)


class LinkExtractor(HTMLParser):
    """Extract href links from an HTML page for multi-page spec crawling."""

    def __init__(self) -> None:
        super().__init__()
        self.hrefs: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag.lower() == "a":
            href = dict(attrs).get("href")
            if href:
                self.hrefs.append(href.strip())


def fetch_url(url: str, timeout: float, retries: int = 3) -> tuple[bytes, str, str]:
    """Fetch URL and return (content_bytes, final_url, content_type)."""
    last_err: Exception | None = None
    for attempt in range(1, retries + 1):
        req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                data = resp.read()
                final_url = resp.geturl()
                ctype = resp.headers.get("Content-Type", "")
                return data, final_url, ctype
        except (urllib.error.URLError, TimeoutError, OSError) as exc:
            last_err = exc
            if attempt < retries:
                time.sleep(0.6 * attempt)
    raise RuntimeError(f"Failed to fetch {url}: {last_err}") from last_err


def sanitize_name(name: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "-", name.strip()).strip("-") or "spec"


def select_spec_urls(spec: SpecEntry, all_versions: bool) -> list[tuple[str, str]]:
    """Select target (url, label) pairs to download for a specification."""
    resolved = [(urllib.parse.urljoin(BASE_URL, href), label) for href, label in spec.links]
    if all_versions:
        return resolved

    single_links = [
        (u, lbl)
        for u, lbl in resolved
        if "latest-single" in u or "one page" in lbl.lower()
    ]
    latest_links = [
        (u, lbl)
        for u, lbl in resolved
        if ("/latest" in u or "latest" in lbl.lower()) and (u, lbl) not in single_links
    ]
    extra_files = [
        (u, lbl)
        for u, lbl in resolved
        if re.search(r"\.(dtd|xml|txt|pdf|rng|xsd|json|md)$", urllib.parse.urlparse(u).path, re.I)
    ]

    chosen: list[tuple[str, str]] = []
    if single_links:
        chosen.append(single_links[0])
    elif latest_links:
        chosen.append(latest_links[0])
    else:
        # Fallback for specs hosted on external docs (e.g., appstream, dbus)
        for u, lbl in resolved:
            if (u, lbl) not in extra_files:
                chosen.append((u, lbl))
                break

    for item in extra_files:
        if item not in chosen:
            chosen.append(item)
    return chosen


def filename_from_url(url: str, content_type: str) -> str:
    parsed = urllib.parse.urlparse(url)
    path = parsed.path
    if not path or path.endswith("/"):
        return "index.html"
    base = posixpath.basename(path)
    if "." not in base and "html" in content_type.lower():
        return f"{base}.html"
    return base or "index.html"


def download_spec(
    spec: SpecEntry,
    output_dir: Path,
    *,
    all_versions: bool,
    timeout: float,
    force: bool,
    dry_run: bool,
    keep_html: bool,
) -> dict:
    spec_dir = output_dir / sanitize_name(spec.name)
    targets = select_spec_urls(spec, all_versions=all_versions)
    saved_files: list[str] = []
    errors: list[str] = []

    if dry_run:
        return {
            "name": spec.name,
            "targets": [u for u, _ in targets],
            "files": [],
            "errors": [],
        }

    spec_dir.mkdir(parents=True, exist_ok=True)
    visited: set[str] = set()

    def save_response(dest_base: Path, raw_bytes: bytes, source_url: str, is_html: bool) -> None:
        if is_html:
            md_dest = dest_base.with_suffix(".md")
            if force or not md_dest.exists():
                md_text = html_to_markdown(raw_bytes.decode("utf-8", "replace"), base_url=source_url)
                md_dest.parent.mkdir(parents=True, exist_ok=True)
                md_dest.write_text(md_text, encoding="utf-8")
            saved_files.append(str(md_dest.relative_to(output_dir)))
            if keep_html:
                if force or not dest_base.exists():
                    dest_base.parent.mkdir(parents=True, exist_ok=True)
                    dest_base.write_bytes(raw_bytes)
                saved_files.append(str(dest_base.relative_to(output_dir)))
        else:
            if force or not dest_base.exists():
                dest_base.parent.mkdir(parents=True, exist_ok=True)
                dest_base.write_bytes(raw_bytes)
            saved_files.append(str(dest_base.relative_to(output_dir)))

    for url, _label in targets:
        if url in visited:
            continue
        visited.add(url)
        try:
            data, final_url, ctype = fetch_url(url, timeout=timeout)
        except Exception as exc:
            errors.append(f"{url}: {exc}")
            continue

        fname = filename_from_url(final_url, ctype)
        is_html = "html" in ctype.lower() or fname.lower().endswith((".html", ".htm"))
        save_response(spec_dir / fname, data, final_url, is_html=is_html)

        # If this is a multi-page HTML spec hosted on specifications.freedesktop.org
        # (and not a single-page dump), also fetch its same-directory sibling .html pages.
        parsed_final = urllib.parse.urlparse(final_url)
        if (
            is_html
            and parsed_final.netloc == "specifications.freedesktop.org"
            and "single" not in parsed_final.path
        ):
            base_dir = posixpath.dirname(parsed_final.path)
            if not base_dir.endswith("/"):
                base_dir += "/"
            extractor = LinkExtractor()
            try:
                extractor.feed(data.decode("utf-8", "replace"))
            except Exception:
                pass
            for rel_href in extractor.hrefs:
                if rel_href.startswith("#") or rel_href.startswith("mailto:"):
                    continue
                sub_url = urllib.parse.urljoin(final_url, rel_href).split("#", 1)[0]
                if sub_url in visited:
                    continue
                sub_parsed = urllib.parse.urlparse(sub_url)
                if (
                    sub_parsed.netloc == parsed_final.netloc
                    and sub_parsed.path.startswith(base_dir)
                    and re.search(r"\.(html?|xml|dtd)$", sub_parsed.path, re.I)
                ):
                    visited.add(sub_url)
                    sub_rel = sub_parsed.path[len(base_dir) :].lstrip("/")
                    if not sub_rel or ".." in sub_rel.split("/"):
                        continue
                    try:
                        sub_data, sub_final, sub_ctype = fetch_url(sub_url, timeout=timeout)
                        sub_is_html = "html" in sub_ctype.lower() or sub_rel.lower().endswith(
                            (".html", ".htm")
                        )
                        save_response(spec_dir / sub_rel, sub_data, sub_final, is_html=sub_is_html)
                    except Exception as exc:
                        errors.append(f"{sub_url}: {exc}")

    return {
        "name": spec.name,
        "targets": [u for u, _ in targets],
        "files": sorted(set(saved_files)),
        "errors": errors,
    }


def write_index(output_dir: Path, results: list[dict]) -> None:
    manifest_path = output_dir / "manifest.json"
    manifest_path.write_text(
        json.dumps(
            {
                "source": BASE_URL,
                "count": len(results),
                "specifications": results,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    lines = [
        "# Freedesktop.org Specifications",
        "",
        f"Downloaded from {BASE_URL} using `tools/freedesktop_specs/download.py`.",
        "",
        "| Specification | Files | Source |",
        "| --- | --- | --- |",
    ]
    for item in sorted(results, key=lambda r: r["name"].lower()):
        name = item["name"]
        files = item["files"]
        first_file = f"[`{files[0]}`]({files[0]})" if files else "*(none)*"
        extra = f" (+{len(files) - 1} more)" if len(files) > 1 else ""
        src = item["targets"][0] if item["targets"] else BASE_URL
        lines.append(f"| `{name}` | {first_file}{extra} | {src} |")
    lines.append("")
    (output_dir / "INDEX.md").write_text("\n".join(lines), encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Download all Freedesktop.org specifications into docs/freedesktop/."
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT_DIR,
        help=f"Destination directory (default: {DEFAULT_OUTPUT_DIR})",
    )
    parser.add_argument(
        "--all-versions",
        action="store_true",
        help="Download all historical versions instead of latest only.",
    )
    parser.add_argument(
        "--filter",
        default="",
        help="Only download specs whose name contains this substring (case-insensitive).",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=6,
        help="Number of parallel download workers (default: 6).",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=20.0,
        help="Per-request HTTP timeout in seconds (default: 20).",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing downloaded files.",
    )
    parser.add_argument(
        "--keep-html",
        action="store_true",
        help="Also save the raw .html files alongside the converted .md files.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="List specifications and target URLs without downloading.",
    )
    args = parser.parse_args(argv)

    print(f"Fetching index from {BASE_URL} ...")
    index_bytes, _, _ = fetch_url(BASE_URL, timeout=args.timeout)
    index_parser = IndexParser()
    index_parser.feed(index_bytes.decode("utf-8", "replace"))
    specs = index_parser.specs

    if args.filter:
        needle = args.filter.lower()
        specs = [s for s in specs if needle in s.name.lower()]

    if not specs:
        print("No specifications matched.", file=sys.stderr)
        return 1

    print(f"Found {len(specs)} specifications -> {args.output}")
    if not args.dry_run:
        args.output.mkdir(parents=True, exist_ok=True)
        if args.keep_html:
            (args.output / "index.html").write_bytes(index_bytes)

    results: list[dict] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.workers)) as pool:
        futures = {
            pool.submit(
                download_spec,
                spec,
                args.output,
                all_versions=args.all_versions,
                timeout=args.timeout,
                force=args.force,
                dry_run=args.dry_run,
                keep_html=args.keep_html,
            ): spec
            for spec in specs
        }
        for fut in concurrent.futures.as_completed(futures):
            res = fut.result()
            results.append(res)
            status = "OK" if not res["errors"] else f"WARN ({len(res['errors'])} errors)"
            if args.dry_run:
                print(f"  [DRY] {res['name']}: {', '.join(res['targets'])}")
            else:
                print(f"  [{status}] {res['name']}: {len(res['files'])} file(s)")

    if not args.dry_run:
        write_index(args.output, results)
        total_files = sum(len(r["files"]) for r in results)
        total_errors = sum(len(r["errors"]) for r in results)
        print(
            f"Done: {len(results)} specs, {total_files} files saved to {args.output} "
            f"({total_errors} errors). Index: {args.output / 'INDEX.md'}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
