# Scroll area / viewport

Status: Partial implementation — nested rectangular clipping; general scroll-area routing remains open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | SpectreOS desktop additions |
| Kind | SpectreOS desktop proposal |
| Priority | P0 |
| Motion profile | [Direct manipulation](../motion.md#motion-profiles) |

## Scope and variants

Clipping, scrolling, nested wheel behavior, visibility reporting.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Own offset, extent, clipping, wheel routing and nested-scroll handoff; reveal keyboard focus when needed.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.


## Motion and animation

- **EdgeStretch overscroll**: Integrated via `EdgeStretch` (see `codes/libs/ui/libbhoo/src/scroll.rs`). Pull, release, and fling absorb follow AOSP `EdgeEffect` (`TYPE_STRETCH`) constants: `NATURAL_FREQUENCY = 24.657`, `DAMPING_RATIO = 0.98`, `LINEAR_STRETCH_INTENSITY = 0.016`, `EXP_STRETCH_INTENSITY = 0.016`, `SCROLL_DIST_AFFECTED_BY_EXP_STRETCH = 0.33`.
- **Cancel pointer input**: `Frame::cancel_pointer_input` (see `codes/libs/ui/libbhoo/src/systems/material/mod.rs`) clears UI active state, sets `input.cancelled`, cancels pending ripple taps `<150 ms`, and allows in‑flight ripples to fade `375 ms`.
- **Validation**: Same unit tests and filmstrip evidence as other components.
- **Remaining limitations**: Full Android parity for scrollbars and advanced scroll snapping not yet implemented.

Direct wheel/drag updates are immediate; optional smooth programmatic scrolling is interruptible.

Use the shared [direct manipulation profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Retain immediate user-controlled movement; disable optional inertial or programmatic travel where practical.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Interrupt programmatic scrolling with a wheel event and verify immediate control returns to the user.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Flutter ScrollView API](https://api.flutter.dev/flutter/widgets/ScrollView-class.html)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Related building blocks only; this is a SpectreOS-specific proposal, not an upstream component implementation.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Compose Material 3 | [ScrollField.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ScrollField.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ScrollField.kt) |
| Compose Material 3 | [Scrollbar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt) |
| Flutter material_ui | [scrollbar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/scrollbar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/scrollbar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Bhoo catalog and reference implementations](../components.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)
