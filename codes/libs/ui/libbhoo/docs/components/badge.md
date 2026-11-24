# Badge

Status: Experimental anchored dot and label badges with mirrored placement and owner descriptions. Native accessibility integration, announcements and full state/visual verification remain open.

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Dot or count/label attached to another element.

`Frame::badge_at` places a 6dp dot or a 16dp-high label of one to four characters relative to an icon; RTL placement mirrors the trailing edge. The owner is laid out independently and the badge creates no additional focus target.

## State and interaction contract

Expose a dot or count through its owning control's description; do not create a new focus target.

The caller provides a localized description and calls `badge_at` after emitting the owning semantic node. Bhoo appends the description to that node; callers control visibility and counts. Native accessibility announcement and localization are not yet implemented.

## Motion and animation

Current badge updates are immediate and static; count-change crossfade and interruption have not been implemented or validated. Avoid continuous pulsing or moving the owning control.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Update from one to multiple digits and verify the parent action retains its size policy and accessible name.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BadgeDrawable.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/badges/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/badge/small-badge-hero.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

The pinned AndroidX [badge tokens](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/BadgeTokens.kt) specify error/on-error colors and 6dp/16dp sizes. Bhoo independently implements placement informed by the [badge layout reference](https://github.com/androidx/androidx/blob/e2398675db4ea10f83f11e49abf45d11fb0c9e0c/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt).

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [BadgeDrawable.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/badge/BadgeDrawable.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/badge/BadgeDrawable.java) |
| Compose Material 3 | [Badge.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt) |
| Angular Material | [badge.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/badge/badge.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/badge/badge.ts) |
| Flutter material_ui | [badge.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/badge.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/badge.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: BadgeDrawable.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BadgeDrawable.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Numeric overflow

`BadgeContent::Number { value, max }` renders the nonnegative decimal value through `max`, then renders the configured maximum followed by `+` when exceeded (for example, 999 then 999+). Its pill grows to fit the formatted text without changing its owner's bounds, and the existing RTL anchor mirrors it. This API uses ASCII digits; callers needing localized digits can supply a formatted `Label` within that API's limits. Updates are immediate, with no count-change animation. Android's configurable character limit, locale formatting and exact theme/motion parity remain open. The behavior is informed by the pinned Apache-2.0 `BadgeDrawable.java` reference above, stored at `context/reference-repos/material-android/lib/java/com/google/android/material/badge/BadgeDrawable.java`.

## Text overflow

`BadgeContent::Text { text, max_graphemes }` displays nonempty text up to the supplied nonzero limit, replacing the last visible slot with `…` if more text follows. The pill fits the resulting text and mirrors with the icon in RTL. Bhoo counts Unicode grapheme clusters to avoid cutting a combined character; Android's referenced implementation counts UTF-16 code units, so truncation can differ for complex scripts and emoji. This is an explicit API alongside the unchanged four-character `Label` option. Shaping, localized overflow formatting and exact Android visual parity remain open. The pinned Apache-2.0 `BadgeDrawable.java` reference above supplies the overflow pattern.

## Logical anchor corners

`Frame::badge_at_gravity` accepts `BadgeGravity::TopStart`, `TopEnd`, `BottomStart` or `BottomEnd`; logical start/end mirror in RTL and bottom placement reflects the top placement around the icon. The existing `badge_at` continues to use `TopEnd`. Dot, text and numeric badges share the same corner options. Bhoo's existing dimensions and offsets are retained, so Android's customizable insets and exact visual tokens remain unverified. The pinned Apache-2.0 Android BadgeDrawable source above documents the four gravity choices.

## Anchor offsets

`Frame::badge_at_with_options` accepts `BadgeOptions` with a logical corner, RTL direction and horizontal/vertical offsets in dp. Positive offsets move toward the icon center; negative offsets move outward. Non-finite offsets are rejected. Defaults preserve `badge_at` and `badge_at_gravity` placement. This is an additional offset relative to Bhoo's existing base placement, not a claim of Android's default token values. The pinned Apache-2.0 `BadgeDrawable.java` reference above defines the toward-center offset direction; separate Android with/without-text offsets and device parity remain open.

## Android XML importer and runtime support (Wave 8A / Wave 8B / Wave 21A / Wave 21B)

During `res/menu/*.xml` inflation in `tools/import-android-layout.py` (for `BottomNavigationView`, `NavigationBarView`, `NavigationRailView`, `NavigationView`, and `BottomAppBar`), visible `<item>` elements with an `android:id` and `app:badgeText` (resolving `@string/` references; ignoring `@null`, `android:visible="false"`, and items without an ID) emit ordered repeatable `RULE_NAV_BADGE = 244` (`"<item_id>:<badge_text>"`, where an empty value `"<item_id>:"` selects a dot badge). In Wave 21B, `parse_badge_ext_attrs` additionally resolves `app:badgeGravity` (`top_end`/`top_start`/`bottom_end`/`bottom_start`), `app:badgeMaxCharacterCount` (`> 0`, default `4`), `app:badgeHorizontalOffset`, and `app:badgeVerticalOffset` (direct, `@integer/`, `@dimen/`, `@style`) both in `parse_nav_menu` (emitting sorted 0-based visible item indices and inheriting container defaults) and on standalone views (`self`), emitting repeatable `RULE_MATERIAL_BADGE_EXT = 297` (`"<idx|self>|<gravity>|<max_chars>|<hoff_dp>,<voff_dp>"`, omitted when unset, strictly rejecting invalid gravities/counts/offsets).

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, independently implemented against pinned Material Components for Android [`BadgeDrawable.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/badge/BadgeDrawable.java) and [`badge/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/badge/res/values/tokens.xml) (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE), local `context/reference-repos/material-android/lib/java/com/google/android/material/badge/`):

- **Runtime precedence & scope (`RULE_NAV_BADGE = 244`, `RULE_MATERIAL_BADGE_EXT = 297`, `LayoutState.nav_badges`):** Wired across `NavigationBar` (`bar`), `NavigationRail` (`rail`), `NavigationDrawer` (`drawer`), `TabWidget` (0-based tab index), `BottomAppBar` actions, and standalone views (`RULE_NAV_BADGE` `"self:..."` / `"-1:..."` or `LayoutState.nav_badges` keyed by the standalone view's XML ID when `RULE_MATERIAL_BADGE_EXT = 297` targets `self`/`-1`). `LayoutState.nav_badges: HashMap<String, String>` takes precedence over declarative `RULE_NAV_BADGE = 244` on navigation items so hosts can update counts or toggle dot/pill badges dynamically.
- **Numeric `maxCharacterCount` truncation (`RULE_MATERIAL_BADGE_EXT = 297`, Wave 21A):** Formats integer badge values matching Android `BadgeDrawable` `maxCharacterCount`: `4` -> `"999+"` (e.g., `1250` -> `"999+"`), `3` -> `"99+"` (e.g., `105` -> `"99+"`), `2` -> `"9+"` (e.g., `12` -> `"9+"`).
- **Visual tokens, 4-corner RTL anchoring & mirrored offsets (`paint_nav_badge`):**
  - Empty text `""`: density-scaled `6dp` circular (`3dp` radius) M3 `error` dot (`#B3261E` in `Material3Light`, `#F2B8B5` in `Material3Dark`).
  - Non-empty text (e.g., `"999+"`, `"99+"`, `"9+"`, `"4"`): density-scaled `16dp`-tall full-pill (`8dp` radius, width `max(text_width + 8dp, 16dp)`) M3 `error` container with centered `11sp` bold (`weight: 700`) `onError` (`#FFFFFF` Light, `#601410` Dark) label.
  - Placement: supports all four logical corners (`top_end` default, `top_start`, `bottom_end`, `bottom_start`) mirrored in RTL, plus density-scaled horizontal (`hoff_dp`, mirrored in RTL so LTR and RTL centers remain symmetric) and vertical (`voff_dp`) offsets, modulated by disabled alpha (`0.38`) without altering slot bounds or adding extra focus targets.
- **Motion:** Immediate static updates (consistent with reduced-motion behavior in [motion.md](../motion.md#motion-profiles)); count-change crossfades remain open.
- **Validation evidence:**
  - Functional Rust runtime tests (`320` passing tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) within `915` default / `921` all-features Rust tests across `76` suites): `nav_bar_and_rail_badges_render_dot_pill_precedence_and_rtl` (verifies declarative `6dp` dot and `16dp` `"99+"` pill badges, RTL top-left mirroring, `LayoutState.nav_badges` runtime override, Light/Dark M3 `error`/`onError` tokens, and `NavigationRail` rendering) and Wave 21A `wave21a_material_badge_ext_truncation_gravity_offsets_and_rtl` (verifies `maxCharacterCount` truncation `4` -> `"999+"`, `3` -> `"99+"`, `2` -> `"9+"`, all four `badgeGravity` corners, horizontal/vertical dp offsets with exact LTR/RTL mirror symmetry, and badge rendering across `NavigationBar`, `NavigationDrawer`, `TabWidget`, `BottomAppBar`, and standalone `Button` views).
  - Importer unit tests (`248` passing tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), `0` diffs across `167` existing layouts): `test_wave8a_nav_badge_rule_244` and Wave 21B `test_wave21b_material_badge_ext_rule_297` (verifies `RULE_MATERIAL_BADGE_EXT = 297` across `BottomNavigationView`, `NavigationRailView`, `NavigationView`, `BottomAppBar`, sorted `orderInCategory` menu items, hidden-item omission, container defaults, `@dimen/`/`@integer/`/`@style`, standalone `self` targets, and strict rejection of invalid gravities/counts/offsets).
  - Functional unit tests only; no side-by-side Waydroid Material 3 visual comparison has been performed for XML badges.
- **Remaining Android XML limitations:** Custom XML badge background/text colors, separate `maxNumber` (beyond `maxCharacterCount`), locale digit formatting, semantic description attachment on XML navigation items, animated badge updates, and Waydroid visual parity remain open. See [Android XML import](../android-xml.md), [Navigation bar](navigation-bar.md), [Navigation drawer](navigation-drawer.md), [Bottom app bar](bottom-app-bar.md), [Tabs](tabs.md), and [licenses and attribution](../licenses-and-attribution.md).
