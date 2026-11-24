# Effects and animation implementation

Status: implemented and tested foundations, not a declaration that the entire component catalog is production complete. The [coverage matrix](demo-coverage.md) retains family-specific gaps.

## Implemented paths

| Behavior | Runtime implementation | Reference |
| --- | --- | --- |
| Rounded, pointer-origin ink | `ripple.rs`; one soft radial draw command; Canvas gradient and GLES SDF | Pinned Material Web ripple |
| Keyboard press ink | Center origin, Space hold/release, Enter short press | Material Web input lifecycle |
| Primary touch ink | 150ms pending deadline; quick taps start ink on release; cancellation before reveal removes it | Material Web touch-delay state machine |
| Standard/Expressive geometry | Analytic damped springs, preserving sampled position and velocity on retarget/profile changes | AndroidX Standard/ExpressiveMotionTokens |
| Menu surface and item stagger | 500ms enter / 150ms exit; 35%-100% height; separate surface/item opacity | Material Web menu implementation |
| Dialog scrim/surface/sections | 500ms enter / 150ms exit; translate/grow; staged headline/content/actions | Material Web dialog animations |
| Bottom/side sheets | Directional geometry from the bottom/trailing edge; immediate logical dismissal | Bhoo spatial adaptation of sheet guidance |
| Expressive loading | Seven exact matched upstream shape paths; indeterminate, determinate, contained/uncontained | MaterialShapes and LoadingIndicator |
| Linear progress | Two independent head/tail segments, rounded caps and track gaps | Compose ProgressIndicator |
| Circular progress | Rounded arcs, track gap, independent rotation/progress timelines | Compose ProgressIndicator |
| Selection/disclosure/content | Animated selected color, switch/tab geometry, chevron rotation, clipped expansion, manual carousel travel, notice reveal | Shared effects/spatial scheme and per-family specifications |

## Ink timing and rendering

The source reference is [Material Web ripple.ts at the pinned revision](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/ripple/internal/ripple.ts), its [styles](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/ripple/internal/_ripple.scss), and [motion curves](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/internal/motion/animation.ts).

- Growth: 450ms, CSS cubic-bezier(0.2, 0, 0, 1), inverted by bounded bisection.
- Initial diameter: floor(20% of the larger control dimension), with a one-pixel guard for degenerate controls.
- Target diameter: diagonal + 10px + max(35% of the larger dimension, 75px).
- Origin travels from the pointer position toward the center; keyboard and label presses outside a compact affordance use its center.
- Press opacity: 12%, 105ms linear fade-in; minimum visible press: 225ms; release fade: 375ms linear.
- Soft edge: the pinned radial-gradient stop calculation, clipped to the actual rounded control shape.
- Primary touch begins on a 150ms deadline. A short tap begins immediately on release, while a canceled pending touch produces no ink. Secondary touch pointers do not steal capture.
- Disabled, removed, blurred and reduced-motion owners remove their ink. Cancellation ends feedback without activating the control. A new press replaces that owner's old lifetime.

Buttons (including icon buttons, chips and action tiles), list/menu rows, checkboxes, switches, radios, tabs and slider endpoints share the same ink system. Tab parts have distinct internal keys. Control semantics and callbacks are independent of drawing.

A held ripple stops requesting frames after growth/opacity settle. Release resumes rendering only for its finite fade. The ink cache is hard-bounded at 1024 lifetimes and prunes unseen owners every frame. No ripple textures, threads, filesystem cache or independent timers are used. GLES reuses the existing indexed quad format and batch; radial parameters occupy the existing UV/mode fields.

This is Material bounded ink shared by the native and web backends. Android `RippleDrawable`, vendor timing differences and Android sparkle/noise shaders are separate platform rendering behavior; those are not claimed as reproduced pixel-for-pixel.

## Motion schemes and duration choreography

The web inspector selects Standard or Expressive. Color/opacity uses critical damping; geometry uses the selected spatial profile. Parameters come from the pinned [Standard tokens](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt) and [Expressive tokens](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt).

| Profile | Standard damping / stiffness | Expressive damping / stiffness |
| --- | --- | --- |
| Fast spatial | 0.9 / 1400 | 0.6 / 800 |
| Default spatial | 0.9 / 700 | 0.8 / 380 |
| Slow spatial | 0.9 / 300 | 0.8 / 200 |
| Default effects | 1.0 / 1600 | 1.0 / 1600 |

Switch/thumb and tab-indicator travel use fast spatial motion. Disclosures and sheet travel use default spatial motion. Direct manipulation follows input without a trailing spring. Spatial output is bounded to valid normalized geometry; the internal oscillator preserves velocity. Changing scheme samples the old spring before applying the new profile.

Menus/dialogs use duration-based curves from the pinned Web implementation, rather than substituting a generic spring. Menu item entry uses 250ms opacity, staggered through the 500ms entrance. Exit reverses item order with 50ms opacity and stagger/delay within the 150ms exit. Dialog headline/content enter after 50ms over 200ms; actions enter after 150ms over 150ms. Headline/content exit over 100ms; actions over 50ms. Surface opacity enters over 50ms and exits in the last 50ms. Scrim opacity is independent, up to 32%.

Use `OverlayKind`, `Frame::overlay_item` and `Frame::overlay_section` to select this choreography. Generic custom overlays do not infer a headline or action layout. Layout remains stable while presentation is clipped to the growing/shrinking surface. Hit testing uses presented geometry; exiting content becomes noninteractive immediately. Reversals preserve sampled opacity/geometry rather than jumping back to an endpoint. Modality, Escape dismissal, disabled state and focus restoration are not delayed until animation completion. Native Escape is routed to transient UI or an active capture before the window-close shortcut. Plain tooltips suppress re-entry after Escape until their owner loses hover/focus.

The duration reference is [menu.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/menu/internal/menu.ts) and [dialog animations.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/dialog/internal/animations.ts). These are intentionally distinct from the selectable Compose spring schemes.

Springs, duration tracks and visible-only clocks share one 4096-entry bound and are reclaimed when not referenced. Delayed touch/tooltip/notice lifetimes use the window's earliest deadline. Idle and settled scenes return to waiting. Reduced motion snaps effects and removes continuous decorative travel while retaining actual semantic progress/state.

## Loading geometry and provenance

[LoadingIndicator.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/LoadingIndicator.kt) defines the sequence SoftBurst, Cookie9Sided, Pentagon, Pill, Sunny, Cookie4Sided and Oval. The determinate path is the rotated Circle-to-SoftBurst pair. Recipes come from [MaterialShapes.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialShapes.kt).

`tools/BhooShapes.java` adapts those recipes and exports matched cubic endpoints through AndroidX graphics-shapes-desktop 1.1.0. The small generated Rust table includes the exact curve correspondence, not hand-drawn approximations or raster frames. Regenerate with a JDK and Python:

```sh
python3 tools/generate-bhoo-loading.py
```

The generator downloads about 3MB of pinned tooling into ignored `context/build/shape-export`, verifies SHA-256 hashes and emits `src/shapes/loading_shapes.rs`. It also obtains the 297ms reference spring duration from FloatSpringSpec (damping 0.6, stiffness 200, visibility threshold 0.1). Loading morphs start every 650ms, with 90-degree morph travel and 4666ms global rotation. The Rust runtime evaluates the oscillator analytically, holds the completed shape between morph starts, and streams transformed cubics without per-frame path/morph allocation.

Canvas renders Bézier curves directly. GLES uses a bounded tessellation of 12 subdivisions per matched cubic, reuses CPU/GPU buffers, clips triangles on the stack, and adds an exterior physical-pixel coverage fringe. Tests establish closed paths, finite/bounded geometry and the star-shaped kernel required by this specialized fill. This is not a general arbitrary SVG path engine, nor a claim of identical antialiasing between renderers.

`LoadingState::pause` excludes hidden time. The native runner also reconciles state through a visibility generation on occlusion, and the browser pauses on hidden documents/documentation tabs. Reduced motion freezes the busy shape and stops the loop. Determinate progress continues to expose its actual value. Transient ink and queued pointer edges are canceled on hiding, so a held press cannot reappear on resume. No JVM dependency, downloaded image collection, icon font or animation asset bundle enters the runtime.

## Progress indicators

[Compose ProgressIndicator.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ProgressIndicator.kt) supplies linear head/tail delays and durations: a 1750ms cycle, first head/tail 1000ms with 0/250ms delay and second head/tail 850ms with 650/900ms delay. The emphasized-accelerate curve is sampled in Rust. Inactive track segments retain gaps; determinate progress has a stop indicator.

Circular progress uses the pinned 6000ms sweep/rotation cycle, 1500ms additional-rotation spacing with 300ms transitions, 0.1–0.87 progress range and rounded caps. Determinate indicators use a critically damped progress spring (stiffness 50). GLES evaluates arc distance and caps analytically; Canvas strokes the same arc command.

The current API covers the four basic linear/circular determinate/indeterminate forms. Buffered/wavy variants and full component token customization remain separate catalog work.

## Verification and limits

- Deterministic Rust tests cover finite lifetimes, short taps, touch delay/cancellation, keyboard origin, tab-part ownership, disable/reduce removal, CSS curve endpoints, spring sampling/reversal, mixed cache bounds, hidden-time exclusion, shape topology, progress cycles and overlay exit input rejection.
- The warmed unchanged-control allocation test remains in place.
- Browser checks operate actual WASM controls and verify radial growth/hold/fade, finite idle scheduling, touch feedback, changing cubic paths, reduced-motion freeze, all 77 pages and mobile layout.
- `tools/test-bhoo-wayland.py` checks actual GLES pixels in isolated Weston at 1×/2×, including rounded ink containment, radial footprint, loading-path fill and circular stroke/empty center.

These checks are not a full upstream screenshot-diff suite, hardware frame-time qualification, assistive-technology bridge, or completion of every component variant. Family-specific remaining scope remains visible in the catalog. Licenses and attribution are retained in [THIRD_PARTY_NOTICES.md](../../../../../THIRD_PARTY_NOTICES.md).
