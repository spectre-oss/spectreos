# Button

Status: Partial implementation — button variants and shared interaction; elevated shadow and full token/variant coverage remain open; radial ink is implemented. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Text, outlined, filled, tonal, elevated; optional leading icon.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose enabled, hovered, pressed, focused and disabled states; activate on a valid pointer release or Enter/Space, never on cancellation.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Blend state-layer colors without changing layout; an optional press treatment must preserve the hit target.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Disable or remove a captured button before release and verify no action fires.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/button.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/buttons)
- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/CommonButton.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/buttons/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/button/hero.webp)
- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/buttons/commonbutton-expressive.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [filled-button.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/button/internal/filled-button.ts) | [Downloaded file](../../../../context/reference-sources/material-web/button/internal/filled-button.ts) |
| Material Web | [button.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/button/internal/button.ts) | [Downloaded file](../../../../context/reference-sources/material-web/button/internal/button.ts) |
| Material Components for Android | [MaterialButton.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/button/MaterialButton.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/button/MaterialButton.java) |
| Compose Material 3 | [Button.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt) |
| Angular Material | [button.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/button/button.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/button/button.ts) |
| Flutter material_ui | [filled_button.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/filled_button.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/filled_button.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

Android XML Button uses centered labels when no gravity is specified, matching the pinned Android 15 [Material button style](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/res/res/values/styles_material.xml), local `context/reference-sources/android-framework/res/values/styles_material.xml` (Apache-2.0). Explicit logical start/end gravity follows layout direction. For full Android XML `Button`/`MaterialButton` variant (`RULE_BUTTON_VARIANT = 229`), Wave 17A/17B chamfered `cut` `ShapeAppearance` (`RULE_CORNER_FAMILY = 278`), centered single-line overhang slack, and validation details, see [Android XML button](android-button.md) and [Android XML support](../android-xml.md). Device appearance and motion parity remain open.

- [Material Web: button.md](../material-design/upstream/docs/components/button.md)
- [Material Android: CommonButton.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/CommonButton.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)
