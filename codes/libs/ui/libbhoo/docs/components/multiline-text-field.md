# Multiline text field

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Direct manipulation](../motion.md#motion-profiles) |

## Scope and variants

Wrapping, selection, scrolling, variable height.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Extend text editing with wrapping, line navigation, scrolling and an explicit height policy.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Update editing and caret-follow scrolling immediately; optional height changes must not obscure the active line.

Use the shared [direct manipulation profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Retain immediate user-controlled movement; disable optional inertial or programmatic travel where practical.

### Angular comparison implementation

The Angular comparison uses the same 150 ms floating-label transition as the
single-line field, with a 56 px label origin even when the textarea is taller.
Focus, blur, interrupted transitions and reduced motion are sampled against the
pinned Angular runtime; multiline shaping, RTL and native IME behavior still need
independent validation. See [Angular form-field structure](https://github.com/angular/components/blob/main/src/material/form-field/_mdc-text-field-structure.scss)
and [the comparison record](../angular-comparison.md).

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Edit near the bottom while resizing and confirm the caret remains visible and composition survives.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Related Material Web guide](../material-design/upstream/docs/components/text-field.md)

- [Local upstream visual example (related pattern)](../material-design/upstream/docs/components/images/textfield/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [text-field.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/textfield/internal/text-field.ts) | [Downloaded file](../../../../context/reference-sources/material-web/textfield/internal/text-field.ts) |
| Material Components for Android | [TextInputEditText.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/TextInputEditText.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/textfield/TextInputEditText.java) |
| Compose Material 3 | [TextField.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt) |
| Angular Material | [input.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/input/input.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/input/input.ts) |
| Flutter material_ui | [text_field.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/text_field.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/text_field.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Multiline text field preview](http://127.0.0.1:8766/#/multiline-text-field).

Demonstrates: Browser multiline editing, selection and IME backed by Rust UTF-8 state.

Remaining scope: Native text layout, scrolling metrics and cross-renderer shaping parity.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
