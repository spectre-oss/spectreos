# Android XML TimePicker spinner mode

Status: Initial explicit spinner mode; clock mode and Android device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The importer accepts TimePicker with explicit timePickerMode="spinner". The widget uses NumberPicker wheels for hour, minute and, in 12-hour mode, AM/PM. The host supplies local time, 12/24-hour preference, localized digits, period labels, hour cycle, hour padding and separator. The wheel supports 0–11, 1–12, 0–23 and 1–24 hour cycles. Minute rollover changes the hour, and the hour wheel changes AM/PM across eleven and twelve. Changing 12/24-hour mode preserves the time.

Clock mode is tracked separately. Exact locale-specific formatting, theme details and Android device comparison remain open.

## State and interaction contract

Taps, drags, flings, holds and text edits update the time and report the TimePicker ID. Minutes and hours wrap; 12-hour period selection changes the half-day. Applications can set the mode on the caller-owned state.

## Motion and animation

Wheels share the NumberPicker touch fling and settle implementation, which remains to be compared with Android.

**Reduced motion:** Wheel settlement snaps.

## Acceptance criteria

- Verify 12/24-hour mode changes preserve time and minute rollover carries across noon and midnight.
- Exercise each wheel and numeric editing on a phone.
- Compare hour cycles, formatting and behavior with Android.

## References

- [Android 15 TimePicker](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TimePicker.java)
- [Android 15 spinner delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TimePickerSpinnerDelegate.java)
- [Android 15 clock delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TimePickerClockDelegate.java)
