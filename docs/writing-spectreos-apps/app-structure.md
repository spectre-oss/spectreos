# App structure, assets, and tests

Every native SpectreOS application lives in its own crate under `codes/apps/<app>/` and follows a uniform directory layout so tooling (`spectre-xtask`, `debug-bhoo-app.py`, packaging, and Wisp's Freedesktop launcher) works identically across all apps.

## Standard directory layout

```text
codes/apps/<app>/
├── .cargo/
│   └── config.toml                    # Uniform per-app Cargo aliases (dev, release, test-web, ...)
├── assets/
│   ├── org.spectreos.<App>.desktop    # Required Freedesktop Desktop Entry v1.5 file
│   └── ...                            # Icons, PNG/SVG art, keyboard layouts, static JSON data
├── test/
│   ├── web-tests.json                 # Reusable input/state test buttons for `cargo test-web`
│   └── ...                            # Future test suites (fixtures, UI/IPC/snapshot tests)
├── src/
│   ├── lib.rs                         # Shared libbhoo UI, pages, state, and host interface
│   ├── main.rs                        # Standalone Wayland entry point (when applicable)
│   └── ...
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## `assets/` and `.desktop` entries

Every SpectreOS app **must** ship its Freedesktop Desktop Entry v1.5 file inside `codes/apps/<app>/assets/` using reverse-DNS naming (`org.spectreos.<App>.desktop`), alongside any app icons, images, or static asset files.

Keeping `.desktop` entries inside `assets/` keeps the crate root clean and puts all installable non-code resources in one predictable directory for Nix/OS packaging and Freedesktop discovery ([Desktop Entry Specification](../freedesktop/desktop-entry-spec/index.md)).

### Required fields

| Key | Requirement | Notes |
| --- | --- | --- |
| `Type` | `Application` | Standard desktop application entry |
| `Version` | `1.5` | Matches Wisp's Desktop Entry v1.5 parser (`codes/apps/wisp/src/freedesktop.rs`) |
| `Name` | User-visible title | Displayed in Wisp's App Drawer, Overview, and Settings → Apps |
| `GenericName` / `Comment` | Short description | Used for launcher search and Settings app details |
| `Exec` | Executable name | Binary installed on `PATH` (e.g. `spectre-settings`, `spectre-files`) |
| `Icon` | Theme icon or app ID | Prefer `org.spectreos.<App>` or a standard XDG icon theme name |
| `Terminal` | `false` | Native libbhoo GUI apps do not launch inside a terminal |
| `Categories` | Semicolon-delimited | Freedesktop menu categories (e.g. `Settings;System;`, `System;FileManager;`) |
| `Keywords` | Semicolon-delimited | Improves App Drawer search matching |
| `StartupWMClass` | `org.spectreos.<App>` | Must match the Wayland `xdg_toplevel` `app_id` so Wisp links windows to icons |

- For **shell or system daemon apps** that should not appear as launchable icons in the App Drawer (such as Wisp itself or the SpectreOS on-screen keyboard), add `NoDisplay=true` (validate entries with `desktop-file-validate assets/*.desktop`).
- For **file or MIME handlers** (such as Calendar `.ics` files), declare `MimeType=` (e.g. `MimeType=text/calendar;`).

Example (`codes/apps/settings/assets/org.spectreos.Settings.desktop`):

```ini
[Desktop Entry]
Type=Application
Version=1.5
Name=SpectreOS Settings
GenericName=System Settings
Comment=Configure SpectreOS display, notifications, lock screen, network, and device preferences
Exec=spectre-settings
Icon=org.spectreos.Settings
Terminal=false
Categories=Settings;System;
Keywords=settings;preferences;wifi;bluetooth;display;notifications;lock;battery;storage;about;
StartupNotify=true
StartupWMClass=org.spectreos.Settings
```

## `test/` directory

Every SpectreOS app keeps its non-unit test suites, fixtures, and browser debugger scenarios inside `codes/apps/<app>/test/` rather than at the crate root so additional test categories (integration harnesses, golden fixtures, D-Bus/portal test scripts, snapshot baselines) have a dedicated home as the app grows.

- **`test/web-tests.json`**: JSON array of input, D-Bus, viewport, theme, and state-assertion test cases automatically loaded by `cargo test-web` at `/__debug/tests` and validated by `cargo test-web-check`. See [Web debugging](web-debugging.md) for the full step schema and assertion operators.
- **Rust unit tests**: Keep fast unit tests inside `src/` (`#[cfg(test)]`) and run them with `cargo app-test`.
- **Future test suites**: Add subdirectories or fixtures inside `test/` (for example `test/fixtures/`, `test/dbus/`, `test/snapshots/`) without cluttering the crate root or shipping test data into production binaries.

## Current SpectreOS apps

| App | Desktop entry (`assets/`) | Web test suite (`test/`) |
| --- | --- | --- |
| **Files** | [`codes/apps/files/assets/org.spectreos.Files.desktop`](../../codes/apps/files/assets/org.spectreos.Files.desktop) | [`codes/apps/files/test/web-tests.json`](../../codes/apps/files/test/web-tests.json) |
| **Settings** | [`codes/apps/settings/assets/org.spectreos.Settings.desktop`](../../codes/apps/settings/assets/org.spectreos.Settings.desktop) | [`codes/apps/settings/test/web-tests.json`](../../codes/apps/settings/test/web-tests.json) |
| **Calendar** | [`codes/apps/calendar/assets/org.spectreos.Calendar.desktop`](../../codes/apps/calendar/assets/org.spectreos.Calendar.desktop) | [`codes/apps/calendar/test/web-tests.json`](../../codes/apps/calendar/test/web-tests.json) |
| **Keyboard** | [`codes/apps/keyboard/assets/org.spectreos.Keyboard.desktop`](../../codes/apps/keyboard/assets/org.spectreos.Keyboard.desktop) | [`codes/apps/keyboard/test/web-tests.json`](../../codes/apps/keyboard/test/web-tests.json) |
| **Wisp** | [`codes/apps/wisp/assets/org.spectreos.Wisp.desktop`](../../codes/apps/wisp/assets/org.spectreos.Wisp.desktop) | [`codes/apps/wisp/test/web-tests.json`](../../codes/apps/wisp/test/web-tests.json) |

## Checklist for adding a new SpectreOS app

1. Create `codes/apps/<app>/` with `Cargo.toml`, `src/lib.rs`, and (for standalone Wayland apps) `src/main.rs`. Keep UI and state separate from host OS services.
2. Create `codes/apps/<app>/assets/org.spectreos.<App>.desktop` (Desktop Entry v1.5) and place icons or static media under `assets/`.
3. Create `codes/apps/<app>/test/web-tests.json` with smoke-test cases for key views, inputs, and state assertions.
4. Add `codes/apps/<app>/.cargo/config.toml` and register the app in [tools/spectre-xtask/src/main.rs](../../tools/spectre-xtask/src/main.rs) (see [Cargo commands](cargo-commands.md)).
5. Wire the debug-only WASM feature in [tools/bhoo-web-debug/](../../tools/bhoo-web-debug/Cargo.toml) (see [Web debugging](web-debugging.md)) and run `cd codes/apps/<app> && cargo test-web-check`.
6. Follow the first-frame and worker-thread rules in [Responsiveness](responsiveness.md).
