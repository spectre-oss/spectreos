# Side sheet

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Standard and modal supplementary content.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Anchor supplementary content to a logical edge; distinguish modal focus trapping from persistent content.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Follow direct dragging if supported; otherwise reveal from the anchored edge with coordinated scrim.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Resize or change direction while open and verify clipping, focus and edge placement remain correct.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/SideSheet.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/side-sheets/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/sidesheet/sidesheet-hero.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [SideSheetBehavior.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/sidesheet/SideSheetBehavior.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/sidesheet/SideSheetBehavior.java) |
| Angular Material | [drawer.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/sidenav/drawer.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/sidenav/drawer.ts) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: SideSheet.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/SideSheet.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Side sheet preview](http://127.0.0.1:8766/#/side-sheet).

Demonstrates: Trailing modal surface with dismissal and focus lifecycle.

Remaining scope: Adaptive standard variant.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Native M3 modal/standard variants and Android XML support (Wave 14C / Wave 15A / Wave 15B / Wave 16A)

- **Native M3 modal `OverlayKind::SideSheet` / `LeftSideSheet` (Wave 15B, [`composition.rs`](../../src/systems/material/mod.rs)):**
  - Clips a `16dp`-rounded `surface_low` (`surfaceContainerLow`) surface so only the inner edge (left corners on right-docked `SideSheet`, right corners on left-docked `LeftSideSheet`) is rounded `16dp` while the outer viewport edge sits flush at `0dp`.
  - Adds horizontal outward drag-to-dismiss without breaking taps on buttons inside the sheet: dragging outward past `40%` of the sheet width dismisses the sheet (`state.open = false`), while shorter drags smoothly settle back open via shared overlay motion (or snap under reduced motion).
- **Native coplanar `Frame::standard_side_sheet` (Wave 15B, [`composition.rs`](../../src/systems/material/mod.rs)):** Shares viewport space coplanarly between main content and a right- or left-edge side sheet separated by a `1dp` vertical `outline_variant` divider (both panes stay interactive simultaneously), animating width smoothly under `MotionPreference::Full` and snapping under `MotionPreference::Reduced`.
- **Android XML `SideSheetBehavior` (`RULE_SIDE_SHEET = 261`) (Wave 14C / Wave 15A / Wave 16A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - `tools/import-android-layout.py` emits `RULE_SIDE_SHEET = 261` (`"<edge:end|start>|<coplanar:0|1>"`) on direct `CoordinatorLayout` children with `SideSheetBehavior`.
  - At runtime, honors `LayoutState.sheet_state` (`SHEET_STATE_HIDDEN = 5` hides), mirrors `start`/`end` edges in RTL, shrinks sibling main content width with a `1dp` `outlineVariant` seam divider when `coplanar = 1`, and renders a translucent scrim (`Color(0, 0, 0, 82)`), elevation shadow, and click-scrim-to-dismiss when modal (`coplanar = 0`).
  - **RTL-aware horizontal drag-to-dismiss & scrim fade (Wave 16A):** Modal XML side sheets support horizontal outward drag-to-dismiss (`8dp` drag slop, `> 30%` panel width or `> 500 dp/s` outward flick dismisses to `SHEET_STATE_HIDDEN` with a `0.25s` ease-out slide-out, while shorter or cancelled drags settle back open over `0.25s`), fading scrim alpha proportionally to open progress (`1.0 - drag_offset / width`) and snapping immediately under `MotionPreference::Reduced`.
- **Validation:**
  - Wave 15B Rust tests in [`tests/wave15b_composition.rs`](../../tests/wave15b_composition.rs): `modal_right_and_left_side_sheets_round_only_inner_edge_16dp`, `modal_side_sheet_horizontal_drag_to_dismiss_and_short_drag_settle`, `modal_left_side_sheet_drag_left_dismisses`, and `standard_side_sheet_shares_coplanar_space_with_divider_and_animates`.
  - Wave 15A/16A/14C tests: `wave15a_side_sheet_modal_coplanar_and_rtl` and `wave16a_modal_side_sheet_drag_to_dismiss_settle_rtl_and_reduced_motion` in [`tests/android_xml.rs`](../../tests/android_xml.rs), and `test_wave14c_bottom_sheet_drag_handle_and_side_sheet_rules_259_260_261` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
- **Remaining limitations:** Standardized header/close/action slots, exact Android spring physics, focus restoration across arbitrary XML children, and Android device visual parity remain open. See [Android XML import](../android-xml.md).
