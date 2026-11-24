# Card

Status: Experimental reusable Rust card surface with passive and actionable variants. Full Material tokens, disabled content styling and accessibility integration remain incomplete.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Filled, outlined, elevated; optional interaction.

`Frame::card` provides filled, outlined and elevated surfaces, caller-provided content, and optional action behavior. `Frame::card_with_state` adds caller-controlled checked and dragged treatments, including the bundled Material Android checked-circle drawable at the logical top end. `Frame::card_with_dragged` remains available. The card processes child targets before its own action so a nested action does not also activate its container. Passive cards add no focus target.

The Android XML importer initially supports `com.google.android.material.card.MaterialCardView` with the Material 3 filled, outlined and elevated style aliases, frame-like children, `android:clickable` and `android:checkable`. It also supports physical `contentPadding` and per-side overrides, rounded-card compat shadow insets, literal/resource and state-list background, foreground, stroke, ripple and checked-icon colors, stroke width, corner radius, `RULE_CLIP_TO_OUTLINE = 234` child clipping, Wave 17A/17B chamfered (`cut`) and mixed `ShapeAppearance` corners (`RULE_CORNER_FAMILY = 278`, rendering anti-aliased cached `cut_shape_images` fills/checked/dragged/state-layer overlays and miter-inset `DrawCommand::Line`/`Arc` outlines; verified by `wave17a_cut_corners_shape_appearance_fill_outline_and_widgets` in [`tests/android_xml.rs`](../../tests/android_xml.rs) and `test_wave17b_cut_corner_family_rule_278` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py)), elevation, and checked icon size, margin and gravity, including local style overrides. A limited set of Material color attributes resolves against the live Bhoo theme. Elevation and ripple rendering use Bhoo approximations, not exact Android rendering. As in AndroidX CardView, ordinary `android:padding` is ignored. The host controls checked and dragged values through `LayoutState.checked` and `LayoutState.view_dragged`; Android does not automatically toggle checked state on click. Non-rounded compat padding, other custom card attributes, general Material themes, and full Android measurement remain open.

## State and interaction contract

Separate a noninteractive grouping surface from a selectable or actionable card; avoid conflicting nested actions.

The caller owns card content, action and checked state; checking is not toggled automatically. Actionable cards expose a button target; disabled cards disable nested targets and dim default labels. Their container and outline colors follow the pinned Compose Material 3 disabled tokens; explicit content colors remain caller-controlled. Android-specific disabled appearance and complex nested interactive behavior still need validation.

## Motion and animation

Animate state layers for interactive cards; elevation changes are optional and must not trigger layout. The checked icon uses Material Android's linear 100 ms fade-in and 50 ms fade-out for Material 3, shortened when interrupted; the checked foreground changes with the state. Android's checked foreground takes priority over the dragged foreground when both states are active; an enabled outlined checked card uses the secondary-color stroke.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Activate a nested card action and verify the card's outer action does not also fire.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Card.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/cards/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/cards/cards-types.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

AndroidX references: [filled tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledCardTokens.kt), [outlined tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/OutlinedCardTokens.kt), and [elevated tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ElevatedCardTokens.kt).

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [MaterialCardView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/card/MaterialCardView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/card/MaterialCardView.java) |
| Compose Material 3 | [Card.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Card.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Card.kt) |
| Angular Material | [card.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/card/card.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/card/card.ts) |
| Flutter material_ui | [card.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/card.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/card.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: Card.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Card.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Card preview](http://127.0.0.1:8766/#/card).

Demonstrates: Filled, outlined and shadowed surfaces with selection.

Remaining scope: Complete elevation/interaction tokens and media anatomy.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
