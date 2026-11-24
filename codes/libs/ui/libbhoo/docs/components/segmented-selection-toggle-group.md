# Segmented selection / toggle group

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Single and multiple selection.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Use a controlled selection set; support one or multiple choices with explicit empty-selection policy and directional keyboard movement.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Move the selection indicator between segments and blend selected colors; do not delay the selection event.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

### Angular comparison implementation

The Angular comparison profile places the selected checkmark 12 px from the
segment's leading edge. Its 18 px reveal and the label's 30 px inset use a
45 ms delay followed by a 150 ms `cubic-bezier(0.4, 0, 0.2, 1)` transition;
interrupted transitions use CSS reversal shortening. Selected and unselected
text, disabled selected surfaces, and segment dividers use the matching shared
theme colors. Reduced motion snaps to the final presentation. See [Angular button-toggle styles](https://github.com/angular/components/blob/main/src/material/button-toggle/button-toggle.scss)
and [Material 3 toggle tokens](https://github.com/angular/components/blob/main/src/material/button-toggle/_m3-button-toggle.scss).

`node tools/test-bhoo-segmented-layout.mjs` checks the displayed initial and
disabled states; `node tools/test-bhoo-segmented-motion.mjs` checks 29 timed
samples for single selection, multiple selection, deselection and reversal.
Hover and ripple layers are clipped to the rounded group outline and their own
segment, so they cannot fill the square corners outside the pill. Run
`node tools/test-bhoo-segmented-corners.mjs` to check both ends in light and
dark themes while hovered, pressed and selected. The checkmark is drawn using
Rust lines and clipped geometry; its edge rasterization, RTL and full native
accessibility still need validation.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Exercise single/multiple selection and RTL arrow order without losing the focused segment.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ToggleButtonGroup.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/segmented-buttons)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/radio/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/buttons/togglebuttongroup-anatomy.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [MaterialButtonToggleGroup.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/button/MaterialButtonToggleGroup.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/button/MaterialButtonToggleGroup.java) |
| Compose Material 3 | [SegmentedButton.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SegmentedButton.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SegmentedButton.kt) |
| Angular Material | [button-toggle.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/button-toggle/button-toggle.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/button-toggle/button-toggle.ts) |
| Flutter material_ui | [segmented_button.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/segmented_button.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/segmented_button.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: ToggleButtonGroup.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ToggleButtonGroup.md)
- [Angular Material: button-toggle/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/button-toggle/)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Segmented selection / toggle group preview](http://127.0.0.1:8766/#/segmented-selection-toggle-group).

Demonstrates: Single-choice view and independent multi-choice formatting.

Remaining scope: Expressive connected shapes and complete segmented-button semantics.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
