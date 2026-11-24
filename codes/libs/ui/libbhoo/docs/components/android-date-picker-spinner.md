# Android XML DatePicker spinner mode

Status: Initial XML spinner mode; Android device parity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The explicit spinner mode displays locale-ordered day, month and year NumberPicker wheels. The day and month carry across boundaries, and all fields respect the configured date range. Tap, drag, fling, hold and text entry reuse NumberPicker. spinnersShown and calendarViewShown control the wheels and synchronized calendar. Detailed styling and platform comparison remain open.

## State and interaction contract

Changes to any wheel update the date and report the DatePicker ID. Date bounds narrow the wheel ranges at the endpoints. Year changes use the platform spinner delegate's lenient calendar semantics.

## Motion and animation

Wheels use shared NumberPicker drag, fling and settle behavior, which still requires device comparison.

**Reduced motion:** Values and settling snap.

## Acceptance criteria

- Select and edit each field, including wrapping across a month and year.
- Verify locale order and date bounds.
- Compare motion, presentation and input behavior with Android.

## References

- [Android 15 DatePicker spinner delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DatePickerSpinnerDelegate.java)
- [Android 15 NumberPicker](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/NumberPicker.java)
