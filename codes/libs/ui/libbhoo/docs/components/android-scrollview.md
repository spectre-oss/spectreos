# Android XML ScrollView

Status: Initial vertical XML scrolling support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android ScrollView hosts a single vertical scrolling child, including a wrapped text layout.

## State and interaction contract

Scroll position is retained by view ID. The child is measured with available width so nested wrapped text contributes to the scroll range. The content clips to the padding by default (`scroll_input = if clip_to_padding_disabled { b } else { inner }`); `android:clipToPadding="false"` lets it draw into and receive scrolling in that padding. `Kind::Scroll` clips child rendering and pointer hit-testing unconditionally via `f.within(scroll_input, ...)` even when padding is zero (`inner == b`) and restores `f.input` after scroll viewport handling, preventing offscreen scrolled children outside the viewport from receiving stray pointer clicks (covered by `overlapping_frame_sibling_and_clickable_container_child_hit_boundaries` in [`tests/android_xml.rs`](../../tests/android_xml.rs)). Exact Android measurement, scroll physics, overscroll and scrollbar appearance remain incomplete.

## Motion and animation

Touch and wheel scrolling use Bhoo viewport behavior.

**Reduced motion:** Direct scrolling remains available.

## Acceptance criteria

- Include wrapped child text in the scroll extent.
- Reach content at the end of the scrollable child.
- Compare scroll motion and layout on Android.

## References

- [Android ScrollView reference](https://developer.android.com/reference/android/widget/ScrollView)

## Updated Motion and Animation Details

- **EdgeStretch overscroll**: Integrated via `EdgeStretch` (see `codes/libs/ui/libbhoo/src/scroll.rs`). Pull, release, and fling absorb behavior follow AOSP `EdgeEffect` (`TYPE_STRETCH`). Constants: `NATURAL_FREQUENCY = 24.657`, `DAMPING_RATIO = 0.98`, `LINEAR_STRETCH_INTENSITY = 0.016`, `EXP_STRETCH_INTENSITY = 0.016`, `SCROLL_DIST_AFFECTED_BY_EXP_STRETCH = 0.33`.
- **Cancellation semantics**: `Frame::cancel_pointer_input` (see `codes/libs/ui/libbhoo/src/systems/material/mod.rs`) clears active UI state and cancels pending ripple taps (`<150 ms`) while allowing in‑flight ripples to fade (`375 ms`).
- **Validation evidence**: `scroll::tests::edge_stretch_pull_release_and_absorb_spring_recoil` (30/30) and `launcher_demo::drag_tests` (14/14). Filmstrip references `context/system-ui-flow/animations/strip-v5-*.png`.
- **Remaining limitations**: Full Android parity for scrollbars, scroll snapping, and scroll bar thumb styling not yet implemented.
