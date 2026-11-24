# Android XML progress bar

Status: Initial XML support using Bhoo's classic circular and linear progress drawing.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

An Android XML ProgressBar with circular or horizontal presentation and determinate or indeterminate state.

## State and interaction contract

The caller owns numeric progress by view ID. The default Android XML style is indeterminate circular; the recognized horizontal style defaults to determinate. Horizontal bars can show a secondary progress value behind the main value. Explicit `android:mirrorForRtl="true"` mirrors horizontal progress in RTL; theme-specific defaults are not yet resolved.

`android:progressTint`, `progressBackgroundTint`, `secondaryProgressTint` and `indeterminateTint` accept literal colors, color resources and supported color-state selectors. The primary and secondary tints affect their respective determinate layers; the indeterminate tint affects the animated indicator. Selectors respond to enabled, pressed, focused, selected, activated and hovered view states. The default Bhoo colors remain in use when an attribute is absent. Custom drawables, tint modes, theme color attributes and full theme resolution remain unsupported; tinting Bhoo's built-in shapes does not reproduce arbitrary Android drawable composition. The determinate tint attributes are also supported on SeekBar and RatingBar.

## Motion and animation

Indeterminate indicators use Bhoo's classic progress timeline, and determinate updates animate using its progress transition.

**Reduced motion:** Indeterminate progress is shown without continuous motion.

## Acceptance criteria

- Import default and horizontal styles and initial progress.
- Change the progress and verify the bar updates.
- Check independently colored primary, secondary and track layers and the indeterminate indicator; select the view and verify its selector changes the primary tint.
- Compare appearance and animation with Android on a device.

## References

- [Pinned AOSP ProgressBar source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/ProgressBar.java)
- [Android ProgressBar reference](https://developer.android.com/reference/android/widget/ProgressBar)
