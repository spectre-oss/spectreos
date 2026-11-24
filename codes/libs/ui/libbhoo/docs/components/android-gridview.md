# Android XML GridView

Status: Initial adapter-backed GridView implementation; additional Android behavior and device styling remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

GridView accepts XML column count or auto-fit, column width, horizontal and vertical spacing, stretch mode and choice mode. The host provides adapter labels and optional stable IDs, enabled flags, and an XML item template using the shared ListView adapter state. An opt-in interactive template supports checkbox, switch and button children; their state follows stable item IDs and child activations are reported separately from item clicks. Rows are measured as they become visible, including wrapped labels and images. XML items support match-parent, wrap-content and fixed widths, with start, center, end, left and right horizontal gravity within each column. Row height uses the tallest cell; exact platform behavior for mixed heights and Android theme styling remain open.

## State and interaction contract

Tap a cell to report its adapter index; single-choice and multiple-choice modes track selected items. Disabled cells do not activate. XML item root backgrounds can use color selectors for pressed, activated and disabled states; checked items supply the activated state. An opt-in item long press reports the item after 500ms, suppresses its tap, and cancels on movement, scrolling or adapter updates. Touch dragging, fling and wheel scrolling use the shared scroll physics and only visible rows are built. RTL reverses column order. Directional navigation, custom child interactions, Android adapter recycling lifecycle and device parity remain open.

## Motion and animation

Touch scrolling uses the shared Android spline fling and bounded edge physics; cells use a shared press state layer.

**Reduced motion:** Fling is disabled when reduced motion is requested.

## Acceptance criteria

- Import and validate GridView column, spacing, stretch and choice attributes.
- Lay out auto-fit and fixed-column grids, including RTL, item widths, gravity and wrapped item layouts.
- Scroll and select items with both small and large adapters.
- Hold an item to dispatch one long-click event and cancel it when scrolling.
- Compare visuals and interaction against Android and complete the remaining variants.

## References

- [Android 15 GridView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/GridView.java)
- [Android 15 AbsListView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/AbsListView.java)

Adapter items can provide sparse per-position text and drawable bindings for XML child IDs. Bindings are isolated between items, remeasure affected visible rows, and follow stable IDs across adapter updates. The example can show individual item content.

A sparse item binding may override the default with a different XML item layout, including opt-in interactive controls and its own primary text ID. Item-specific layouts, text and images follow stable IDs across adapter updates; visible item geometry is remeasured when the override changes.

## `RecyclerView` `GridLayoutManager` and `StaggeredGridLayoutManager` support (Wave 22A / Wave 22B)

- **XML importer (`tools/import-android-layout.py`, Wave 22B):** Lowers `androidx.recyclerview.widget.RecyclerView`, legacy `android.support.v7.widget.RecyclerView`, and `androidx.preference.PreferenceRecyclerView` to `Kind::GridView` when `app:layoutManager` names `GridLayoutManager` or `StaggeredGridLayoutManager`, forwarding `app:spanCount` (`1..=256`, direct or via `@style` through `RECYCLER_VIEW_ATTRS`) to `numColumns` when unset, and emits `RULE_RECYCLER_LAYOUT_EXT = 301` (`"<mgr>|<orient>|<span_count>|<reverse:0|1>|<stack_end:0|1>"`) whenever `mgr=staggered`, `orient=h`, `reverse=1`, or `stack_end=1`.
- **Runtime (`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, Wave 22A):** Evaluates `RULE_RECYCLER_LAYOUT_EXT = 301` on `Kind::GridView` independently against the public Apache-2.0 AndroidX `RecyclerView`, `GridLayoutManager`, and `StaggeredGridLayoutManager` contracts (see [licenses and attribution](../licenses-and-attribution.md)):
  - **Staggered grid (`mgr=staggered`):** Places variable-size items greedily into the shortest track across multi-column vertical or multi-row horizontal layouts in both LTR and RTL.
  - **Horizontal scrolling (`orient=h`), `reverse=1` & `stack_end=1`:** Routes horizontal grids through `scroll_viewport_horizontal_rtl` (reduced motion disables fling per [motion.md](../motion.md#motion-profiles)), reverses visual order while preserving original adapter indices when `reverse=1`, and aligns short content to the bottom/end when `stack_end=1`.
- **Validation evidence:**
  - Rust runtime test in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`324` passing tests within `919` default / `925` all-features Rust tests across `76` suites): `wave22a_recycler_layout_ext_horizontal_reverse_stack_end_staggered_and_rtl`.
  - Python XML importer test in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`252` passing tests, `0` diffs across `167` existing layouts): `test_wave22b_recycler_layout_ext_rule_301`.
- **Remaining limitations:** `SpanSizeLookup` multi-span cells, staggered full-span headers/gap-handling strategies, custom `LayoutManager`/`ItemDecoration`/`ItemAnimator`/`ViewHolder` APIs, and Android device visual/gesture parity remain open. See [Android XML ListView](android-listview.md) and [Android XML import](../android-xml.md).
