# Theme tokens

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Shared visual/interaction foundation |
| Priority | Shared foundation |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Color, type, shape, spacing, density, motion.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Resolve semantic color, type, shape, spacing, density and motion values; invalidation depends on token category.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Color-only transitions may interpolate; font/density changes are discrete layout invalidations by default.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Change theme during interaction and verify contrast, cache invalidation and final values agree.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Related Material Web guide](../material-design/upstream/docs/components/../theming/README.md)

- [Local upstream visual example (related pattern)](../material-design/upstream/docs/components/../theming/images/theming.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Shared infrastructure reference; these files are not standalone widgets.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [_md-sys-motion.scss](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/tokens/_md-sys-motion.scss) | [Downloaded file](../../../../context/reference-sources/material-web/tokens/_md-sys-motion.scss) |
| Material Components for Android | [MotionUtils.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/motion/MotionUtils.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/motion/MotionUtils.java) |
| Compose Material 3 | [MaterialTheme.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialTheme.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MaterialTheme.kt) |
| Compose Material 3 | [ColorScheme.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ColorScheme.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ColorScheme.kt) |
| Flutter material_ui | [theme_data.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/theme_data.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/theme_data.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Material Web theming](../material-design/upstream/docs/theming/README.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Theme tokens preview](http://127.0.0.1:8766/#/theme-tokens).

Demonstrates: Theme color-role samples with host light/dark switching.

Remaining scope: Full tonal palette, type scale, density and dynamic-color resolution.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
