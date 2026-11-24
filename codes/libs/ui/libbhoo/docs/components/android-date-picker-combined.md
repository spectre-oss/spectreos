# Android XML DatePicker with spinner and calendar

Status: Initial synchronized spinner-mode wheels and embedded calendar; Android device parity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Spinner-mode DatePicker accepts spinnersShown and calendarViewShown. Both default to true, and when both are false the wheels remain visible. When both are enabled, the three locale-ordered wheels and the calendar appear side by side, following the Android legacy layout. The current sizing and styling are approximate.

## State and interaction contract

A date chosen on the calendar updates the wheels; changing a wheel updates the calendar. Browsing a different month alone does not change the selected date. Dates are restricted to the configured range.

## Motion and animation

The calendar and wheels use their shared gesture and settling behavior; comparison with Android remains open.

**Reduced motion:** Page and wheel settlement snap.

## Acceptance criteria

- Verify changes synchronize both ways and scrolling the calendar does not change the selected date.
- Test both visibility attributes, including when both are false.
- Compare the layout, range behavior and motion against Android.

## References

- [Android 15 DatePicker spinner delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DatePickerSpinnerDelegate.java)
- [Android 15 legacy DatePicker layout](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/date_picker_legacy.xml)
