# SpectreOS Settings (`codes/apps/settings/`)

Android 13 / AOSP-style System Settings application for SpectreOS built directly on [`libbhoo`](../../libs/ui/libbhoo/README.md).

## Features
- Full 26-page Android 13 Material You Settings hierarchy with collapsing toolbars, Search container-transform transition, and dual-page Shared-Axis X push/pop animations.
- High-resolution `96×96` Material Symbols Outlined icons inside tinted circular category badges, plus vector-illustrated hero figures (Lock Screen Notification preview, System Navigation phone diagram, live Battery & Storage usage cards, and real Linux About Phone hardware telemetry).
- Live bidirectional persistence via `~/.config/wisp/settings.conf` and `~/.config/wisp/lock.conf` so standalone `spectre-settings` and Wisp stay synced instantaneously.
- Full Freedesktop Notification settings (`Notifications on lock screen`: Show all / Hide silent / Hide sensitive content / Don't show; `Sensitive notifications`; `Wake screen for notifications`; `Hide silent in status bar`; `Notification dots`; `Do Not Disturb`; live `Notification history`; and D-Bus `org.freedesktop.Notifications.Notify` test trigger).

## Build & Run
- Library check (used by Wisp & `bhoo-web`): `cargo check --manifest-path codes/apps/settings/Cargo.toml`
- Standalone Wayland binary: `cargo run --manifest-path codes/apps/settings/Cargo.toml --release --features wayland --bin spectre-settings`
- Open directly to Notifications page: `spectre-settings --page notifications`
