# Wisp Documentation

**Wisp** is the unified SpectreOS Wayland Compositor, System UI, Lock Screen, and Launcher built directly on [`libbhoo`](../../../libs/ui/libbhoo/docs/README.md) and Smithay.

## Architecture

- **`WispShell<E: EmbeddedApps = NoopEmbeddedApps>`** (aliased from `LauncherDemo<E>` for backward compatibility) owns shell state across:
  1. Home workspace, dock, app drawer, widgets picker, and wallpaper/style customization (`src/lib.rs`).
  2. Two-stage Notification Shade, Quick Settings grid, QS Tile Editor, Volume HUD, and Power Menu (`src/lib.rs`).
  3. Multitasking: Overview / Recents carousel, live Wayland app switching, split-screen divider, and freeform windows (`src/lib.rs`).
  4. Native `libbhoo` Lock Screen (`src/lockscreen.rs`) with Swipe/PIN/Password unlock, persistent `~/.config/wisp/lock.conf`, and auto-lock timeouts.
  5. Interactive System Settings hierarchy (`src/settings.rs`), including Security → Screen lock management.
  6. Built-in apps and snapshot previews (`src/apps.rs`) plus high-resolution `176×176` `xxxhdpi` app icons (`src/icons.rs`, `assets/icons/*.png`).
  7. Freedesktop.org integration (`src/freedesktop.rs`): XDG Base Directory v0.8, Desktop Entry v1.5 (`.desktop` discovery across `XDG_DATA_HOME`/`XDG_DATA_DIRS` including NixOS `/run/current-system/sw/share`, localized strings, `Exec=` quoting & field codes, `TryExec`, `Terminal`, `DBusActivatable`, `StartupWMClass`, `Keywords`, `[Desktop Action]`, `X-Shiraz-Package`, and live directory hot-reload), and Icon Theme v0.13 (`index.theme` lookup, `Inherits` + `hicolor` + `pixmaps`, PNG decoding & native SVG rasterization with `resvg` into `libbhoo` GPU textures).
  8. Shiraz Android Compatibility subsystem (`src/services/shiraz.rs`, `src/backends/linux/shiraz.rs`, `spectre-settings` Shiraz page): Notification Center `Enable Shiraz` Quick Settings tile, dedicated `Settings → Shiraz` page synced via `$XDG_CONFIG_HOME/wisp/settings.conf` and `$XDG_RUNTIME_DIR/shiraz/status.json`, persistent warm `shiraz-daemon` IPC (`$XDG_RUNTIME_DIR/shiraz/control.sock`) with `dex2oat` AOT boot/framework/APK cache (`$XDG_CACHE_HOME/shiraz/aot/`) and `mmap` + `MADV_WILLNEED` page-cache warming, XDG Autostart v0.5 (`org.spectreos.Shiraz.desktop`), MIME Apps v1.0.1 (`application/vnd.android.package-archive`), and seamless Wayland/SpectreOS Keyboard (`org.spectreos.OSK0`)/Notifications/multi-touch wiring.
- **Integrated Wayland Compositor & Embedded Smithay Core (`src/compositor.rs` + `smithay/`, `--features compositor`)**:
  - The Smithay compositor engine (`smithay/src/`, MIT-licensed in `smithay/LICENSE.txt`) lives directly inside `codes/apps/wisp/smithay/` as a workspace member so compositor engine internals and Wisp UI evolve together in one repository folder.
  - Single-binary DRM/KMS + GBM + GLES3 compositor living inside the `wisp` crate (`wisp::compositor::run()`).
  - Directly reads and mutates `WispShell` internals (`sys`, `lockscreen`, `mode`, `tasks`, `shade_stage`, `power_state`, `vol_open`, `host_commands`) with zero IPC boundary between compositor and launcher chrome.
  - **`HardwareBridge`** syncs `WispShell::sys` with live Linux hardware:
    - Battery capacity & charging state (`/sys/class/power_supply/*`)
    - Display backlight & DPMS/bl_power sleep/wake (`/sys/class/backlight/*`)
    - Flashlight / status LED (`/sys/class/leds/*`)
    - Wi-Fi & Airplane radio control (`nmcli`) and Bluetooth (`rfkill`)
    - System volume & mute (`wpctl`)
    - Power button (short press: sleep/wake + lock, long press >=650ms: Power Menu) and real `systemctl poweroff`/`reboot`
    - Inactivity auto-lock and pre-sleep dimming tied to `lockscreen.timeout_secs()`
    - Interactive Adreno GPU devfreq boost (`/sys/class/devfreq/*gpu*/{min_freq,polling_interval}`: boosts to max clock on touch/animation, returns to idle clock after 600ms or screen off)
  - **60 Hz Frame Pacing & GLES3 Pipeline**:
    - High-priority EGL context (`ContextPriority::High`), `zwp_linux_dmabuf_v1` v4 feedback on `/dev/dri/renderD128` (zero-copy Adreno 630 GPU buffers for Wayland clients instead of CPU `llvmpipe`), `xdg_popup` constrained menus/popovers with popup hit-testing & outside-tap dismissal, `wlr-layer-shell` + `zwp_text_input_v3` + `zwp_input_method_v2` + `zwp_virtual_keyboard_v1` + `zwlr_data_control_manager_v1` + `zwlr_foreign_toplevel_manager_v1` + `zphoc_device_state_v1` (hosting SpectreOS's Rust + `libbhoo` English/Persian `spectre-keyboard` OSK with automatic viewport resize), `dbus-update-activation-environment` sync, clean child env (`GSK_RENDERER=gl`, `NO_AT_BRIDGE=1`, `MESA_DISK_CACHE_SINGLE_FILE=1`, `XDG_SESSION_TYPE=wayland`), background pre-warming (`spectre-keyboard` + `nautilus --gapplication-service`), left/right Edge-Back gesture over Wayland apps, live Overview/Recents sync with `SIGTERM` fallback close, VBlank-gated Lock Screen wake (zero 1-frame flash), Double-Tap to Wake (DT2W), `BhooElement::opaque_regions`, hardware-VBlank phase-locked animation clock, zero-lag multi-event touch drain, upfront GPU atlas/FBO pre-warming, and `WISP_PERF=1` / `WISP_BENCH=1` VBlank telemetry.
    - Backed by `libbhoo`'s Adreno-optimized GLES3 renderer (single-pass analytical soft shadows, streaming append-only VBO orphaning, TBDR `glInvalidateFramebuffer` + scissored layer clears + opaque layer bypass, dirty-row atlas uploads, and hot-path fragment shader ordering).
- **`EmbeddedApps` trait**: keeps `wisp` independent of `bhoo-web`. Standalone compositor/window runs use `NoopEmbeddedApps`; web/capture hosts (`codes/apps/bhoo-web`, `libbhoo/examples/android_launcher_capture.rs`) plug in their XML-backed Contacts and Mail apps without circular crate dependencies.

## Integration

- **Unified device compositor binary**: `codes/apps/wisp/src/main.rs` (`cargo build --manifest-path codes/apps/wisp/Cargo.toml --release --features compositor --bin wisp`, started by `modules/wisp.nix` via `~/run-wisp.sh`).
- **Desktop preview window**: `cargo run --manifest-path codes/apps/wisp/Cargo.toml --release --features wayland --bin wisp -- --window`.
- **WebAssembly / WebXDC host**: `codes/apps/bhoo-web` depends on `wisp` and exposes it as scene `10009` (`site/launcher.html`, `bhoo-launcher.xdc`).

## Development remote_control

The optional `remote_control` module uses a private Unix socket forwarded through
SSH. Its framed JSON/H.264 protocol with PNG compatibility, session lifecycle, offscreen full-scene capture,
input validation and client tools live under `src/remote_control/`. Compositor
integration shares physical touch routing; a topmost red banner identifies every
session. Local input revokes control, while disconnect/timeout cancel contacts and
release keys. The `development` Cargo feature enables it; startup additionally
requires `WISP_REMOTE_CONTROL=1`. See [remote_control](remote_control/README.md),
[protocol](remote_control/protocol.md), and [validation](remote_control/validation.md).

The stateful host client (`src/remote_control/session.py`) multiplexes commands
with continuously received stream events over one Wisp connection. Private local
IPC lets CLI commands, ordered input batches, attached viewers and video recorders
share the session. The physical red badge is rendered after capture readback;
blue tap markers are native render elements included in both outputs. See the
[current validation](remote_control/h264_validation.md).
