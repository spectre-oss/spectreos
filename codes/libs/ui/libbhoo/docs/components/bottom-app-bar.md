# Bottom app bar

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Layout](../motion.md#motion-profiles) |

## Scope and variants

Bottom actions and optional FAB relationship.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Group bottom actions and optional FAB placement; preserve action labels and keyboard access.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Translate the bar only on explicit visibility changes; any FAB cutout/placement changes stay synchronized.

Use the shared [layout profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Apply final geometry immediately and retain the same stable focus/selection identity; no sliding or size tween.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Resize with the FAB visible and verify controls do not overlap or become unreachable.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomAppBar.md)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/bottomappbar/bottomappbar-basic.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [BottomAppBar.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomappbar/BottomAppBar.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/bottomappbar/BottomAppBar.java) |
| Compose Material 3 | [AppBar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt) |
| Angular Material | [toolbar.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/toolbar/toolbar.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/toolbar/toolbar.ts) |
| Flutter material_ui | [bottom_app_bar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/bottom_app_bar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/bottom_app_bar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: BottomAppBar.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomAppBar.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Bottom app bar preview](http://127.0.0.1:8766/#/bottom-app-bar).

Demonstrates: An 80 dp bottom app bar anchored to the bottom of a sample viewport, Material icon actions, an optional 56 dp end-aligned FAB and RTL placement. A narrow-width option moves actions that do not fit into an overflow menu; if necessary the FAB also moves into that menu. Disabled actions stay disabled, keyboard navigation and label typeahead can reach offscreen menu items, and Escape restores focus. Enable Many actions to try touch-drag scrolling; dragging cancels item activation, and a quick swipe continues with bounded momentum until interrupted or stopped at an edge. Reduced motion disables momentum. The overflow API supports up to 64 actions and requires a viewport at least 56 dp wide; the older `bottom_app_bar_at` API still omits actions that do not fit. An optional exit-always mode hides the bar as content consumes forward scroll and reveals it on reverse scroll; the pinned mode keeps it visible. The preview routes wheel, touch and keyboard input through the shared scrolling helpers, including Page Up and Page Down. After a touch release or its content fling, a partially hidden bar settles to the nearest edge; a new gesture interrupts the settle and reduced motion snaps immediately. The bar is clipped to its viewport, and fully hidden actions leave the focus order. The bar dimensions and post-scroll offset behavior are informed by the local Apache-2.0 Compose `BottomAppBar` implementation and tokens. Icon source and license details are in [third-party notices](../../../../../../THIRD_PARTY_NOTICES.md).

Remaining scope: A general scaffold with system insets, exact Android overflow fling physics and complete RTL text shaping, exact Android fling and spring settling, full Material color tokens and accessibility bridge. The bar currently follows the shared approximate content fling and uses a 250 ms settle curve; wheel and keyboard scrolling remain direct. The demo anchors within its own preview viewport rather than a platform window. This initial implementation does not claim full visual parity.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML importer and runtime support (Wave 8A / Wave 8B / Wave 21A / Wave 21B / Wave 22A / Wave 22B)

`tools/import-android-layout.py` lowers `com.google.android.material.bottomappbar.BottomAppBar` to `Kind::Toolbar` emitting `RULE_BOTTOM_APP_BAR = 243` (`anchor: "center"` by default or `"end"` from `app:fabAlignmentMode`, directly or via `@style`) and, in Wave 22B, `RULE_BOTTOM_APP_BAR_EXT = 300` (`"<align>|<anchor>|<margin>,<rad>,<voff>|<hide:0|1>"`, resolving `app:fabAlignmentMode` `center`/`end`, `app:fabAnchorMode` `cradle`/`embed`, `app:fabCradleMargin` default `5`, `app:fabCradleRoundedCornerRadius` default `8`, `app:fabCradleVerticalOffset` default `0`, and `app:hideOnScroll` `true`/`false`; omitted when none of those attributes are set, preserving `RULE_BOTTOM_APP_BAR = 243`). When `app:menu="@menu/<name>"` is declared and `res/menu/<name>.xml` is available in `--resources`, the importer inflates visible menu items into ordered repeatable `RULE_NAV_ITEM = 238` (`"<id>|<title>|<icon_name>"`) rules sorted by `android:orderInCategory` (plus `RULE_NAV_BADGE = 244` when an item specifies `app:badgeText` and Wave 21B `RULE_MATERIAL_BADGE_EXT = 297` for `app:badgeGravity`/`badgeMaxCharacterCount`/`badgeHorizontalOffset`/`badgeVerticalOffset`), degrading cleanly on `@null` or missing menu files while strictly rejecting malformed menu references or invalid badge attributes. The importer also validates `app:fabAnimationMode` (`scale`/`slide`), `app:fabAnchorMode` (`embed`/`cradle`), `app:menuAlignmentMode` (`auto`/`start`), `app:fabCradleMargin`/`RoundedCornerRadius`/`VerticalOffset`, `app:fabAlignmentModeEndMargin`, `app:removeEmbeddedFabElevation`, `app:addElevationShadow`, `app:hideOnScroll`, and `app:paddingBottom/Left/RightSystemWindowInsets`.

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, independently implemented against pinned Material Components for Android [`BottomAppBar.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomappbar/BottomAppBar.java) and [`bottomappbar/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomappbar/res/values/tokens.xml) (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE), local `context/reference-repos/material-android/lib/java/com/google/android/material/bottomappbar/`):

- **Geometry and visual tokens:** `80dp` wrap height (density-scaled via `LayoutState.effective_density()`), M3 `surfaceContainer` fill (`#F3EDF7` in `Material3Light`/`MaterialLight`, `#211F26` in `Material3Dark`/`MaterialDark`, `t.surface_low` under `Bhoo`) unless overridden by explicit background, tint, gradient, or layers.
- **FAB cradle/embed cutout, icon reservation & `CoordinatorLayout` FAB docking (`RULE_BOTTOM_APP_BAR_EXT = 300`, Wave 22A/22B):** Renders an analytical anti-aliased SDF transparent FAB cradle cutout (rasterized into bounded `LayoutState.bottom_app_bar_images`, capped at `512px`, `32` entries, `4 MB`) when `anchor == cradle`, or a recessed circular pocket when `anchor == embed`, placed at `center` or RTL-aware `end`. Action icons reserve `[fab_cx - cutout_r, fab_cx + cutout_r]` so slots do not collide with the FAB cutout. Inside `CoordinatorLayout`, FABs anchored to the `BottomAppBar` via `RULE_COORDINATOR_ANCHOR = 227` dock automatically into the resolved cradle/embed coordinate.
- **Navigation, action icons, action badges & RTL mirroring:** Optional leading `48 × 48dp` navigation icon slot (at `4dp` leading inset, `24dp` `onSurfaceVariant` icon, updating `LayoutState.toolbar_navigation_clicked`) followed by leading `48 × 48dp` slots for each `RULE_NAV_ITEM = 238` action (`24dp` icon from tinted `LayoutState.images` or bundled `MaterialIcon`, circular state layer and focus ring, `0.38` alpha when disabled, plus Wave 21A `RULE_NAV_BADGE = 244` & `RULE_MATERIAL_BADGE_EXT = 297` action badges with `maxCharacterCount` numeric truncation, 4-corner `badgeGravity`, and RTL-mirrored dp offsets; see [Badge](badge.md)). In RTL (`LayoutState.rtl`), slots step leftward from the physical right edge (`4dp` right inset) while respecting the reserved FAB cutout band.
- **Embedded children and `fabAlignmentMode`:** Child views (such as an embedded FAB) render vertically centered and horizontally aligned by explicit child `layout_gravity` or by `fabAlignmentMode` (`center` centers horizontally; `end` aligns to the trailing edge with `16dp` end padding, mirrored to physical left in RTL).
- **Interaction and `hideOnScroll` motion (`RULE_BOTTOM_APP_BAR_EXT = 300`, Wave 22A/22B):** Clicking a menu action slot returns its XML item ID (or the bar's ID if unnamed). When `hide_on_scroll == 1`, bounded `LayoutState.bottom_app_bar_hide_motion` (`64`-entry cap) tracks sibling scroll deltas and smoothly slides both the `BottomAppBar` and any `CoordinatorLayout`-anchored FAB off-screen on downward scroll and back on upward scroll over `0.22s` ease-out, snapping immediately without intermediate frames under `MotionPreference::Reduced` (aligning with [motion.md](../motion.md#motion-profiles)).
- **Validation evidence:**
  - Functional Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`324` tests within `919` default / `925` all-features Rust tests across `76` suites): `bottom_app_bar_rule_renders_surface_leading_actions_rtl_and_clicks` (verifies `80dp` wrap height, Light/Dark `surfaceContainer` fills, 3 leading `24dp` action slots, interactive action click returning `"bab_edit"`, RTL mirroring and RTL click routing, and `center` `fabAlignmentMode` child centering), Wave 21A `wave21a_material_badge_ext_truncation_gravity_offsets_and_rtl` (verifies `BottomAppBar` action badge rendering and `maxCharacterCount` truncation), and Wave 22A `wave22a_bottom_app_bar_ext_cradle_embed_cutout_icons_anchor_rtl_and_hide_on_scroll` (verifies analytical SDF `cradle` cutout & `embed` recessed pocket, bounded `bottom_app_bar_images` cache reuse, `[fab_cx - cutout_r, fab_cx + cutout_r]` icon reservation, `CoordinatorLayout` anchored FAB docking in LTR/RTL, `0.22s` ease-out `hideOnScroll` synchronized bar+FAB slide, and reduced-motion instant snap).
  - Importer unit tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`252` passing tests, `0` diffs across `167` existing layouts): `test_wave8a_bottom_app_bar_rule_243_and_menu_inflation`, Wave 21B `test_wave21b_material_badge_ext_rule_297`, and Wave 22B `test_wave22b_bottom_app_bar_ext_rule_300`.
  - Functional unit tests only; no side-by-side Waydroid Material 3 visual capture has been performed for `BottomAppBar`.
- **Remaining Android XML limitations:** Overflow menu collapse when actions exceed available width (overflowing slots are currently omitted rather than moved to a menu), custom XML badge colors, `fabAnimationMode` (`scale`/`slide`) alignment-switch transitions, `menuAlignmentMode`, `fabAlignmentModeEndMargin`, `removeEmbeddedFabElevation`, system-window inset padding, and Waydroid visual comparison remain open. See [Android XML import](../android-xml.md) and [licenses and attribution](../licenses-and-attribution.md).
