# Carousel

Status: Experimental Rust uncontained, viewport-wide, centered-hero and multi-browse layouts with drag, snapping and virtualization. Exact Material keyline behavior remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Direct manipulation](../motion.md#motion-profiles) |

## Scope and variants

Scrolling collection with item sizing and alignment behavior.

`Frame::carousel` provides fixed-width uncontained and viewport-wide layouts, and
an initial `CenteredHero` layout with a large centered item and smaller neighbors.
The caller owns `CarouselState`; items can use images or colors, with RTL placement,
optional uncontained snapping and mandatory viewport-wide/hero snapping. The browser
example supplies layout switches, Previous, Next and Show all controls. The hero
layout interpolates position and width as the user drags, shifts toward the edges
at the first and last item, and omits visual text on very narrow neighbors while
retaining their semantic labels. This is an approximation of the Apache-2.0 Compose
keyline strategy, not its arrangement solver, edge keylines, mask interpolation,
parallax, or a pixel-parity claim. The initial multi-browse layout arranges large,
medium and small masks where space allows, interpolates them as items move, and
mirrors the end arrangement. It is a separate approximation rather than a port of
Compose's arrangement solver and keyline shifting. Wider viewports can fit
multiple large items before the smaller item; their count is bounded by the
available items and visible items remain virtualized. Rust tests cover wide
arrangements and intermediate end transitions; the browser test exercises a
600 px viewport. Exact arrangement priorities and edge behavior remain open.
For hero and multi-browse image items, Bhoo fits the image to the large item
first and crops its centered source coordinates as the visible mask narrows;
it does not rescale the image to each smaller mask. A Rust draw-command test
checks the source crop against the mask size. Full parallax, edge-specific masks
and upstream visual parity remain open.
The current hero interpolation preserves item order and spacing between snapped
positions. Rust tests sample the drag at nine intermediate positions in both
directions, and the browser test checks intermediate canvas semantics during a
right-to-left drag; these tests do not establish upstream motion parity.

## State and interaction contract

Track visible item, scroll offset and snap policy; provide keyboard and non-gesture controls.

The caller owns items and state. Visible items have stable actionable IDs;
dragging cancels the pressed item, arrow keys skip disabled items, and Home/End
move focus to the first/last enabled item. Fully hidden items are not emitted
as semantic nodes. Rust and browser tests cover focus navigation, including
worker and main-thread one-canvas routing. The collection and its visible
items expose distinct semantic roles and positions. The browser playground and
one-canvas hosts mirror visible items into a labeled carousel region with
actionable slide buttons; keyboard commands and activation are sent to Rust.
Tab transfers keyboard focus from the canvas to a slide button and back to the
canvas when the next target is outside the carousel; Shift+Tab reverses it.
`tools/test-bhoo-canvas-carousel-a11y.mjs` checks labels, Tab/Shift+Tab, End
navigation and activation in the playground and both one-canvas rendering modes. Actual
screen-reader use, a native accessibility bridge, scrollbar semantics and
complete disabled visual treatment remain open.

## Motion and animation

Scroll follows direct manipulation; optional snapping uses an initial 300ms cubic curve without autoplay. It is an approximation, not a claim of exact Material physics or parallax. Resizing retargets an in-flight snap.

Use the shared [direct manipulation profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Retain immediate user-controlled movement; disable optional inertial or programmatic travel where practical.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Resize while snapping and verify the selected item stays visible and reduced motion disables decorative travel.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Carousel.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/carousel/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/carousel/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/carousel/fullscreen-anatomy.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [CarouselLayoutManager.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/carousel/CarouselLayoutManager.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/carousel/CarouselLayoutManager.java) |
| Compose Material 3 | [Carousel.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/carousel/Carousel.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/carousel/Carousel.kt) |
| Flutter material_ui | [carousel.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/carousel.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/carousel.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: Carousel.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Carousel.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Carousel preview](http://127.0.0.1:8766/#/carousel).

Demonstrates: Uncontained, viewport-wide, initial centered-hero and multi-browse layouts with drag, wheel, arrow controls, RTL placement, image items and a Show all list.

The `carousel_named` API supplies a custom accessible region name, and the web adapter updates it when it changes. Accessible slide buttons transfer keyboard focus to the canvas, including its visible focus state. Virtualized slide controls retain their logical order when earlier slides return to view. Screen-reader device verification remains open.

Remaining scope: Exact multi-browse/hero arrangement and masking,
variable aspect ratios, parallax, exact snap physics, platform accessibility and image descriptions.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML `MaskableFrameLayout` support (`RULE_CAROUSEL_ITEM = 273`, Wave 14C / Wave 15A)

- `tools/import-android-layout.py` lowers `com.google.android.material.carousel.MaskableFrameLayout` to `Kind::Frame` emitting `RULE_CAROUSEL_ITEM = 273` (`anchor: ""`, composing with `app:shapeAppearance` and automatic `RULE_CLIP_TO_OUTLINE = 234`).
- At runtime in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `RULE_CAROUSEL_ITEM` clips child draw commands to the item's rounded outline, defaulting to `28dp` corner radius when no explicit radius or `RULE_CORNER_RADII` overrides it.

### Android XML `CarouselLayoutManager` support (`RULE_CAROUSEL_LAYOUT = 304`, Wave 23A / Wave 23B)

- **XML importer (`tools/import-android-layout.py`, Wave 23B):** Adds `CAROUSEL_ATTRS` (`carousel_strategy`, `carousel_alignment`, `carousel_itemSpacing`) across `RecyclerView`, `HorizontalScrollView`, and `LinearLayout`, detecting `CarouselLayoutManager` (via `app:layoutManager`) or explicit `carousel_*` attributes and emitting `RULE_CAROUSEL_LAYOUT = 304` (`"<strategy:multi_browse|hero|uncontained|full_screen>|<spacing_dp>|<align:start|center>"`).
- **Runtime ([`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../../src/systems/android/xml/mod.rs), Wave 23A):** Evaluates `RULE_CAROUSEL_LAYOUT = 304` on `Kind::ListView`, `Kind::GridView`, `Kind::HorizontalScroll`, and `Kind::Linear` independently against pinned Apache-2.0 Material Components for Android `CarouselLayoutManager.java`, `MultiBrowseCarouselStrategy.java`, `HeroCarouselStrategy.java`, `UncontainedCarouselStrategy.java`, and `FullScreenCarouselStrategy.java` (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, local `context/reference-sources/material-android/lib/java/com/google/android/material/carousel/`; see [licenses and attribution](../licenses-and-attribution.md)):
  - Supports horizontal scrolling and pointer dragging with smooth `0.24s` ease-out focal snapping via bounded `LayoutState.carousel_motion` (`64`-entry cap), snapping immediately under `MotionPreference::Reduced` per [motion.md](../motion.md#motion-profiles).
  - Computes dynamic scroll-interpolated keyline widths across all four Material strategies (`multi_browse` `55%/26%/12%`, `hero` `78%/18%`, `uncontained` `60%`, and `full_screen` `100%`), applies `28dp` rounded mask clipping to items, honors `align=start|center`, and mirrors layout and drag direction in RTL across both child nodes and `state.list_items`.
- **Validation:**
  - Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`328` passing tests within `923` default / `929` all-features Rust tests across `76` suites): `wave15a_floating_toolbar_button_group_split_button_and_carousel_item` and Wave 23A `wave23a_carousel_layout_strategies_interpolation_snap_mask_align_rtl_and_reduced_motion`.
  - Python importer tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`256` passing tests, `0` diffs across `167` existing layouts): `test_wave14c_m3_expressive_tags_rules_270_to_273` and Wave 23B `test_wave23b_carousel_layout_rule_304`.
- **Remaining Android XML limitations:** Custom `CarouselStrategy` subclasses, `OnMaskChangedListener` mask-rect callbacks, vertical carousel orientation, full parallax image cropping, and Android device visual/physics parity remain open. See [Android XML import](../android-xml.md) and [Android XML ListView](android-listview.md).
