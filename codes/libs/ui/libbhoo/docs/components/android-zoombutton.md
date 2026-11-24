# Android XML ZoomButton

Status: Initial support for the deprecated Android zoom button.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P2 |

## Scope and variants

ZoomButton is an ImageButton that repeats its click action when held, sharing ImageButton drawable rendering including Wave 24A/24B sampled `<animated-vector>` `src`/`bg` keyframe playback (`RULE_ANIMATED_VECTOR = 309`, verified on `Kind::Zoom` in `wave24a_animated_vector_src_repeat_oneshot_clip_gating_rtl_mirror_and_reduced_motion`; without per-frame SVG `pathData` morphing — see [ImageButton](android-imagebutton.md)). Android deprecated the widget.

## State and interaction contract

Tap emits one action; a hold starts repeating after 500 ms at one-second intervals until release or cancellation. The host can override the interval in milliseconds via `LayoutState.zoom_speed_ms`, keyed by view ID. Zero is limited to one millisecond to prevent a busy loop. Disabled buttons do not activate. Moving beyond an approximate touch slop before the hold starts cancels the pending action. The host owns the result of each action. Device-specific long-press timeout, touch slop and full long-press lifecycle behavior remain incomplete.

## Motion and animation

The image button uses its normal pressed feedback and shared `Kind::Icon`/`Kind::Zoom` drawable playback (`RULE_ANIMATED_VECTOR = 309` via `f.time()` and clip-gated `f.wake_at()`).

**Reduced motion:** Repeat timing remains available; `<animated-vector>` drawables settle immediately on `len - 1` (`repeat == 0`) or frame `0` (`repeat == 1`) with zero wakeups.

## Acceptance criteria

- Import ZoomButton, draw its image and emit tap and repeated hold actions.
- Verify cancellation, disabled state and compare interaction timing with Android on a device.

## References

- [Android ZoomButton](https://developer.android.com/reference/android/widget/ZoomButton)
- Android 15 `ZoomButton.java` in `context/reference-sources/android-framework/`, Apache-2.0.
