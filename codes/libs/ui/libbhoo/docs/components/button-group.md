# Button group

Status: Experimental standard action and connected selection groups. Standard overflow and optional bundled icons on visible and overflow actions are implemented; platform accessibility bridges and full Material fidelity remain open.

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Grouped related actions; expressive spacing and shape behavior.

`Frame::button_group` implements an equal-width standard group with 12dp gaps, 40dp visual buttons and at least 48dp hit targets. `button_group_direction` mirrors its logical order in RTL, including during press-width animation. `button_group_with_icons` adds optional 20dp bundled Material icons and mirrors their placement; `button_group_with_overflow_icons` preserves these icons when actions move into the overflow menu. `Frame::connected_button_group` adds single or multiple selection, optional required selection, disabled children, 2dp gaps and independent animated corners. Connected controls expose toggle and checked state in Bhoo semantics, support Left/Right/Home/End focus movement with disabled items skipped, and draw a clipped ink ripple. `connected_button_group_direction` mirrors placement and horizontal arrow navigation in RTL while preserving logical action indices; the example includes an RTL switch. The browser regression `node tools/test-bhoo-button-group-rtl.mjs` exercises mirroring, action targeting, selection retention and a horizontal arrow key through the demo. `button_group_with_overflow` accepts up to 32 actions, uses a 40dp filled Material more-icon trigger with a 48dp hit target, and moves actions that do not fit into an anchored menu, with RTL placement, wheel scrolling, typeahead and keyboard navigation. The minimum width is 48dp. Touch dragging cancels item activation and supports bounded momentum; reduced motion suppresses momentum. Rust tests cover disabled menu items and focus restoration after Escape or expansion to a wide layout. `connected_button_group_with_overflow` preserves single or multiple selection across visible and hidden actions, including required selection and RTL. The original connected APIs still require room for all targets. The demo includes narrow-width and more-actions toggles for testing the scrollable menu, including icon-bearing actions. The browser regression `node tools/test-bhoo-button-group-overflow.mjs` checks hidden-action activation, touch dragging without selection, and wheel scrolling.

## State and interaction contract

Keep each child action independently named and focusable; define order and disabled-child behavior.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Pressing an enabled child springs its width toward a 15% expansion by compressing its neighbors, subject to their 48dp minimum hit widths. Child semantics follow the presented geometry. Connected buttons animate their inner corner between 8dp, 4dp pressed and 20dp selected while leaving neighboring widths unchanged. Standard small buttons also morph from a full 20dp radius to the Compose small-button pressed radius of 8dp. Exact upstream motion tokens and visual validation remain open.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Press adjacent actions rapidly and confirm their state and actions remain independent.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ButtonGroup.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/button-groups/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/buttons/buttongroup-expressive.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [MaterialButtonGroup.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/button/MaterialButtonGroup.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/button/MaterialButtonGroup.java) |
| Compose Material 3 | [ButtonGroup.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt) |
| Angular Material | [button-toggle.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/button-toggle/button-toggle.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/button-toggle/button-toggle.ts) |
| Flutter material_ui | [toggle_buttons.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/toggle_buttons.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/toggle_buttons.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: ButtonGroup.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ButtonGroup.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Button group preview](http://127.0.0.1:8766/#/button-group).

Demonstrates: Grouped independent cut/copy/paste actions.

Remaining scope: standard shape morph, icon and size variants, overflow, RTL, platform accessibility bridging and exact motion and token parity.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML `MaterialButtonGroup` support (`RULE_BUTTON_GROUP = 271`, Wave 14C / Wave 15A)

- `tools/import-android-layout.py` lowers `com.google.android.material.button.MaterialButtonGroup` to horizontal `Kind::Linear` emitting `RULE_BUTTON_GROUP = 271` (`anchor: "<spacing_dp>"`, defaulting to `12.0` or `2.0` for `*Connected*` styles, or explicit `android:spacing`/`app:spacing`) and validating `app:overflowMode` (`none`/`menu`/`wrap`).
- At runtime in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `RULE_BUTTON_GROUP` inserts density-scaled horizontal spacing between visible child buttons.
- **Validation:** Rust runtime test `wave15a_floating_toolbar_button_group_split_button_and_carousel_item` in [`tests/android_xml.rs`](../../tests/android_xml.rs) and Python importer test `test_wave14c_m3_expressive_tags_rules_270_to_273` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
- **Remaining Android XML limitations:** Interactive press-width neighbor compression, dynamic overflow menu/wrap collapse in XML, and Android device visual parity remain open (native Rust `Frame::button_group` / `button_group_with_overflow` / `connected_button_group` implement press-width and overflow separately). See [Android XML import](../android-xml.md).
