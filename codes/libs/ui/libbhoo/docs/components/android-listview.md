# Android XML ListView

Status: Initial Android XML ListView implementation; Android device styling and additional adapter features remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

ListView supports XML string-array entries, host-supplied labels, a solid divider drawable and divider height, and none, singleChoice and multipleChoice modes. Visible rows are constructed on demand; nonvisible rows are not rendered. Default text rows use a 48dp minimum height and 16dp horizontal padding, and grow for wrapping text as in the Android framework simple list item. Visible rows are measured lazily; unseen rows use estimated heights until reached. The host can bind each label into an XML row template with text, images and layout containers; an opt-in ListView template also supports checkbox, switch and button controls with row-local state; visible wrap-content rows are measured and cached. The host can add multiple fixed XML header and footer views, optionally selectable, and configure header and footer dividers in XML. Colors use the Bhoo theme rather than Android theme resources.

## State and interaction contract

Rows activate on release and report the ListView XML ID and clicked index. XML item roots receive the current pressed, enabled and activated state for supported background color selectors. The host can provide enabled flags and read or change checked indices. `set_list_adapter` validates item IDs and preserves checked items by stable ID across reordering or removes checks for deleted items; without stable IDs, valid positions remain checked. Use the setter for subsequent adapter updates when stable IDs are in use. Single choice replaces the previous choice; multiple choice toggles each row. Touch dragging, fling and mouse wheel scroll the list; dragging does not activate an item. An opt-in item long press fires once after 500ms, suppresses the normal click, and is cancelled by movement, scrolling, disablement or an adapter update. The host reads and removes the event from `list_long_clicked`. Directional, Home, End and page keys move between enabled rows and scroll them into view; Enter and Space activate the focused row. Fixed-view clicks report the header/footer group and its index separately from adapter item indices; fixed views do not participate in checked-item state yet. Fixed headers and footers can contain checkbox, switch and button controls with independent state and events. Other interactive row and fixed-view types, Android adapter recycling lifecycle and device parity remain open. Visible row edits and template image, width or density changes trigger remeasurement. Unseen rows retain estimated or previously measured heights until reached.


## Motion and animation

- **EdgeStretch overscroll**: Integrated via `EdgeStretch` in `codes/libs/ui/libbhoo/src/scroll.rs`. Pull, release, and fling absorb follow AOSP `EdgeEffect` (`TYPE_STRETCH`) constants (`NATURAL_FREQUENCY = 24.657`, `DAMPING_RATIO = 0.98`, etc.).
- **Cancel pointer input**: `Frame::cancel_pointer_input` in `codes/libs/ui/libbhoo/src/systems/material/mod.rs` clears UI active state, sets `input.cancelled`, cancels pending ripple taps `<150 ms`, allows in‑flight ripples to fade `375 ms`.
- **Validation**: `scroll::tests::edge_stretch_pull_release_and_absorb_spring_recoil` (30/30) and `launcher_demo::drag_tests` (14/14) plus filmstrip evidence `context/system-ui-flow/animations/strip-v5-*.png`.
- **Remaining limitations**: Full Android parity for scrollbar thumb styling and scroll snapping remains open.

Touch scrolling uses Bhoo's shared Android spline fling and bounded edge physics. Row interaction uses the shared state layer.

**Reduced motion:** Fling is disabled when reduced motion is requested.

## Acceptance criteria

- Import XML entries, choice mode and supported divider attributes.
- Activate rows, enforce choice mode and disabled items, and avoid clicks during scrolling.
- Hold a row to dispatch one long-click event; cancel when the pointer moves or the adapter changes.
- Exercise a large host adapter with only visible rows rendered.
- Reorder and remove stable-ID items while preserving the correct checked choices.
- Navigate enabled rows and scroll to them with directional and page keys.
- Wrap default long text without rendering offscreen rows in a large adapter.
- Bind and measure XML row templates with images and wrapped text; retain selection and dividers.
- Scroll, measure and activate selectable fixed headers and footers, respecting divider flags.
- Compare appearance, scrolling and selection against an Android device and implement remaining adapter features.

## References

- [Android 15 ListView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/ListView.java)
- [Android 15 AbsListView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/AbsListView.java)

Android framework `simple_list_item_1.xml` and Material platform theme values were checked for default row minimum height and padding. Waydroid was running, but the ADB device was unauthorized during this check; pixel-level device comparison remains open.

The focused example can switch to 100,000 rows. Enabling the XML row layout exercises the same visible-row measurement path while scrolling.

Adapter items can provide sparse per-position text and drawable bindings for XML child IDs. Bindings are isolated between items, remeasure affected visible rows, and follow stable IDs across adapter updates. The example can show individual item content.

A sparse item binding may override the default with a different XML item layout, including opt-in interactive controls and its own primary text ID. Item-specific layouts, text and images follow stable IDs across adapter updates; visible item geometry is remeasured when the override changes.

Interactive row controls report the adapter position and child XML ID separately from row clicks. Checkbox and switch state follows stable adapter IDs across reorder and is removed for deleted items; when stable-ID mode changes, row control state follows positions.

## `RecyclerView` lowering, `RULE_RECYCLER_LAYOUT_EXT = 301` (Wave 22A / Wave 22B) & `RULE_CAROUSEL_LAYOUT = 304` (Wave 23A / Wave 23B)

- **XML importer (`tools/import-android-layout.py`, Wave 22B / Wave 23B):** Lowers `androidx.recyclerview.widget.RecyclerView`, legacy `android.support.v7.widget.RecyclerView` (Wave 22B alias), and `androidx.preference.PreferenceRecyclerView` to `Kind::ListView` when `app:layoutManager` is omitted or resolves to `LinearLayoutManager` or `CarouselLayoutManager` (and to `Kind::GridView` for `GridLayoutManager` or `StaggeredGridLayoutManager`; see [Android XML GridView](android-gridview.md)), with `RECYCLER_VIEW_ATTRS` (`spanCount`, `reverseLayout`, `stackFromEnd`, `fastScrollEnabled`) and `CAROUSEL_ATTRS` (`carousel_strategy`, `carousel_alignment`, `carousel_itemSpacing`) resolved directly and via `@style`. Resolves `app:layoutManager`, `android:orientation` (`vertical` -> `v`, `horizontal` -> `h`), `app:spanCount` (`1..=256`), `app:reverseLayout` (`true`/`false`), and `app:stackFromEnd` (`true`/`false`), emitting `RULE_RECYCLER_LAYOUT_EXT = 301` (`"<mgr>|<orient>|<span_count>|<reverse:0|1>|<stack_end:0|1>"`) whenever non-default (omitted for default vertical linear lists), or `RULE_CAROUSEL_LAYOUT = 304` (`"<strategy:multi_browse|hero|uncontained|full_screen>|<spacing_dp>|<align:start|center>"`) when `CarouselLayoutManager` or `carousel_*` attributes are present (see [Carousel](carousel.md)).
- **Runtime (`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, Wave 22A / Wave 23A):** Evaluates `RULE_RECYCLER_LAYOUT_EXT = 301` on `Kind::ListView` (and `Kind::GridView`) independently against the public Apache-2.0 AndroidX `RecyclerView` / `LinearLayoutManager` API contracts (see [licenses and attribution](../licenses-and-attribution.md)), and evaluates `RULE_CAROUSEL_LAYOUT = 304` for keyline focal sizing, `28dp` rounded mask clipping, and `0.24s` ease-out focal snap (see [Carousel](carousel.md)):
  - **Horizontal orientation (`orient=h`):** Uses `scroll_viewport_horizontal_rtl` for RTL-aware horizontal scrolling (touch drag, fling disabled under reduced motion per [motion.md](../motion.md#motion-profiles), and wheel scroll).
  - **Visual reversal (`reverse=1`):** Reverses visual item layout order along the primary axis while preserving original adapter indices for clicks, selection, and per-item bindings.
  - **End alignment (`stack_end=1`):** Aligns short content flush to the bottom/end of the viewport instead of the top/start.
  - **Staggered grid (`mgr=staggered`):** Greedy shortest-track multi-column/row placement across LTR and RTL (documented further in [Android XML GridView](android-gridview.md)).
- **Validation evidence:**
  - Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`328` passing tests within `923` default / `929` all-features Rust tests across `76` suites): `wave22a_recycler_layout_ext_horizontal_reverse_stack_end_staggered_and_rtl` and Wave 23A `wave23a_carousel_layout_strategies_interpolation_snap_mask_align_rtl_and_reduced_motion`.
  - Python XML importer tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`256` passing tests, `0` diffs across `167` existing layouts): `test_wave22b_recycler_layout_ext_rule_301` and Wave 23B `test_wave23b_carousel_layout_rule_304`.
- **Remaining limitations:** Custom `RecyclerView.LayoutManager` subclasses, `ItemDecoration`, `ItemAnimator`, `SnapHelper`, `ViewHolder` pooling/recycling lifecycle APIs, `fastScrollEnabled` interactive thumb, and Android device visual/gesture parity remain open. See [Android XML import](../android-xml.md) and [Carousel](carousel.md).
