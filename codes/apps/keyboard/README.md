# SpectreOS Keyboard (`spectre-keyboard`)

Default standalone on-screen keyboard for SpectreOS, written in Rust and rendered with [`libbhoo`](../../libs/ui/libbhoo/README.md).

## Features

- **Pure Rust + `libbhoo` GLES 3.0 rendering**: Crisp vector keycaps, key-press preview popups, long-press alternate character pickers, Shift/Caps Lock indicators, and footer controls at any fractional DPI (`wp_fractional_scale_v1` + `wp_viewporter`).
- **Built-in Layouts**:
  - **English (`us` / `en-US`)**: 1080×777 measured QWERTY, Shift/Caps Lock, Symbols (`?123`), Extended Symbols (`=\<`), long-press top-row digits (`1..0`), and emoji picker.
  - **Persian (`ir` / `fa-IR`)**: 11-column measured Persian layout (`ض..ج`, `ش..گ`, `ظ..چ`), dedicated ZWNJ half-space (`◀┆▶`), Persian digits and punctuation (`۱..۰`, `،`, `؛`, `؟`, `٪`), and long-press alternates (`آ أ إ ء ٱ`, `ئ ي ى`, `ة ۀ هٔ ه‍`, `ك`, `ؤ`), shaped with HarfRust/Shabnam via `libbhoo`.
  - **Terminal (`terminal`)**: Adds an `Esc / Tab / Ctrl / Alt / ← ↓ ↑ →` control row above QWERTY (auto-selected on `zwp_input_method_v2` terminal content purpose).
  - **Emoji Picker**: Built-in categorized emoji grid with Unicode grapheme-cluster erase support.
- **Adaptive Touch Model**: Faithful Rust port of SpectreOS's Gaussian plateau + English prefix prior touch scorer (`src/touch.rs`), automatically disabled on password/PIN fields or when `SPECTRE_ADAPTIVE_TYPING=0`.
- **Standard Wayland OSK Protocols** (natively supported by [Wisp](../wisp/README.md) and other wlroots/Smithay compositors):
  - `zwlr_layer_shell_v1` (bottom-anchored `Overlay` surface with `exclusive_zone` so Wisp automatically resizes apps above the keyboard, and `KeyboardInteractivity::None` so taps never steal focus).
  - `zwp_input_method_v2` (activate/deactivate, `commit_string`, `delete_surrounding_text`, content purpose/hint, surrounding text).
  - `zwp_virtual_keyboard_v1` (evdev control keys, modifiers, and dynamic Unicode XKB fallback when no `text-input-v3` session is active).
- **Freedesktop D-Bus Session Service** (`docs/freedesktop/dbus/dbus-specification.md`):
  - Primary bus name: `org.spectreos.OSK0`
  - Object path: `/org/spectreos/OSK0`
  - Interface: `org.spectreos.OSK0`
    - `SetVisible(b visible)`
    - `SetLayout(s layout)` (`"us"`, `"ir"`, `"terminal"`)
    - Property `Visible` (`b`, readwrite, emits `org.freedesktop.DBus.Properties.PropertiesChanged`)
    - Property `ActiveLayout` (`s`, read, emits `org.freedesktop.DBus.Properties.PropertiesChanged`)
  - Implements standard `org.freedesktop.DBus.Peer`, `org.freedesktop.DBus.Introspectable`, and `org.freedesktop.DBus.Properties`.
  - Also exposes a best-effort compatibility alias at `sm.puri.OSK0` (`/sm/puri/OSK0`) unless `--no-compat-dbus` is passed, so third-party Linux mobile apps interoperate seamlessly.

## Build & Run

```bash
# Run unit tests
cargo test

# Build Wayland OSK daemon (default feature: osk)
cargo build --release

# Run Wayland OSK daemon (connects to WAYLAND_DISPLAY and session D-Bus)
./target/release/spectre-keyboard

# Run interactive desktop preview window (requires preview feature)
cargo run --features preview -- --preview
```

## Controlling Visibility over D-Bus

```bash
# Show keyboard
gdbus call --session --dest org.spectreos.OSK0 --object-path /org/spectreos/OSK0 \
  --method org.spectreos.OSK0.SetVisible true

# Hide keyboard
gdbus call --session --dest org.spectreos.OSK0 --object-path /org/spectreos/OSK0 \
  --method org.spectreos.OSK0.SetVisible false

# Switch to Persian layout
gdbus call --session --dest org.spectreos.OSK0 --object-path /org/spectreos/OSK0 \
  --method org.spectreos.OSK0.SetLayout ir

# Query current visibility
gdbus call --session --dest org.spectreos.OSK0 --object-path /org/spectreos/OSK0 \
  --method org.freedesktop.DBus.Properties.Get org.spectreos.OSK0 Visible
```
