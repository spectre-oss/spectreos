# Android XML NumberPicker

Status: Initial XML wheel with tap, drag, spline fling, repeat-on-hold and host text editing; Android device comparison remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The three-row NumberPicker is used independently and by Android's spinner-mode DatePicker and TimePicker. The initial model holds minimum, maximum and current values, remembers wrap preference across range changes, enables wrapping only when the range spans at least three increments, and supports caller-supplied labels. Invalid range or label changes fail without modifying existing state. The host supplies localized numeric digits for wheel labels and accepts those digits or ASCII digits in the center editor. The center opens a host text editor. Enter or focus loss commits a numeric value or a case-insensitive displayed-label prefix; Escape cancels and empty input keeps the prior value.

The importer accepts NumberPicker and the host supplies the value range and labels, as in Android. The initial wheel displays three rows, responds to taps and touch dragging, and settles to the nearest row. Detailed wrapping arithmetic, other locale-specific parsing, selector appearance, exact fling and long-press parity and device comparison remain open.

## State and interaction contract

Stepping changes the value or returns no change at a nonwrapping boundary. Programmatic selection and bounds updates do not imply a user callback. A host should emit a callback only when user interaction changes the value. The wheel's previous/current/next rows can use the neighbor API.

## Motion and animation

Dragging follows the finger; release may start an Android-derived spline fling, followed by a short cubic settle. Holding an end row starts repeating after 500 ms at 300 ms intervals. Exact timing, velocity tracking and edge effects still need Android comparison.

**Reduced motion:** Value changes and settlement snap.

## Acceptance criteria

- Implement localized numeric input and remaining XML attributes; compare editing, fling and long press with Android.
- Compare wheel geometry, inertia, input behavior and timing with Android.
- Compare DatePicker’s integrated locale-ordered day/month/year wheels and date range rules with Android.
- Deliver a focused Bhoo-only webxdc when the widget is usable.

## References

- [Android 15 NumberPicker source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/NumberPicker.java)
- [Android 15 DatePicker spinner delegate (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DatePickerSpinnerDelegate.java)
