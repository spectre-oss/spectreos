# Android XML TextClock

Status: Initial time-only XML clock support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

TextClock displays host-supplied local wall-clock date and time with numeric 12-hour and 24-hour patterns.

## State and interaction contract

The host supplies Unix time, UTC offset and the preferred hour cycle. Supported pattern symbols are y, M, d, H, h, K, k, m, s and a, with quoted literals. Localized digits and month or weekday names, explicit time zones and full Android pattern support remain incomplete.

## Motion and animation

The clock schedules updates on local second or minute boundaries without continuous redraw; seconds inside quoted literals do not trigger second-by-second updates.

**Reduced motion:** Clock updates remain available.

## Acceptance criteria

- Format host time in 12-hour and 24-hour patterns.
- Schedule updates at the appropriate time boundary.
- Compare locale and time-zone behavior with Android.

## References

- [Android TextClock reference](https://developer.android.com/reference/android/widget/TextClock)
