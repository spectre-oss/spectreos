# Switch

Status: Partial implementation — controlled toggle, optional icons and pressed thumb sizing; exact platform motion and device parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

On/off, label, and no-icon, selected-only or both-icon variants. The default has no icon.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose checked and disabled state with a label; Space toggles the controlled value.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Move the thumb and blend track color with a bounded selection transition; reversing retargets from the current position. The unselected handle is 16 dp without an icon, the selected or icon-bearing handle is 24 dp, and pressing expands it toward 28 dp.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Toggle repeatedly during motion and confirm the thumb ends at the actual value.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/switch.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/switch)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/switch/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/switch/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [switch.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/switch/internal/switch.ts) | [Downloaded file](../../../../context/reference-sources/material-web/switch/internal/switch.ts) |
| Material Components for Android | [MaterialSwitch.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/materialswitch/MaterialSwitch.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/materialswitch/MaterialSwitch.java) |
| Compose Material 3 | [Switch.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt) |
| Angular Material | [slide-toggle.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/slide-toggle/slide-toggle.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/slide-toggle/slide-toggle.ts) |
| Flutter material_ui | [switch.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/switch.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/switch.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: switch.md](../material-design/upstream/docs/components/switch.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)
