# Android XML TimePicker clock mode

Status: Initial radial clock and keyboard modes; Android device parity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The importer accepts explicit timePickerMode="clock". The dial supports 12-hour and 24-hour selection, with two hour rings in 24-hour mode, localized digits and AM/PM labels. Selecting an hour advances to the minute dial on release. Minute selection uses Android's weighted snapping toward the visible five-minute marks. The host supplies local time, 12/24-hour preference, hour cycle and padding. The header supports 0–11, 1–12, 0–23 and 1–24 formats; the 24-hour inner midnight mark is shown with two digits.

The initial presentation uses framework dimension references but includes a keyboard toggle with localized hour and minute fields and validation. Direct header editing, complete Android radial animation fidelity, all theme attributes or exact device styling.

## State and interaction contract

The keyboard/clock toggle uses the pinned Android framework vector icons. The user can tap or drag around the dial, return to hours or minutes through the header, select AM/PM in 12-hour mode, or switch to typed hour and minute fields. Invalid values are marked without replacing the selected time. Hour and minute changes report the TimePicker ID; selecting an unchanged hour still advances to minutes. Switching 12/24-hour mode preserves the selected time.

## Motion and animation

The dial follows the pointer immediately. Hour/minute changes crossfade labels and interpolate the selector over 500 ms with an accelerate-decelerate curve, following the framework radial picker. Interrupted transitions resume from their current position. Touch-specific timing and exact clipping remain open.

**Reduced motion:** Dial transitions snap.

## Acceptance criteria

- Exercise dial dragging, 12/24-hour rings, minute snapping and AM/PM on a phone.
- Compare keyboard input, radial transitions, appearance and behavior with Android.

## References

- [Android 15 TimePicker clock delegate](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TimePickerClockDelegate.java)
- [Android 15 RadialTimePickerView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/RadialTimePickerView.java)
- [Android 15 dimensions](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/values/dimens_material.xml)

- [Android 15 text input picker](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TextInputTimePickerView.java)
- [Android 15 TimePicker layout](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/time_picker_material.xml)

## Material 3 theme chrome (Wave 15A)

Under `AndroidTheme::Material3Light` and `AndroidTheme::Material3Dark` in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `Kind::TimePicker` replaces the legacy Material 2 solid primary header with a `28dp`-rounded `surfaceContainerHigh` container, `primaryContainer` active hour/minute display boxes, and a `tertiaryContainer` AM/PM segmented toggle. Verified by `wave15a_m3_date_picker_and_time_picker_chrome` in [`tests/android_xml.rs`](../../tests/android_xml.rs). Full M3 radial dial token polish and Android device visual parity remain open.
