# Writing SpectreOS apps

Native SpectreOS apps must be written in **Rust** and use **libbhoo** for their user interface. The Cargo package is named `libbhoo` and its Rust API is imported as `bhoo`. [libbhoo documentation](../../codes/libs/ui/libbhoo/README.md)

## Structure a native app

Keep each app in its own directory under `codes/apps/<app>/` using the standard layout:

- **`src/`** — owns the app's pages, navigation, and presentation state. When the app runs both inside Wisp and as a standalone Wayland client, share that code between the two entry points and let the host supply system data and execute actions requested by the UI (for example, Settings accepts a `SettingsHostContext` and returns a `SettingsAction`). [Settings UI and host interface](../../codes/apps/settings/src/lib.rs) · [Standalone entry point](../../codes/apps/settings/src/main.rs)
- **`assets/`** — stores the app's required Freedesktop Desktop Entry (`org.spectreos.<App>.desktop`) alongside icons, layouts, and static media.
- **`test/`** — stores the app's test suites and fixtures, including `test/web-tests.json` for the debug-only browser workbench and future integration/UI/snapshot tests.
- **`.cargo/config.toml`** — wires the uniform per-app `cargo dev`, `cargo release`, `cargo app-test`, `cargo app-clean`, and `cargo test-web` aliases.

## Chapters

- [App structure, assets, and tests](app-structure.md) defines the required `codes/apps/<app>/` directory layout, `assets/org.spectreos.<App>.desktop` Freedesktop entries, `test/` organization, and new-app checklist.
- [Cargo commands](cargo-commands.md) covers per-app and repo-wide `cargo dev`, `cargo release`, `cargo app-test`, `cargo app-clean`, and `cargo test-web` workflows.
- [Web debugging](web-debugging.md) covers the debug-only WASM browser workbench, reusable test-case buttons (`test/web-tests.json`), and live System Notification / D-Bus inspection.
- [Responsiveness](responsiveness.md) covers the first frame, background work, caching, and measuring launch latency on the phone.
