# Text field

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Filled/outlined; labels, helper/error text, prefix/suffix, icons.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Control text, selection, composing range, validation and disabled/read-only states; support clipboard, IME and grapheme-aware editing.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Interpolate label position, outline and helper appearance; typed glyphs and the caret position update immediately.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

### Angular comparison implementation

The Angular comparison profile animates floating label position, scale and color
for 150 ms using `cubic-bezier(0.4, 0, 0.2, 1)`. A filled field animates its
centered active underline over 180 ms; an outlined field changes its border and
notch with focus. Hover, disabled, error and required-marker presentation use the
shared comparison palette. Interruptions continue from the current presentation,
and reduced motion snaps to the final state. The browser adapter exposes the
required state on the native input. See [Angular form-field structure](https://github.com/angular/components/blob/main/src/material/form-field/_mdc-text-field-structure.scss)
and [the comparison record](../angular-comparison.md).

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Compose mixed Persian/English text and confirm label animation never resets IME or selection.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/text-field.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/text-fields)

- [Local upstream visual example](../material-design/upstream/docs/components/images/textfield/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [filled-text-field.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/textfield/internal/filled-text-field.ts) | [Downloaded file](../../../../context/reference-sources/material-web/textfield/internal/filled-text-field.ts) |
| Material Web | [text-field.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/textfield/internal/text-field.ts) | [Downloaded file](../../../../context/reference-sources/material-web/textfield/internal/text-field.ts) |
| Material Components for Android | [TextInputLayout.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/TextInputLayout.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/textfield/TextInputLayout.java) |
| Compose Material 3 | [TextField.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt) |
| Angular Material | [input.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/input/input.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/input/input.ts) |
| Flutter material_ui | [text_field.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/text_field.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/text_field.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: text-field.md](../material-design/upstream/docs/components/text-field.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Text field preview](http://127.0.0.1:8766/#/text-field).

Demonstrates: Outlined single-line field, native browser editing and character count.

Remaining scope: Native Wayland text editing/IME and complete filled/outlined tokens.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
