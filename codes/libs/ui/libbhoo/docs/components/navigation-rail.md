# Navigation rail

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Layout](../motion.md#motion-profiles) |

## Scope and variants

Leading-edge navigation for wider layouts.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose destinations, selected value and optional labels; keep the focused destination stable across compact/expanded layouts.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Interpolate rail width and label opacity; indicator movement uses selection motion.

Use the shared [layout profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Apply final geometry immediately and retain the same stable focus/selection identity; no sliding or size tween.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Expand the rail with a keyboard-focused destination and preserve focus and content space.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/NavigationRail.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/navigation-rail/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/navigationrail/navigation-rail-hero.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [NavigationRailView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigationrail/NavigationRailView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/navigationrail/NavigationRailView.java) |
| Compose Material 3 | [NavigationRail.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationRail.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationRail.kt) |
| Compose Material 3 | [WideNavigationRail.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/WideNavigationRail.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/WideNavigationRail.kt) |
| Flutter material_ui | [navigation_rail.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/navigation_rail.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/navigation_rail.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: NavigationRail.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/NavigationRail.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Navigation rail preview](http://127.0.0.1:8766/#/navigation-rail).

Demonstrates: The Android expressive reference's Alarm, Clock, Timer and Stopwatch destinations, a menu button that expands the rail, and an Add timer FAB. The bundled Google Material Icons use filled and outlined variants. The Rust API also supports badges, disabled destinations, RTL and selected-label-only behavior. Selection takes effect immediately; indicator and rail geometry animate using Bhoo motion and snap under reduced motion. Labels remain in the Rust semantic snapshot when visually hidden.

The preview also composes a modal expanded expressive rail from the shared overlay and expressive destinations. It has a menu close control, timer FAB, content-width selected indicator and a consistent surface. A scrim blocks background interaction, Escape or a scrim click dismisses it, focus stays inside while open and returns to the trigger on dismissal. The rail enters from the leading edge, including RTL. This is an initial composition, not the complete Compose modal rail or a separate platform window.

The demo follows the Android guide's [expressive illustration](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/navigationrail/navigation-rail-expressive.png): a 96 dp collapsed rail, 64 dp items, 4 dp spacing, 44 dp top margin, a 40 dp header gap, and a content-width expanded indicator. The 56 × 32 dp collapsed indicator follows the Android guide. The prior 80 dp rail remains available in the Rust API. The bundled icon sources and their licenses are documented in [third-party notices](../../../../../../THIRD_PARTY_NOTICES.md).

The expanded rail uses a 240 dp width where space permits. `expressive_navigation_rail` animates width, item bounds, indicator, icon, and label positions using Bhoo's spatial spring. The demo samples the same transition for the menu position and timer FAB width, and fades the FAB label as it expands. Interrupted transitions reverse from the current geometry, and reduced motion snaps to the requested layout. The menu icon and destination label alignment still switch during the transition; these are approximations, not Android motion parity.

Remaining scope: Full modal and expressive variants, exact header/FAB motion and menu-icon morphing, scrolling, adaptive scaffold placement, exact Material colors and motion, and platform accessibility bridging. Android uses dynamic color schemes; this demo uses the selected Bhoo theme. This implementation does not claim pixel parity.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
