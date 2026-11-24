# Android XML Chronometer

Status: Initial XML timer support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android Chronometer displays elapsed time or a countdown relative to a monotonic base.

## State and interaction contract

The host owns the base time and running state. XML supports `countDown` and formats with `%s`, indexed `%1$s`, literal `%%` and `%n`. Full Java formatting, locale-specific digits, tick listeners and Android lifecycle parity remain incomplete.

## Motion and animation

The timer rounds elapsed time and schedules ticks relative to its base, following Android; it updates while running and visible without continuous redraw.

**Reduced motion:** Timer updates remain available.

## Acceptance criteria

- Format count-up and countdown values and stop updates when stopped.
- Schedule the next tick while running and visible.
- Compare formatting and lifecycle behavior with Android.

## References

- [Android Chronometer reference](https://developer.android.com/reference/android/widget/Chronometer)
- Pinned Android 15 Chronometer.java in `context/reference-sources/android-framework/Chronometer.java` (Apache-2.0).
