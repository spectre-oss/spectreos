# Navigation bar

Status: Experimental Rust navigation bar available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Primary destination items and selected state.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Control the selected destination and badges; activation changes navigation through the caller, not through animation.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Move or resize the selected indicator and blend labels while committing the destination immediately.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Rapidly switch destinations and verify content, selected state and semantics remain consistent.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomNavigation.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/navigation-bar/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/bottomnav/bottomnav-hero.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [BottomNavigationView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomnavigation/BottomNavigationView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/bottomnavigation/BottomNavigationView.java) |
| Material Components for Android | [NavigationBarView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/navigation/NavigationBarView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/navigation/NavigationBarView.java) |
| Compose Material 3 | [NavigationBar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationBar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationBar.kt) |
| Flutter material_ui | [navigation_bar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/navigation_bar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/navigation_bar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: BottomNavigation.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomNavigation.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Navigation bar preview](http://127.0.0.1:8766/#/navigation-bar).

Demonstrates: Exclusive destination selection, caller-supplied icon glyphs, a labeled badge, corresponding content, RTL ordering and a selected-label-only option. The Rust API also supports disabled destinations and reduced motion. A selected pill expands or contracts using Bhoo's spatial motion, and selection commits immediately. When only the selected label is shown, unselected icons move to the center and labels fade while their accessible names remain available. Badges append their description to the destination's semantic label.

The preview uses filled and outlined Google Material Icons for selected and unselected destinations. Their sampled, triangulated paths are bundled for both Canvas and GLES; these are Material Icons, not the newer Material Symbols. Source revisions, licenses and modifications are recorded in [third-party notices](../../../../../../THIRD_PARTY_NOTICES.md). Arbitrary destinations can still provide text glyphs.

Remaining scope: A general vector icon resource API and Material Symbols, exact Material motion and color tokens, adaptive sizing and platform accessibility validation. This implementation draws on the Apache-2.0 licensed Compose navigation bar reference above for indicator, label placement and state-layer anatomy; it does not claim pixel parity.

The vertical indicator uses the 56 × 32 dp dimensions in the local Compose `NavigationBarVerticalItemTokens.kt` snapshot. The selected label uses the bar's foreground rather than the indicator's foreground; this is an approximation until Bhoo has the separate Material secondary and on-surface color roles. The Rust semantic nodes use tab roles, with Tab and Enter available for keyboard selection; platform accessibility bridging and a complete grouped navigation model remain open.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML importer and runtime support (Wave 7A / Wave 7B / Wave 8A / Wave 8B / Wave 21A / Wave 21B)

`tools/import-android-layout.py` lowers `com.google.android.material.bottomnavigation.BottomNavigationView` and `com.google.android.material.navigation.NavigationBarView` to `Kind::Linear` with `RULE_NAV_BAR = 237` (`"bar|<auto|labeled|selected|unlabeled>"`, defaulting to `auto` unless `app:labelVisibilityMode` is set), and shares the same `RULE_NAV_BAR = 237` / `RULE_NAV_ITEM = 238` / `RULE_NAV_BADGE = 244` / `RULE_MATERIAL_BADGE_EXT = 297` protocol with `com.google.android.material.navigationrail.NavigationRailView` (`rail`, see [Navigation rail](navigation-rail.md)) and `com.google.android.material.navigation.NavigationView` (`drawer|labeled`, see [Navigation drawer](navigation-drawer.md)).

When `app:menu="@menu/<name>"` is present and `res/menu/<name>.xml` is available in `--resources`, the importer inflates the menu into ordered repeatable `RULE_NAV_ITEM = 238` (`"<id>|<title>|<icon_name>"`) rules — handling `<group>`/`<item>` `android:visible="false"` exclusion, nested `<menu>` submenus, `android:orderInCategory` sorting, `@string/` title resolution, and icon stem normalization to bundled `MaterialIcon` names — extracts `app:badgeText` on visible identified `<item>` elements (resolving `@string/`, ignoring `@null`, hidden items, and items without `android:id`) into repeatable `RULE_NAV_BADGE = 244` (`"<item_id>:<badge_text>"`, where `"<item_id>:"` denotes a dot badge), and in Wave 21B (`parse_badge_ext_attrs`) additionally resolves `app:badgeGravity`, `app:badgeMaxCharacterCount`, `app:badgeHorizontalOffset`, and `app:badgeVerticalOffset` (per item or inherited from container defaults, keyed by sorted 0-based visible item index) into repeatable `RULE_MATERIAL_BADGE_EXT = 297` (`"<idx>|<gravity>|<max_chars>|<hoff_dp>,<voff_dp>"`). `@menu/@null` or missing menu files degrade cleanly without emitting `238`/`244`, while invalid menu references, non-`<menu>` roots, malformed item IDs, or invalid badge attributes fail compilation strictly.

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, informed by pinned `BottomNavigationView.java`, `NavigationBarView.java`, `BadgeDrawable.java`, `navigation/res/values/tokens.xml`, and `badge/res/values/tokens.xml` (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE), local `context/reference-repos/material-android/lib/java/com/google/android/material/navigation/` and `badge/`):

- **Geometry and visual tokens (`bar`):** `80dp` wrap height (density-scaled via `LayoutState.effective_density()`), M3 surface (`#F3EDF7` in `Material3Light`, `#211F26` in `Material3Dark`), RTL-aware equal destination slots, centered `64 × 32dp` `secondaryContainer` active pill (`16dp` radius), `24dp` icon (`onSecondaryContainer` active / `onSurfaceVariant` inactive, using tinted `LayoutState.images` drawables when registered or falling back to bundled `MaterialIcon`), and `12sp` label (`700` active `onSurface` / `500` inactive `onSurfaceVariant`) following `auto` (`<= 3` items or selected) / `labeled` / `selected` / `unlabeled`. Shared variants render `rail` (`80dp` wrap width, `56 × 32dp` pill, `56dp`/`48dp` slots) and `drawer` (`300dp` wrap width, `surfaceContainerLow`, `56dp`-tall rows with `12dp` horizontal inset and full-pill `28dp`-radius indicator).
- **Badges (`RULE_NAV_BADGE = 244`, `RULE_MATERIAL_BADGE_EXT = 297`, & `LayoutState.nav_badges`, Wave 8A / Wave 8B / Wave 21A / Wave 21B):** Across `NavigationBar` (`bar`), `NavigationRail` (`rail`), and `NavigationDrawer` (`drawer`), `LayoutState.nav_badges: HashMap<String, String>` (keyed by item XML ID) takes runtime precedence over declarative `RULE_NAV_BADGE = 244` (`Node::nav_badge_for`). Empty text renders a density-scaled `6dp` circular M3 `error` dot (`#B3261E` Light / `#F2B8B5` Dark); non-empty text renders a `16dp`-tall M3 `error` pill (`8dp` radius, width `max(text_width + 8dp, 16dp)`) with centered `11sp` bold `onError` (`#FFFFFF` Light / `#601410` Dark) label, formatted via `RULE_MATERIAL_BADGE_EXT = 297` `maxCharacterCount` numeric truncation (`4` -> `"999+"`, `3` -> `"99+"`, `2` -> `"9+"`) and anchored to any of the four RTL-mirrored corners (`top_end`/`top_start`/`bottom_end`/`bottom_start`) plus mirrored horizontal and vertical dp offsets, modulated by disabled alpha. See [Badge](badge.md).
- **Interaction and motion:** Clicking any slot updates `LayoutState.selected_index` (keyed by container XML ID) immediately and returns the clicked destination's XML ID (or the container ID if unnamed). The Android XML runtime currently commits the active pill and badge updates immediately without spring or crossfade animation (matching reduced-motion snap behavior; animated transitions remain open and follow [motion.md](../motion.md#motion-profiles)).
- **Validation evidence:**
  - Functional Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`320` tests within `915` default / `921` all-features Rust tests across `76` suites): `nav_bar_rail_and_drawer_rules_render_m3_indicators_and_update_selection`, `nav_bar_and_rail_badges_render_dot_pill_precedence_and_rtl`, and Wave 21A `wave21a_material_badge_ext_truncation_gravity_offsets_and_rtl`.
  - Importer unit tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`248` passing tests, `0` diffs across `167` existing layouts): `test_wave7a_nav_bar_and_menu_inflation_rules_237_238`, `test_wave8a_nav_badge_rule_244`, and Wave 21B `test_wave21b_material_badge_ext_rule_297`.
  - Visual showcase captures: bottom `NavigationBar` rendered in both Light (`context/android-xml-comparison/bhoo-m3-showcase.png`, `Material3Light`) and Dark (`context/android-xml-comparison/bhoo-m3-showcase-dark.png`, `Material3Dark`) via [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) under headless Weston + Mesa software GLES (static renders without badges; side-by-side Waydroid Material 3 pixel-parity proof remains open).
- **Remaining Android XML limitations:** Custom XML badge colors and separate `maxNumber` (beyond `maxCharacterCount`), runtime application of custom `itemIconTint`/`itemTextColor`/`itemActiveIndicatorStyle`/`itemRippleColor`/`itemTextAppearance*` attributes, `app:headerLayout` inflation on `NavigationRailView`, animated active-indicator and badge transitions, adaptive horizontal-item layouts, and Waydroid M3 visual comparison remain open. See [Android XML import](../android-xml.md) and [Validation and Roadmap](../validation.md).
