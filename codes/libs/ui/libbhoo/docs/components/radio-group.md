# Radio group

Status: Partial implementation — radio choice, directional group layout and roving group navigation; device parity remains open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Mutually exclusive choices; group keyboard model. The directional group API mirrors indicators, labels and horizontal arrow direction in right-to-left layouts.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Own one selected value in a named group; arrows move selection and focus, respecting disabled options.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

The selected dot grows from zero over 300 ms with Material Web’s emphasized-decelerate curve; its opacity and the ring color transition over 50 ms. The ring is transparent inside, so it works over non-background surfaces. Disabled and reduced-motion indicators update immediately.

Motion follows the pinned Material Web reference for the core dot and ring transition. Platform-specific interruption and device parity remain to be compared.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Navigate a group with disabled items and confirm only one option is selected.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/radio.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/radio-button)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/radio/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/radio/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [radio.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/radio/internal/radio.ts) | [Downloaded file](../../../../context/reference-sources/material-web/radio/internal/radio.ts) |
| Material Components for Android | [MaterialRadioButton.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/radiobutton/MaterialRadioButton.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/radiobutton/MaterialRadioButton.java) |
| Compose Material 3 | [RadioButton.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt) |
| Angular Material | [radio.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/radio/radio.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/radio/radio.ts) |
| Flutter material_ui | [radio.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/radio.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/radio.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: radio.md](../material-design/upstream/docs/components/radio.md)
- [Angular Material: radio/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/radio/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)
