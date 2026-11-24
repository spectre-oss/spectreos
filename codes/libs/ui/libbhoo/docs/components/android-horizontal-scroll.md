# Android horizontal scroll

Status: Partial Android XML implementation with touch, wheel, logical RTL start and shared fling physics.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework container |
| Priority | P1 |

## Scope and variants

A one-child `HorizontalScrollView` containing content wider or narrower than its viewport, with optional `fillViewport` sizing.

## State and interaction contract

The `fillViewport` XML flag controls whether short content expands to the viewport, accounting for child margins. Child margins also contribute to the scroll range. The caller owns the scroll position, represented as distance from the logical start. In a right-to-left layout, the initial visible edge is on the right, short children align to that edge, and a rightward touch drag advances toward the other end. In a left-to-right layout the initial edge and touch direction are reversed. Horizontal wheel/trackpad deltas follow physical horizontal direction, reversing their effect on the logical offset in RTL; a vertical mouse wheel also advances logically through this example. Multiple scroll views keep independent positions.

Changing layout direction cancels any active horizontal fling or drag and retains the clamped logical position, preventing momentum from the old coordinate direction from moving the newly mirrored content. `Kind::HorizontalScroll` shares `Kind::Scroll`'s unconditional viewport clipping (`f.within(scroll_input, ...)`) even when padding is zero (`inner == b`) and restores `f.input` after scroll viewport handling so offscreen scrolled children outside the viewport do not receive pointer hits. The example's Android XML buttons use centered default labels, matching the referenced Android Material button style; explicitly supplied logical start/end gravity follows layout direction.

## Motion and animation

Touch scrolling uses Bhoo's existing scroll and fling controller.

**Reduced motion:** Touch release stops without a fling.

## Acceptance criteria

- Import a one-child Android XML layout and reject multiple children.
- Clip content to the viewport and clamp direct scrolling to the valid range.
- Exercise dragging, fling and reduced motion on a device.
- Verify initial and short-content placement, touch direction and end clamping in both layout directions.

## Validation

Rust Android XML tests exercise logical start, touch and physical horizontal wheel direction, end clamping, direction changes during a fling, short-content placement, and button label alignment. Focused browser tests exercise two independent example rows with phone-sized touch input. The user accepted linked RTL retest `horizontal-scroll-rtl-r2` on 2026-11-12, resolving `issue-horizontal-scroll-rtl-r1`; this acceptance does not establish broader Android parity. Exact Android direction-change behavior, wheel conventions, physics, platform styling and device-level parity remain open.

## Guidelines and visual references

The [Android HorizontalScrollView reference](https://developer.android.com/reference/android/widget/HorizontalScrollView) describes the one-child container. No verified upstream visual example for the RTL case is recorded; the Bhoo preview is a test illustration.

## Downloaded reference implementation

The [Android 15 HorizontalScrollView snapshot](../../../../context/reference-sources/android-framework/HorizontalScrollView.java) retains its Apache-2.0 license header. The [pinned AOSP source](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/HorizontalScrollView.java) was consulted for initial RTL scroll position. See the [license and attribution record](../licenses-and-attribution.md).

## References

- [Android HorizontalScrollView reference](https://developer.android.com/reference/android/widget/HorizontalScrollView)

## Updated Motion and Animation Details

- **EdgeStretch overscroll**: Integrated via `EdgeStretch` (see `codes/libs/ui/libbhoo/src/scroll.rs`). Pull, release, and fling absorb behavior follow AOSP `EdgeEffect` (`TYPE_STRETCH`). Constants: `NATURAL_FREQUENCY = 24.657`, `DAMPING_RATIO = 0.98`, `LINEAR_STRETCH_INTENSITY = 0.016`, `EXP_STRETCH_INTENSITY = 0.016`, `SCROLL_DIST_AFFECTED_BY_EXP_STRETCH = 0.33`.
- **Cancellation semantics**: `Frame::cancel_pointer_input` (see `codes/libs/ui/libbhoo/src/systems/material/mod.rs`) clears active UI state and cancels pending ripple taps (`<150 ms`) while allowing in‑flight ripples to fade (`375 ms`).
- **Validation evidence**: `scroll::tests::edge_stretch_pull_release_and_absorb_spring_recoil` (30/30) and `launcher_demo::drag_tests` (14/14). Filmstrip references `context/system-ui-flow/animations/strip-v5-*.png`.
- **Remaining limitations**: Full Android parity for scrollbars, scroll snapping, and scroll bar thumb styling not yet implemented.
