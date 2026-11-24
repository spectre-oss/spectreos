# Android XML DigitalClock

Status: Initial support for the deprecated Android clock widget.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P2 |

## Scope and variants

DigitalClock displays the host's local time using its preferred 12-hour or 24-hour cycle. Android deprecated it in favor of TextClock.

## State and interaction contract

The host provides wall-clock time, UTC offset and hour-cycle preference. This initial implementation uses generic numeric formats; locale-specific time patterns and digits are not yet supported.

## Motion and animation

Time updates on local minute boundaries without continuous redraw.

**Reduced motion:** Time updates remain available.

## Acceptance criteria

- Import the DigitalClock XML tag and display the host time.
- Compare locale formatting and lifecycle behavior with Android.

## References

- [Android DigitalClock](https://developer.android.com/reference/android/widget/DigitalClock)
- Android 15 `DigitalClock.java` in `context/reference-sources/android-framework/`, Apache-2.0.
