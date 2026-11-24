# `libbhoo` Development Guide

This guide documents the architecture, folder layout, testing rules, and extension recipes for `libbhoo` (Rust crate `bhoo`).

Every directory has one clear responsibility so that adding a widget, a new design system (such as Apple HIG / Cupertino), an icon set, or a new platform renderer has one predictable home.

---

## Folder Structure

```text
codes/libs/ui/libbhoo/
├── Cargo.toml
├── Makefile                    ← Local build, test, Wayland, WASM, WebXDC & review targets (`make help`)
├── README.md
├── DEVELOPMENT.md              ← This guide
├── assets/                     ← Bundled raster assets (e.g. 9-patch / Android drawables)
├── docs/                       ← Technical design docs, component specs, motion & review records
├── examples/                   ← Native Wayland/GLES example apps & capture harnesses
│
├── src/                        ← Production code + strict 1-to-1 unit specs (`<file>.spec.rs`)
│   ├── lib.rs (+ lib.spec.rs)  ← Core engine: Ui, Frame, Input, Rect, Color, Theme, DrawCommand,
│   │                             Role, SemanticNode, public re-exports
│   ├── media.rs (+ media.spec.rs)
│   ├── motion.rs               ← Spring physics and motion schemes
│   ├── timing.rs               ← Tween and cubic-bezier easing
│   ├── scroll.rs               ← Scroll physics, spline fling, overscroll edge stretch
│   ├── ripple.rs               ← Press ink/ripple state machine
│   │
│   ├── platform/               ← OS windowing and GPU renderer backends
│   │   ├── mod.rs
│   │   ├── window.rs           ← Wayland windowing runner (feature = "wayland")
│   │   └── gles/               ← OpenGL ES 3.0 renderer (feature = "wayland")
│   │       ├── mod.rs          ← Renderer lifecycle, FBO layers, vertex flush, capture
│   │       ├── atlas.rs        ← Glyph and image texture atlas packing
│   │       ├── commands.rs     ← DrawCommand execution loop
│   │       ├── fonts.rs        ← System font discovery, weight/italic selection, TextMetrics
│   │       ├── geometry.rs     ← Affine2D transforms and quad/shape/polyline mesh builders
│   │       └── shaders.rs      ← GLSL ES 3.00 vertex & fragment shaders
│   │
│   ├── icons/                  ← Bundled vector & raster icon catalogs + decoders
│   │   ├── mod.rs
│   │   ├── material_symbols.rs (+ material_symbols.spec.rs)
│   │   ├── material_system.rs  (+ material_system.spec.rs — 59 Material Outlined RLE4 icons)
│   │   ├── material_navigation.rs
│   │   ├── material_card.rs
│   │   ├── android_search.rs
│   │   └── android_time_picker.rs
│   │
│   ├── shapes/                 ← Auto-generated non-icon vector shapes (never hand-edited)
│   │   ├── mod.rs              ← Shared shape_data(index) lookup across shapes & vector icons
│   │   ├── beercss_shapes.rs   (+ beercss_shapes.spec.rs)
│   │   ├── loading_shapes.rs
│   │   └── android_clock_shapes.rs
│   │
│   └── systems/                ← Design systems & platform UI compatibility layers
│       ├── mod.rs
│       ├── material/           ← Material Design 3 / Bhoo native widgets (1 file per family)
│       │   ├── mod.rs          ← Module declarations, re-exports, shared control helpers
│       │   ├── app_bar.rs (+ app_bar.spec.rs)
│       │   ├── avatar.rs (+ avatar.spec.rs)
│       │   ├── badge.rs (+ badge.spec.rs)
│       │   ├── card.rs (+ card.spec.rs)
│       │   ├── menu.rs (+ menu.spec.rs)
│       │   ├── navigation.rs (+ navigation.spec.rs)
│       │   ├── overlay.rs (+ overlay.spec.rs)
│       │   ├── scrollbar.rs (+ scrollbar.spec.rs)
│       │   ├── transitions.rs (+ transitions.spec.rs)
│       │   └── ... (button, fab, fab_menu, chip, slider, switch, tabs, toolbar, etc.)
│       └── android/            ← Android UI compatibility layer
│           ├── mod.rs
│           ├── reference.rs    ← Static Android NativeNode resource-tree bridge
│           ├── state/          ← Pure-Rust Android widget state machines (1 file + 1 .spec.rs each)
│           │   ├── mod.rs
│           │   ├── calendar.rs (+ calendar.spec.rs)
│           │   ├── date_spinner.rs (+ date_spinner.spec.rs)
│           │   ├── dialer_filter.rs (+ dialer_filter.spec.rs)
│           │   ├── expandable_list.rs (+ expandable_list.spec.rs)
│           │   ├── number_picker.rs (+ number_picker.spec.rs)
│           │   ├── time_clock.rs (+ time_clock.spec.rs)
│           │   └── time_spinner.rs (+ time_spinner.spec.rs)
│           └── xml/            ← Android XML layout, drawables, solvers & painters
│               ├── mod.rs, state.rs, specs.rs, drawables.rs, text.rs, helpers.rs,
│               ├── measure.rs, render.rs, draw.rs, draw_adapter_views.rs,
│               └── draw_containers.rs, draw_widgets.rs, constraint_layout.rs, relative_layout.rs
│
└── tests/                      ← Integration & cross-module tests (mirrored by area)
    ├── allocations.rs          ← Standalone heap-allocation test (uses #[global_allocator])
    ├── core/                   ← Core cross-widget / frame integration tests (`main.rs` + modules)
    ├── material/               ← Cross-component Material composition tests (`main.rs` + modules)
    └── android/                ← End-to-end Android XML fixture & layout tests (`main.rs` + modules)
```

---

## Where Does New Code Go?

| What you are adding | Where it goes |
| --- | --- |
| Core input, focus, `DrawCommand`, animation, or scroll primitive | `src/` root (`lib.rs`, `motion.rs`, `scroll.rs`, etc.) |
| A new Material / Bhoo widget family | `src/systems/material/<widget>.rs` + `src/systems/material/<widget>.spec.rs` |
| A new Android widget state machine | `src/systems/android/state/<widget>.rs` + `<widget>.spec.rs` |
| Android XML layout, drawable, or widget painting | `src/systems/android/xml/` |
| A new design system (e.g. Apple HIG / Cupertino, Fluent, Spectre) | `src/systems/<system>/` (e.g. `src/systems/apple/`) |
| A new bundled icon set (vector or raster) | Generator in `tools/generate-*.py` → `src/icons/<name>.rs`, registered in `src/icons/mod.rs` (and `src/shapes/mod.rs::shape_data` if vector `ShapeData`) |
| A new bundled non-icon vector shape set | Generator in `tools/generate-*.py` → `src/shapes/<name>.rs`, registered in `src/shapes/mod.rs` |
| A new windowing or GPU renderer backend (e.g. Vulkan, Metal) | `src/platform/<backend>/` |
| **Unit tests** for a single source file `<name>.rs` | `<name>.spec.rs` right beside `<name>.rs` (**exact same filename stem**) |
| **Integration tests** spanning multiple modules, XML fixtures, or public API flows | `tests/<area>/` (`tests/core/`, `tests/material/`, `tests/android/`, future `tests/apple/`) |
| Component specifications, motion docs, validation & review records | `docs/` (see [docs/components/README.md](docs/components/README.md)) |

---

## Testing Guidelines: Unit Specs (`*.spec.rs`) vs. Integration Tests (`tests/`)

### 1. Unit tests (`<name>.spec.rs` — strict 1-to-1 naming rule)

Unit tests live directly next to the source file they test using the **exact same filename stem** plus `.spec.rs` (like Angular's `.spec.ts`):

```text
src/systems/material/
├── card.rs
└── card.spec.rs
```

**Strict rules:**
- **Every `<name>.spec.rs` MUST have a matching `<name>.rs` in the same folder.** Never create an orphan spec like `states.spec.rs` without `states.rs`, or split multiple differently named `.spec.rs` files for one `.rs` file.
- If a single source file has several unit-test groups (for example `app_bar.rs` covering top & bottom app bars, or `navigation.rs` covering bar, rail, and chrome), group them as submodules (`mod top_app_bar { ... }`, `mod bottom_app_bar { ... }`) inside the **one** matching `app_bar.spec.rs` / `navigation.spec.rs`.
- Wire `<name>.spec.rs` at the bottom of `<name>.rs`:
  ```rust
  #[cfg(test)]
  #[path = "<name>.spec.rs"]
  mod tests;
  ```
- Exception for auto-generated files in `src/shapes/` and `src/icons/`: wire `beercss_shapes.spec.rs` from `src/shapes/mod.rs` and `material_symbols.spec.rs` from `src/icons/mod.rs` so re-running `tools/generate-*.py` does not overwrite the `mod` declaration (they still keep the exact matching `<name>.spec.rs` filename).
- Small core modules with only a few private unit tests (`motion.rs`, `timing.rs`, `ripple.rs`, `scroll.rs`, `platform/window.rs`, `platform/gles/mod.rs`) may keep an inline `#[cfg(test)] mod tests { ... }` block.
- `src/lib.rs` declares `#[cfg(test)] extern crate self as bhoo;`, so both `crate::*` and `bhoo::*` work inside unit specs.

### 2. Integration tests (`tests/`)

Put tests in **`tests/`** whenever they test **integration across multiple modules/components**, load **external XML layout fixtures** (e.g. `bhoo-web` generated layouts), or require a **separate test binary** (such as `#[global_allocator]`):

- `tests/allocations.rs` — standalone binary with a custom `#[global_allocator]`
- `tests/core/` (`main.rs` + modules) — multi-widget core engine, state, ripple/hover, and text-metrics integration tests
- `tests/material/` (`main.rs` + modules) — cross-component Material composition & wave suites
- `tests/android/` (`main.rs` + modules) — end-to-end Android XML fixture, layout container, and widget integration tests
- *(Future design systems add `tests/<system>/main.rs`, e.g. `tests/apple/`)*

### Running tests

```sh
# Run everything (all unit specs + all integration suites)
cargo test --locked --all-features

# Run only unit tests (all *.spec.rs + inline unit tests)
cargo test --locked --all-features --lib

# Filter unit specs by module path
cargo test --locked --all-features --lib systems::material::card
cargo test --locked --all-features --lib systems::android::state::calendar
cargo test --locked --all-features --lib icons::
cargo test --locked --all-features --lib shapes::

# Run a specific integration test suite
cargo test --locked --test core
cargo test --locked --test material
cargo test --locked --test android
cargo test --locked --test allocations
```

---

## Step-by-Step Recipes

### 1. Adding a new widget to `systems/material/`
1. Create `src/systems/material/<widget>.rs` with its option/state types and `impl Frame<'_>` methods.
2. Create `src/systems/material/<widget>.spec.rs` (exact same name) and wire it at the bottom of `<widget>.rs`:
   ```rust
   #[cfg(test)]
   #[path = "<widget>.spec.rs"]
   mod tests;
   ```
3. Declare `mod <widget>;` and re-export its public types in `src/systems/material/mod.rs` (and in `src/lib.rs` if exposed at crate root).
4. If the widget needs multi-component or fixture-based integration tests, add a module under `tests/material/` and register it in `tests/material/main.rs`.
5. Add or update its component specification under `docs/components/`.
6. Verify with `cargo test --locked --lib systems::material::<widget>`.

### 2. Adding a new design system (e.g. Apple HIG / Cupertino)
1. Create `src/systems/apple/mod.rs` and register `pub mod apple;` in `src/systems/mod.rs`.
2. Add one file + matching `<widget>.spec.rs` per component family inside `src/systems/apple/` (e.g. `button.rs` + `button.spec.rs`, `sheet.rs` + `sheet.spec.rs`, `theme.rs`).
3. If needed, add cross-component integration tests in `tests/apple/main.rs`.
4. Keep shared physics (`motion.rs`, `timing.rs`, `scroll.rs`) and rendering (`DrawCommand`, `platform/gles/`) design-system-agnostic; if Apple needs new rendering primitives (such as backdrop blur / vibrancy), add a `DrawCommand` variant in `src/lib.rs` and implement it in `src/platform/gles/` (and `codes/apps/bhoo-web/site/canvas-painter.js` for web demos).
5. If Apple needs bundled vector or raster icons, add a generator in `tools/` outputting to `src/icons/` and register it in `src/icons/mod.rs` (and `src/shapes/mod.rs::shape_data` if using `ShapeData`).

### 3. Adding bundled icons or shapes
1. Add or update the generator script in `tools/generate-*.py` to emit icons into `codes/libs/ui/libbhoo/src/icons/<name>.rs` or non-icon shapes into `codes/libs/ui/libbhoo/src/shapes/<name>.rs`.
2. Declare the module in `src/icons/mod.rs` or `src/shapes/mod.rs`, and register any vector `ShapeData` index range in `src/shapes/mod.rs` (`shape_data`).
3. Record license and attribution details in the repo-root `THIRD_PARTY_NOTICES.md` (`../../../../THIRD_PARTY_NOTICES.md`) and `docs/licenses-and-attribution.md`.

---

## Validation & Quality Checks

From `codes/libs/ui/libbhoo/`:

```sh
cargo check --locked --no-default-features
cargo check --locked --all-features --examples
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
```

From the repository root:

```sh
# Symlink project docs into root docs/
make init

# Check the browser/WASM demo crate against libbhoo
cargo check --manifest-path codes/apps/bhoo-web/Cargo.toml --locked

# Headless Wayland/Weston pixel test
python3 tools/test-bhoo-wayland.py
```
