# Android XML rating bar

Status: Initial XML support for fractional ratings and read-only indicators.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android RatingBar with configurable number of stars, fractional rating, step size and indicator mode.

## State and interaction contract

The caller owns the rating by view ID. Pointer selection follows the configured step size. Indicator mode is read-only. The importer currently limits star count to 20; custom drawables remain unsupported. Explicit `android:mirrorForRtl="true"` mirrors fill and interaction in RTL; Android theme-specific defaults are not yet resolved.

`android:progressTint`, `progressBackgroundTint` and `secondaryProgressTint` accept literal colors, color resources and supported state selectors. When a secondary tint is present it fills each star through the next whole-star boundary, underneath the fractional primary fill, following Android's secondary-progress rule. The default appearance remains unchanged when no tint is provided. Indeterminate mode and tint, inherited thumb tint, tint blend modes, arbitrary drawables and theme attribute resolution remain unsupported.

## Motion and animation

The rating follows direct pointer movement. Stars use a filled Material Icons vector, with a muted background and clipped fractional fill. Exact Android bitmap and theme fidelity remains unverified.

**Reduced motion:** Direct manipulation remains immediate.

## Acceptance criteria

- Import star count, rating, step size and indicator state.
- Drag across half-star steps and confirm the indicator remains read-only.
- Check primary, secondary and background star colors, including the remaining half of a partially filled star and the pressed state selector.
- Compare appearance and behavior with Android on a device.

## Validation and visual status

Importer and Rust tests cover the three layers, pressed tint and dragging. The focused webxdc exposes a read-only indicator and RTL interaction for review. No Android device comparison has verified pixel or motion parity; Bhoo currently uses a Material icon vector instead of the Android framework rating bitmap.

## Guidelines and visual references

- [Android RatingBar reference](https://developer.android.com/reference/android/widget/RatingBar); no verified side-by-side screenshot is available for this change.

## Downloaded reference implementations

- Android 15 [RatingBar.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/RatingBar.java), local `context/reference-sources/android-framework/RatingBar.java` (Apache-2.0), specifies secondary progress through the next whole star.
- Android 15 [ProgressBar.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/ProgressBar.java), local `context/reference-sources/android-framework/ProgressBar.java` (Apache-2.0), specifies inherited tint attributes. Bhoo renders its own vector and independently implements the layer coloring.

## References

- [Material Icons filled grade asset (Apache-2.0)](https://github.com/google/material-design-icons/blob/master/src/action/grade/materialicons/24px.svg)
- [Android RatingBar reference](https://developer.android.com/reference/android/widget/RatingBar)
