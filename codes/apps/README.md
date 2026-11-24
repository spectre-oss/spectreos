# SpectreOS Apps (`codes/apps/`)

See the [SpectreOS app authoring guide](../../docs/writing-spectreos-apps/README.md) for app structure, responsive launch, background work, and on-device validation.

Applications and interactive demos built on [`libbhoo`](../libs/ui/libbhoo/README.md):

| App | Path | Purpose |
| --- | --- | --- |
| **Wisp** | [`wisp/`](wisp/README.md) | Unified SpectreOS Wayland Compositor, System UI, Lock Screen, Freedesktop Notifications Service, and Launcher (DRM/KMS + Smithay compositor & Linux hardware bridge, Home workspace, App Drawer, Notification Shade, Quick Settings, Overview/Split-Screen/Freeform multitasking, built-in apps). Runs natively on device (`--features compositor`), in a desktop preview window (`--features wayland`), or embedded in WebAssembly. |
| **SpectreOS Keyboard** | [`keyboard/`](keyboard/README.md) | Default standalone SpectreOS on-screen keyboard (`spectre-keyboard`, `org.spectreos.Keyboard`) built in Rust with `libbhoo` GLES 3.0 rendering, English (`en-US`), Persian (`fa-IR` with ZWNJ half-space), Terminal, and Emoji layouts, adaptive touch scoring, Wayland (`zwlr_layer_shell_v1`, `zwp_input_method_v2`, `zwp_virtual_keyboard_v1`), and Freedesktop D-Bus (`org.spectreos.OSK0`). |
| **SpectreOS Settings** | [`settings/`](settings/README.md) | Extracted Android 13 / AOSP-style System Settings app (`spectre-settings`) with high-resolution `96×96` Material Symbols icons, illustrated Settings figures, live hardware telemetry, Freedesktop notification & lock-screen privacy settings, and live `~/.config/wisp/` sync. Runs standalone on Wayland (`--features wayland`) or embedded inside Wisp / `bhoo-web`. |
| **SpectreOS Files** | [`files/`](files/README.md) | Native file manager and XDG file chooser (`spectre-files`) supporting local directories, mounts, Trash, recursive search, image previews, and ZIP archive creation/extraction. Runs standalone on Wayland (`--features wayland`) or embedded inside Wisp. |
| **SpectreOS Calendar** | [`calendar/`](calendar/README.md) | Native Wisp calendar with local calendars, day/week/month/agenda views, recurring events, reminders, and iCalendar import and export. |
| **Bhoo Web Demos** | [`bhoo-web/`](bhoo-web/README.md) | Interactive WebAssembly demo apps: Bhoo component landing & playground, Angular Material side-by-side comparison, single-canvas gallery, and Android XML demos (Chat, Gmail, Contacts, and Wisp launcher web host). |
