# Implementation Status and Source Decisions

Status: first control and motion implementation pass. The 77-entry catalog is not complete, and this prototype is not yet a production OS toolkit. This page records code that exists separately from the broader design requirements.

## Implemented code

| Area | Current behavior | Remaining requirements |
| --- | --- | --- |
| Buttons | Neutral, filled, tonal, outlined, text and elevated color treatments; enabled/disabled, hover, press, keyboard focus, Enter/Space activation, cancellation | Radial ink is implemented; elevated shadow and full shape/size token parity remain open |
| Icon button | Separate glyph presentation and semantic action label | Icon resource pipeline and complete size/variant validation |
| Checkbox | Binary and unchecked/checked/mixed APIs, disabled/cancelled states, animated mark/fill, semantic value | Error variant, full visual/token and native accessibility validation |
| Switch | Caller-owned value, immediate logical update, animated track/thumb selection, keyboard activation | Full thumb/icon/press-shape variants and token parity |
| Radio | Individual choice and one-Tab-stop group; arrows/Home/End skip disabled choices; selection motion | RTL-aware arrow policy and full visual/semantic platform validation |
| Slider | Direct pointer drag, keyboard stepping/Home/End, clamping, disabled state and value semantics | Ticks, labels/value bubbles, RTL and all Material sizes |
| Range slider | Two focusable endpoints, direct drag and bounded keyboard stepping without crossing | Coincident-thumb pointer policy, range announcements and value bubbles |
| Tabs | Pointer/keyboard single selection, moving indicator and distinct child semantic parts | Overflow, content-panel associations, disabled individual tabs and RTL |
| Chip | Selected/disabled tonal action primitive | Separate assist/filter/input/suggestion contracts and removal affordance |
| List item | Pointer/keyboard activation, selected/disabled presentation and feedback | Rich leading/trailing content and complete list semantics |
| Virtualized list | Calls the row builder only for the visible range; wheel scrolling, keyboard selection/reveal, clipped rows | Variable row heights, overscan, model-mutation anchoring, virtualization-aware accessibility and exact scrollbar integration |
| Progress | Determinate linear values and visible-only indeterminate movement; static busy treatment with reduced motion | Circular/expressive paths and progress announcements |
| Tooltip | Hover deadline, immediate keyboard-focus eligibility, opacity transition, deferred drawing | Rich interactive tooltip policy and monitor-aware placement |
| Badge/divider | Static text badge and horizontal divider | Parent semantic integration and complete visual variants |
| Panel/viewport | Fixed-size grouping, nested clipping, same-frame clipped hit testing | General layout engine, scroll routing, flexible cards/surfaces |
| Text | Cached owned labels and existing glyph renderer | Shaping/bidi, line layout, fallback, editing, clipboard and IME |

Entry points are in [widgets.rs](../src/systems/material/mod.rs), with shared input/state in [lib.rs](../src/lib.rs), the [Wayland runner](../src/platform/window.rs), and the [GLES renderer](../src/platform/gles/mod.rs).

The [component gallery](../../../codes/libs/ui/libbhoo/examples/components.rs) exercises a subset of these controls and a 100,000-row model. It builds only visible rows, rather than creating 100,000 widgets. It does not demonstrate the unimplemented catalog families.

## State and keyboard decisions

Application values remain caller-owned. A shared interaction path handles capture, cancellation, disabled controls, focus and activation. Enter activates once on key press; Space activates once on key release; repeated Enter key events do not retrigger. Tab/Shift+Tab use the previous frame's enabled, visible focus order. Hidden, removed or disabled captured controls cannot activate on a stale release.

Native input preserves separate press/release and key snapshots. Adjacent motion-only events coalesce without overwriting keyboard/scroll edges. Focus loss clears capture and focus; pointer leave cancels capture. The event queue is not yet hard-bounded, and burst latency remains to be measured.

`SemanticNode` records role, name, focus, selected/checked/value state, bounds and enabled state. A component ID plus a part number distinguishes composite children from neighboring controls. This is an internal semantic model, not an AT-SPI/native screen-reader bridge.

## Source references and reuse record

The downloaded sources were inspected locally. Per-family [reference implementation links](reference-implementations.md) remain the detailed source index. The following references directly informed this implementation pass:

| Topic | Pinned source | Bhoo decision |
| --- | --- | --- |
| Switch state and activation | [Material Web switch](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/switch/internal/switch.ts) | Keep the logical value independent of visual progress; explicit disabled and keyboard activation handling |
| Checkbox controlled state | [Flutter checkbox](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/checkbox.dart) | Separate binary and tri-state APIs; caller owns the value |
| Motion model | [Compose MotionScheme](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MotionScheme.kt) | Shared property motion with explicit effects behavior, rather than a timer for each widget |
| Effects spring tokens | [StandardMotionTokens](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt) | Standard effects damping 1.0 and stiffness 1600; fast effects stiffness 3800 |
| Keyboard conventions | [WAI interaction patterns](https://www.w3.org/WAI/ARIA/apg/patterns/) | Button activation, radio traversal and slider adjustment are adapted to native input |

The standard effects token values are attributed in [motion.rs](../src/motion.rs) and the repository [third-party notices](../../../../../THIRD_PARTY_NOTICES.md). Their origin is Copyright 2024 The Android Open Source Project, Apache-2.0; the [license text](reference-licenses/compose-material3/LICENSE.txt) is retained. Token names are adapted to Bhoo's API. The analytic spring solver is independently written, not copied from Compose's animation implementation.

This pass does not claim pixel, accessibility or animation conformance with every upstream variant. The catalog's per-component gaps remain applicable until verified and explicitly closed.

## Motion implementation

The dependency-free [motion module](../src/motion.rs) evaluates normalized critically damped effects analytically from elapsed monotonic time. It does not integrate an Euler solver once per display frame. Retargeting samples the current presentation and retains velocity. Output is bounded to [0,1], and settled properties snap exactly to their target.

Motion state is keyed by widget/property, capped at 4096 records, and pruned when the owner/property is absent from a frame. Overflow snaps new properties to their target. Reduced motion snaps active effects and replaces indeterminate movement with static busy feedback. Springs for expressive spatial motion are not implemented.

The window runner requests another paced redraw for active effects only. Tooltip reveal uses the earliest monotonic deadline and `WaitUntil`, not a busy loop. After effects settle and no deadline/input remains, it returns to `Wait`. Native desktop preference discovery and window-occlusion tracking remain future work; reduced motion is currently application-configured through `WindowOptions`.

## Performance and storage choices

- No new Cargo dependencies, bundled fonts, images, SDKs or vendored framework source were added for these controls and effects.
- Labels use shared `Arc<str>` commands and a cache capped at 1024 entries / 128 KiB of text. Cache misses and dynamic caller formatting can still allocate; unused text is reclaimed periodically.
- Commands, semantic snapshots, focus order and CPU vertex buffers retain capacity. The static-control allocation test verifies zero core allocations over 120 warmed frames in its specific scene, not all workloads or driver activity.
- Quads now use four 56-byte vertices and six 32-bit indices: 248 geometry bytes instead of the previous 336 per quad, before capacity overhead. The renderer uploads vertex contents into retained GPU capacity; index data changes only when its capacity grows.
- Nested clips crop geometry/UVs on the CPU while preserving rounded-rectangle local coordinates. This retains one ordered draw batch for the currently supported primitives.
- Dev/test profiles disable debug information and incremental build caches. Release strips symbols and uses thin LTO with one codegen unit. Build artifacts remain ignored and can be cleaned independently of source and reference documents.

No hardware latency, idle-CPU, RSS or GPU benchmark has been established. Static allocation results and geometry byte counts must not be presented as those measurements.

The measured stripped gallery executable for this Linux build is **2,007,104 bytes** (about 1.91 MiB), excluding system libraries, drivers and the external font. A ready-to-run local copy is retained at ignored `context/bhoo-components`; its screenshot is `context/bhoo-components.png`. Build directories are disposable and are cleaned after validation to avoid retaining hundreds of megabytes of intermediate artifacts. The executable size is a build-specific observation, not an ABI or cross-platform size guarantee.

## Validation

Run from the repository root:

```sh
cargo test --manifest-path codes/libs/ui/libbhoo/Cargo.toml --all-features
cargo clippy --manifest-path codes/libs/ui/libbhoo/Cargo.toml --all-targets --all-features -- -D warnings
python3 tools/test-bhoo-wayland.py
cargo run --manifest-path codes/libs/ui/libbhoo/Cargo.toml --release --features wayland --example components
```

Tests cover clock sampling, retargeting, settling, reduced motion, removed motion owners, cache limits, keyboard press/release, cancellation, disabled controls, tri-state/binary values, radio navigation, slider bounds, clipped hit testing, virtualized row counts, tooltip deadlines and semantic part identities. The warmed allocation test uses a counting allocator on its test thread.

The current all-features run passes 29 tests: 14 module tests, 14 public state/interaction tests and one allocation test. Formatting and Clippy with warnings denied also pass. These checks cover the implemented subset; they are not a certificate for unimplemented states or the entire component catalog.

The Weston test checks launcher and gallery framebuffer pixels at 1× and 2× scale, including the gallery with reduced motion. It verifies static presentation, text, disabled fill and clipping through GLES/Mesa. It does not inject real pointer/keyboard input or validate intermediate animation frames on a physical display.

## Remaining delivery scope

The remaining catalog families are still planned. In particular: editable search/text, Persian shaping/bidi, IME, clipboard, the native accessibility bridge, menus/select/autocomplete, modal focus management, dialogs/sheets, date/time pickers, tables/trees, application activation and shell surface integration require further implementation and dedicated validation.

Correct text/input and accessibility infrastructure should precede expanding the collection with superficially similar painted controls. Every component needs its missing states and source-guideline checks closed before being marked complete.

## Browser catalog expansion (2026-11-09)

All 77 entries have a live Rust scene. The 59 additional scenes are experimental application compositions rather than 59 completed exported widget APIs. Shared composition helpers support rows, drawing, editable-field boundaries, modal focus/dismissal, direct manipulation and a fixed-cell virtualized grid. The browser hosts native HTML editing/IME and file selection adapters; no JavaScript widget-state replicas or runtime dependencies were added. See [coverage](demo-coverage.md) and [run/validation](web-demo.md).

Standard spatial spring parameters (damping 0.9 / stiffness 700) are adapted from the same pinned Apache-2.0 AndroidX token source as the effects values. The independently written analytic solver now handles underdamping as well as critical damping. This initial expansion used Standard profiles; the subsequent [effects work](effects-implementation.md) adds the Expressive scheme and Web menu/dialog choreography. Production geometry, semantics, platform integrations and several exact effect variants remain documented work.

## Radial ink and animation expansion (2026-11-09)

The [effects implementation record](effects-implementation.md) is the current provenance and verification source. Implemented paths now include pointer/keyboard/primary-touch radial ink with rounded clipping and finite lifetimes; Standard/Expressive geometry; typed menu/dialog timing and item/section staging; directional sheet travel; seven upstream matched loading shapes plus determinate loading; linear/circular progress; rotating disclosure indicators, carousel travel and reversible notice reveal. The browser and GLES command renderers both consume the new primitives. These changes complete those foundational mechanisms, not the full 77-family production specification.
