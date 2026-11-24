# Debugging SpectreOS apps in the browser (WASM)

SpectreOS includes a **debug-only** WebAssembly workbench (`tools/bhoo-web-debug/`) so you can inspect layouts, gestures, state transitions, Persian (`Shabnam`) typography, system notifications, and D-Bus calls in any desktop browser without flashing a phone.

## Zero overhead in production phone builds

The browser debugger is strictly isolated from release builds:

1. [tools/bhoo-web-debug/](../../tools/bhoo-web-debug/Cargo.toml) is a standalone `wasm32-unknown-unknown` `cdylib` crate that is **never** a dependency of `spectre-files`, `spectre-settings`, `spectre-calendar`, `spectre-keyboard`, or `wisp`.
2. Any browser-only helpers inside an app crate (such as Files' in-memory seeded virtual filesystem in [codes/apps/files/src/wasm_fs.rs](../../codes/apps/files/src/wasm_fs.rs), Calendar's injected wall clock, or Wisp's debug notification state snapshot) are gated behind `#[cfg(target_arch = "wasm32")]`. Native Linux and ARM64 phone builds (`cargo release`) compile **0 bytes** of debug-web code.
3. Per-app Cargo features let `cargo test-web` inside an app directory compile **only** that app and `libbhoo`.

## Running the debugger

```bash
# Inside a single app directory (fast single-app debug build)
cd codes/apps/files
cargo test-web                    # build once and serve http://127.0.0.1:8768/?app=files
cargo test-web-watch              # auto-rebuild & hot-reload on .rs edits
cargo test-web-check              # headless WASM + canvas-painter + test/web-tests.json & desktop check

# From the repository root (all apps in one switcher)
cargo test-web                    # serves files, settings, calendar, keyboard, notifications
cargo test-web keyboard           # single app from root
cargo test-web-watch all --port 8770
```

## Workbench capabilities

- **Card-based App Picker (`K`)**: Click the **App** button in the topbar (or press `K`) to open a searchable modal of app cards powered by each app's `assets/*.desktop` metadata and `test/web-tests.json` count.
- **Visual Feedback Draw Mode (`Alt+F`)**: Click the phone stage and press `Alt+F` (or click the **Feedback · Alt+F** button) to enter Draw Mode, drag a **Box**, **Freehand** mark, or **Arrow** over the phone UI, type your feedback, and press **`Enter`**. Saved feedbacks appear in a compact bottom dock with annotated thumbnails plus one-click **Copy for AI** (structured Markdown with app, viewport, region coordinates, overlapping `bhoo::SemanticNode` targets, and state JSON) and **Copy Image** (copies the 2× annotated PNG—or combined contact sheet for multiple items—straight to your clipboard for pasting into AI chat).
- **Appearance & Settings modal (`,`)**: Click the gear icon in the top-right corner to open the Notion-style settings modal (menu on the left, content on the right) to toggle the **Debugger UI theme** (Dark / Light / System), **App theme** (Dark / Light), dot-grid backdrop, pointer simulation, reduced motion, font diagnostics, and saved layout resets.
- **Viewports & overlays**: Switch between Poco F1 (`400×832`), Compact Phone (`360×740`), Tablet, Desktop, Custom `W×H`, Rotate (`832×400` landscape), **Bounds** overlay for `bhoo::SemanticNode` hit-boxes, and draggable vertical/horizontal Inspector splitters.
- **System Notifications & D-Bus monitor (`Sys & D-Bus` tab)**:
  - Outbound actions from every app (`org.freedesktop.Notifications`, `org.spectreos.OSK0`, `org.freedesktop.portal.FileChooser` / `OpenURI`, `org.freedesktop.NetworkManager`, `org.bluez`, `org.freedesktop.login1`, `org.spectreos.Shiraz1`) appear in real time with `CALL` / `SIGNAL` / `NOTIFY` badges.
  - System notifications emitted by or injected into any app slide in as interactive banners over the top of the phone frame; clicking action pills or dismissing emits `ActionInvoked` / `NotificationClosed` back onto the D-Bus monitor.
  - Use the built-in simulator to inject custom notifications or D-Bus method calls directly into the running app.
- **Persian / Farsi typography (`Shabnam`)**: Mirrors [codes/apps/wisp/src/services/fonts.rs](../../codes/apps/wisp/src/services/fonts.rs) and [codes/os/specrte-nix-f1/modules/fonts.nix](../../codes/os/specrte-nix-f1/modules/fonts.nix) by serving SIL-OFL-1.1 **Shabnam** Without-Latin (WOL) weights so Persian text renders in Shabnam without altering Latin metrics.

## Creating reusable Input & State Test Case buttons

You can define test cases in three ways — every saved or repo-defined test case appears as a one-click button in the **Test Bar** and the **Tests** tab:

1. **Interactive builder**: Click **New Test** in the Test Bar (or **Tests → New**), pick a button name, target app, optional reset-before-run, and add steps (`tapNode`, `tap`, `type`, `key`, `viewport`, `theme`, `filesMode`, `notifMode`, `kbLayout`, `pushNotif`, `dbus`, `back`, `wait`, `assertState`).
2. **Live input recorder**: Click **Record**, interact with the app directly on the canvas, then click **Stop Rec** to turn the recorded steps into a saved test button.
3. **Per-app `test/web-tests.json` (recommended for repo fixtures)**: Place a JSON array at `codes/apps/<app>/test/web-tests.json` (see [App structure, assets, and tests](app-structure.md)). The debug server automatically loads and serves it at `/__debug/tests` whenever that app is active, and `cargo test-web-check` validates its schema.

Example `codes/apps/files/test/web-tests.json`:

```json
[
  {
    "id": "files-images-picker",
    "name": "Images Picker",
    "app": "files",
    "pinned": true,
    "resetBefore": true,
    "steps": [
      { "type": "filesMode", "value": "2" },
      { "type": "wait", "ms": 60 },
      { "type": "assertState", "path": "picker", "expected": "true" }
    ]
  }
]
```

Supported `assertState` operators in `expected`: exact match (`"true"`, `"ir"`), numeric comparison (`">=1"`, `">0"`, `"<=5"`), and case-insensitive substring match (`"~downloads"`).

For programmatic extensions in browser devtools or custom scripts, call:

```js
window.bhooDebug.registerTestCase({
  name: 'My Smoke Test',
  app: 'files',
  pinned: true,
  resetBefore: true,
  steps: [
    { type: 'tapNode', label: 'Images' },
    { type: 'assertState', path: 'location', expected: '~Images' },
  ],
});
```

## Wiring a new app into `cargo test-web`

1. Keep OS-specific I/O behind `#[cfg(not(target_arch = "wasm32"))]` (or behind a host trait/context) and provide a small `#[cfg(target_arch = "wasm32")]` demo state or virtual backend if needed.
2. Expose a `#[cfg(target_arch = "wasm32")] pub fn debug_state_json(&self) -> String` helper so Live App State and `assertState` work automatically.
3. Add an optional dependency and Cargo feature in [tools/bhoo-web-debug/Cargo.toml](../../tools/bhoo-web-debug/Cargo.toml) with `default-features = false`, handle the new `AppKind` in [tools/bhoo-web-debug/src/lib.rs](../../tools/bhoo-web-debug/src/lib.rs), and add the slug to `ALL_APPS` in [tools/debug-bhoo-app.py](../../tools/debug-bhoo-app.py).
4. Add `codes/apps/<app>/assets/org.spectreos.<App>.desktop` and `codes/apps/<app>/test/web-tests.json`, then run `cd codes/apps/<app> && cargo test-web-check`.
