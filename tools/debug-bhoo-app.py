#!/usr/bin/env python3
"""Debug-only WebAssembly build & live workbench server for libbhoo apps.

Compiles selected SpectreOS libbhoo apps (files, settings, calendar, keyboard, notifications, or all)
to wasm32-unknown-unknown using the standalone tools/bhoo-web-debug crate — completely
separate from production phone builds.

Examples:
    python3 tools/debug-bhoo-app.py files
    python3 tools/debug-bhoo-app.py files --watch
    python3 tools/debug-bhoo-app.py all --port 8768
    python3 tools/debug-bhoo-app.py files --build-only
"""

from __future__ import annotations

import argparse
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
DEBUG_CRATE = ROOT / "tools" / "bhoo-web-debug"
WEB_DIR = DEBUG_CRATE / "web"
PAINTER_JS = ROOT / "codes" / "apps" / "bhoo-web" / "site" / "canvas-painter.js"
ALL_APPS = ("files", "settings", "calendar", "keyboard", "notifications")


def parse_apps(raw: str) -> list[str]:
    if raw.strip().lower() == "all":
        return list(ALL_APPS)
    selected = []
    for part in raw.split(","):
        name = part.strip().lower()
        if not name:
            continue
        if name not in ALL_APPS:
            raise SystemExit(
                f"Unknown app '{name}'. Supported: {', '.join(ALL_APPS)}, or 'all'"
            )
        if name not in selected:
            selected.append(name)
    if not selected:
        raise SystemExit("Specify at least one app (e.g. files, settings, calendar, keyboard, all)")
    return selected


def default_target_dir() -> Path:
    env = os.environ.get("BHOO_DEBUG_TARGET_DIR") or os.environ.get("CARGO_TARGET_DIR")
    if env:
        return Path(env)
    return Path("/tmp/bhoo-web-debug-target")


def wasm_output_path(target_dir: Path, release: bool) -> Path:
    profile = "release" if release else "debug"
    return target_dir / "wasm32-unknown-unknown" / profile / "bhoo_web_debug.wasm"


def build_wasm(apps: list[str], target_dir: Path, release: bool, offline: bool) -> Path:
    targets = subprocess.check_output(["rustup", "target", "list", "--installed"], text=True)
    if "wasm32-unknown-unknown" not in targets:
        raise SystemExit(
            "Missing wasm32 target. Install with: rustup target add wasm32-unknown-unknown"
        )
    cmd = [
        "cargo",
        "build",
        "--manifest-path",
        str(DEBUG_CRATE / "Cargo.toml"),
        "--target",
        "wasm32-unknown-unknown",
        "--target-dir",
        str(target_dir),
        "--no-default-features",
        "--features",
        ",".join(apps),
    ]
    if release:
        cmd.append("--release")
    if offline:
        cmd.append("--offline")
    t0 = time.monotonic()
    subprocess.run(cmd, cwd=ROOT, check=True)
    wasm = wasm_output_path(target_dir, release)
    dt = time.monotonic() - t0
    print(
        f"[bhoo-web-debug] built ({','.join(apps)}) -> {wasm} "
        f"({wasm.stat().st_size / 1024:.1f} KB in {dt:.2f}s)",
        flush=True,
    )
    return wasm


def snapshot_mtimes(apps: list[str]) -> dict[Path, float]:
    roots = [
        DEBUG_CRATE / "src",
        ROOT / "codes" / "libs" / "ui" / "libbhoo" / "src",
    ]
    for app in apps:
        folder = "wisp" if app == "notifications" else app
        roots.append(ROOT / "codes" / "apps" / folder / "src")
    mtimes: dict[Path, float] = {}
    for folder in roots:
        if not folder.is_dir():
            continue
        for p in folder.rglob("*.rs"):
            try:
                mtimes[p] = p.stat().st_mtime
            except OSError:
                pass
    return mtimes


def load_app_web_tests(apps: list[str]) -> list[dict]:
    tests: list[dict] = []
    seen: set[str] = set()
    for app in apps:
        folder = "wisp" if app == "notifications" else app
        path = ROOT / "codes" / "apps" / folder / "test" / "web-tests.json"
        if not path.is_file():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(data, list):
            raise SystemExit(f"{path}: expected a JSON array of test cases")
        for idx, item in enumerate(data):
            if not isinstance(item, dict) or not item.get("name") or not isinstance(item.get("steps"), list):
                raise SystemExit(f"{path}[{idx}]: invalid test case (requires 'name' and 'steps' list)")
            tc = dict(item)
            tc.setdefault("id", f"{app}-test-{idx}")
            tc.setdefault("app", app)
            tc.setdefault("pinned", True)
            tc["source"] = str(path.relative_to(ROOT))
            if tc["id"] not in seen:
                seen.add(tc["id"])
                tests.append(tc)
    return tests


def parse_desktop_entry(path: Path) -> dict[str, str]:
    in_entry = False
    kv: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("["):
            in_entry = line == "[Desktop Entry]"
            continue
        if in_entry and "=" in line:
            k, v = line.split("=", 1)
            kv.setdefault(k.strip(), v.strip())
    return kv


def load_app_catalog(apps: list[str]) -> list[dict]:
    all_tests = load_app_web_tests(apps)
    test_counts: dict[str, int] = {}
    for tc in all_tests:
        a = tc.get("app", "")
        test_counts[a] = test_counts.get(a, 0) + 1

    catalog: list[dict] = []
    for app in apps:
        folder = "wisp" if app == "notifications" else app
        assets_dir = ROOT / "codes" / "apps" / folder / "assets"
        desktop_files = sorted(assets_dir.glob("*.desktop")) if assets_dir.is_dir() else []
        meta = parse_desktop_entry(desktop_files[0]) if desktop_files else {}
        name = meta.get("Name") or app.capitalize()
        generic = meta.get("GenericName", "")
        comment = meta.get("Comment", "")
        if app == "notifications":
            name = "Notifications"
            generic = "Wisp System UI & Alerts"
            comment = "Test Wisp heads-up banners, notification shade, lock screen alerts, and Freedesktop D-Bus actions"
        desktop_id = meta.get("StartupWMClass") or (desktop_files[0].stem if desktop_files else f"org.spectreos.{app.capitalize()}")
        categories = [c for c in meta.get("Categories", "").split(";") if c]
        keywords = [k for k in meta.get("Keywords", "").split(";") if k]
        catalog.append(
            {
                "slug": app,
                "folder": folder,
                "name": name,
                "genericName": generic,
                "comment": comment,
                "desktopId": desktop_id,
                "categories": categories,
                "keywords": keywords,
                "testCount": test_counts.get(app, 0),
                "desktopPath": str(desktop_files[0].relative_to(ROOT)) if desktop_files else None,
            }
        )
    return catalog


def validate_app_desktop_entries(apps: list[str]) -> list[str]:
    found: list[str] = []
    for app in apps:
        folder = "wisp" if app == "notifications" else app
        assets_dir = ROOT / "codes" / "apps" / folder / "assets"
        entries = sorted(assets_dir.glob("*.desktop")) if assets_dir.is_dir() else []
        if not entries:
            raise SystemExit(f"codes/apps/{folder}/assets/: missing Freedesktop .desktop file")
        for entry in entries:
            text = entry.read_text(encoding="utf-8")
            if "[Desktop Entry]" not in text:
                raise SystemExit(f"{entry}: missing [Desktop Entry] header")
            found.append(str(entry.relative_to(ROOT)))
    return found


def run_headless_check(apps: list[str], wasm_path: Path) -> None:
    tests = load_app_web_tests(apps)
    desktop_entries = validate_app_desktop_entries(apps)
    script = r"""
import fs from 'node:fs';
import { paintPacket } from './codes/apps/bhoo-web/site/canvas-painter.js';

const wasmPath = process.argv[1];
const expectedApps = JSON.parse(process.argv[2]);
const bytes = fs.readFileSync(wasmPath);
const mod = await WebAssembly.compile(bytes);
const imports = {};
for (const imp of WebAssembly.Module.imports(mod)) {
  imports[imp.module] ||= {};
  if (imp.kind === 'function') imports[imp.module][imp.name] = () => 0;
}
const { exports: eng } = await WebAssembly.instantiate(mod, imports);
const dec = new TextDecoder();
const str = (p, l) => dec.decode(new Uint8Array(eng.memory.buffer, p, l));
eng.debug_set_clock(Date.now() / 1000, 0);
const listed = JSON.parse(str(eng.debug_list_apps_ptr(), eng.debug_list_apps_len()));
for (const app of expectedApps) {
  if (!listed.includes(app)) throw new Error(`Missing app '${app}' in WASM (${listed})`);
}
globalThis.ImageData = class { constructor(d,w,h){ this.data=d; this.width=w; this.height=h; } };
globalThis.OffscreenCanvas = class { constructor(w,h){ this.width=w; this.height=h; } getContext(){ return { putImageData(){} }; } };
const fakeCtx = new Proxy({
  save(){}, restore(){}, beginPath(){}, closePath(){}, moveTo(){}, lineTo(){}, bezierCurveTo(){},
  quadraticCurveTo(){}, arc(){}, ellipse(){}, rect(){}, roundRect(){}, fill(){}, stroke(){}, clip(){},
  fillRect(){}, strokeRect(){}, clearRect(){}, translate(){}, scale(){}, rotate(){}, setTransform(){},
  drawImage(){}, createRadialGradient(){ return { addColorStop(){} }; },
  measureText(t){ return { width: t.length * 7 }; }, fillText(){},
}, { set(o,k,v){ o[k]=v; return true; } });
for (let i = 0; i < listed.length; i++) {
  eng.debug_select_app(i);
  const ptr = eng.debug_frame(400, 832, 1.0, NaN, NaN, 0, 0, 0, 1);
  const pkt = new Uint8Array(eng.memory.buffer, ptr, eng.debug_packet_len());
  const frame = paintPacket(fakeCtx, eng, pkt, [0, 0, 400, 832], [0, 0, 400, 832]);
  if (frame.commands <= 0) throw new Error(`App '${listed[i]}' emitted 0 draw commands`);
  JSON.parse(str(eng.debug_state_ptr(), eng.debug_state_len()));
  console.log(`[bhoo-web-check] ${listed[i]}: ok (cmds=${frame.commands}, nodes=${frame.semantics.length})`);
}
"""
    subprocess.run(
        ["node", "--input-type=module", "-e", script, str(wasm_path), json.dumps(apps)],
        cwd=ROOT,
        check=True,
    )
    print(
        f"[bhoo-web-check] validated {len(tests)} test/web-tests.json test case(s) and {len(desktop_entries)} assets/*.desktop entry/entries for {', '.join(apps)}",
        flush=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Debug libbhoo apps in the browser via WASM")
    parser.add_argument(
        "apps",
        nargs="?",
        default="files",
        help="Comma-separated app list (files,settings,calendar,keyboard,notifications) or 'all' (default: files)",
    )
    parser.add_argument("--port", type=int, default=8768, help="Loopback HTTP port (default: 8768)")
    parser.add_argument("--watch", action="store_true", help="Auto-rebuild WASM when .rs files change")
    parser.add_argument("--build-only", action="store_true", help="Compile debug WASM and exit")
    parser.add_argument("--check", action="store_true", help="Build debug WASM, run headless Node smoke check + validate test/web-tests.json & assets/*.desktop, and exit")
    parser.add_argument("--no-build", action="store_true", help="Serve existing debug WASM without rebuilding")
    parser.add_argument(
        "--release-wasm",
        action="store_true",
        help="Optimize the debug WASM bundle for smaller/faster browser playback (still debug-only workbench)",
    )
    parser.add_argument("--offline", action="store_true", help="Pass --offline to cargo")
    parser.add_argument(
        "--target-dir",
        type=Path,
        default=default_target_dir(),
        help="Cargo target directory (defaults to /tmp/bhoo-web-debug-target)",
    )
    args = parser.parse_args()

    apps = parse_apps(args.apps)
    wasm_path = wasm_output_path(args.target_dir, args.release_wasm)
    state = {"status": "ready", "watch": args.watch, "apps": apps}

    if not args.no_build:
        wasm_path = build_wasm(apps, args.target_dir, args.release_wasm, args.offline)
    elif not wasm_path.is_file():
        raise SystemExit(f"WASM not found at {wasm_path}; run without --no-build first")

    if args.check:
        run_headless_check(apps, wasm_path)
        return

    if args.build_only:
        return

    if args.watch:
        def watch_loop() -> None:
            prev = snapshot_mtimes(apps)
            while True:
                time.sleep(0.8)
                cur = snapshot_mtimes(apps)
                if cur != prev:
                    prev = cur
                    state["status"] = "rebuilding…"
                    print("[bhoo-web-debug] change detected, rebuilding WASM…", flush=True)
                    try:
                        build_wasm(apps, args.target_dir, args.release_wasm, args.offline)
                        state["status"] = "ready"
                    except subprocess.CalledProcessError:
                        state["status"] = "build failed (see terminal)"

        threading.Thread(target=watch_loop, daemon=True).start()

    class DebugHandler(SimpleHTTPRequestHandler):
        extensions_map = {
            **SimpleHTTPRequestHandler.extensions_map,
            ".wasm": "application/wasm",
            ".js": "text/javascript; charset=utf-8",
            ".css": "text/css; charset=utf-8",
            ".ttf": "font/ttf",
            ".otf": "font/otf",
            ".woff2": "font/woff2",
        }

        def __init__(self, *h_args, **h_kwargs):
            super().__init__(*h_args, directory=str(WEB_DIR), **h_kwargs)

        def end_headers(self) -> None:
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            super().end_headers()

        def do_GET(self) -> None:
            clean = self.path.split("?", 1)[0]
            if clean == "/__debug/version":
                info = dict(state)
                try:
                    info["mtime"] = str(wasm_path.stat().st_mtime_ns)
                except FileNotFoundError:
                    info["mtime"] = None
                body = json.dumps(info).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if clean == "/__debug/tests":
                body = json.dumps(load_app_web_tests(apps)).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if clean == "/__debug/apps":
                body = json.dumps(load_app_catalog(apps)).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if clean == "/__debug/fonts":
                fonts_dir = WEB_DIR / "fonts"
                shabnam_files = sorted(
                    p.name for p in fonts_dir.glob("Shabnam*.ttf")
                ) if fonts_dir.is_dir() else []
                body = json.dumps(
                    {
                        "latinFamily": "Roboto",
                        "persianFamily": "Shabnam",
                        "webPersianFonts": [f"/fonts/{name}" for name in shabnam_files],
                    }
                ).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            if clean == "/debug.wasm":
                try:
                    with wasm_path.open("rb") as binary:
                        mtime = os.fstat(binary.fileno()).st_mtime_ns
                        data = binary.read()
                except FileNotFoundError:
                    self.send_error(404, "debug.wasm not built")
                    return
                self.send_response(200)
                self.send_header("Content-Type", "application/wasm")
                self.send_header("X-Debug-Wasm-Version", str(mtime))
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            if clean == "/bhoo-web/canvas-painter.js":
                data = PAINTER_JS.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "text/javascript; charset=utf-8")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
                return
            super().do_GET()

        def log_message(self, fmt: str, *log_args) -> None:
            if "/__debug/version" not in fmt and not (
                log_args and isinstance(log_args[0], str) and "/__debug/version" in log_args[0]
            ):
                super().log_message(fmt, *log_args)

    with ThreadingHTTPServer(("127.0.0.1", args.port), DebugHandler) as server:
        first = apps[0]
        print(
            f"Bhoo Web App Debugger ({', '.join(apps)}): "
            f"http://127.0.0.1:{server.server_port}/?app={first}",
            flush=True,
        )
        if args.watch:
            print("Watch mode enabled: editing Rust sources auto-rebuilds & reloads WASM.", flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
