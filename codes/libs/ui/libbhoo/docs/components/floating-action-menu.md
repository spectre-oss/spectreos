# Floating action menu

Status: Experimental Rust FAB menu with staggered actions, input dismissal and reduced motion. Full Material motion and accessibility integration remain incomplete.

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Expandable group of floating actions.

`Frame::fab_menu` draws a 56dp FAB with up to eight caller-owned labeled actions above it. `Frame::fab_menu_with` adds logical RTL placement and a clipped scrollable action region when the supplied area is short. It uses 56dp action height, 4dp item spacing and an 8dp trigger gap. The caller owns open and scroll state.

## State and interaction contract

Track expanded state and active action; Escape collapses and restores focus to the trigger.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Reveal the action surface from its anchor; any item staggering must not delay input availability.

The trigger rotates and changes shape, while action width and opacity reveal from the trailing side with a 45ms stagger. This is an initial Bhoo approximation, not a claim of exact Compose choreography. Functional state changes do not wait for exit visuals.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Dismiss halfway through opening and verify no invisible action remains focusable.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/FloatingActionButtonMenu.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/fab-menu/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/fabmenu/fabmenu-expressive.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

The pinned AndroidX [FAB menu tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FabMenuBaselineTokens.kt) specify the trigger and item dimensions. The [Compose implementation](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButtonMenu.kt) supplies interaction and motion reference.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Compose Material 3 | [FloatingActionButtonMenu.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButtonMenu.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButtonMenu.kt) |
| Angular Material | [menu.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/menu/menu.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/menu/menu.ts) |
| Flutter material_ui | [floating_action_button.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/floating_action_button.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/floating_action_button.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: FloatingActionButtonMenu.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/FloatingActionButtonMenu.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Floating action menu preview](http://127.0.0.1:8766/#/floating-action-menu).

Demonstrates: A 56dp FAB expanding labeled actions above the trigger with staggered reveal, action selection and Escape dismissal.

Remaining scope: Bidirectional text shaping, platform expanded-state semantics, precise icons/colors/elevation and exact expressive choreography.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
