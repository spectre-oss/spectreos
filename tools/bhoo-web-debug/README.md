# Bhoo Web App Debugger (`tools/bhoo-web-debug`)

**Debug-only** WebAssembly browser workbench for interactively debugging standalone `libbhoo` SpectreOS apps (`files`, `settings`, `calendar`, `keyboard`, `notifications`) without touching production phone builds.

## Why Zero Production Overhead?

- This crate lives entirely under `tools/bhoo-web-debug/` and is **never** a dependency of any phone binary (`spectre-files`, `spectre-settings`, `spectre-calendar`, `spectre-keyboard`, `wisp`).
- On `wasm32` builds only (`#[cfg(target_arch = "wasm32")]`), `spectre-files` uses an in-memory seeded SpectreOS virtual filesystem (`codes/apps/files/src/wasm_fs.rs`) and `wisp` exposes debug-only notification helpers, while **zero bytes** are compiled into native Linux / ARM64 release builds.
- Per-app Cargo features (`files`, `settings`, `calendar`, `keyboard`, `notifications`) let you compile only the single app you are actively debugging for fast rebuild loops.

## Quick Start

```bash
# Debug Files app only (fastest build)
python3 tools/debug-bhoo-app.py files

# Debug Notifications only (Wisp heads-up banner, shade, lock screen)
python3 tools/debug-bhoo-app.py notifications --watch

# Debug all apps (Files, Settings, Calendar, Keyboard, Notifications) in one switcher
python3 tools/debug-bhoo-app.py all --watch

# Debug Settings + Files only, optimized WASM
python3 tools/debug-bhoo-app.py files,settings --release-wasm
```

Then open **http://127.0.0.1:8768/?app=files** or **http://127.0.0.1:8768/?app=notifications** in your browser.

## Workbench Features

- **Device Viewports**: Poco F1 (`400×832`), Compact Phone (`360×740`), Files Window (`480×750`), Tablet (`768×1024`), Desktop (`1024×720`), Custom W×H, Rotate.
- **Files Mode Switcher**: Switch live between Normal Browser, Open File Picker, Multi-Image Picker, Folder Picker, and Save As Picker.
- **Notifications Workbench**: Switch live between Pinned Heads-Up Banner, Notification Shade, Home + Auto-Expire Heads-Up, and Lock Screen; push Normal, Interactive Actions, or Critical (`image-data`) notifications on demand and inspect emitted `HostCommand` signals.
- **Live Controls**: App Back (`← Back` / `Esc`), Reset State & Virtual FS, Dark/Light theme, Touch vs Mouse pointer simulation, Reduced Motion.
- **Input & State Test Case Buttons**: Create, record, edit, pin, run, and export/import reusable input/state test cases (with state assertions) as clean one-click buttons in the Test Bar and Tests tab, or register them via `window.bhooDebug.registerTestCase(...)`.
- **System Notifications & D-Bus Monitor**: Captures outbound/inbound D-Bus calls, signals (`org.freedesktop.Notifications`, `org.spectreos.OSK0`, `org.freedesktop.portal.*`, `org.freedesktop.NetworkManager`, `org.bluez`, `org.spectreos.Shiraz`), renders live floating system notification banners over any app with Action/Dismiss feedback, and lets you inject D-Bus/notification payloads interactively.
- **Resizable Inspector & Overlays**:
  - Drag the vertical divider between the canvas and Inspector (or the horizontal divider between Live App State and Action Log); double-click either divider to reset.
  - Uses Heroicons (`heroicons.com`, MIT) icons and sleek custom dark scrollbars throughout.
  - **Bounds Overlay**: Draws hit-box rectangles for every `bhoo::SemanticNode` and highlights hovered targets.
  - **Frame Telemetry**: WASM build time, Canvas2D paint time, DrawCommand count, SemanticNode count, packet bytes, idle/active state.
  - **Live App State JSON** + timestamped **Action & Event Log**.
  - **Filterable Semantics Tree** with hover-to-highlight on the live canvas.

## Adding Another `libbhoo` App

1. Add an optional dependency + feature in `tools/bhoo-web-debug/Cargo.toml` with `default-features = false`.
2. Add an entry in `tools/bhoo-web-debug/src/lib.rs` (render frame, optional `back()`, `edit_text()`, `set_edit_text()`, `debug_state_json()`).
3. Add its feature name to `ALL_APPS` in `tools/debug-bhoo-app.py`.
