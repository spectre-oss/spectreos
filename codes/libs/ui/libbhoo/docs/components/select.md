# Select

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Filled/outlined; keyboard selection and popup list.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Track selected value, open state and active option; support type-ahead, arrows, Enter and Escape.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Animate the anchored option surface and trigger feedback, preserving the anchor during scale/resize changes.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Cancel an open select and verify the previous value and trigger focus are preserved.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/select.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/menus/overview#b1704de4-94b7-4177-9e96-b677ae767cb4)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/select/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [select.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/select/internal/select.ts) | [Downloaded file](../../../../context/reference-sources/material-web/select/internal/select.ts) |
| Material Components for Android | [MaterialAutoCompleteTextView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/MaterialAutoCompleteTextView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/textfield/MaterialAutoCompleteTextView.java) |
| Compose Material 3 | [ExposedDropdownMenu.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt) |
| Angular Material | [select.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/select/select.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/select/select.ts) |
| Flutter material_ui | [dropdown_menu.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/dropdown_menu.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/dropdown_menu.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: select.md](../material-design/upstream/docs/components/select.md)
- [Angular Material: select/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/select/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Select preview](http://127.0.0.1:8766/#/select).

Demonstrates: Profile trigger, selectable popup, arrow navigation and Escape.

The long-option switch demonstrates that the trigger clips its label and selected value before the trailing arrow. The same field geometry reserves icon space for calendar and other popup fields.
Navigating the open menu changes the highlighted option; the selected value changes only when an option is activated, and Escape preserves the prior value.
The current option has a leading checkmark, and reopening the menu focuses that option.
Printable keys select matching options while the trigger is focused and the menu is closed; while open they move the menu highlight. Arrow, Home and End keys also open the menu at the selected, first or last item as appropriate.

Remaining scope: Complete filled/outlined field anatomy, comprehensive type-ahead and platform parity.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
