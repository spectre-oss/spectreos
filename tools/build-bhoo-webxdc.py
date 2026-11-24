#!/usr/bin/env python3
"""Package the built, offline Bhoo/Angular comparison as a webxdc."""
import argparse
import json
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "codes/apps/bhoo-web/site"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true", help="Reuse the built site")
    parser.add_argument("--output", type=Path, default=ROOT / "context/dist/bhoo-demo.xdc")
    parser.add_argument("--plain", action="store_true", help="Show only the selected Bhoo canvas, without comparison panels")
    parser.add_argument("--component", help="Restrict the packaged UI to this component slug")
    parser.add_argument("--description", default="", help="Describe the fix in the package")
    parser.add_argument("--explorer", action="store_true", help="Open the full component explorer by default")
    parser.add_argument("--gallery", action="store_true", help="Package the single-page Bhoo-only gallery")
    parser.add_argument("--android-demo", action="store_true", help="Package the separate fictional Android XML chat demo")
    parser.add_argument("--gmail-demo", action="store_true", help="Package the Gmail UI Bhoo mockup")
    parser.add_argument("--contacts-demo", action="store_true", help="Package the Android Contacts Bhoo demo")
    parser.add_argument("--launcher-demo", action="store_true", help="Package the Android System UI & Launcher Bhoo demo")
    parser.add_argument("--canvas-gallery", action="store_true", help="Package the one-canvas Rust-scrolled gallery")
    args = parser.parse_args()
    if sum(bool(x) for x in (args.explorer, args.gallery, args.canvas_gallery, args.android_demo, args.gmail_demo, args.contacts_demo, args.launcher_demo, args.component)) > 1:
        parser.error("--explorer, --gallery and --component are mutually exclusive")
    if args.plain and not args.component:
        parser.error("--plain requires --component")
    if not args.no_build:
        subprocess.run(["python3", str(ROOT / "tools/build-bhoo-web.py")], check=True)
    required = ["playground.html", "bhoo.wasm", "catalog.json", "source.zip", "licenses/GPL-3.0.txt"]
    if not (args.gallery or args.canvas_gallery or args.android_demo or args.gmail_demo or args.contacts_demo or args.launcher_demo or args.plain):
        required += ["index.html", "angular.html", "angular/reference.js", "angular/theme.css", "angular/THIRD_PARTY_LICENSES.txt"]
    for name in required:
        if not (SITE / name).is_file():
            raise SystemExit(f"Missing {name}; build the complete web demo first")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    files = sorted(p for p in SITE.rglob("*") if p.is_file() and p.suffix != ".xdc")
    if args.plain:
        allowed={"playground.html","app.js","live-status.js","dialog-accessibility.js","carousel-accessibility.js","time-picker-accessibility.js","style.css","mark.svg","icon.png","bhoo.wasm","catalog.json","source.zip","licenses/GPL-3.0.txt","licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt","licenses/NOTICES.md","beercss/LICENSE.txt"}
        if args.component == "android-video-view": allowed.update({"video-bridge.js","bhoo-test-video.webm"})
        files=[p for p in files if p.relative_to(SITE).as_posix() in allowed]
    if args.gallery:
        gallery_files = {"gallery.html", "gallery.css", "gallery.js", "playground.html", "app.js", "video-bridge.js", "bhoo-test-video.webm", "live-status.js", "dialog-accessibility.js", "carousel-accessibility.js", "time-picker-accessibility.js", "style.css", "mark.svg","icon.png", "bhoo.wasm", "catalog.json", "source.zip", "licenses/GPL-3.0.txt", "licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt", "licenses/NOTICES.md", "beercss/LICENSE.txt"}
        files = [p for p in files if p.relative_to(SITE).as_posix() in gallery_files]
        missing = gallery_files - {p.relative_to(SITE).as_posix() for p in files}
        if missing:
            raise SystemExit(f"Missing gallery files: {sorted(missing)}")
    if args.canvas_gallery:
        allowed={"canvas.html","single-canvas.js","canvas-runtime.js","live-status.js","dialog-accessibility.js","carousel-accessibility.js", "time-picker-accessibility.js","canvas-worker.js","canvas-painter.js","showcase-data.js","mark.svg","icon.png","bhoo.wasm","catalog.json","source.zip","licenses/GPL-3.0.txt","licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt","licenses/NOTICES.md","beercss/LICENSE.txt"}
        files=[p for p in files if p.relative_to(SITE).as_posix() in allowed]
        missing=allowed-{p.relative_to(SITE).as_posix() for p in files}
        if missing:raise SystemExit(f"Missing canvas gallery files: {sorted(missing)}")
    if args.android_demo:
        allowed={"chat.html","chat.js","canvas-painter.js","mark.svg","icon.png","bhoo.wasm","source.zip","licenses/GPL-3.0.txt","licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt","licenses/NOTICES.md"}
        files=[p for p in files if p.relative_to(SITE).as_posix() in allowed]
        missing=allowed-{p.relative_to(SITE).as_posix() for p in files}
        if missing:raise SystemExit(f"Missing Android demo files: {sorted(missing)}")
    if args.gmail_demo:
        allowed={"gmail.html","gmail.js","canvas-painter.js","mark.svg","icon.png","bhoo.wasm","source.zip","licenses/GPL-3.0.txt","licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt","licenses/NOTICES.md"}
        files=[p for p in files if p.relative_to(SITE).as_posix() in allowed]
        missing=allowed-{p.relative_to(SITE).as_posix() for p in files}
        if missing:raise SystemExit(f"Missing Gmail demo files: {sorted(missing)}")
    if args.contacts_demo:
        allowed={"contacts.html","contacts.js","canvas-painter.js","mark.svg","icon.png","bhoo.wasm","source.zip","licenses/GPL-3.0.txt","licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt","licenses/NOTICES.md"}
        files=[p for p in files if p.relative_to(SITE).as_posix() in allowed]
        missing=allowed-{p.relative_to(SITE).as_posix() for p in files}
        if missing:raise SystemExit(f"Missing Contacts demo files: {sorted(missing)}")
    if args.launcher_demo:
        allowed={"launcher.html","launcher.js","canvas-painter.js","mark.svg","icon.png","bhoo.wasm","source.zip","licenses/GPL-3.0.txt","licenses/Apache-2.0.txt","licenses/Unicode-Segmentation-MIT.txt","licenses/Unicode-Width-MIT.txt","licenses/Unicode-License.txt","licenses/Unicode-DFS-2016.txt","licenses/NOTICES.md"}
        files=[p for p in files if p.relative_to(SITE).as_posix() in allowed]
        missing=allowed-{p.relative_to(SITE).as_posix() for p in files}
        if missing:raise SystemExit(f"Missing Launcher demo files: {sorted(missing)}")
    single_catalog = None
    if args.component:
        catalog = json.loads((SITE / "catalog.json").read_text())
        matches = [component for component in catalog["components"] if component["slug"] == args.component]
        if len(matches) != 1:
            raise SystemExit(f"Unknown component: {args.component}")
        component = matches[0]
        catalog["components"] = matches
        catalog["overview"] = component
        catalog["live_count"] = 1
        catalog["single_component"] = args.component
        catalog["update_description"] = args.description
        single_catalog = (json.dumps(catalog, separators=(",", ":")) + "\n").encode()
    # Fixed archive metadata makes identical built inputs produce identical packages.
    with zipfile.ZipFile(args.output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        title = f"Bhoo · {component['name']}" if args.component else "Bhoo · All examples" if args.gallery else "Bhoo · One canvas" if args.canvas_gallery else "Bhoo · Component explorer" if args.explorer else "Bhoo · Angular comparison"
        if args.android_demo: title="Delta Chat UI · Bhoo demo"
        if args.gmail_demo: title="Gmail UI · Bhoo mockup"
        if args.contacts_demo: title="Contacts · Bhoo demo"
        if args.launcher_demo: title="Android Launcher · Bhoo demo"
        manifest = f"name = {json.dumps(title, ensure_ascii=False)}\n"
        if args.description:
            manifest += f"description = {json.dumps(args.description, ensure_ascii=False)}\n"
        entries = [("manifest.toml", manifest.encode())]
        if args.plain:
            entries.append(("index.html", (SITE / "playground.html").read_bytes().replace(b'<html lang="en" data-theme="light">', b'<html lang="en" data-theme="light" data-embed="true">', 1)))
        if args.android_demo:
            entries.append(("index.html", (SITE / "chat.html").read_bytes()))
        if args.gmail_demo:
            entries.append(("index.html", (SITE / "gmail.html").read_bytes()))
        if args.contacts_demo:
            entries.append(("index.html", (SITE / "contacts.html").read_bytes()))
        if args.launcher_demo:
            entries.append(("index.html", (SITE / "launcher.html").read_bytes()))
        if args.gallery:
            entries.append(("index.html", (SITE / "gallery.html").read_bytes()))
        if args.canvas_gallery:
            entries.append(("index.html", (SITE / "canvas.html").read_bytes()))
        for p in files:
            name = p.relative_to(SITE).as_posix()
            content = single_catalog if single_catalog is not None and name == "catalog.json" else p.read_bytes()
            if args.gallery and name == "playground.html":
                content = content.replace(b'<a href="https://m3.material.io/"', b'<a href="index.html">All examples</a><a href="https://m3.material.io/"', 1)
            if args.explorer and name == "index.html":
                entries.append(("comparison.html", content))
                content = (SITE / "playground.html").read_bytes().replace(
                    b'<a href="https://m3.material.io/"',
                    b'<a href="comparison.html">Compare components</a><a href="https://m3.material.io/"',
                    1,
                ).replace(b'<script type="module" src="app.js"></script>',
                          b'<script>if (!location.hash) location.hash = "#/overview";</script>\n<script type="module" src="app.js"></script>', 1)
            entries.append((name, content))
        for name, content in entries:
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    with zipfile.ZipFile(args.output) as archive:
        assert archive.testzip() is None
        assert "index.html" in archive.namelist()
        assert "webxdc.js" not in archive.namelist(), "The messenger supplies webxdc.js"
    print(f"Created {args.output} ({args.output.stat().st_size:,} bytes; {len(entries)} entries)")


if __name__ == "__main__":
    main()
