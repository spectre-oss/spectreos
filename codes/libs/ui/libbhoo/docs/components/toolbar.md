# Toolbar

Status: Experimental Rust toolbar API and browser example; complete Material behavior remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Inline groups of application actions.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Group actions with an explicit tab/arrow navigation policy and overflow handling.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Blend per-action feedback; overflow redistribution is a deliberate layout change rather than per-frame motion.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Resize until actions overflow and verify all actions remain available and named.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/DockedToolbar.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/toolbars/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/dockedtoolbar/docked-toolbar-anatomy.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [HideViewOnScrollBehavior.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/behavior/HideViewOnScrollBehavior.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/behavior/HideViewOnScrollBehavior.java) |
| Material Components for Android | [DockedToolbarLayout.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/dockedtoolbar/DockedToolbarLayout.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/dockedtoolbar/DockedToolbarLayout.java) |
| Compose Material 3 | [AppBarRow.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBarRow.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBarRow.kt) |
| Angular Material | [toolbar.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/toolbar/toolbar.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/toolbar/toolbar.ts) |
| Flutter material_ui | [app_bar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/app_bar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/app_bar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: DockedToolbar.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/DockedToolbar.md)
- [Angular Material: toolbar/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/toolbar/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)


The `Frame::docked_toolbar_with_overflow` API draws the Android-style 64 dp standard or vibrant surface, with 16 dp side padding, weighted 48 dp minimum action slots, bundled Material icons and an optional embedded primary action. Its labels remain available to accessibility backends. The lower-level `Frame::toolbar` API also draws text actions and horizontal or vertical floating surfaces. The caller owns placement and action handling. The original API displays only actions that fit. `Frame::toolbar_with_overflow` adds an anchored menu for up to 64 actions, preserving the Android overflow layout’s original action order and indices, disabled states, keyboard navigation, label typeahead, wheel and touch scrolling, touch-fling momentum, and focus restoration. The fling uses Bhoo’s bounded exponential decay, not Android’s exact scroll physics; reduced motion disables it. It requires a 64 dp toolbar axis and room in the viewport for the menu; it reserves IDs through `id + 131`. The docked overflow carries each bundled action icon into its menu row and mirrors its position in RTL. `Frame::docked_toolbar_scroll_offset` follows positive and negative consumed scroll with 175 ms hide and 225 ms show fallback timings, supports interruption and reduced motion, and forces the bar visible when the host reports touch exploration. `Frame::docked_toolbar_with_insets` applies host-provided top and bottom insets automatically for the dock edge or via explicit overrides, extending the surface while keeping actions out of the inset. The host must supply accessibility state, content padding and resolved system insets; native accessibility integration and exact themed timing remain open. `Frame::toolbar_roving` provides one tab stop, orientation-aware arrow keys, Home/End, disabled-item skipping and RTL placement; the caller retains a `ToolbarState`. Reusing that state with the generic roving API clears the docked styling and closes an open overflow.

## Browser demonstration

[Open the live Toolbar preview](http://127.0.0.1:8766/#/toolbar).

Demonstrates: Standard and vibrant icon-based docked toolbars with Back, Forward, Add, Tabs and More actions, an embedded primary action, roving keyboard focus, RTL and narrow-width switches, an overflow menu, and a scroll-hide preview with touch-exploration and bottom-inset switches. The Android guide uses multiple possible child arrangements; the demo follows its illustrated five-action arrangement.

Remaining scope: Platform system-inset and accessibility-state integration, arbitrary overflow icons and exact overflow fling physics, platform toolbar group semantics, full tokens and exact motion. The overflow is an initial Bhoo composition; exact upstream layout and motion are not yet verified.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
