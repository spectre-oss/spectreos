# State layer / ripple

Status: Material bounded radial ink implemented in Rust, Canvas 2D and GLES. Platform-specific Android sparkle rendering is not claimed.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Shared visual/interaction foundation |
| Priority | Shared foundation |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Hover, focus, press and selected feedback.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Derive hover, focus, press and selection feedback from the owning control; never capture input itself.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.


## Motion and animation (updated)

- **Cancel pointer input**: `Frame::cancel_pointer_input` (see `codes/libs/ui/libbhoo/src/systems/material/mod.rs`) is invoked when a parent scroll intercepts during a touch‑hold. It clears `ui.active`, sets `input.cancelled`, cancels pending ripple taps under 150 ms, and allows any already‑bloomed ripples to fade out over **375 ms**. This ensures ripple feedback does not erroneously fire on dragged items and matches Android behavior.
- **Validation**: Covered by `launcher_demo::drag_tests` (`settings_one_second_hold_then_drag_down_shows_ripple_without_selecting`) and associated unit tests.
- **Remaining limitations**: Full Android `RippleDrawable` pixel‑perfect parity and hardware‑accelerated sparkle rendering are still open.

Run a bounded press effect and fade state layers; reduced motion replaces ripple expansion with a static tint.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Cancel a press or remove the owner and verify the visual state terminates without emitting actions.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/ripple.md)
- [Material design guidance linked by upstream](https://m3.material.io/foundations/interaction/states/state-layers)

- [Local upstream visual example](../material-design/upstream/docs/components/images/ripple/hero.gif)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Shared infrastructure reference; these files are not standalone widgets.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [ripple.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/ripple/internal/ripple.ts) | [Downloaded file](../../../../context/reference-sources/material-web/ripple/internal/ripple.ts) |
| Material Components for Android | [RippleDrawableCompat.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/ripple/RippleDrawableCompat.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/ripple/RippleDrawableCompat.java) |
| Compose Material 3 | [Ripple.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Ripple.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Ripple.kt) |
| Angular Material | [ripple.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/core/ripple/ripple.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/core/ripple/ripple.ts) |
| Flutter material_ui | [ink_ripple.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/ink_ripple.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/ink_ripple.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: ripple.md](../material-design/upstream/docs/components/ripple.md)
- [Angular Material source](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live State layer / ripple preview](http://127.0.0.1:8766/#/state-layer-ripple).

Demonstrates: Pointer-origin radial ink, rounded clipping, hold/release fade, keyboard-centered ink, cancellation and reduced motion.

Remaining scope: Android platform RippleDrawable/sparkle shader pixel parity and hardware/device qualification.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML runtime state-layer details

In [`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../../src/systems/android/xml/mod.rs):

- **Bounded & unbounded `<ripple>` (`RULE_RIPPLE_COLOR = 209`) with zero-alpha suppression:** `paint_state_layer` applies explicit `<ripple>` and `app:rippleColor` tints (including unbounded circular ripples and asymmetric `RULE_CORNER_RADII = 206` clipped passes). When the resolved ripple color has zero alpha (`color.3 == 0`, e.g., `<ripple android:color="#00000000">` used on Contacts v2 `scrim_bg.xml`, `dialog_card.xml`, and `popup_menu_card.xml`), `paint_state_layer` returns early without painting hover or press ripples so clickable scrims and overlay cards block underlying input without flashing ink.
- **Neutral clickable container state layer:** Clickable Android XML container views (`container_clickable`) default their state-layer tint to neutral `t.text` rather than `t.primary`, matching Material neutral ink on clickable rows and cards. See [Android XML support](../android-xml.md).
