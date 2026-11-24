# Scrollbar

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Data and application patterns |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Direct manipulation](../motion.md#motion-profiles) |

## Scope and variants

Visible position, thumb interaction, hover behavior.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose scroll position/range and thumb dragging; provide keyboard and track-click behavior where supported.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Thumb position follows scroll immediately; optional hover visibility uses feedback motion.

Use the shared [direct manipulation profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Retain immediate user-controlled movement; disable optional inertial or programmatic travel where practical.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Drag near both ends and verify exact clamping and no eased lag between thumb and content.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Flutter Scrollbar API and samples](https://api.flutter.dev/flutter/material/Scrollbar-class.html)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Compose Material 3 | [Scrollbar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt) |
| Flutter material_ui | [scrollbar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/scrollbar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/scrollbar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Scrollbar preview](http://127.0.0.1:8766/#/scrollbar).

Demonstrates: Vertical draggable thumb and wheel-driven virtualized content.

The vertical example preserves the pointer's position within the thumb when a drag starts. A press on the track moves by one viewport; Page Up and Page Down do the same, while Home and End move to the bounds. The thumb length follows the visible-content fraction with a 32 dp minimum. Dragging follows the pointer directly and clamps to the scroll range. These interactions are a Bhoo implementation informed by the related scrollbar references above; exact platform track styling and touch physics have not been compared on a device.

The example also offers a separate non-interactive Material scroll indicator, based on the pinned Compose Material 3 reference. It supports vertical and horizontal orientation, logical RTL placement, and an always-visible option. Its default thumb is 4 dp thick, at least 24 dp long, at most 90% of the inset track, and uses the outline color at 70% opacity. It is hidden when content fits. With fading enabled it appears when the offset changes, holds for 400 ms, and fades over 250 ms; a further scroll restarts the hold. Reduced motion snaps it hidden after the hold. This indicator does not intercept scrolling input.

Validation: `scrollbar_drag` exercises anchored dragging, track paging, pointer cancellation, Page Up/Down, end bounds and interruption of a fling; `scroll_indicator` checks orientation, RTL placement and horizontal touch direction, bounds and fade timing. Focused browser tests exercise the examples at phone width.

Remaining scope: Touch expansion, nested-scroll routing, complete Material visual tokens and Android/device comparison.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
