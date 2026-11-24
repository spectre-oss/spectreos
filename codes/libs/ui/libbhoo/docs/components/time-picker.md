# Time picker

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Clock and text-entry modes; locale-aware formatting.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Control hours/minutes and locale presentation; provide keyboard/text input equivalent to any clock face.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Move clock selection only for discrete choices; direct hand dragging follows the pointer immediately.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

### Angular comparison implementation

The demo uses a 256 px surface with 8 px vertical padding and 48 px options,
showing five half-hour choices at a time. The active option uses the Angular
focus color; pointer and keyboard selection, wheel scrolling and Escape are
supported. Entry scales vertically and fades over 120 ms; exit fades over
100 ms. Run `node tools/test-bhoo-time-picker.mjs` and
`node tools/test-bhoo-time-picker-motion.mjs` to compare the pinned Angular
reference, including 13 motion samples. See [Angular timepicker styles](https://github.com/angular/components/blob/main/src/material/timepicker/timepicker.scss)
and [option styles](https://github.com/angular/components/blob/main/src/material/core/option/option.scss).

The scrollbar is drawn by Bhoo and supports wheel scrolling and pointer thumb dragging. Selected custom rows now report their state in the Rust semantic tree for pointer and keyboard selection. The browser mirrors visible options into a named listbox with selection and keyboard controls in the playground and both one-canvas paths. Direct text editing, trigger semantics, screen-reader device testing and complete accessibility parity remain open.
The two comparison panels maintain independent selected values.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Switch input mode during an incomplete edit and verify no silent time conversion or data loss.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/TimePicker.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/time-pickers/overview/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/timepicker/timepicker-types.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [MaterialTimePicker.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/timepicker/MaterialTimePicker.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/timepicker/MaterialTimePicker.java) |
| Compose Material 3 | [TimePicker.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt) |
| Angular Material | [timepicker.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/timepicker/timepicker.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/timepicker/timepicker.ts) |
| Flutter material_ui | [time_picker.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/time_picker.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/time_picker.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: TimePicker.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/TimePicker.md)
- [Angular Material: timepicker/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/timepicker/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Time picker preview](http://127.0.0.1:8766/#/time-picker).

Demonstrates: Filled trigger and scrollable half-hour options in 12-hour format.

Remaining scope: Minute-level entry, arbitrary intervals, locale formatting and complete listbox keyboard behavior.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
