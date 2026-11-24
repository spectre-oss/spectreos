# Cargo commands for SpectreOS apps

Every SpectreOS app under `codes/apps/` ships with a uniform set of Cargo commands powered by the zero-dependency task runner in [tools/spectre-xtask/](../../tools/spectre-xtask/src/main.rs) and per-app `.cargo/config.toml` aliases.

Every command works **app by app** when run inside an app folder (`cd codes/apps/files && cargo …`) and **in general** when run from the repository root (`cargo … <app>` or `cargo … all`).

## Quick reference

| Command | Inside `codes/apps/<app>/` | From repository root | Purpose |
| --- | --- | --- | --- |
| `cargo dev` / `cargo app-build` | Builds this app in dev mode with its required native features | `cargo dev [app\|all]` | Native desktop / device development build |
| `cargo release` / `cargo app-release` | Builds optimized **production / phone release** binary (`--release`) | `cargo release [app\|all]` | Production binary (**0 bytes** of browser debug code) |
| `cargo app-check` | Runs `cargo check` across this app's targets | `cargo app-check [app\|all]` | Fast type/borrow check without linking |
| `cargo app-test` | Runs native Rust unit tests (`--lib`) | `cargo app-test [app\|all]` | Native Rust test suite |
| `cargo app-run` | Launches native Wayland desktop window (`--preview` / `--window`) | `cargo app-run <app>` | Run natively on a Wayland desktop |
| `cargo app-clean` / `cargo clean-all` | Cleans this app's target + `bhoo-web-debug` artifacts | `cargo app-clean [app\|all]` | Reclaim build disk space (`all` also clears debug WASM cache) |
| `cargo test-web` | Compiles **only this app** to debug WASM and starts the browser debugger | `cargo test-web [app\|all]` | Interactive browser debugging (`http://127.0.0.1:8768`) |
| `cargo test-web-watch` | Same as `cargo test-web` with `--watch` live auto-rebuild | `cargo test-web-watch [app\|all]` | Live edit → auto-rebuild → auto-reload loop |
| `cargo test-web-check` | Builds debug WASM, renders a frame via `canvas-painter.js`, and validates `test/web-tests.json` + `assets/*.desktop` | `cargo test-web-check [app\|all]` | Headless CI / smoke check for web debugging |
| `cargo app-help` | Prints available commands and app slugs | `cargo app-help` | Built-in CLI summary |

> Built-in Cargo commands (`cargo build`, `cargo test`, `cargo clean`) remain completely standard and unmodified because Cargo reserves built-in names. Use `cargo dev`, `cargo release`, `cargo app-clean`, and `cargo test-web` for feature-aware SpectreOS workflows.

## Supported apps and native feature mapping

| App slug | Directory | Dev build (`cargo dev`) | Phone release (`cargo release`) | Web slug (`cargo test-web`) |
| --- | --- | --- | --- | --- |
| `files` | `codes/apps/files` | `--features wayland` | `--release --features wayland` | `files` |
| `settings` | `codes/apps/settings` | `--features wayland` | `--release --features wayland` | `settings` |
| `calendar` | `codes/apps/calendar` | `--lib` | `--release --lib` | `calendar` |
| `keyboard` | `codes/apps/keyboard` | `--features osk` | `--release --features osk` | `keyboard` |
| `wisp` / `notifications` | `codes/apps/wisp` | `--features wayland` | `--release --features compositor` | `notifications` |

## Common examples

```bash
# Work on one app only (fastest compile loop)
cd codes/apps/files
cargo test-web-watch              # live browser debugger ONLY for Files
cargo test-web-check --offline    # headless WASM + test/web-tests.json & desktop check
cargo app-test                    # native Rust unit tests
cargo dev                         # native Wayland dev build
cargo release                     # optimized production phone build
cargo app-clean                   # clean Files + debug WASM artifact

# Work across the repository from the root
cargo test-web                    # browser debugger with ALL apps in one switcher
cargo test-web keyboard           # browser debugger ONLY for Keyboard
cargo test-web-watch files,settings --port 8770
cargo app-check all
cargo app-test all
cargo release keyboard
cargo clean-all all
```

## Disk-space safety and shared target directories

Because compiling `libbhoo`, `image`, `chrono`, and `smithay` separately inside every app's `target/` directory duplicates gigabytes of artifacts:

- `tools/spectre-xtask` shares one target directory across all apps by default (`/tmp/spectre-apps-target`, or override with `SPECTRE_TARGET_DIR` / `CARGO_TARGET_DIR`).
- `tools/debug-bhoo-app.py` keeps debug-only `wasm32-unknown-unknown` builds in `/tmp/bhoo-web-debug-target` (override with `BHOO_DEBUG_TARGET_DIR`).
- `cargo clean-all all` removes both app targets and the debug WASM cache.

## Adding Cargo commands to a new SpectreOS app

1. Create your crate under `codes/apps/<name>/` following [App structure, assets, and tests](app-structure.md) (including `assets/org.spectreos.<Name>.desktop`).
2. Add a `codes/apps/<name>/.cargo/config.toml` copying the alias block from [codes/apps/files/.cargo/config.toml](../../codes/apps/files/.cargo/config.toml) and replacing `--app files` with `--app <name>`.
3. Add an `AppSpec` entry to `APPS` in [tools/spectre-xtask/src/main.rs](../../tools/spectre-xtask/src/main.rs).
4. Add `codes/apps/<name>/test/web-tests.json` and wire the crate feature into [tools/bhoo-web-debug/Cargo.toml](../../tools/bhoo-web-debug/Cargo.toml) (see [Web debugging](web-debugging.md)).
