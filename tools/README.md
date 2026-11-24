# SpectreOS Tooling (`tools/`)

Developer, build, asset-generation, and verification tooling for the SpectreOS monorepo.

## Prerequisites

- **Rust / Cargo** (`edition = "2024"`, targets `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `wasm32-unknown-unknown`) — for `spectre-xtask`, `bhoo-web-debug`, `msdf-icon-gen`, and WASM demos.
- **Python 3** — for build/packaging scripts, Android XML importers, Wayland verification, and review tooling.
- **Node.js & Chromium / Playwright** — for `build-bhoo-angular.mjs`, `bhoo-png-diff.mjs`, and web verification suites.
- **Java** — only needed if regenerating `loading_shapes.rs` via `BhooShapes.java`.

---

## Monorepo Task Runner & App Workbench

- [`spectre-xtask/`](spectre-xtask/Cargo.toml) — Zero-dependency Rust task runner wired into root [`.cargo/config.toml`](../.cargo/config.toml) (`cargo dev`, `cargo release`, `cargo app-check`, `cargo app-test`, `cargo app-run`, `cargo test-web`, `cargo test-web-watch`, `cargo clean-all`). See [`docs/writing-spectreos-apps/cargo-commands.md`](../docs/writing-spectreos-apps/cargo-commands.md).
- [`debug-bhoo-app.py`](debug-bhoo-app.py) — Compiles and serves the interactive WASM browser workbench for `files`, `settings`, `calendar`, `keyboard`, and `wisp` notifications (`cargo test-web <app>`).
- [`bhoo-web-debug/`](bhoo-web-debug/README.md) — Debug-only WASM browser workbench crate (`bhoo-web-debug`) hosting SpectreOS apps in a simulated phone viewport. Never linked into phone release binaries.

---

## Web, WASM & WebXDC Packaging (`libbhoo` / `bhoo-web`)

- [`build-bhoo-web.py`](build-bhoo-web.py) — Builds the Bhoo WASM playground (`--rust-only`) or full Bhoo + Angular Material comparison site.
- [`build-bhoo-angular.mjs`](build-bhoo-angular.mjs) — Bundles the optional Angular Material comparison UI.
- [`serve-bhoo-web.py`](serve-bhoo-web.py) — Local development HTTP server for `codes/apps/bhoo-web/site`.
- [`build-bhoo-webxdc.py`](build-bhoo-webxdc.py) — Packages focused components, gallery, or Android demos (`--plain`, `--component`, `--gallery`, `--launcher-demo`, `--contacts-demo`, `--gmail-demo`, `--android-demo`) as standalone `.xdc` WebXDC bundles.
- [`build-bhoo-app-tests.py`](build-bhoo-app-tests.py) — Packages the Android XML app-test suite as a WebXDC bundle.

---

## Android XML, Menu & Asset Importers

- [`import-android-layout.py`](import-android-layout.py) / [`test-android-layout.py`](test-android-layout.py) — Android layout XML importer and unit test suite.
- [`import-android-menu.py`](import-android-menu.py) / [`test-android-menu.py`](test-android-menu.py) — Android menu XML importer and unit test suite.
- [`import-android-app-tests.py`](import-android-app-tests.py) — Imports Android app layout test definitions for `bhoo-web`.
- [`android_png.py`](android_png.py) & [`android_vector.py`](android_vector.py) — Nine-patch/PNG and `VectorDrawable` converters for `libbhoo`.

---

## Icon, Shape & Spec Generators

- [`msdf-icon-gen/`](msdf-icon-gen/Cargo.toml) — Rust multi-channel signed distance field (MSDF) generator producing `codes/libs/ui/libbhoo/assets/material-system/icons.msdf`.
- [`freedesktop_specs/`](freedesktop_specs/README.md) — Mirrors and converts Freedesktop.org specifications into `docs/freedesktop/`.
- [`BhooShapes.java`](BhooShapes.java) & [`generate-bhoo-loading.py`](generate-bhoo-loading.py) — Generates Material 3 loading indicator polygon morph shapes (`loading_shapes.rs`).
- [`generate-beercss-shapes.py`](generate-beercss-shapes.py) — Generates BeerCSS decorative shapes (`beercss_shapes.rs`).
- [`generate-material-symbols.py`](generate-material-symbols.py), [`generate-material-system-icons.py`](generate-material-system-icons.py), [`generate-material-navigation-icons.py`](generate-material-navigation-icons.py), [`generate-material-card-icon.py`](generate-material-card-icon.py) — Regenerates bundled Material icon tables.
- [`generate-android-clock-shapes.py`](generate-android-clock-shapes.py), [`generate-android-expandable-indicators.py`](generate-android-expandable-indicators.py), [`generate-android-search-icons.py`](generate-android-search-icons.py), [`generate-android-time-picker-icons.py`](generate-android-time-picker-icons.py), [`generate-android-zoomcontrols.py`](generate-android-zoomcontrols.py) — Regenerates Android widget vector/bitmap tables.
- [`generate-bhoo-media.mjs`](generate-bhoo-media.mjs) & [`generate-bhoo-test-video.py`](generate-bhoo-test-video.py) — Generates synthetic demo media assets for `bhoo-web`.

---

## Verification & Component Review Workflow

- [`test-bhoo-wayland.py`](test-bhoo-wayland.py) — Headless Weston + Mesa GLES pixel verification suite (`make -C codes/libs/ui/libbhoo test-wayland`).
- [`test-bhoo-web.mjs`](test-bhoo-web.mjs) — Core Bhoo WebAssembly playground verification.
- [`test-bhoo-comparison.mjs`](test-bhoo-comparison.mjs) — Bhoo vs. Angular Material visual comparison check.
- [`test-bhoo-app-tests.mjs`](test-bhoo-app-tests.mjs) — Android XML app-test WebXDC verification.
- [`bhoo-png-diff.mjs`](bhoo-png-diff.mjs) — Pixel-wise PNG diff helper.
- [`bhoo-review.py`](bhoo-review.py), [`bhoo-review-hook.py`](bhoo-review-hook.py), [`bhoo_review/`](bhoo_review/), [`test_bhoo_review.py`](test_bhoo_review.py) — Human-in-the-loop WebXDC component review packaging and ledger workflow (see [`codes/libs/ui/libbhoo/docs/review-loop.md`](../codes/libs/ui/libbhoo/docs/review-loop.md)).

---

*Invoke any script with `--help` for its full CLI options.*
