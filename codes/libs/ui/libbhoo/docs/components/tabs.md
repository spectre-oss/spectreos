# Tabs

Status: Partial implementation — Wave 14B native Primary/Secondary/Scrollable `Frame::tabs_with_options` (`TabStyle`, `TabOptions`), Wave 14C/15A Android XML `TabLayout` (`RULE_TAB_OPTIONS = 266`), and Wave 16A/23A/23B runtime `TabLayout` ↔ `ViewPager`/`ViewPager2` two-way sync (`LayoutState::tab_pager_links`, `RULE_VIEW_PAGER = 248`, `RULE_VIEW_PAGER_EXT = 305`); leading/stacked icons, per-tab disabled state, RTL scrollable validation, and device visual parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Primary/secondary styling, selected indicator, overflow/scrolling.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Own selected tab and keyboard focus with explicit automatic/manual activation policy; support overflow and disabled tabs.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Translate the active indicator; scroll the selected tab into view without animating hidden panel contents.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Remove the selected tab during transition and choose a documented focus/selection fallback.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/tabs.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/tabs)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/tabs/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [tabs.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/tabs/internal/tabs.ts) | [Downloaded file](../../../../context/reference-sources/material-web/tabs/internal/tabs.ts) |
| Material Components for Android | [TabLayout.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/tabs/TabLayout.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/tabs/TabLayout.java) |
| Compose Material 3 | [TabRow.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt) |
| Compose Material 3 | [Tab.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt) |
| Angular Material | [tab-group.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/tabs/tab-group.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/tabs/tab-group.ts) |
| Flutter material_ui | [tabs.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/tabs.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/tabs.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: tabs.md](../material-design/upstream/docs/components/tabs.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Native Primary/Secondary/Scrollable Tabs and Android XML support (Wave 14B / Wave 14C / Wave 15A / Wave 16A / Wave 21A / Wave 23A / Wave 23B)

- **Native M3 `Frame::tabs_with_options` (Wave 14B, [`widgets.rs`](../../src/systems/material/mod.rs), `TabStyle` & `TabOptions` re-exported at crate root in [`lib.rs`](../../src/lib.rs)):**
  - `TabStyle::{Primary, Secondary}` and `TabOptions { style, scrollable, enabled }`:
    - `Primary`: `3dp` top-rounded (`radius: 3.0`, clipped to `3dp` height) label-width active indicator (`>= 24dp`) with `theme.primary` active label color.
    - `Secondary`: `2dp` sharp (`radius: 0.0`) full-tab-width active indicator (`theme.primary`) with `theme.text` (`onSurface`) active label color.
    - `scrollable: true`: enables horizontal wheel scrolling across overflow tab widths with offset-adjusted pointer hit-testing.
- **Android XML `TabLayout` (`RULE_TAB_OPTIONS = 266`), tab badges (`RULE_NAV_BADGE = 244` / `RULE_MATERIAL_BADGE_EXT = 297`), & `ViewPager`/`ViewPager2` sync (`RULE_VIEW_PAGER = 248` / `RULE_VIEW_PAGER_EXT = 305`, Wave 14C / Wave 15A / Wave 16A / Wave 21A / Wave 23A / Wave 23B, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - `tools/import-android-layout.py` emits `RULE_TAB_OPTIONS = 266` (`"<mode:fixed|scrollable>|<style:primary|secondary>|<indicator_rgba>"`) on `com.google.android.material.tabs.TabLayout`. At runtime on `Kind::TabWidget`, selects Primary (`3dp` top-rounded label-width) vs. Secondary (`2dp` full-width) indicators, applies custom indicator RGBA overrides, and supports horizontal wheel scrolling when `mode == "scrollable"`.
  - **Two-way `TabLayout` ↔ `ViewPager`/`ViewPager2` synchronization & extended pager controls (Wave 16A / Wave 23A / Wave 23B):** `LayoutState.tab_pager_links: HashMap<String, String>` links a `TabLayout` (`Kind::TabWidget`) XML ID with a `ViewPager2`/`ViewPager` (including Wave 23B legacy `android.support.v4.view.ViewPager`, `RULE_VIEW_PAGER = 248` / `RULE_VIEW_PAGER_EXT = 305`) XML ID symmetrically (`linked_tab_pager_partners`). Clicking a tab updates the linked pager's `selected_index` and `displayed_children` (even when `app:userInputEnabled="false"` locks direct drag/keyboard input on the pager), while interactive horizontal (RTL-aware) or vertical drag-to-scrub (`8dp` slop, `> 30%` / `> 400 dp/s` threshold, `0.25s` ease-out settle, instant under reduced motion, optional `pageMargin` and `zoom_out`/`depth` page transformers) updates the linked tab selection immediately. See [`ViewFlipper`](android-viewflipper.md).
  - **Tab badges (`RULE_NAV_BADGE = 244` & `RULE_MATERIAL_BADGE_EXT = 297`, Wave 21A):** Renders dot and pill badges keyed by 0-based tab index with Android `BadgeDrawable` `maxCharacterCount` numeric truncation (`4` -> `"999+"`, `3` -> `"99+"`, `2` -> `"9+"`), 4-corner RTL-aware `badgeGravity`, and mirrored horizontal/vertical dp offsets. See [Badge](badge.md).
- **Validation:**
  - Wave 14B Rust test: `tabs_with_options_differentiates_primary_vs_secondary_and_supports_horizontal_scroll` in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs).
  - Wave 15A/16A/21A/23A/14C/23B tests: `wave15a_tab_options_primary_secondary_custom_color_and_scrollable`, `wave16a_tab_layout_view_pager_two_way_sync`, `wave21a_material_badge_ext_truncation_gravity_offsets_and_rtl`, and Wave 23A `wave23a_view_pager_ext_user_input_margin_velocity_transformers_tabs_rtl_and_reduced_motion` in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`328` passing tests within `923` default / `929` all-features Rust tests across `76` suites), and `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269` plus Wave 23B `test_wave23b_view_pager_ext_rule_305` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`256` passing tests, `0` diffs across `167` existing layouts).
- **Remaining limitations:** Stacked icon+label primary tabs, animated indicator width/position interpolation on XML `TabWidget`, per-tab disabled state, touch drag scrolling on narrow viewports, custom `ViewPager2.PageTransformer` callbacks beyond `zoom_out`/`depth`, runtime `offscreenPageLimit` recycling, and Android device visual parity remain open. See [Android XML import](../android-xml.md) and [`ViewFlipper`](android-viewflipper.md).
