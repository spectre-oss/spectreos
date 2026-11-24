# Bhoo

Minimal GUI foundations for SpectreOS, written in Rust. The package is `libbhoo`; the Rust API is `bhoo`.

Inspired by CUI's immediate-mode approach. The core provides controlled widgets, shared pointer/keyboard state, focus, bounded motion/effects, cached labels, semantic snapshots, clipping, fixed-height list virtualization, and Android XML layout/widget support. The optional `wayland` feature adds a native Wayland window and an OpenGL ES 3.0 renderer.

Run from this directory (run `make help` for all build, test, web, and WebXDC targets):

```sh
make help
make test
make run-launcher
make run-components
make webxdc-gallery
```

The example opens a window with a working click counter and a close button. Escape also closes it. Run it inside an active Wayland desktop session. This backend requires Linux, a Wayland compositor, EGL with OpenGL ES 3.0 or newer, libxkbcommon, and a TTF/OTF font. It does not fall back to X11 or desktop OpenGL.

If a default font cannot be found, supply one explicitly (useful on NixOS):

```sh
BHOO_FONT=/absolute/path/to/font.ttf cargo run --release --features wayland --example launcher
```

## Project Layout & Development Guide

`libbhoo` is organized so every subsystem has one predictable home and scales cleanly across multiple design systems (Material, Android, and future systems such as Apple HIG / Cupertino) and platform backends:

```text
codes/libs/ui/libbhoo/
├── src/
│   ├── lib.rs, motion.rs, timing.rs, scroll.rs, media.rs, ripple.rs
│   │     → Core UI engine (Ui, Frame, Input, DrawCommand, physics)
│   ├── platform/       → OS windowing (window.rs) & GPU renderer (gles/)
│   ├── icons/          → Bundled icon catalogs (Material Symbols, 59 Material system icons, Android icons)
│   ├── shapes/         → Auto-generated non-icon vector shapes + shape_data()
│   └── systems/        → Design systems & UI compatibility layers
│       ├── material/   → Material Design 3 / Bhoo native widgets (1 file per family)
│       └── android/    → Android state models (state/), XML runtime (xml/), reference bridge
├── tests/
│   ├── allocations.rs  → Standalone #[global_allocator] heap-allocation test
│   ├── core/           → Core cross-module integration tests
│   ├── material/       → Cross-component Material integration tests
│   └── android/        → End-to-end Android XML fixture integration tests
├── docs/               → Technical design docs, component specs, motion & review records
├── examples/           → Native Wayland/GLES demos & capture harnesses
└── DEVELOPMENT.md      → Full architecture, folder rules, testing rules & extension recipes
```

Unit tests live next to their source file with the **exact same filename stem** (`card.rs` + `card.spec.rs`); multi-module and fixture-based integration tests live in `tests/`. Read [DEVELOPMENT.md](DEVELOPMENT.md) for the full folder guide, test commands, and step-by-step instructions for adding widgets, shapes, backends, or new design systems.

## Window and renderer

- `winit`: Wayland window, events, physical/logical coordinates, and redraw notifications.
- `glutin` / `glutin-winit`: EGL surface and explicitly requested GLES 3.0 context with VSync.
- `glow`: GLES calls and custom GLSL ES 3.00 shaders (`src/platform/gles/shaders.rs`).
- `fontdue`: glyph rasterization into a fixed grayscale atlas (`src/platform/gles/atlas.rs`, `fonts.rs`).
- `bytemuck`: safe conversion of vertex floats to upload bytes.

Normal callbacks return `FrameAction::Wait`; the runner requests more frames while core effects are active, or waits for input/tooltip deadlines when settled. Use `FrameAction::Redraw` for additional application-owned animation, or `FrameAction::Exit` to submit the final frame and close. CPU/GPU buffers retain capacity, labels use a bounded shared-text cache, and text/geometry share one indexed draw batch with premultiplied alpha. The glyph atlas is uploaded only when changed and rebuilt on scale changes.

## Components & Documentation

Read the technical documentation under [docs/](docs/README.md), including:

- [Development Guide](DEVELOPMENT.md) — folder structure, colocated `.spec.rs` tests, and adding new widgets or design systems.
- [Implementation Progress](docs/implementation-progress.md) — tested states, source references, optimization evidence, and remaining variants.
- [Component Catalog](docs/components/README.md) — per-family specifications, tokens, and motion records.
- [Goal & Review Workflow](docs/goal.md) — overall Android UI and Material Design 3 parity goal.

This is an ordinary Wayland toplevel, not yet a layer-shell launcher surface. Text currently uses individual glyph advances: Persian/Arabic shaping, bidi, font fallback, native text editing, and IME remain future work. Native screen-reader integration, full widget variants, atlas paging/eviction, and partial repainting remain ongoing. A full atlas produces an error rather than silently corrupting text.

## Validation

```sh
cargo test --all-features
cargo test --test allocations
cargo clippy --all-targets --all-features -- -D warnings
```

From the repository root, run the headless graphics test with Weston installed:

```sh
python3 tools/test-bhoo-wayland.py
```

It starts isolated Weston sessions at 1× and 2× scale and checks launcher/gallery framebuffer pixels, disabled fill, text and clipping, including the gallery with reduced motion. It uses Mesa software rendering for repeatability; it is not a hardware performance benchmark.

Dev/test builds omit debug information and incremental caches; release strips symbols and uses thin LTO. Use `cargo clean --profile dev` to reclaim debug/test build artifacts while retaining release output.

For a diagnostic screenshot inside a Wayland session:

```sh
BHOO_CAPTURE=/tmp/bhoo.ppm cargo run --features wayland --example launcher -- --smoke-test
```

Keep `Ui` between frames and supply pointer press/release edges to `Ui::frame`, or reuse a command buffer with `Ui::frame_into`. Build widgets each frame with unique, stable widget IDs. Application state stays with the caller.

Licensed under the [GNU GPL v3](../../../../LICENSE).
