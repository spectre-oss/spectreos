# libbhoo Technical Design

Status: Wayland/GLES prototype with the first stateful controls and a shared effects engine. Performance budgets, complete catalog coverage, and production readiness remain open.

## Design documents

- [Effects implementation and provenance](effects-implementation.md): radial ink, touch/keyboard input, motion schemes, menu/dialog choreography, generated loading geometry, progress indicators and exact validation limits.

- [Browser demo coverage](demo-coverage.md): all 77 live previews, precise demonstrated behavior and remaining scope.
- [Browser component explorer](web-demo.md): real Rust/WASM previews, catalog pages, local run instructions and browser validation.

- [Implementation status and source decisions](implementation-progress.md): current code, tested states, upstream references, optimization choices, and explicit remaining gaps.
- [Full implementation goal and review loop](goal.md): required in-app OK/rejection feedback, durable repair queue, documentation standards and reviewed-message cleanup.
- [Architecture and window lifecycle](architecture.md): module boundaries, application API, input ordering, scaling, and resource ownership.
- [GLES rendering and text](rendering.md): command translation, shaders, batching, font atlas, and allocation costs.
- [Validation and roadmap](validation.md): reproducible checks, their limits, performance measurements, and unresolved decisions.
- [Material Design 3 reference](material-design/README.md): local upstream component and theming documentation, with source revision and license.
- [Material 3 catalog crosswalk](material-component-crosswalk.md): official component pages, local specification mapping, open-source clones, and remaining work.
- [Component catalog and implementation references](components.md): broader inventory across Material Web, Android, Compose, Angular, and Flutter, plus proposed desktop additions and priorities.
- [Angular Material comparison lab](angular-comparison.md): real reference controls, shared settings, overlay/wipe and visual evidence.
- [Individual component specifications](components/README.md): one Markdown file per catalog family and shared foundation, including interaction and animation requirements.
- [Motion and animation design](motion.md): proposed profiles, timing tokens, interruption, reduced motion, scheduling, and validation.
- [Downloaded reference implementations](reference-implementations.md): pinned source snapshots and component implementation mappings.
- [Licenses and attribution](licenses-and-attribution.md): reviewed upstream license texts, source reuse conditions, and provenance records.

These documents distinguish implemented behavior from planned capabilities. Update them alongside changes to the corresponding implementation.

## Purpose

Bhoo is a minimal Rust GUI toolkit for the SpectreOS launcher and applications. The Cargo package is `libbhoo`, and its public Rust API uses `bhoo`.

We want a consistent visual language, responsive input, low idle resource usage, and predictable memory consumption. CUI is a reference for architectural ideas; we are developing Bhoo's implementation independently.

## Current implementation

The prototype in `codes/libs/ui/libbhoo` provides a dark theme, vertical layout, labels, pointer-operated buttons with stable IDs, and per-frame drawing commands. Tests cover basic button interaction and layout.

The launcher example opens a native Wayland window using an optional backend: winit for events, glutin for EGL and a GLES 3.0 context, glow for rendering, and fontdue for basic glyph rasterization. It renders labels and rounded buttons, supports a click counter, resizing, display scaling, and closing with Escape or a button.

The runner now supports keyboard focus/activation, wheel input, effect-driven redraws and tooltip deadlines. Initial controls include selection inputs, sliders, tabs, list rows and a virtualized list. Text commands share cached labels, geometry uses indexed quads, and GPU capacity is reused. See the implementation-status document for exact coverage; these families are not complete Material implementations. The simple glyph path is not suitable for Persian text yet, and native accessibility remains open.

## Architecture decisions

### Rust and a small public API

Use Rust for the toolkit. Keep application state with the caller and expose a small set of composable UI functions. Add components when an actual launcher or application needs them.

### Immediate-mode API with separate internal stages

Applications describe their interface when a frame is needed. Internally, separate input processing, layout, painting, and presentation. Widgets use unique, stable IDs for interaction state across frames.

A temporary layout tree is a candidate for richer layouts. We must explicitly define how hit testing handles layout changes, removed widgets, clipping, and pointer capture. CUI uses previous-frame rectangles; Bhoo has not committed to that approach.

### Event-driven frame scheduling

Wait for events when the interface is idle. Request frames for input, state changes, resize events, pending resource updates, and active animations. Schedule timers at their next deadline rather than polling continuously.

Pace animation frames to the display when supported. Stop requesting animation frames once motion settles, and support reduced motion.

### Renderer-independent drawing

Keep platform windows and graphics APIs outside widget logic. Produce an ordered draw list that a renderer can consume. Define clipping, scale factors, resource lifetimes, and alpha blending consistently across the boundary.

Batch adjacent compatible drawing commands while preserving visual order. Measure vertex formats and upload costs before choosing more complex rendering optimizations.

The first backend uses Wayland + EGL + OpenGL ES 3.0. Desktop OpenGL and X11 are not fallback paths. Rounded rectangles and glyphs share a shader pipeline and one ordered triangle batch. Glyphs use a fixed grayscale atlas, uploaded when changed and rebuilt on scale changes. Atlas paging and eviction are not implemented; exhausting the atlas returns an error.

A dedicated software renderer and partial repainting remain options to evaluate. Mesa can execute the current GLES path in software; that is distinct from implementing Bhoo's own software rasterizer.

### Reuse memory and bound caches

Reuse command and layout buffers between frames. Avoid allocating owned copies of unchanged text on every frame. Make temporary data lifetimes explicit, and keep long-lived resources separate from frame data.

Give text, image, and interaction caches clear eviction policies. Reclaim stale widget state and allow oversized buffers to shrink after sustained lower usage. Aim for low steady-state allocation counts, then verify them with profiling.

### Process visible content

Provide virtualized lists for application search results and other large collections. Build and paint the visible range, with a small overscan where needed. Clipping alone does not replace virtualization because invisible widgets can still incur layout and text work.

### Treat text as core infrastructure

Plan for Persian, mixed RTL/LTR text, font fallback, shaping, and bidirectional layout from the beginning. Displaying shaped text and correctly editing it are separate requirements.

Prefer established text infrastructure over writing a shaping engine. Cache shaped runs and rasterized glyphs with keys that account for font, size, scale, direction, and shaping settings. Keep measurement and rendering consistent.

Text editing must eventually handle cursor movement, selection, grapheme boundaries, clipboard operations, and IME composition. Specific libraries remain to be selected.

### Keyboard and accessibility

Design focus order, keyboard activation, and semantic widget information alongside pointer input. Accessibility belongs in the architecture even though the current prototype has no accessibility integration.

### Shared visual tokens

Use semantic tokens for colors, spacing, typography, radii, and motion. Keep styling separate from widget behavior. Start with a small coherent theme and add light mode and customization through the same token system.

### Linux and Wayland

SpectreOS is based on NixOS, so native Linux and Wayland integration are primary targets. The initial backend creates a normal Wayland toplevel. Layer-shell integration, launcher focus policy, and monitor placement require further design; the current window is not yet a desktop shell surface.

## Relationship to existing toolkits

The current direction is a small Bhoo core with replaceable platform and rendering integrations. We are not committing to an egui fork. We also have no benchmark showing that Bhoo is faster than egui or other toolkits.

Use a representative launcher prototype to evaluate whether an existing toolkit can meet our requirements with less maintenance. Any decision to own more infrastructure should identify the measured limitation it addresses.

## Validation and performance

Measure release builds on documented hardware and display settings. Track:

- Startup time to the first usable frame.
- Idle CPU usage and unnecessary wakeups.
- Input-to-presentation latency and frame-time percentiles during search, scrolling, and animation.
- Resident memory, toolkit allocations, cache sizes, and steady-state allocations per frame.
- Behavior after repeated resize, scale, theme, and application-list changes.

Set numerical budgets after collecting a baseline. Do not describe targets as achieved results.

Keep interaction tests for capture, cancellation, focus, and widget removal. Add layout and rendering checks, mixed Persian/English text cases, and long-running interaction tests. If incremental rendering is introduced, compare its output against full repainting to detect invalidation errors.

## Next implementation steps

1. Establish allocation, startup, idle usage, and frame-time measurements for the Wayland/GLES prototype.
2. Integrate text shaping and validate Persian display, editing, keyboard input, and focus.
3. Add a search field and virtualized application list to the launcher.
4. Define cache budgets, reduce text allocations, and add timer-driven animation scheduling.
5. Evaluate layer-shell integration for the actual launcher.

## Initial window validation

Unit tests cover interaction, input edge ordering, cancellation, motion coalescing, layout, and glyph clipping. The headless Weston test at `tools/test-bhoo-wayland.py` verifies framebuffer pixels at 1× and 2× scale through the actual Wayland/EGL/GLES path. These runs use Mesa software rendering and do not establish hardware GPU performance or input-to-display latency.

## References

- [Bhoo source and usage](../../../codes/libs/ui/libbhoo/README.md)
- CUI reference checkout: `context/CUI` (local and ignored by Git).
- [CUI source](https://github.com/yappologistic/CUI)

The CUI review covered its frame scheduler, temporary layout tree, reusable buffers, batched draw data, incremental software renderer, text limitations, and long-running performance tests. These are design inputs rather than performance results verified for Bhoo.

## Documentation Index

- [Android XML](android-xml.md)
- [Android Widget Inventory](android-widget-inventory.md)
- [Android App Tests](android-app-tests.md)
- [Material Component Crosswalk](material-component-crosswalk.md)
- [Scroll and Worker](scroll-and-worker.md)
- [Web Demo](web-demo.md)
- [WebXDC](webxdc.md)
- [Demo Coverage](demo-coverage.md)
- [Review Loop](review-loop.md)
- [Motion](motion.md)
- [Rendering](rendering.md)
- [Validation](validation.md)
- [Licenses and Attribution](licenses-and-attribution.md)
- [Tools](../../../tools/README.md)
