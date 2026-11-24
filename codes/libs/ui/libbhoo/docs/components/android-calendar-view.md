# Android XML CalendarView

Status: Initial interactive XML calendar with touch paging and host-provided Gregorian localization; Android device comparison remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android's Material CalendarView delegates to DayPickerView and a paged SimpleMonthView. The initial XML widget supports minDate, maxDate and firstDayOfWeek, and draws a navigable month grid. Its inset, row and text sizes and selected-day circle follow the Android Material framework dimension resources; today receives the accent color when not selected, and unavailable navigation buttons are hidden. Its calendar model tracks selection separately from the displayed month, enforces date bounds, computes Gregorian day positions for a configurable Android first-day-of-week value, and leaves out-of-range dates visible but disabled. The host must supply a local civil date; the current model uses a proleptic Gregorian calendar. The browser supplies localized Gregorian month and weekday labels, digits, month/year order and the locale week start when the browser exposes it. The host supplies locale layout direction so the weekday grid, arrows and swipe direction mirror in right-to-left locales. Explicit XML layout direction and firstDayOfWeek override the host defaults; older browsers without week information fall back to Sunday. Non-Gregorian calendars, platform styling, the legacy Holo delegate, other XML attributes, device parity and full gesture physics remain open.

## State and interaction contract

Programmatic date setting clamps to the range and shows the selected month. Page navigation changes the visible month without changing the selected date. A user selection must fall in the visible month and within the inclusive range. Changing the range clamps the selection and shows its month. This model does not emit callbacks; the widget integration must do so only for user selections.

## Motion and animation

Horizontal dragging moves adjacent month pages with the finger. A short fast fling or a longer drag changes the page; cancellation returns to the original page. Arrows and released drags settle with a quintic ease informed by Android ViewPager. Timing and gesture physics are approximate and still need device comparison.

**Reduced motion:** Month changes and release settlement snap immediately.

## Acceptance criteria

- Import CalendarView XML including range and first-day-of-week attributes, rejecting unsupported attributes explicitly.
- Display localized month and weekday labels and date cells; enforce range and selected state.
- Verify month navigation, paging gestures, motion and appearance against Android.
- Deliver a focused, interactive Bhoo-only webxdc after the widget is usable.

## References

- [Android 15 CalendarView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/CalendarView.java)
- [Android 15 DayPickerView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DayPickerView.java)
- [Android 15 SimpleMonthView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/SimpleMonthView.java)

- [Android 15 ViewPager](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/com/android/internal/widget/ViewPager.java)

- [Android 15 calendar dimension resources](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/values/dimens_material.xml)
- [Android 15 month layout](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/date_picker_month_item_material.xml)
- [Android 15 day picker layout](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/day_picker_content_material.xml)
