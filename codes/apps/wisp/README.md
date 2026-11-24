# Wisp — SpectreOS Unified Wayland Compositor, Launcher & System UI

**Wisp** *(a faint, lightweight ghost or flicker of light — matching **SpectreOS** and **Bhoo**)* is the unified Wayland compositor, System UI, Lock Screen, and Launcher application built on [`libbhoo`](../../libs/ui/libbhoo/README.md) and Smithay.

## Features

- **Unified Wayland Compositor & Hardware Bridge (`--features compositor`)**: Runs directly on DRM/KMS + GBM + EGL/GLES3 (`src/compositor.rs`), hosting real Wayland applications discovered via Freedesktop `.desktop` entries (plus built-in fallbacks like `epiphany`, `nautilus`, `spectre-camera`/`megapixels`, `gnome-calls`, `kgx`) while directly binding launcher/System UI state to Linux hardware (battery `/sys/class/power_supply/*`, panel backlight `/sys/class/backlight/*`, flashlight/status LED `/sys/class/leds/*`, Wi-Fi/Airplane via `nmcli`, Bluetooth via `rfkill`, volume/mute via `wpctl`, Power button sleep/wake & long-press Power Menu, real `systemctl poweroff`/`reboot`, and inactivity auto-lock).
- **Freedesktop.org App & Icon Support (`src/freedesktop.rs`)**: Implements XDG Base Directory v0.8, Desktop Entry v1.5 (`.desktop` discovery, Desktop File ID deduplication, POSIX locale strings, `TryExec`, `OnlyShowIn`/`NotShowIn`, `Exec=` unquoting & field codes, `Path`, `Terminal`, `DBusActivatable`, `StartupWMClass`, `Keywords`, `[Desktop Action]`, `X-Shiraz-Package`, and live mtime hot-reload), and Icon Theme v0.13 (`index.theme` inheritance, `hicolor`/`pixmaps` fallbacks, PNG decoding + native SVG rasterization via `resvg`) directly inside the App Drawer, Overview/Recents, and Wayland app launcher.
- **Shiraz Android Compatibility Integration (`src/services/shiraz.rs`, `src/backends/linux/shiraz.rs`)**: First-class Notification Center Quick Settings tile (`Shiraz` / `Enable Shiraz`), dedicated `Settings → Shiraz` page (with live warm RAM, AOT status, installed Android apps, and optimization/sandbox controls), persistent warm `shiraz-daemon` (`mmap` + `MADV_WILLNEED` page-cache pre-warming + relocatable `dex2oat` AOT boot image & per-APK `.odex`), direct Wayland `xdg_toplevel.app_id` matching (`<pkg>.Shiraz`), Freedesktop Notifications v1.3 (`desktop-entry` + theme icon), SpectreOS Keyboard (`org.spectreos.OSK0` / `zwp_text_input_v3`), Edge-Back → `XF86Back`, and multi-touch pinch-to-zoom forwarding.
- **Home Workspace & Dock**: Multi-page app workspace, hotseat dock, drag-to-reorder, long-press app popups, App Drawer, Widgets sheet, and Wallpaper & Style picker with high-resolution `176×176` (`xxxhdpi`) icons.
- **Native Lock Screen**: Built directly on `libbhoo` (`src/lockscreen.rs`) supporting Swipe, PIN, and Password unlock, persistent lock configuration (`~/.config/wisp/lock.conf`), and configurable auto-lock timeouts.
- **System Shade & Quick Settings**: Two-stage notification shade, Quick Settings tile grid & tile editor, brightness pill slider, volume panel, and power menu.
- **Multitasking**: Overview / Recents carousel, live Wayland app switching & closing, split-screen resizable divider, and draggable/resizable freeform windows.
- **Built-in System Apps & Settings**: Full interactive Settings hierarchy (`src/settings.rs`, including Security → Screen lock PIN/Password management) plus built-in Clock, Calculator, Phone, Messaging, Browser, Gallery, Music, Camera, Calendar, Files, F-Droid, and Recorder surfaces (`src/apps.rs`).
- **Host-Pluggable External Apps**: `EmbeddedApps` trait allows standalone execution (`NoopEmbeddedApps`) or embedding full demo apps (e.g. `bhoo-web` Contacts & Gmail).

## Folder Structure

```text
codes/apps/wisp/
├── Cargo.toml             ← Unified workspace & `wisp` package (`smithay` included as local member)
├── Makefile
├── README.md
├── assets/
│   └── icons/             ← High-res 176×176 xxxhdpi Android launcher app icons
├── docs/
│   └── README.md          ← Architecture & design notes (symlinked into /docs/wisp via `make init`)
├── smithay/               ← Embedded editable Smithay Wayland compositor engine (DRM/KMS, GBM, EGL, libinput, xdg-shell)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── LICENSE.txt        ← MIT license for embedded Smithay core
│   └── src/
└── src/
    ├── lib.rs             ← Core WispShell / LauncherDemo<E>, EmbeddedApps trait, gestures & chrome
    ├── lib.spec.rs        ← 1-to-1 unit specs for lib.rs
    ├── compositor.rs      ← Integrated DRM/KMS/GLES3 Wayland compositor & HardwareBridge (`--features compositor`)
    ├── lockscreen.rs      ← Native libbhoo Lock Screen (Swipe / PIN / Password + auto-lock persistence)
    ├── settings.rs        ← Interactive System Settings subsystem (including Security → Screen lock)
    ├── apps.rs            ← Built-in system apps & card previews
    ├── freedesktop.rs     ← XDG BaseDir v0.8, Desktop Entry v1.5 & Icon Theme v0.13 (PNG + SVG)
    ├── icons.rs           ← High-res 176×176 xxxhdpi Android app icon loader + libbhoo SystemIcon bridge
    └── main.rs            ← Unified `wisp` binary (compositor by default under `compositor`, or `--window` preview under `wayland`)
```

## Quick Start

```sh
# Check library, Wayland preview, and unified DRM/KMS compositor
make -C codes/apps/wisp check

# Run unit tests
make -C codes/apps/wisp test

# Build unified DRM/KMS compositor + launcher binary
make -C codes/apps/wisp build-compositor

# Launch desktop preview window on an existing Wayland desktop
make -C codes/apps/wisp run

# Package webxdc preview (via bhoo-web host)
make -C codes/apps/wisp webxdc
```

## Slow animations for capture

In Wisp Settings, choose **Developer options → Window animation scale**, or run
`wisp --animation-scale 5` as the compositor user. On the Poco development
installation, use `~/run-wisp.sh --animation-scale 5`. The running compositor
loads the setting automatically. Run the same command with `1` to restore normal
speed, or omit the number to read the current value.

The shared scale applies to launcher and Settings transitions, heads-up motion,
and the lock fade. Gesture tracking and notification timeouts use real time.

## Development remote_control

Build with `make build_development` and run with `WISP_REMOTE_CONTROL=1` to enable
an owner-only Unix socket for SSH-forwarded screenshots, live H.264 video (with PNG compatibility), touch,
keyboard and navigation control. The phone displays a red REMOTE CONTROL banner;
physical input takes over and disconnects the controller. The banner is visible
only on the phone; blue tap markers appear on the phone and in the stream. A
persistent CLI supports an owned SSH tunnel, fresh stream frames, video recording,
remote keyboard entry and serialized command batches. Ordinary compositor
builds do not include this feature. See [setup and client commands](docs/remote_control/README.md)
and the [remote_control v1 protocol](docs/remote_control/protocol.md).
