# Android XML ViewFlipper

Status: Initial XML automatic child flipping support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android ViewFlipper automatically cycles through child views at a configured interval.

## State and interaction contract

`autoStart` and `flipInterval` initialize a host-controlled running state and timer. The timer stops scheduling while paused or outside the viewport and restarts its interval after an application pause. Full Android attachment and visibility lifecycle parity remains incomplete.

## Motion and animation

Supported XML alpha, translate, Wave 16A/16B `<scale>` and `<rotate>` (`RULE_SCALE_IN = 274`, `RULE_SCALE_OUT = 275`, `RULE_ROTATE_IN = 276`, `RULE_ROTATE_OUT = 277`, emitted from `res/anim/*.xml` and `res/animator/*.xml` `scaleX`/`scaleY`/`rotation`), and Wave 19A/19B `elevation` / `translationZ` object-animator resources (`RULE_ELEVATION_IN = 288`, `RULE_ELEVATION_OUT = 289`, `"<from_dp>,<to_dp>|<dur_ms>|<offset_ms>|<interp>"`, supporting sequential/together `<set>` combining `elevation` + `translationZ`, density-aware `px` -> `dp` scaling, and all 7 `AnimInterp` curves: `linear`, `accelerate`, `decelerate`, `fast_out_slow_in`, `overshoot`, `anticipate`, `accelerate_decelerate`) animate child changes via balanced `DrawCommand::PushTransform` / `PopTransform` calls and continuous `paint_elevation_dp` interpolation (suppressing static child `RULE_ELEVATION` during active transitions to prevent double shadows).

**Reduced motion:** The views continue cycling without transition motion (scale/rotate/translate/alpha transitions cut immediately, `RULE_ELEVATION_IN`/`RULE_ELEVATION_OUT` snap immediately to `to_dp`, and `ViewPager`/`ViewPager2` page settles snap immediately without `0.25s` travel).

## `ViewPager` / `ViewPager2` lowering (`RULE_VIEW_PAGER = 248` / `RULE_VIEW_PAGER_EXT = 305`, Wave 9B / Wave 10A / Wave 16A / Wave 23A / Wave 23B)

- **XML importer ([`tools/import-android-layout.py`](../../../../tools/import-android-layout.py), Wave 10A / Wave 23B):** Lowers `androidx.viewpager2.widget.ViewPager2`, `androidx.viewpager.widget.ViewPager`, and (Wave 23B) legacy `android.support.v4.view.ViewPager` to `Kind::ViewFlipper` emitting `RULE_VIEW_PAGER = 248`. Wave 23B adds `VIEW_PAGER_ATTRS` (`userInputEnabled`, `pageMargin`, `pageTransformer`, `offscreenPageLimit`), resolving `android:orientation` (`h`/`v`), `app:userInputEnabled`, `app:pageMargin`, and `app:pageTransformer` (`none`/`zoom_out`/`depth`), strictly validating `app:offscreenPageLimit`, and emitting `RULE_VIEW_PAGER_EXT = 305` (`"<orient>|<user_input:0|1>|<page_margin_dp>|<transformer>"`) whenever non-default.
- **Runtime ([`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../../src/systems/android/xml/mod.rs), Wave 9B / Wave 16A / Wave 23A):** Independently implements `RULE_VIEW_PAGER = 248` and `RULE_VIEW_PAGER_EXT = 305` against the public Apache-2.0 AndroidX `ViewPager` / `ViewPager2` contracts (see [licenses and attribution](../licenses-and-attribution.md)):
  - **Interactive drag-to-scrub & settle:** Supports horizontal (RTL-aware) and vertical pointer/touch drag-to-scrub (`8dp` drag slop, advancing page on release past `> 30%` viewport drag or `> 400 dp/s` flick velocity, settling smoothly over `0.25s` ease-out, or snapping immediately under `MotionPreference::Reduced` per [motion.md](../motion.md#motion-profiles)), synchronized bidirectionally with `LayoutState::tab_pager_links`, `displayed_children`, and `selected_index`, alongside page-indicator dots. See [Tabs](tabs.md).
  - **Input lock, inter-page margin & page transformers:** When `user_input == 0`, blocks user drag and keyboard page-switching while preserving programmatic and linked `TabLayout` switches; inserts density-scaled `page_margin_dp` gaps between adjacent pages during scrub/settle; and applies `zoom_out` and `depth` page transformers via balanced `DrawCommand::PushTransform` / `PopTransform` and `DrawCommand::PushLayer` / `PopLayer` passes.

## Acceptance criteria

- Start, pause and resume automatic child flipping.
- Respect the configured interval and avoid continuous redraw while waiting.
- Compare lifecycle, drag-to-scrub paging, transformers, and transition behavior on Android.

## Validation

- Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`328` passing tests within `923` default / `929` all-features Rust tests across `76` suites): `wave16a_scale_and_rotate_transition_animations_and_reduced_motion`, `wave16a_tab_layout_view_pager_two_way_sync`, `view_pager_rule_supports_horizontal_vertical_swipe_rtl_state_sync_and_dots`, `wave19a_elevation_object_animator_transitions_and_reduced_motion`, and Wave 23A `wave23a_view_pager_ext_user_input_margin_velocity_transformers_tabs_rtl_and_reduced_motion`.
- Python XML importer tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`256` passing tests, `0` diffs across `167` existing layouts): `test_wave16b_scale_and_rotate_transition_rules_274_to_277`, `test_wave10a_view_pager_rule_248`, `test_wave19b_elevation_and_translation_z_animators_rules_288_289`, and Wave 23B `test_wave23b_view_pager_ext_rule_305`.
- **Remaining limitations:** Full Android window-attachment lifecycle, arbitrary custom `<pathInterpolator>` curves beyond the 7 `AnimInterp` curves, `repeatCount`/`repeatMode` runtime playback, `fillBefore`/`fillAfter` retention, custom `ViewPager2.PageTransformer` callbacks beyond `zoom_out`/`depth`, runtime `offscreenPageLimit` / `FragmentStateAdapter` page recycling, `fakeDrag`, and Android device timing/visual parity remain open. See [Android XML import](../android-xml.md) and [Tabs](tabs.md).

## References

- [Android ViewFlipper reference](https://developer.android.com/reference/android/widget/ViewFlipper)
- [AndroidX ViewPager2 reference](https://developer.android.com/reference/androidx/viewpager2/widget/ViewPager2)
