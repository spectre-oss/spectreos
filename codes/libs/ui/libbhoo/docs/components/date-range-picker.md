# Date-range picker

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Start/end dates and range validation.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Track start/end dates, partial range and validation; define keyboard behavior and inclusive endpoint semantics.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Highlight the range without moving cell targets; animate month changes as a separate layout transition.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

### Angular comparison implementation

The browser demo uses a 296 by 354 px rounded calendar surface with the
Angular comparison theme's surface-container-high color. Committed range cells
use primary-container, with half-cell bridges at the endpoints. Selecting the
first date updates the displayed value immediately. The browser check
`node tools/test-bhoo-date-range.mjs` covers opening, choosing both dates,
reopening and Escape dismissal. See [Angular datepicker tokens](https://github.com/angular/components/blob/main/src/material/datepicker/_m3-datepicker.scss)
and [range cell styles](https://github.com/angular/components/blob/main/src/material/datepicker/calendar-body.scss).

The example remains experimental: independent editable start/end inputs, keyboard range previews, dynamic today and
full keyboard navigation have not yet been matched to Angular.

The dropdown entry scales vertically from 0.8 to 1 while fading in over 120 ms
with `cubic-bezier(0, 0, 0.2, 1)`. A settled dropdown fades out linearly over
100 ms; reduced motion snaps. The phase comparison against the pinned Angular
runtime is `node tools/test-bhoo-datepicker-motion.mjs`. Bhoo currently applies
opacity per draw command rather than compositing the whole popup as one layer,
so overlapping primitives can differ during the fade. Native GLES behavior has
not been visually certified. See [Angular datepicker animation source](https://github.com/angular/components/blob/main/src/material/datepicker/datepicker-content.scss).

For an incomplete range, hovering a later day draws a one-pixel dashed primary
outline through each day, including across week boundaries, with rounded
endpoints. Moving to an earlier day removes the preview; selecting an end date
commits the range. The preview is an immediate state change in the referenced
Angular CSS, rather than a timed animation. See [Angular range preview styles](https://github.com/angular/components/blob/main/src/material/datepicker/calendar-body.scss)
and run `node tools/test-bhoo-range-preview.mjs` to compare the two previews.
The dashed endpoint segments approximate CSS rasterization; keyboard preview
and touch dragging remain open.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Select a reversed or partially disabled range and verify validation and cancellation restore the prior value.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/DatePicker.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/date-pickers/overview/)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/datepicker/datepickers-types.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [MaterialDatePicker.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/datepicker/MaterialDatePicker.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/datepicker/MaterialDatePicker.java) |
| Compose Material 3 | [DateRangePicker.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DateRangePicker.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DateRangePicker.kt) |
| Angular Material | [date-range-picker.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/datepicker/date-range-picker.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/datepicker/date-range-picker.ts) |
| Flutter material_ui | [date_picker.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/date_picker.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/date_picker.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Date-range picker preview](http://127.0.0.1:8766/#/date-range-picker).

Demonstrates: Ordered start/end selection within a navigable month.

Remaining scope: Cross-month ranges, locale input, disabled dates and keyboard grid.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

In the Angular comparison, the calendar toggle is a separate 40px circular target. Its hover, keyboard focus and pressed state layers and ripple use the icon color; opening the calendar does not focus the date input. The toggle icon uses the primary color while the calendar is open.
