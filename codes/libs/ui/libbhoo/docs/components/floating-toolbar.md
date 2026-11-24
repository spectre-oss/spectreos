# Floating toolbar

Status: Experimental Rust toolbar API and browser example; complete Material behavior remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Floating action surface; placement and collapse behavior.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Track visibility, anchor and available actions; ensure a focused toolbar cannot disappear silently.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Translate/fade the container on explicit show/hide; retarget placement changes without animating every child.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Move its anchor while an action is focused and verify on-screen placement and focus persist.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/FloatingToolbar.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/toolbars/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/floatingtoolbar/ftbvertical.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [FloatingToolbarLayout.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/floatingtoolbar/FloatingToolbarLayout.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/floatingtoolbar/FloatingToolbarLayout.java) |
| Compose Material 3 | [FloatingToolbar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingToolbar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingToolbar.kt) |
| Angular Material | [toolbar.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/toolbar/toolbar.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/toolbar/toolbar.ts) |
| Flutter material_ui | [app_bar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/app_bar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/app_bar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: FloatingToolbar.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/FloatingToolbar.md)
- [Material Android: DockedFloatingToolbars.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/DockedFloatingToolbars.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)


Paired FAB placement, activation, expansion geometry and interruption are covered by Rust tests for horizontal/vertical and RTL arrangements, and by 320 × 440 browser interaction tests for horizontal placement and expand/collapse. A 320 × 440 browser interaction test covers the vertical example, including expanded/collapsed geometry, top/bottom placement and RTL. A separate Rust test verifies that reversing the vertical expansion midway preserves the FAB size and position at both ends. Device styling and vertical visual parity remain unverified.

The hide-on-scroll and compact-collapse implementations are validated by Rust timing/threshold tests and 320 × 440 browser touch tests. These checks do not establish Android device motion or pixel parity.

For the paired FAB, standard uses Bhoo's primary-container roles and vibrant uses tertiary-container roles, following the pinned Compose color mapping. The standard toolbar surface uses Bhoo's control/surface-container approximation; vibrant uses primary-container. A Rust color-role test and a browser canvas pixel test cover both variants using the current Bhoo theme. Exact Android dynamic color and shadow parity remain unverified.

The `Frame::toolbar` API draws standard or vibrant docked, horizontal floating, and vertical floating surfaces with labeled actions. `toolbar_with_overflow` offers a menu for actions that do not fit, and `toolbar_roving` supports arrow-key navigation. The caller owns placement and action handling. `floating_toolbar_scroll_offset` supplies a bottom-anchored hide-on-consumed-scroll offset using the shared interruptible timing; callers provide the full hide distance including the bottom gap, clip the toolbar to the viewport, and reserve any necessary content space. Positive consumed scroll hides, negative scroll shows, and reduced motion snaps. A paired 56 dp FAB can be placed at logical start/end or top/bottom with an 8 dp gap; `floating_toolbar_with_fab` preserves toolbar overflow and reports FAB and action activation separately. The caller supplies the overall bounds and FAB icon. The optional `floating_toolbar_with_fab_expanded` variant contracts the toolbar while the FAB grows from 56 dp to 80 dp, using interruptible fast-spatial motion; reduced motion snaps, and toolbar actions stop accepting input when collapse is requested. The caller controls expansion. `FloatingToolbarExpansionScrollState::update` can change that state from consumed vertical scroll with independent expand and collapse thresholds (40 dp in the example) and reverse-layout support; it does not itself consume scroll. The paired toolbar uses level 1 elevation while expanded and transitions to level 0 on collapse; its FAB uses level 2 at rest and level 3 on hover. Bhoo approximates these shadows with its shared renderer. Rust tests cover toolbar and FAB elevation roles, and a browser pixel check verifies that the toolbar shadow appears when expanded and disappears when collapsed. Other FAB sizes and complete device parity remain open.

## Browser demonstration

[Open the live Floating toolbar preview](http://127.0.0.1:8766/#/floating-toolbar).

Demonstrates: Rounded floating surfaces, an optional paired FAB, overflow, and an interruptible bottom-anchored hide-on-scroll example.

Remaining scope: General anchored placement, drag interactions, additional FAB sizes and Android device visual parity.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML `FloatingToolbarLayout` support (`RULE_FLOATING_TOOLBAR = 270`, Wave 14C / Wave 15A)

- `tools/import-android-layout.py` lowers `com.google.android.material.floatingtoolbar.FloatingToolbarLayout` to horizontal `Kind::Linear` emitting `RULE_FLOATING_TOOLBAR = 270` (`anchor: ""`) and validating system-window inset booleans.
- At runtime in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `RULE_FLOATING_TOOLBAR` renders a `64dp` full-pill (`height * 0.5` corner radius) `surfaceContainer` container with elevation shadow and occludes lower siblings from pointer input in overlapping containers (`blocks_pointer_for_lower_siblings`).
- **Validation:** Rust runtime test `wave15a_floating_toolbar_button_group_split_button_and_carousel_item` in [`tests/android_xml.rs`](../../tests/android_xml.rs), Python importer test `test_wave14c_m3_expressive_tags_rules_270_to_273` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), and static Light/Dark Mesa GLES Page-3 captures (`context/android-xml-comparison/bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png`).
- **Remaining Android XML limitations:** Scroll-coupled hide/collapse animations, vertical XML floating toolbar variants, vibrant container themes, and Android device visual parity remain open. See [Android XML import](../android-xml.md).
