# Android XML AbsoluteLayout

Status: Initial support for fixed-position children in the deprecated Android container.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework layout |
| Priority | P2 |

## Scope and variants

AbsoluteLayout positions children at physical `layout_x` and `layout_y` coordinates relative to its padding. Android deprecated it because fixed positions do not adapt to screen sizes.

## State and interaction contract

Children can overlap and retain their own state. Fixed and wrap-content sizes are supported with approximate measurement; gone children do not affect the wrap-content bounds. Negative coordinates are supported. Coordinates are physical and do not mirror in right-to-left layouts. Exact Android measurement and non-pixel resource qualifiers remain incomplete.

## Motion and animation

Layout is direct; child controls provide their own interaction motion.

**Reduced motion:** Direct positioning is unchanged.

## Acceptance criteria

- Import fixed x and y coordinates and position children relative to parent padding.
- Compare sizing and clipping on an Android device.

## References

- [Android AbsoluteLayout](https://developer.android.com/reference/android/widget/AbsoluteLayout)
- Android 15 `AbsoluteLayout.java` in `context/reference-sources/android-framework/`, Apache-2.0.
