# Android XML Space

Status: Initial Android XML Space support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android Space reserves an empty region in a layout, including fixed-size and weighted gaps.

## State and interaction contract

Space draws no content, including its background. Visible and invisible spaces participate in layout; gone spaces do not. Explicit `minWidth` and `minHeight`, and intrinsic sizes from supported background shapes, contribute to wrap-content size. Fixed dimensions still take precedence. Exact platform measurement constraints remain to be validated on Android.

## Motion and animation

Space has no visual content or built-in animation.

**Reduced motion:** No motion is required.

## Acceptance criteria

- Reserve fixed and weighted gaps without drawing content.
- Remove a gone Space from its parent layout.
- Compare measurement with Android on a device.

## References

- [Android Space source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/master/core/java/android/widget/Space.java)
- [Android Space reference](https://developer.android.com/reference/android/widget/Space)
