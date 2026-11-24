# Search

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Search entry, suggestions, results surface, clear action.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Track query, active result, loading/error state and clear action; Enter activates the selected result and Escape follows an explicit dismissal policy.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Keep query updates immediate; crossfade result-state changes without animating every result or blocking typing.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Replace results while navigating by keyboard and preserve active identity when the result still exists.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Search.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/search/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/search/appbar-with-search.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [text-field.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/textfield/internal/text-field.ts) | [Downloaded file](../../../../context/reference-sources/material-web/textfield/internal/text-field.ts) |
| Material Components for Android | [SearchBar.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/search/SearchBar.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/search/SearchBar.java) |
| Material Components for Android | [SearchView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/search/SearchView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/search/SearchView.java) |
| Compose Material 3 | [SearchBar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt) |
| Angular Material | [autocomplete.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/autocomplete/autocomplete.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/autocomplete/autocomplete.ts) |
| Flutter material_ui | [search_anchor.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/search_anchor.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/search_anchor.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: Search.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Search.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Search preview](http://127.0.0.1:8766/#/search).

Demonstrates: Editable query, filtered application results, empty result and clear.

Remaining scope: Full expanded-search overlay and suggestion history.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML importer and runtime support (Wave 7A / Wave 7B / Wave 23A / Wave 23B)

`tools/import-android-layout.py` lowers `com.google.android.material.search.SearchBar` to `Kind::SearchView` with `RULE_SEARCH_BAR = 239` carrying the resolved hint text (`android:hint`, `app:hint`, or `queryHint`, normalizing `@null` to `""` and also forwarding it to `kind: 107` `queryHint`), accepting Material 3 `SearchBar` attributes (`app:strokeColor`, `app:strokeWidth`, `app:textCentered`, `app:liftOnScroll`, `app:navigationIcon`, `app:menu`, etc.) and `?attr/materialSearchBarStyle`. In Wave 23B, `com.google.android.material.search.SearchView` lowers to `Kind::Frame` (supporting child result containers and `--merge-parent`), adds `SEARCH_VIEW_ATTRS` (`autoShowKeyboard`, `hideNavigationIcon`, `animateNavigationIcon`, `animateMenuItems`, `useDrawerArrowDrawable`, `headerLayout`, `backHandlingEnabled`) to `style_attributes()`, resolves `app:layout_anchor` (stripping `@+id/`/`@id/` without emitting a conflicting `RULE_COORDINATOR_ANCHOR = 227`), `android:hint`, `app:hideNavigationIcon`, and `app:useDrawerArrowDrawable`, validates `autoShowKeyboard`, `animateNavigationIcon`, `animateMenuItems`, `backHandlingEnabled`, `headerLayout`, `backgroundTint`, `elevation`, and `text`, and emits `RULE_SEARCH_VIEW_EXT = 302` (`"<anchor_bar_id>|<hint>|<hide_nav:0|1>|<use_drawer_arrow:0|1>"`). `androidx.appcompat.widget.SearchView` continues to lower to `Kind::SearchView` without `RULE_SEARCH_BAR` or `RULE_SEARCH_VIEW_EXT`.

At runtime in [`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../../src/systems/android/xml/mod.rs), `RULE_SEARCH_BAR = 239` and Wave 23A `RULE_SEARCH_VIEW_EXT = 302` are independently implemented against pinned `SearchBar.java`, `SearchView.java`, and `search/res/values/tokens.xml` (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, Apache-2.0, local `context/reference-repos/material-android/lib/java/com/google/android/material/search/res/values/tokens.xml` and `context/reference-sources/material-android/lib/java/com/google/android/material/search/{SearchBar,SearchView}.java`; see [licenses and attribution](../licenses-and-attribution.md)):

- **`SearchBar` geometry and visual tokens (`RULE_SEARCH_BAR = 239`):** `56dp` default wrap height (`240dp` default horizontal wrap width, scaled by `LayoutState.effective_density()`), full-pill corner radius (`height * 0.5` unless overridden by `radius > 0`), `surfaceContainerHigh` fill (`#ECE6F0` in `Material3Light`, `#2B2930` in `Material3Dark`, overridden by explicit `background` or `backgroundTint`), optional `RULE_ELEVATION = 193` shadow, `24dp` leading `MaterialIcon::Search` at `16dp` leading inset (mirrored to the physical right edge when `LayoutState.rtl` is true), and `16sp` `onSurfaceVariant` hint text replaced by `onSurface` live query text from `LayoutState.text`. Clicking focuses the search bar, sets `LayoutState.search_open` and `LayoutState.search_focus_requested` (and opens any linked `RULE_SEARCH_VIEW_EXT = 302` overlay whose `anchor_bar_id` matches the bar's ID via `search_bar_links`), and returns the node's XML ID; pressing Enter while focused submits via `LayoutState.submit_search`.
- **`SearchView` fullscreen/modal overlay (`RULE_SEARCH_VIEW_EXT = 302`, Wave 23A):** Couples `RULE_SEARCH_BAR = 239` clicks to `LayoutState.search_open[sv_id] = true`, animates an M3 `surfaceContainerHigh` overlay over `0.24s` ease-out via bounded `LayoutState.search_view_motion` (`64`-entry cap; snaps instantaneously under `MotionPreference::Reduced` per [motion.md](../motion.md#motion-profiles)), renders an RTL-mirrored `56dp` header (Back button unless `hide_nav == 1`, interactive `Role::TextField` backed by `LayoutState.text`, trailing Clear button, and `Key::Escape` close), paints a `1dp` `outline_variant` divider below the header, and renders clipped child result containers while avoiding lower-sibling pointer occlusion when closed (`blocks_pointer_for_lower_siblings_in`).
- **Validation evidence:**
  - Functional Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`328` passing tests within `923` default / `929` all-features Rust tests across `76` suites): `search_bar_rule_renders_m3_pill_surface_icon_hint_query_and_focus` (verifies `56dp` height, full-pill `surfaceContainerHigh` light/dark colors, leading `MaterialIcon::Search`, RTL mirroring, hint/query substitution, and click focus/action) and Wave 23A `wave23a_search_view_ext_bar_coupling_animation_controls_rtl_and_reduced_motion` (verifies `SearchBar` coupling, `0.24s` ease-out overlay animation, Back/Clear/`Escape` controls, `hide_nav`, RTL header mirroring, closed non-occlusion, and reduced-motion snap).
  - Importer unit tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`256` passing tests, `0` diffs across `167` existing layouts): `test_wave7a_search_bar_rule_239` and Wave 23B `test_wave23b_search_view_ext_rule_302`.
  - Visual showcase captures: `SearchBar` is rendered in both Light (`context/android-xml-comparison/bhoo-m3-showcase.png`, `Material3Light`) and Dark (`context/android-xml-comparison/bhoo-m3-showcase-dark.png`, `Material3Dark`) via [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) under headless Weston + Mesa software GLES (static renders, not side-by-side Waydroid Material 3 pixel-parity proof).
- **Remaining Android XML limitations:** Custom leading navigation icons, trailing menu/avatar actions, `app:textCentered` runtime centering, container-transform morphing from the exact `SearchBar` rect, `app:headerLayout` inflation inside `SearchView`, `autoShowKeyboard` IME dispatch, `animateNavigationIcon` drawer-arrow morphing, built-in suggestion history persistence, and Waydroid M3 visual comparison remain open. See [Android XML import](../android-xml.md), [FrameLayout](android-frame-layout.md), and [Validation and Roadmap](../validation.md).

## Native M3 `Frame::search_bar_overlay` and `SearchOverlayState` (Wave 15B)

`codes/libs/ui/libbhoo/src/systems/material/mod.rs` (re-exported at the crate root in [`lib.rs`](../../src/lib.rs)) provides `SearchOverlayState { open: bool, query: String, selected: Option<usize> }` and `Frame::search_bar_overlay(id, state, placeholder, suggestions) -> Option<usize>`:

- **Visual tokens & container-transform motion:** Closed state renders a `56dp`-high M3 pill `SearchBar` (`28dp` radius, `surface_high` / `surfaceContainerHigh`). Tapping expands via an animated container transform under `MotionPreference::Full` (snapping immediately under `MotionPreference::Reduced`) into a `28dp`-rounded `surface_high` `SearchView` with a `1dp` `outline_variant` header divider, `"Back"` navigation button, conditional `"Clear"` button when `query` is non-empty, and filtered `Role::ListItem` suggestion rows.
- **Interaction:** Supports direct keyboard typing to filter suggestions case-insensitively, clicking a suggestion row (returning its original slice index, updating `state.selected` and `state.query`, and closing), clicking `"Clear"` (clearing `query` and `selected` while staying open), and dismissing via `"Back"` or `Key::Escape` (`has_escape_target()`).
- **Validation:** `search_bar_overlay_expands_filters_selects_clears_and_dismisses`, `search_bar_overlay_animates_container_transform_under_full_motion`, and `crate_root_reexports_wave14b_and_wave15b_types` in [`tests/wave15b_composition.rs`](../../tests/wave15b_composition.rs).
- **Remaining native limitations:** Persistent search history, keyboard `Up`/`Down`/`Enter` suggestion traversal, full-screen docked/mobile breakpoint switching, trailing avatar/voice slots, and device visual parity remain open.
