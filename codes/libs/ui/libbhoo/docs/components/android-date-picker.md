# Android XML DatePicker

Status: Initial explicit calendar and spinner modes; Android device parity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The importer accepts DatePicker with explicit android:datePickerMode="calendar" or "spinner", minDate, maxDate and firstDayOfWeek. The picker combines a date header with the CalendarView month grid and a scrollable year list. Changing year retains the month and day, clamps the day for leap years, clamps to the configured range, and returns to the day view. Calendar labels and layout direction use the host locale.

Header theme overrides, some input methods and precise device appearance remain open. The importer rejects missing or invalid modes rather than assuming the caller’s theme.

## State and interaction contract

The spinner-mode date model follows the Android delegate: changing the day at a month boundary carries to the adjacent month; changing December to January or vice versa carries the year; month changes clamp the day; year changes use the delegate’s lenient calendar behavior (February 29 becomes March 1 in a common year). Date bounds clamp the result and narrow the relevant wheel ranges at the exact endpoints. Spinner mode uses three NumberPicker wheels ordered by the host locale, with taps, dragging, fling, hold and center-field editing. spinnersShown and calendarViewShown control the wheels and the synchronized embedded calendar.

Selecting a date or a year reports the DatePicker ID. Paging between months does not change selection. The year list centers the selected year on entry and scrolls using the shared touch scroll engine. The host supplies locale-specific weekday, month and day labels and ordering for the header. Programmatic changes use the contained calendar state.

## Motion and animation

Month paging uses CalendarView motion. Switching between year and day lists uses the framework’s 250 ms fade, with quadratic accelerate-in and decelerate-out curves.

**Reduced motion:** Month paging snaps; view switches snap.

## Acceptance criteria

- Select a day, open the year list, choose another year and verify leap-day and date-range clamping.
- Import explicit calendar-mode XML and reject unsupported spinner mode.
- Compare spinner and calendar modes, header formatting, view-switch animation and appearance against Android.

## References

- [Android 15 DatePicker](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DatePicker.java)
- [Android 15 calendar delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DatePickerCalendarDelegate.java)
- [Android 15 spinner delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DatePickerSpinnerDelegate.java)
- [Android 15 year picker](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/YearPickerView.java)

- [Android 15 fade-in resource](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/anim/date_picker_fade_in_material.xml)
- [Android 15 fade-out resource](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/anim/date_picker_fade_out_material.xml)

## Material 3 theme chrome (Wave 15A)

Under `AndroidTheme::Material3Light` and `AndroidTheme::Material3Dark` in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `Kind::DatePicker` replaces the legacy Material 2 solid primary header with a `28dp`-rounded `surfaceContainerHigh` container and renders an outlined `primary` ring around today's date when another day is selected. Verified by `wave15a_m3_date_picker_and_time_picker_chrome` in [`tests/android_xml.rs`](../../tests/android_xml.rs). Material 3 modal text-input toggle, date-range mode in XML, and Android device visual parity remain open.
