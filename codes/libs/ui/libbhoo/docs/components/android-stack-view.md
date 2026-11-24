# Android XML StackView

Status: Initial adapter-backed stack and vertical swipe behavior; full Android motion and device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The XML importer accepts an empty StackView and the boolean `android:loopViews` attribute. The host supplies adapter labels and an optional noninteractive XML row template using the existing list adapter state. Up to four items are shown with an initial Android-inspired perspective offset; rendering borrows the adapter instead of copying all its rows per frame. Position-based adapter replacement avoids building a separate set of all row positions. Empty adapters are safe. By default selection stops at the ends; `loopViews="true"` or a host override enables wrapping.

## State and interaction contract

A downward swipe advances; an upward swipe reverses. The gesture begins after an 8dp touch slop, locks its initial direction and uses the release position for the selection threshold. A mouse wheel moves through items with Android's 100 ms pacing. At a non-looping end, a swipe moves the card only a limited distance and returns it. A short or cancelled swipe returns to the current item. Tapping within the front card reports the adapter position; taps in the exposed margins do not. Hosts can advance or reverse using stack_step. Replacing adapter data cancels a pending press or transition. Platform-configured touch slop, multi-pointer gestures, remote adapters, item virtualization, exact Android drawing order and platform lifecycle remain open.

## Motion and animation

A rejected swipe settles back. A downward swipe slides the current card out; an upward swipe brings the previous card in from below. The moving card follows a fraction of the finger's sideways movement, fades, and uses a vertical projection of Android's X rotation. Programmatic previous and next changes use the corresponding transitions. Swipe finishing durations depend on remaining travel distance. This is an initial approximation; perspective distortion, velocity-dependent durations, highlight and press feedback remain open.

**Reduced motion:** Completed and rejected gestures settle immediately.

## Acceptance criteria

- Import a StackView, supply adapter rows and swipe in both directions.
- Keep the selection on short and cancelled swipes and report front-card taps.
- Implement Android motion and adapter details and compare against Android on a device.

## References

- [Android 15 StackView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/StackView.java)
