# Android XML Gallery

Status: Initial implementation of the deprecated Android Gallery; Android device parity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (deprecated) |
| Priority | P1 |

## Scope and variants

An empty Gallery XML element accepts host-provided adapter labels and an optional existing XML row template. The selected item is centered with neighboring items visible. The importer supports `spacing`, `unselectedAlpha`, and `animationDuration`; explicitly sized XML row templates set the item width. Other item sizing and Android theme defaults remain approximations.

## State and interaction contract

Tap an item to select and report its position, or drag horizontally to select an item. After tapping, left and right arrows select the adjacent item with direction reversed in right-to-left layouts; Enter invokes the selected item. A cancelled drag returns to its prior selection. Replacing adapter data cancels an active gesture or fling, retains selection by stable ID where possible, and clamps selection when items disappear. Empty adapters are safe. A fast drag continues with a velocity-based fling before centering an item. Right-to-left layout reverses item order and swipe direction. Exact Android velocity tracking, variable width children, adapter callbacks and device parity remain open.

## Motion and animation

A tap or completed drag settles the selected item to the center using an initial decelerating approximation with the XML duration (400 ms by default). Fling distance, duration and sampled interpolation follow Android Scroller's default friction formula and spline. Velocity estimation, edge behavior and final centering still need Android device comparison.

**Reduced motion:** Selection settles immediately.

## Acceptance criteria

- Import Gallery and provide adapter rows.
- Center selected items on tap and horizontal drag.
- Implement remaining platform behavior and compare with an Android device.

## References

- [Android 15 Gallery source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/Gallery.java)
