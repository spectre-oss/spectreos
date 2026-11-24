# SpectreOS

**SpectreOS** is an open-source mobile Linux operating system, Wayland compositor/shell, UI toolkit, native application suite, and Android compatibility stack organized as a unified monorepo.

---

## Repository Structure

```text
spectreos/
├── codes/
│   ├── apps/
│   │   ├── wisp/               # Wayland compositor (Smithay), System UI, Lock Screen & Launcher
│   │   ├── keyboard/           # Wayland on-screen keyboard (English, Persian, Terminal, Emoji)
│   │   ├── settings/           # System Settings app (standalone Wayland or embedded in Wisp)
│   │   ├── files/              # Native file manager & XDG file chooser
│   │   ├── calendar/           # Native calendar (Day/Week/Month/Agenda, RRULE, iCalendar)
│   │   └── bhoo-web/           # WebAssembly demos, gallery, and Android XML test scenes
│   ├── libs/
│   │   └── ui/libbhoo/         # Minimal Rust GUI toolkit (GLES 3.0, Wayland, WASM, Android XML)
│   ├── os/
│   │   └── specrte-nix-f1/     # Mobile NixOS port for Xiaomi Pocophone F1 (beryllium, SDM845)
│   ├── shiraz-core/            # Git submodule: Shiraz Android compatibility layer (spectre-oss/shiraz)
│   └── utils/
│       └── wisp_remote_viewer/ # Native Rust H.264/PNG viewer & input controller for Wisp remote_control
├── docs/
│   ├── writing-spectreos-apps/ # App authoring guide (structure, Cargo commands, web debugging, latency)
│   └── freedesktop/            # Local Freedesktop.org specifications mirror (XDG, D-Bus, Notifications)
├── tools/
│   ├── spectre-xtask/          # Unified Cargo task runner (`cargo dev`, `cargo release`, `cargo test-web`)
│   ├── bhoo-web-debug/         # Interactive WASM browser workbench for developing SpectreOS apps
│   ├── msdf-icon-gen/          # Multi-channel SDF (MSDF) icon atlas generator
│   └── ...                     # WebXDC packagers, Android XML importers, asset generators, test suites
├── .cargo/config.toml          # Repo-wide Cargo aliases wired to `tools/spectre-xtask`
├── Makefile                    # `make init` links subproject `docs/` folders under `docs/`
├── LICENSE                     # GNU GPL v3 (`GPL-3.0-only`)
├── SECURITY.md                 # Private vulnerability reporting policy
└── THIRD_PARTY_NOTICES.md      # Third-party attributions and licenses
```

### Subprojects & Core Components

| Component | Path | Description |
| --- | --- | --- |
| **`libbhoo`** | [`codes/libs/ui/libbhoo/`](codes/libs/ui/libbhoo/README.md) | Immediate-mode-inspired Rust UI toolkit (`bhoo` crate) with analytical critically damped motion, ink ripples, HarfRust/Skrifa shaping, GLES 3.0 / Wayland / WASM backends, Material Design 3 components, and Android XML layout/menu runtime. |
| **Wisp** | [`codes/apps/wisp/`](codes/apps/wisp/README.md) | Unified Wayland compositor (embedded Smithay + DRM/KMS + GBM + EGL/GLES3 + libinput), System UI, Lock Screen, `org.freedesktop.Notifications` service, Home/Drawer/Overview/Split-Screen/Freeform shell, Linux hardware bridge, and optional `remote_control` H.264 stream. |
| **SpectreOS Keyboard** | [`codes/apps/keyboard/`](codes/apps/keyboard/README.md) | Standalone Wayland OSK (`spectre-keyboard`, `org.spectreos.OSK0`) with `libbhoo` GLES 3.0 rendering, English (`en-US`), Persian (`fa-IR` with ZWNJ), Terminal, and Emoji layouts, adaptive touch scoring, and `zwlr_layer_shell_v1` / `zwp_input_method_v2` / `zwp_virtual_keyboard_v1` support. |
| **SpectreOS Settings** | [`codes/apps/settings/`](codes/apps/settings/README.md) | System Settings app synced with `~/.config/wisp/settings.conf` and `lock.conf`, including hardware telemetry, lock-screen/notification controls, and Shiraz management. |
| **SpectreOS Files** | [`codes/apps/files/`](codes/apps/files/README.md) | File manager and XDG file chooser with Trash, recursive search, image previews, and ZIP archive support. |
| **SpectreOS Calendar** | [`codes/apps/calendar/`](codes/apps/calendar/README.md) | Local calendar library embedded in Wisp with Day/Week/Month/Agenda views, recurrence (`rrule`), reminders, and iCalendar import/export. |
| **Bhoo Web Demos** | [`codes/apps/bhoo-web/`](codes/apps/bhoo-web/README.md) | WebAssembly (`wasm32-unknown-unknown`) playground, component gallery, Angular Material side-by-side comparison, and Android XML demos. |
| **Shiraz** *(submodule)* | [`codes/shiraz-core/`](https://github.com/spectre-oss/shiraz) | Android compatibility layer (ART + SDL2 + EGL/GLES + Skia Ganesh + OS bridges) running Android APKs natively on Linux, integrated into Wisp and `specrte-nix-f1`. |
| **Spectre NixOS (Poco F1)** | [`codes/os/specrte-nix-f1/`](codes/os/specrte-nix-f1/README.md) | Mobile NixOS system configuration, mainline SDM845 kernel config & patches, firmware verification, camera/sensor/fingerprint modules, and flashing scripts for Xiaomi Pocophone F1 (`beryllium`). |
| **Wisp Remote Viewer** | [`codes/utils/wisp_remote_viewer/`](codes/utils/wisp_remote_viewer/README.md) | Native Rust H.264/PNG display viewer and touch/keyboard forwarder for Wisp development builds. |

---

## Getting Started

### Prerequisites

- **Rust & Cargo** (`edition = "2024"`):
  - Targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (device cross-builds), and `wasm32-unknown-unknown` (web workbench / `bhoo-web`).
- **System Libraries (Linux desktop/device builds)**: Wayland client/EGL/GLES3 headers, `libinput`, `libudev`, `libxkbcommon`, `libseat`, `libgbm`, `pkg-config`.
- **Python 3**: Build helpers, Android XML importers, Wayland pixel tests, and NixOS helper scripts.
- **Nix** *(for `codes/os/specrte-nix-f1`)*: Cross-compiling system closures, kernel, and boot images for `aarch64-linux`.
- **Node.js & Chromium/Playwright** *(optional)*: Angular Material comparison build and web verification tests.

### Clone with Submodules

```sh
git clone --recurse-submodules git@github.com:spectre-oss/spectreos.git
cd spectreos
```

If already cloned without submodules:

```sh
git submodule update --init --recursive
```

Optional: symlink subproject `docs/` directories into the top-level `docs/` folder:

```sh
make init
```

---

## Common Workflows

### Unified App Commands (`.cargo/config.toml` + `spectre-xtask`)

All SpectreOS Rust apps share a unified task runner (`cargo app-help` for full usage) that shares a common build directory (`/tmp/spectre-apps-target`, overridable via `SPECTRE_TARGET_DIR` or `CARGO_TARGET_DIR`):

```sh
# Check, test, or build a single app (`files`, `settings`, `calendar`, `keyboard`, `wisp`) or `all`
cargo app-check all
cargo app-test all
cargo dev wisp
cargo release all

# Run a standalone desktop preview window (Wayland)
cargo app-run settings
cargo app-run files
cargo app-run wisp

# Launch the interactive WASM browser workbench (`bhoo-web-debug`)
cargo test-web settings
cargo test-web-watch calendar

# Clean shared Cargo target directories
cargo clean-all
```

### `libbhoo` UI Toolkit & Web Demos

```sh
# Core unit & integration tests
make -C codes/libs/ui/libbhoo check
make -C codes/libs/ui/libbhoo test

# Headless Weston + Mesa GLES pixel tests
make -C codes/libs/ui/libbhoo test-wayland

# Build & serve the Rust-only WASM playground on http://127.0.0.1:8766
make -C codes/libs/ui/libbhoo serve

# Package WebXDC demo bundles
make -C codes/libs/ui/libbhoo webxdc-gallery
```

### Wisp Compositor & Remote Viewer

```sh
# Wisp tests and compositor builds
make -C codes/apps/wisp test
make -C codes/apps/wisp build-wayland       # desktop preview window
make -C codes/apps/wisp build-compositor    # production DRM/KMS device compositor
make -C codes/apps/wisp build_development   # device compositor with remote_control enabled

# Native Rust remote viewer
cargo test --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml
cargo run --release --locked --manifest-path codes/utils/wisp_remote_viewer/Cargo.toml -- --session poco
```

### Mobile NixOS (`codes/os/specrte-nix-f1`)

```sh
# Fetch pinned upstream sources, evaluate, test, and build Poco F1 images
make -C codes/os/specrte-nix-f1 test
make -C codes/os/specrte-nix-f1 eval
make -C codes/os/specrte-nix-f1 boot-image PANEL=tianma
make -C codes/os/specrte-nix-f1 build PANEL=tianma
```

See [`codes/os/specrte-nix-f1/README.md`](codes/os/specrte-nix-f1/README.md) for diverted Nix store usage, panel selection (`tianma` / `ebbg`), and installation instructions.

### Shiraz Android Compatibility Layer (`codes/shiraz-core`)

```sh
make -C codes/shiraz-core deps
make -C codes/shiraz-core build
make -C codes/shiraz-core run APK=/path/to/app.apk
```

See [`codes/shiraz-core/README.md`](codes/shiraz-core/README.md) for AppImage/Flatpak packaging, warm daemon setup, and platform notes.

---

## Documentation Index

- **App Authoring Guide**: [`docs/writing-spectreos-apps/README.md`](docs/writing-spectreos-apps/README.md)
- **Apps Overview**: [`codes/apps/README.md`](codes/apps/README.md)
- **`libbhoo` Documentation**: [`codes/libs/ui/libbhoo/docs/README.md`](codes/libs/ui/libbhoo/docs/README.md)
- **Wisp Compositor & Shell Docs**: [`codes/apps/wisp/docs/README.md`](codes/apps/wisp/docs/README.md)
- **Poco F1 NixOS Docs**: [`codes/os/specrte-nix-f1/docs/README.md`](codes/os/specrte-nix-f1/docs/README.md)
- **Utilities**: [`codes/utils/README.md`](codes/utils/README.md)
- **Tooling Reference**: [`tools/README.md`](tools/README.md)
- **Freedesktop.org Local Specs**: [`docs/freedesktop/INDEX.md`](docs/freedesktop/INDEX.md)

---

## Security & Licensing

- **License**: SpectreOS original source code is licensed under the **[GNU GPL v3 (`GPL-3.0-only`)](LICENSE)**.
- **Embedded & Third-Party Components**: Embedded `codes/apps/wisp/smithay` is MIT-licensed ([`LICENSE.txt`](codes/apps/wisp/smithay/LICENSE.txt)); `codes/os/specrte-nix-f1` integrates MIT-licensed `mobile-nixos` and requires user-supplied proprietary device firmware kept out of Git. See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for full upstream attributions and license notices.
- **Security Policy**: Report vulnerabilities privately via GitHub Security Advisories as described in [`SECURITY.md`](SECURITY.md).
