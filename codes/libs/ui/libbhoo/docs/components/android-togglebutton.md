# Android XML toggle button

Status: Initial XML support for on/off labels and checked state using Bhoo rendering.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android XML ToggleButton with independent on/off labels and enabled or disabled state.

## State and interaction contract

The `android:checked`, `android:textOn` and `android:textOff` attributes set initial state and labels. Activation toggles caller-owned state by view ID. Explicit `android:textAllCaps`, `textStyle="bold"` and letter spacing apply to both state labels. `android:disabledAlpha` sets the opacity of Bhoo's indicator when disabled, defaulting to Android's 0.5; enabled indicators are unaffected. The background and text retain their separate disabled treatment. Custom Android drawable and theme resolution are not supported.

## Motion and animation

The button uses Bhoo's shared press state layer; the checked indicator changes immediately.

**Reduced motion:** The checked state changes immediately.

## Acceptance criteria

- Import initial state and both labels.
- Toggle repeatedly and verify state persists; disabled controls stay unchanged.
- Check that both state labels receive explicit text styling and disabledAlpha changes only the indicator opacity.
- Compare appearance and animation with Android on a device.

## Validation and visual status

Importer and Rust rendering tests verify styled state labels and disabled indicator opacity, alongside toggling. The focused webxdc exposes full and zero indicator opacity for review. Android device pixel and motion parity are not established; Bhoo draws its own background and indicator.

## Guidelines and visual references

- [Android ToggleButton reference](https://developer.android.com/reference/android/widget/ToggleButton); no verified side-by-side screenshot is available for this change.

## Downloaded reference implementations

- Android 15 [ToggleButton.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/ToggleButton.java), local `context/reference-sources/android-framework/ToggleButton.java` (Apache-2.0), for state labels and disabled indicator alpha. Bhoo renders its own button independently.

## References

- [Android ToggleButton reference](https://developer.android.com/reference/android/widget/ToggleButton)
- [Pinned AOSP ToggleButton source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/ToggleButton.java)

## Wrap-content measurement

A wrap-content ToggleButton measures the current checked state's `textOn` or `textOff`, including supported case transformation, text style, letter spacing and horizontal padding, then respects Bhoo's 48 dp minimum and the available width. The width is recalculated after state changes in supported linear and frame-style containers. Android theme/background minimums, font rendering and device visual parity remain unverified. The Apache-2.0 Android 15 ToggleButton source above supplies the state-label behavior.
