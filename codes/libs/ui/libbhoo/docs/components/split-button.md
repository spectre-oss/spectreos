# Split button

Status: Experimental split-button API with five sizes, separate actions, RTL placement and an anchored menu example. Full Material state semantics and visual parity remain incomplete.

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Primary action plus a separate secondary-action menu trigger.

`Frame::split_button` provides a small button, while `Frame::split_button_with` supports 32, 40, 56, 96 and 136dp visual heights, a 2dp gap, separately enabled actions, mirrored RTL placement and filled, tonal, outlined and elevated treatments. The caller owns expanded state and positions the menu using the trailing response. Leading icons, large-size typography, flexible layout, precise shadows and exact variant tokens remain open.

## State and interaction contract

Expose distinct primary and menu actions; provide separate focus targets and expanded state for the trailing trigger.

The menu action description includes expanded or collapsed state, and each half can be disabled independently. A dedicated platform accessibility expanded-state bridge and localized state descriptions remain open.

## Motion and animation

Each half has independently animated inner corners and clipped press feedback. The expanded trailing action also keeps a 12% content-color state layer as in the local Compose reference. The menu chevron rotates when the caller changes expanded state; reduced motion snaps the chevron and corners. Exact upstream motion validation remains open.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Activate the trailing trigger with a keyboard and confirm the primary action never fires.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/SplitButton.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/split-button/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/buttons/splitbutton-expressive.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

The pinned AndroidX [small split-button tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SplitButtonSmallTokens.kt) specify a 40dp container and 2dp gap.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [MaterialSplitButton.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/button/MaterialSplitButton.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/button/MaterialSplitButton.java) |
| Compose Material 3 | [SplitButton.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SplitButton.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SplitButton.kt) |
| Angular Material | [menu-trigger.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/menu/menu-trigger.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/menu/menu-trigger.ts) |
| Flutter material_ui | [menu_anchor.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/menu_anchor.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/menu_anchor.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: SplitButton.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/SplitButton.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Split button preview](http://127.0.0.1:8766/#/split-button).

Demonstrates: Connected split buttons in five sizes and several styles, an animated trailing chevron, independent primary and menu disable controls, and RTL placement. The outlined variant draws a transparent interior; its disabled border and foreground follow the local Apache-2.0 Compose Button defaults.

Remaining scope: Inner corners use normalized spring targets and elevated shadows follow the same asymmetric shape. Rust tests cover the five token sizes, hover, press, expansion, RTL, reduced-motion snapping, and reversal during an animation. Press takes precedence over the expanded shape and keyboard focus alone retains the resting shape. The local Compose implementation morphs on press and checked state; Bhoo also applies the hover corner defined by the split-button tokens, so toolkit behavior differs on hover. Canvas and headless Wayland/OpenGL ES pixel checks cover the transparent interior and representative border pixels; the GLES check runs at 1× and 2× scale. Full edge antialiasing parity, typography, elevation, tokens, motion, and the accessibility bridge remain open.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML `MaterialSplitButton` support (`RULE_SPLIT_BUTTON = 272`, Wave 14C / Wave 15A)

- `tools/import-android-layout.py` lowers `com.google.android.material.button.MaterialSplitButton` to horizontal `Kind::Linear` emitting `RULE_SPLIT_BUTTON = 272` (`anchor: ""`, supporting `--merge-parent` and `--custom-view-map`).
- At runtime in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `RULE_SPLIT_BUTTON` separates the leading primary button and trailing menu-trigger button with a `2dp` gap, draws a trailing chevron polyline on the secondary button, and mirrors both halves in RTL (`LayoutState.rtl`).
- **Validation:** Rust runtime test `wave15a_floating_toolbar_button_group_split_button_and_carousel_item` in [`tests/android_xml.rs`](../../tests/android_xml.rs), Python importer test `test_wave14c_m3_expressive_tags_rules_270_to_273` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), and static Light/Dark Mesa GLES Page-3 captures (`context/android-xml-comparison/bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png`).
- **Remaining Android XML limitations:** Animated inner-corner morphing on press/expand, animated chevron rotation on menu expansion in XML, and Android device visual parity remain open (native Rust `Frame::split_button_with` implements animated corners and chevron rotation separately). See [Android XML import](../android-xml.md).
