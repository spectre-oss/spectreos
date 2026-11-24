# Divider

Status: Horizontal, vertical, inset and heavy decorative dividers are implemented. Pixel-scale and Android device validation remain open.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Static](../motion.md#motion-profiles) |

## Scope and variants

Horizontal/vertical, full-width or inset.

`Frame::divider_at` draws a one-dp outline-variant line centered in a caller-supplied rectangle, with physical start/end insets. `Frame::divider_styled_at` adds configurable thickness and color, and maps horizontal logical insets for RTL; vertical insets run top to bottom. Android's regular and heavy variants use 1dp and 8dp respectively. `Frame::divider` remains the horizontal full-width layout helper.

The Android XML importer supports the fully qualified `com.google.android.material.divider.MaterialDivider` tag with explicit `app:dividerThickness`, `app:dividerColor`, `app:dividerInsetStart`, and `app:dividerInsetEnd`; `wrap_content` height uses the thickness. It recognizes Material 3 regular and heavy divider styles, their default theme attribute aliases, `?attr/colorOutlineVariant`, and local inherited styles containing these divider properties. General Material theme/style resolution remains unsupported.

## State and interaction contract

Provide horizontal/vertical and inset forms; decorative dividers are not focusable or actionable.

Decorative dividers do not accept input or create accessibility focus targets. Use `divider_styled_at` for logical horizontal leading/trailing insets.

## Motion and animation

No intrinsic animation; inherit a parent reveal without introducing a private clock.

Use the shared [static profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Remain static; the component owns no animation clock in either mode.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Verify a static divider schedules no frames and remains crisp at different display scales.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/divider.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/divider)

- [Local upstream visual example](../material-design/upstream/docs/components/images/divider/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

The pinned AndroidX [divider tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DividerTokens.kt) specify 1dp thickness and the outline-variant color.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [divider.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/divider/internal/divider.ts) | [Downloaded file](../../../../context/reference-sources/material-web/divider/internal/divider.ts) |
| Material Components for Android | [MaterialDivider.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/divider/MaterialDivider.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/divider/MaterialDivider.java) |
| Compose Material 3 | [Divider.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Divider.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Divider.kt) |
| Angular Material | [divider.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/divider/divider.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/divider/divider.ts) |
| Flutter material_ui | [divider.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/divider.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/divider.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: divider.md](../material-design/upstream/docs/components/divider.md)
- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)
