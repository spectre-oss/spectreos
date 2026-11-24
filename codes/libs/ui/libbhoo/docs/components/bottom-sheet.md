# Bottom sheet

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Standard and modal; dismissal and focus containment.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Define standard/modal forms, snap states, dismissal and drag ownership; provide keyboard alternatives to dragging.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Follow dragging directly, then settle to a bounded snap point; coordinate scrim and surface exit.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Interrupt settling with a new drag and verify no jump, overshoot into invalid space or stale focus trap.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomSheet.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/bottom-sheets/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/bottomsheet/bottomsheet-standard1.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [BottomSheetBehavior.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomsheet/BottomSheetBehavior.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/bottomsheet/BottomSheetBehavior.java) |
| Compose Material 3 | [ModalBottomSheet.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ModalBottomSheet.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ModalBottomSheet.kt) |
| Angular Material | [bottom-sheet.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/bottom-sheet/bottom-sheet.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/bottom-sheet/bottom-sheet.ts) |
| Flutter material_ui | [bottom_sheet.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/bottom_sheet.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/bottom_sheet.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: BottomSheet.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomSheet.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Bottom sheet preview](http://127.0.0.1:8766/#/bottom-sheet).

Demonstrates: Bottom-aligned modal surface with scrim and focus lifecycle.

Remaining scope: Drag detents, nested scroll.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

The Angular comparison fixture uses a full-width, square-cornered sheet with a surface-container-low background at the demo viewport. Its first action receives focus; Cancel, Save, Escape and the scrim dismiss it. The sheet slides in over 195 ms and out over 375 ms, using Angular’s easing curves. The browser checks compare the action geometry and surface in both themes and sample the slide against Angular.

## Native M3 geometry and Android XML support (Wave 14C / Wave 15A / Wave 15B / Wave 16A / Wave 19A)

- **Native M3 `OverlayKind::BottomSheet` (Wave 15B, [`composition.rs`](../../src/systems/material/mod.rs)):** Clips a `28dp`-rounded `surface_low` (`surfaceContainerLow`) surface strictly to the sheet's bounds while extending the drawn rectangle past the bottom clip floor (`>= 28dp`) so only the top two corners are rounded `28dp` and the bottom edge sits strictly `0dp` flat and flush against the viewport floor (both with and without `drag_to_dismiss`). Covered by `bottom_sheet_renders_top_only_28dp_corners_with_flush_bottom_edge` in [`tests/wave15b_composition.rs`](../../tests/wave15b_composition.rs).
- **Android XML `BottomSheetBehavior` (`RULE_BOTTOM_SHEET = 259`) & `BottomSheetDragHandleView` (`RULE_DRAG_HANDLE = 260`) (Wave 14C / Wave 15A / Wave 16A / Wave 19A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - `tools/import-android-layout.py` emits `RULE_BOTTOM_SHEET = 259` (`"<peek_h_dp>|<hideable:0|1>|<fit_to_contents:0|1>|<half_ratio>|<skip_collapsed:0|1>"`) on direct `CoordinatorLayout` children with `BottomSheetBehavior` and lowers `BottomSheetDragHandleView` to `Kind::View` with `RULE_DRAG_HANDLE = 260`.
  - At runtime, `LayoutState.sheet_state` (`SHEET_STATE_EXPANDED = 3`, `SHEET_STATE_COLLAPSED = 4`, `SHEET_STATE_HIDDEN = 5`, `SHEET_STATE_HALF_EXPANDED = 6`) positions the sheet flush at the bottom edge with `surfaceContainerLow` fill, top-only `28dp` corners, and level-2 elevation shadow, and hides it when `SHEET_STATE_HIDDEN` and `hideable = 1`.
  - **Interactive drag detents, flick-to-hide & clickable drag handle (Wave 16A):** Pointer/touch vertical dragging (`8dp` drag slop via `state.dp(8.0)`) scrubs sheet height directly across `EXPANDED`, `HALF_EXPANDED` (when `!fit_to_contents`), `COLLAPSED` (when `!skip_collapsed`), and `HIDDEN` (when `hideable = 1`), settles to the nearest detent or hides on downward flick velocity (`> 500 dp/s`, `state.dp(500.0)`) over `0.25s` ease-out (`XmlSheetMotion`), and restores the pre-drag height on pointer cancel/blur. `RULE_DRAG_HANDLE = 260` measures `48dp` default touch height, paints a centered `32 × 4dp` (`2dp` radius) pill (`onSurfaceVariant` `0.4` alpha or `RULE_BACKGROUND_TINT`), and clicking it (without dragging) toggles between `SHEET_STATE_COLLAPSED` and `SHEET_STATE_EXPANDED` (or `SHEET_STATE_HIDDEN` when `skip_collapsed && hideable`) over `0.25s`.
  - **Nested scrolling handoff (Wave 19A):** `collect_descendant_scrolls` inspects descendant `ScrollView`, `NestedScrollView`, `ListView`, `GridView`, and `ExpandableListView` offsets inside an expanded `BottomSheet`. While an inner scroll container is scrolled (`offset > 0.5`) or the user drags upward (`dy < 0.0`), the inner scroll owns the gesture and re-anchors `motion.origin` (keeping `drag_base` at the expanded height); once inner scroll reaches top (`offset <= 0.5`) and the user drags downward past `8dp` slop (`dy > slop && dy.abs() >= dx.abs()`), the sheet takes over seamlessly, cancels active inner scroll tracking via `f.cancel_scroll(...)`, clamps any negative rubber-band offset back to `0.0`, and scrubs/collapses smoothly from anywhere on the sheet body.
  - **Motion:** Both drag settling (including after nested-scroll takeover) and drag-handle toggles use `0.25s` ease-out transitions under `MotionPreference::Full` and snap immediately to final state geometry under `MotionPreference::Reduced`. Native `OverlayKind::BottomSheet` uses the shared overlay transition and snaps under reduced motion.
  - **Validation:** Functional Rust tests `wave15a_bottom_sheet_states_and_drag_handle`, `wave16a_bottom_sheet_drag_detents_settle_flick_handle_and_reduced_motion`, and `wave19a_bottom_sheet_nested_scrolling_handoff_scroll_and_listview` in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`312` passing tests; `907` default / `913` all-features Rust tests across `76` suites), Python importer test `test_wave14c_bottom_sheet_drag_handle_and_side_sheet_rules_259_260_261` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`241` passing tests), and static Light/Dark Mesa GLES Page-3 captures (`context/android-xml-comparison/bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png` via [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) `--page3`).
  - **Remaining limitations:** Exact Android spring/fling velocity curves, post-expand continuous upward-scroll carry-over into inner scroll within a single unbroken gesture, keyboard detent shortcuts, and Android device visual parity remain open. See [Android XML import](../android-xml.md).
