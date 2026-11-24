# Navigation drawer

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Persistent or modal destination list.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Support persistent/modal forms; modal form owns dismissal, focus containment and focus restoration.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Slide from the logical leading edge with a scrim; persistent width changes use layout motion.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Open and cancel in RTL, then verify focus returns to the invoking control.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/NavigationDrawer.md)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/navigationdrawer/navigation-drawer-overview.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [NavigationView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigation/NavigationView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/navigation/NavigationView.java) |
| Material Components for Android | [DrawerLayoutUtils.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigation/DrawerLayoutUtils.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/navigation/DrawerLayoutUtils.java) |
| Compose Material 3 | [NavigationDrawer.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationDrawer.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationDrawer.kt) |
| Angular Material | [drawer.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/sidenav/drawer.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/sidenav/drawer.ts) |
| Flutter material_ui | [navigation_drawer.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/navigation_drawer.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/navigation_drawer.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: NavigationDrawer.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/NavigationDrawer.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Navigation drawer preview](http://127.0.0.1:8766/#/navigation-drawer).

Demonstrates: Persistent destination-list selection.

Remaining scope: Adaptive placement and drawer-specific modal variant.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android modal drawer geometry used by the Thunderbird demo

The [pinned Android guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/NavigationDrawer.md) and its [modal reference image](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/navigationdrawer/navigation-drawer-modal.png) describe the container, header, headline, icon, label, active indicator and scrim. The demo uses a surface-container-low modal surface, square leading edge, 16dp trailing corners, 56dp rows with 12dp indicator insets, 28dp icon inset, 24dp Material symbols and 12dp icon-to-label spacing. The selected label is bold; selection, scrim dismissal, focus restoration and reduced motion use shared Bhoo interactions. The header and section title are application content.

There is a discrepancy in the pinned upstream source: the guide's table states a 280dp maximum width, while the actual `Widget.Material3.NavigationView` style references `m3_comp_navigation_drawer_container_width`, whose pinned generated token is **360dp**. This implementation follows the runtime style/token and reserves 64dp for the modal scrim on narrow screens. The actual Android layout can be configured by the application. The guide now recommends an expanded navigation rail for the Material 3 Expressive update; the Thunderbird fixture retains a modal drawer because its source layout uses a `DrawerLayout`.

The opening animation uses Bhoo's existing interruptible side-sheet motion. Predictive back and full Android accessibility integration are not implemented in this demo.

## Android XML importer and runtime support (Wave 7A / Wave 7B / Wave 8A / Wave 8B / Wave 16A / Wave 16B / Wave 17A / Wave 18A / Wave 20A / Wave 20B / Wave 21A / Wave 21B)

- **`NavigationView` (`RULE_NAV_BAR = 237` `"drawer|labeled"`, `RULE_NAV_ITEM = 238`, `RULE_NAV_BADGE = 244`, `RULE_MATERIAL_BADGE_EXT = 297`, & Wave 16B/17A `app:headerLayout` inflation + runtime rendering):** `tools/import-android-layout.py` lowers `com.google.android.material.navigation.NavigationView` to `Kind::Linear`, inflates `app:menu="@menu/..."` into ordered `RULE_NAV_ITEM = 238` (plus `RULE_NAV_BADGE = 244` and Wave 21B `RULE_MATERIAL_BADGE_EXT = 297`) rules, and (Wave 16B) inflates `app:headerLayout="@layout/..."` (direct or inherited via `@style`, supporting `<merge>`, DataBinding `<layout>` unwrapping, AOSP resource qualifiers including Wave 18B `--locale`/`--ldrtl`/`--ldltr`, and cycle detection) as leading child views ahead of menu destinations. At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, `NavBarKind::Drawer` renders `300dp` wrap width, `surfaceContainerLow` surface, (Wave 17A) inflated `app:headerLayout` children sequentially above a `1dp` `t.outline_variant` divider before the `12dp`-padded `56dp`-tall drawer rows (keeping interactive controls inside header layouts clickable, reserving header heights + margins + `1dp` divider in `Size::Wrap` measurement, and enabling clipped `f.scroll_viewport` vertical scrolling via `LayoutState.offsets` when header + items overflow the viewport), `12dp` horizontal row inset, full-pill (`28dp` radius) `secondaryContainer` active indicator, `24dp` leading (RTL-trailing) icon, `14sp` label, Wave 21A `RULE_NAV_BADGE = 244` / `RULE_MATERIAL_BADGE_EXT = 297` drawer item badges (with `maxCharacterCount` numeric truncation, 4-corner `badgeGravity`, and RTL-mirrored dp offsets; see [Badge](badge.md)), and interactive `LayoutState.selected_index` selection.
- **`DrawerLayout` (`RULE_DRAWER_LAYOUT = 247`, `RULE_DRAWER_LOCK_MODE = 291`, `LayoutState.drawer_open`, & `LayoutState.drawer_lock_mode`, Wave 8A / Wave 8B / Wave 16A / Wave 18A / Wave 20A / Wave 20B):** `tools/import-android-layout.py` lowers `androidx.drawerlayout.widget.DrawerLayout` (including `--merge-parent` and `--custom-view-map`) to `Kind::Frame` with `RelativeRule { kind: 247, anchor: "" }`, preserves explicit child `android:layout_gravity` (`start`, `end`, `left`, `right` with `layout_gravity_explicit: true`), emits `RULE_DRAWER_LOCK_MODE = 291` (`"<unlocked|locked_closed|locked_open>"`) from `app:drawerLockMode` (`"unlocked"`/`"0"`, `"locked_closed"`/`"1"`, `"locked_open"`/`"2"`) on `DrawerLayout` or drawer children (Wave 20B), and strips design-time `tools:openDrawer`.

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, independently implemented against pinned Material Components for Android [`NavigationView.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigation/NavigationView.java), [`DrawerLayoutUtils.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigation/DrawerLayoutUtils.java), and [`navigation/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigation/res/values/tokens.xml) (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE), local `context/reference-repos/material-android/lib/java/com/google/android/material/navigation/`) alongside the public AndroidX `DrawerLayout` API contract (Apache-2.0):

- **Panel identification, dual-edge independence (Wave 18A), and closed state:** Drawer panel children are recognized either by `RULE_NAV_BAR` (`drawer`) or by explicit horizontal `layout_gravity` (`start`/`end`/`left`/`right`) with non-`Match` width (`Node::is_drawer_panel_child`), and partitioned into logical `start` and logical `end` slots (`Node::drawer_panel_is_logical_end`). When both start and end drawers exist in one `DrawerLayout`, `LayoutState.drawer_open["{id}"]` controls only the logical start drawer while `LayoutState.drawer_open["{id}:end"]` controls only the logical end drawer with independent `XmlDrawerMotion` timelines (single end-only drawer layouts continue accepting either `"{id}"` or `"{id}:end"` for backwards compatibility). While a slot is closed (and any close transition has settled), its panels are excluded from measurement and rendering and main content remains fully interactive with no scrim.
- **Open modal state, geometry, and RTL mirroring:** When a drawer slot is open (or sliding/dragging), background content input is disabled (`f.without_input(true, ...)`), a full-bounds translucent scrim `Color(0, 0, 0, 82)` scaled by slide progress is painted over the viewport, and each drawer panel renders at width `min(300dp, viewport_width - 56dp)` (or explicit `Size::Dp` clamped to `viewport_width - 56dp`) with level-1 shadow, `surfaceContainerLow` (`#F7F2FA` in `Material3Light`, `#1D1B20` in `Material3Dark`) fill unless a custom background is set, and `16dp` end-rounded M3 modal corners (`[0, 16, 16, 0]` dp for left-docked panels, mirrored to `[16, 0, 0, 16]` dp for right-docked / RTL `start` / LTR `end` panels).
- **Interaction, animated slide-in/out, drag-to-close (Wave 16A), closed-edge swipe-to-open (Wave 18A) & dismissal:** Clicking inside an open drawer activates drawer items (for example returning a `NavigationView` item XML ID) without triggering scrim dismissal. Opening/closing `LayoutState.drawer_open[key]`, clicking the scrim outside the panel, or pressing `Escape` (`Key::Escape`) animates the drawer slide and scrim opacity over `0.25s` (`XmlDrawerMotion`, snapping immediately under `MotionPreference::Reduced`). Dragging an open drawer horizontally toward its docked edge (`8dp` drag slop, RTL-aware) scrubs drawer progress and scrim opacity directly; releasing past `30%` closed (`(1.0 - progress) > 0.30`) or flicking faster than `500 dp/s` closes `LayoutState.drawer_open[key] = false` over `0.25s`, while shorter or cancelled drags settle back open. When closed, pressing within `24dp` (`state.dp(24.0)`) of a panel's docked physical edge (left edge for LTR `start` / RTL `end`; right edge for LTR `end` / RTL `start`) and dragging inward past `8dp` slop cancels underlying content clicks (`f.ui.active = None`), tracks panel progress and scrim opacity live, and settles open (`LayoutState.drawer_open[key] = true`) on release if `progress > 0.30` or inward velocity exceeds `500 dp/s` (`state.dp(500.0)`), otherwise animating back closed (vertical drags abort edge opening cleanly).
- **Drawer lock modes (`RULE_DRAWER_LOCK_MODE = 291` & `LayoutState::drawer_lock_mode`, Wave 20A + 20B):** Exposes `DRAWER_LOCK_MODE_UNLOCKED (0)`, `DRAWER_LOCK_MODE_LOCKED_CLOSED (1)`, `DRAWER_LOCK_MODE_LOCKED_OPEN (2)`, and `DRAWER_LOCK_MODE_UNDEFINED (3)` alongside `LayoutState::drawer_lock_mode: HashMap<String, u8>` keyed by `"{id}"` (logical start) and `"{id}:end"` (logical end), resolving with precedence runtime `LayoutState::drawer_lock_mode` -> drawer child static `RULE_DRAWER_LOCK_MODE` -> `DrawerLayout` static `RULE_DRAWER_LOCK_MODE` -> `DRAWER_LOCK_MODE_UNLOCKED`. `LOCKED_CLOSED` forces the panel closed and blocks closed-edge swipe-to-open; `LOCKED_OPEN` forces the panel open and blocks drag-to-close, scrim-tap, and `Escape` dismissal across both LTR and RTL (honoring reduced-motion instant transitions).
- **Validation evidence:**
  - Functional Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`320` passing tests; `915` default / `921` all-features Rust tests across `76` suites): Wave 21A `wave21a_material_badge_ext_truncation_gravity_offsets_and_rtl` (verifies `NavigationDrawer` truncated badge rendering with `RULE_MATERIAL_BADGE_EXT = 297`), `wave20a_drawer_lock_mode_static_runtime_dual_rtl_and_reduced_motion` (verifies static `RULE_DRAWER_LOCK_MODE = 291` and runtime `LayoutState::drawer_lock_mode` precedence across `"{id}"` / `"{id}:end"`, `LOCKED_CLOSED` blocking closed-edge swipe-to-open, `LOCKED_OPEN` blocking drag-to-close, scrim-tap, and `Escape` dismissal, dual-edge independence, LTR/RTL mirroring, and reduced-motion behavior), `wave18a_drawer_layout_closed_edge_swipe_and_dual_edge_independence` (verifies independent `"{id}"` vs. `"{id}:end"` programmatic state, ignoring drags starting outside the `24dp` edge zone, live tracking + underlying tap cancellation + close settle on short edge drags, opening past `30%` or via `> 500 dp/s` inward flick on start and end drawers, `Escape` dismissal, and RTL edge mirroring), `wave17a_navigation_drawer_header_layout_divider_and_scrolling` (verifies sequential header layout rendering, `1dp` `outline_variant` divider, clickable header button action, `Size::Wrap` height reservation `120 + 1 + 6×56 + 24 = 481dp`, and clipped `scroll_viewport` overflow scroll), `drawer_layout_rule_hides_closed_drawer_shows_scrim_corners_rtl_and_dismisses`, `wave16a_drawer_layout_animated_slide_scrim_fade_drag_close_and_reduced_motion` (verifies `0.25s` slide-in/out & scrim fade, RTL-aware horizontal drag-to-close, short-drag settle-back, and reduced-motion snap), and `nav_bar_rail_and_drawer_rules_render_m3_indicators_and_update_selection`.
  - Importer unit tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`248` passing tests, plus `0` diffs across `167` existing layouts): Wave 21B `test_wave21b_material_badge_ext_rule_297` (verifies `RULE_MATERIAL_BADGE_EXT = 297` on `NavigationView` menus), `test_wave20b_app_bar_ext_and_drawer_lock_mode_rules_290_291` (verifies `app:drawerLockMode` `"unlocked"`/`"0"`, `"locked_closed"`/`"1"`, `"locked_open"`/`"2"` on `DrawerLayout` and drawer children -> `RULE_DRAWER_LOCK_MODE = 291` emission plus invalid-value rejection), `test_wave8a_swipe_refresh_and_drawer_layout_rules_246_247`, `test_wave7a_nav_bar_and_menu_inflation_rules_237_238`, and `test_wave16b_navigation_view_header_layout` (verifying direct/styled `app:headerLayout`, `<merge>`, DataBinding `<layout>`, qualifiers, and cycle rejection).
  - Static visual captures (Mesa software GLES): Page-2 Light/Dark Material 3 showcases (`context/android-xml-comparison/bhoo-m3-showcase-page2.png`, `bhoo-m3-showcase-page2-dark.png`) and pure-XML Contacts drawer capture (`context/android-xml-comparison/contacts-xml-drawer.png` via [`android_contacts_capture.rs`](../../../../codes/libs/ui/libbhoo/examples/android_contacts_capture.rs), visually verified against `context/contacts-flow/`); no automated pixel-diff against Android devices has been performed.
- **Remaining Android XML limitations:** Focus containment and restoration across arbitrary drawer children, custom XML badge colors, predictive back, and pixel-level Android device comparison remain open. See [Android XML import](../android-xml.md) and [licenses and attribution](../licenses-and-attribution.md).
